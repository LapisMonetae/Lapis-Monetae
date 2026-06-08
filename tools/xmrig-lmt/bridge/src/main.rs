use std::{net::SocketAddr, sync::Arc, time::Duration};

use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use lmt_consensus_core::{hashing, header::Header};
use lmt_grpc_client::GrpcClient;
use lmt_math::Uint256;
use lmt_rpc_core::{
    api::rpc::RpcApi,
    model::{
        address::RpcAddress,
        message::{GetBlockTemplateRequest, RpcExtraData, SubmitBlockRequest},
        RpcRawBlock, RpcRawHeader,
    },
};
use lmt_utils::hex::{FromHex, ToHex};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::{net::TcpListener, sync::RwLock};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

#[derive(Parser, Debug)]
#[command(author, version, about = "LMT Stratum bridge for RandomX mining")]
struct Args {
    /// Stratum listen address (host:port)
    #[arg(long, default_value = "0.0.0.0:3333")]
    listen: String,
    /// gRPC RPC URL, e.g. grpc://127.0.0.1:26110
    #[arg(long, default_value = "grpc://127.0.0.1:26110")]
    rpc_url: String,
    /// LMT pay address for coinbase rewards
    #[arg(long)]
    pay_address: String,
    /// Extra data in hex (optional)
    #[arg(long, default_value = "")]
    extra_data_hex: String,
    /// Allow non-DAA blocks when submitting (useful for tests)
    #[arg(long, default_value_t = true)]
    allow_non_daa: bool,
    /// Template refresh interval in ms
    #[arg(long, default_value_t = 5000)]
    refresh_ms: u64,
    /// Max submit requests per second per client (rate limiting)
    #[arg(long, default_value_t = 50)]
    max_submits_per_sec: u32,
}

// ── Data types ──

#[derive(Debug, Clone)]
struct Template {
    job_id: u64,
    block: RpcRawBlock,
    pre_pow_hash_hex: String,
    timestamp: u64,
    bits_hex: String,
    target_hex: String,
}

#[derive(Debug, Deserialize)]
struct StratumRequest {
    id: Option<u64>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct StratumResponse<'a> {
    id: u64,
    result: &'a Value,
    error: Option<Value>,
}

#[derive(Debug, Serialize)]
struct StratumNotification<'a> {
    method: &'a str,
    params: Value,
}

#[derive(Debug)]
struct SubmitParams {
    job_id: u64,
    nonce: u64,
    timestamp: Option<u64>,
}

// ── Metrics ──

struct Metrics {
    blocks_accepted: AtomicU64,
    blocks_rejected: AtomicU64,
    stale_shares: AtomicU64,
    clients_total: AtomicU64,
    clients_active: AtomicU64,
}

impl Metrics {
    fn new() -> Self {
        Self {
            blocks_accepted: AtomicU64::new(0),
            blocks_rejected: AtomicU64::new(0),
            stale_shares: AtomicU64::new(0),
            clients_total: AtomicU64::new(0),
            clients_active: AtomicU64::new(0),
        }
    }

    fn log_summary(&self) {
        info!(
            "Metrics: accepted={}, rejected={}, stale={}, clients={}/{}",
            self.blocks_accepted.load(Ordering::Relaxed),
            self.blocks_rejected.load(Ordering::Relaxed),
            self.stale_shares.load(Ordering::Relaxed),
            self.clients_active.load(Ordering::Relaxed),
            self.clients_total.load(Ordering::Relaxed),
        );
    }
}

// ── Main ──

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    let pay_address = RpcAddress::try_from(args.pay_address.as_str())?;
    let extra_data: RpcExtraData =
        if args.extra_data_hex.is_empty() { Vec::new() } else { Vec::<u8>::from_hex(args.extra_data_hex.as_str())? };

    info!("Connecting to gRPC at {}", args.rpc_url);
    let client = GrpcClient::connect(args.rpc_url.clone()).await?;
    client.start(None).await;

    // Verify connectivity
    tokio::time::timeout(
        Duration::from_secs(10),
        client.get_block_template_call(None, GetBlockTemplateRequest::new(pay_address.clone(), extra_data.clone())),
    )
    .await
    .map_err(|_| "initial gRPC connection timed out after 10s")?
    .map_err(|e| format!("initial gRPC call failed: {e}"))?;
    info!("gRPC connection verified");

    let listener = TcpListener::bind(&args.listen).await?;
    info!("LMT Stratum bridge listening on {}", args.listen);
    info!("Pay address: {}", args.pay_address);
    info!("Template refresh: {}ms", args.refresh_ms);

    let client = Arc::new(client);
    let extra_data = Arc::new(extra_data);
    let pay_address = Arc::new(pay_address);
    let refresh = Duration::from_millis(args.refresh_ms);
    let allow_non_daa = args.allow_non_daa;
    let max_submits = args.max_submits_per_sec;
    let metrics = Arc::new(Metrics::new());

    // Shared template — RwLock for better concurrency (many readers, one writer)
    let shared_template: Arc<RwLock<Option<Template>>> = Arc::new(RwLock::new(None));
    let job_counter = Arc::new(AtomicU64::new(0));

    // Background template refresh task
    let tmpl_client = client.clone();
    let tmpl_addr = pay_address.clone();
    let tmpl_extra = extra_data.clone();
    let tmpl_state = shared_template.clone();
    let tmpl_counter = job_counter.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(refresh);
        interval.tick().await;
        loop {
            interval.tick().await;
            let jc = tmpl_counter.fetch_add(1, Ordering::Relaxed) + 1;
            match fetch_template(&tmpl_client, &tmpl_addr, &tmpl_extra, jc).await {
                Ok(t) => {
                    debug!("New template job_id={} daa={}", t.job_id, t.block.header.daa_score);
                    *tmpl_state.write().await = Some(t);
                }
                Err(e) => warn!("Template refresh failed: {e}"),
            }
        }
    });

    // Periodic metrics logging
    let metrics_ref = metrics.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            metrics_ref.log_summary();
        }
    });

    // Accept clients
    loop {
        let (stream, addr) = listener.accept().await?;
        let client = client.clone();
        let pay_address = pay_address.clone();
        let extra_data = extra_data.clone();
        let template = shared_template.clone();
        let jc = job_counter.clone();
        let m = metrics.clone();
        m.clients_total.fetch_add(1, Ordering::Relaxed);
        m.clients_active.fetch_add(1, Ordering::Relaxed);
        info!("Client connected: {addr}");

        tokio::spawn(async move {
            if let Err(err) =
                handle_client(stream, addr, client, pay_address, extra_data, allow_non_daa, template, jc, m.clone(), max_submits).await
            {
                warn!("Client {addr} error: {err}");
            }
            m.clients_active.fetch_sub(1, Ordering::Relaxed);
            info!("Client disconnected: {addr}");
        });
    }
}

// ── Client handler ──

async fn handle_client(
    stream: tokio::net::TcpStream,
    addr: SocketAddr,
    client: Arc<GrpcClient>,
    pay_address: Arc<RpcAddress>,
    extra_data: Arc<RpcExtraData>,
    allow_non_daa: bool,
    shared_template: Arc<RwLock<Option<Template>>>,
    job_counter: Arc<AtomicU64>,
    metrics: Arc<Metrics>,
    max_submits_per_sec: u32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (reader, writer) = stream.into_split();
    let mut lines = FramedRead::new(reader, LinesCodec::new());
    let mut sink = FramedWrite::new(writer, LinesCodec::new());

    // Rate limiting state
    let mut submit_count: u32 = 0;
    let mut rate_limit_reset = tokio::time::Instant::now() + Duration::from_secs(1);

    // Track last sent job_id to detect stale
    let mut last_sent_job: u64 = 0;

    // Notify timer — re-send template if shared one changed
    let mut notify_check = tokio::time::interval(Duration::from_millis(500));
    notify_check.tick().await;

    loop {
        tokio::select! {
            maybe_line = lines.next() => {
                let Some(line) = maybe_line else { break; };
                let line = line?;
                let req: StratumRequest = match serde_json::from_str(&line) {
                    Ok(req) => req,
                    Err(_) => continue,
                };
                match req.method.as_str() {
                    "mining.subscribe" => {
                        let result = serde_json::json!({ "protocol": "lmt-stratum/1.0" });
                        send_response(&mut sink, req.id, result).await?;
                        // Send initial job
                        if let Some(t) = shared_template.read().await.as_ref() {
                            send_notify(&mut sink, t).await?;
                            last_sent_job = t.job_id;
                        } else {
                            // No template yet — fetch one
                            let jc = job_counter.fetch_add(1, Ordering::Relaxed) + 1;
                            let t = fetch_template_with_retry(&client, &pay_address, &extra_data, jc, 3).await?;
                            *shared_template.write().await = Some(t.clone());
                            send_notify(&mut sink, &t).await?;
                            last_sent_job = t.job_id;
                        }
                    }
                    "mining.authorize" => {
                        let result = serde_json::json!(true);
                        send_response(&mut sink, req.id, result).await?;
                        debug!("Client {addr} authorized");
                    }
                    "mining.submit" => {
                        // Rate limiting
                        let now = tokio::time::Instant::now();
                        if now >= rate_limit_reset {
                            submit_count = 0;
                            rate_limit_reset = now + Duration::from_secs(1);
                        }
                        submit_count += 1;
                        if submit_count > max_submits_per_sec {
                            warn!("Client {addr} rate limited ({submit_count}/s)");
                            let result = serde_json::json!(false);
                            send_response(&mut sink, req.id, result).await?;
                            continue;
                        }

                        let params = req.params.unwrap_or(Value::Null);
                        let submit = match parse_submit(params) {
                            Ok(s) => s,
                            Err(e) => {
                                warn!("Client {addr} bad submit: {e}");
                                let result = serde_json::json!(false);
                                send_response(&mut sink, req.id, result).await?;
                                continue;
                            }
                        };

                        let template_guard = shared_template.read().await;
                        let Some(template) = template_guard.as_ref() else {
                            let result = serde_json::json!(false);
                            send_response(&mut sink, req.id, result).await?;
                            continue;
                        };

                        // Stale job detection
                        if submit.job_id != template.job_id {
                            metrics.stale_shares.fetch_add(1, Ordering::Relaxed);
                            debug!("Client {addr} stale share: job {} (current {})", submit.job_id, template.job_id);
                            let err = serde_json::json!({"code": 21, "message": "stale job"});
                            send_response_with_error(&mut sink, req.id, Value::Bool(false), err).await?;
                            continue;
                        }

                        let mut block = template.block.clone();
                        drop(template_guard); // Release read lock before slow gRPC call

                        block.header.nonce = submit.nonce;
                        if let Some(ts) = submit.timestamp {
                            block.header.timestamp = ts;
                        }

                        let submit_req = SubmitBlockRequest::new(block, allow_non_daa);
                        let submit_res = match tokio::time::timeout(
                            Duration::from_secs(15),
                            client.submit_block_call(None, submit_req),
                        ).await {
                            Ok(Ok(_)) => {
                                metrics.blocks_accepted.fetch_add(1, Ordering::Relaxed);
                                info!("BLOCK ACCEPTED from {addr} (job {})", submit.job_id);
                                true
                            }
                            Ok(Err(e)) => {
                                metrics.blocks_rejected.fetch_add(1, Ordering::Relaxed);
                                warn!("Block rejected from {addr}: {e}");
                                false
                            }
                            Err(_) => {
                                error!("submit_block timed out for {addr}");
                                false
                            }
                        };
                        let result = serde_json::json!(submit_res);
                        send_response(&mut sink, req.id, result).await?;
                    }
                    other => {
                        debug!("Client {addr} unknown method: {other}");
                        let result = serde_json::json!(null);
                        send_response(&mut sink, req.id, result).await?;
                    }
                }
            }
            _ = notify_check.tick() => {
                // Check if template changed and push new job
                if let Some(t) = shared_template.read().await.as_ref() {
                    if t.job_id != last_sent_job {
                        send_notify(&mut sink, t).await?;
                        last_sent_job = t.job_id;
                    }
                }
            }
        }
    }

    Ok(())
}

// ── Template fetching ──

async fn fetch_template_with_retry(
    client: &GrpcClient,
    pay_address: &RpcAddress,
    extra_data: &RpcExtraData,
    job_id: u64,
    max_retries: u32,
) -> Result<Template, Box<dyn std::error::Error + Send + Sync>> {
    let mut last_err: Option<Box<dyn std::error::Error + Send + Sync>> = None;
    for attempt in 0..=max_retries {
        if attempt > 0 {
            let backoff = Duration::from_millis(500 * 2u64.pow(attempt - 1));
            warn!("fetch_template retry {}/{} after {:?}", attempt, max_retries, backoff);
            tokio::time::sleep(backoff).await;
        }
        match fetch_template(client, pay_address, extra_data, job_id).await {
            Ok(t) => return Ok(t),
            Err(e) => {
                warn!("fetch_template attempt {} failed: {}", attempt + 1, e);
                last_err = Some(e);
            }
        }
    }
    Err(last_err.unwrap_or_else(|| "fetch_template failed after retries".into()))
}

async fn fetch_template(
    client: &GrpcClient,
    pay_address: &RpcAddress,
    extra_data: &RpcExtraData,
    job_id: u64,
) -> Result<Template, Box<dyn std::error::Error + Send + Sync>> {
    let response = tokio::time::timeout(
        Duration::from_secs(10),
        client.get_block_template_call(None, GetBlockTemplateRequest::new(pay_address.clone(), extra_data.clone())),
    )
    .await
    .map_err(|_| "get_block_template timed out after 10s")?
    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
    let block = response.block;
    let header = raw_header_to_header(&block.header);
    let pre_pow_hash = hashing::header::hash_override_nonce_time(&header, 0, 0);
    let bits_hex = format!("0x{:08x}", block.header.bits);
    let target = Uint256::from_compact_target_bits(block.header.bits);
    let target_le = target.to_le_bytes();
    let target_hex = target_le[24..32].to_vec().to_hex();
    Ok(Template {
        job_id,
        pre_pow_hash_hex: pre_pow_hash.as_bytes().to_vec().to_hex(),
        timestamp: block.header.timestamp,
        bits_hex,
        target_hex,
        block,
    })
}

fn raw_header_to_header(header: &RpcRawHeader) -> Header {
    Header::new_finalized(
        header.version,
        header.parents_by_level.clone(),
        header.hash_merkle_root,
        header.accepted_id_merkle_root,
        header.utxo_commitment,
        header.timestamp,
        header.bits,
        header.nonce,
        header.daa_score,
        header.blue_work,
        header.blue_score,
        header.pruning_point,
    )
}

// ── Stratum protocol helpers ──

async fn send_response(
    sink: &mut FramedWrite<tokio::net::tcp::OwnedWriteHalf, LinesCodec>,
    id: Option<u64>,
    result: Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(id) = id {
        let response = StratumResponse { id, result: &result, error: None };
        sink.send(serde_json::to_string(&response)?).await?;
    }
    Ok(())
}

async fn send_response_with_error(
    sink: &mut FramedWrite<tokio::net::tcp::OwnedWriteHalf, LinesCodec>,
    id: Option<u64>,
    result: Value,
    error: Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(id) = id {
        let response = StratumResponse { id, result: &result, error: Some(error) };
        sink.send(serde_json::to_string(&response)?).await?;
    }
    Ok(())
}

async fn send_notify(
    sink: &mut FramedWrite<tokio::net::tcp::OwnedWriteHalf, LinesCodec>,
    template: &Template,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let notification = StratumNotification {
        method: "mining.notify",
        params: serde_json::json!([
            template.job_id.to_string(),
            template.pre_pow_hash_hex,
            template.timestamp,
            template.bits_hex,
            template.target_hex
        ]),
    };
    sink.send(serde_json::to_string(&notification)?).await?;
    Ok(())
}

fn build_notify_params(template: &Template) -> Value {
    serde_json::json!([
        template.job_id.to_string(),
        template.pre_pow_hash_hex,
        template.timestamp,
        template.bits_hex,
        template.target_hex
    ])
}

fn parse_submit(params: Value) -> Result<SubmitParams, Box<dyn std::error::Error + Send + Sync>> {
    let params = params.as_array().cloned().unwrap_or_default();
    if params.len() < 3 {
        return Err("submit requires at least 3 params".into());
    }
    let job_id = params[1].as_str().ok_or("job_id missing")?.parse::<u64>()?;
    let nonce_str = params[2].as_str().ok_or("nonce missing")?;
    let nonce_str = nonce_str.trim_start_matches("0x");
    let nonce = u64::from_str_radix(nonce_str, 16)?;
    let timestamp = params.get(3).and_then(|v| v.as_u64()).filter(|ts| *ts > 0);
    Ok(SubmitParams { job_id, nonce, timestamp })
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use lmt_consensus_core::header::Header;

    #[test]
    fn parse_submit_accepts_hex_nonce_and_optional_timestamp() {
        let params = serde_json::json!(["worker", "42", "0x00000000000000ff", 123456u64]);
        let parsed = parse_submit(params).expect("submit params should parse");
        assert_eq!(parsed.job_id, 42);
        assert_eq!(parsed.nonce, 255);
        assert_eq!(parsed.timestamp, Some(123456));
    }

    #[test]
    fn parse_submit_rejects_short_or_invalid_params() {
        let short = serde_json::json!(["worker", "42"]);
        assert!(parse_submit(short).is_err());
        let invalid_nonce = serde_json::json!(["worker", "42", "not_hex"]);
        assert!(parse_submit(invalid_nonce).is_err());
    }

    #[test]
    fn mining_notify_payload_shape_matches_protocol() {
        let header = Header::new_finalized(
            0,
            vec![],
            Default::default(),
            Default::default(),
            Default::default(),
            0,
            0,
            0,
            0,
            0.into(),
            0,
            Default::default(),
        );
        let template = Template {
            job_id: 7,
            block: RpcRawBlock { header: (&header).into(), transactions: vec![] },
            pre_pow_hash_hex: "aa".repeat(32),
            timestamp: 123u64,
            bits_hex: "0x1d00ffff".to_string(),
            target_hex: "00ff00ff00ff00ff".to_string(),
        };
        let params = build_notify_params(&template);
        let arr = params.as_array().expect("should be array");
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0], Value::String("7".to_string()));
        assert_eq!(arr[1], Value::String("aa".repeat(32)));
        assert_eq!(arr[2], Value::from(123u64));
    }

    #[allow(dead_code)]
    async fn rpc_call_signature_smoke<T: RpcApi>(api: &T, addr: RpcAddress, extra: RpcExtraData, block: RpcRawBlock) {
        let _ = api.get_block_template_call(None, GetBlockTemplateRequest::new(addr, extra)).await;
        let _ = api.submit_block_call(None, SubmitBlockRequest::new(block, false)).await;
    }

    #[test]
    fn rpc_call_signature_smoke_compiles() {
        let _ = rpc_call_signature_smoke::<GrpcClient>;
    }
}
