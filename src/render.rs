//! Procedural look: silhouettes, code-driven motion, and all the spectacle in
//! effects. Everything here reads the simulation; nothing writes back to it.
use arpg_machine::content::{Family, Loot};
use arpg_machine::math::V2;
use arpg_machine::rng::Rng;
use arpg_machine::world::{BossKind, EnemyKind, Event, Phase, ShotStyle, ShrineKind, World, PLAYER_RADIUS, WARD_RADIUS};
use macroquad::prelude::*;

const MAX_PARTICLES: usize = 6000;
/// World units visible across the screen width.
pub const VIEW_WIDTH: f32 = 1500.0;

pub const BACKGROUND: Color = Color::new(0.035, 0.04, 0.055, 1.0);
const FLOOR: Color = Color::new(0.06, 0.068, 0.09, 1.0);
const GRID_LINE: Color = Color::new(0.11, 0.12, 0.16, 1.0);

pub fn family_color(family: Family) -> Color {
    let (r, g, b) = match family {
        Family::Arcane => (0.74, 0.47, 1.0),
        Family::Fire => (1.0, 0.52, 0.15),
        Family::Frost => (0.55, 0.9, 1.0),
        Family::Lightning => (1.0, 0.95, 0.45),
        Family::Poison => (0.5, 0.95, 0.32),
        Family::Blood => (0.92, 0.18, 0.28),
        Family::Shadow => (0.55, 0.42, 0.72),
        Family::Light => (1.0, 0.93, 0.7),
        Family::Nature => (0.6, 0.78, 0.3),
        Family::Steel => (0.84, 0.87, 0.94),
        Family::Earth => (0.75, 0.56, 0.36),
        Family::Time => (0.35, 0.72, 1.0),
        Family::Spirits => (0.6, 0.98, 0.86),
        Family::Alchemy => (0.82, 0.92, 0.2),
        Family::Astral => (0.6, 0.62, 1.0),
    };
    Color::new(r, g, b, 1.0)
}

fn enemy_color(kind: EnemyKind) -> Color {
    match kind {
        EnemyKind::Grunt => Color::new(0.78, 0.2, 0.3, 1.0),
        EnemyKind::Runner => Color::new(0.96, 0.43, 0.26, 1.0),
        EnemyKind::Caster => Color::new(0.22, 0.8, 0.74, 1.0),
        EnemyKind::Brute => Color::new(0.6, 0.32, 0.9, 1.0),
        EnemyKind::Boss => Color::new(0.95, 0.38, 0.18, 1.0),
        EnemyKind::Add => Color::new(0.64, 0.36, 0.38, 1.0),
    }
}

/// Each boss has its own color.
pub fn boss_color(kind: BossKind) -> Color {
    match kind {
        BossKind::EmberWarden => Color::new(0.95, 0.38, 0.18, 1.0),
        BossKind::IronJuggernaut => Color::new(0.62, 0.7, 0.8, 1.0),
        BossKind::HollowOracle => Color::new(0.7, 0.45, 1.0, 1.0),
    }
}

fn shot_color(style: ShotStyle) -> Color {
    match style {
        ShotStyle::Orb => enemy_color(EnemyKind::Caster),
        ShotStyle::Ember => Color::new(1.0, 0.45, 0.15, 1.0),
        ShotStyle::Iron => Color::new(0.8, 0.86, 0.95, 1.0),
        ShotStyle::Void => Color::new(0.78, 0.4, 1.0, 1.0),
    }
}

/// A kill streak ends after this long without a kill.
const STREAK_GAP: f32 = 1.5;

/// The card's size as short bars sitting on its top edge, right-aligned: one
/// bar per size.
pub fn size_pips(r: Rect, size: u8, scale: f32) {
    let (w, h, gap) = (12.0 * scale, 4.0 * scale, 3.0 * scale);
    let total = size as f32 * w + size.saturating_sub(1) as f32 * gap;
    let x0 = r.right() - 8.0 * scale - total;
    let y = r.y - h / 2.0;
    for k in 0..size {
        let x = x0 + k as f32 * (w + gap);
        draw_rectangle(x - 1.5, y - 1.5, w + 3.0, h + 3.0, Color::new(0.02, 0.02, 0.03, 1.0));
        draw_rectangle(x, y, w, h, Color::new(0.95, 0.95, 0.99, 1.0));
    }
}

pub const SIDE_ARROW: Color = Color::new(0.45, 0.82, 1.0, 1.0);

/// Small arrows on a card's edges: this spell charges, hastes or empowers the
/// neighbor on that side of the bar.
pub fn side_arrows(r: Rect, sides: (bool, bool), scale: f32) {
    // Drawn just outside the edge, pointing at the neighbor, so they never cover text.
    let size = 6.0 * scale;
    let y = r.y + r.h / 2.0;
    if sides.0 {
        let base = r.x - 1.5 * scale;
        draw_triangle(vec2(base - size, y), vec2(base, y - size), vec2(base, y + size), SIDE_ARROW);
    }
    if sides.1 {
        let base = r.right() + 1.5 * scale;
        draw_triangle(vec2(base + size, y), vec2(base, y - size), vec2(base, y + size), SIDE_ARROW);
    }
}

pub fn loot_color(loot: Loot) -> Color {
    match loot {
        Loot::Support(_) => Color::new(1.0, 0.84, 0.35, 1.0),
        Loot::Artifact(_, tier) => tier_color(tier),
    }
}

/// Artifact tiers, ARPG-style: white, green, blue, purple, legendary orange.
pub fn tier_color(tier: u8) -> Color {
    match tier {
        0 | 1 => Color::new(0.85, 0.82, 0.76, 1.0),
        2 => Color::new(0.45, 0.85, 0.4, 1.0),
        3 => Color::new(0.35, 0.6, 1.0, 1.0),
        4 => Color::new(0.72, 0.45, 1.0, 1.0),
        _ => Color::new(1.0, 0.55, 0.15, 1.0),
    }
}

pub fn shrine_color(kind: ShrineKind) -> Color {
    match kind {
        ShrineKind::Fury => Color::new(1.0, 0.35, 0.3, 1.0),
        ShrineKind::Swiftness => Color::new(0.4, 1.0, 0.7, 1.0),
        ShrineKind::Haste => Color::new(0.4, 0.75, 1.0, 1.0),
        ShrineKind::Warding => Color::new(0.8, 0.85, 1.0, 1.0),
        ShrineKind::Vitality => Color::new(0.45, 1.0, 0.45, 1.0),
        ShrineKind::Ancient => Color::new(1.0, 0.82, 0.3, 1.0),
    }
}

/// Boss attacks read as fire; a Brute slam uses the Brute's own color.
fn telegraph_color(owner: EnemyKind) -> Color {
    match owner {
        EnemyKind::Brute => Color::new(0.75, 0.45, 1.0, 1.0),
        _ => Color::new(1.0, 0.3, 0.12, 1.0),
    }
}

fn v(p: V2) -> Vec2 {
    vec2(p.x, p.y)
}

fn with_alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, a.a + (b.a - a.a) * t)
}

struct Particle {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    max: f32,
    size: f32,
    color: Color,
    drag: f32,
}

enum Shape {
    Ring { pos: Vec2, from: f32, to: f32, color: Color, width: f32 },
    Disc { pos: Vec2, radius: f32, color: Color },
    Cone { pos: Vec2, dir: Vec2, range: f32, half_angle: f32, color: Color },
    Bolt { points: Vec<Vec2>, color: Color },
    Line { a: Vec2, b: Vec2, color: Color },
    Ghost { pos: Vec2 },
}

/// A soul freed by a kill, drawn in toward you.
struct Wisp {
    pos: Vec2,
    vel: Vec2,
    color: Color,
    age: f32,
}

/// A damage number rising off the boss.
struct Number {
    pos: Vec2,
    value: f32,
    life: f32,
}

struct Transient {
    shape: Shape,
    life: f32,
    max: f32,
}

/// Visual-only state fed by simulation events.
pub struct Fx {
    particles: Vec<Particle>,
    transients: Vec<Transient>,
    flash: Vec<f32>,
    pub cast_pulse: Vec<f32>,
    shake: f32,
    pub player_hurt: f32,
    /// Brightens you as souls reach you.
    player_glow: f32,
    /// A boss mechanic just started: its text and how long to show it.
    pub boss_banner: Option<(String, f32)>,
    barrier_flash: f32,
    heal_flash: f32,
    /// Name of the shrine just claimed, and how long to show it.
    pub shrine_banner: Option<(ShrineKind, f32)>,
    pub loot_banner: Option<(Loot, f32)>,
    wisps: Vec<Wisp>,
    numbers: Vec<Number>,
    /// Boss health at the last frame, and damage gathered since the last number.
    boss_hp: Option<f32>,
    boss_damage: (f32, f32),
    /// Kills in a row with less than `STREAK_GAP` between them.
    pub streak: u32,
    streak_timer: f32,
    pub streak_pop: f32,
    /// Seconds of slow motion left (boss death), for the main loop.
    pub slow_motion: f32,
    rng: Rng,
    time: f32,
}

impl Fx {
    pub fn new() -> Self {
        Fx {
            particles: Vec::new(),
            transients: Vec::new(),
            flash: Vec::new(),
            cast_pulse: Vec::new(),
            shake: 0.0,
            player_hurt: 0.0,
            player_glow: 0.0,
            boss_banner: None,
            barrier_flash: 0.0,
            heal_flash: 0.0,
            shrine_banner: None,
            loot_banner: None,
            wisps: Vec::new(),
            numbers: Vec::new(),
            boss_hp: None,
            boss_damage: (0.0, 0.0),
            streak: 0,
            streak_timer: 0.0,
            streak_pop: 0.0,
            slow_motion: 0.0,
            rng: Rng::new(0xF00D),
            time: 0.0,
        }
    }

    fn burst(&mut self, pos: Vec2, count: usize, speed: f32, size: f32, life: f32, color: Color) {
        for _ in 0..count {
            if self.particles.len() >= MAX_PARTICLES {
                return;
            }
            let angle = self.rng.range(0.0, std::f32::consts::TAU);
            let s = speed * self.rng.range(0.3, 1.0);
            let l = life * self.rng.range(0.6, 1.0);
            self.particles.push(Particle {
                pos,
                vel: vec2(angle.cos(), angle.sin()) * s,
                life: l,
                max: l,
                size: size * self.rng.range(0.6, 1.2),
                color,
                drag: 4.0,
            });
        }
    }

    fn transient(&mut self, shape: Shape, life: f32) {
        self.transients.push(Transient { shape, life, max: life });
    }

    pub fn consume(&mut self, world: &World) {
        if self.flash.len() < world.enemies.len() {
            self.flash.resize(world.enemies.len(), 0.0);
        }
        if self.cast_pulse.len() != world.slot_count() {
            self.cast_pulse = vec![0.0; world.slot_count()];
        }
        for event in &world.events {
            match event {
                Event::Cast { slot } => self.cast_pulse[*slot] = 0.25,
                Event::Hit { enemy, family } => {
                    self.flash[*enemy] = 0.09;
                    if self.rng.chance(0.25) {
                        let pos = v(world.enemies[*enemy].pos);
                        self.burst(pos, 1, 120.0, 2.5, 0.25, family_color(*family));
                    }
                }
                Event::Death { pos, kind, family, radius } => {
                    let pos = v(*pos);
                    let n = 8 + (*radius * 0.6) as usize;
                    self.burst(pos, n, 260.0 + radius * 4.0, 3.5, 0.55, enemy_color(*kind));
                    self.burst(pos, n / 2 + 2, 200.0, 3.0, 0.45, family_color(*family));
                    self.transient(
                        Shape::Ring { pos, from: radius * 0.6, to: radius * 1.8, color: family_color(*family), width: 2.0 },
                        0.2,
                    );
                    let souls = if *kind == EnemyKind::Boss { 60 } else if *kind == EnemyKind::Brute { 4 } else { 1 };
                    for _ in 0..souls {
                        if self.wisps.len() < 260 {
                            let dir = Vec2::from_angle(self.rng.range(0.0, std::f32::consts::TAU));
                            let speed = self.rng.range(80.0, 220.0) * if *kind == EnemyKind::Boss { 3.0 } else { 1.0 };
                            self.wisps.push(Wisp { pos, vel: dir * speed, color: mix(enemy_color(*kind), WHITE, 0.5), age: 0.0 });
                        }
                    }
                    if *kind != EnemyKind::Boss {
                        self.streak += 1;
                        self.streak_timer = STREAK_GAP;
                        self.streak_pop = 0.12;
                    }
                    if *kind == EnemyKind::Boss {
                        self.slow_motion = 1.0;
                        self.burst(pos, 240, 700.0, 6.0, 1.4, boss_color(world.boss_def.kind));
                        self.shake += 18.0;
                    }
                }
                Event::Nova { pos, radius, family } => {
                    let pos = v(*pos);
                    let color = family_color(*family);
                    self.transient(Shape::Ring { pos, from: radius * 0.25, to: *radius, color, width: 6.0 }, 0.28);
                    self.transient(Shape::Disc { pos, radius: *radius, color: with_alpha(color, 0.12) }, 0.2);
                    self.burst(pos, 18, radius * 3.2, 3.0, 0.35, color);
                }
                Event::Cone { pos, dir, range, half_angle, family } => {
                    let (pos, dir) = (v(*pos), v(*dir));
                    let color = family_color(*family);
                    self.transient(Shape::Cone { pos, dir, range: *range, half_angle: *half_angle, color }, 0.2);
                    for _ in 0..10 {
                        if self.particles.len() >= MAX_PARTICLES {
                            break;
                        }
                        let a = self.rng.range(-half_angle, *half_angle);
                        let d = Vec2::from_angle(a).rotate(dir);
                        let s = range * self.rng.range(1.5, 3.0);
                        self.particles.push(Particle {
                            pos,
                            vel: d * s,
                            life: 0.35,
                            max: 0.35,
                            size: 4.0,
                            color,
                            drag: 5.0,
                        });
                    }
                }
                Event::Chain { points, family } => {
                    let color = family_color(*family);
                    let pts: Vec<Vec2> = points.iter().map(|p| v(*p)).collect();
                    let mut jagged = Vec::new();
                    for pair in pts.windows(2) {
                        let (a, b) = (pair[0], pair[1]);
                        let normal = (b - a).perp().normalize_or_zero();
                        jagged.push(a);
                        for k in 1..5 {
                            let t = k as f32 / 5.0;
                            jagged.push(a.lerp(b, t) + normal * self.rng.range(-14.0, 14.0));
                        }
                    }
                    if let Some(&last) = pts.last() {
                        jagged.push(last);
                    }
                    self.transient(Shape::Bolt { points: jagged, color }, 0.16);
                    for p in pts.iter().skip(1) {
                        self.burst(*p, 4, 160.0, 2.5, 0.25, color);
                    }
                }
                Event::Explosion { pos, radius } => {
                    let pos = v(*pos);
                    let fire = family_color(Family::Fire);
                    self.transient(Shape::Disc { pos, radius: *radius, color: with_alpha(fire, 0.35) }, 0.22);
                    self.transient(Shape::Ring { pos, from: radius * 0.3, to: *radius, color: fire, width: 4.0 }, 0.25);
                    self.burst(pos, 14, 380.0, 4.0, 0.5, fire);
                    self.shake = (self.shake + 1.5).min(10.0);
                }
                Event::Spread { from, to } => {
                    self.transient(Shape::Line { a: v(*from), b: v(*to), color: family_color(Family::Poison) }, 0.3);
                }
                Event::PlayerHurt { amount } => {
                    self.player_hurt = 0.2;
                    if *amount > 10.0 {
                        self.shake = (self.shake + amount * 0.15).min(14.0);
                    }
                }
                Event::Dash { from, dir } => {
                    for k in 0..5 {
                        let pos = v(*from) + v(*dir) * (k as f32 * 60.0);
                        self.transient(Shape::Ghost { pos }, 0.18 + k as f32 * 0.04);
                    }
                }
                Event::BarrierHit => self.barrier_flash = 0.15,
                Event::Compacted => self.flash.iter_mut().for_each(|f| *f = 0.0),
                Event::LootDropped { pos, loot } => {
                    let pos = v(*pos);
                    let color = loot_color(*loot);
                    self.transient(Shape::Ring { pos, from: 10.0, to: 120.0, color, width: 5.0 }, 0.6);
                    self.burst(pos, 40, 300.0, 4.0, 0.9, color);
                }
                Event::LootPicked { loot } => {
                    let pos = v(world.player.pos);
                    self.burst(pos, 30, 260.0, 3.5, 0.6, loot_color(*loot));
                    self.loot_banner = Some((*loot, 2.5));
                }
                Event::ShrineClaimed { pos, kind } => {
                    let pos = v(*pos);
                    let color = shrine_color(*kind);
                    self.transient(Shape::Ring { pos, from: 20.0, to: 220.0, color, width: 6.0 }, 0.5);
                    self.burst(pos, 50, 420.0, 4.5, 0.8, color);
                    self.shrine_banner = Some((*kind, 2.2));
                }
                Event::Heal => self.heal_flash = 0.3,
                Event::SelfCast { family } => {
                    let pos = v(world.player.pos);
                    let color = family_color(*family);
                    self.transient(Shape::Ring { pos, from: 10.0, to: 60.0, color, width: 3.0 }, 0.3);
                    self.burst(pos, 10, 160.0, 3.0, 0.4, color);
                }
                Event::Blast { pos, radius, family } => {
                    let pos = v(*pos);
                    let color = family_color(*family);
                    self.transient(Shape::Disc { pos, radius: *radius, color: with_alpha(color, 0.3) }, 0.22);
                    self.transient(Shape::Ring { pos, from: radius * 0.4, to: *radius, color, width: 4.0 }, 0.25);
                    self.burst(pos, 12, radius * 2.5, 3.5, 0.4, color);
                }
                Event::EnemyFire { pos } => {
                    self.burst(v(*pos), 6, 140.0, 3.0, 0.3, enemy_color(EnemyKind::Caster));
                }
                Event::Detonate { pos, radius, owner } => {
                    let pos = v(*pos);
                    let fire = telegraph_color(*owner);
                    self.transient(Shape::Disc { pos, radius: *radius, color: with_alpha(fire, 0.5) }, 0.3);
                    self.burst(pos, 24, 420.0, 5.0, 0.6, fire);
                    self.shake = (self.shake + 5.0).min(14.0);
                }
                Event::BossSpawned { pos } => {
                    let pos = v(*pos);
                    let color = boss_color(world.boss_def.kind);
                    self.transient(Shape::Ring { pos, from: 20.0, to: 260.0, color, width: 8.0 }, 0.6);
                    self.burst(pos, 80, 500.0, 5.0, 0.9, color);
                    self.shake += 10.0;
                }
                Event::BossAlert { pos, text } => {
                    let pos = v(*pos);
                    let color = boss_color(world.boss_def.kind);
                    self.transient(Shape::Ring { pos, from: 40.0, to: 320.0, color, width: 10.0 }, 0.7);
                    self.boss_banner = Some((text.clone(), 2.5));
                    self.shake += 8.0;
                }
                Event::ShotHit { pos } => {
                    self.burst(v(*pos), 10, 220.0, 3.0, 0.3, WHITE);
                    self.shake = (self.shake + 3.0).min(14.0);
                }
            }
        }
    }

    /// Ambient effects that come from ongoing state rather than events.
    pub fn ambient(&mut self, world: &World, dt: f32) {
        // Souls fly out, then curve into you.
        let target = v(world.player.pos);
        for w in &mut self.wisps {
            w.age += dt;
            let to = target - w.pos;
            let pull = 900.0 + 2600.0 * w.age;
            w.vel += to.normalize_or_zero() * pull * dt;
            w.vel *= (1.0 - 3.0 * dt).max(0.0);
            w.pos += w.vel * dt;
        }
        let before = self.wisps.len();
        self.wisps.retain(|w| w.pos.distance(target) > 16.0 && w.age < 3.0);
        if self.wisps.len() < before {
            self.player_glow = (self.player_glow + 0.04 * (before - self.wisps.len()) as f32).min(0.4);
        }
        // Damage numbers off the boss, gathered over a short window.
        if let Some(b) = world.boss.as_ref().filter(|b| world.enemies.get(b.index).is_some_and(|e| e.kind == EnemyKind::Boss)) {
            let e = &world.enemies[b.index];
            let hp = e.hp.max(0.0);
            if let Some(last) = self.boss_hp {
                if hp < last {
                    self.boss_damage.0 += last - hp;
                }
            }
            self.boss_hp = Some(hp);
            self.boss_damage.1 += dt;
            if self.boss_damage.1 > 0.18 {
                if self.boss_damage.0 >= 1.0 {
                    let offset = vec2(self.rng.range(-40.0, 40.0), self.rng.range(-20.0, 10.0));
                    self.numbers.push(Number { pos: v(e.pos) + offset - vec2(0.0, e.radius), value: self.boss_damage.0, life: 0.9 });
                }
                self.boss_damage = (0.0, 0.0);
            }
        } else {
            self.boss_hp = None;
        }
        if world.momentum > 0 && self.rng.chance(world.momentum as f32 * dt * 2.0) {
            let pos = v(world.player.pos) + Vec2::from_angle(self.rng.range(0.0, std::f32::consts::TAU)) * 20.0;
            self.particles.push(Particle {
                pos,
                vel: vec2(self.rng.range(-20.0, 20.0), -90.0),
                life: 0.5,
                max: 0.5,
                size: 3.0,
                color: Color::new(1.0, 0.8, 0.3, 1.0),
                drag: 1.0,
            });
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;
        for n in &mut self.numbers {
            n.pos.y -= 60.0 * dt;
            n.life -= dt;
        }
        self.numbers.retain(|n| n.life > 0.0);
        self.streak_timer -= dt;
        if self.streak_timer <= 0.0 {
            self.streak = 0;
        }
        self.streak_pop = (self.streak_pop - dt).max(0.0);
        self.slow_motion = (self.slow_motion - dt).max(0.0);
        self.player_glow = (self.player_glow - dt).max(0.0);
        for p in &mut self.particles {
            p.pos += p.vel * dt;
            p.vel *= (1.0 - p.drag * dt).max(0.0);
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);
        for t in &mut self.transients {
            t.life -= dt;
        }
        self.transients.retain(|t| t.life > 0.0);
        for f in &mut self.flash {
            *f = (*f - dt).max(0.0);
        }
        for c in &mut self.cast_pulse {
            *c = (*c - dt).max(0.0);
        }
        self.shake = (self.shake - dt * 40.0).max(0.0);
        self.player_hurt = (self.player_hurt - dt).max(0.0);
        if let Some((_, left)) = &mut self.boss_banner {
            *left -= dt;
            if *left <= 0.0 {
                self.boss_banner = None;
            }
        }
        self.barrier_flash = (self.barrier_flash - dt).max(0.0);
        self.heal_flash = (self.heal_flash - dt).max(0.0);
        if let Some((_, left)) = &mut self.shrine_banner {
            *left -= dt;
            if *left <= 0.0 {
                self.shrine_banner = None;
            }
        }
        if let Some((_, left)) = &mut self.loot_banner {
            *left -= dt;
            if *left <= 0.0 {
                self.loot_banner = None;
            }
        }
    }

    pub fn camera(&mut self, world: &World) -> Camera2D {
        let view_h = VIEW_WIDTH * screen_height() / screen_width();
        let jitter = if self.shake > 0.0 {
            vec2(self.rng.range(-1.0, 1.0), self.rng.range(-1.0, 1.0)) * self.shake
        } else {
            Vec2::ZERO
        };
        Camera2D::from_display_rect(Rect::new(
            world.player.pos.x - VIEW_WIDTH / 2.0 + jitter.x,
            world.player.pos.y - view_h / 2.0 + jitter.y,
            VIEW_WIDTH,
            view_h,
        ))
    }
}

/// Screen space is y-down; macroquad's display-rect camera is y-up, so flip it.
pub fn fix_camera(mut camera: Camera2D) -> Camera2D {
    camera.zoom.y = -camera.zoom.y;
    camera
}

pub fn draw_world(world: &World, fx: &Fx, camera: &Camera2D) {
    set_camera(camera);
    let view_h = VIEW_WIDTH * screen_height() / screen_width();
    let center = v(world.player.pos);
    let view = Rect::new(center.x - VIEW_WIDTH / 2.0 - 80.0, center.y - view_h / 2.0 - 80.0, VIEW_WIDTH + 160.0, view_h + 160.0);
    let t = fx.time;

    // Floor and a faint grid, so movement reads on the endless map.
    draw_rectangle(view.x, view.y, view.w, view.h, FLOOR);
    let step = 160.0;
    let mut x = (view.x / step).floor() * step;
    while x < view.right() {
        draw_line(x, view.y, x, view.bottom(), 1.0, GRID_LINE);
        x += step;
    }
    let mut y = (view.y / step).floor() * step;
    while y < view.bottom() {
        draw_line(view.x, y, view.right(), y, 1.0, GRID_LINE);
        y += step;
    }

    // Loot on the ground: a beam you can see from afar.
    for drop in world.drops.iter().filter(|d| !d.picked) {
        let p = v(drop.pos);
        if !view.contains(p) {
            continue;
        }
        let c = loot_color(drop.loot);
        let pulse = 0.5 + 0.5 * (t * 4.0).sin();
        draw_rectangle(p.x - 5.0, p.y - 140.0, 10.0, 140.0, with_alpha(c, 0.18 + 0.12 * pulse));
        draw_rectangle(p.x - 2.0, p.y - 140.0, 4.0, 140.0, with_alpha(c, 0.5));
        draw_circle(p.x, p.y, 22.0 + 4.0 * pulse, with_alpha(c, 0.18));
        draw_poly(p.x, p.y, 4, 12.0, 45.0 + t * 90.0, c);
        draw_poly_lines(p.x, p.y, 4, 16.0, 45.0 - t * 60.0, 2.0, WHITE);
    }

    // Shrines.
    for shrine in &world.shrines {
        let p = v(shrine.pos);
        if !view.contains(p) {
            continue;
        }
        let c = shrine_color(shrine.kind);
        let pulse = 0.5 + 0.5 * (t * 3.0 + shrine.id as f32).sin();
        draw_circle(p.x, p.y, 46.0 + 6.0 * pulse, with_alpha(c, 0.12));
        draw_circle_lines(p.x, p.y, 40.0, 2.0, with_alpha(c, 0.5 + 0.4 * pulse));
        draw_poly(p.x, p.y, 4, 18.0, 45.0 + t * 40.0, c);
        draw_poly_lines(p.x, p.y, 4, 24.0, 45.0 - t * 30.0, 2.0, with_alpha(WHITE, 0.6));
        let beam = 60.0 + 20.0 * pulse;
        draw_rectangle(p.x - 3.0, p.y - beam, 6.0, beam - 20.0, with_alpha(c, 0.35));
    }

    // Ground layer.
    for zone in &world.zones {
        // A pending strike: a ring that fills up until the next tick lands.
        let p = v(zone.pos);
        let c = family_color(zone.family);
        let fill = 1.0 - (zone.timer / zone.interval).clamp(0.0, 1.0);
        draw_circle(p.x, p.y, zone.radius * fill, with_alpha(c, 0.12));
        draw_circle_lines(p.x, p.y, zone.radius, 2.0, with_alpha(c, 0.5 + 0.3 * (t * 8.0).sin().abs()));
    }
    // The Oracle's circle of light: bright when you stand in it.
    if let Some(ward) = world.boss.as_ref().and_then(|b| b.ward) {
        let p = v(ward);
        let inside = world.player.pos.dist(ward) <= WARD_RADIUS;
        let gold = Color::new(1.0, 0.88, 0.5, 1.0);
        draw_circle(p.x, p.y, WARD_RADIUS, with_alpha(gold, if inside { 0.16 } else { 0.07 + 0.05 * (t * 4.0).sin().abs() }));
        draw_circle_lines(p.x, p.y, WARD_RADIUS, if inside { 4.0 } else { 2.5 }, with_alpha(gold, if inside { 0.9 } else { 0.55 }));
        for k in 0..12 {
            let a = t * 0.6 + k as f32 * std::f32::consts::TAU / 12.0;
            let q = p + Vec2::from_angle(a) * WARD_RADIUS;
            draw_circle(q.x, q.y, 3.0, with_alpha(gold, 0.8));
        }
    }
    for tele in &world.telegraphs {
        let p = v(tele.pos);
        let progress = (tele.elapsed / tele.warning).clamp(0.0, 1.0);
        let c = if tele.owner == EnemyKind::Boss { boss_color(world.boss_def.kind) } else { telegraph_color(tele.owner) };
        let blink = 0.5 + 0.4 * (t * 18.0).sin().abs();
        match tele.line {
            None => {
                draw_circle(p.x, p.y, tele.radius * progress, with_alpha(c, 0.22));
                draw_circle_lines(p.x, p.y, tele.radius, 3.0, with_alpha(c, blink));
            }
            Some((dir, length)) => {
                let d = v(dir);
                let end = p + d * length;
                let side = d.perp() * tele.radius;
                draw_line(p.x, p.y, end.x, end.y, tele.radius * 2.0, with_alpha(c, 0.1));
                let fill = p + d * (length * progress);
                draw_line(p.x, p.y, fill.x, fill.y, tele.radius * 2.0, with_alpha(c, 0.22));
                for s in [side, -side] {
                    draw_line(p.x + s.x, p.y + s.y, end.x + s.x, end.y + s.y, 3.0, with_alpha(c, blink));
                }
            }
        }
    }
    if let Phase::BossIncoming { timer, pos } = world.phase {
        let p = v(pos);
        let pulse = (t * 10.0).sin().abs();
        let c = boss_color(world.boss_def.kind);
        draw_circle_lines(p.x, p.y, 60.0 + timer * 60.0, 4.0, with_alpha(c, 0.4 + 0.5 * pulse));
        draw_circle(p.x, p.y, 30.0, with_alpha(c, 0.2 + 0.2 * pulse));
    }

    // Orbit trails under the bodies.
    for orbit in &world.orbits {
        draw_circle_lines(center.x, center.y, orbit.radius, 10.0, with_alpha(family_color(orbit.family), 0.05));
    }

    for (i, e) in world.enemies.iter().enumerate() {
        if !e.alive || !view.contains(v(e.pos)) {
            continue;
        }
        draw_enemy(world, e, fx.flash.get(i).copied().unwrap_or(0.0), t);
    }

    for p in &world.projectiles {
        let pos = v(p.pos);
        let tail = pos - v(p.vel) * 0.035;
        let c = family_color(p.family);
        draw_line(tail.x, tail.y, pos.x, pos.y, 9.0, with_alpha(c, 0.25));
        draw_line(tail.x, tail.y, pos.x, pos.y, 4.0, c);
        draw_circle(pos.x, pos.y, 4.0, WHITE);
    }

    for shot in &world.enemy_shots {
        let pos = v(shot.pos);
        let tail = pos - v(shot.vel) * 0.08;
        let c = shot_color(shot.style);
        draw_line(tail.x, tail.y, pos.x, pos.y, shot.radius * 1.2, with_alpha(c, 0.25));
        draw_circle(pos.x, pos.y, shot.radius * 1.6, with_alpha(c, 0.2));
        draw_circle(pos.x, pos.y, shot.radius, c);
        draw_circle(pos.x, pos.y, shot.radius * 0.45, WHITE);
    }

    for orbit in &world.orbits {
        let c = family_color(orbit.family);
        for b in world.blade_positions(orbit) {
            let b = v(b);
            draw_circle(b.x, b.y, 14.0, with_alpha(c, 0.12));
            if orbit.family == Family::Steel {
                let angle = (b - center).to_angle() + std::f32::consts::FRAC_PI_2;
                draw_rectangle_ex(b.x, b.y, 30.0, 8.0, DrawRectangleParams { offset: vec2(0.5, 0.5), rotation: angle, color: c });
            } else {
                draw_circle(b.x, b.y, 8.0, c);
                draw_circle(b.x, b.y, 3.5, WHITE);
            }
        }
    }

    draw_player(world, fx, t);

    for tr in &fx.transients {
        let k = tr.life / tr.max;
        match &tr.shape {
            Shape::Ring { pos, from, to, color, width } => {
                let r = to + (from - to) * k;
                draw_circle_lines(pos.x, pos.y, r, *width * k.max(0.3), with_alpha(*color, k));
            }
            Shape::Disc { pos, radius, color } => draw_circle(pos.x, pos.y, *radius, with_alpha(*color, color.a * k)),
            Shape::Cone { pos, dir, range, half_angle, color } => {
                let c = *color;
                let segments = 10;
                for s in 0..segments {
                    let a0 = -half_angle + 2.0 * half_angle * s as f32 / segments as f32;
                    let a1 = -half_angle + 2.0 * half_angle * (s + 1) as f32 / segments as f32;
                    let p0 = *pos + Vec2::from_angle(a0).rotate(*dir) * *range;
                    let p1 = *pos + Vec2::from_angle(a1).rotate(*dir) * *range;
                    draw_triangle(*pos, p0, p1, with_alpha(c, 0.28 * k));
                }
            }
            Shape::Bolt { points, color } => {
                for pair in points.windows(2) {
                    draw_line(pair[0].x, pair[0].y, pair[1].x, pair[1].y, 8.0, with_alpha(*color, 0.25 * k));
                    draw_line(pair[0].x, pair[0].y, pair[1].x, pair[1].y, 2.5, with_alpha(WHITE, k));
                }
            }
            Shape::Line { a, b, color } => draw_line(a.x, a.y, b.x, b.y, 2.0, with_alpha(*color, 0.8 * k)),
            Shape::Ghost { pos } => draw_circle(pos.x, pos.y, PLAYER_RADIUS, Color::new(0.6, 0.9, 1.0, 0.35 * k)),
        }
    }

    for p in &fx.particles {
        let k = p.life / p.max;
        draw_circle(p.pos.x, p.pos.y, p.size * (0.4 + 0.6 * k), with_alpha(p.color, k));
    }
    for w in &fx.wisps {
        let tail = w.pos - w.vel * 0.03;
        draw_line(tail.x, tail.y, w.pos.x, w.pos.y, 3.0, with_alpha(w.color, 0.35));
        draw_circle(w.pos.x, w.pos.y, 3.2, w.color);
        draw_circle(w.pos.x, w.pos.y, 1.4, WHITE);
    }
    if fx.player_glow > 0.0 {
        let p = v(world.player.pos);
        draw_circle(p.x, p.y, PLAYER_RADIUS + 8.0, Color::new(0.8, 0.95, 1.0, fx.player_glow * 0.5));
    }
}

/// Damage numbers rising off the boss, drawn in screen space over the world.
pub fn draw_numbers(ui: &crate::ui::Ui, world: &World, fx: &Fx) {
    let (w, h) = (screen_width(), screen_height());
    let scale = w / VIEW_WIDTH;
    let center = vec2(w / 2.0, h / 2.0);
    let p = v(world.player.pos);
    for n in &fx.numbers {
        let at = center + (n.pos - p) * scale;
        let a = (n.life / 0.3).min(1.0);
        let pop = 1.0 + ((n.life - 0.75) / 0.15).clamp(0.0, 1.0) * 0.5;
        let size = ((16.0 + n.value.sqrt() * 0.9).min(34.0) * pop) as u16;
        let text = format!("{:.0}", n.value);
        let tw = ui.width(&text, size);
        ui.text(&text, at.x - tw / 2.0 + 2.0, at.y + 2.0, size, Color::new(0.0, 0.0, 0.0, 0.6 * a));
        ui.text(&text, at.x - tw / 2.0, at.y, size, Color::new(1.0, 0.95, 0.75, a));
    }
}

/// Each boss has its own silhouette: the Warden a spiked octagon, the
/// Juggernaut a heavy square with plates, the Oracle a floating eye.
fn draw_boss(world: &World, p: Vec2, r: f32, t: f32, fill: Color, outline: Color) {
    let Some(b) = world.boss.as_ref() else {
        return;
    };
    let body = boss_color(b.def.kind);
    match b.def.kind {
        BossKind::EmberWarden => {
            let spin = t * 0.8;
            for k in 0..8 {
                let a = spin + k as f32 * std::f32::consts::TAU / 8.0;
                let dir = Vec2::from_angle(a);
                let tip = p + dir * r * 1.45;
                let side = dir.perp() * r * 0.25;
                draw_triangle(tip, p + dir * r * 0.9 + side, p + dir * r * 0.9 - side, outline);
            }
            draw_circle(p.x, p.y, r * 1.6, with_alpha(body, 0.1));
            draw_poly(p.x, p.y, 8, r, spin.to_degrees(), fill);
            draw_poly_lines(p.x, p.y, 8, r, spin.to_degrees(), 3.0, outline);
            draw_circle(p.x, p.y, r * (0.35 + 0.08 * (t * 5.0).sin()), Color::new(1.0, 0.85, 0.5, 1.0));
            if b.burn_immunity > 0.0 {
                let pulse = 0.5 + 0.5 * (t * 8.0).sin();
                draw_circle_lines(p.x, p.y, r * 1.9, 5.0, Color::new(1.0, 0.6, 0.1, 0.4 + 0.5 * pulse));
            }
        }
        BossKind::IronJuggernaut => {
            let e = &world.enemies[b.index];
            let rot = e.heading.y.atan2(e.heading.x).to_degrees() + 45.0;
            draw_circle(p.x, p.y, r * 1.5, with_alpha(body, 0.1));
            // Plates around the body.
            for k in 0..4 {
                let a = (rot + 45.0).to_radians() + k as f32 * std::f32::consts::FRAC_PI_2;
                let q = p + Vec2::from_angle(a) * r * 1.05;
                draw_poly(q.x, q.y, 4, r * 0.38, rot, mix(body, BLACK, 0.3));
            }
            draw_poly(p.x, p.y, 4, r * 1.05, rot, fill);
            draw_poly_lines(p.x, p.y, 4, r * 1.05, rot, 3.0, outline);
            let core = if b.charge.is_some() { Color::new(1.0, 0.4, 0.3, 1.0) } else { Color::new(0.9, 0.95, 1.0, 1.0) };
            draw_circle(p.x, p.y, r * 0.3, core);
            if b.shield > 0.0 {
                let k = b.shield / b.shield_max.max(1.0);
                let pulse = 0.5 + 0.5 * (t * 6.0).sin();
                draw_circle(p.x, p.y, r * 1.7, Color::new(0.5, 0.8, 1.0, 0.12 + 0.1 * k));
                draw_poly_lines(p.x, p.y, 6, r * 1.75, t * 40.0, 3.0 + 3.0 * k, Color::new(0.6, 0.85, 1.0, 0.5 + 0.4 * pulse));
            }
            if b.stagger > 0.0 {
                // Stars circling the head.
                for k in 0..3 {
                    let a = t * 5.0 + k as f32 * 2.1;
                    let q = p + vec2(a.cos() * r * 0.8, -r * 1.3 + a.sin() * r * 0.25);
                    draw_poly(q.x, q.y, 5, 6.0, t * 200.0, Color::new(1.0, 0.95, 0.5, 1.0));
                }
            }
        }
        BossKind::HollowOracle => {
            let hover = (t * 2.0).sin() * 5.0;
            let p = p + vec2(0.0, hover);
            let warded = b.warded_against(world.player.pos);
            draw_circle(p.x, p.y, r * 1.7, with_alpha(body, if warded { 0.18 } else { 0.08 }));
            for k in 0..6 {
                let a = t * 1.2 + k as f32 * std::f32::consts::TAU / 6.0;
                let q = p + Vec2::from_angle(a) * r * 1.35;
                draw_circle(q.x, q.y, 6.0, with_alpha(body, 0.9));
            }
            draw_poly(p.x, p.y, 3, r, -90.0 + (t * 0.5).sin() * 8.0, fill);
            draw_poly(p.x, p.y, 3, r, 90.0 + (t * 0.5).sin() * 8.0, with_alpha(fill, 0.8));
            draw_poly_lines(p.x, p.y, 6, r * 0.9, t * 15.0, 2.0, outline);
            // The eye looks at you.
            let look = (v(world.player.pos) - p).normalize_or_zero() * r * 0.15;
            draw_circle(p.x, p.y, r * 0.38, Color::new(0.95, 0.9, 1.0, 1.0));
            draw_circle(p.x + look.x, p.y + look.y, r * 0.17, Color::new(0.2, 0.05, 0.3, 1.0));
            if warded {
                draw_circle_lines(p.x, p.y, r * 1.25, 4.0, Color::new(1.0, 0.88, 0.5, 0.5 + 0.3 * (t * 6.0).sin().abs()));
            }
        }
    }
}

fn draw_enemy(world: &World, e: &arpg_machine::world::Enemy, flash: f32, t: f32) {
    let p = v(e.pos);
    let mut body = if e.kind == EnemyKind::Boss { boss_color(world.boss_def.kind) } else { enemy_color(e.kind) };
    if !e.awake {
        body = mix(body, FLOOR, 0.45);
    }
    let bob = if e.awake { 1.0 + 0.07 * (t * 9.0 + e.id as f32).sin() } else { 1.0 };
    let r = e.radius * bob;
    let fill = mix(body, WHITE, (flash / 0.09).clamp(0.0, 1.0) * 0.85);
    let outline = mix(body, WHITE, 0.35);
    match e.kind {
        EnemyKind::Grunt | EnemyKind::Add => {
            draw_poly(p.x, p.y, 4, r, 45.0, fill);
            draw_poly_lines(p.x, p.y, 4, r, 45.0, 1.5, outline);
        }
        EnemyKind::Runner => {
            let h = v(e.heading);
            let side = h.perp();
            let a = p + h * r * 1.3;
            let b = p - h * r * 0.8 + side * r;
            let c = p - h * r * 0.8 - side * r;
            draw_triangle(a, b, c, fill);
            draw_triangle_lines(a, b, c, 1.5, outline);
        }
        EnemyKind::Caster => {
            let rot = e.heading.y.atan2(e.heading.x).to_degrees();
            draw_poly(p.x, p.y, 5, r, rot, fill);
            draw_poly_lines(p.x, p.y, 5, r, rot, 1.5, outline);
            draw_circle(p.x, p.y, r * 0.3, with_alpha(WHITE, 0.7));
        }
        EnemyKind::Brute => {
            if e.windup > 0.0 {
                let pulse = 0.5 + 0.5 * (t * 30.0).sin();
                draw_circle(p.x, p.y, r * 1.9, with_alpha(telegraph_color(EnemyKind::Brute), 0.25 + 0.3 * pulse));
            }
            draw_circle(p.x, p.y, r * 1.5, with_alpha(body, 0.12));
            draw_poly(p.x, p.y, 6, r, t * 20.0, fill);
            draw_poly_lines(p.x, p.y, 6, r, t * 20.0, 2.5, outline);
        }
        EnemyKind::Boss => draw_boss(world, p, r, t, fill, outline),
    }
    if e.poison > 0 {
        draw_circle_lines(p.x, p.y, r + 4.0, 2.0, with_alpha(family_color(Family::Poison), 0.7));
    }
    if e.burn > 0 {
        let flicker = 0.6 + 0.4 * (t * 25.0 + e.id as f32).sin();
        let fire = family_color(Family::Fire);
        draw_circle(p.x, p.y - r - 5.0, 3.0 + (e.burn as f32).sqrt(), with_alpha(fire, flicker));
    }
    if e.freeze > 0.0 {
        draw_rectangle(p.x - r * 1.1, p.y - r * 1.1, r * 2.2, r * 2.2, Color::new(0.7, 0.92, 1.0, 0.45));
        draw_rectangle_lines(p.x - r * 1.1, p.y - r * 1.1, r * 2.2, r * 2.2, 2.0, Color::new(0.85, 0.97, 1.0, 0.9));
    }
    if e.bleed > 0.0 {
        let drip = (t * 3.0 + e.id as f32 * 0.37).fract();
        draw_circle(p.x + r * 0.4, p.y + r * (0.2 + drip), 2.5, family_color(Family::Blood));
    }
    if e.chill > 0.0 {
        draw_poly_lines(p.x, p.y, 6, r + 6.0, 0.0, 1.5, with_alpha(family_color(Family::Frost), 0.8));
    }
    if e.kind == EnemyKind::Brute && e.hp < e.max_hp {
        let w = e.radius * 2.4;
        draw_rectangle(p.x - w / 2.0, p.y + r + 8.0, w, 4.0, Color::new(0.0, 0.0, 0.0, 0.6));
        draw_rectangle(p.x - w / 2.0, p.y + r + 8.0, w * (e.hp / e.max_hp).max(0.0), 4.0, body);
    }
}

fn draw_player(world: &World, fx: &Fx, t: f32) {
    let pl = &world.player;
    let p = v(pl.pos);
    let hurt = (fx.player_hurt / 0.2).clamp(0.0, 1.0);
    let base = mix(Color::new(0.86, 0.96, 1.0, 1.0), Color::new(1.0, 0.3, 0.3, 1.0), hurt * 0.7);
    let alpha = if pl.invulnerable > 0.0 { 0.55 + 0.3 * (t * 40.0).sin() } else { 1.0 };
    if world.momentum > 0 {
        let m = world.momentum as f32 / arpg_machine::world::MOMENTUM_MAX as f32;
        let gold = Color::new(1.0, 0.78, 0.3, 1.0);
        draw_circle_lines(p.x, p.y, PLAYER_RADIUS + 10.0 + 6.0 * m, 2.0 + 3.0 * m, with_alpha(gold, 0.3 + 0.5 * m * world.momentum_left()));
    }
    if pl.barrier > 0.0 {
        let flash = (fx.barrier_flash / 0.15).clamp(0.0, 1.0);
        let c = Color::new(0.75, 0.9, 1.0, 0.35 + 0.4 * flash);
        draw_poly_lines(p.x, p.y, 6, PLAYER_RADIUS + 7.0, t * 40.0, 1.5 + pl.barrier / 25.0, c);
    }
    if world.thorns_active() {
        let c = family_color(Family::Nature);
        for k in 0..8 {
            let a = t * 1.5 + k as f32 * std::f32::consts::TAU / 8.0;
            let dir = Vec2::from_angle(a);
            let base = p + dir * (PLAYER_RADIUS + 4.0);
            let side = dir.perp() * 3.5;
            draw_triangle(p + dir * (PLAYER_RADIUS + 13.0), base + side, base - side, with_alpha(c, 0.8));
        }
    }
    if fx.heal_flash > 0.0 {
        let k = fx.heal_flash / 0.3;
        draw_circle_lines(p.x, p.y, PLAYER_RADIUS + 18.0 - 8.0 * k, 3.0, Color::new(0.45, 1.0, 0.55, k));
    }
    draw_circle(p.x, p.y, PLAYER_RADIUS * 2.0, Color::new(0.5, 0.85, 1.0, 0.12 * alpha));
    draw_circle(p.x, p.y, PLAYER_RADIUS, with_alpha(base, alpha));
    draw_circle(p.x, p.y, PLAYER_RADIUS * 0.45, Color::new(0.35, 0.8, 1.0, alpha));
    let f = v(pl.facing);
    let tip = p + f * (PLAYER_RADIUS + 9.0);
    let side = f.perp() * 6.0;
    draw_triangle(tip, p + f * PLAYER_RADIUS + side, p + f * PLAYER_RADIUS - side, with_alpha(base, alpha));
}
