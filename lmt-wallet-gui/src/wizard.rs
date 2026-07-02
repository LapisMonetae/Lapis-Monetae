use eframe::egui;
use rand::seq::SliceRandom;

use crate::theme::*;

#[derive(Debug, Clone, PartialEq)]
pub enum WizardFlow {
    None,
    Create,
    Import,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum WizardStep {
    Welcome,
    SafetyChecklist,
    NamePassword,
    ShowMnemonic,
    VerifyBackup,
    ImportSeed,
    Done,
}

pub struct WizardState {
    pub flow: WizardFlow,
    pub step: WizardStep,
    pub wallet_name: String,
    pub password: String,
    pub password_confirm: String,
    pub mnemonic_words: Vec<String>,
    pub import_seed: String,
    pub check_safe_place: bool,
    pub check_shown_once: bool,
    pub check_no_screenshot: bool,
    pub verify_indices: [usize; 3],
    pub verify_inputs: [String; 3],
    pub verify_error: String,
    pub error_msg: String,
}

impl Default for WizardState {
    fn default() -> Self {
        Self {
            flow: WizardFlow::None,
            step: WizardStep::Welcome,
            wallet_name: String::new(),
            password: String::new(),
            password_confirm: String::new(),
            mnemonic_words: Vec::new(),
            import_seed: String::new(),
            check_safe_place: false,
            check_shown_once: false,
            check_no_screenshot: false,
            verify_indices: [0, 0, 0],
            verify_inputs: [String::new(), String::new(), String::new()],
            verify_error: String::new(),
            error_msg: String::new(),
        }
    }
}

impl WizardState {
    pub fn start_create(&mut self) {
        self.flow = WizardFlow::Create;
        self.step = WizardStep::SafetyChecklist;
        self.reset_fields();
    }

    pub fn start_import(&mut self) {
        self.flow = WizardFlow::Import;
        self.step = WizardStep::NamePassword;
        self.reset_fields();
    }

    fn reset_fields(&mut self) {
        self.wallet_name.clear();
        self.password.clear();
        self.password_confirm.clear();
        self.mnemonic_words.clear();
        self.import_seed.clear();
        self.check_safe_place = false;
        self.check_shown_once = false;
        self.check_no_screenshot = false;
        self.verify_inputs = [String::new(), String::new(), String::new()];
        self.verify_error.clear();
        self.error_msg.clear();
    }

    pub fn set_mnemonic(&mut self, words: Vec<String>) {
        self.mnemonic_words = words;
        let mut rng = rand::thread_rng();
        let count = self.mnemonic_words.len();
        if count > 3 {
            let mut indices: Vec<usize> = (0..count).collect();
            indices.shuffle(&mut rng);
            self.verify_indices = [indices[0], indices[1], indices[2]];
            self.verify_indices.sort();
        }
    }

    pub fn verify_backup(&mut self) -> bool {
        for (i, idx) in self.verify_indices.iter().enumerate() {
            let input = self.verify_inputs[i].trim().to_lowercase();
            if *idx < self.mnemonic_words.len() && input != self.mnemonic_words[*idx].to_lowercase() {
                self.verify_error =
                    format!("Word #{} is incorrect. Expected '{}', got '{}'", idx + 1, self.mnemonic_words[*idx], input);
                return false;
            }
        }
        self.verify_error.clear();
        true
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> WizardAction {
        let mut action = WizardAction::None;

        match self.step {
            WizardStep::Welcome => self.show_welcome(ui),
            WizardStep::SafetyChecklist => self.show_safety_checklist(ui),
            WizardStep::NamePassword => action = self.show_name_password(ui),
            WizardStep::ShowMnemonic => self.show_mnemonic(ui),
            WizardStep::VerifyBackup => action = self.show_verify_backup(ui),
            WizardStep::ImportSeed => { /* handled inside NamePassword for import flow */ }
            WizardStep::Done => action = self.show_done(ui),
        }

        action
    }

    // ── Step 1: Welcome ──────────────────────────────────────────────
    fn show_welcome(&mut self, ui: &mut egui::Ui) {
        ui.add_space(60.0);
        ui.vertical_centered(|ui| {
            ui.set_max_width(480.0);

            card(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    icon(ui, Icon::Wallet, 48.0, BLUE);
                    ui.add_space(16.0);
                    ui.label(heading("Welcome to Lapis Monetae"));
                    ui.add_space(6.0);
                    ui.label(body_text("Create a new wallet or import an existing one to get started."));
                    ui.add_space(28.0);

                    if btn_primary(ui, "Create New Wallet").clicked() {
                        self.start_create();
                    }
                    ui.add_space(10.0);
                    if btn_secondary(ui, "Import Existing Wallet").clicked() {
                        self.start_import();
                    }

                    ui.add_space(20.0);
                });
            });
        });
    }

    // ── Step 2: Safety Checklist ─────────────────────────────────────
    fn show_safety_checklist(&mut self, ui: &mut egui::Ui) {
        ui.add_space(30.0);
        ui.vertical_centered(|ui| {
            ui.set_max_width(520.0);

            section(ui, Icon::Shield, "Safety Checklist", ORANGE);
            ui.add_space(4.0);
            ui.label(body_text("Before creating your wallet, confirm the following:"));
            ui.add_space(12.0);

            card_colored(ui, ORANGE, |ui| {
                ui.add_space(4.0);
                alert_warning(ui, "Your seed phrase is the ONLY way to recover your wallet. Take this seriously.");
                ui.add_space(14.0);

                ui.checkbox(&mut self.check_safe_place, "I am in a safe, private place");
                ui.add_space(8.0);
                ui.checkbox(&mut self.check_shown_once, "I understand the seed phrase is shown ONCE and cannot be recovered");
                ui.add_space(8.0);
                ui.checkbox(&mut self.check_no_screenshot, "I will NOT take a screenshot \u{2014} I will write it down on paper");
                ui.add_space(4.0);
            });

            ui.add_space(20.0);

            let all_checked = self.check_safe_place && self.check_shown_once && self.check_no_screenshot;

            ui.horizontal(|ui| {
                if btn_secondary(ui, "Back").clicked() {
                    self.step = WizardStep::Welcome;
                }
                ui.add_space(8.0);
                ui.add_enabled_ui(all_checked, |ui| {
                    if btn_primary(ui, "Continue").clicked() {
                        self.step = WizardStep::NamePassword;
                    }
                });
            });
        });
    }

    // ── Step 3: Name & Password (also handles import seed) ───────────
    fn show_name_password(&mut self, ui: &mut egui::Ui) -> WizardAction {
        let mut action = WizardAction::None;
        let is_import = self.flow == WizardFlow::Import;

        ui.add_space(30.0);
        ui.vertical_centered(|ui| {
            ui.set_max_width(480.0);

            section(
                ui,
                if is_import { Icon::Import } else { Icon::Wallet },
                if is_import { "Import Wallet" } else { "Create Wallet" },
                BLUE,
            );
            ui.add_space(4.0);

            card(ui, |ui| {
                ui.add_space(4.0);

                ui.label(label_text("Wallet Name"));
                ui.add_space(4.0);
                let name_edit = egui::TextEdit::singleline(&mut self.wallet_name).desired_width(f32::INFINITY).hint_text("My Wallet");
                ui.add(name_edit);
                ui.add_space(14.0);

                divider(ui);

                ui.label(label_text("Password"));
                ui.add_space(4.0);
                let pw = egui::TextEdit::singleline(&mut self.password)
                    .password(true)
                    .desired_width(f32::INFINITY)
                    .hint_text("At least 4 characters");
                ui.add(pw);
                ui.add_space(10.0);

                ui.label(label_text("Confirm Password"));
                ui.add_space(4.0);
                let pw2 = egui::TextEdit::singleline(&mut self.password_confirm).password(true).desired_width(f32::INFINITY);
                ui.add(pw2);

                if is_import {
                    ui.add_space(14.0);
                    divider(ui);

                    ui.label(label_text("Seed Phrase (12 or 24 words)"));
                    ui.add_space(4.0);
                    let seed_edit = egui::TextEdit::multiline(&mut self.import_seed)
                        .desired_width(f32::INFINITY)
                        .desired_rows(3)
                        .hint_text("Enter your seed words separated by spaces");
                    ui.add(seed_edit);
                }

                if !self.error_msg.is_empty() {
                    ui.add_space(10.0);
                    alert_error(ui, &self.error_msg);
                }

                ui.add_space(8.0);
            });

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                if btn_secondary(ui, "Back").clicked() {
                    self.step = if is_import { WizardStep::Welcome } else { WizardStep::SafetyChecklist };
                }
                ui.add_space(8.0);
                if btn_primary(ui, "Continue").clicked() {
                    action = self.validate_name_password();
                }
            });
        });

        action
    }

    fn validate_name_password(&mut self) -> WizardAction {
        if self.wallet_name.trim().is_empty() {
            self.error_msg = "Wallet name is required".into();
            return WizardAction::None;
        }
        if self.password.len() < 4 {
            self.error_msg = "Password must be at least 4 characters".into();
            return WizardAction::None;
        }
        if self.password != self.password_confirm {
            self.error_msg = "Passwords do not match".into();
            return WizardAction::None;
        }
        if self.flow == WizardFlow::Import {
            let word_count = self.import_seed.split_whitespace().count();
            if word_count != 12 && word_count != 24 {
                self.error_msg = "Seed must be 12 or 24 words".into();
                return WizardAction::None;
            }
            self.error_msg.clear();
            WizardAction::DoImport
        } else {
            self.error_msg.clear();
            WizardAction::DoCreate
        }
    }

    // ── Step 4: Show Mnemonic ────────────────────────────────────────
    fn show_mnemonic(&mut self, ui: &mut egui::Ui) {
        ui.add_space(30.0);
        ui.vertical_centered(|ui| {
            ui.set_max_width(560.0);

            section(ui, Icon::Key, "Your Seed Phrase", RED);
            ui.add_space(4.0);

            card_colored(ui, RED, |ui| {
                ui.add_space(4.0);
                alert_warning(ui, "WRITE THESE WORDS DOWN ON PAPER. This is the ONLY time they will be shown!");
                ui.add_space(14.0);

                // Word grid with terminal-style background
                egui::Frame::new()
                    .fill(TERMINAL_BG)
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::same(16))
                    .show(ui, |ui| {
                        egui::Grid::new("mnemonic_grid").num_columns(4).spacing([20.0, 10.0]).show(ui, |ui| {
                            for (i, word) in self.mnemonic_words.iter().enumerate() {
                                ui.label(mono_term(&format!("{:>2}. {}", i + 1, word)));
                                if (i + 1) % 4 == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                    });

                ui.add_space(10.0);
                alert_error(ui, "Never share your seed phrase. Anyone with these words can steal your funds.");
                ui.add_space(4.0);
            });

            ui.add_space(20.0);

            if btn_success(ui, "I have written it down").clicked() {
                self.step = WizardStep::VerifyBackup;
            }
        });
    }

    // ── Step 5: Verify Backup ────────────────────────────────────────
    fn show_verify_backup(&mut self, ui: &mut egui::Ui) -> WizardAction {
        let mut action = WizardAction::None;

        ui.add_space(30.0);
        ui.vertical_centered(|ui| {
            ui.set_max_width(480.0);

            section(ui, Icon::Check, "Verify Your Backup", GREEN);
            ui.add_space(4.0);
            ui.label(body_text("Enter the following words from your seed phrase to confirm your backup."));
            ui.add_space(12.0);

            card(ui, |ui| {
                ui.add_space(4.0);
                for i in 0..3 {
                    let idx = self.verify_indices[i];
                    ui.label(label_text(&format!("Word #{}", idx + 1)));
                    ui.add_space(4.0);
                    let edit = egui::TextEdit::singleline(&mut self.verify_inputs[i])
                        .desired_width(f32::INFINITY)
                        .hint_text(format!("Enter word #{}", idx + 1));
                    ui.add(edit);
                    if i < 2 {
                        ui.add_space(12.0);
                    }
                }

                if !self.verify_error.is_empty() {
                    ui.add_space(10.0);
                    alert_error(ui, &self.verify_error);
                }

                ui.add_space(8.0);
            });

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                if btn_secondary(ui, "Back").clicked() {
                    self.step = WizardStep::ShowMnemonic;
                }
                ui.add_space(8.0);
                if btn_success(ui, "Verify").clicked() && self.verify_backup() {
                    action = WizardAction::BackupVerified;
                    self.step = WizardStep::Done;
                }
            });
        });

        action
    }

    // ── Step 7: Done ─────────────────────────────────────────────────
    fn show_done(&mut self, ui: &mut egui::Ui) -> WizardAction {
        let mut action = WizardAction::None;

        ui.add_space(60.0);
        ui.vertical_centered(|ui| {
            ui.set_max_width(480.0);

            card_colored(ui, GREEN, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    icon(ui, Icon::Check, 48.0, GREEN);
                    ui.add_space(16.0);
                    ui.label(heading("Wallet Ready!"));
                    ui.add_space(8.0);

                    let verb = if self.flow == WizardFlow::Create { "created" } else { "imported" };
                    ui.label(body_text(&format!("Your wallet '{}' has been successfully {}.", self.wallet_name, verb,)));

                    ui.add_space(8.0);
                    pill(ui, "Ready", GREEN_BG, GREEN);

                    ui.add_space(24.0);

                    if btn_success(ui, "Open Wallet").clicked() {
                        action = WizardAction::Finish;
                    }

                    ui.add_space(20.0);
                });
            });
        });

        action
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WizardAction {
    None,
    DoCreate,
    DoImport,
    BackupVerified,
    Finish,
}
