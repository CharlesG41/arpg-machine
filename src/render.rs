//! Procedural look: silhouettes, code-driven motion, and all the spectacle in
//! effects. Everything here reads the simulation; nothing writes back to it.
use arpg_machine::content::{Family, Loot};
use arpg_machine::math::V2;
use arpg_machine::rng::Rng;
use arpg_machine::world::{EnemyKind, Event, Phase, ShrineKind, World, PLAYER_RADIUS};
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

pub fn loot_color(loot: Loot) -> Color {
    match loot {
        Loot::Support(_) => Color::new(1.0, 0.84, 0.35, 1.0),
        Loot::Artifact(_) => Color::new(1.0, 0.55, 0.22, 1.0),
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
    pub boss_phase_banner: f32,
    barrier_flash: f32,
    heal_flash: f32,
    /// Name of the shrine just claimed, and how long to show it.
    pub shrine_banner: Option<(ShrineKind, f32)>,
    pub loot_banner: Option<(Loot, f32)>,
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
            boss_phase_banner: 0.0,
            barrier_flash: 0.0,
            heal_flash: 0.0,
            shrine_banner: None,
            loot_banner: None,
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
                    if *kind == EnemyKind::Boss {
                        self.burst(pos, 240, 700.0, 6.0, 1.4, family_color(Family::Fire));
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
                    self.transient(Shape::Ring { pos, from: 20.0, to: 260.0, color: enemy_color(EnemyKind::Boss), width: 8.0 }, 0.6);
                    self.burst(pos, 80, 500.0, 5.0, 0.9, enemy_color(EnemyKind::Boss));
                    self.shake += 10.0;
                }
                Event::BossPhaseTwo { pos } => {
                    let pos = v(*pos);
                    self.transient(Shape::Ring { pos, from: 40.0, to: 320.0, color: family_color(Family::Fire), width: 10.0 }, 0.7);
                    self.boss_phase_banner = 2.5;
                    self.shake += 8.0;
                }
            }
        }
    }

    /// Ambient effects that come from ongoing state rather than events.
    pub fn ambient(&mut self, world: &World, dt: f32) {
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
        self.boss_phase_banner = (self.boss_phase_banner - dt).max(0.0);
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
    for tele in &world.telegraphs {
        let p = v(tele.pos);
        let progress = (tele.elapsed / tele.warning).clamp(0.0, 1.0);
        let c = telegraph_color(tele.owner);
        draw_circle(p.x, p.y, tele.radius * progress, with_alpha(c, 0.22));
        draw_circle_lines(p.x, p.y, tele.radius, 3.0, with_alpha(c, 0.5 + 0.4 * (t * 18.0).sin().abs()));
    }
    if let Phase::BossIncoming { timer, pos } = world.phase {
        let p = v(pos);
        let pulse = (t * 10.0).sin().abs();
        draw_circle_lines(p.x, p.y, 60.0 + timer * 60.0, 4.0, with_alpha(enemy_color(EnemyKind::Boss), 0.4 + 0.5 * pulse));
        draw_circle(p.x, p.y, 30.0, with_alpha(enemy_color(EnemyKind::Boss), 0.2 + 0.2 * pulse));
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
        let c = enemy_color(EnemyKind::Caster);
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
}

fn draw_enemy(world: &World, e: &arpg_machine::world::Enemy, flash: f32, t: f32) {
    let p = v(e.pos);
    let mut body = enemy_color(e.kind);
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
        EnemyKind::Boss => {
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
            if world.boss.as_ref().is_some_and(|b| b.burn_immunity > 0.0) {
                let pulse = 0.5 + 0.5 * (t * 8.0).sin();
                draw_circle_lines(p.x, p.y, r * 1.9, 5.0, Color::new(1.0, 0.6, 0.1, 0.4 + 0.5 * pulse));
            }
        }
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
