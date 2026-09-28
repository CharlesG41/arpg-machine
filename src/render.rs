//! Procedural look: silhouettes, code-driven motion, and all the spectacle in
//! effects. Everything here reads the simulation; nothing writes back to it.
use arpg_machine::content::Element;
use arpg_machine::math::V2;
use arpg_machine::rng::Rng;
use arpg_machine::world::{EnemyKind, Event, Phase, World, MAP_H, MAP_W, PLAYER_RADIUS};
use macroquad::prelude::*;

const MAX_PARTICLES: usize = 6000;
/// World units visible across the screen width.
pub const VIEW_WIDTH: f32 = 1500.0;

pub const BACKGROUND: Color = Color::new(0.035, 0.04, 0.055, 1.0);
const FLOOR: Color = Color::new(0.06, 0.068, 0.09, 1.0);
const GRID_LINE: Color = Color::new(0.11, 0.12, 0.16, 1.0);

pub fn element_color(element: Element) -> Color {
    match element {
        Element::Arcane => Color::new(0.74, 0.47, 1.0, 1.0),
        Element::Frost => Color::new(0.48, 0.86, 1.0, 1.0),
        Element::Fire => Color::new(1.0, 0.56, 0.16, 1.0),
        Element::Poison => Color::new(0.5, 0.95, 0.32, 1.0),
        Element::Physical => Color::new(0.86, 0.89, 0.96, 1.0),
        Element::Lightning => Color::new(1.0, 0.95, 0.5, 1.0),
    }
}

fn enemy_color(kind: EnemyKind) -> Color {
    match kind {
        EnemyKind::Grunt => Color::new(0.78, 0.2, 0.3, 1.0),
        EnemyKind::Runner => Color::new(0.96, 0.43, 0.26, 1.0),
        EnemyKind::Brute => Color::new(0.6, 0.32, 0.9, 1.0),
        EnemyKind::Boss => Color::new(0.95, 0.38, 0.18, 1.0),
        EnemyKind::Add => Color::new(0.64, 0.36, 0.38, 1.0),
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
    Cone { pos: Vec2, dir: Vec2, range: f32, half_angle: f32 },
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
                Event::Hit { enemy, element } => {
                    self.flash[*enemy] = 0.09;
                    if self.rng.chance(0.25) {
                        let pos = v(world.enemies[*enemy].pos);
                        self.burst(pos, 1, 120.0, 2.5, 0.25, element_color(*element));
                    }
                }
                Event::Death { pos, kind, element, radius } => {
                    let pos = v(*pos);
                    let n = 8 + (*radius * 0.6) as usize;
                    self.burst(pos, n, 260.0 + radius * 4.0, 3.5, 0.55, enemy_color(*kind));
                    self.burst(pos, n / 2 + 2, 200.0, 3.0, 0.45, element_color(*element));
                    self.transient(
                        Shape::Ring { pos, from: radius * 0.6, to: radius * 1.8, color: element_color(*element), width: 2.0 },
                        0.2,
                    );
                    if *kind == EnemyKind::Boss {
                        self.burst(pos, 240, 700.0, 6.0, 1.4, element_color(Element::Fire));
                        self.shake += 18.0;
                    }
                }
                Event::Nova { pos, radius } => {
                    let pos = v(*pos);
                    let color = element_color(Element::Frost);
                    self.transient(Shape::Ring { pos, from: radius * 0.25, to: *radius, color, width: 6.0 }, 0.28);
                    self.transient(Shape::Disc { pos, radius: *radius, color: with_alpha(color, 0.12) }, 0.2);
                    self.burst(pos, 18, radius * 3.2, 3.0, 0.35, color);
                }
                Event::Cone { pos, dir, range, half_angle } => {
                    let (pos, dir) = (v(*pos), v(*dir));
                    self.transient(Shape::Cone { pos, dir, range: *range, half_angle: *half_angle }, 0.2);
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
                            color: element_color(Element::Fire),
                            drag: 5.0,
                        });
                    }
                }
                Event::Chain { points } => {
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
                    self.transient(Shape::Bolt { points: jagged, color: element_color(Element::Lightning) }, 0.16);
                    for p in pts.iter().skip(1) {
                        self.burst(*p, 4, 160.0, 2.5, 0.25, element_color(Element::Lightning));
                    }
                }
                Event::Explosion { pos, radius } => {
                    let pos = v(*pos);
                    let fire = element_color(Element::Fire);
                    self.transient(Shape::Disc { pos, radius: *radius, color: with_alpha(fire, 0.35) }, 0.22);
                    self.transient(Shape::Ring { pos, from: radius * 0.3, to: *radius, color: fire, width: 4.0 }, 0.25);
                    self.burst(pos, 14, 380.0, 4.0, 0.5, fire);
                    self.shake = (self.shake + 1.5).min(10.0);
                }
                Event::Spread { from, to } => {
                    self.transient(Shape::Line { a: v(*from), b: v(*to), color: element_color(Element::Poison) }, 0.3);
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
                Event::Detonate { pos, radius } => {
                    let pos = v(*pos);
                    let fire = Color::new(1.0, 0.35, 0.12, 1.0);
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
                    self.transient(Shape::Ring { pos, from: 40.0, to: 320.0, color: element_color(Element::Fire), width: 10.0 }, 0.7);
                    self.boss_phase_banner = 2.5;
                    self.shake += 8.0;
                }
            }
        }
    }

    /// Ambient effects that come from ongoing state rather than events.
    pub fn ambient(&mut self, world: &World, dt: f32) {
        for pool in &world.pools {
            if self.rng.chance(pool.radius * dt * 0.25) {
                let offset = Vec2::from_angle(self.rng.range(0.0, std::f32::consts::TAU)) * self.rng.range(0.0, pool.radius);
                let pos = v(pool.pos) + offset;
                self.particles.push(Particle {
                    pos,
                    vel: vec2(0.0, -30.0),
                    life: 0.6,
                    max: 0.6,
                    size: 3.0,
                    color: element_color(Element::Poison),
                    drag: 1.0,
                });
            }
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

    // Floor and a faint grid, so movement reads even in an empty area.
    draw_rectangle(0.0, 0.0, MAP_W, MAP_H, FLOOR);
    let step = 160.0;
    let mut x = (view.x / step).floor() * step;
    while x < view.right() {
        if (0.0..=MAP_W).contains(&x) {
            draw_line(x, view.y.max(0.0), x, view.bottom().min(MAP_H), 1.0, GRID_LINE);
        }
        x += step;
    }
    let mut y = (view.y / step).floor() * step;
    while y < view.bottom() {
        if (0.0..=MAP_H).contains(&y) {
            draw_line(view.x.max(0.0), y, view.right().min(MAP_W), y, 1.0, GRID_LINE);
        }
        y += step;
    }
    draw_rectangle_lines(0.0, 0.0, MAP_W, MAP_H, 6.0, Color::new(0.3, 0.2, 0.35, 1.0));

    // Ground layer.
    for pool in &world.pools {
        let p = v(pool.pos);
        let fade = (pool.time_left / 0.4).min(1.0) * ((pool.duration - pool.time_left) / 0.2).min(1.0);
        let c = element_color(Element::Poison);
        draw_circle(p.x, p.y, pool.radius, with_alpha(c, 0.14 * fade));
        let wobble = 1.0 + 0.03 * (t * 6.0).sin();
        draw_circle_lines(p.x, p.y, pool.radius * wobble, 2.0, with_alpha(c, 0.55 * fade));
    }
    for tele in &world.telegraphs {
        let p = v(tele.pos);
        let progress = (tele.elapsed / tele.warning).clamp(0.0, 1.0);
        let c = Color::new(1.0, 0.3, 0.12, 1.0);
        draw_circle(p.x, p.y, tele.radius * progress, with_alpha(c, 0.22));
        draw_circle_lines(p.x, p.y, tele.radius, 3.0, with_alpha(c, 0.5 + 0.4 * (t * 18.0).sin().abs()));
    }
    if let Phase::BossIncoming { timer, pos } = world.phase {
        let p = v(pos);
        let pulse = (t * 10.0).sin().abs();
        draw_circle_lines(p.x, p.y, 60.0 + timer * 60.0, 4.0, with_alpha(enemy_color(EnemyKind::Boss), 0.4 + 0.5 * pulse));
        draw_circle(p.x, p.y, 30.0, with_alpha(enemy_color(EnemyKind::Boss), 0.2 + 0.2 * pulse));
    }

    // Orbit trail under the bodies.
    if let Some(blades) = world.blade_positions() {
        let r = blades[0].dist(world.player.pos);
        draw_circle_lines(center.x, center.y, r, 10.0, with_alpha(element_color(Element::Physical), 0.05));
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
        let c = element_color(p.element);
        draw_line(tail.x, tail.y, pos.x, pos.y, 9.0, with_alpha(c, 0.25));
        draw_line(tail.x, tail.y, pos.x, pos.y, 4.0, c);
        draw_circle(pos.x, pos.y, 4.0, WHITE);
    }

    if let Some(blades) = world.blade_positions() {
        let c = element_color(Element::Physical);
        for b in blades {
            let b = v(b);
            let angle = (b - center).to_angle() + std::f32::consts::FRAC_PI_2;
            draw_circle(b.x, b.y, 14.0, with_alpha(c, 0.12));
            draw_rectangle_ex(b.x, b.y, 30.0, 8.0, DrawRectangleParams { offset: vec2(0.5, 0.5), rotation: angle, color: c });
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
            Shape::Cone { pos, dir, range, half_angle } => {
                let c = element_color(Element::Fire);
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
        EnemyKind::Brute => {
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
        draw_circle_lines(p.x, p.y, r + 4.0, 2.0, with_alpha(element_color(Element::Poison), 0.7));
    }
    if e.burn > 0 {
        let flicker = 0.6 + 0.4 * (t * 25.0 + e.id as f32).sin();
        let fire = element_color(Element::Fire);
        draw_circle(p.x, p.y - r - 5.0, 3.0 + (e.burn as f32).sqrt(), with_alpha(fire, flicker));
    }
    if e.chill > 0.0 {
        draw_poly_lines(p.x, p.y, 6, r + 6.0, 0.0, 1.5, with_alpha(element_color(Element::Frost), 0.8));
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
    draw_circle(p.x, p.y, PLAYER_RADIUS * 2.0, Color::new(0.5, 0.85, 1.0, 0.12 * alpha));
    draw_circle(p.x, p.y, PLAYER_RADIUS, with_alpha(base, alpha));
    draw_circle(p.x, p.y, PLAYER_RADIUS * 0.45, Color::new(0.35, 0.8, 1.0, alpha));
    let f = v(pl.facing);
    let tip = p + f * (PLAYER_RADIUS + 9.0);
    let side = f.perp() * 6.0;
    draw_triangle(tip, p + f * PLAYER_RADIUS + side, p + f * PLAYER_RADIUS - side, with_alpha(base, alpha));
}
