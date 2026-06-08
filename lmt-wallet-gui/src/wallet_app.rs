use eframe::egui;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use crate::cli_bridge::*;
use crate::config::*;
use crate::contacts::ContactsManager;
use crate::theme::*;
use crate::tx_history::*;
use crate::validators;
use crate::wizard::*;

fn load_logo(ctx: &egui::Context) -> egui::TextureHandle {
    let png_data = include_bytes!("../assets/lmt_logo.png");
    let img = image::load_from_memory(png_data).unwrap().into_rgba8();
    let size = [img.width() as _, img.height() as _];
    let pixels = img.into_raw();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
    ctx.load_texture("lmt_logo", color_image, egui::TextureOptions::LINEAR)
}

// ── Screens ──────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
enum Screen {
    Setup,
    Wizard,
    Main,
}

#[derive(Debug, Clone, PartialEq)]
enum Tab {
    Actions,
    Send,
    History,
    Node,
    Config,
    Console,
    Contacts,
}

// ── Wallet lifecycle ─────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
enum WalletState {
    Closed,
    Opening,
    Open,
    Locked,
}

// ── Toast system ─────────────────────────────────────────────────────
#[derive(Debug, Clone)]
struct ToastMsg {
    message: String,
    kind: ToastKind,
    created: Instant,
}

#[derive(Debug, Clone)]
enum ToastKind {
    Ok,
    Error,
    Info,
    Warn,
}

impl ToastKind {
    fn color(&self) -> egui::Color32 {
        match self {
            ToastKind::Ok => GREEN,
            ToastKind::Error => RED,
            ToastKind::Info => BLUE,
            ToastKind::Warn => ORANGE,
        }
    }
}

// ── Async CLI channel ────────────────────────────────────────────────
type CliChannel = (mpsc::Sender<(String, CliResult)>, mpsc::Receiver<(String, CliResult)>);

// ═════════════════════════════════════════════════════════════════════
// WalletApp
// ═════════════════════════════════════════════════════════════════════
pub struct WalletApp {
    config: AppConfig,
    screen: Screen,
    tab: Tab,
    prev_tab: Tab,
    tab_switch_time: Instant,
    wizard: WizardState,
    contacts: ContactsManager,
    logo: Option<egui::TextureHandle>,

    // Wallet lifecycle
    wallet_state: WalletState,
    wallet_name: String,
    balance: String,
    address: String,
    transactions: Vec<Transaction>,
    console_lines: Vec<String>,

    // Node
    node_info: NodeInfo,
    peer_info: PeerInfo,
    node_latency_ms: Option<u64>,
    last_node_poll: Instant,

    // Send form
    send_address: String,
    send_amount: String,
    send_fee: String,
    send_error: String,
    show_send_dialog: bool,

    // Transfer form
    transfer_account: String,
    transfer_amount: String,
    transfer_fee: String,
    transfer_error: String,
    show_transfer_dialog: bool,

    // Session
    last_activity: Instant,
    busy: bool,
    busy_text: String,

    // Toasts
    toasts: Vec<ToastMsg>,

    // Async
    cli_channel: CliChannel,

    // Setup form
    setup_cli_path: String,
    setup_network: String,
    setup_error: String,

    // About dialog
    show_about: bool,

    // Password prompt (embedded, replaces Terminal launch)
    password_prompt: Option<String>,
    password_input: String,
}

// ── Helpers ──────────────────────────────────────────────────────────
impl WalletApp {
    fn is_wallet_open(&self) -> bool {
        self.wallet_state == WalletState::Open
    }

    fn require_wallet_open(&mut self) -> bool {
        if self.is_wallet_open() {
            return true;
        }
        self.push_toast("Wallet must be open before this operation", ToastKind::Warn);
        false
    }

    fn push_toast(&mut self, msg: &str, kind: ToastKind) {
        self.toasts.push(ToastMsg { message: msg.to_string(), kind, created: Instant::now() });
    }

    fn log(&mut self, msg: &str) {
        let ts = chrono::Local::now().format("%H:%M:%S").to_string();
        self.console_lines.push(format!("[{ts}] {msg}"));
        if self.console_lines.len() > 2000 {
            self.console_lines.drain(..500);
        }
    }

    fn clear_sensitive(&mut self) {
        self.balance.clear();
        self.address.clear();
        self.transactions.clear();
        self.send_address.clear();
        self.send_amount.clear();
        self.send_fee.clear();
        self.send_error.clear();
        self.transfer_account.clear();
        self.transfer_amount.clear();
        self.transfer_fee.clear();
        self.transfer_error.clear();
        self.show_send_dialog = false;
        self.show_transfer_dialog = false;
    }

    fn lock_wallet(&mut self) {
        self.wallet_state = WalletState::Locked;
        self.wallet_name.clear();
        self.clear_sensitive();
    }

    fn run_cli_async(&self, tag: &str, args: Vec<String>) {
        let cli = self.config.cli_path.clone();
        let tag = tag.to_string();
        let tx = self.cli_channel.0.clone();
        thread::spawn(move || {
            let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            let result = run_cli(&cli, &str_args, 25);
            let _ = tx.send((tag, result));
        });
    }

    fn poll_cli_results(&mut self) {
        while let Ok((tag, result)) = self.cli_channel.1.try_recv() {
            self.busy = false;
            let snippet: String = result.output.chars().take(200).collect();
            self.log(&format!("[{tag}] exit={} output: {snippet}", result.exit_code));

            match tag.as_str() {
                "balance" => {
                    if result.exit_code == 0 {
                        self.balance = result.output.clone();
                    }
                }
                "address" => {
                    if result.exit_code == 0 {
                        for line in result.output.lines() {
                            let l = line.trim();
                            if l.starts_with("lmt:") || l.starts_with("lmttest:") {
                                self.address = l.to_string();
                                break;
                            }
                        }
                        if self.address.is_empty() {
                            self.address = result.output.lines().last().unwrap_or("").trim().to_string();
                        }
                    }
                }
                "new_address" => {
                    if result.exit_code == 0 {
                        for line in result.output.lines() {
                            let l = line.trim();
                            if l.starts_with("lmt:") || l.starts_with("lmttest:") {
                                self.address = l.to_string();
                                break;
                            }
                        }
                        self.push_toast("New address generated", ToastKind::Ok);
                    } else {
                        self.push_toast("Failed to generate address", ToastKind::Error);
                    }
                }
                "history" => {
                    if result.exit_code == 0 {
                        self.transactions = parse_transactions(&result.output);
                    }
                }
                "node_info" => {
                    if result.exit_code == 0 {
                        self.node_info = parse_node_info(&result.output);
                    }
                }
                "peer_info" => {
                    if result.exit_code == 0 {
                        self.peer_info = parse_peer_info(&result.output);
                    }
                }
                "ping" => {
                    if result.exit_code == 0 {
                        self.node_latency_ms = Some(50);
                    } else {
                        self.node_latency_ms = None;
                    }
                }
                "create_wallet" => {
                    if result.exit_code == 0 {
                        let words = extract_mnemonic(&result.output);
                        if !words.is_empty() {
                            self.wizard.set_mnemonic(words);
                            self.wizard.step = WizardStep::ShowMnemonic;
                        } else {
                            self.wizard.step = WizardStep::Done;
                        }
                        self.push_toast("Wallet created", ToastKind::Ok);
                    } else {
                        self.wizard.error_msg = format!("Failed: {}", result.output);
                    }
                }
                "import_wallet" => {
                    if result.exit_code == 0 {
                        self.wizard.step = WizardStep::Done;
                        self.push_toast("Wallet imported", ToastKind::Ok);
                    } else {
                        self.wizard.error_msg = format!("Failed: {}", result.output);
                    }
                }
                "send_tx" => {
                    if result.exit_code == 0 {
                        self.push_toast("Send successful", ToastKind::Ok);
                        self.refresh_wallet_data();
                    } else {
                        self.push_toast("Send failed", ToastKind::Error);
                    }
                }
                "transfer_tx" => {
                    if result.exit_code == 0 {
                        self.push_toast("Transfer successful", ToastKind::Ok);
                        self.refresh_wallet_data();
                    } else {
                        self.push_toast("Transfer failed", ToastKind::Error);
                    }
                }
                "lock" => {
                    self.lock_wallet();
                    self.push_toast("Wallet locked", ToastKind::Info);
                }
                "open_wallet" => {
                    if result.exit_code == 0 {
                        self.wallet_state = WalletState::Open;
                        self.push_toast("Wallet opened", ToastKind::Ok);
                        self.refresh_wallet_data();
                    } else {
                        self.wallet_state = WalletState::Closed;
                        self.push_toast("Failed to open wallet", ToastKind::Error);
                    }
                }
                _ => {}
            }
        }
    }

    fn poll_node(&mut self) {
        if self.last_node_poll.elapsed().as_secs() >= 12 && !self.config.cli_path.is_empty() {
            self.last_node_poll = Instant::now();
            self.run_cli_async("node_info", vec!["rpc".into(), "get_block_dag_info".into()]);
            self.run_cli_async("peer_info", vec!["rpc".into(), "get_connected_peer_info".into()]);
            self.run_cli_async("ping", vec!["ping".into()]);
        }
    }

    fn check_session_timeout(&mut self) {
        if self.config.session_timeout_min > 0
            && self.is_wallet_open()
            && self.last_activity.elapsed().as_secs() > (self.config.session_timeout_min as u64 * 60)
        {
            self.run_cli_async("lock", vec!["wallet".into(), "close".into()]);
            self.lock_wallet();
            self.push_toast("Session timed out - wallet locked", ToastKind::Warn);
        }
    }

    fn refresh_wallet_data(&mut self) {
        if !self.is_wallet_open() {
            return;
        }
        self.run_cli_async("balance", vec!["list".into()]);
        self.run_cli_async("address", vec!["address".into()]);
        self.run_cli_async("history", vec!["history".into(), "list".into(), "30".into()]);
    }
}

// ── Constructor ──────────────────────────────────────────────────────
impl WalletApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        let config = AppConfig::load();
        let contacts = ContactsManager::new(config.contacts.clone(), config.network.clone());
        let setup_cli_path = config.cli_path.clone();
        let setup_network = config.network.clone();

        let config_path = AppConfig::config_path();
        let config_exists = config_path.exists() && !config.cli_path.is_empty();

        let mut app = Self {
            config,
            screen: Screen::Setup, // default; overridden below
            tab: Tab::Actions,
            prev_tab: Tab::Actions,
            tab_switch_time: Instant::now(),
            wizard: WizardState::default(),
            contacts,
            wallet_state: WalletState::Closed,
            wallet_name: String::new(),
            balance: String::new(),
            address: String::new(),
            transactions: Vec::new(),
            console_lines: Vec::new(),
            node_info: NodeInfo::default(),
            peer_info: PeerInfo::default(),
            node_latency_ms: None,
            last_node_poll: Instant::now(),
            send_address: String::new(),
            send_amount: String::new(),
            send_fee: String::new(),
            send_error: String::new(),
            show_send_dialog: false,
            transfer_account: String::new(),
            transfer_amount: String::new(),
            transfer_fee: String::new(),
            transfer_error: String::new(),
            show_transfer_dialog: false,
            last_activity: Instant::now(),
            busy: false,
            busy_text: String::new(),
            toasts: Vec::new(),
            cli_channel: mpsc::channel(),
            setup_cli_path,
            setup_network,
            setup_error: String::new(),
            show_about: false,
            password_prompt: None,
            password_input: String::new(),
            logo: None,
        };

        // Load logo texture
        app.logo = Some(load_logo(&_cc.egui_ctx));

        // Auto-detect CLI
        if app.setup_cli_path.is_empty() {
            if let Some(p) = app.config.resolve_cli() {
                app.setup_cli_path = p.to_string_lossy().to_string();
            }
        }

        // Decide initial screen:
        // - Config exists AND CLI valid => Main dashboard (wallet still closed)
        // - Otherwise => Setup (onboarding)
        if config_exists && !app.setup_cli_path.is_empty() && verify_cli(&app.setup_cli_path) {
            app.config.cli_path = app.setup_cli_path.clone();
            app.screen = Screen::Main;
        } else {
            app.screen = Screen::Setup;
        }

        app
    }
}

// ── eframe::App ──────────────────────────────────────────────────────
impl eframe::App for WalletApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        setup_theme(ctx);
        self.poll_cli_results();
        self.poll_node();
        self.check_session_timeout();

        // Expire toasts after 3 seconds
        self.toasts.retain(|t| t.created.elapsed().as_secs_f32() < 3.0);

        // Keep repainting for timers / animations
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        match self.screen {
            Screen::Setup => self.show_setup(ctx),
            Screen::Wizard => self.show_wizard(ctx),
            Screen::Main => self.show_main(ctx),
        }

        // Toast overlay
        self.show_toasts(ctx);
    }
}

// ═════════════════════════════════════════════════════════════════════
// SETUP (Onboarding)
// ═════════════════════════════════════════════════════════════════════
impl WalletApp {
    fn show_setup(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                icon(ui, Icon::Wallet, 48.0, BLUE);
                ui.add_space(8.0);
                ui.label(heading("Lapis Monetae Wallet"));
                ui.add_space(4.0);
                ui.label(subheading("First-time setup - configure your wallet"));
            });
            ui.add_space(30.0);

            card(ui, |ui| {
                section(ui, Icon::Config, "CLI Configuration", BLUE);

                ui.label(label_text("Path to lmt-cli binary:"));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.setup_cli_path);
                    if btn_secondary(ui, "Browse...").clicked() {
                        if let Some(path) = rfd::FileDialog::new().set_title("Select lmt-cli binary").pick_file() {
                            self.setup_cli_path = path.to_string_lossy().to_string();
                        }
                    }
                });
                ui.add_space(12.0);

                ui.label(label_text("Network:"));
                ui.add_space(4.0);
                egui::ComboBox::from_id_salt("network_select").selected_text(&self.setup_network).show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.setup_network, "mainnet".into(), "mainnet");
                    ui.selectable_value(&mut self.setup_network, "testnet-10".into(), "testnet-10");
                    ui.selectable_value(&mut self.setup_network, "testnet-11".into(), "testnet-11");
                });

                if !self.setup_error.is_empty() {
                    ui.add_space(8.0);
                    alert_error(ui, &self.setup_error);
                }

                ui.add_space(16.0);
                ui.vertical_centered(|ui| {
                    if btn_primary(ui, "Continue").clicked() {
                        if self.setup_cli_path.is_empty() {
                            self.setup_error = "Please specify the lmt-cli path".into();
                        } else if !verify_cli(&self.setup_cli_path) {
                            self.setup_error = "Invalid lmt-cli binary (could not run --version)".into();
                        } else {
                            self.config.cli_path = self.setup_cli_path.clone();
                            self.config.network = self.setup_network.clone();
                            self.config.save();
                            self.setup_error.clear();

                            let _ = run_cli(&self.config.cli_path, &["network", &self.config.network], 5);
                            self.screen = Screen::Wizard;
                            self.wizard.step = WizardStep::Welcome;
                        }
                    }
                });
            });
        });
    }

    // ═════════════════════════════════════════════════════════════════
    // WIZARD
    // ═════════════════════════════════════════════════════════════════
    fn show_wizard(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let action = self.wizard.show(ui);

            match action {
                WizardAction::DoCreate => {
                    self.password_prompt = Some(format!("create:{}", self.wizard.wallet_name));
                    self.password_input.clear();
                    self.log("Enter password to create wallet");
                }
                WizardAction::DoImport => {
                    self.password_prompt = Some(format!("import:{}", self.wizard.wallet_name));
                    self.password_input.clear();
                    self.log("Enter password to import wallet");
                }
                WizardAction::BackupVerified => {
                    self.config.seed_backups_confirmed.push(self.wizard.wallet_name.clone());
                    self.config.save();
                }
                WizardAction::Finish => {
                    self.wallet_name = self.wizard.wallet_name.clone();
                    self.config.last_wallet = self.wallet_name.clone();
                    self.config.save();
                    self.screen = Screen::Main;
                    // After wizard the user still needs to open the wallet
                    // via the interactive terminal (password entry).
                    self.wallet_state = WalletState::Closed;
                }
                WizardAction::None => {}
            }
        });
    }

    // ═════════════════════════════════════════════════════════════════
    // MAIN SCREEN
    // ═════════════════════════════════════════════════════════════════
    fn show_main(&mut self, ctx: &egui::Context) {
        // ── Top header ───────────────────────────────────────────────
        egui::TopBottomPanel::top("top_bar")
            .frame(
                egui::Frame::new().fill(BG_WHITE).inner_margin(egui::Margin::symmetric(16, 10)).stroke(egui::Stroke::new(1.0, BORDER)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Logo image in header
                    if let Some(ref logo) = self.logo {
                        let size = egui::Vec2::new(28.0, 28.0);
                        ui.image(egui::load::SizedTexture::new(logo.id(), size));
                    } else {
                        icon(ui, Icon::Wallet, 28.0, BLUE);
                    }
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("Lapis Monetae Wallet")
                            .font(egui::FontId::proportional(17.0))
                            .color(TEXT_PRIMARY)
                            .strong(),
                    );
                    ui.add_space(12.0);

                    match self.wallet_state {
                        WalletState::Open => {
                            pill(ui, &format!(" {} ", self.wallet_name), GREEN, TEXT_WHITE);
                        }
                        WalletState::Opening => {
                            pill(ui, " Opening... ", ORANGE_BG, ORANGE);
                        }
                        WalletState::Locked => {
                            pill(ui, " Locked ", RED_BG, RED);
                        }
                        WalletState::Closed => {
                            pill(ui, " No Wallet ", BG_INPUT, TEXT_MUTED);
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        pill(ui, &format!(" {} ", self.config.network), BLUE_BG, BLUE);
                        if self.node_latency_ms.is_some() {
                            pill(ui, " Connected ", GREEN_BG, GREEN);
                        } else {
                            pill(ui, " Offline ", RED_BG, RED);
                        }
                        if btn_secondary(ui, "About").clicked() {
                            self.show_about = true;
                        }
                    });
                });
            });

        // ── Gradient bar ─────────────────────────────────────────────
        egui::TopBottomPanel::top("gradient").frame(egui::Frame::NONE).exact_height(3.0).show(ctx, |ui| {
            gradient_bar(ui);
        });

        // ── Tab bar ──────────────────────────────────────────────────
        egui::TopBottomPanel::top("tab_bar")
            .frame(
                egui::Frame::new().fill(BG_WHITE).inner_margin(egui::Margin::symmetric(8, 0)).stroke(egui::Stroke::new(1.0, BORDER)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let tabs: [(Tab, &str, Icon); 7] = [
                        (Tab::Actions, "Actions", Icon::Wallet),
                        (Tab::Send, "Send", Icon::Send),
                        (Tab::History, "History", Icon::History),
                        (Tab::Node, "Node", Icon::Globe),
                        (Tab::Contacts, "Contacts", Icon::Contacts),
                        (Tab::Config, "Config", Icon::Config),
                        (Tab::Console, "Console", Icon::Terminal),
                    ];
                    for (t, name, ico) in tabs {
                        let selected = self.tab == t;
                        let color = if selected { BLUE } else { TEXT_MUTED };
                        ui.add_space(2.0);
                        let resp = ui
                            .horizontal(|ui| {
                                icon(ui, ico, 16.0, color);
                                let text = egui::RichText::new(name).font(egui::FontId::proportional(13.0)).color(color).strong();
                                ui.selectable_label(selected, text)
                            })
                            .inner;
                        if resp.clicked() {
                            self.prev_tab = self.tab.clone();
                            self.tab = t;
                            self.tab_switch_time = Instant::now();
                            self.last_activity = Instant::now();
                        }
                    }
                });
            });

        // ── Central content ──────────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            self.last_activity = Instant::now();

            // Animated opacity for tab transitions
            let tab_id = format!("tab_{:?}", self.tab);
            let opacity = animated_opacity(ctx, &tab_id, true);
            ui.set_opacity(opacity);

            match self.tab {
                Tab::Actions => self.show_actions(ui),
                Tab::Send => self.show_send_tab(ui),
                Tab::History => self.show_history(ui),
                Tab::Node => self.show_node(ui),
                Tab::Contacts => self.show_contacts(ui),
                Tab::Config => self.show_config(ui),
                Tab::Console => self.show_console(ui),
            }

            // Busy overlay
            if self.busy {
                let rect = ui.max_rect();
                ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, egui::Color32::from_black_alpha(150));
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &self.busy_text,
                    egui::FontId::proportional(18.0),
                    TEXT_WHITE,
                );
            }
        });

        // ── Modal dialogs ────────────────────────────────────────────
        if self.show_send_dialog {
            self.show_send_dialog_window(ctx);
        }
        if self.show_transfer_dialog {
            self.show_transfer_dialog_window(ctx);
        }
        if self.contacts.show_dialog {
            self.show_contact_dialog(ctx);
        }
        if self.show_about {
            self.show_about_dialog(ctx);
        }
        if self.password_prompt.is_some() {
            self.show_password_dialog(ctx);
        }
    }

    // ═════════════════════════════════════════════════════════════════
    // TAB: Actions (Dashboard)
    // ═════════════════════════════════════════════════════════════════
    fn show_actions(&mut self, ui: &mut egui::Ui) {
        ui.add_space(14.0);

        // ── Top row: 3 stat boxes (equal width, same row) ────────────
        let total_w = ui.available_width();
        let box_w = (total_w - 16.0) / 3.0; // 2 gaps of 8px
        let box_h = 80.0;

        let bal_display = if self.balance.is_empty() { "--".to_string() } else { self.balance.clone() };
        let addr_short = if self.address.is_empty() {
            "--".to_string()
        } else if self.address.len() > 16 {
            format!("{}...", &self.address[..16])
        } else {
            self.address.clone()
        };
        let (net_status, net_col) = if self.node_latency_ms.is_some() { ("Online", GREEN) } else { ("Offline", RED) };

        ui.columns(3, |cols| {
            cols[0].allocate_ui_with_layout(egui::Vec2::new(box_w, box_h), egui::Layout::top_down(egui::Align::Min), |ui| {
                stat_box(ui, "Balance", &bal_display, Icon::Wallet, GREEN);
            });
            cols[1].allocate_ui_with_layout(egui::Vec2::new(box_w, box_h), egui::Layout::top_down(egui::Align::Min), |ui| {
                stat_box(ui, "Address", &addr_short, Icon::Copy, BLUE);
            });
            cols[2].allocate_ui_with_layout(egui::Vec2::new(box_w, box_h), egui::Layout::top_down(egui::Align::Min), |ui| {
                stat_box(ui, "Network", net_status, Icon::Globe, net_col);
            });
        });
        ui.add_space(14.0);

        // ── Wallet actions ───────────────────────────────────────────
        card(ui, |ui| {
            section(ui, Icon::Wallet, "Wallet Actions", BLUE);
            ui.horizontal_wrapped(|ui| {
                if btn_primary(ui, "Open Wallet").clicked() {
                    self.password_prompt = Some("open".to_string());
                    self.password_input.clear();
                    self.log("Enter password to open wallet...");
                }
                if btn_success(ui, "Create Wallet").clicked() {
                    self.screen = Screen::Wizard;
                    self.wizard.start_create();
                }
                if btn_secondary(ui, "Import").clicked() {
                    self.screen = Screen::Wizard;
                    self.wizard.start_import();
                }
                if self.is_wallet_open() {
                    if btn_warning(ui, "Lock").clicked() {
                        self.run_cli_async("lock", vec!["wallet".into(), "close".into()]);
                    }
                }
            });
        });
        ui.add_space(14.0);

        // ── Address card ─────────────────────────────────────────────
        if self.is_wallet_open() {
            card(ui, |ui| {
                section(ui, Icon::Copy, "Receive Address", BLUE);
                if self.address.is_empty() {
                    ui.label(body_text("No address available"));
                } else {
                    ui.label(mono(&self.address));
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if btn_small(ui, "Copy", BLUE).clicked() {
                            ui.ctx().copy_text(self.address.clone());
                            self.push_toast("Address copied", ToastKind::Ok);
                        }
                        if btn_small(ui, "New Address", GREEN).clicked() {
                            if self.require_wallet_open() {
                                self.run_cli_async("new_address", vec!["address".into(), "new".into()]);
                            }
                        }
                    });
                }
            });
            ui.add_space(14.0);
        }

        // ── Quick actions row (inside card, aligned) ────────────────
        card(ui, |ui| {
            section(ui, Icon::Send, "Quick Actions", ORANGE);
            ui.horizontal(|ui| {
                if btn_primary(ui, "Refresh").clicked() {
                    if self.is_wallet_open() {
                        self.refresh_wallet_data();
                    } else {
                        self.push_toast("Open a wallet first", ToastKind::Warn);
                    }
                }
                ui.add_space(4.0);
                if btn_warning(ui, "Send").clicked() {
                    if self.require_wallet_open() {
                        self.show_send_dialog = true;
                        self.send_error.clear();
                    }
                }
                ui.add_space(4.0);
                if btn_secondary(ui, "Transfer").clicked() {
                    if self.require_wallet_open() {
                        self.show_transfer_dialog = true;
                        self.transfer_error.clear();
                    }
                }
            });
        });
        ui.add_space(14.0);

        // ── Recent activity (last 5 console lines) ──────────────────
        card(ui, |ui| {
            section(ui, Icon::Terminal, "Recent Activity", TEXT_SECONDARY);
            if self.console_lines.is_empty() {
                ui.label(body_text("No activity yet"));
            } else {
                let start = self.console_lines.len().saturating_sub(5);
                egui::Frame::new()
                    .fill(TERMINAL_BG)
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        for line in &self.console_lines[start..] {
                            ui.label(mono_term(line));
                        }
                    });
            }
        });
    }

    // ═════════════════════════════════════════════════════════════════
    // TAB: Send
    // ═════════════════════════════════════════════════════════════════
    fn show_send_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(14.0);

        if !self.is_wallet_open() {
            alert_warning(ui, "You must open a wallet before sending transactions.");
            return;
        }

        card(ui, |ui| {
            section(ui, Icon::Send, "Send LMT", ORANGE);

            ui.label(label_text("Recipient Address"));
            ui.add_space(2.0);
            ui.text_edit_singleline(&mut self.send_address);
            ui.add_space(8.0);

            ui.label(label_text("Amount"));
            ui.add_space(2.0);
            ui.text_edit_singleline(&mut self.send_amount);
            ui.add_space(8.0);

            ui.label(label_text("Fee (optional)"));
            ui.add_space(2.0);
            ui.text_edit_singleline(&mut self.send_fee);

            // Quick pick from contacts
            if !self.contacts.contacts.is_empty() {
                ui.add_space(10.0);
                divider(ui);
                ui.label(label_text("Quick pick from contacts:"));
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    for c in &self.contacts.contacts {
                        let short = if c.address.len() > 16 { &c.address[..16] } else { &c.address };
                        if btn_small(ui, &format!("{} ({}...)", c.name, short), TEAL).clicked() {
                            self.send_address = c.address.clone();
                        }
                    }
                });
            }

            if !self.send_error.is_empty() {
                ui.add_space(8.0);
                alert_error(ui, &self.send_error);
            }

            ui.add_space(14.0);
            if btn_warning(ui, "Send Transaction").clicked() {
                self.do_send();
            }
        });
    }

    fn do_send(&mut self) {
        if !self.require_wallet_open() {
            return;
        }
        if let Err(e) = validators::validate_address(&self.send_address, &self.config.network) {
            self.send_error = e;
        } else if let Err(e) = validators::validate_amount(&self.send_amount) {
            self.send_error = e;
        } else if let Err(e) = validators::validate_fee(&self.send_fee) {
            self.send_error = e;
        } else {
            self.send_error.clear();
            let fee = if self.send_fee.is_empty() { "0".to_string() } else { self.send_fee.clone() };
            self.password_prompt = Some(format!("send:{}:{}:{}", self.send_address, self.send_amount, fee));
            self.password_input.clear();
            self.log(&format!("Send {} LMT to {} - enter password", self.send_amount, self.send_address));
        }
    }

    // ═════════════════════════════════════════════════════════════════
    // TAB: History
    // ═════════════════════════════════════════════════════════════════
    fn show_history(&mut self, ui: &mut egui::Ui) {
        ui.add_space(14.0);

        card(ui, |ui| {
            ui.horizontal(|ui| {
                icon(ui, Icon::History, 22.0, BLUE);
                ui.add_space(4.0);
                ui.label(heading("Transaction History"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if btn_small(ui, "Refresh", BLUE).clicked() {
                        if self.is_wallet_open() {
                            self.run_cli_async("history", vec!["history".into(), "list".into(), "30".into()]);
                        }
                    }
                });
            });
            divider(ui);

            if self.transactions.is_empty() {
                ui.label(body_text("No transactions found"));
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("tx_grid").num_columns(4).spacing([15.0, 6.0]).striped(true).show(ui, |ui| {
                        ui.label(label_text("TxID"));
                        ui.label(label_text("Direction"));
                        ui.label(label_text("Amount"));
                        ui.label(label_text("Status"));
                        ui.end_row();

                        for tx in &self.transactions {
                            let short_id = if tx.tx_id.len() > 12 {
                                format!("{}...{}", &tx.tx_id[..6], &tx.tx_id[tx.tx_id.len() - 6..])
                            } else {
                                tx.tx_id.clone()
                            };
                            ui.label(mono(&short_id));

                            let (dir_text, dir_color) = match tx.direction {
                                TxDirection::Incoming => ("IN", GREEN),
                                TxDirection::Outgoing => ("OUT", ORANGE),
                            };
                            pill(ui, dir_text, dir_color.linear_multiply(0.15), dir_color);
                            ui.label(mono(&tx.amount));

                            let status_color = if tx.status == "confirmed" { GREEN } else { AMBER };
                            pill(ui, &tx.status, status_color.linear_multiply(0.15), status_color);
                            ui.end_row();
                        }
                    });
                });
            }
        });
    }

    // ═════════════════════════════════════════════════════════════════
    // TAB: Node
    // ═════════════════════════════════════════════════════════════════
    fn show_node(&mut self, ui: &mut egui::Ui) {
        ui.add_space(14.0);

        card(ui, |ui| {
            section(ui, Icon::Globe, "Node Information", BLUE);

            let connected = self.node_latency_ms.is_some();

            egui::Grid::new("node_grid").num_columns(2).spacing([20.0, 10.0]).show(ui, |ui| {
                ui.label(label_text("Status:"));
                if connected {
                    pill(ui, "Connected", GREEN_BG, GREEN);
                } else {
                    pill(ui, "Disconnected", RED_BG, RED);
                }
                ui.end_row();

                ui.label(label_text("Network:"));
                ui.label(mono(if self.node_info.network.is_empty() { &self.config.network } else { &self.node_info.network }));
                ui.end_row();

                ui.label(label_text("DAA Score:"));
                ui.label(mono(&self.node_info.daa_score.to_string()));
                ui.end_row();

                ui.label(label_text("Difficulty:"));
                ui.label(mono(&format!("{:.2}", self.node_info.difficulty)));
                ui.end_row();

                if !self.node_info.tip_hash.is_empty() {
                    ui.label(label_text("Tip Hash:"));
                    ui.label(mono(&self.node_info.tip_hash));
                    ui.end_row();
                }

                ui.label(label_text("Header Count:"));
                ui.label(mono(&self.node_info.header_count.to_string()));
                ui.end_row();

                ui.label(label_text("Block Count:"));
                ui.label(mono(&self.node_info.block_count.to_string()));
                ui.end_row();

                ui.label(label_text("Peers:"));
                ui.label(mono(&self.peer_info.count.to_string()));
                ui.end_row();

                if let Some(ms) = self.node_latency_ms {
                    ui.label(label_text("Latency:"));
                    ui.label(mono(&format!("{ms} ms")));
                    ui.end_row();
                }
            });
        });
    }

    // ═════════════════════════════════════════════════════════════════
    // TAB: Contacts
    // ═════════════════════════════════════════════════════════════════
    fn show_contacts(&mut self, ui: &mut egui::Ui) {
        ui.add_space(14.0);

        card(ui, |ui| {
            ui.horizontal(|ui| {
                icon(ui, Icon::Contacts, 22.0, BLUE);
                ui.add_space(4.0);
                ui.label(heading("Contacts"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if btn_small(ui, "+ Add", GREEN).clicked() {
                        self.contacts.open_add();
                    }
                });
            });
            divider(ui);

            ui.label(label_text("Search:"));
            ui.add_space(2.0);
            ui.text_edit_singleline(&mut self.contacts.search);
            ui.add_space(10.0);

            let filtered: Vec<(usize, crate::config::Contact)> =
                self.contacts.filtered().into_iter().map(|(i, c)| (i, c.clone())).collect();
            if filtered.is_empty() {
                ui.label(body_text("No contacts"));
            } else {
                let mut to_remove = None;
                let mut to_edit = None;
                let mut copied_addr: Option<String> = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (idx, contact) in filtered.iter() {
                        egui::Frame::new()
                            .fill(BG_INPUT)
                            .corner_radius(egui::CornerRadius::same(8))
                            .inner_margin(egui::Margin::same(10))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new(&contact.name).color(TEXT_PRIMARY).strong());
                                        ui.label(mono(&contact.address));
                                        if !contact.note.is_empty() {
                                            ui.label(label_text(&contact.note));
                                        }
                                    });
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if btn_small(ui, "Delete", RED).clicked() {
                                            to_remove = Some(*idx);
                                        }
                                        if btn_small(ui, "Edit", BLUE).clicked() {
                                            to_edit = Some(*idx);
                                        }
                                        if btn_small(ui, "Copy", TEAL).clicked() {
                                            copied_addr = Some(contact.address.clone());
                                        }
                                    });
                                });
                            });
                        ui.add_space(4.0);
                    }
                });
                if let Some(addr) = copied_addr {
                    ui.ctx().copy_text(addr);
                    self.push_toast("Address copied", ToastKind::Ok);
                }
                if let Some(idx) = to_remove {
                    self.contacts.remove(idx);
                    self.config.contacts = self.contacts.contacts.clone();
                    self.config.save();
                }
                if let Some(idx) = to_edit {
                    self.contacts.open_edit(idx);
                }
            }
        });
    }

    // ═════════════════════════════════════════════════════════════════
    // TAB: Config
    // ═════════════════════════════════════════════════════════════════
    fn show_config(&mut self, ui: &mut egui::Ui) {
        ui.add_space(14.0);

        card(ui, |ui| {
            section(ui, Icon::Config, "Configuration", BLUE);

            ui.label(label_text("CLI Binary Path"));
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.config.cli_path);
                if btn_secondary(ui, "Browse").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.config.cli_path = path.to_string_lossy().to_string();
                    }
                }
            });
            ui.add_space(10.0);

            ui.label(label_text("Network:"));
            ui.add_space(2.0);
            let prev_network = self.config.network.clone();
            egui::ComboBox::from_id_salt("config_network").selected_text(&self.config.network).show_ui(ui, |ui| {
                ui.selectable_value(&mut self.config.network, "mainnet".into(), "mainnet");
                ui.selectable_value(&mut self.config.network, "testnet-10".into(), "testnet-10");
                ui.selectable_value(&mut self.config.network, "testnet-11".into(), "testnet-11");
            });
            if self.config.network != prev_network {
                let _ = run_cli(&self.config.cli_path, &["network", &self.config.network], 5);
                self.log(&format!("Switched network to {}", self.config.network));
            }
            ui.add_space(10.0);

            ui.label(label_text("Session Timeout (minutes, 0 = disabled):"));
            ui.add_space(2.0);
            ui.add(egui::Slider::new(&mut self.config.session_timeout_min, 0..=60));

            divider(ui);

            if btn_primary(ui, "Save Configuration").clicked() {
                self.config.save();
                self.push_toast("Configuration saved", ToastKind::Ok);
            }
        });
    }

    // ═════════════════════════════════════════════════════════════════
    // TAB: Console
    // ═════════════════════════════════════════════════════════════════
    fn show_console(&mut self, ui: &mut egui::Ui) {
        ui.add_space(14.0);

        card(ui, |ui| {
            ui.horizontal(|ui| {
                icon(ui, Icon::Terminal, 22.0, BLUE);
                ui.add_space(4.0);
                ui.label(heading("Console"));
                ui.add_space(8.0);
                pill(ui, &format!("{} lines", self.console_lines.len()), BLUE_BG, BLUE);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if btn_small(ui, "Clear", RED).clicked() {
                        self.console_lines.clear();
                    }
                });
            });
            divider(ui);

            egui::Frame::new().fill(TERMINAL_BG).corner_radius(egui::CornerRadius::same(6)).inner_margin(egui::Margin::same(10)).show(
                ui,
                |ui| {
                    egui::ScrollArea::vertical().stick_to_bottom(true).max_height(ui.available_height() - 8.0).show(ui, |ui| {
                        for line in &self.console_lines {
                            ui.label(mono_term(line));
                        }
                    });
                },
            );
        });
    }

    // ═════════════════════════════════════════════════════════════════
    // DIALOG: Send
    // ═════════════════════════════════════════════════════════════════
    fn show_send_dialog_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_send_dialog;
        egui::Window::new("Send LMT").open(&mut open).resizable(false).collapsible(false).show(ctx, |ui| {
            if !self.is_wallet_open() {
                alert_warning(ui, "Wallet is not open.");
                return;
            }

            ui.label(label_text("Address:"));
            ui.text_edit_singleline(&mut self.send_address);
            ui.add_space(5.0);
            ui.label(label_text("Amount:"));
            ui.text_edit_singleline(&mut self.send_amount);
            ui.add_space(5.0);
            ui.label(label_text("Fee:"));
            ui.text_edit_singleline(&mut self.send_fee);

            if !self.send_error.is_empty() {
                ui.add_space(5.0);
                alert_error(ui, &self.send_error);
            }

            ui.add_space(10.0);
            if btn_success(ui, "Confirm Send").clicked() {
                self.do_send();
                if self.send_error.is_empty() {
                    self.show_send_dialog = false;
                }
            }
        });
        self.show_send_dialog = open;
    }

    // ═════════════════════════════════════════════════════════════════
    // DIALOG: Transfer
    // ═════════════════════════════════════════════════════════════════
    fn show_transfer_dialog_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_transfer_dialog;
        egui::Window::new("Transfer Between Accounts").open(&mut open).resizable(false).collapsible(false).show(ctx, |ui| {
            if !self.is_wallet_open() {
                alert_warning(ui, "Wallet is not open.");
                return;
            }

            ui.label(label_text("Target Account:"));
            ui.text_edit_singleline(&mut self.transfer_account);
            ui.add_space(5.0);
            ui.label(label_text("Amount:"));
            ui.text_edit_singleline(&mut self.transfer_amount);
            ui.add_space(5.0);
            ui.label(label_text("Fee:"));
            ui.text_edit_singleline(&mut self.transfer_fee);

            if !self.transfer_error.is_empty() {
                ui.add_space(5.0);
                alert_error(ui, &self.transfer_error);
            }

            ui.add_space(10.0);
            if btn_warning(ui, "Confirm Transfer").clicked() {
                if !self.require_wallet_open() {
                    return;
                }
                if self.transfer_account.trim().is_empty() {
                    self.transfer_error = "Target account is required".into();
                } else if let Err(e) = validators::validate_amount(&self.transfer_amount) {
                    self.transfer_error = e;
                } else if let Err(e) = validators::validate_fee(&self.transfer_fee) {
                    self.transfer_error = e;
                } else {
                    self.transfer_error.clear();
                    let fee = if self.transfer_fee.is_empty() { "0".into() } else { self.transfer_fee.clone() };
                    self.password_prompt = Some(format!("transfer:{}:{}:{}", self.transfer_account, self.transfer_amount, fee));
                    self.password_input.clear();
                    self.show_transfer_dialog = false;
                    self.push_toast("Enter password to confirm transfer", ToastKind::Info);
                    self.log(&format!("Transfer {} LMT to account {} - enter password", self.transfer_amount, self.transfer_account));
                }
            }
        });
        self.show_transfer_dialog = open;
    }

    // ═════════════════════════════════════════════════════════════════
    // DIALOG: Contact editor
    // ═════════════════════════════════════════════════════════════════
    fn show_contact_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.contacts.show_dialog;
        let title = if self.contacts.editing_index.is_some() { "Edit Contact" } else { "Add Contact" };
        egui::Window::new(title).open(&mut open).resizable(false).show(ctx, |ui| {
            ui.label(label_text("Name:"));
            ui.text_edit_singleline(&mut self.contacts.edit_name);
            ui.add_space(5.0);
            ui.label(label_text("Address:"));
            ui.text_edit_singleline(&mut self.contacts.edit_address);
            ui.add_space(5.0);
            ui.label(label_text("Note (optional):"));
            ui.text_edit_singleline(&mut self.contacts.edit_note);

            if !self.contacts.validation_error.is_empty() {
                ui.add_space(8.0);
                alert_error(ui, &self.contacts.validation_error);
            }

            ui.add_space(10.0);
            if btn_primary(ui, "Save").clicked() {
                self.contacts.network = self.config.network.clone();
                if self.contacts.save_contact() {
                    self.config.contacts = self.contacts.contacts.clone();
                    self.config.save();
                    self.push_toast("Contact saved", ToastKind::Ok);
                }
            }
        });
        self.contacts.show_dialog = open;
    }

    // ═════════════════════════════════════════════════════════════════
    // TOAST overlay
    // ═════════════════════════════════════════════════════════════════
    // ═════════════════════════════════════════════════════════════════
    // DIALOG: About
    // ═════════════════════════════════════════════════════════════════
    fn show_about_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.show_about;
        egui::Window::new("About").open(&mut open).resizable(false).collapsible(false).show(ctx, |ui| {
            card(ui, |ui| {
                ui.vertical_centered(|ui| {
                    icon(ui, Icon::Wallet, 48.0, BLUE);
                    ui.add_space(8.0);
                    ui.label(heading("Lapis Monetae Wallet"));
                    ui.add_space(4.0);
                    ui.label(body_text("Version 1.0.1"));
                    ui.add_space(4.0);
                    ui.label(body_text("Lapis Monetae Project"));
                    ui.add_space(12.0);
                    if btn_secondary(ui, "Close").clicked() {
                        self.show_about = false;
                    }
                });
            });
        });
        self.show_about = open;
    }

    // ═════════════════════════════════════════════════════════════════
    // DIALOG: Password prompt
    // ═════════════════════════════════════════════════════════════════
    fn show_password_dialog(&mut self, ctx: &egui::Context) {
        let prompt_label = match self.password_prompt.as_deref() {
            Some(s) if s.starts_with("create:") => "Enter password to create wallet",
            Some(s) if s.starts_with("import:") => "Enter password to import wallet",
            Some("open") => "Enter password to open wallet",
            Some(s) if s.starts_with("send:") => "Enter password to confirm send",
            Some(s) if s.starts_with("transfer:") => "Enter password to confirm transfer",
            _ => "Enter password",
        };

        let mut open = true;
        egui::Window::new("Password Required").open(&mut open).resizable(false).collapsible(false).show(ctx, |ui| {
            card(ui, |ui| {
                ui.vertical_centered(|ui| {
                    icon(ui, Icon::Lock, 32.0, BLUE);
                    ui.add_space(8.0);
                    ui.label(body_text(prompt_label));
                });
                ui.add_space(8.0);
                ui.label(label_text("Password:"));
                let response = ui.add(egui::TextEdit::singleline(&mut self.password_input).password(true));

                // Auto-focus the password field
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.execute_password_operation();
                    return;
                }

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if btn_primary(ui, "Confirm").clicked() {
                        self.execute_password_operation();
                    }
                    if btn_secondary(ui, "Cancel").clicked() {
                        self.password_prompt = None;
                        self.password_input.clear();
                    }
                });
            });
        });
        if !open {
            self.password_prompt = None;
            self.password_input.clear();
        }
    }

    fn execute_password_operation(&mut self) {
        let prompt = match self.password_prompt.take() {
            Some(p) => p,
            None => return,
        };
        let password = std::mem::take(&mut self.password_input);
        let cli = self.config.cli_path.clone();

        if prompt.starts_with("create:") {
            let name = prompt.strip_prefix("create:").unwrap_or("").to_string();
            let tx = self.cli_channel.0.clone();
            // Password twice (password + confirm) + "y" for phishing check + "y" for BIP39 check
            let stdin_data = format!("{password}\n{password}\ny\ny\n");
            self.busy = true;
            self.busy_text = "Creating wallet...".into();
            thread::spawn(move || {
                let args: Vec<&str> = vec!["wallet", "create", &name];
                let result = run_cli_with_stdin(&cli, &args, &stdin_data, 30);
                let _ = tx.send(("create_wallet".to_string(), result));
            });
        } else if prompt.starts_with("import:") {
            let name = prompt.strip_prefix("import:").unwrap_or("").to_string();
            let tx = self.cli_channel.0.clone();
            let stdin_data = format!("{password}\n{password}\n");
            self.busy = true;
            self.busy_text = "Importing wallet...".into();
            thread::spawn(move || {
                let args: Vec<&str> = vec!["wallet", "import", &name];
                let result = run_cli_with_stdin(&cli, &args, &stdin_data, 30);
                let _ = tx.send(("import_wallet".to_string(), result));
            });
        } else if prompt == "open" {
            let tx = self.cli_channel.0.clone();
            let stdin_data = format!("{password}\n");
            self.busy = true;
            self.busy_text = "Opening wallet...".into();
            self.wallet_state = WalletState::Opening;
            thread::spawn(move || {
                let args: Vec<&str> = vec!["wallet", "open"];
                let result = run_cli_with_stdin(&cli, &args, &stdin_data, 15);
                let _ = tx.send(("open_wallet".to_string(), result));
            });
        } else if prompt.starts_with("send:") {
            let parts: Vec<&str> = prompt.strip_prefix("send:").unwrap_or("").splitn(3, ':').collect();
            if parts.len() == 3 {
                let addr = parts[0].to_string();
                let amount = parts[1].to_string();
                let fee = parts[2].to_string();
                let tx = self.cli_channel.0.clone();
                let stdin_data = format!("{password}\n");
                self.busy = true;
                self.busy_text = "Sending...".into();
                thread::spawn(move || {
                    let args: Vec<&str> = vec!["send", &addr, &amount, &fee];
                    let result = run_cli_with_stdin(&cli, &args, &stdin_data, 30);
                    let _ = tx.send(("send_tx".to_string(), result));
                });
            }
        } else if prompt.starts_with("transfer:") {
            let parts: Vec<&str> = prompt.strip_prefix("transfer:").unwrap_or("").splitn(3, ':').collect();
            if parts.len() == 3 {
                let account = parts[0].to_string();
                let amount = parts[1].to_string();
                let fee = parts[2].to_string();
                let tx = self.cli_channel.0.clone();
                let stdin_data = format!("{password}\n");
                self.busy = true;
                self.busy_text = "Transferring...".into();
                thread::spawn(move || {
                    let args: Vec<&str> = vec!["transfer", &account, &amount, &fee];
                    let result = run_cli_with_stdin(&cli, &args, &stdin_data, 30);
                    let _ = tx.send(("transfer_tx".to_string(), result));
                });
            }
        }
    }

    fn show_toasts(&self, ctx: &egui::Context) {
        if self.toasts.is_empty() {
            return;
        }
        egui::Area::new(egui::Id::new("toasts")).anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-20.0, 70.0)).show(ctx, |ui| {
            for t in &self.toasts {
                let alpha = 1.0 - (t.created.elapsed().as_secs_f32() / 3.0).min(1.0);
                toast(ui, &t.message, t.kind.color(), alpha);
                ui.add_space(6.0);
            }
        });
    }
}

// ── Mnemonic extraction helper ───────────────────────────────────────
fn extract_mnemonic(output: &str) -> Vec<String> {
    let words: Vec<&str> = output.split_whitespace().collect();
    let mut start = None;
    for (i, w) in words.iter().enumerate() {
        if w.chars().all(|c| c.is_ascii_lowercase()) && w.len() >= 3 && w.len() <= 10 {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(s) = start {
            let count = i - s;
            if count == 12 || count == 24 {
                return words[s..i].iter().map(|w| w.to_string()).collect();
            }
            start = None;
        }
    }
    if let Some(s) = start {
        let count = words.len() - s;
        if count == 12 || count == 24 {
            return words[s..].iter().map(|w| w.to_string()).collect();
        }
    }
    Vec::new()
}
