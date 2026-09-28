//! Screen-space interface: build screen, in-map HUD, result panel.
use crate::render::element_color;
use arpg_machine::content::{presets, Build, SpellKind, SupportKind};
use arpg_machine::world::{Phase, World, DASH_COOLDOWN, MOMENTUM_MAX, PROGRESS_NEEDED};
use macroquad::prelude::*;

const PANEL: Color = Color::new(0.08, 0.09, 0.12, 0.92);
const PANEL_HOVER: Color = Color::new(0.13, 0.14, 0.19, 0.95);
const BORDER: Color = Color::new(0.25, 0.27, 0.35, 1.0);
const TEXT: Color = Color::new(0.9, 0.92, 0.96, 1.0);
const MUTED: Color = Color::new(0.55, 0.58, 0.66, 1.0);
const ACCENT: Color = Color::new(1.0, 0.78, 0.3, 1.0);
const DANGER: Color = Color::new(0.95, 0.3, 0.3, 1.0);

pub struct Ui {
    pub font: Font,
}

impl Ui {
    pub fn text(&self, s: &str, x: f32, y: f32, size: u16, color: Color) {
        draw_text_ex(s, x, y, TextParams { font: Some(&self.font), font_size: size, color, ..Default::default() });
    }

    pub fn width(&self, s: &str, size: u16) -> f32 {
        measure_text(s, Some(&self.font), size, 1.0).width
    }

    fn centered(&self, s: &str, cx: f32, y: f32, size: u16, color: Color) {
        self.text(s, cx - self.width(s, size) / 2.0, y, size, color);
    }

    /// Word-wrapped text; returns the y after the last line.
    fn wrapped(&self, s: &str, x: f32, mut y: f32, max_w: f32, size: u16, color: Color) -> f32 {
        let mut line = String::new();
        for word in s.split_whitespace() {
            let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
            if self.width(&candidate, size) > max_w && !line.is_empty() {
                self.text(&line, x, y, size, color);
                y += size as f32 * 1.2;
                line = word.to_string();
            } else {
                line = candidate;
            }
        }
        if !line.is_empty() {
            self.text(&line, x, y, size, color);
            y += size as f32 * 1.2;
        }
        y
    }

    fn button(&self, r: Rect, label: &str, enabled: bool, active: bool) -> bool {
        let hover = enabled && r.contains(mouse_position().into());
        let fill = if active { Color::new(0.24, 0.2, 0.1, 0.95) } else if hover { PANEL_HOVER } else { PANEL };
        draw_rectangle(r.x, r.y, r.w, r.h, fill);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, if active { 2.0 } else { 1.0 }, if active { ACCENT } else { BORDER });
        let size = 18;
        let color = if enabled { TEXT } else { MUTED };
        self.text(label, r.x + (r.w - self.width(label, size)) / 2.0, r.y + r.h / 2.0 + 6.0, size, color);
        hover && is_mouse_button_pressed(MouseButton::Left)
    }
}

pub struct BuildState {
    pub build: Build,
    pub selected: Option<usize>,
    pub tier: u32,
    pub seed: u64,
}

pub enum BuildAction {
    None,
    Play,
}

pub fn build_screen(ui: &Ui, st: &mut BuildState) -> BuildAction {
    let (w, h) = (screen_width(), screen_height());
    ui.text("ARPG Machine", 40.0, 60.0, 40, TEXT);
    ui.text("Prototype: can a build click?", 40.0, 92.0, 20, MUTED);

    // Presets.
    let mut x = 40.0;
    ui.text("Ready builds:", x, 138.0, 18, MUTED);
    x += 120.0;
    for (name, preset) in presets() {
        let r = Rect::new(x, 116.0, 130.0, 34.0);
        if ui.button(r, name, true, st.build == preset) {
            st.build = preset;
            st.selected = None;
        }
        x += 140.0;
    }

    let col_w = (w - 120.0) / 3.0;
    let top = 180.0;

    // Spells.
    ui.text("Spells (click: add to the bar)", 40.0, top, 20, ACCENT);
    let mut y = top + 16.0;
    for spell in SpellKind::ALL {
        let r = Rect::new(40.0, y, col_w, 70.0);
        let in_bar = st.build.contains(spell);
        let can = st.build.can_add(spell);
        let hover = r.contains(mouse_position().into());
        draw_rectangle(r.x, r.y, r.w, r.h, if hover && can { PANEL_HOVER } else { PANEL });
        draw_rectangle(r.x, r.y, 5.0, r.h, element_color(spell.element()));
        let color = if can || in_bar { TEXT } else { MUTED };
        ui.text(&format!("{}   [size {}]", spell.name(), spell.size()), r.x + 16.0, r.y + 24.0, 20, color);
        ui.wrapped(spell.description(), r.x + 16.0, r.y + 46.0, r.w - 30.0, 16, MUTED);
        if in_bar {
            ui.text("on the bar", r.right() - 90.0, r.y + 24.0, 16, ACCENT);
        }
        if hover && can && is_mouse_button_pressed(MouseButton::Left) {
            st.build.add(spell);
            st.selected = Some(st.build.slots.len() - 1);
        }
        y += 78.0;
    }

    // Bar.
    let bx = 60.0 + col_w;
    ui.text(&format!("Bar ({} / {})", st.build.used(), st.build.capacity), bx, top, 20, ACCENT);
    let unit = (col_w - 10.0) / st.build.capacity as f32;
    let mut sx = bx;
    let mut clicked_slot = None;
    for (i, slot) in st.build.slots.iter().enumerate() {
        let r = Rect::new(sx, top + 16.0, unit * slot.spell.size() as f32 - 6.0, 130.0);
        let selected = st.selected == Some(i);
        let hover = r.contains(mouse_position().into());
        draw_rectangle(r.x, r.y, r.w, r.h, if hover { PANEL_HOVER } else { PANEL });
        draw_rectangle(r.x, r.y, r.w, 5.0, element_color(slot.spell.element()));
        draw_rectangle_lines(r.x, r.y, r.w, r.h, if selected { 3.0 } else { 1.0 }, if selected { ACCENT } else { BORDER });
        ui.wrapped(slot.spell.name(), r.x + 8.0, r.y + 28.0, r.w - 16.0, 17, TEXT);
        // The support sits on the bottom edge so long names wrapping above never reach it.
        let support = slot.support.map_or("no support", |s| s.name());
        ui.text(support, r.x + 8.0, r.bottom() - 12.0, 14, if slot.support.is_some() { ACCENT } else { MUTED });
        if hover && is_mouse_button_pressed(MouseButton::Left) {
            clicked_slot = Some(i);
        }
        sx += unit * slot.spell.size() as f32;
    }
    for k in st.build.used()..st.build.capacity {
        let r = Rect::new(bx + unit * k as f32, top + 16.0, unit - 6.0, 130.0);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, Color::new(0.2, 0.21, 0.26, 1.0));
    }
    if let Some(i) = clicked_slot {
        st.selected = if st.selected == Some(i) { None } else { Some(i) };
    }
    let mut info_y = top + 180.0;
    if let Some(i) = st.selected.filter(|&i| i < st.build.slots.len()) {
        let by = info_y;
        if ui.button(Rect::new(bx, by, 60.0, 34.0), "<", i > 0, false) {
            st.build.slots.swap(i, i - 1);
            st.selected = Some(i - 1);
        }
        if ui.button(Rect::new(bx + 70.0, by, 60.0, 34.0), ">", i + 1 < st.build.slots.len(), false) {
            st.build.slots.swap(i, i + 1);
            st.selected = Some(i + 1);
        }
        if ui.button(Rect::new(bx + 140.0, by, 110.0, 34.0), "Remove", true, false) {
            st.build.slots.remove(i);
            st.selected = None;
        }
        info_y += 50.0;
    }
    info_y = ui.wrapped(
        "Click a spell on the bar to select it, then a support on the right to socket it. \
         Order matters: Echo reads the spell to its left. Each support can be used once per build.",
        bx,
        info_y + 10.0,
        col_w - 10.0,
        16,
        MUTED,
    );

    // Tier and seed.
    let ty = info_y + 30.0;
    ui.text(&format!("Tier {}", st.tier), bx, ty + 22.0, 22, TEXT);
    if ui.button(Rect::new(bx + 90.0, ty, 40.0, 34.0), "-", st.tier > 0, false) {
        st.tier -= 1;
    }
    if ui.button(Rect::new(bx + 140.0, ty, 40.0, 34.0), "+", true, false) {
        st.tier += 1;
    }
    ui.text(&format!("Seed {}", st.seed), bx, ty + 72.0, 20, MUTED);
    if ui.button(Rect::new(bx + 160.0, ty + 50.0, 140.0, 34.0), "New seed", true, false) {
        st.seed = macroquad::rand::rand() as u64 % 100_000;
    }

    // Supports.
    let sx0 = 80.0 + col_w * 2.0;
    let target = st.selected.and_then(|i| st.build.slots.get(i).map(|s| (i, s.spell, s.support)));
    ui.text(
        &match target {
            Some((_, spell, _)) => format!("Supports for {}", spell.name()),
            None => "Supports (select a spell first)".to_string(),
        },
        sx0,
        top,
        20,
        ACCENT,
    );
    let mut y = top + 16.0;
    for support in SupportKind::ALL {
        let r = Rect::new(sx0, y, col_w, 70.0);
        let fits = target.is_some_and(|(_, spell, _)| support.fits(spell));
        let on_target = target.is_some_and(|(_, _, s)| s == Some(support));
        let used = st.build.support_used(support);
        let hover = r.contains(mouse_position().into());
        draw_rectangle(r.x, r.y, r.w, r.h, if hover && fits { PANEL_HOVER } else { PANEL });
        draw_rectangle_lines(r.x, r.y, r.w, r.h, if on_target { 2.0 } else { 1.0 }, if on_target { ACCENT } else { BORDER });
        ui.text(support.name(), r.x + 12.0, r.y + 24.0, 20, if fits || on_target { TEXT } else { MUTED });
        if on_target {
            ui.text("on this spell (click: remove)", r.right() - 200.0, r.y + 24.0, 15, ACCENT);
        } else if used {
            ui.text("used elsewhere", r.right() - 115.0, r.y + 24.0, 15, MUTED);
        }
        ui.wrapped(support.description(), r.x + 12.0, r.y + 44.0, r.w - 24.0, 14, MUTED);
        if hover && is_mouse_button_pressed(MouseButton::Left) {
            if let Some((i, _, _)) = target {
                if on_target {
                    st.build.slots[i].support = None;
                } else if fits {
                    st.build.socket(i, support);
                }
            }
        }
        y += 76.0;
    }

    let play = Rect::new(w - 280.0, h - 80.0, 240.0, 50.0);
    let can_play = !st.build.slots.is_empty() && st.build.is_valid();
    ui.text("WASD / ZQSD: move    Space: dash    Tab: stats    F2: autopilot    Esc: back", 40.0, h - 48.0, 18, MUTED);
    if ui.button(play, "Play (Enter)", can_play, true) || (can_play && is_key_pressed(KeyCode::Enter)) {
        return BuildAction::Play;
    }
    BuildAction::None
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

pub fn hud(ui: &Ui, world: &World, fx: &crate::render::Fx, show_stats: bool, autopilot: bool) {
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

    // Spell bar, bottom center.
    let unit = 86.0;
    let n_units: f32 = world.build.slots.iter().map(|s| s.spell.size() as f32).sum::<f32>() + 1.0;
    let total = n_units * unit + 20.0;
    let mut x = (w - total) / 2.0;
    let y = h - 100.0;

    // Movement slot.
    let dash_frac = 1.0 - world.player.dash_cooldown / DASH_COOLDOWN;
    slot_box(ui, x, y, unit - 6.0, "Dash", "Space", dash_frac, 0.0, Color::new(0.5, 0.85, 1.0, 1.0));
    x += unit + 20.0;
    for (i, slot) in world.build.slots.iter().enumerate() {
        let sw = unit * slot.spell.size() as f32 - 6.0;
        let pulse = fx.cast_pulse.get(i).copied().unwrap_or(0.0);
        let support = slot.support.map_or("", |s| s.name());
        slot_box(ui, x, y, sw, slot.spell.name(), support, world.slot_ready_fraction(i), pulse, element_color(slot.spell.element()));
        x += unit * slot.spell.size() as f32;
    }

    // Health and momentum above the bar.
    let hx = (w - total) / 2.0;
    bar(hx, y - 26.0, total - 6.0, 12.0, world.player.hp / world.player.max_hp, DANGER);
    ui.text(&format!("{:.0} / {:.0}", world.player.hp.max(0.0), world.player.max_hp), hx, y - 32.0, 16, TEXT);
    let pip_w = 10.0;
    let mx = hx + total - 6.0 - MOMENTUM_MAX as f32 * (pip_w + 3.0);
    for k in 0..MOMENTUM_MAX {
        let on = k < world.momentum;
        let c = if on { Color::new(1.0, 0.78, 0.3, 0.4 + 0.6 * world.momentum_left()) } else { Color::new(0.2, 0.2, 0.25, 0.8) };
        draw_rectangle(mx + k as f32 * (pip_w + 3.0), y - 46.0, pip_w, 12.0, c);
    }
    if world.momentum > 0 {
        ui.text(
            &format!("Momentum +{:.0}% cooldown rate", (world.cooldown_rate() - 1.0) * 100.0),
            mx - 220.0,
            y - 36.0,
            16,
            ACCENT,
        );
    }

    if show_stats {
        stats_panel(ui, world, w - 360.0, 20.0);
    }
}

fn slot_box(ui: &Ui, x: f32, y: f32, w: f32, name: &str, sub: &str, ready: f32, pulse: f32, color: Color) {
    let h = 80.0;
    draw_rectangle(x, y, w, h, PANEL);
    draw_rectangle(x, y, w, 4.0, color);
    ui.wrapped(name, x + 6.0, y + 24.0, w - 12.0, 15, TEXT);
    ui.wrapped(sub, x + 6.0, y + 64.0, w - 12.0, 13, ACCENT);
    if ready < 1.0 {
        draw_rectangle(x, y, w, h * (1.0 - ready), Color::new(0.0, 0.0, 0.0, 0.55));
    }
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
    let ph = 330.0 + world.stats.len() as f32 * 26.0;
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
    stats_panel(ui, world, px + 90.0, py + 214.0);
    let by = py + ph - 60.0;
    let replay = ui.button(Rect::new(px + 40.0, by, 210.0, 40.0), "Replay, new seed (R)", true, true);
    let edit = ui.button(Rect::new(px + 270.0, by, 210.0, 40.0), "Edit build (B)", true, false);
    if replay || is_key_pressed(KeyCode::R) {
        ResultAction::Replay
    } else if edit || is_key_pressed(KeyCode::B) || is_key_pressed(KeyCode::Escape) {
        ResultAction::Edit
    } else {
        ResultAction::None
    }
}
