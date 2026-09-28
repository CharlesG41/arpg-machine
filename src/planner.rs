//! Planning screens: the skill tree, and the loadout where spells, supports and
//! artifacts are dragged into place.
use crate::render::family_color;
use crate::ui::{Ui, ACCENT, BORDER, DANGER, GOOD, MUTED, PANEL, PANEL_HOVER, TEXT};
use arpg_machine::content::{presets, ArtifactKind, Build, Family, SpellId, SupportKind};
use arpg_machine::math::V2;
use arpg_machine::tree::{SkillTree, ORIGIN};
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Tree,
    Loadout,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Item {
    Spell(SpellId),
    Support(SupportKind),
    Artifact(ArtifactKind),
}

/// Where a dragged item was picked up.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Source {
    Collection,
    Bar,
    Socket,
    ArtifactBar,
}

struct Drag {
    item: Item,
    source: Source,
    start: Vec2,
    active: bool,
}

pub enum PlanAction {
    None,
    Play,
}

const HEADER: f32 = 76.0;
pub const DEFAULT_POINTS: u32 = 25;
const CARD_H: f32 = 92.0;
const DRAG_THRESHOLD: f32 = 5.0;

// Loadout layout (logical pixels, laid out for the default 1600 × 900 window).
const COLLECTION_X: f32 = 24.0;
const LIST_Y: f32 = 150.0;
/// The spell collection starts below the family filter.
const SPELLS_Y: f32 = LIST_Y + 34.0;
const FAMILIES: [Family; 15] = [
    Family::Arcane,
    Family::Fire,
    Family::Frost,
    Family::Lightning,
    Family::Poison,
    Family::Blood,
    Family::Shadow,
    Family::Light,
    Family::Nature,
    Family::Steel,
    Family::Earth,
    Family::Time,
    Family::Spirits,
    Family::Alchemy,
    Family::Astral,
];
const BAR_X: f32 = 530.0;
const BAR_Y: f32 = 150.0;
const BAR_H: f32 = 132.0;
const ART_Y: f32 = 350.0;
const ART_UNIT: f32 = 108.0;
const ART_H: f32 = 74.0;

fn tree_card_w(size: u8) -> f32 {
    if size >= 2 {
        265.0
    } else {
        218.0
    }
}

fn v2(p: V2) -> Vec2 {
    vec2(p.x, p.y)
}

pub struct Planner {
    pub tree: SkillTree,
    pub build: Build,
    pub tier: u32,
    pub sandbox: bool,
    pub tab: Tab,
    pan: Vec2,
    zoom: f32,
    fitted: bool,
    pan_drag: Option<(Vec2, Vec2, bool)>,
    drag: Option<Drag>,
    notice: Option<(String, f32)>,
    /// Scroll offset of the spell collection, in pixels.
    list_scroll: f32,
    /// Show only this family in the spell collection.
    family_filter: Option<Family>,
}

impl Planner {
    pub fn new(seed: u64) -> Self {
        Planner {
            tree: SkillTree::generate(seed, DEFAULT_POINTS),
            build: Build::empty(),
            tier: 0,
            sandbox: false,
            tab: Tab::Tree,
            pan: Vec2::ZERO,
            zoom: 0.5,
            fitted: false,
            pan_drag: None,
            drag: None,
            notice: None,
            list_scroll: 0.0,
            family_filter: None,
        }
    }

    /// Spells shown in the collection: available ones, through the family filter.
    fn listed(&self) -> Vec<SpellId> {
        self.available().into_iter().filter(|s| self.family_filter.is_none_or(|f| s.family() == f)).collect()
    }

    fn filter_rect(i: usize) -> Rect {
        Rect::new(COLLECTION_X + i as f32 * 32.4, LIST_Y, 29.0, 24.0)
    }

    /// Spells the loadout may use: the tree's, or all of them in sandbox.
    fn available(&self) -> Vec<SpellId> {
        if self.sandbox {
            SpellId::all().collect()
        } else {
            let mut spells = self.tree.acquired_spells();
            spells.sort();
            spells
        }
    }

    /// Drops bar spells the tree no longer grants.
    fn prune(&mut self) {
        let available = self.available();
        let before = self.build.slots.len();
        self.build.slots.retain(|s| available.contains(&s.spell));
        if self.build.slots.len() < before {
            self.say("Refunded spells left the spell bar.");
        }
    }

    fn say(&mut self, message: &str) {
        self.notice = Some((message.to_string(), 3.0));
    }

    pub fn new_character(&mut self) {
        let seed = self.tree.seed.wrapping_add(1);
        self.tree = SkillTree::generate(seed, self.tree.points);
        self.fitted = false;
        self.prune();
    }

    pub fn load_preset(&mut self, build: Build) {
        self.sandbox = true;
        self.build = build;
    }

    pub fn draw(&mut self, ui: &Ui) -> PlanAction {
        match self.tab {
            Tab::Tree => self.tree_screen(ui),
            Tab::Loadout => self.loadout_screen(ui),
        }
        self.header(ui)
    }

    fn header(&mut self, ui: &Ui) -> PlanAction {
        let w = screen_width();
        draw_rectangle(0.0, 0.0, w, HEADER, Color::new(0.045, 0.05, 0.068, 0.98));
        draw_line(0.0, HEADER, w, HEADER, 1.0, BORDER);
        ui.text("ARPG Machine", 24.0, 46.0, 30, TEXT);
        if ui.button(Rect::new(240.0, 20.0, 150.0, 38.0), "Skill Tree", true, self.tab == Tab::Tree) {
            self.tab = Tab::Tree;
        }
        if ui.button(Rect::new(400.0, 20.0, 150.0, 38.0), "Loadout", true, self.tab == Tab::Loadout) {
            self.tab = Tab::Loadout;
        }
        if is_key_pressed(KeyCode::Tab) {
            self.tab = if self.tab == Tab::Tree { Tab::Loadout } else { Tab::Tree };
        }
        ui.text(&format!("Map tier {}", self.tier), w - 520.0, 46.0, 20, TEXT);
        if ui.button(Rect::new(w - 410.0, 22.0, 36.0, 34.0), "-", self.tier > 0, false) {
            self.tier -= 1;
        }
        if ui.button(Rect::new(w - 368.0, 22.0, 36.0, 34.0), "+", true, false) {
            self.tier += 1;
        }
        let can_play = !self.build.slots.is_empty() && self.build.is_valid();
        let label = if can_play { "Play (Enter)" } else { "Add a spell first" };
        if let Some((text, time)) = &mut self.notice {
            *time -= get_frame_time();
            let a = time.min(1.0);
            let tw = ui.width(text, 20);
            let (nx, ny) = ((w - tw) / 2.0, screen_height() - 28.0);
            draw_rectangle(nx - 16.0, ny - 26.0, tw + 32.0, 38.0, Color::new(0.08, 0.09, 0.12, 0.9 * a));
            ui.text(text, nx, ny, 20, Color::new(ACCENT.r, ACCENT.g, ACCENT.b, a));
            if *time <= 0.0 {
                self.notice = None;
            }
        }
        if ui.button(Rect::new(w - 300.0, 16.0, 276.0, 44.0), label, can_play, can_play)
            || (can_play && is_key_pressed(KeyCode::Enter))
        {
            return PlanAction::Play;
        }
        PlanAction::None
    }

    // ---- skill tree ---------------------------------------------------------

    fn canvas(&self) -> Rect {
        Rect::new(0.0, HEADER, screen_width(), screen_height() - HEADER)
    }

    fn to_screen(&self, p: V2) -> Vec2 {
        self.canvas().center() + self.pan + v2(p) * self.zoom
    }

    fn node_rect(&self, id: usize) -> Rect {
        let node = &self.tree.nodes[id];
        let c = self.to_screen(node.pos);
        let (w, h) = match node.spell {
            Some(s) => (tree_card_w(s.size()), CARD_H),
            None => (90.0, 90.0),
        };
        Rect::new(c.x - w * self.zoom / 2.0, c.y - h * self.zoom / 2.0, w * self.zoom, h * self.zoom)
    }

    fn fit(&mut self) {
        let canvas = self.canvas();
        let (mut min, mut max) = (vec2(f32::MAX, f32::MAX), vec2(f32::MIN, f32::MIN));
        for n in &self.tree.nodes {
            min = min.min(v2(n.pos));
            max = max.max(v2(n.pos));
        }
        let size = max - min + vec2(320.0, 200.0);
        let fit_all = (canvas.w / size.x).min(canvas.h / size.y);
        // Big trees start readable around the Origin; zoom out to see the rest.
        self.zoom = fit_all.max(0.55).clamp(0.2, 1.2);
        self.pan = if self.zoom > fit_all { Vec2::ZERO } else { -(min + max) / 2.0 * self.zoom };
        self.fitted = true;
    }

    fn tree_screen(&mut self, ui: &Ui) {
        let (w, h) = (screen_width(), screen_height());
        if !self.fitted {
            self.fit();
        }
        let canvas = self.canvas();
        let mouse: Vec2 = mouse_position().into();
        let panel = Rect::new(16.0, HEADER + 14.0, 320.0, 214.0);
        let over_panel = panel.contains(mouse);
        let in_canvas = canvas.contains(mouse) && !over_panel;
        let hovered = if in_canvas { (0..self.tree.nodes.len()).find(|&i| self.node_rect(i).contains(mouse)) } else { None };
        let route = hovered.and_then(|id| self.tree.route_to(id)).unwrap_or_default();
        let affordable = route.len() as u32 <= self.tree.remaining();

        // Zoom around the cursor.
        let wheel = mouse_wheel().1;
        if in_canvas && wheel != 0.0 {
            let factor = if wheel > 0.0 { 1.12 } else { 1.0 / 1.12 };
            let zoom = (self.zoom * factor).clamp(0.15, 1.5);
            let world = (mouse - canvas.center() - self.pan) / self.zoom;
            self.zoom = zoom;
            self.pan = mouse - canvas.center() - world * zoom;
        }
        // Left: drag to pan, click to buy the route. Right: refund.
        if in_canvas && is_mouse_button_pressed(MouseButton::Left) {
            self.pan_drag = Some((mouse, self.pan, false));
        }
        if let Some((start, pan0, mut moved)) = self.pan_drag {
            if (mouse - start).length() > DRAG_THRESHOLD {
                moved = true;
            }
            if moved {
                self.pan = pan0 + (mouse - start);
            }
            self.pan_drag = Some((start, pan0, moved));
            if is_mouse_button_released(MouseButton::Left) {
                if !moved {
                    if let Some(id) = hovered {
                        self.buy(id);
                    }
                }
                self.pan_drag = None;
            }
        }
        if is_mouse_button_pressed(MouseButton::Right) {
            if let Some(id) = hovered {
                let refunded = self.tree.refund(id);
                if refunded > 0 {
                    self.say(&format!("Refunded {} point{} (respec is free).", refunded, if refunded > 1 { "s" } else { "" }));
                    self.prune();
                }
            }
        }

        // Edges.
        let owned = |i: usize| self.tree.acquired[i];
        let on_route = |i: usize| route.contains(&i);
        let route_color = if affordable { Color::new(0.4, 0.85, 1.0, 1.0) } else { DANGER };
        for (i, node) in self.tree.nodes.iter().enumerate() {
            for &j in &node.neighbors {
                if j < i {
                    continue;
                }
                let (a, b) = (self.to_screen(node.pos), self.to_screen(self.tree.nodes[j].pos));
                let (color, width) = if owned(i) && owned(j) {
                    (ACCENT, 4.0)
                } else if (on_route(i) || on_route(j)) && (on_route(i) || owned(i)) && (on_route(j) || owned(j)) {
                    (route_color, 4.0)
                } else {
                    (Color::new(0.24, 0.26, 0.33, 1.0), 2.0)
                };
                draw_line(a.x, a.y, b.x, b.y, (width * self.zoom).max(1.5), color);
            }
        }

        // Nodes.
        let z = self.zoom;
        for id in 0..self.tree.nodes.len() {
            let r = self.node_rect(id);
            if !r.overlaps(&canvas) {
                continue;
            }
            let node = &self.tree.nodes[id];
            let is_owned = self.tree.acquired[id];
            let Some(spell) = node.spell else {
                let c = r.center();
                draw_circle(c.x, c.y, r.w / 2.0, Color::new(0.2, 0.16, 0.07, 1.0));
                draw_circle_lines(c.x, c.y, r.w / 2.0, 3.0, ACCENT);
                let tw = ui.width("Origin", 20) * z;
                ui.text_scaled("Origin", c.x - tw / 2.0, c.y + 7.0 * z, 20, z, ACCENT);
                continue;
            };
            let fill = if is_owned { Color::new(0.17, 0.15, 0.09, 1.0) } else if Some(id) == hovered { PANEL_HOVER } else { PANEL };
            draw_rectangle(r.x, r.y, r.w, r.h, fill);
            draw_rectangle(r.x, r.y, 7.0 * z, r.h, family_color(spell.family()));
            let (border, thick) = if on_route(id) {
                (route_color, 3.0)
            } else if is_owned {
                (ACCENT, 3.0)
            } else if Some(id) == hovered {
                (WHITE, 2.0)
            } else {
                (BORDER, 1.0)
            };
            draw_rectangle_lines(r.x, r.y, r.w, r.h, thick, border);
            let name_color = if is_owned { TEXT } else { Color::new(0.78, 0.8, 0.86, 1.0) };
            ui.text_scaled(spell.name(), r.x + 16.0 * z, r.y + 36.0 * z, 26, z, name_color);
            if z > 0.24 {
                let line = format!("{} · {} · size {}", spell.family().name(), spell.delivery().name(), spell.size());
                ui.text_scaled(&line, r.x + 16.0 * z, r.y + 64.0 * z, 17, z, MUTED);
            }
            if self.build.contains(spell) {
                ui.text_scaled("ON BAR", r.right() - 74.0 * z, r.y + 84.0 * z, 16, z, GOOD);
            }
        }

        // Hover label.
        if let Some(id) = hovered {
            let label = if self.tree.acquired[id] {
                if id == ORIGIN {
                    "Origin".to_string()
                } else {
                    "Owned — right click to refund (free)".to_string()
                }
            } else if affordable {
                format!("Click: buy route, {} point{}", route.len(), if route.len() > 1 { "s" } else { "" })
            } else {
                format!("Needs {} points, {} left", route.len(), self.tree.remaining())
            };
            let tw = ui.width(&label, 18);
            let (lx, ly) = ((mouse.x + 18.0).min(w - tw - 24.0), mouse.y - 14.0);
            draw_rectangle(lx - 8.0, ly - 20.0, tw + 16.0, 28.0, Color::new(0.03, 0.035, 0.05, 0.95));
            ui.text(&label, lx, ly, 18, if affordable || self.tree.acquired[id] { TEXT } else { DANGER });
            if let Some(spell) = self.tree.nodes[id].spell {
                spell_details(ui, spell, Rect::new(16.0, h - 128.0, 560.0, 112.0));
            }
        }

        // Points panel (stands in for character levels: 1 point per level).
        draw_rectangle(panel.x, panel.y, panel.w, panel.h, PANEL);
        draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 1.0, BORDER);
        ui.text(&format!("Skill points: {} / {}", self.tree.remaining(), self.tree.points), panel.x + 16.0, panel.y + 32.0, 22, ACCENT);
        if ui.button(Rect::new(panel.x + 230.0, panel.y + 10.0, 36.0, 32.0), "-", self.tree.points > 0, false) {
            let points = self.tree.points - 1;
            self.tree.set_points(points);
            self.prune();
        }
        if ui.button(Rect::new(panel.x + 272.0, panel.y + 10.0, 36.0, 32.0), "+", self.tree.points < 60, false) {
            let points = self.tree.points + 1;
            self.tree.set_points(points);
        }
        ui.text("1 point per level in the full game.", panel.x + 16.0, panel.y + 58.0, 15, MUTED);
        if ui.button(Rect::new(panel.x + 16.0, panel.y + 72.0, 140.0, 34.0), "Reset (free)", self.tree.spent() > 0, false) {
            self.tree.reset();
            self.prune();
        }
        if ui.button(Rect::new(panel.x + 164.0, panel.y + 72.0, 140.0, 34.0), "Center view", true, false) {
            self.fitted = false;
        }
        if ui.button(Rect::new(panel.x + 16.0, panel.y + 114.0, 288.0, 34.0), "New character (new tree)", true, false) {
            self.new_character();
            self.say(&format!("New character: tree seed {}.", self.tree.seed));
        }
        ui.text(&format!("Tree seed {}", self.tree.seed), panel.x + 16.0, panel.y + 172.0, 15, MUTED);
        ui.text("Left click: buy · Right click: refund", panel.x + 16.0, panel.y + 190.0, 15, MUTED);
        ui.text("Drag: move · Wheel: zoom", panel.x + 16.0, panel.y + 206.0, 15, MUTED);
    }

    fn buy(&mut self, id: usize) {
        let Some(route) = self.tree.route_to(id) else {
            return;
        };
        if route.is_empty() {
            return;
        }
        if self.tree.acquire(id) {
            let spells: Vec<&str> = route.iter().filter_map(|&n| self.tree.nodes[n].spell).map(|s| s.name()).collect();
            self.say(&format!("Learned {}.", spells.join(", ")));
        } else {
            self.say(&format!("Not enough skill points: this route needs {}.", route.len()));
        }
    }

    // ---- loadout ------------------------------------------------------------

    fn spell_rect(&self, i: usize) -> Rect {
        Rect::new(COLLECTION_X + (i % 3) as f32 * 162.0, SPELLS_Y + (i / 3) as f32 * 50.0 - self.list_scroll, 154.0, 44.0)
    }

    /// The visible part of the spell collection; cards outside it are hidden.
    fn list_area() -> Rect {
        Rect::new(COLLECTION_X, SPELLS_Y - 4.0, 3.0 * 162.0, screen_height() - SPELLS_Y - 16.0)
    }

    fn right_x() -> f32 {
        screen_width() - 24.0 - 464.0
    }

    fn support_rect(i: usize) -> Rect {
        Rect::new(Self::right_x() + (i % 2) as f32 * 232.0, LIST_Y + (i / 2) as f32 * 52.0, 224.0, 46.0)
    }

    fn artifact_rect(i: usize) -> Rect {
        Rect::new(Self::right_x() + (i % 2) as f32 * 232.0, LIST_Y + 4.0 * 52.0 + 50.0 + (i / 2) as f32 * 52.0, 224.0, 46.0)
    }

    /// Width of one spell-bar slot: the bar fits between the collection and the supports.
    fn bar_unit(&self) -> f32 {
        ((Self::right_x() - 30.0 - BAR_X) / self.build.capacity as f32).min(90.0)
    }

    fn bar_slot_rects(&self) -> Vec<Rect> {
        let unit = self.bar_unit();
        let mut x = BAR_X;
        self.build
            .slots
            .iter()
            .map(|s| {
                let r = Rect::new(x, BAR_Y, s.spell.size() as f32 * unit - 8.0, BAR_H);
                x += s.spell.size() as f32 * unit;
                r
            })
            .collect()
    }

    fn socket_rect(slot: Rect) -> Rect {
        Rect::new(slot.x + 6.0, slot.bottom() - 42.0, slot.w - 12.0, 36.0)
    }

    fn artifact_slot_rects(&self) -> Vec<Rect> {
        let mut x = BAR_X;
        self.build
            .artifacts
            .iter()
            .map(|a| {
                let r = Rect::new(x, ART_Y, a.size() as f32 * ART_UNIT - 8.0, ART_H);
                x += a.size() as f32 * ART_UNIT;
                r
            })
            .collect()
    }

    fn bar_zone(&self) -> Rect {
        Rect::new(BAR_X - 24.0, BAR_Y - 34.0, self.build.capacity as f32 * self.bar_unit() + 48.0, BAR_H + 60.0)
    }

    fn artifact_zone(&self) -> Rect {
        Rect::new(BAR_X - 24.0, ART_Y - 34.0, self.build.artifact_capacity as f32 * ART_UNIT + 48.0, ART_H + 60.0)
    }

    /// Insertion index for a drop at `x` among cards laid out in `rects`.
    fn insertion(rects: &[Rect], x: f32) -> usize {
        rects.iter().filter(|r| r.center().x < x).count()
    }

    /// What is under the mouse, as a draggable item and where it lives.
    fn pick(&self, mouse: Vec2) -> Option<(Item, Source)> {
        for (i, r) in self.bar_slot_rects().into_iter().enumerate() {
            if Self::socket_rect(r).contains(mouse) {
                if let Some(s) = self.build.slots[i].support {
                    return Some((Item::Support(s), Source::Socket));
                }
            }
            if r.contains(mouse) {
                return Some((Item::Spell(self.build.slots[i].spell), Source::Bar));
            }
        }
        for (i, r) in self.artifact_slot_rects().into_iter().enumerate() {
            if r.contains(mouse) {
                return Some((Item::Artifact(self.build.artifacts[i]), Source::ArtifactBar));
            }
        }
        for (i, spell) in self.listed().into_iter().enumerate() {
            if Self::list_area().contains(mouse) && self.spell_rect(i).contains(mouse) {
                return Some((Item::Spell(spell), Source::Collection));
            }
        }
        for (i, s) in SupportKind::ALL.into_iter().enumerate() {
            if Self::support_rect(i).contains(mouse) {
                return Some((Item::Support(s), Source::Collection));
            }
        }
        for (i, a) in ArtifactKind::ALL.into_iter().enumerate() {
            if Self::artifact_rect(i).contains(mouse) {
                return Some((Item::Artifact(a), Source::Collection));
            }
        }
        None
    }

    fn drop_item(&mut self, item: Item, source: Source, mouse: Vec2) {
        match item {
            Item::Spell(spell) => {
                if self.bar_zone().contains(mouse) {
                    let index = Self::insertion(&self.bar_slot_rects(), mouse.x);
                    if !self.build.place_spell(spell, index) {
                        self.say(&format!(
                            "Not enough room: {} needs {} slot(s), {} free.",
                            spell.name(),
                            spell.size(),
                            self.build.capacity - self.build.used()
                        ));
                    }
                } else if source == Source::Bar {
                    self.build.remove_spell(spell);
                }
            }
            Item::Support(support) => {
                let target = self.bar_slot_rects().iter().position(|r| r.contains(mouse));
                match target {
                    Some(i) => {
                        let spell = self.build.slots[i].spell;
                        if !self.build.socket(i, support) {
                            self.say(&format!("{} does not fit {}.", support.name(), spell.name()));
                        }
                    }
                    None if source == Source::Socket => self.build.unsocket(support),
                    None => {}
                }
            }
            Item::Artifact(artifact) => {
                if self.artifact_zone().contains(mouse) {
                    let index = Self::insertion(&self.artifact_slot_rects(), mouse.x);
                    if !self.build.place_artifact(artifact, index) {
                        self.say(&format!(
                            "Not enough artifact capacity: {} needs {}, {} free.",
                            artifact.name(),
                            artifact.size(),
                            self.build.artifact_capacity - self.build.artifacts_used()
                        ));
                    }
                } else if source == Source::ArtifactBar {
                    self.build.remove_artifact(artifact);
                }
            }
        }
    }

    /// A click without dragging: quick-add from a collection.
    fn click_item(&mut self, item: Item, source: Source) {
        match (item, source) {
            (Item::Spell(spell), Source::Collection) if !self.build.contains(spell) => {
                let end = self.build.slots.len();
                if !self.build.place_spell(spell, end) {
                    self.say("Not enough room on the spell bar.");
                }
            }
            (Item::Artifact(a), Source::Collection) if !self.build.has_artifact(a) => {
                let end = self.build.artifacts.len();
                if !self.build.place_artifact(a, end) {
                    self.say("Not enough artifact capacity.");
                }
            }
            (Item::Support(_), Source::Collection) => self.say("Drag the support onto a spell on the bar."),
            _ => {}
        }
    }

    fn loadout_screen(&mut self, ui: &Ui) {
        let mouse: Vec2 = mouse_position().into();
        // Family filter: click a color to show only that family, again to show all.
        for (i, family) in FAMILIES.into_iter().enumerate() {
            let r = Self::filter_rect(i);
            let on = self.family_filter == Some(family);
            let c = family_color(family);
            draw_rectangle(r.x, r.y, r.w, r.h, Color::new(c.r, c.g, c.b, if on || self.family_filter.is_none() { 0.85 } else { 0.25 }));
            if on {
                draw_rectangle_lines(r.x - 2.0, r.y - 2.0, r.w + 4.0, r.h + 4.0, 2.0, WHITE);
            }
            if r.contains(mouse) {
                ui.text(family.name(), COLLECTION_X + 250.0, LIST_Y - 16.0, 14, TEXT);
                if is_mouse_button_pressed(MouseButton::Left) {
                    self.family_filter = if on { None } else { Some(family) };
                    self.list_scroll = 0.0;
                }
            }
        }
        let available = self.listed();

        // Input.
        if is_mouse_button_pressed(MouseButton::Left) && mouse.y > HEADER {
            if let Some((item, source)) = self.pick(mouse) {
                self.drag = Some(Drag { item, source, start: mouse, active: false });
            }
        }
        if let Some(drag) = &mut self.drag {
            if (mouse - drag.start).length() > DRAG_THRESHOLD {
                drag.active = true;
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            if let Some(drag) = self.drag.take() {
                if drag.active {
                    self.drop_item(drag.item, drag.source, mouse);
                } else {
                    self.click_item(drag.item, drag.source);
                }
            }
        }
        let dragging = self.drag.as_ref().filter(|d| d.active).map(|d| (d.item, d.source));
        let area = Self::list_area();
        let rows = available.len().div_ceil(3) as f32;
        let max_scroll = (rows * 50.0 - area.h).max(0.0);
        let wheel = mouse_wheel().1;
        if area.contains(mouse) && wheel != 0.0 {
            self.list_scroll -= wheel.signum() * 60.0;
        }
        self.list_scroll = self.list_scroll.clamp(0.0, max_scroll);
        let hovered = dragging.map(|d| d.0).or_else(|| self.pick(mouse).map(|p| p.0));

        // Spell collection.
        let title = if self.sandbox { format!("All spells ({})", available.len()) } else { format!("Your spells ({})", available.len()) };
        ui.text(&title, COLLECTION_X, LIST_Y - 16.0, 20, ACCENT);
        if max_scroll > 0.0 {
            ui.text("wheel: scroll", COLLECTION_X + 390.0, LIST_Y - 16.0, 14, MUTED);
            let track = Rect::new(area.right() - 2.0, area.y, 4.0, area.h);
            let knob_h = (area.h * area.h / (rows * 50.0)).max(30.0);
            let knob_y = track.y + (track.h - knob_h) * (self.list_scroll / max_scroll);
            draw_rectangle(track.x, track.y, track.w, track.h, Color::new(0.12, 0.13, 0.17, 1.0));
            draw_rectangle(track.x, knob_y, track.w, knob_h, BORDER);
        }
        if available.is_empty() {
            let hint = if self.family_filter.is_some() { "No spell of this family." } else { "Buy spells in the Skill Tree first (Tab)." };
            ui.wrapped(hint, COLLECTION_X, SPELLS_Y + 24.0, 460.0, 18, MUTED);
        }
        for (i, spell) in available.iter().enumerate() {
            let r = self.spell_rect(i);
            if r.y < area.y || r.bottom() > area.bottom() {
                continue;
            }
            let on_bar = self.build.contains(*spell);
            let hover = r.contains(mouse) && dragging.is_none();
            draw_rectangle(r.x, r.y, r.w, r.h, if hover { PANEL_HOVER } else { PANEL });
            draw_rectangle(r.x, r.y, 5.0, r.h, family_color(spell.family()));
            let color = if on_bar { MUTED } else { TEXT };
            ui.text(spell.name(), r.x + 12.0, r.y + 20.0, 16, color);
            let sub = if on_bar { "on the bar".to_string() } else { format!("{} · size {}", spell.delivery().name(), spell.size()) };
            ui.text(&sub, r.x + 12.0, r.y + 37.0, 13, if on_bar { GOOD } else { MUTED });
        }

        // Spell bar.
        ui.text(&format!("Spell bar ({} / {})", self.build.used(), self.build.capacity), BAR_X, BAR_Y - 16.0, 20, ACCENT);
        ui.text("casts left to right · Echo reads its left neighbor", BAR_X + 190.0, BAR_Y - 16.0, 14, MUTED);
        let unit = self.bar_unit();
        for k in 0..self.build.capacity {
            let r = Rect::new(BAR_X + k as f32 * unit, BAR_Y, unit - 8.0, BAR_H);
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, Color::new(0.18, 0.19, 0.24, 1.0));
        }
        let slot_rects = self.bar_slot_rects();
        for (i, r) in slot_rects.iter().enumerate() {
            let slot = self.build.slots[i];
            let lifted = dragging == Some((Item::Spell(slot.spell), Source::Bar));
            draw_rectangle(r.x, r.y, r.w, r.h, if lifted { Color::new(0.05, 0.05, 0.07, 1.0) } else { PANEL });
            draw_rectangle(r.x, r.y, r.w, 5.0, family_color(slot.spell.family()));
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, BORDER);
            let name_color = if lifted { MUTED } else { TEXT };
            ui.wrapped(slot.spell.name(), r.x + 6.0, r.y + 24.0, r.w - 10.0, 15, name_color);
            ui.text(slot.spell.family().name(), r.x + 8.0, r.y + 72.0, 13, MUTED);
            // Socket.
            let s = Self::socket_rect(*r);
            let highlight = match dragging {
                Some((Item::Support(sup), _)) => Some(if sup.fits(slot.spell) { GOOD } else { DANGER }),
                _ => None,
            };
            draw_rectangle(s.x, s.y, s.w, s.h, Color::new(0.04, 0.045, 0.06, 1.0));
            draw_rectangle_lines(s.x, s.y, s.w, s.h, if highlight.is_some() { 2.0 } else { 1.0 }, highlight.unwrap_or(BORDER));
            match slot.support {
                Some(sup) if dragging != Some((Item::Support(sup), Source::Socket)) => {
                    ui.text(sup.name(), s.x + 4.0, s.y + 23.0, 12, ACCENT)
                }
                _ => ui.text("support", s.x + 6.0, s.y + 23.0, 13, Color::new(0.35, 0.37, 0.44, 1.0)),
            }
        }
        if let Some((Item::Spell(_), _)) = dragging {
            if self.bar_zone().contains(mouse) {
                let index = Self::insertion(&slot_rects, mouse.x);
                let x = slot_rects.get(index).map_or_else(|| slot_rects.last().map_or(BAR_X, |r| r.right() + 4.0), |r| r.x - 4.0);
                draw_line(x, BAR_Y - 6.0, x, BAR_Y + BAR_H + 6.0, 3.0, GOOD);
            }
        }

        // Artifact bar.
        ui.text(
            &format!("Artifact bar ({} / {})", self.build.artifacts_used(), self.build.artifact_capacity),
            BAR_X,
            ART_Y - 16.0,
            20,
            ACCENT,
        );
        for k in 0..self.build.artifact_capacity {
            let r = Rect::new(BAR_X + k as f32 * ART_UNIT, ART_Y, ART_UNIT - 8.0, ART_H);
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, Color::new(0.18, 0.19, 0.24, 1.0));
        }
        let art_rects = self.artifact_slot_rects();
        for (i, r) in art_rects.iter().enumerate() {
            let a = self.build.artifacts[i];
            let lifted = dragging == Some((Item::Artifact(a), Source::ArtifactBar));
            draw_rectangle(r.x, r.y, r.w, r.h, if lifted { Color::new(0.05, 0.05, 0.07, 1.0) } else { Color::new(0.12, 0.1, 0.07, 1.0) });
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, Color::new(0.5, 0.4, 0.2, 1.0));
            ui.wrapped(a.name(), r.x + 8.0, r.y + 24.0, r.w - 16.0, 15, if lifted { MUTED } else { TEXT });
            ui.text(&format!("size {}", a.size()), r.x + 8.0, r.bottom() - 10.0, 12, MUTED);
        }
        if let Some((Item::Artifact(_), _)) = dragging {
            if self.artifact_zone().contains(mouse) {
                let index = Self::insertion(&art_rects, mouse.x);
                let x = art_rects.get(index).map_or_else(|| art_rects.last().map_or(BAR_X, |r| r.right() + 4.0), |r| r.x - 4.0);
                draw_line(x, ART_Y - 6.0, x, ART_Y + ART_H + 6.0, 3.0, GOOD);
            }
        }

        // Details of what is hovered or dragged.
        let details = Rect::new(BAR_X, ART_Y + ART_H + 40.0, 540.0, 150.0);
        match hovered {
            Some(Item::Spell(s)) => spell_details(ui, s, details),
            Some(Item::Support(s)) => item_details(ui, s.name(), "Support · once per build", s.description(), details),
            Some(Item::Artifact(a)) => {
                item_details(ui, a.name(), &format!("Artifact · size {}", a.size()), a.description(), details)
            }
            None => item_details(
                ui,
                "Drag and drop",
                "Spells go on the spell bar, supports in a spell's socket, artifacts on the artifact bar.",
                "Click a spell or artifact to add it at the end. Drag a card off its bar to remove it. Drag cards along a bar to reorder them.",
                details,
            ),
        }

        // Sandbox and ready-made builds.
        let py = details.bottom() + 30.0;
        if ui.button(Rect::new(BAR_X, py, 260.0, 36.0), if self.sandbox { "Sandbox: ON (all spells)" } else { "Sandbox: OFF (tree spells)" }, true, self.sandbox) {
            self.sandbox = !self.sandbox;
            self.prune();
        }
        ui.text("Ready builds (sandbox):", BAR_X, py + 70.0, 16, MUTED);
        for (k, (name, preset)) in presets().into_iter().enumerate() {
            let r = Rect::new(BAR_X + k as f32 * 134.0, py + 82.0, 126.0, 34.0);
            if ui.button(r, name, true, self.sandbox && self.build == preset) {
                self.load_preset(preset);
            }
        }

        // Supports.
        let rx = Self::right_x();
        ui.text("Supports", rx, LIST_Y - 16.0, 20, ACCENT);
        ui.text("drag onto a spell's socket", rx + 100.0, LIST_Y - 16.0, 14, MUTED);
        for (i, s) in SupportKind::ALL.into_iter().enumerate() {
            let r = Self::support_rect(i);
            let used_on = self.build.slots.iter().find(|slot| slot.support == Some(s)).map(|slot| slot.spell);
            let hover = r.contains(mouse) && dragging.is_none();
            draw_rectangle(r.x, r.y, r.w, r.h, if hover { PANEL_HOVER } else { PANEL });
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, BORDER);
            ui.text(s.name(), r.x + 12.0, r.y + 21.0, 17, if used_on.is_some() { MUTED } else { TEXT });
            let sub = used_on.map_or_else(|| "available".to_string(), |spell| format!("on {}", spell.name()));
            ui.text(&sub, r.x + 12.0, r.y + 39.0, 13, if used_on.is_some() { GOOD } else { MUTED });
        }

        // Artifacts.
        let ay = Self::artifact_rect(0).y;
        ui.text("Artifacts", rx, ay - 16.0, 20, ACCENT);
        ui.text("drag onto the artifact bar", rx + 100.0, ay - 16.0, 14, MUTED);
        for (i, a) in ArtifactKind::ALL.into_iter().enumerate() {
            let r = Self::artifact_rect(i);
            let equipped = self.build.has_artifact(a);
            let hover = r.contains(mouse) && dragging.is_none();
            draw_rectangle(r.x, r.y, r.w, r.h, if hover { PANEL_HOVER } else { Color::new(0.1, 0.09, 0.07, 0.95) });
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, Color::new(0.35, 0.3, 0.2, 1.0));
            ui.text(a.name(), r.x + 12.0, r.y + 21.0, 17, if equipped { MUTED } else { TEXT });
            let sub = if equipped { "equipped".to_string() } else { format!("size {}", a.size()) };
            ui.text(&sub, r.x + 12.0, r.y + 39.0, 13, if equipped { GOOD } else { MUTED });
        }

        // The card under the cursor while dragging.
        if let Some((item, _)) = dragging {
            let (name, color) = match item {
                Item::Spell(s) => (s.name(), family_color(s.family())),
                Item::Support(s) => (s.name(), ACCENT),
                Item::Artifact(a) => (a.name(), Color::new(0.8, 0.6, 0.3, 1.0)),
            };
            let r = Rect::new(mouse.x - 80.0, mouse.y - 22.0, 160.0, 44.0);
            draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.14, 0.15, 0.2, 0.92));
            draw_rectangle(r.x, r.y, 5.0, r.h, color);
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, color);
            ui.text(name, r.x + 12.0, r.y + 28.0, 17, TEXT);
        }
    }
}

fn item_details(ui: &Ui, title: &str, kind: &str, description: &str, r: Rect) {
    draw_rectangle(r.x, r.y, r.w, r.h, PANEL);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, BORDER);
    ui.text(title, r.x + 16.0, r.y + 32.0, 22, TEXT);
    ui.text(kind, r.x + 16.0, r.y + 56.0, 15, ACCENT);
    ui.wrapped(description, r.x + 16.0, r.y + 84.0, r.w - 32.0, 16, MUTED);
}

fn spell_details(ui: &Ui, spell: SpellId, r: Rect) {
    let cooldown = if spell.casts() { format!("{:.2} s cooldown", spell.cooldown()) } else { "always on".to_string() };
    let kind = format!("{} · {} · size {} · {}", spell.family().name(), spell.delivery().name(), spell.size(), cooldown);
    item_details(ui, spell.name(), &kind, &spell.description(), r);
}
