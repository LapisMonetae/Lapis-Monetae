#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli_bridge;
mod config;
mod contacts;
mod theme;
mod tx_history;
mod validators;
mod wallet_app;
mod wizard;

use eframe::egui;
use wallet_app::WalletApp;

fn load_icon() -> Option<egui::IconData> {
    let png_data = include_bytes!("../assets/lmt_icon_64.png");
    let image = image::load_from_memory(png_data).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Some(egui::IconData { rgba: image.into_raw(), width, height })
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1100.0, 750.0])
        .with_min_inner_size([800.0, 550.0])
        .with_title("Lapis Monetae Wallet");

    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }

    let options = eframe::NativeOptions { viewport, ..Default::default() };
    eframe::run_native("Lapis Monetae Wallet", options, Box::new(|cc| Ok(Box::new(WalletApp::new(cc)))))
}
