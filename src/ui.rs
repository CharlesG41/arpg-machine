//! Shared interface helpers, the in-map HUD and the result panel.
use crate::render::{family_color, loot_color, shrine_color, side_arrows, VIEW_WIDTH};
use arpg_machine::character::Character;
use arpg_machine::world::{Phase, ShrineKind, World, MOMENTUM_MAX, PROGRESS_NEEDED};
use macroquad::prelude::*;

pub const PANEL: Color = Color::new(0.08, 0.09, 0.12, 0.94);
pub const PANEL_HOVER: Color = Color::new(0.13, 0.14, 0.19, 0.96);
/// Fully opaque card fills, for cards drawn over lines (the skill tree).
pub const CARD: Color = Color::new(0.08, 0.09, 0.12, 1.0);
pub const CARD_HOVER: Color = Color::new(0.13, 0.14, 0.19, 1.0);
pub const BORDER: Color = Color::new(0.25, 0.27, 0.35, 1.0);
pub const TEXT: Color = Color::new(0.9, 0.92, 0.96, 1.0);
pub const MUTED: Color = Color::new(0.55, 0.58, 0.66, 1.0);
pub const ACCENT: Color = Color::new(1.0, 0.78, 0.3, 1.0);
pub const DANGER: Color = Color::new(0.95, 0.3, 0.3, 1.0);
pub const GOOD: Color = Color::new(0.45, 0.9, 0.55, 1.0);

/// The cursor, or `ARPG_MOUSE=x,y` when set (screenshots cannot move the real mouse).
pub fn mouse() -> Vec2 {
    static FORCED: std::sync::OnceLock<Option<Vec2>> = std::sync::OnceLock::new();
    let forced = FORCED.get_or_init(|| {
        let raw = std::env::var("ARPG_MOUSE").ok()?;
        let (x, y) = raw.split_once(',')?;
        Some(vec2(x.trim().parse().ok()?, y.trim().parse().ok()?))
    });
    forced.unwrap_or_else(|| mouse_position().into())
}

pub struct Ui {
    pub font: Font,
}

impl Ui {
    pub fn text(&self, s: &str, x: f32, y: f32, size: u16, color: Color) {
        draw_text_ex(s, x, y, TextParams { font: Some(&self.font), font_size: size, color, ..Default::default() });
    }

    /// Text drawn at a fixed raster size and scaled, so zooming the tree does not
    /// rasterize a new glyph size every frame.
    pub fn text_scaled(&self, s: &str, x: f32, y: f32, size: u16, scale: f32, color: Color) {
        draw_text_ex(s, x, y, TextParams { font: Some(&self.font), font_size: size, font_scale: scale, color, ..Default::default() });
    }

    pub fn width(&self, s: &str, size: u16) -> f32 {
        measure_text(s, Some(&self.font), size, 1.0).width
    }

    pub fn centered(&self, s: &str, cx: f32, y: f32, size: u16, color: Color) {
        self.text(s, cx - self.width(s, size) / 2.0, y, size, color);
    }

    /// Splits text into lines that fit `max_w`.
    pub fn wrap_lines(&self, s: &str, max_w: f32, size: u16) -> Vec<String> {
        let mut lines = Vec::new();
        let mut line = String::new();
        for word in s.split_whitespace() {
            let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
            if self.width(&candidate, size) > max_w && !line.is_empty() {
                lines.push(std::mem::replace(&mut line, word.to_string()));
            } else {
                line = candidate;
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
        lines
    }

    /// Word-wrapped text; returns the y after the last line.
    pub fn wrapped(&self, s: &str, x: f32, mut y: f32, max_w: f32, size: u16, color: Color) -> f32 {
        for line in self.wrap_lines(s, max_w, size) {
            self.text(&line, x, y, size, color);
            y += size as f32 * 1.25;
        }
        y
    }

    /// A box next to the cursor explaining what is under it. It flips to stay on screen.
    pub fn tooltip(&self, tip: &Tooltip) {
        let (mx, my) = mouse().into();
        let (sw, sh) = (screen_width(), screen_height());
        let width = 400.0;
        let inner = width - 32.0;
        let body = self.wrap_lines(&tip.body, inner, 17);
        let footer = tip.footer.as_ref().map(|(text, _)| self.wrap_lines(text, inner, 15)).unwrap_or_default();
        let height = 30.0
            + 26.0
            + 14.0
            + body.len() as f32 * 21.0
            + if footer.is_empty() { 4.0 } else { 14.0 + footer.len() as f32 * 19.0 }
            + 12.0;
        let mut x = mx + 20.0;
        let mut y = my + 20.0;
        if x + width > sw - 8.0 {
            x = mx - width - 20.0;
        }
        if y + height > sh - 8.0 {
            y = (my - height - 12.0).max(8.0);
        }
        draw_rectangle(x + 4.0, y + 5.0, width, height, Color::new(0.0, 0.0, 0.0, 0.45));
        draw_rectangle(x, y, width, height, Color::new(0.06, 0.065, 0.09, 0.98));
        draw_rectangle(x, y, width, 4.0, tip.color);
        draw_rectangle_lines(x, y, width, height, 1.0, BORDER);
        let mut ty = y + 34.0;
        self.text(&tip.title, x + 16.0, ty, 24, TEXT);
        ty += 24.0;
        self.text(&tip.subtitle, x + 16.0, ty, 15, tip.color);
        ty += 12.0;
        draw_line(x + 16.0, ty, x + width - 16.0, ty, 1.0, BORDER);
        ty += 24.0;
        for line in &body {
            self.text(line, x + 16.0, ty, 17, Color::new(0.8, 0.82, 0.88, 1.0));
            ty += 21.0;
        }
        if let Some((_, color)) = &tip.footer {
            ty += 8.0;
            for line in &footer {
                self.text(line, x + 16.0, ty, 15, *color);
                ty += 19.0;
            }
        }
    }

    pub fn button(&self, r: Rect, label: &str, enabled: bool, active: bool) -> bool {
        let hover = enabled && r.contains(mouse());
        let fill = if active { Color::new(0.24, 0.2, 0.1, 0.95) } else if hover { PANEL_HOVER } else { PANEL };
        draw_rectangle(r.x, r.y, r.w, r.h, fill);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, if active { 2.0 } else { 1.0 }, if active { ACCENT } else { BORDER });
        let size = 18;
        let color = if enabled { TEXT } else { MUTED };
        self.text(label, r.x + (r.w - self.width(label, size)) / 2.0, r.y + r.h / 2.0 + 6.0, size, color);
        hover && is_mouse_button_pressed(MouseButton::Left)
    }
}

/// What a tooltip shows: a title, a line of facts, the description and an
/// optional hint (what you can do with it here).
pub struct Tooltip {
    pub title: String,
    pub color: Color,
    pub subtitle: String,
    pub body: String,
    pub footer: Option<(String, Color)>,
}

fn fmt_time(seconds: f32) -> String {
    let s = seconds.max(0.0) as u32;
    format!("{}:{:02}", s / 60, s % 60)
}

fn bar(x: f32, y: f32, w: f32, h: f32, frac: f32, fill: Color) {
    draw_rectangle(x, y, w, h, Color::new(0.0, 0.0, 0.0, 0.6));
    draw_rectangle(x, y, w * frac.clamp(0.0, 1.0), h, fill);
    draw_rectangle_lines(x, y, w, h, 1.0, BORDER);
}

pub fn hud(ui: &Ui, world: &World, fx: &crate::render::Fx, show_stats: bool, autopilot: bool, character: Character, pending_xp: u32) {
    let (w, h) = (screen_width(), screen_height());

    // Progress or boss health, top center.
    let bw = 620.0f32.min(w - 80.0);
    let bx = (w - bw) / 2.0;
    match world.phase {
        Phase::Clearing => {
            bar(bx, 24.0, bw, 18.0, world.progress / PROGRESS_NEEDED, ACCENT);
            ui.centered(&format!("Progress {:.0} / {:.0}", world.progress, PROGRESS_NEEDED), w / 2.0, 64.0, 18, TEXT);
        }
        Phase::BossIncoming { .. } => {
            bar(bx, 24.0, bw, 18.0, 1.0, ACCENT);
            ui.centered("The Ember Warden is coming…", w / 2.0, 64.0, 22, DANGER);
        }
        _ => {
            if let Some(b) = &world.boss {
                let e = &world.enemies[b.index];
                bar(bx, 24.0, bw, 18.0, e.hp / e.max_hp, DANGER);
                let label = if b.burn_immunity > 0.0 {
                    format!("Ember Warden — immune to Burn ({:.0} s)", b.burn_immunity)
                } else {
                    "Ember Warden".to_string()
                };
                ui.centered(&label, w / 2.0, 64.0, 18, TEXT);
            }
        }
    }
    if let Some((kind, left)) = fx.shrine_banner {
        let a = left.min(1.0);
        let c = shrine_color(kind);
        ui.centered(kind.name(), w / 2.0, 150.0, 30, Color::new(c.r, c.g, c.b, a));
        ui.centered(kind.effect(), w / 2.0, 180.0, 20, Color::new(TEXT.r, TEXT.g, TEXT.b, a));
    }
    shrine_arrows(ui, world);
    loot_labels(ui, world);
    if let Some((loot, left)) = fx.loot_banner {
        let a = left.min(1.0);
        let c = loot_color(loot);
        ui.centered(&format!("Found: {}", loot.name()), w / 2.0, 230.0, 28, Color::new(c.r, c.g, c.b, a));
        ui.centered(loot.kind_name(), w / 2.0, 256.0, 18, Color::new(TEXT.r, TEXT.g, TEXT.b, a));
    }
    if fx.boss_phase_banner > 0.0 {
        let a = (fx.boss_phase_banner / 0.5).min(1.0);
        ui.centered("Immune to Burn! Adds incoming!", w / 2.0, 120.0, 30, Color::new(1.0, 0.6, 0.2, a));
    }

    // Numbers, top left.
    let lines = [
        format!("Time {}", fmt_time(world.time)),
        format!("Kills {}", world.kills),
        format!("Kills/min {:.0}  (peak {:.0})", world.kills_per_minute(), world.peak_kills_per_minute),
        format!("Awake enemies {}", world.awake_enemies()),
        format!("Tier {}   Seed {}", world.tier, world.seed),
        format!("{} FPS{}", get_fps(), if autopilot { "   AUTOPILOT (F2)" } else { "" }),
    ];
    for (i, l) in lines.iter().enumerate() {
        ui.text(l, 20.0, 30.0 + i as f32 * 22.0, 18, if i == 2 { ACCENT } else { TEXT });
    }
    // Level: the character plus what this map has earned so far.
    let mut projected = character;
    let gained = projected.add_xp(pending_xp);
    let ly = 30.0 + lines.len() as f32 * 22.0 + 8.0;
    let level_text = if gained > 0 {
        format!("Level {} (+{gained} this map, spend points after)", projected.level)
    } else {
        format!("Level {}", projected.level)
    };
    ui.text(&level_text, 20.0, ly, 18, if gained > 0 { ACCENT } else { TEXT });
    bar(20.0, ly + 6.0, 220.0, 6.0, projected.level_fraction(), Color::new(0.55, 0.45, 1.0, 1.0));

    // Shrine modifiers still running.
    let mut by = ly + 36.0;
    for (kind, left) in &world.shrine_buffs {
        let c = shrine_color(*kind);
        draw_rectangle(20.0, by - 12.0, 12.0, 12.0, c);
        let time = if left.is_infinite() { "map".to_string() } else { format!("{:.0} s", left) };
        ui.text(&format!("{}  {}", short_effect(*kind), time), 40.0, by, 16, c);
        by += 20.0;
    }

    // Spell bar, bottom center.
    let unit = 86.0;
    let n_units: f32 = world.build.slots.iter().map(|s| s.spell.size() as f32).sum::<f32>() + 1.0;
    let total = n_units * unit + 20.0;
    let mut x = (w - total) / 2.0;
    let y = h - 100.0;

    // Movement slot.
    let dash_frac = 1.0 - world.player.dash_cooldown / world.player.dash_cooldown_max;
    slot_box(ui, x, y, unit - 6.0, "Dash", "Space", dash_frac, 0.0, Color::new(0.5, 0.85, 1.0, 1.0), (false, false));
    x += unit + 20.0;
    for (i, slot) in world.build.slots.iter().enumerate() {
        let sw = unit * slot.spell.size() as f32 - 6.0;
        let pulse = fx.cast_pulse.get(i).copied().unwrap_or(0.0);
        let support = slot.support.map_or("", |s| s.name());
        slot_box(
            ui,
            x,
            y,
            sw,
            slot.spell.name(),
            support,
            world.slot_ready_fraction(i),
            pulse,
            family_color(slot.spell.family()),
            slot.spell.affects_sides(),
        );
        x += unit * slot.spell.size() as f32;
    }

    // Health, Barrier and momentum above the bar.
    let hx = (w - total) / 2.0;
    bar(hx, y - 26.0, total - 6.0, 12.0, world.player.hp / world.max_hp(), DANGER);
    if world.player.barrier > 0.0 {
        draw_rectangle(hx, y - 26.0, (total - 6.0) * (world.player.barrier / world.max_hp()).min(1.0), 4.0, Color::new(0.75, 0.9, 1.0, 1.0));
    }
    let barrier = if world.player.barrier > 0.0 { format!("   +{:.0} Barrier", world.player.barrier) } else { String::new() };
    ui.text(&format!("{:.0} / {:.0}{}", world.player.hp.max(0.0), world.max_hp(), barrier), hx, y - 32.0, 16, TEXT);
    let pip_w = 10.0;
    let mx = hx + total - 6.0 - MOMENTUM_MAX as f32 * (pip_w + 3.0);
    for k in 0..MOMENTUM_MAX {
        let on = k < world.momentum;
        let c = if on { Color::new(1.0, 0.78, 0.3, 0.4 + 0.6 * world.momentum_left()) } else { Color::new(0.2, 0.2, 0.25, 0.8) };
        draw_rectangle(mx + k as f32 * (pip_w + 3.0), y - 46.0, pip_w, 12.0, c);
    }
    if world.momentum > 0 {
        ui.text(&format!("Momentum +{:.0}% cooldown rate", (world.cooldown_rate() - 1.0) * 100.0), mx, y - 54.0, 16, ACCENT);
    }

    if show_stats {
        stats_panel(ui, world, w - 360.0, 20.0);
    }
}

fn short_effect(kind: ShrineKind) -> &'static str {
    match kind {
        ShrineKind::Fury => "+20% damage",
        ShrineKind::Swiftness => "+15% move speed",
        ShrineKind::Haste => "+20% cooldown rate",
        ShrineKind::Warding => "+15 PR and MR",
        ShrineKind::Vitality => "healed",
        ShrineKind::Ancient => "+10% damage",
    }
}

/// Names above loot beams that are on screen.
fn loot_labels(ui: &Ui, world: &World) {
    let (w, h) = (screen_width(), screen_height());
    let scale = w / VIEW_WIDTH;
    let center = vec2(w / 2.0, h / 2.0);
    let p = world.player.pos;
    for drop in world.drops.iter().filter(|d| !d.picked) {
        let at = center + vec2(drop.pos.x - p.x, drop.pos.y - p.y) * scale;
        if at.x < 0.0 || at.x > w || at.y < 0.0 || at.y > h {
            continue;
        }
        let label = drop.loot.name();
        let y = at.y - 140.0 * scale - 8.0;
        let tw = ui.width(label, 16);
        draw_rectangle(at.x - tw / 2.0 - 6.0, y - 16.0, tw + 12.0, 22.0, Color::new(0.03, 0.035, 0.05, 0.85));
        ui.text(label, at.x - tw / 2.0, y, 16, loot_color(drop.loot));
    }
}

/// Arrows on the screen edge toward the nearest shrines out of view.
fn shrine_arrows(ui: &Ui, world: &World) {
    let (w, h) = (screen_width(), screen_height());
    let scale = w / VIEW_WIDTH;
    let center = vec2(w / 2.0, h / 2.0);
    let p = world.player.pos;
    let mut shrines: Vec<_> = world.shrines.iter().map(|s| (s.pos.dist(p), s)).filter(|(d, _)| *d < 2600.0).collect();
    shrines.sort_by(|a, b| a.0.total_cmp(&b.0));
    let margin = 44.0;
    for (dist, shrine) in shrines.into_iter().take(4) {
        let offset = vec2(shrine.pos.x - p.x, shrine.pos.y - p.y) * scale;
        let on_screen = center + offset;
        if on_screen.x > margin && on_screen.x < w - margin && on_screen.y > margin + 60.0 && on_screen.y < h - 140.0 {
            continue;
        }
        let dir = offset.normalize_or_zero();
        // Walk from the center along the direction until the safe frame.
        let half = vec2(w / 2.0 - margin, (h - 200.0) / 2.0 - margin);
        let t = (half.x / dir.x.abs().max(1e-4)).min(half.y / dir.y.abs().max(1e-4));
        let at = center + dir * t;
        let c = shrine_color(shrine.kind);
        let side = dir.perp() * 10.0;
        draw_triangle(at + dir * 16.0, at - dir * 6.0 + side, at - dir * 6.0 - side, c);
        let label = format!("{:.0} m", dist / 100.0);
        ui.text(&label, at.x - ui.width(&label, 14) / 2.0, at.y - 14.0, 14, c);
    }
}

#[allow(clippy::too_many_arguments)]
fn slot_box(ui: &Ui, x: f32, y: f32, w: f32, name: &str, sub: &str, ready: f32, pulse: f32, color: Color, sides: (bool, bool)) {
    let h = 80.0;
    draw_rectangle(x, y, w, h, PANEL);
    draw_rectangle(x, y, w, 4.0, color);
    ui.wrapped(name, x + 6.0, y + 24.0, w - 12.0, 15, TEXT);
    ui.text(sub, x + 6.0, y + h - 10.0, 13, ACCENT);
    if ready < 1.0 {
        draw_rectangle(x, y, w, h * (1.0 - ready), Color::new(0.0, 0.0, 0.0, 0.55));
    }
    side_arrows(Rect::new(x, y, w, h), sides, 0.9);
    if pulse > 0.0 {
        draw_rectangle_lines(x - 2.0, y - 2.0, w + 4.0, h + 4.0, 3.0, Color::new(color.r, color.g, color.b, pulse / 0.25));
    } else {
        draw_rectangle_lines(x, y, w, h, 1.0, BORDER);
    }
}

fn stats_panel(ui: &Ui, world: &World, x: f32, y: f32) {
    let total: f32 = world.stats.iter().map(|s| s.damage).sum::<f32>().max(1.0);
    let h = 50.0 + world.stats.len() as f32 * 26.0;
    draw_rectangle(x, y, 340.0, h, PANEL);
    draw_rectangle_lines(x, y, 340.0, h, 1.0, BORDER);
    ui.text("Spell", x + 12.0, y + 26.0, 16, MUTED);
    ui.text("Damage", x + 190.0, y + 26.0, 16, MUTED);
    ui.text("Kills", x + 280.0, y + 26.0, 16, MUTED);
    for (i, (slot, s)) in world.build.slots.iter().zip(&world.stats).enumerate() {
        let ly = y + 52.0 + i as f32 * 26.0;
        ui.text(slot.spell.name(), x + 12.0, ly, 16, TEXT);
        ui.text(&format!("{:.0}%", s.damage / total * 100.0), x + 190.0, ly, 16, TEXT);
        ui.text(&format!("{}", s.kills), x + 280.0, ly, 16, TEXT);
    }
}

pub enum ResultAction {
    None,
    Replay,
    Edit,
}

pub fn result_panel(ui: &Ui, world: &World) -> ResultAction {
    let (w, h) = (screen_width(), screen_height());
    draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, 0.55));
    let pw = 520.0;
    let loot = world.loot();
    let loot_text = if loot.is_empty() {
        "Loot: nothing this time.".to_string()
    } else {
        let names: Vec<String> = loot.iter().map(|l| format!("{} ({})", l.name(), l.kind_name().to_lowercase())).collect();
        format!("Loot: {}", names.join(", "))
    };
    let loot_lines = ui.wrap_lines(&loot_text, pw - 80.0, 18);
    let ph = 330.0 + world.stats.len() as f32 * 26.0 + loot_lines.len() as f32 * 22.0 + 10.0;
    let (px, py) = ((w - pw) / 2.0, (h - ph) / 2.0);
    draw_rectangle(px, py, pw, ph, PANEL);
    draw_rectangle_lines(px, py, pw, ph, 2.0, BORDER);
    let victory = world.phase == Phase::Victory;
    ui.centered(if victory { "Victory" } else { "Defeat" }, w / 2.0, py + 56.0, 40, if victory { ACCENT } else { DANGER });
    let end = world.end_time.unwrap_or(world.time);
    let clear = world.clear_time;
    let boss = world.boss_start.map(|s| end - s);
    let lines = [
        format!("Clear: {}", clear.map_or("—".into(), fmt_time)),
        format!("Boss: {}", boss.map_or("—".into(), fmt_time)),
        format!("Total: {}", fmt_time(end)),
        format!("Kills: {}    peak: {:.0} kills/min", world.kills, world.peak_kills_per_minute),
    ];
    for (i, l) in lines.iter().enumerate() {
        ui.text(l, px + 40.0, py + 100.0 + i as f32 * 28.0, 20, TEXT);
    }
    for (i, line) in loot_lines.iter().enumerate() {
        ui.text(line, px + 40.0, py + 216.0 + i as f32 * 22.0, 18, Color::new(1.0, 0.75, 0.35, 1.0));
    }
    stats_panel(ui, world, px + 90.0, py + 226.0 + loot_lines.len() as f32 * 22.0);
    let by = py + ph - 60.0;
    let replay = ui.button(Rect::new(px + 40.0, by, 210.0, 40.0), "Replay, new map (R)", true, true);
    let edit = ui.button(Rect::new(px + 270.0, by, 210.0, 40.0), "Edit build (B)", true, false);
    if replay || is_key_pressed(KeyCode::R) {
        ResultAction::Replay
    } else if edit || is_key_pressed(KeyCode::B) || is_key_pressed(KeyCode::Escape) {
        ResultAction::Edit
    } else {
        ResultAction::None
    }
}
