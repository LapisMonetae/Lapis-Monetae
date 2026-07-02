#![allow(dead_code)]

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use regex::Regex;

#[derive(Debug, Clone)]
pub struct ConsoleLine {
    pub timestamp: String,
    pub tag: String,
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct MinerMetrics {
    pub hashrate_10s: f64,
    pub hashrate_60s: f64,
    pub hashrate_15m: f64,
    pub shares_accepted: u64,
    pub shares_rejected: u64,
    pub pool_latency_ms: u64,
}

#[derive(Debug, Clone)]
pub enum ProcessEvent {
    Line(String, String), // (tag, text)
    Metrics(MinerMetrics),
    Exited(String, i32), // (tag, exit_code)
}

pub struct ProcessManager {
    bridge_child: Option<Child>,
    miner_child: Option<Child>,
    pub event_rx: mpsc::Receiver<ProcessEvent>,
    event_tx: mpsc::Sender<ProcessEvent>,
    pub bridge_running: bool,
    pub miner_running: bool,
    pub bridge_start_time: Option<Instant>,
    pub miner_start_time: Option<Instant>,
    // Auto-restart
    pub bridge_auto_restart: bool,
    pub miner_auto_restart: bool,
    bridge_restart_count: u32,
    miner_restart_count: u32,
    bridge_last_stable: Option<Instant>,
    miner_last_stable: Option<Instant>,
}

impl ProcessManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            bridge_child: None,
            miner_child: None,
            event_rx: rx,
            event_tx: tx,
            bridge_running: false,
            miner_running: false,
            bridge_start_time: None,
            miner_start_time: None,
            bridge_auto_restart: true,
            miner_auto_restart: false,
            bridge_restart_count: 0,
            miner_restart_count: 0,
            bridge_last_stable: None,
            miner_last_stable: None,
        }
    }

    pub fn start_bridge(&mut self, binary: &str, args: Vec<String>) -> Result<(), String> {
        if self.bridge_running {
            return Err("Bridge already running".into());
        }
        if !std::path::Path::new(binary).exists() {
            return Err(format!("Bridge binary not found: {binary}"));
        }

        let mut child = Command::new(binary)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start bridge: {e}"))?;

        let tx = self.event_tx.clone();
        // Pump stdout
        if let Some(stdout) = child.stdout.take() {
            let tx2 = tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = tx2.send(ProcessEvent::Line("bridge".into(), line));
                }
            });
        }
        // Pump stderr
        if let Some(stderr) = child.stderr.take() {
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = tx.send(ProcessEvent::Line("bridge".into(), line));
                }
            });
        }

        self.bridge_child = Some(child);
        self.bridge_running = true;
        self.bridge_start_time = Some(Instant::now());
        self.bridge_last_stable = Some(Instant::now());
        Ok(())
    }

    pub fn stop_bridge(&mut self) {
        if let Some(mut child) = self.bridge_child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.bridge_running = false;
        self.bridge_start_time = None;
    }

    pub fn start_miner(&mut self, binary: &str, args: Vec<String>) -> Result<(), String> {
        if self.miner_running {
            return Err("Miner already running".into());
        }
        if !std::path::Path::new(binary).exists() {
            return Err(format!("Miner binary not found: {binary}"));
        }

        let mut child = Command::new(binary)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start miner: {e}"))?;

        let tx = self.event_tx.clone();
        if let Some(stdout) = child.stdout.take() {
            let tx2 = tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                let hr_re = Regex::new(r"speed\s+[\d.]+\s+H/s\s+([\d.]+)\s+H/s\s+([\d.]+)\s+H/s\s+([\d.]+)").ok();
                let hr_simple = Regex::new(r"([\d.]+)\s*H/s").ok();
                let accepted_re = Regex::new(r"accepted\s*\((\d+)").ok();
                let rejected_re = Regex::new(r"rejected\s*\((\d+)").ok();

                let mut metrics = MinerMetrics::default();

                for line in reader.lines().map_while(Result::ok) {
                    // Parse metrics from xmrig output
                    if let Some(ref re) = hr_re {
                        if let Some(caps) = re.captures(&line) {
                            metrics.hashrate_10s = caps[1].parse().unwrap_or(0.0);
                            metrics.hashrate_60s = caps[2].parse().unwrap_or(0.0);
                            metrics.hashrate_15m = caps[3].parse().unwrap_or(0.0);
                            let _ = tx2.send(ProcessEvent::Metrics(metrics.clone()));
                        }
                    }
                    if let Some(ref re) = hr_simple {
                        if let Some(caps) = re.captures(&line) {
                            let hr: f64 = caps[1].parse().unwrap_or(0.0);
                            if hr > 0.0 {
                                metrics.hashrate_10s = hr;
                                let _ = tx2.send(ProcessEvent::Metrics(metrics.clone()));
                            }
                        }
                    }
                    if let Some(ref re) = accepted_re {
                        if let Some(caps) = re.captures(&line) {
                            metrics.shares_accepted = caps[1].parse().unwrap_or(0);
                        }
                    }
                    if let Some(ref re) = rejected_re {
                        if let Some(caps) = re.captures(&line) {
                            metrics.shares_rejected = caps[1].parse().unwrap_or(0);
                        }
                    }

                    let _ = tx2.send(ProcessEvent::Line("miner".into(), line));
                }
                let _ = tx2.send(ProcessEvent::Exited("miner".into(), 0));
            });
        }
        if let Some(stderr) = child.stderr.take() {
            let tx2 = tx;
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = tx2.send(ProcessEvent::Line("miner".into(), line));
                }
            });
        }

        self.miner_child = Some(child);
        self.miner_running = true;
        self.miner_start_time = Some(Instant::now());
        self.miner_last_stable = Some(Instant::now());
        Ok(())
    }

    pub fn stop_miner(&mut self) {
        if let Some(mut child) = self.miner_child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.miner_running = false;
        self.miner_start_time = None;
    }

    pub fn check_processes(&mut self) {
        // Check bridge
        if self.bridge_running {
            if let Some(ref mut child) = self.bridge_child {
                if let Ok(Some(status)) = child.try_wait() {
                    self.bridge_running = false;
                    let code = status.code().unwrap_or(-1);
                    let _ = self.event_tx.send(ProcessEvent::Exited("bridge".into(), code));
                    self.bridge_child = None;
                }
            }
        }
        // Check miner
        if self.miner_running {
            if let Some(ref mut child) = self.miner_child {
                if let Ok(Some(status)) = child.try_wait() {
                    self.miner_running = false;
                    let code = status.code().unwrap_or(-1);
                    let _ = self.event_tx.send(ProcessEvent::Exited("miner".into(), code));
                    self.miner_child = None;
                }
            }
        }
    }

    pub fn bridge_uptime(&self) -> String {
        self.bridge_start_time.map(|t| format_duration(t.elapsed())).unwrap_or_else(|| "—".into())
    }

    pub fn miner_uptime(&self) -> String {
        self.miner_start_time.map(|t| format_duration(t.elapsed())).unwrap_or_else(|| "—".into())
    }

    fn restart_delay(count: u32) -> Duration {
        let secs = (1u64 << count.min(5)).min(60);
        Duration::from_secs(secs)
    }

    pub fn should_restart_bridge(&mut self) -> bool {
        if !self.bridge_auto_restart || self.bridge_running || self.bridge_restart_count >= 5 {
            return false;
        }
        // Reset counter if stable for 60s
        if let Some(stable) = self.bridge_last_stable {
            if stable.elapsed().as_secs() > 60 {
                self.bridge_restart_count = 0;
            }
        }
        self.bridge_restart_count += 1;
        true
    }

    pub fn should_restart_miner(&mut self) -> bool {
        if !self.miner_auto_restart || self.miner_running || self.miner_restart_count >= 5 {
            return false;
        }
        if let Some(stable) = self.miner_last_stable {
            if stable.elapsed().as_secs() > 60 {
                self.miner_restart_count = 0;
            }
        }
        self.miner_restart_count += 1;
        true
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        self.stop_bridge();
        self.stop_miner();
    }
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

pub fn test_tcp(addr: &str, timeout_ms: u64) -> bool {
    use std::net::TcpStream;
    TcpStream::connect_timeout(&addr.parse().unwrap_or_else(|_| "127.0.0.1:1".parse().unwrap()), Duration::from_millis(timeout_ms))
        .is_ok()
}
