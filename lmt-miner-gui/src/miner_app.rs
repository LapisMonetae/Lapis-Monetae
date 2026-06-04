use eframe::egui;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

use crate::process_mgr::*;
use crate::theme::*;

fn load_logo(ctx: &egui::Context) -> egui::TextureHandle {
    let png_data = include_bytes!("../assets/lmt_logo.png");
    let img = image::load_from_memory(png_data).unwrap().into_rgba8();
    let size = [img.width() as _, img.height() as _];
    let pixels = img.into_raw();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
    ctx.load_texture("lmt_logo", color_image, egui::TextureOptions::LINEAR)
}

// ── Config persistence ──────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerConfig {
    pub bridge_binary: String,
    pub listen_address: String,
    pub grpc_url: String,
    pub pay_address: String,
    pub extra_data_hex: String,
    pub refresh_ms: String,
    pub allow_non_daa: bool,
    pub bridge_auto_restart: bool,
    pub miner_binary: String,
    pub stratum_url: String,
    pub miner_extra_args: String,
    pub miner_auto_start: bool,
    pub miner_auto_restart: bool,
}

impl Default for MinerConfig {
    fn default() -> Self {
        Self {
            bridge_binary: String::new(),
            listen_address: "0.0.0.0:3333".into(),
            grpc_url: "grpc://127.0.0.1:26110".into(),
            pay_address: String::new(),
            extra_data_hex: String::new(),
            refresh_ms: "5000".into(),
            allow_non_daa: true,
            bridge_auto_restart: true,
            miner_binary: String::new(),
            stratum_url: "stratum+tcp://127.0.0.1:3333".into(),
            miner_extra_args: String::new(),
            miner_auto_start: false,
            miner_auto_restart: false,
        }
    }
}

impl MinerConfig {
    fn config_dir() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_default());
        base.join("lapis-monetae")
    }

    fn config_path() -> PathBuf {
        Self::config_dir().join("miner-gui-config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, data);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Tab {
    Bridge,
    Miner,
    Console,
    Metrics,
    Help,
}

#[derive(Debug, Clone)]
struct Toast {
    message: String,
    color: egui::Color32,
    created: Instant,
}

pub struct MinerApp {
    tab: Tab,
    logo: Option<egui::TextureHandle>,

    // Bridge config
    bridge_binary: String,
    listen_address: String,
    grpc_url: String,
    pay_address: String,
    extra_data_hex: String,
    refresh_ms: String,
    allow_non_daa: bool,
    bridge_auto_restart: bool,

    // Miner config
    miner_binary: String,
    stratum_url: String,
    miner_extra_args: String,
    miner_auto_start: bool,
    miner_auto_restart: bool,

    // Process manager
    proc_mgr: ProcessManager,

    // Console
    console_lines: Vec<ConsoleLine>,

    // Metrics
    metrics: MinerMetrics,

    // Toasts
    toasts: Vec<Toast>,

    // Error display
    bridge_error: String,
    miner_error: String,

    // About dialog
    show_about: bool,
}

impl MinerApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        let cfg = MinerConfig::load();
        Self {
            tab: Tab::Bridge,
            bridge_binary: cfg.bridge_binary,
            listen_address: cfg.listen_address,
            grpc_url: cfg.grpc_url,
            pay_address: cfg.pay_address,
            extra_data_hex: cfg.extra_data_hex,
            refresh_ms: cfg.refresh_ms,
            allow_non_daa: cfg.allow_non_daa,
            bridge_auto_restart: cfg.bridge_auto_restart,
            miner_binary: cfg.miner_binary,
            stratum_url: cfg.stratum_url,
            miner_extra_args: cfg.miner_extra_args,
            miner_auto_start: cfg.miner_auto_start,
            miner_auto_restart: cfg.miner_auto_restart,
            proc_mgr: ProcessManager::new(),
            console_lines: Vec::new(),
            metrics: MinerMetrics::default(),
            toasts: Vec::new(),
            bridge_error: String::new(),
            miner_error: String::new(),
            show_about: false,
            logo: Some(load_logo(&_cc.egui_ctx)),
        }
    }

    fn current_config(&self) -> MinerConfig {
        MinerConfig {
            bridge_binary: self.bridge_binary.clone(),
            listen_address: self.listen_address.clone(),
            grpc_url: self.grpc_url.clone(),
            pay_address: self.pay_address.clone(),
            extra_data_hex: self.extra_data_hex.clone(),
            refresh_ms: self.refresh_ms.clone(),
            allow_non_daa: self.allow_non_daa,
            bridge_auto_restart: self.bridge_auto_restart,
            miner_binary: self.miner_binary.clone(),
            stratum_url: self.stratum_url.clone(),
            miner_extra_args: self.miner_extra_args.clone(),
            miner_auto_start: self.miner_auto_start,
            miner_auto_restart: self.miner_auto_restart,
        }
    }

    fn save_config(&self) {
        self.current_config().save();
    }

    fn toast(&mut self, msg: &str, color: egui::Color32) {
        self.toasts.push(Toast {
            message: msg.to_string(),
            color,
            created: Instant::now(),
        });
    }

    fn log(&mut self, tag: &str, text: &str) {
        let ts = chrono::Local::now().format("%H:%M:%S").to_string();
        self.console_lines.push(ConsoleLine {
            timestamp: ts,
            tag: tag.to_string(),
            text: text.to_string(),
        });
        if self.console_lines.len() > 2000 {
            self.console_lines.drain(..500);
        }
    }

    fn build_bridge_args(&self) -> Vec<String> {
        let mut args = vec![
            "--listen".into(),
            self.listen_address.clone(),
            "--rpc-url".into(),
            self.grpc_url.clone(),
            "--pay-address".into(),
            self.pay_address.clone(),
            "--refresh-ms".into(),
            self.refresh_ms.clone(),
        ];
        if !self.extra_data_hex.is_empty() {
            args.push("--extra-data".into());
            args.push(self.extra_data_hex.clone());
        }
        if self.allow_non_daa {
            args.push("--allow-non-daa".into());
        }
        args
    }

    fn build_miner_args(&self) -> Vec<String> {
        let mut args = vec![
            "-o".into(),
            self.stratum_url.clone(),
            "-u".into(),
            self.pay_address.clone(),
            "-p".into(),
            "x".into(),
            "--no-color".into(),
        ];
        if !self.miner_extra_args.is_empty() {
            for a in self.miner_extra_args.split_whitespace() {
                args.push(a.to_string());
            }
        }
        args
    }

    fn poll_events(&mut self) {
        self.proc_mgr.check_processes();

        while let Ok(event) = self.proc_mgr.event_rx.try_recv() {
            match event {
                ProcessEvent::Line(tag, text) => {
                    self.log(&tag, &text);
                }
                ProcessEvent::Metrics(m) => {
                    self.metrics = m;
                }
                ProcessEvent::Exited(tag, code) => {
                    self.log("system", &format!("{tag} exited with code {code}"));
                    if tag == "bridge" {
                        self.proc_mgr.bridge_running = false;
                        self.toast(&format!("Bridge stopped (code {code})"), RED);
                    }
                    if tag == "miner" {
                        self.proc_mgr.miner_running = false;
                        self.toast(&format!("Miner stopped (code {code})"), RED);
                    }
                }
            }
        }

        // Auto-restart logic
        if !self.proc_mgr.bridge_running && self.bridge_auto_restart {
            if self.proc_mgr.should_restart_bridge() {
                self.log("system", "Auto-restarting bridge...");
                let args = self.build_bridge_args();
                if let Err(e) = self.proc_mgr.start_bridge(&self.bridge_binary, args) {
                    self.log("error", &format!("Bridge restart failed: {e}"));
                }
            }
        }
        if !self.proc_mgr.miner_running && self.miner_auto_restart {
            if self.proc_mgr.should_restart_miner() {
                self.log("system", "Auto-restarting miner...");
                let args = self.build_miner_args();
                if let Err(e) = self.proc_mgr.start_miner(&self.miner_binary, args) {
                    self.log("error", &format!("Miner restart failed: {e}"));
                }
            }
        }
    }
}

impl eframe::App for MinerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        setup_theme(ctx);
        self.poll_events();
        self.toasts.retain(|t| t.created.elapsed().as_secs_f32() < 3.0);
        ctx.request_repaint_after(std::time::Duration::from_millis(500));

        // ── Top header bar ───────────────────────────────────────────
        egui::TopBottomPanel::top("top_bar")
            .frame(
                egui::Frame::new()
                    .fill(BG_WHITE)
                    .inner_margin(egui::Margin::symmetric(16, 10))
                    .stroke(egui::Stroke::new(1.0, BORDER)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(ref logo) = self.logo {
                        ui.image(egui::load::SizedTexture::new(logo.id(), egui::Vec2::new(28.0, 28.0)));
                    } else {
                        icon(ui, Icon::Mining, 28.0, BLUE);
                    }
                    ui.label(
                        egui::RichText::new("  LMT Miner Control Center")
                            .font(egui::FontId::proportional(17.0))
                            .color(TEXT_PRIMARY)
                            .strong(),
                    );
                    ui.add_space(12.0);

                    if self.proc_mgr.bridge_running {
                        pill(ui, "  BRIDGE: RUNNING  ", GREEN_BG, GREEN);
                    } else {
                        pill(ui, "  BRIDGE: STOPPED  ", BG_INPUT, TEXT_MUTED);
                    }
                    ui.add_space(4.0);
                    if self.proc_mgr.miner_running {
                        pill(ui, "  MINER: RUNNING  ", GREEN_BG, GREEN);
                    } else {
                        pill(ui, "  MINER: STOPPED  ", BG_INPUT, TEXT_MUTED);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if btn_secondary(ui, "About").clicked() {
                            self.show_about = !self.show_about;
                        }
                        if self.proc_mgr.miner_running && self.metrics.hashrate_10s > 0.0 {
                            ui.label(big_number(
                                &format!("{:.1} H/s", self.metrics.hashrate_10s),
                                GREEN,
                            ));
                        }
                    });
                });
            });

        // ── Gradient bar under header ────────────────────────────────
        egui::TopBottomPanel::top("gradient")
            .frame(egui::Frame::NONE)
            .exact_height(3.0)
            .show(ctx, |ui| {
                gradient_bar(ui);
            });

        // ── Tab bar ──────────────────────────────────────────────────
        egui::TopBottomPanel::top("tab_bar")
            .frame(
                egui::Frame::new()
                    .fill(BG_WHITE)
                    .inner_margin(egui::Margin::symmetric(8, 0))
                    .stroke(egui::Stroke::new(1.0, BORDER)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for (t, name, ico) in [
                        (Tab::Bridge, "Bridge", Icon::Bridge),
                        (Tab::Miner, "Miner", Icon::Mining),
                        (Tab::Console, "Console", Icon::Terminal),
                        (Tab::Metrics, "Metrics", Icon::Chart),
                        (Tab::Help, "Help", Icon::Help),
                    ] {
                        let selected = self.tab == t;
                        let color = if selected { BLUE } else { TEXT_MUTED };
                        ui.add_space(2.0);
                        let resp = ui
                            .horizontal(|ui| {
                                icon(ui, ico, 16.0, color);
                                let text = egui::RichText::new(name)
                                    .font(egui::FontId::proportional(13.0))
                                    .color(color)
                                    .strong();
                                ui.selectable_label(selected, text)
                            })
                            .inner;
                        if resp.clicked() {
                            self.tab = t;
                        }
                    }
                });
            });

        // ── Content with tab animation ───────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            let tab_id = match self.tab {
                Tab::Bridge => "tab_bridge",
                Tab::Miner => "tab_miner",
                Tab::Console => "tab_console",
                Tab::Metrics => "tab_metrics",
                Tab::Help => "tab_help",
            };
            let opacity = animated_opacity(ctx, tab_id, true);
            ui.set_opacity(opacity);

            match self.tab {
                Tab::Bridge => self.show_bridge(ui),
                Tab::Miner => self.show_miner(ui),
                Tab::Console => self.show_console(ui),
                Tab::Metrics => self.show_metrics(ui),
                Tab::Help => self.show_help(ui),
            }
        });

        // ── About dialog ─────────────────────────────────────────────
        if self.show_about {
            egui::Window::new("About")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .frame(
                    egui::Frame::new()
                        .fill(BG_CARD)
                        .stroke(egui::Stroke::new(1.0, BORDER))
                        .corner_radius(egui::CornerRadius::same(12))
                        .inner_margin(egui::Margin::same(24))
                        .shadow(egui::epaint::Shadow {
                            offset: [0, 4],
                            blur: 16,
                            spread: 0,
                            color: egui::Color32::from_black_alpha(30),
                        }),
                )
                .show(ctx, |ui| {
                    card(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            icon(ui, Icon::Mining, 48.0, BLUE);
                            ui.add_space(8.0);
                            ui.label(heading("LMT Miner Control Center"));
                            ui.add_space(4.0);
                            ui.label(body_text("Version: 1.0.1"));
                            ui.add_space(4.0);
                            ui.label(body_text("Lapis Monetae Project"));
                            ui.add_space(16.0);
                            if btn_secondary(ui, "Close").clicked() {
                                self.show_about = false;
                            }
                        });
                    });
                });
        }

        // ── Toasts ───────────────────────────────────────────────────
        if !self.toasts.is_empty() {
            egui::Area::new(egui::Id::new("toasts"))
                .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-20.0, 60.0))
                .show(ctx, |ui| {
                    for t in &self.toasts {
                        let alpha =
                            1.0 - (t.created.elapsed().as_secs_f32() / 3.0).min(1.0);
                        toast(ui, &t.message, t.color, alpha);
                        ui.add_space(4.0);
                    }
                });
        }
    }
}

// ── Tab implementations ──────────────────────────────────────────────────
impl MinerApp {
    // ── Bridge Tab ───────────────────────────────────────────────────
    fn show_bridge(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);

        card(ui, |ui| {
            section(ui, Icon::Bridge, "Stratum Bridge Configuration", BLUE);
            divider(ui);
            ui.add_space(4.0);

            egui::Grid::new("bridge_config")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label(label_text("Bridge Binary:"));
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.bridge_binary);
                        if btn_secondary(ui, "Browse").clicked() {
                            if let Some(p) =
                                rfd::FileDialog::new().set_title("Select bridge binary").pick_file()
                            {
                                self.bridge_binary = p.to_string_lossy().to_string();
                            }
                        }
                    });
                    ui.end_row();

                    ui.label(label_text("Listen Address:"));
                    ui.text_edit_singleline(&mut self.listen_address);
                    ui.end_row();

                    ui.label(label_text("gRPC RPC URL:"));
                    ui.text_edit_singleline(&mut self.grpc_url);
                    ui.end_row();

                    ui.label(label_text("Pay Address:"));
                    ui.text_edit_singleline(&mut self.pay_address);
                    ui.end_row();

                    ui.label(label_text("Extra Data (hex):"));
                    ui.text_edit_singleline(&mut self.extra_data_hex);
                    ui.end_row();

                    ui.label(label_text("Refresh Interval (ms):"));
                    ui.text_edit_singleline(&mut self.refresh_ms);
                    ui.end_row();

                    ui.label(label_text(""));
                    ui.checkbox(&mut self.allow_non_daa, "Allow non-DAA blocks");
                    ui.end_row();

                    ui.label(label_text(""));
                    ui.checkbox(&mut self.bridge_auto_restart, "Auto-restart on crash");
                    ui.end_row();
                });

            if !self.bridge_error.is_empty() {
                ui.add_space(8.0);
                alert_error(ui, &self.bridge_error);
            }

            divider(ui);

            ui.horizontal(|ui| {
                if self.proc_mgr.bridge_running {
                    if btn_danger(ui, "Stop Bridge").clicked() {
                        self.proc_mgr.stop_bridge();
                        self.log("system", "Bridge stopped");
                        self.toast("Bridge stopped", ORANGE);
                    }
                } else if btn_success(ui, "Start Bridge").clicked() {
                    self.bridge_error.clear();
                    self.save_config();
                    if self.pay_address.is_empty() {
                        self.bridge_error = "Pay address is required".into();
                    } else {
                        let args = self.build_bridge_args();
                        match self.proc_mgr.start_bridge(&self.bridge_binary, args) {
                            Ok(()) => {
                                self.log("system", "Bridge started");
                                self.toast("Bridge started", GREEN);
                                if self.miner_auto_start {
                                    let margs = self.build_miner_args();
                                    let _ =
                                        self.proc_mgr.start_miner(&self.miner_binary, margs);
                                    self.log("system", "Miner auto-started");
                                }
                            }
                            Err(e) => {
                                self.bridge_error = e;
                            }
                        }
                    }
                }

                if btn_secondary(ui, "Test RPC").clicked() {
                    let addr = self.grpc_url.replace("grpc://", "");
                    if test_tcp(&addr, 5000) {
                        self.toast("RPC: Connected", GREEN);
                        self.log("system", "RPC connectivity test: OK");
                    } else {
                        self.toast("RPC: Connection failed", RED);
                        self.log("error", "RPC connectivity test: FAILED");
                    }
                }

                if btn_secondary(ui, "Test Stratum").clicked() {
                    if test_tcp(&self.listen_address, 5000) {
                        self.toast("Stratum: Listening", GREEN);
                    } else {
                        self.toast("Stratum: Not listening", RED);
                    }
                }

                if btn_secondary(ui, "Save Config").clicked() {
                    self.save_config();
                    self.toast("Config saved", GREEN);
                    self.log("system", "Configuration saved");
                }
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if self.proc_mgr.bridge_running {
                    pill(ui, "Bridge Active", GREEN_BG, GREEN);
                    ui.add_space(8.0);
                    ui.label(body_text(&format!("Uptime: {}", self.proc_mgr.bridge_uptime())));
                }
            });
        });
    }

    // ── Miner Tab ────────────────────────────────────────────────────
    fn show_miner(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);

        card(ui, |ui| {
            section(ui, Icon::Mining, "XMRig Miner Configuration", PURPLE);
            divider(ui);
            ui.add_space(4.0);

            egui::Grid::new("miner_config")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label(label_text("XMRig Binary:"));
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.miner_binary);
                        if btn_secondary(ui, "Browse").clicked() {
                            if let Some(p) =
                                rfd::FileDialog::new().set_title("Select XMRig binary").pick_file()
                            {
                                self.miner_binary = p.to_string_lossy().to_string();
                            }
                        }
                    });
                    ui.end_row();

                    ui.label(label_text("Stratum URL:"));
                    ui.text_edit_singleline(&mut self.stratum_url);
                    ui.end_row();

                    ui.label(label_text("Extra Arguments:"));
                    ui.text_edit_singleline(&mut self.miner_extra_args);
                    ui.end_row();

                    ui.label(label_text(""));
                    ui.checkbox(&mut self.miner_auto_start, "Start miner with bridge");
                    ui.end_row();

                    ui.label(label_text(""));
                    ui.checkbox(&mut self.miner_auto_restart, "Auto-restart on crash");
                    ui.end_row();
                });

            if !self.miner_error.is_empty() {
                ui.add_space(8.0);
                alert_error(ui, &self.miner_error);
            }

            divider(ui);

            ui.horizontal(|ui| {
                if self.proc_mgr.miner_running {
                    if btn_danger(ui, "Stop Miner").clicked() {
                        self.proc_mgr.stop_miner();
                        self.log("system", "Miner stopped");
                        self.toast("Miner stopped", ORANGE);
                    }
                } else if btn_success(ui, "Start Miner").clicked() {
                    self.miner_error.clear();
                    self.save_config();
                    let args = self.build_miner_args();
                    match self.proc_mgr.start_miner(&self.miner_binary, args) {
                        Ok(()) => {
                            self.log("system", "Miner started");
                            self.toast("Miner started", GREEN);
                        }
                        Err(e) => {
                            self.miner_error = e;
                        }
                    }
                }
            });
        });

        // Live stats
        if self.proc_mgr.miner_running && self.metrics.hashrate_10s > 0.0 {
            ui.add_space(16.0);
            ui.label(subheading("Live Mining Stats"));
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                stat_box(
                    ui,
                    "Hashrate (10s)",
                    &format!("{:.1} H/s", self.metrics.hashrate_10s),
                    Icon::Chart,
                    GREEN,
                );
                ui.add_space(12.0);
                stat_box(
                    ui,
                    "Accepted",
                    &self.metrics.shares_accepted.to_string(),
                    Icon::Check,
                    BLUE,
                );
                ui.add_space(12.0);
                let rej_color = if self.metrics.shares_rejected > 0 {
                    RED
                } else {
                    TEXT_MUTED
                };
                stat_box(
                    ui,
                    "Rejected",
                    &self.metrics.shares_rejected.to_string(),
                    Icon::Warning,
                    rej_color,
                );
            });
        }
    }

    // ── Console Tab ──────────────────────────────────────────────────
    fn show_console(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.label(heading("Console"));
            ui.add_space(8.0);
            pill(
                ui,
                &format!("{} lines", self.console_lines.len()),
                BLUE_BG,
                BLUE,
            );
            ui.add_space(8.0);
            if btn_secondary(ui, "Clear").clicked() {
                self.console_lines.clear();
            }
        });
        ui.add_space(8.0);

        egui::Frame::new()
            .fill(TERMINAL_BG)
            .corner_radius(egui::CornerRadius::same(10))
            .inner_margin(egui::Margin::same(12))
            .stroke(egui::Stroke::new(1.0, BORDER))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for line in &self.console_lines {
                            let tag_color = match line.tag.as_str() {
                                "bridge" => BLUE_LIGHT,
                                "miner" => GREEN,
                                "system" => ORANGE,
                                "error" => RED,
                                _ => TEXT_MUTED,
                            };
                            ui.horizontal(|ui| {
                                ui.label(mono_term(&format!("[{}]", line.timestamp)));
                                // Color-coded tag pill
                                let tag_bg = tag_color.linear_multiply(0.2);
                                pill(ui, &line.tag, tag_bg, tag_color);
                                ui.label(mono_term(&line.text));
                            });
                        }
                    });
            });
    }

    // ── Metrics Tab ──────────────────────────────────────────────────
    fn show_metrics(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);
        ui.label(heading("Mining Metrics"));
        ui.add_space(16.0);

        // Row 1: hashrate cards
        ui.horizontal(|ui| {
            stat_box(
                ui,
                "Hashrate (10s)",
                &format!("{:.2} H/s", self.metrics.hashrate_10s),
                Icon::Chart,
                BLUE,
            );
            ui.add_space(12.0);
            stat_box(
                ui,
                "Hashrate (60s)",
                &format!("{:.2} H/s", self.metrics.hashrate_60s),
                Icon::Chart,
                PURPLE,
            );
            ui.add_space(12.0);
            stat_box(
                ui,
                "Hashrate (15m)",
                &format!("{:.2} H/s", self.metrics.hashrate_15m),
                Icon::Chart,
                TEAL,
            );
        });

        ui.add_space(12.0);

        // Row 2: shares and uptime
        ui.horizontal(|ui| {
            stat_box(
                ui,
                "Shares Accepted",
                &self.metrics.shares_accepted.to_string(),
                Icon::Check,
                GREEN,
            );
            ui.add_space(12.0);
            let rej_color = if self.metrics.shares_rejected > 0 {
                RED
            } else {
                TEXT_MUTED
            };
            stat_box(
                ui,
                "Shares Rejected",
                &self.metrics.shares_rejected.to_string(),
                Icon::Warning,
                rej_color,
            );
        });

        ui.add_space(12.0);

        // Row 3: uptime cards
        ui.horizontal(|ui| {
            card_colored(ui, BLUE, |ui| {
                ui.horizontal(|ui| {
                    icon(ui, Icon::Bridge, 22.0, BLUE);
                    ui.vertical(|ui| {
                        ui.label(label_text("Bridge Uptime"));
                        ui.label(big_number(&self.proc_mgr.bridge_uptime(), BLUE));
                    });
                });
            });
            ui.add_space(12.0);
            card_colored(ui, PURPLE, |ui| {
                ui.horizontal(|ui| {
                    icon(ui, Icon::Mining, 22.0, PURPLE);
                    ui.vertical(|ui| {
                        ui.label(label_text("Miner Uptime"));
                        ui.label(big_number(&self.proc_mgr.miner_uptime(), PURPLE));
                    });
                });
            });
        });
    }

    // ── Help Tab ─────────────────────────────────────────────────────
    fn show_help(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);

        // Quick Start card
        card(ui, |ui| {
            section(ui, Icon::Play, "Quick Start Guide", GREEN);
            divider(ui);

            let steps = [
                "1. Make sure lmtd (node) is running and synced",
                "2. In the Bridge tab, set the path to lmt-stratum-bridge binary",
                "3. Enter your LMT wallet address in 'Pay Address'",
                "4. Click 'Start Bridge' to begin serving mining jobs",
                "5. In the Miner tab, set the path to XMRig binary",
                "6. Click 'Start Miner' to begin mining",
                "7. Monitor hashrate and shares in the Metrics tab",
            ];
            for step in steps {
                ui.label(body_text(step));
                ui.add_space(3.0);
            }
        });

        ui.add_space(12.0);

        // Requirements card
        card(ui, |ui| {
            section(ui, Icon::Config, "System Requirements", ORANGE);
            divider(ui);

            ui.label(body_text("- lmtd node running and fully synced"));
            ui.label(body_text("- lmt-stratum-bridge binary"));
            ui.label(body_text("- XMRig binary (CPU miner)"));
            ui.label(body_text("- Valid LMT wallet address for rewards"));
        });

        ui.add_space(12.0);

        // Algorithm card
        card(ui, |ui| {
            section(ui, Icon::Shield, "Mining Algorithm", PURPLE);
            divider(ui);

            ui.label(body_text(
                "Lapis Monetae uses RandomX (CPU-friendly PoW algorithm)",
            ));
            ui.add_space(2.0);
            ui.label(body_text(
                "Mining is optimized for modern CPUs with AES-NI support",
            ));
        });
    }
}
