use regex::Regex;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CliResult {
    pub exit_code: i32,
    pub output: String,
}

/// Strip ANSI escape codes from text
pub fn strip_ansi(text: &str) -> String {
    let re = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(text, "").to_string()
}

/// Run a CLI command and capture output with timeout
pub fn run_cli(cli_path: &str, args: &[&str], timeout_secs: u64) -> CliResult {
    let child = Command::new(cli_path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match child {
        Ok(child) => {
            let (tx, rx) = mpsc::channel();
            let handle = thread::spawn(move || {
                let output = child.wait_with_output();
                let _ = tx.send(output);
            });

            match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
                Ok(Ok(output)) => {
                    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
                    let stderr = strip_ansi(&String::from_utf8_lossy(&output.stderr));
                    let combined = if stderr.is_empty() {
                        stdout
                    } else {
                        format!("{stdout}\n{stderr}")
                    };
                    CliResult {
                        exit_code: output.status.code().unwrap_or(-1),
                        output: combined.trim().to_string(),
                    }
                }
                Ok(Err(e)) => CliResult {
                    exit_code: -1,
                    output: format!("Process error: {e}"),
                },
                Err(_) => {
                    drop(handle);
                    CliResult {
                        exit_code: -1,
                        output: "Command timed out".into(),
                    }
                }
            }
        }
        Err(e) => CliResult {
            exit_code: -1,
            output: format!("Failed to launch CLI: {e}"),
        },
    }
}

/// Run a CLI command with data piped to stdin and capture output with timeout
pub fn run_cli_with_stdin(cli_path: &str, args: &[&str], stdin_data: &str, timeout_secs: u64) -> CliResult {
    use std::io::Write;

    let child = Command::new(cli_path)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match child {
        Ok(mut child) => {
            if let Some(mut stdin) = child.stdin.take() {
                let data = stdin_data.to_string();
                let _ = stdin.write_all(data.as_bytes());
                drop(stdin);
            }

            let (tx, rx) = mpsc::channel();
            let handle = thread::spawn(move || {
                let output = child.wait_with_output();
                let _ = tx.send(output);
            });

            match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
                Ok(Ok(output)) => {
                    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
                    let stderr = strip_ansi(&String::from_utf8_lossy(&output.stderr));
                    let combined = if stderr.is_empty() {
                        stdout
                    } else {
                        format!("{stdout}\n{stderr}")
                    };
                    CliResult {
                        exit_code: output.status.code().unwrap_or(-1),
                        output: combined.trim().to_string(),
                    }
                }
                Ok(Err(e)) => CliResult {
                    exit_code: -1,
                    output: format!("Process error: {e}"),
                },
                Err(_) => {
                    drop(handle);
                    CliResult {
                        exit_code: -1,
                        output: "Command timed out".into(),
                    }
                }
            }
        }
        Err(e) => CliResult {
            exit_code: -1,
            output: format!("Failed to launch CLI: {e}"),
        },
    }
}

/// Verify CLI binary is valid
pub fn verify_cli(path: &str) -> bool {
    if !Path::new(path).exists() {
        return false;
    }
    let result = run_cli(path, &["--version"], 5);
    result.exit_code == 0
}

/// Parse node info from RPC response
pub fn parse_node_info(output: &str) -> NodeInfo {
    let mut info = NodeInfo::default();

    let daa_re = Regex::new(r"(?i)daa.?score[:\s]+(\d+)").unwrap();
    let tip_re = Regex::new(r"(?i)tip.?hash[es]*[:\s]+([a-f0-9]{64})").unwrap();
    let diff_re = Regex::new(r"(?i)difficulty[:\s]+([\d.eE+\-]+)").unwrap();
    let net_re = Regex::new(r"(?i)network.?name[:\s]+(\S+)").unwrap();
    let header_re = Regex::new(r"(?i)header.?count[:\s]+(\d+)").unwrap();
    let block_re = Regex::new(r"(?i)block.?count[:\s]+(\d+)").unwrap();

    for line in output.lines() {
        if let Some(cap) = daa_re.captures(line) {
            if let Ok(v) = cap[1].parse() {
                info.daa_score = v;
            }
        }
        if let Some(cap) = tip_re.captures(line) {
            info.tip_hash = cap[1].to_string();
        }
        if let Some(cap) = diff_re.captures(line) {
            if let Ok(v) = cap[1].parse() {
                info.difficulty = v;
            }
        }
        if let Some(cap) = net_re.captures(line) {
            info.network = cap[1].to_string();
        }
        if let Some(cap) = header_re.captures(line) {
            if let Ok(v) = cap[1].parse() {
                info.header_count = v;
            }
        }
        if let Some(cap) = block_re.captures(line) {
            if let Ok(v) = cap[1].parse() {
                info.block_count = v;
            }
        }
    }
    info
}

#[derive(Debug, Clone, Default)]
pub struct NodeInfo {
    pub daa_score: u64,
    pub tip_hash: String,
    pub difficulty: f64,
    pub network: String,
    pub header_count: u64,
    pub block_count: u64,
}

#[derive(Debug, Clone, Default)]
pub struct PeerInfo {
    pub count: u32,
}

pub fn parse_peer_info(output: &str) -> PeerInfo {
    let re = Regex::new(r"(?i)RpcPeerInfo").unwrap();
    let count = re.find_iter(output).count() as u32;
    PeerInfo { count }
}
