#![allow(dead_code)]

use eframe::egui::{self, Color32, CornerRadius, FontId, Margin, Pos2, RichText, Stroke, StrokeKind, Vec2, Visuals};

// ── Light palette ─────────────────────────────────────────────────────
pub const BG_PAGE: Color32 = Color32::from_rgb(244, 246, 250);
pub const BG_WHITE: Color32 = Color32::WHITE;
pub const BG_CARD: Color32 = Color32::WHITE;
pub const BG_INPUT: Color32 = Color32::from_rgb(241, 245, 249);
pub const BG_HOVER: Color32 = Color32::from_rgb(239, 246, 255);
pub const BORDER: Color32 = Color32::from_rgb(226, 232, 240);

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(15, 23, 42);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(71, 85, 105);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 116, 139);
pub const TEXT_WHITE: Color32 = Color32::WHITE;

pub const BLUE: Color32 = Color32::from_rgb(37, 99, 235);
pub const BLUE_LIGHT: Color32 = Color32::from_rgb(59, 130, 246);
pub const BLUE_BG: Color32 = Color32::from_rgb(239, 246, 255);
pub const GREEN: Color32 = Color32::from_rgb(22, 163, 94);
pub const GREEN_BG: Color32 = Color32::from_rgb(240, 253, 244);
pub const ORANGE: Color32 = Color32::from_rgb(217, 119, 6);
pub const ORANGE_BG: Color32 = Color32::from_rgb(255, 251, 235);
pub const RED: Color32 = Color32::from_rgb(239, 68, 68);
pub const RED_BG: Color32 = Color32::from_rgb(254, 242, 242);
pub const PURPLE: Color32 = Color32::from_rgb(124, 58, 237);
pub const AMBER: Color32 = Color32::from_rgb(251, 191, 36);
pub const TEAL: Color32 = Color32::from_rgb(6, 182, 212);

pub const TERMINAL_BG: Color32 = Color32::from_rgb(30, 30, 46);
pub const TERMINAL_TEXT: Color32 = Color32::from_rgb(166, 227, 161);

// ── Theme ─────────────────────────────────────────────────────────────
pub fn setup_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::light();
    visuals.panel_fill = BG_PAGE;
    visuals.window_fill = BG_WHITE;
    visuals.extreme_bg_color = BG_INPUT;
    visuals.faint_bg_color = BG_INPUT;
    visuals.widgets.noninteractive.bg_fill = BG_WHITE;
    visuals.widgets.noninteractive.weak_bg_fill = BG_INPUT;
    visuals.widgets.inactive.bg_fill = BG_INPUT;
    visuals.widgets.inactive.weak_bg_fill = BG_INPUT;
    visuals.widgets.hovered.bg_fill = BG_HOVER;
    visuals.widgets.active.bg_fill = BLUE_BG;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, BLUE_LIGHT);
    visuals.widgets.active.bg_stroke = Stroke::new(1.5, BLUE);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_MUTED);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, BLUE);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(8);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(8);
    visuals.widgets.active.corner_radius = CornerRadius::same(8);
    visuals.window_corner_radius = CornerRadius::same(12);
    visuals.selection.bg_fill = BLUE.linear_multiply(0.15);
    visuals.selection.stroke = Stroke::new(1.0, BLUE);
    ctx.set_visuals(visuals);
}

// ── Icons ─────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Icon {
    Wallet,
    Lock,
    Send,
    Transfer,
    History,
    Contacts,
    Node,
    Config,
    Plus,
    Import,
    Play,
    Stop,
    Copy,
    Browse,
    Terminal,
    Chart,
    Help,
    Bridge,
    Mining,
    Shield,
    Eye,
    Refresh,
    Check,
    Warning,
    Globe,
    Key,
}

pub fn icon(ui: &mut egui::Ui, i: Icon, sz: f32, col: Color32) {
    let (r, _) = ui.allocate_exact_size(Vec2::splat(sz), egui::Sense::hover());
    if !ui.is_rect_visible(r) {
        return;
    }
    let p = ui.painter_at(r);
    let c = r.center();
    let s = sz * 0.42;
    match i {
        Icon::Wallet => {
            let bx = egui::Rect::from_center_size(c, Vec2::new(s * 1.7, s * 1.3));
            p.rect(bx, CornerRadius::same(3), Color32::TRANSPARENT, Stroke::new(1.8, col), StrokeKind::Outside);
            let flap = egui::Rect::from_min_size(Pos2::new(bx.right() - s * 0.65, c.y - s * 0.3), Vec2::new(s * 0.65, s * 0.6));
            p.rect(flap, CornerRadius::same(2), Color32::TRANSPARENT, Stroke::new(1.5, col), StrokeKind::Outside);
            p.circle_filled(Pos2::new(flap.center().x - s * 0.05, flap.center().y), s * 0.09, col);
        }
        Icon::Lock => {
            let body = egui::Rect::from_center_size(Pos2::new(c.x, c.y + s * 0.18), Vec2::new(s * 1.1, s * 0.9));
            p.rect_filled(body, CornerRadius::same(2), col);
            p.circle(Pos2::new(c.x, body.top()), s * 0.38, Color32::TRANSPARENT, Stroke::new(2.0, col));
        }
        Icon::Send => {
            let tip = Pos2::new(c.x + s * 0.65, c.y - s * 0.65);
            p.line_segment([Pos2::new(c.x - s * 0.55, c.y + s * 0.55), tip], Stroke::new(2.0, col));
            p.line_segment([tip, Pos2::new(tip.x - s * 0.45, tip.y)], Stroke::new(2.0, col));
            p.line_segment([tip, Pos2::new(tip.x, tip.y + s * 0.45)], Stroke::new(2.0, col));
        }
        Icon::Transfer => {
            p.arrow(Pos2::new(c.x - s * 0.35, c.y - s * 0.45), Vec2::new(0.0, s * 0.9), Stroke::new(1.8, col));
            p.arrow(Pos2::new(c.x + s * 0.35, c.y + s * 0.45), Vec2::new(0.0, -s * 0.9), Stroke::new(1.8, col));
        }
        Icon::History => {
            p.circle(c, s * 0.7, Color32::TRANSPARENT, Stroke::new(1.8, col));
            p.line_segment([c, Pos2::new(c.x, c.y - s * 0.42)], Stroke::new(1.8, col));
            p.line_segment([c, Pos2::new(c.x + s * 0.32, c.y + s * 0.05)], Stroke::new(1.8, col));
        }
        Icon::Contacts => {
            p.circle_filled(Pos2::new(c.x, c.y - s * 0.28), s * 0.32, col);
            p.rect_filled(
                egui::Rect::from_min_max(Pos2::new(c.x - s * 0.55, c.y + s * 0.12), Pos2::new(c.x + s * 0.55, c.y + s * 0.65)),
                CornerRadius { nw: 4, ne: 4, sw: 0, se: 0 },
                col,
            );
        }
        Icon::Node | Icon::Globe => {
            p.circle(c, s * 0.65, Color32::TRANSPARENT, Stroke::new(1.6, col));
            p.line_segment([Pos2::new(c.x - s * 0.65, c.y), Pos2::new(c.x + s * 0.65, c.y)], Stroke::new(1.0, col));
            p.line_segment([Pos2::new(c.x, c.y - s * 0.65), Pos2::new(c.x, c.y + s * 0.65)], Stroke::new(1.0, col));
            // Ellipse approx
            for &dx in &[-s * 0.25, s * 0.25] {
                p.line_segment([Pos2::new(c.x + dx, c.y - s * 0.55), Pos2::new(c.x + dx, c.y + s * 0.55)], Stroke::new(0.8, col));
            }
        }
        Icon::Config => {
            p.circle(c, s * 0.28, Color32::TRANSPARENT, Stroke::new(1.6, col));
            for a in 0..6 {
                let rad = (a as f32 * 60.0).to_radians();
                p.line_segment(
                    [
                        Pos2::new(c.x + rad.cos() * s * 0.38, c.y + rad.sin() * s * 0.38),
                        Pos2::new(c.x + rad.cos() * s * 0.65, c.y + rad.sin() * s * 0.65),
                    ],
                    Stroke::new(2.5, col),
                );
            }
        }
        Icon::Plus => {
            p.line_segment([Pos2::new(c.x, c.y - s * 0.5), Pos2::new(c.x, c.y + s * 0.5)], Stroke::new(2.0, col));
            p.line_segment([Pos2::new(c.x - s * 0.5, c.y), Pos2::new(c.x + s * 0.5, c.y)], Stroke::new(2.0, col));
        }
        Icon::Import => {
            p.line_segment([Pos2::new(c.x, c.y - s * 0.55), Pos2::new(c.x, c.y + s * 0.25)], Stroke::new(2.0, col));
            p.line_segment([Pos2::new(c.x, c.y + s * 0.25), Pos2::new(c.x - s * 0.28, c.y)], Stroke::new(2.0, col));
            p.line_segment([Pos2::new(c.x, c.y + s * 0.25), Pos2::new(c.x + s * 0.28, c.y)], Stroke::new(2.0, col));
            p.line_segment(
                [Pos2::new(c.x - s * 0.5, c.y + s * 0.55), Pos2::new(c.x + s * 0.5, c.y + s * 0.55)],
                Stroke::new(2.0, col),
            );
        }
        Icon::Play => {
            let pts = vec![
                Pos2::new(c.x - s * 0.35, c.y - s * 0.5),
                Pos2::new(c.x + s * 0.5, c.y),
                Pos2::new(c.x - s * 0.35, c.y + s * 0.5),
            ];
            p.add(egui::Shape::convex_polygon(pts, col, Stroke::NONE));
        }
        Icon::Stop => {
            p.rect_filled(egui::Rect::from_center_size(c, Vec2::splat(s * 0.95)), CornerRadius::same(2), col);
        }
        Icon::Copy => {
            let r1 = egui::Rect::from_min_size(Pos2::new(c.x - s * 0.5, c.y - s * 0.5), Vec2::splat(s * 0.75));
            let r2 = egui::Rect::from_min_size(Pos2::new(c.x - s * 0.2, c.y - s * 0.2), Vec2::splat(s * 0.75));
            p.rect(r1, CornerRadius::same(2), Color32::TRANSPARENT, Stroke::new(1.5, col), StrokeKind::Outside);
            p.rect_filled(r2, CornerRadius::same(2), BG_PAGE);
            p.rect(r2, CornerRadius::same(2), Color32::TRANSPARENT, Stroke::new(1.5, col), StrokeKind::Outside);
        }
        Icon::Browse => {
            let bx = egui::Rect::from_center_size(Pos2::new(c.x, c.y + s * 0.08), Vec2::new(s * 1.5, s * 0.95));
            p.rect_filled(bx, CornerRadius::same(2), col);
            p.rect_filled(
                egui::Rect::from_min_size(bx.left_top() - Vec2::new(0.0, s * 0.22), Vec2::new(s * 0.55, s * 0.25)),
                CornerRadius { nw: 2, ne: 2, sw: 0, se: 0 },
                col,
            );
        }
        Icon::Terminal => {
            let bx = egui::Rect::from_center_size(c, Vec2::new(s * 1.5, s * 1.15));
            p.rect(bx, CornerRadius::same(3), Color32::TRANSPARENT, Stroke::new(1.8, col), StrokeKind::Outside);
            p.line_segment(
                [Pos2::new(bx.left() + s * 0.18, c.y - s * 0.15), Pos2::new(bx.left() + s * 0.48, c.y + s * 0.05)],
                Stroke::new(1.8, col),
            );
            p.line_segment(
                [Pos2::new(bx.left() + s * 0.48, c.y + s * 0.05), Pos2::new(bx.left() + s * 0.18, c.y + s * 0.25)],
                Stroke::new(1.8, col),
            );
            p.line_segment(
                [Pos2::new(c.x - s * 0.05, c.y + s * 0.25), Pos2::new(c.x + s * 0.35, c.y + s * 0.25)],
                Stroke::new(1.8, col),
            );
        }
        Icon::Chart => {
            let base_y = c.y + s * 0.55;
            let bw = s * 0.28;
            for (i, h) in [0.45, 1.0, 0.65].iter().enumerate() {
                let x = c.x - s * 0.55 + (bw + s * 0.1) * i as f32;
                let ht = s * h * 1.1;
                p.rect_filled(
                    egui::Rect::from_min_size(Pos2::new(x, base_y - ht), Vec2::new(bw, ht)),
                    CornerRadius { nw: 2, ne: 2, sw: 0, se: 0 },
                    col,
                );
            }
        }
        Icon::Help => {
            p.circle(c, s * 0.62, Color32::TRANSPARENT, Stroke::new(1.8, col));
            p.text(c - Vec2::new(0.0, s * 0.05), egui::Align2::CENTER_CENTER, "?", FontId::proportional(s * 1.0), col);
        }
        Icon::Bridge => {
            p.line_segment([Pos2::new(c.x - s * 0.6, c.y - s * 0.25), Pos2::new(c.x, c.y + s * 0.25)], Stroke::new(2.0, col));
            p.line_segment([Pos2::new(c.x, c.y + s * 0.25), Pos2::new(c.x + s * 0.6, c.y - s * 0.25)], Stroke::new(2.0, col));
            p.line_segment([Pos2::new(c.x + s * 0.6, c.y + s * 0.25), Pos2::new(c.x, c.y - s * 0.25)], Stroke::new(2.0, col));
            p.line_segment([Pos2::new(c.x, c.y - s * 0.25), Pos2::new(c.x - s * 0.6, c.y + s * 0.25)], Stroke::new(2.0, col));
        }
        Icon::Mining => {
            p.line_segment(
                [Pos2::new(c.x - s * 0.5, c.y + s * 0.5), Pos2::new(c.x + s * 0.25, c.y - s * 0.25)],
                Stroke::new(2.5, col),
            );
            p.line_segment(
                [Pos2::new(c.x + s * 0.25, c.y - s * 0.25), Pos2::new(c.x + s * 0.55, c.y - s * 0.55)],
                Stroke::new(2.5, col),
            );
            p.line_segment(
                [Pos2::new(c.x + s * 0.25, c.y - s * 0.55), Pos2::new(c.x + s * 0.55, c.y - s * 0.25)],
                Stroke::new(2.0, col),
            );
        }
        Icon::Shield => {
            let pts = vec![
                Pos2::new(c.x, c.y - s * 0.7),
                Pos2::new(c.x + s * 0.55, c.y - s * 0.3),
                Pos2::new(c.x + s * 0.55, c.y + s * 0.15),
                Pos2::new(c.x, c.y + s * 0.7),
                Pos2::new(c.x - s * 0.55, c.y + s * 0.15),
                Pos2::new(c.x - s * 0.55, c.y - s * 0.3),
            ];
            p.add(egui::Shape::convex_polygon(pts, Color32::TRANSPARENT, Stroke::new(1.8, col)));
        }
        Icon::Eye => {
            p.circle_filled(c, s * 0.2, col);
            p.circle(c, s * 0.45, Color32::TRANSPARENT, Stroke::new(1.5, col));
            p.line_segment([Pos2::new(c.x - s * 0.7, c.y), Pos2::new(c.x - s * 0.45, c.y - s * 0.2)], Stroke::new(1.5, col));
            p.line_segment([Pos2::new(c.x + s * 0.7, c.y), Pos2::new(c.x + s * 0.45, c.y - s * 0.2)], Stroke::new(1.5, col));
        }
        Icon::Refresh => {
            p.circle(c, s * 0.55, Color32::TRANSPARENT, Stroke::new(1.8, col));
            p.arrow(Pos2::new(c.x + s * 0.4, c.y - s * 0.35), Vec2::new(s * 0.2, -s * 0.15), Stroke::new(1.5, col));
        }
        Icon::Check => {
            p.line_segment([Pos2::new(c.x - s * 0.4, c.y), Pos2::new(c.x - s * 0.1, c.y + s * 0.3)], Stroke::new(2.5, col));
            p.line_segment(
                [Pos2::new(c.x - s * 0.1, c.y + s * 0.3), Pos2::new(c.x + s * 0.45, c.y - s * 0.35)],
                Stroke::new(2.5, col),
            );
        }
        Icon::Warning => {
            let pts =
                vec![Pos2::new(c.x, c.y - s * 0.6), Pos2::new(c.x + s * 0.6, c.y + s * 0.5), Pos2::new(c.x - s * 0.6, c.y + s * 0.5)];
            p.add(egui::Shape::convex_polygon(pts, Color32::TRANSPARENT, Stroke::new(1.8, col)));
            p.line_segment([Pos2::new(c.x, c.y - s * 0.15), Pos2::new(c.x, c.y + s * 0.15)], Stroke::new(2.0, col));
            p.circle_filled(Pos2::new(c.x, c.y + s * 0.3), s * 0.07, col);
        }
        Icon::Key => {
            p.circle(Pos2::new(c.x - s * 0.25, c.y - s * 0.15), s * 0.35, Color32::TRANSPARENT, Stroke::new(1.8, col));
            p.line_segment([Pos2::new(c.x, c.y + s * 0.05), Pos2::new(c.x + s * 0.55, c.y + s * 0.05)], Stroke::new(1.8, col));
            p.line_segment(
                [Pos2::new(c.x + s * 0.55, c.y + s * 0.05), Pos2::new(c.x + s * 0.55, c.y + s * 0.25)],
                Stroke::new(1.8, col),
            );
            p.line_segment(
                [Pos2::new(c.x + s * 0.35, c.y + s * 0.05), Pos2::new(c.x + s * 0.35, c.y + s * 0.2)],
                Stroke::new(1.5, col),
            );
        }
    }
}

// ── Text helpers ──────────────────────────────────────────────────────
pub fn heading(t: &str) -> RichText {
    RichText::new(t).font(FontId::proportional(22.0)).color(TEXT_PRIMARY).strong()
}
pub fn subheading(t: &str) -> RichText {
    RichText::new(t).font(FontId::proportional(15.0)).color(TEXT_SECONDARY).strong()
}
pub fn section_title(t: &str) -> RichText {
    RichText::new(t).font(FontId::proportional(14.0)).color(TEXT_PRIMARY).strong()
}
pub fn label_text(t: &str) -> RichText {
    RichText::new(t).font(FontId::proportional(12.0)).color(TEXT_SECONDARY).strong()
}
pub fn body_text(t: &str) -> RichText {
    RichText::new(t).font(FontId::proportional(14.0)).color(TEXT_PRIMARY)
}
pub fn mono(t: &str) -> RichText {
    RichText::new(t).font(FontId::monospace(13.0)).color(TEXT_PRIMARY)
}
pub fn mono_term(t: &str) -> RichText {
    RichText::new(t).font(FontId::monospace(12.0)).color(TERMINAL_TEXT)
}
pub fn big_number(t: &str, col: Color32) -> RichText {
    RichText::new(t).font(FontId::proportional(28.0)).color(col).strong()
}

// ── Card ──────────────────────────────────────────────────────────────
pub fn card(ui: &mut egui::Ui, f: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(16))
        .shadow(egui::epaint::Shadow { offset: [0, 1], blur: 4, spread: 0, color: Color32::from_black_alpha(12) })
        .show(ui, |ui| f(ui));
}

pub fn card_colored(ui: &mut egui::Ui, accent: Color32, f: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.5, accent.linear_multiply(0.3)))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(16))
        .shadow(egui::epaint::Shadow { offset: [0, 2], blur: 6, spread: 0, color: accent.linear_multiply(0.08) })
        .show(ui, |ui| f(ui));
}

// ── Section divider ───────────────────────────────────────────────────
pub fn section(ui: &mut egui::Ui, ico: Icon, title: &str, col: Color32) {
    ui.horizontal(|ui| {
        icon(ui, ico, 18.0, col);
        ui.label(section_title(title));
    });
    ui.add_space(6.0);
}

pub fn divider(ui: &mut egui::Ui) {
    ui.add_space(6.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter_at(rect).rect_filled(rect, CornerRadius::ZERO, BORDER);
    ui.add_space(6.0);
}

// ── Status pill ───────────────────────────────────────────────────────
pub fn pill(ui: &mut egui::Ui, text: &str, bg: Color32, fg: Color32) {
    let galley = ui.painter().layout_no_wrap(text.to_string(), FontId::proportional(11.0), fg);
    let desired = Vec2::new(galley.size().x + 20.0, 24.0);
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, CornerRadius::same(12), bg);
    painter.text(rect.center(), egui::Align2::CENTER_CENTER, text, FontId::proportional(11.0), fg);
}

// ── Gradient header ───────────────────────────────────────────────────
pub fn gradient_bar(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 3.0), egui::Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let painter = ui.painter_at(rect);
    let n = 80;
    let w = rect.width() / n as f32;
    for i in 0..n {
        let t = i as f32 / n as f32;
        let color = if t < 0.5 {
            let u = t * 2.0;
            Color32::from_rgb((37.0 + (87.0 * u)) as u8, (99.0 + (-41.0 * u)) as u8, (235.0 + (2.0 * u)) as u8)
        } else {
            let u = (t - 0.5) * 2.0;
            Color32::from_rgb((124.0 + (121.0 * u)) as u8, (58.0 + (100.0 * u)) as u8, (237.0 + (-226.0 * u)) as u8)
        };
        painter.rect_filled(
            egui::Rect::from_min_size(Pos2::new(rect.left() + w * i as f32, rect.top()), Vec2::new(w + 1.0, 3.0)),
            CornerRadius::ZERO,
            color,
        );
    }
}

// ── Buttons ───────────────────────────────────────────────────────────
pub fn btn_primary(ui: &mut egui::Ui, txt: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(txt).font(FontId::proportional(13.0)).color(TEXT_WHITE).strong())
            .fill(BLUE)
            .corner_radius(CornerRadius::same(8))
            .min_size(Vec2::new(110.0, 36.0)),
    )
}
pub fn btn_success(ui: &mut egui::Ui, txt: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(txt).font(FontId::proportional(13.0)).color(TEXT_WHITE).strong())
            .fill(GREEN)
            .corner_radius(CornerRadius::same(8))
            .min_size(Vec2::new(110.0, 36.0)),
    )
}
pub fn btn_warning(ui: &mut egui::Ui, txt: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(txt).font(FontId::proportional(13.0)).color(TEXT_WHITE).strong())
            .fill(ORANGE)
            .corner_radius(CornerRadius::same(8))
            .min_size(Vec2::new(110.0, 36.0)),
    )
}
pub fn btn_danger(ui: &mut egui::Ui, txt: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(txt).font(FontId::proportional(13.0)).color(TEXT_WHITE).strong())
            .fill(RED)
            .corner_radius(CornerRadius::same(8))
            .min_size(Vec2::new(110.0, 36.0)),
    )
}
pub fn btn_secondary(ui: &mut egui::Ui, txt: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(txt).font(FontId::proportional(13.0)).color(TEXT_SECONDARY).strong())
            .fill(BG_INPUT)
            .stroke(Stroke::new(1.0, BORDER))
            .corner_radius(CornerRadius::same(8))
            .min_size(Vec2::new(90.0, 36.0)),
    )
}
pub fn btn_small(ui: &mut egui::Ui, txt: &str, col: Color32) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(txt).font(FontId::proportional(11.0)).color(TEXT_WHITE))
            .fill(col)
            .corner_radius(CornerRadius::same(6))
            .min_size(Vec2::new(65.0, 26.0)),
    )
}
pub fn btn_icon(ui: &mut egui::Ui, ico: Icon, txt: &str, col: Color32) -> egui::Response {
    let r = ui.horizontal(|ui| {
        icon(ui, ico, 16.0, TEXT_WHITE);
        ui.add(
            egui::Button::new(RichText::new(txt).font(FontId::proportional(13.0)).color(TEXT_WHITE).strong())
                .fill(col)
                .corner_radius(CornerRadius::same(8))
                .min_size(Vec2::new(90.0, 36.0)),
        )
    });
    r.inner
}

// ── Alert boxes ───────────────────────────────────────────────────────
pub fn alert_error(ui: &mut egui::Ui, text: &str) {
    egui::Frame::new()
        .fill(RED_BG)
        .stroke(Stroke::new(1.0, RED.linear_multiply(0.3)))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                icon(ui, Icon::Warning, 16.0, RED);
                ui.label(RichText::new(text).font(FontId::proportional(13.0)).color(RED));
            });
        });
}
pub fn alert_success(ui: &mut egui::Ui, text: &str) {
    egui::Frame::new()
        .fill(GREEN_BG)
        .stroke(Stroke::new(1.0, GREEN.linear_multiply(0.3)))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                icon(ui, Icon::Check, 16.0, GREEN);
                ui.label(RichText::new(text).font(FontId::proportional(13.0)).color(GREEN));
            });
        });
}
pub fn alert_warning(ui: &mut egui::Ui, text: &str) {
    egui::Frame::new()
        .fill(ORANGE_BG)
        .stroke(Stroke::new(1.0, ORANGE.linear_multiply(0.3)))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                icon(ui, Icon::Warning, 16.0, ORANGE);
                ui.label(RichText::new(text).font(FontId::proportional(13.0)).color(ORANGE));
            });
        });
}
pub fn alert_info(ui: &mut egui::Ui, text: &str) {
    egui::Frame::new()
        .fill(BLUE_BG)
        .stroke(Stroke::new(1.0, BLUE.linear_multiply(0.3)))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                icon(ui, Icon::Help, 16.0, BLUE);
                ui.label(RichText::new(text).font(FontId::proportional(13.0)).color(BLUE));
            });
        });
}

// ── Toast ─────────────────────────────────────────────────────────────
pub fn toast(ui: &mut egui::Ui, text: &str, color: Color32, alpha: f32) {
    egui::Frame::new()
        .fill(color.linear_multiply(alpha * 0.95))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(18, 10))
        .shadow(egui::epaint::Shadow { offset: [0, 4], blur: 12, spread: 0, color: Color32::from_black_alpha((25.0 * alpha) as u8) })
        .show(ui, |ui| {
            ui.label(RichText::new(text).font(FontId::proportional(13.0)).color(TEXT_WHITE).strong());
        });
}

// ── Tab animation helper ──────────────────────────────────────────────
pub fn animated_opacity(ctx: &egui::Context, id: &str, visible: bool) -> f32 {
    ctx.animate_bool_with_time(egui::Id::new(id), visible, 0.2)
}

// ── Stat box (for dashboard metrics) ──────────────────────────────────
pub fn stat_box(ui: &mut egui::Ui, label: &str, value: &str, ico: Icon, col: Color32) {
    card_colored(ui, col, |ui| {
        ui.horizontal(|ui| {
            icon(ui, ico, 22.0, col);
            ui.vertical(|ui| {
                ui.label(label_text(label));
                ui.label(big_number(value, col));
            });
        });
    });
}
