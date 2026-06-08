#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod miner_app;
mod process_mgr;
mod theme;

use eframe::egui;
use miner_app::MinerApp;

fn load_icon() -> Option<egui::IconData> {
    let png_data = include_bytes!("../assets/lmt_icon_64.png");
    let image = image::load_from_memory(png_data).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Some(egui::IconData { rgba: image.into_raw(), width, height })
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1000.0, 700.0])
        .with_min_inner_size([750.0, 500.0])
        .with_title("LMT Miner Control Center");

    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }

    let options = eframe::NativeOptions { viewport, ..Default::default() };
    eframe::run_native("LMT Miner Control Center", options, Box::new(|cc| Ok(Box::new(MinerApp::new(cc)))))
}
