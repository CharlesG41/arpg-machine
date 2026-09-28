//! One map: packs to clear, a progress bar, then a boss.
use crate::content::{Build, Element, Shape, SpellKind, SupportKind};
use crate::grid::Grid;
use crate::math::V2;
use crate::rng::Rng;
use std::collections::VecDeque;
use std::f32::consts::TAU;

pub const DT: f32 = 1.0 / 60.0;
pub const MAP_W: f32 = 4200.0;
pub const MAP_H: f32 = 4200.0;
pub const PLAYER_RADIUS: f32 = 14.0;
const PLAYER_SPEED: f32 = 230.0;
const DASH_DISTANCE: f32 = 260.0;
const DASH_TIME: f32 = 0.14;
pub const DASH_COOLDOWN: f32 = 1.8;
const AGGRO_RANGE: f32 = 560.0;
const PACKS: usize = 110;
pub const PROGRESS_NEEDED: f32 = 500.0;
const BOSS_WARNING: f32 = 1.6;
pub const MOMENTUM_MAX: u8 = 20;
pub const MOMENTUM_DECAY: f32 = 2.0;
const MOMENTUM_RATE: f32 = 0.03;
const DOT_TICK: f32 = 0.5;
/// Source index for damage that no spell owns.
pub const NO_SOURCE: u8 = u8::MAX;
const ORBIT_RADIUS: f32 = 120.0;
const BLADE_RADIUS: f32 = 16.0;
pub const BLADES: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    Grunt,
    Runner,
    Brute,
    Boss,
    Add,
}

impl EnemyKind {
    fn hp(self) -> f32 {
        match self {
            EnemyKind::Grunt => 40.0,
            EnemyKind::Runner => 24.0,
            EnemyKind::Brute => 320.0,
            EnemyKind::Boss => 4000.0,
            EnemyKind::Add => 30.0,
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            EnemyKind::Grunt => 13.0,
            EnemyKind::Runner => 10.0,
            EnemyKind::Brute => 24.0,
            EnemyKind::Boss => 46.0,
            EnemyKind::Add => 11.0,
        }
    }

    fn speed(self) -> f32 {
        match self {
            EnemyKind::Grunt => 95.0,
            EnemyKind::Runner => 165.0,
            EnemyKind::Brute => 70.0,
            EnemyKind::Boss => 62.0,
            EnemyKind::Add => 120.0,
        }
    }

    fn contact_dps(self) -> f32 {
        match self {
            EnemyKind::Grunt => 12.0,
            EnemyKind::Runner => 8.0,
            EnemyKind::Brute => 25.0,
            EnemyKind::Boss => 40.0,
            EnemyKind::Add => 10.0,
        }
    }

    fn progress(self) -> f32 {
        match self {
            EnemyKind::Grunt | EnemyKind::Runner => 1.0,
            EnemyKind::Brute => 6.0,
            EnemyKind::Boss | EnemyKind::Add => 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Enemy {
    pub id: u32,
    pub kind: EnemyKind,
    pub pos: V2,
    pub heading: V2,
    pub hp: f32,
    pub max_hp: f32,
    pub radius: f32,
    pub alive: bool,
    pub awake: bool,
    pub poison: u16,
    poison_time: f32,
    pub burn: u16,
    pub chill: f32,
    dot_timer: f32,
    /// Damage already on its way (projectiles in flight): anti-overkill.
    pending: f32,
    pub last_source: u8,
    blade_cooldown: f32,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub pos: V2,
    pub hp: f32,
    pub max_hp: f32,
    pub facing: V2,
    pub dash_cooldown: f32,
    pub dash_time: f32,
    dash_dir: V2,
    pub invulnerable: f32,
}

#[derive(Clone, Debug)]
pub struct Projectile {
    pub pos: V2,
    pub vel: V2,
    pub element: Element,
    damage: f32,
    radius: f32,
    pierce: u8,
    range_left: f32,
    source: u8,
    target: Option<(usize, u32)>,
    hits: Vec<u32>,
    pub alive: bool,
}

#[derive(Clone, Debug)]
pub struct Pool {
    pub pos: V2,
    pub radius: f32,
    pub time_left: f32,
    pub duration: f32,
    tick: f32,
    source: u8,
    damage: f32,
    poison: u16,
    burn: u16,
}

#[derive(Clone, Debug)]
pub struct Telegraph {
    pub pos: V2,
    pub radius: f32,
    pub warning: f32,
    pub elapsed: f32,
    damage: f32,
}

#[derive(Clone, Debug)]
pub struct BossState {
    pub index: usize,
    circle_timer: f32,
    /// Seconds of Burn immunity left (phase 2).
    pub burn_immunity: f32,
    phase_two: bool,
    add_waves_left: u8,
    add_timer: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Clearing,
    BossIncoming { timer: f32, pos: V2 },
    Boss,
    Victory,
    Defeat,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SpellStats {
    pub damage: f32,
    pub kills: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    pub move_dir: V2,
    pub dash: bool,
}

/// What happened this step, for the renderer's effects. The simulation never
/// reads these back.
#[derive(Clone, Debug)]
pub enum Event {
    Cast { slot: usize },
    Hit { enemy: usize, element: Element },
    Death { pos: V2, kind: EnemyKind, element: Element, radius: f32 },
    Nova { pos: V2, radius: f32 },
    Cone { pos: V2, dir: V2, range: f32, half_angle: f32 },
    Chain { points: Vec<V2> },
    Explosion { pos: V2, radius: f32 },
    Spread { from: V2, to: V2 },
    PlayerHurt { amount: f32 },
    Dash { from: V2, dir: V2 },
    Detonate { pos: V2, radius: f32 },
    BossSpawned { pos: V2 },
    BossPhaseTwo { pos: V2 },
}

#[derive(Clone, Copy, Debug)]
struct SlotState {
    spell: SpellKind,
    support: Option<SupportKind>,
    cooldown: f32,
}

#[derive(Clone, Copy, Default)]
struct HitStatus {
    poison: u16,
    burn: u16,
    chill: f32,
}

pub struct World {
    pub seed: u64,
    pub tier: u32,
    rng: Rng,
    pub build: Build,
    slots: Vec<SlotState>,
    pub player: Player,
    pub enemies: Vec<Enemy>,
    next_id: u32,
    grid: Grid,
    pub projectiles: Vec<Projectile>,
    pub pools: Vec<Pool>,
    pub telegraphs: Vec<Telegraph>,
    pub time: f32,
    pub phase: Phase,
    pub progress: f32,
    pub momentum: u8,
    momentum_timer: f32,
    pub orbit_angle: f32,
    pub boss: Option<BossState>,
    pub stats: Vec<SpellStats>,
    pub kills: u32,
    kill_times: VecDeque<f32>,
    pub peak_kills_per_minute: f32,
    pub clear_time: Option<f32>,
    pub boss_start: Option<f32>,
    pub end_time: Option<f32>,
    pub events: Vec<Event>,
    death_queue: Vec<(usize, Element)>,
    moves: Vec<(usize, V2)>,
    near: Vec<usize>,
}

impl World {
    pub fn new(build: Build, seed: u64, tier: u32) -> Self {
        let slots = build
            .slots
            .iter()
            .map(|s| SlotState { spell: s.spell, support: s.support, cooldown: 0.0 })
            .collect::<Vec<_>>();
        let stats = vec![SpellStats::default(); slots.len()];
        let start = V2::new(MAP_W / 2.0, MAP_H / 2.0);
        let mut world = World {
            seed,
            tier,
            rng: Rng::new(seed),
            build,
            slots,
            player: Player {
                pos: start,
                hp: 300.0,
                max_hp: 300.0,
                facing: V2::new(1.0, 0.0),
                dash_cooldown: 0.0,
                dash_time: 0.0,
                dash_dir: V2::ZERO,
                invulnerable: 0.0,
            },
            enemies: Vec::new(),
            next_id: 0,
            grid: Grid::new(MAP_W, MAP_H, 80.0),
            projectiles: Vec::new(),
            pools: Vec::new(),
            telegraphs: Vec::new(),
            time: 0.0,
            phase: Phase::Clearing,
            progress: 0.0,
            momentum: 0,
            momentum_timer: 0.0,
            orbit_angle: 0.0,
            boss: None,
            stats,
            kills: 0,
            kill_times: VecDeque::new(),
            peak_kills_per_minute: 0.0,
            clear_time: None,
            boss_start: None,
            end_time: None,
            events: Vec::new(),
            death_queue: Vec::new(),
            moves: Vec::new(),
            near: Vec::new(),
        };
        world.generate(start);
        world
    }

    fn hp_mult(&self) -> f32 {
        1.3f32.powi(self.tier as i32)
    }

    fn damage_mult(&self) -> f32 {
        1.2f32.powi(self.tier as i32)
    }

    fn spawn(&mut self, kind: EnemyKind, pos: V2, awake: bool) -> usize {
        let hp = kind.hp() * self.hp_mult();
        self.enemies.push(Enemy {
            id: self.next_id,
            kind,
            pos,
            heading: V2::new(1.0, 0.0),
            hp,
            max_hp: hp,
            radius: kind.radius(),
            alive: true,
            awake,
            poison: 0,
            poison_time: 0.0,
            burn: 0,
            chill: 0.0,
            dot_timer: 0.0,
            pending: 0.0,
            last_source: NO_SOURCE,
            blade_cooldown: 0.0,
        });
        self.next_id += 1;
        self.enemies.len() - 1
    }

    fn generate(&mut self, start: V2) {
        let margin = 150.0;
        for _ in 0..PACKS {
            let mut center = start;
            for _ in 0..12 {
                center = V2::new(self.rng.range(margin, MAP_W - margin), self.rng.range(margin, MAP_H - margin));
                if center.dist(start) > 600.0 {
                    break;
                }
            }
            let count = self.rng.range_usize(8, 16);
            let spread = 30.0 + count as f32 * 6.0;
            for _ in 0..count {
                let kind = if self.rng.chance(0.25) { EnemyKind::Runner } else { EnemyKind::Grunt };
                let offset = V2::from_angle(self.rng.range(0.0, TAU)) * self.rng.range(0.0, spread);
                self.spawn(kind, clamp_to_map(center + offset, 20.0), false);
            }
            if self.rng.chance(0.3) {
                self.spawn(EnemyKind::Brute, center, false);
            }
        }
    }

    // ---- queries ------------------------------------------------------------

    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// Cooldown progress of a bar slot, 0 (just cast) to 1 (ready).
    pub fn slot_ready_fraction(&self, index: usize) -> f32 {
        let slot = &self.slots[index];
        let cd = slot.spell.cooldown();
        if cd <= 0.0 {
            1.0
        } else {
            1.0 - (slot.cooldown / cd).clamp(0.0, 1.0)
        }
    }

    pub fn cooldown_rate(&self) -> f32 {
        1.0 + MOMENTUM_RATE * self.momentum as f32
    }

    pub fn momentum_left(&self) -> f32 {
        if self.momentum == 0 {
            0.0
        } else {
            1.0 - (self.momentum_timer / MOMENTUM_DECAY).clamp(0.0, 1.0)
        }
    }

    pub fn kills_per_minute(&self) -> f32 {
        self.kill_times.len() as f32 * 6.0
    }

    pub fn awake_enemies(&self) -> usize {
        self.enemies.iter().filter(|e| e.alive && e.awake).count()
    }

    pub fn blade_positions(&self) -> Option<[V2; BLADES]> {
        let slot = self.slots.iter().find(|s| s.spell == SpellKind::OrbitBlades)?;
        let radius = ORBIT_RADIUS * area_mult(slot.support);
        Some(std::array::from_fn(|k| {
            self.player.pos + V2::from_angle(self.orbit_angle + k as f32 * TAU / BLADES as f32) * radius
        }))
    }

    fn support_active(&self, support: SupportKind) -> Option<usize> {
        self.slots.iter().position(|s| s.support == Some(support))
    }

    /// Alive enemies whose body overlaps the circle, into `self.near`.
    fn collect_near(&mut self, pos: V2, radius: f32) {
        let mut near = std::mem::take(&mut self.near);
        near.clear();
        let enemies = &self.enemies;
        self.grid.for_each_near(pos, radius + 50.0, |id| {
            let e = &enemies[id as usize];
            let reach = radius + e.radius;
            if e.alive && e.pos.dist_sq(pos) <= reach * reach {
                near.push(id as usize);
            }
        });
        self.near = near;
    }

    /// Best targets in range by the spell's targeting rule, skipping enemies that
    /// the damage already on its way will kill.
    fn pick_targets(&self, from: V2, range: f32, count: usize, hunter: bool) -> Vec<usize> {
        let mut candidates: Vec<(f32, usize)> = Vec::new();
        self.grid.for_each_near(from, range + 50.0, |id| {
            let e = &self.enemies[id as usize];
            let d2 = e.pos.dist_sq(from);
            if !e.alive || d2 > range * range || e.hp - e.pending <= 0.0 {
                return;
            }
            let priority = if hunter {
                match e.kind {
                    EnemyKind::Boss => -2e9,
                    EnemyKind::Brute => -1e9,
                    _ => 0.0,
                }
            } else {
                0.0
            };
            candidates.push((priority + d2, id as usize));
        });
        candidates.sort_by(|a, b| a.0.total_cmp(&b.0));
        candidates.into_iter().take(count).map(|(_, i)| i).collect()
    }

    // ---- step ---------------------------------------------------------------

    pub fn step(&mut self, input: &Input) {
        if matches!(self.phase, Phase::Victory | Phase::Defeat) {
            return;
        }
        self.time += DT;
        self.update_player(input);
        self.move_enemies();
        self.rebuild_grid();
        self.contact_damage();
        self.update_momentum();
        self.update_spells();
        self.update_projectiles();
        self.update_pools();
        self.update_orbit();
        self.update_statuses();
        self.update_boss();
        self.update_telegraphs();
        self.process_deaths();
        self.update_phase();
        while self.kill_times.front().is_some_and(|t| self.time - t > 10.0) {
            self.kill_times.pop_front();
        }
        // Wait a few seconds so the rolling rate is not inflated by one early pack.
        if self.time > 10.0 {
            self.peak_kills_per_minute = self.peak_kills_per_minute.max(self.kills_per_minute());
        }
        if self.player.hp <= 0.0 && self.phase != Phase::Victory {
            self.phase = Phase::Defeat;
            self.end_time = Some(self.time);
        }
    }

    fn update_player(&mut self, input: &Input) {
        let p = &mut self.player;
        p.dash_cooldown = (p.dash_cooldown - DT).max(0.0);
        p.invulnerable = (p.invulnerable - DT).max(0.0);
        let dir = input.move_dir.norm();
        if dir != V2::ZERO {
            p.facing = dir;
        }
        if input.dash && p.dash_cooldown <= 0.0 && p.dash_time <= 0.0 {
            p.dash_time = DASH_TIME;
            p.dash_dir = if dir != V2::ZERO { dir } else { p.facing };
            p.dash_cooldown = DASH_COOLDOWN;
            p.invulnerable = DASH_TIME + 0.1;
            self.events.push(Event::Dash { from: p.pos, dir: p.dash_dir });
        }
        if p.dash_time > 0.0 {
            p.pos += p.dash_dir * (DASH_DISTANCE / DASH_TIME * DT);
            p.dash_time -= DT;
        } else {
            p.pos += dir * (PLAYER_SPEED * DT);
        }
        p.pos = clamp_to_map(p.pos, PLAYER_RADIUS);
        p.hp = (p.hp + 3.0 * DT).min(p.max_hp);
    }

    fn move_enemies(&mut self) {
        let player = self.player.pos;
        let mut moves = std::mem::take(&mut self.moves);
        moves.clear();
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            if !e.alive {
                continue;
            }
            if !e.awake {
                if e.pos.dist_sq(player) < AGGRO_RANGE * AGGRO_RANGE {
                    self.enemies[i].awake = true;
                }
                continue;
            }
            let to_player = player - e.pos;
            let dist = to_player.len();
            let speed = e.kind.speed() * if e.chill > 0.0 { 0.5 } else { 1.0 };
            let mut step = if dist > PLAYER_RADIUS + e.radius {
                to_player.norm() * (speed * DT)
            } else {
                V2::ZERO
            };
            if e.kind != EnemyKind::Boss {
                // Separation keeps packs readable instead of stacking into one blob.
                let mut push = V2::ZERO;
                let enemies = &self.enemies;
                self.grid.for_each_near(e.pos, e.radius * 2.0, |id| {
                    let other = &enemies[id as usize];
                    if id as usize == i || !other.alive {
                        return;
                    }
                    let min = e.radius + other.radius;
                    let d = e.pos - other.pos;
                    let d2 = d.len_sq();
                    if d2 < min * min && d2 > 1e-4 {
                        let len = d2.sqrt();
                        push += d * ((min - len) / len * 0.5);
                    }
                });
                step += push;
            }
            moves.push((i, step));
        }
        for &(i, step) in &moves {
            let e = &mut self.enemies[i];
            e.pos = clamp_to_map(e.pos + step, e.radius);
            if step.len_sq() > 1e-3 {
                e.heading = step.norm();
            }
        }
        self.moves = moves;
    }

    fn rebuild_grid(&mut self) {
        self.grid.clear();
        for (i, e) in self.enemies.iter().enumerate() {
            if e.alive {
                self.grid.insert(i as u32, e.pos);
            }
        }
    }

    fn contact_damage(&mut self) {
        if self.player.invulnerable > 0.0 {
            return;
        }
        let pos = self.player.pos;
        self.collect_near(pos, PLAYER_RADIUS + 2.0);
        let dps: f32 = self.near.iter().map(|&i| self.enemies[i].kind.contact_dps()).sum();
        if dps > 0.0 {
            let amount = dps * self.damage_mult() * DT;
            self.player.hp -= amount;
            self.events.push(Event::PlayerHurt { amount });
        }
    }

    fn update_momentum(&mut self) {
        self.momentum_timer += DT;
        if self.momentum > 0 && self.momentum_timer > MOMENTUM_DECAY {
            self.momentum = 0;
        }
    }

    fn update_spells(&mut self) {
        let rate = self.cooldown_rate();
        for i in 0..self.slots.len() {
            let spell = self.slots[i].spell;
            if !spell.casts() {
                continue;
            }
            self.slots[i].cooldown -= DT * rate;
            if self.slots[i].cooldown <= 0.0 {
                if self.cast(i) {
                    self.slots[i].cooldown = (self.slots[i].cooldown + spell.cooldown()).max(0.0);
                } else {
                    // Hold the charge: cast as soon as something is in reach.
                    self.slots[i].cooldown = 0.0;
                }
            }
        }
    }

    fn hit_status(&self, slot: usize) -> HitStatus {
        let s = &self.slots[slot];
        let mut status = HitStatus::default();
        match s.spell {
            SpellKind::FrostNova => status.chill = 2.0,
            SpellKind::FlameCone => status.burn = 4,
            SpellKind::VenomPool => status.poison = 2,
            _ => {}
        }
        match s.support {
            Some(SupportKind::Contagion) => status.poison += 3,
            Some(SupportKind::Combustion) => status.burn += 3,
            _ => {}
        }
        status
    }

    /// Casts a slot; an Echo on the right neighbor casts it too. Returns false when
    /// there was nothing to cast at.
    fn cast(&mut self, slot: usize) -> bool {
        let SlotState { spell, support, .. } = self.slots[slot];
        let hunter = support == Some(SupportKind::Hunter);
        let status = self.hit_status(slot);
        let source = slot as u8;
        let origin = self.player.pos;
        let cast = match spell.shape() {
            Shape::Projectile => {
                let multishot = support == Some(SupportKind::Multishot);
                let count = if multishot { 3 } else { 1 };
                let damage = spell.damage() * if multishot { 0.8 } else { 1.0 };
                let targets = self.pick_targets(origin, 650.0, count, hunter);
                if targets.is_empty() {
                    false
                } else {
                    for k in 0..count {
                        let (aim, target) = match targets.get(k) {
                            Some(&t) => (self.enemies[t].pos - origin, Some(t)),
                            // Fewer targets than projectiles: fan the extras out.
                            None => {
                                let base = self.enemies[targets[0]].pos - origin;
                                let side = if k % 2 == 0 { 1.0 } else { -1.0 };
                                (base.rotate(side * 0.2 * k as f32), None)
                            }
                        };
                        if let Some(t) = target {
                            self.enemies[t].pending += damage;
                        }
                        self.projectiles.push(Projectile {
                            pos: origin,
                            vel: aim.norm() * 900.0,
                            element: spell.element(),
                            damage,
                            radius: 7.0,
                            pierce: if support == Some(SupportKind::Pierce) { 3 } else { 0 },
                            range_left: 720.0,
                            source,
                            target: target.map(|t| (t, self.enemies[t].id)),
                            hits: Vec::new(),
                            alive: true,
                        });
                    }
                    true
                }
            }
            Shape::Chain => {
                let jumps = if support == Some(SupportKind::Multishot) { 5 } else { 3 };
                let damage = spell.damage() * if support == Some(SupportKind::Multishot) { 0.8 } else { 1.0 };
                match self.pick_targets(origin, 520.0, 1, hunter).first() {
                    None => false,
                    Some(&first) => {
                        let mut points = vec![origin];
                        let mut hit_ids = Vec::new();
                        let mut current = first;
                        for _ in 0..=jumps {
                            let pos = self.enemies[current].pos;
                            points.push(pos);
                            hit_ids.push(self.enemies[current].id);
                            self.strike(current, damage, source, spell.element(), status);
                            self.collect_near(pos, 230.0);
                            let next = self
                                .near
                                .iter()
                                .copied()
                                .filter(|&i| !hit_ids.contains(&self.enemies[i].id))
                                .min_by(|&a, &b| {
                                    self.enemies[a].pos.dist_sq(pos).total_cmp(&self.enemies[b].pos.dist_sq(pos))
                                });
                            match next {
                                Some(n) => current = n,
                                None => break,
                            }
                        }
                        self.events.push(Event::Chain { points });
                        true
                    }
                }
            }
            Shape::Nova => {
                let radius = 170.0 * area_mult(support);
                self.collect_near(origin, radius * 0.9);
                if self.near.is_empty() {
                    false
                } else {
                    self.collect_near(origin, radius);
                    let near = self.near.clone();
                    for i in near {
                        self.strike(i, spell.damage(), source, spell.element(), status);
                    }
                    self.events.push(Event::Nova { pos: origin, radius });
                    true
                }
            }
            Shape::Cone => {
                let range = 260.0 * (1.0 + (area_mult(support) - 1.0) * 0.7);
                let half_angle: f32 = 0.61;
                match self.pick_targets(origin, range * 1.1, 1, hunter).first() {
                    None => false,
                    Some(&t) => {
                        let dir = (self.enemies[t].pos - origin).norm();
                        self.collect_near(origin, range);
                        let cos_half = half_angle.cos();
                        let hits: Vec<usize> = self
                            .near
                            .iter()
                            .copied()
                            .filter(|&i| {
                                let to = self.enemies[i].pos - origin;
                                to.len() < self.enemies[i].radius + 20.0 || to.norm().dot(dir) >= cos_half
                            })
                            .collect();
                        for i in hits {
                            self.strike(i, spell.damage(), source, spell.element(), status);
                        }
                        self.events.push(Event::Cone { pos: origin, dir, range, half_angle });
                        true
                    }
                }
            }
            Shape::GroundZone => {
                let radius = 110.0 * area_mult(support);
                match self.densest_point(origin, 520.0, radius, hunter) {
                    None => false,
                    Some(pos) => {
                        self.pools.push(Pool {
                            pos,
                            radius,
                            time_left: 3.5,
                            duration: 3.5,
                            tick: DOT_TICK,
                            source,
                            damage: spell.damage(),
                            poison: status.poison,
                            burn: status.burn,
                        });
                        true
                    }
                }
            }
            Shape::Orbit => false,
        };
        if cast {
            self.events.push(Event::Cast { slot });
            if self.slots.get(slot + 1).is_some_and(|s| s.support == Some(SupportKind::Echo)) {
                self.cast(slot + 1);
            }
        }
        cast
    }

    /// Where a ground zone hits the most enemies (or the priority target for Hunter).
    fn densest_point(&mut self, from: V2, range: f32, radius: f32, hunter: bool) -> Option<V2> {
        if hunter {
            if let Some(&t) = self.pick_targets(from, range, 1, true).first() {
                if matches!(self.enemies[t].kind, EnemyKind::Boss | EnemyKind::Brute) {
                    return Some(self.enemies[t].pos);
                }
            }
        }
        self.collect_near(from, range);
        let candidates = self.near.clone();
        if candidates.is_empty() {
            return None;
        }
        let step = (candidates.len() / 20).max(1);
        let mut best = (0usize, candidates[0]);
        for &c in candidates.iter().step_by(step) {
            let center = self.enemies[c].pos;
            let count = candidates.iter().filter(|&&o| self.enemies[o].pos.dist_sq(center) <= radius * radius).count();
            if count > best.0 {
                best = (count, c);
            }
        }
        Some(self.enemies[best.1].pos)
    }

    /// Damage plus the statuses of the source spell.
    fn strike(&mut self, index: usize, damage: f32, source: u8, element: Element, status: HitStatus) {
        self.damage(index, damage, source, element);
        let burn_immune = self.boss.as_ref().is_some_and(|b| b.index == index && b.burn_immunity > 0.0);
        let e = &mut self.enemies[index];
        if !e.alive {
            // Statuses still matter on the corpse: Contagion and Combustion read them.
            e.poison = e.poison.saturating_add(status.poison).min(80);
            if !burn_immune {
                e.burn = e.burn.saturating_add(status.burn).min(60);
            }
            return;
        }
        if status.poison > 0 {
            e.poison = (e.poison + status.poison).min(80);
            e.poison_time = 4.0;
        }
        if status.burn > 0 && !burn_immune {
            e.burn = (e.burn + status.burn).min(60);
        }
        e.chill = e.chill.max(status.chill);
    }

    fn damage(&mut self, index: usize, amount: f32, source: u8, element: Element) {
        let e = &mut self.enemies[index];
        if !e.alive {
            return;
        }
        let dealt = amount.min(e.hp);
        e.hp -= amount;
        e.awake = true;
        if source != NO_SOURCE {
            e.last_source = source;
            if let Some(stats) = self.stats.get_mut(source as usize) {
                stats.damage += dealt;
            }
        }
        self.events.push(Event::Hit { enemy: index, element });
        if e.hp <= 0.0 {
            e.alive = false;
            self.death_queue.push((index, element));
        }
    }

    fn release_pending(&mut self, target: Option<(usize, u32)>, amount: f32) {
        if let Some((index, id)) = target {
            let e = &mut self.enemies[index];
            if e.id == id {
                e.pending = (e.pending - amount).max(0.0);
            }
        }
    }

    fn update_projectiles(&mut self) {
        let mut projectiles = std::mem::take(&mut self.projectiles);
        for p in &mut projectiles {
            let travel = p.vel * DT;
            p.pos += travel;
            p.range_left -= travel.len();
            let status = self.hit_status(p.source as usize);
            self.collect_near(p.pos, p.radius);
            let near = self.near.clone();
            for i in near {
                if !p.alive {
                    break;
                }
                let id = self.enemies[i].id;
                if p.hits.contains(&id) {
                    continue;
                }
                p.hits.push(id);
                if p.target.is_some_and(|(_, tid)| tid == id) {
                    self.release_pending(p.target.take(), p.damage);
                }
                self.strike(i, p.damage, p.source, p.element, status);
                if p.pierce == 0 {
                    p.alive = false;
                } else {
                    p.pierce -= 1;
                }
            }
            if p.range_left <= 0.0 || !inside_map(p.pos) {
                p.alive = false;
            }
            if !p.alive {
                self.release_pending(p.target.take(), p.damage);
            }
        }
        projectiles.retain(|p| p.alive);
        self.projectiles = projectiles;
    }

    fn update_pools(&mut self) {
        let mut pools = std::mem::take(&mut self.pools);
        for pool in &mut pools {
            pool.time_left -= DT;
            pool.tick -= DT;
            if pool.tick <= 0.0 {
                pool.tick += DOT_TICK;
                self.collect_near(pool.pos, pool.radius);
                let near = self.near.clone();
                let status = HitStatus { poison: pool.poison, burn: pool.burn, chill: 0.0 };
                for i in near {
                    self.strike(i, pool.damage, pool.source, Element::Poison, status);
                }
            }
        }
        pools.retain(|p| p.time_left > 0.0);
        self.pools = pools;
    }

    fn update_orbit(&mut self) {
        let Some(slot) = self.slots.iter().position(|s| s.spell == SpellKind::OrbitBlades) else {
            return;
        };
        self.orbit_angle = (self.orbit_angle + 3.2 * DT) % TAU;
        let status = self.hit_status(slot);
        let blades = self.blade_positions().unwrap_or([V2::ZERO; BLADES]);
        for blade in blades {
            self.collect_near(blade, BLADE_RADIUS);
            let near = self.near.clone();
            for i in near {
                if self.enemies[i].blade_cooldown <= 0.0 {
                    self.enemies[i].blade_cooldown = 0.35;
                    self.strike(i, SpellKind::OrbitBlades.damage(), slot as u8, Element::Physical, status);
                }
            }
        }
    }

    fn update_statuses(&mut self) {
        for i in 0..self.enemies.len() {
            let e = &mut self.enemies[i];
            if !e.alive || (e.poison == 0 && e.burn == 0 && e.chill <= 0.0 && e.blade_cooldown <= 0.0) {
                continue;
            }
            e.chill = (e.chill - DT).max(0.0);
            e.blade_cooldown -= DT;
            if e.poison > 0 {
                e.poison_time -= DT;
                if e.poison_time <= 0.0 {
                    e.poison = 0;
                }
            }
            e.dot_timer -= DT;
            if e.dot_timer <= 0.0 {
                e.dot_timer += DOT_TICK;
                // Poison: 2 damage per stack per second. Burn: 3 per stack, decays by 1 per tick.
                let damage = e.poison as f32 * 1.0 + e.burn as f32 * 1.5;
                let element = if e.poison as f32 >= e.burn as f32 * 1.5 { Element::Poison } else { Element::Fire };
                e.burn = e.burn.saturating_sub(1);
                let source = e.last_source;
                if damage > 0.0 {
                    self.damage(i, damage, source, element);
                }
            }
        }
    }

    fn process_deaths(&mut self) {
        let contagion = self.support_active(SupportKind::Contagion).is_some();
        let combustion = self.support_active(SupportKind::Combustion);
        let frenzy = self.support_active(SupportKind::Frenzy);
        let mut guard = 0;
        while let Some((index, element)) = self.death_queue.pop() {
            guard += 1;
            if guard > 4000 {
                self.death_queue.clear();
                break;
            }
            let e = self.enemies[index].clone();
            self.kills += 1;
            self.kill_times.push_back(self.time);
            if let Some(stats) = self.stats.get_mut(e.last_source as usize) {
                stats.kills += 1;
            }
            if frenzy.is_some_and(|slot| slot as u8 == e.last_source) {
                self.momentum = (self.momentum + 1).min(MOMENTUM_MAX);
                self.momentum_timer = 0.0;
            }
            if self.phase == Phase::Clearing {
                self.progress = (self.progress + e.kind.progress()).min(PROGRESS_NEEDED);
            }
            self.events.push(Event::Death { pos: e.pos, kind: e.kind, element, radius: e.radius });
            if e.kind == EnemyKind::Boss {
                self.phase = Phase::Victory;
                self.end_time = Some(self.time);
                continue;
            }
            if contagion && e.poison > 0 {
                self.collect_near(e.pos, 170.0);
                let mut near = self.near.clone();
                near.truncate(6);
                for i in near {
                    let status = HitStatus { poison: e.poison.max(3), ..HitStatus::default() };
                    self.strike(i, 0.0, e.last_source, Element::Poison, status);
                    self.events.push(Event::Spread { from: e.pos, to: self.enemies[i].pos });
                }
            }
            if let (Some(slot), true) = (combustion, e.burn > 0) {
                let radius = 110.0;
                let damage = 0.25 * e.max_hp + 6.0;
                self.collect_near(e.pos, radius);
                let near = self.near.clone();
                let status = HitStatus { burn: 3, ..HitStatus::default() };
                for i in near {
                    self.strike(i, damage, slot as u8, Element::Fire, status);
                }
                self.events.push(Event::Explosion { pos: e.pos, radius });
            }
        }
    }

    fn update_phase(&mut self) {
        match self.phase {
            Phase::Clearing if self.progress >= PROGRESS_NEEDED => {
                self.clear_time = Some(self.time);
                // Flatten the direction: the screen is wider than tall, and the HUD
                // covers the top and bottom edges.
                let raw = V2::from_angle(self.rng.range(0.0, TAU));
                let dir = V2::new(raw.x, raw.y * 0.5).norm();
                let pos = clamp_to_map(self.player.pos + dir * 320.0, 120.0);
                self.phase = Phase::BossIncoming { timer: BOSS_WARNING, pos };
            }
            Phase::BossIncoming { timer, pos } => {
                if timer - DT <= 0.0 {
                    let index = self.spawn(EnemyKind::Boss, pos, true);
                    self.boss = Some(BossState {
                        index,
                        circle_timer: 2.0,
                        burn_immunity: 0.0,
                        phase_two: false,
                        add_waves_left: 0,
                        add_timer: 0.0,
                    });
                    self.boss_start = Some(self.time);
                    self.phase = Phase::Boss;
                    self.events.push(Event::BossSpawned { pos });
                } else {
                    self.phase = Phase::BossIncoming { timer: timer - DT, pos };
                }
            }
            _ => {}
        }
    }

    /// Ember Warden: ember circles to dodge; at half health it becomes
    /// immune to Burn for 15 s and calls three waves of adds.
    fn update_boss(&mut self) {
        let Some(mut boss) = self.boss.take() else {
            return;
        };
        let e = &self.enemies[boss.index];
        if e.alive {
            let boss_pos = e.pos;
            let low = e.hp < e.max_hp * 0.5;
            boss.circle_timer -= DT;
            if boss.circle_timer <= 0.0 {
                boss.circle_timer = 3.5;
                let damage = 55.0 * self.damage_mult();
                let target = self.player.pos;
                self.telegraphs.push(Telegraph { pos: target, radius: 90.0, warning: 1.3, elapsed: 0.0, damage });
                for _ in 0..2 {
                    let offset = V2::from_angle(self.rng.range(0.0, TAU)) * self.rng.range(120.0, 220.0);
                    let pos = clamp_to_map(target + offset, 90.0);
                    self.telegraphs.push(Telegraph { pos, radius: 90.0, warning: 1.3, elapsed: 0.0, damage });
                }
            }
            if low && !boss.phase_two {
                boss.phase_two = true;
                boss.burn_immunity = 15.0;
                boss.add_waves_left = 3;
                boss.add_timer = 0.0;
                self.enemies[boss.index].burn = 0;
                self.events.push(Event::BossPhaseTwo { pos: boss_pos });
            }
            boss.burn_immunity = (boss.burn_immunity - DT).max(0.0);
            if boss.add_waves_left > 0 {
                boss.add_timer -= DT;
                if boss.add_timer <= 0.0 {
                    boss.add_timer = 4.0;
                    boss.add_waves_left -= 1;
                    for k in 0..10 {
                        let pos = clamp_to_map(boss_pos + V2::from_angle(k as f32 * TAU / 10.0) * 140.0, 20.0);
                        self.spawn(EnemyKind::Add, pos, true);
                    }
                }
            }
        }
        self.boss = Some(boss);
    }

    fn update_telegraphs(&mut self) {
        let player = self.player.pos;
        let mut hurt = 0.0;
        for t in &mut self.telegraphs {
            t.elapsed += DT;
            if t.elapsed >= t.warning {
                if player.dist(t.pos) < t.radius + PLAYER_RADIUS && self.player.invulnerable <= 0.0 {
                    hurt += t.damage;
                }
                self.events.push(Event::Detonate { pos: t.pos, radius: t.radius });
            }
        }
        self.telegraphs.retain(|t| t.elapsed < t.warning);
        if hurt > 0.0 {
            self.player.hp -= hurt;
            self.events.push(Event::PlayerHurt { amount: hurt });
        }
    }
}

fn area_mult(support: Option<SupportKind>) -> f32 {
    if support == Some(SupportKind::Amplify) {
        1.5
    } else {
        1.0
    }
}

fn clamp_to_map(pos: V2, radius: f32) -> V2 {
    V2::new(pos.x.clamp(radius, MAP_W - radius), pos.y.clamp(radius, MAP_H - radius))
}

fn inside_map(pos: V2) -> bool {
    pos.x >= 0.0 && pos.y >= 0.0 && pos.x <= MAP_W && pos.y <= MAP_H
}

/// A simple autopilot: dodge boss circles, keep some distance from what is
/// close, otherwise walk to the nearest enemy. Used for benchmarks and demos.
pub fn autopilot(world: &World) -> Input {
    let p = world.player.pos;
    for t in &world.telegraphs {
        let d = p - t.pos;
        if d.len() < t.radius + PLAYER_RADIUS + 10.0 {
            let away = if d.len_sq() < 1.0 { V2::new(1.0, 0.0) } else { d.norm() };
            return Input { move_dir: away, dash: t.warning - t.elapsed < 0.35 };
        }
    }
    let mut nearest: Option<(f32, V2)> = None;
    let mut crowd = V2::ZERO;
    for e in world.enemies.iter().filter(|e| e.alive) {
        let d2 = e.pos.dist_sq(p);
        if nearest.is_none_or(|(best, _)| d2 < best) {
            nearest = Some((d2, e.pos));
        }
        if d2 < 110.0 * 110.0 {
            crowd += (p - e.pos).norm();
        }
    }
    let Some((d2, target)) = nearest else {
        return Input::default();
    };
    let low = world.player.hp < world.player.max_hp * 0.35;
    if crowd != V2::ZERO && (low || d2 < 70.0 * 70.0) {
        // Kite sideways out of the crowd so spells keep hitting it.
        let away = crowd.norm();
        return Input { move_dir: (away + away.rotate(1.2) * 0.6).norm(), dash: low };
    }
    if d2 < 140.0 * 140.0 {
        return Input { move_dir: (target - p).norm().rotate(1.4), dash: false };
    }
    Input { move_dir: (target - p).norm(), dash: false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::presets;

    fn run(build: Build, seed: u64, max_seconds: f32) -> World {
        let mut world = World::new(build, seed, 0);
        while world.time < max_seconds && !matches!(world.phase, Phase::Victory | Phase::Defeat) {
            let input = autopilot(&world);
            world.step(&input);
            world.events.clear();
        }
        world
    }

    #[test]
    fn same_seed_same_map() {
        let a = World::new(presets()[0].1.clone(), 9, 0);
        let b = World::new(presets()[0].1.clone(), 9, 0);
        assert_eq!(a.enemies.len(), b.enemies.len());
        assert!(a.enemies.iter().zip(&b.enemies).all(|(x, y)| x.pos == y.pos && x.kind == y.kind));
    }

    #[test]
    fn a_strong_build_clears_the_map_and_kills_the_boss() {
        let build = presets().into_iter().find(|(n, _)| *n == "Contagion").unwrap().1;
        let world = run(build, 3, 600.0);
        assert_eq!(world.phase, Phase::Victory, "time {} progress {}", world.time, world.progress);
    }

    #[test]
    fn echo_casts_with_its_left_neighbor() {
        use crate::content::Slot;
        let build = Build::new(vec![
            Slot { spell: SpellKind::ArcaneBolt, support: None },
            Slot { spell: SpellKind::FrostNova, support: Some(SupportKind::Echo) },
        ]);
        let mut world = World::new(build, 1, 0);
        // Put a sleeping enemy right next to the player; the nova's own cooldown is spent.
        let pos = world.player.pos + V2::new(60.0, 0.0);
        world.spawn(EnemyKind::Brute, pos, false);
        world.slots[1].cooldown = 99.0;
        world.rebuild_grid();
        assert!(world.cast(0));
        let novas = world.events.iter().filter(|e| matches!(e, Event::Nova { .. })).count();
        assert_eq!(novas, 1);
    }

    #[test]
    fn momentum_caps_and_decays() {
        let mut world = World::new(presets()[0].1.clone(), 1, 0);
        world.momentum = MOMENTUM_MAX;
        world.momentum_timer = 0.0;
        for _ in 0..((MOMENTUM_DECAY / DT) as usize + 2) {
            world.update_momentum();
        }
        assert_eq!(world.momentum, 0);
    }
}
