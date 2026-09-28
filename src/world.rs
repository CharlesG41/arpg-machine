//! One map: packs to clear, a progress bar, then a boss.
use crate::content::{
    ArtifactKind, Build, Delivery, Family, Hit, Own, Scale, Scope, SelfCond, SpellId, Stat, SupportKind, Targeting,
    COOLDOWN_SCALE,
};
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
const DASH_COOLDOWN: f32 = 1.8;
const AGGRO_RANGE: f32 = 560.0;
/// Self-cast spells only fire while an awake enemy is this close.
const COMBAT_RANGE: f32 = 700.0;
const PACKS: usize = 110;
pub const PROGRESS_NEEDED: f32 = 500.0;
const BOSS_WARNING: f32 = 1.6;
pub const MOMENTUM_MAX: u8 = 20;
pub const MOMENTUM_DECAY: f32 = 2.0;
const MOMENTUM_RATE: f32 = 0.03;
const DOT_TICK: f32 = 0.5;
/// Source index for damage that no spell owns.
pub const NO_SOURCE: u8 = u8::MAX;
const ORBIT_RADIUS: f32 = 110.0;
const ORBIT_SPACING: f32 = 45.0;
const BLADE_RADIUS: f32 = 16.0;
pub const BLADES: usize = 3;
const MAX_SLOTS: usize = 10;
const PROJECTILE_RANGE: f32 = 650.0;
const TARGET_RANGE: f32 = 520.0;
/// Caster (pentagon): keeps its distance and fires slow orbs you can dodge.
const CASTER_RANGE: f32 = 320.0;
const CASTER_COOLDOWN: f32 = 3.0;
const SHOT_SPEED: f32 = 260.0;
/// Brute (hexagon): winds up a ground slam when you get close.
const SLAM_TRIGGER: f32 = 150.0;
const SLAM_RADIUS: f32 = 120.0;
const SLAM_WINDUP: f32 = 0.9;
const SLAM_COOLDOWN: f32 = 4.0;

/// Your base stats before artifacts and buffs.
pub const BASE_AD: f32 = 30.0;
pub const BASE_AP: f32 = 30.0;
pub const BASE_ARMOR: f32 = 20.0;
pub const BASE_MR: f32 = 20.0;
pub const BASE_HP: f32 = 300.0;

/// Enemy silhouettes: Grunt = square, Runner = triangle, Caster = pentagon,
/// Brute = hexagon, Boss = octagon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    Grunt,
    Runner,
    Caster,
    Brute,
    Boss,
    Add,
}

/// Each boss has its own stats.
#[derive(Clone, Copy, Debug)]
pub struct BossDef {
    pub name: &'static str,
    pub hp: f32,
    pub armor: f32,
    pub mr: f32,
    pub speed: f32,
    pub contact_dps: f32,
}

pub const EMBER_WARDEN: BossDef =
    BossDef { name: "Ember Warden", hp: 4000.0, armor: 25.0, mr: 40.0, speed: 62.0, contact_dps: 40.0 };

impl EnemyKind {
    fn hp(self) -> f32 {
        match self {
            EnemyKind::Grunt => 40.0,
            EnemyKind::Runner => 24.0,
            EnemyKind::Caster => 70.0,
            EnemyKind::Brute => 320.0,
            EnemyKind::Boss => EMBER_WARDEN.hp,
            EnemyKind::Add => 30.0,
        }
    }

    /// Physical and magic resistance (PR, MR).
    pub fn resists(self) -> (f32, f32) {
        match self {
            EnemyKind::Grunt | EnemyKind::Runner | EnemyKind::Add => (0.0, 0.0),
            EnemyKind::Caster => (0.0, 30.0),
            EnemyKind::Brute => (30.0, 0.0),
            EnemyKind::Boss => (EMBER_WARDEN.armor, EMBER_WARDEN.mr),
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            EnemyKind::Grunt => 13.0,
            EnemyKind::Runner => 10.0,
            EnemyKind::Caster => 15.0,
            EnemyKind::Brute => 24.0,
            EnemyKind::Boss => 46.0,
            EnemyKind::Add => 11.0,
        }
    }

    fn speed(self) -> f32 {
        match self {
            EnemyKind::Grunt => 95.0,
            EnemyKind::Runner => 165.0,
            EnemyKind::Caster => 80.0,
            EnemyKind::Brute => 70.0,
            EnemyKind::Boss => EMBER_WARDEN.speed,
            EnemyKind::Add => 120.0,
        }
    }

    fn contact_dps(self) -> f32 {
        match self {
            EnemyKind::Grunt => 12.0,
            EnemyKind::Runner => 8.0,
            EnemyKind::Caster => 6.0,
            EnemyKind::Brute => 25.0,
            EnemyKind::Boss => EMBER_WARDEN.contact_dps,
            EnemyKind::Add => 10.0,
        }
    }

    fn progress(self) -> f32 {
        match self {
            EnemyKind::Grunt | EnemyKind::Runner => 1.0,
            EnemyKind::Caster => 2.0,
            EnemyKind::Brute => 6.0,
            EnemyKind::Boss | EnemyKind::Add => 0.0,
        }
    }

    pub fn elite(self) -> bool {
        matches!(self, EnemyKind::Brute | EnemyKind::Boss)
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
    armor: f32,
    mr: f32,
    shred_armor: (f32, f32),
    shred_mr: (f32, f32),
    pub poison: u16,
    poison_time: f32,
    pub burn: u16,
    pub chill: f32,
    pub freeze: f32,
    pub bleed: f32,
    bleed_time: f32,
    pub weaken: f32,
    poison_amp: f32,
    dot_timer: f32,
    /// Damage already on its way (projectiles in flight): anti-overkill.
    pending: f32,
    pub last_source: u8,
    blade_cooldown: [f32; MAX_SLOTS],
    attack_timer: f32,
    /// Seconds left winding up an attack; the enemy stands still meanwhile.
    pub windup: f32,
}

impl Enemy {
    fn resist(&self, magic: bool, pen: f32) -> f32 {
        let (base, shred) = if magic { (self.mr, self.shred_mr.0) } else { (self.armor, self.shred_armor.0) };
        (base * (1.0 - pen) - shred).max(0.0)
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    pub pos: V2,
    pub hp: f32,
    pub barrier: f32,
    pub facing: V2,
    pub dash_cooldown: f32,
    pub dash_cooldown_max: f32,
    pub dash_time: f32,
    dash_dir: V2,
    pub invulnerable: f32,
    speed: f32,
}

/// Everything a hit needs to know about the cast that produced it.
#[derive(Clone, Copy, Debug)]
struct CastCtx {
    slot: usize,
    /// Damage multiplier fixed at cast time (boosts, falloff).
    mult: f32,
    /// Flat damage from growth, added to the first damage hit.
    growth: f32,
}

#[derive(Clone, Debug)]
pub struct Projectile {
    pub pos: V2,
    pub vel: V2,
    pub family: Family,
    ctx: CastCtx,
    estimate: f32,
    radius: f32,
    pierce: u8,
    range_left: f32,
    target: Option<(usize, u32)>,
    hits: Vec<u32>,
    pub alive: bool,
}

/// A slow orb fired by a Caster.
#[derive(Clone, Debug)]
pub struct EnemyShot {
    pub pos: V2,
    pub vel: V2,
    pub radius: f32,
    damage: f32,
    life: f32,
}

/// Repeated strikes on a spot (Zone delivery).
#[derive(Clone, Debug)]
pub struct Zone {
    pub pos: V2,
    pub radius: f32,
    pub family: Family,
    pub ticks_left: u8,
    pub interval: f32,
    pub timer: f32,
    ctx: CastCtx,
}

/// One ring of orbiting projectiles, from one Orbit spell on the bar.
#[derive(Clone, Copy, Debug)]
pub struct Orbit {
    pub slot: usize,
    pub family: Family,
    pub radius: f32,
    pub angle: f32,
    speed: f32,
}

#[derive(Clone, Debug)]
pub struct Telegraph {
    pub pos: V2,
    pub radius: f32,
    pub warning: f32,
    pub elapsed: f32,
    /// Who is attacking, so the renderer can color it.
    pub owner: EnemyKind,
    damage: f32,
    magic: bool,
}

#[derive(Clone, Debug)]
pub struct BossState {
    pub index: usize,
    pub def: BossDef,
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
    SelfCast { family: Family },
    Hit { enemy: usize, family: Family },
    Death { pos: V2, kind: EnemyKind, family: Family, radius: f32 },
    Nova { pos: V2, radius: f32, family: Family },
    Cone { pos: V2, dir: V2, range: f32, half_angle: f32, family: Family },
    Chain { points: Vec<V2>, family: Family },
    Blast { pos: V2, radius: f32, family: Family },
    Explosion { pos: V2, radius: f32 },
    Spread { from: V2, to: V2 },
    PlayerHurt { amount: f32 },
    BarrierHit,
    Heal,
    Dash { from: V2, dir: V2 },
    EnemyFire { pos: V2 },
    Detonate { pos: V2, radius: f32, owner: EnemyKind },
    BossSpawned { pos: V2 },
    BossPhaseTwo { pos: V2 },
}

#[derive(Clone, Copy, Debug)]
struct SlotState {
    spell: SpellId,
    support: Option<SupportKind>,
    cooldown: f32,
    casts: u32,
    haste: f32,
    haste_time: f32,
}

/// A timed effect on you, keyed by the slot and effect that cast it so a
/// recast refreshes instead of stacking.
#[derive(Clone, Copy, Debug)]
struct Timed {
    key: (usize, usize),
    kind: TimedKind,
    time: f32,
}

#[derive(Clone, Copy, Debug)]
enum TimedKind {
    Buff { stat: Stat, amount: f32 },
    Regen { per_sec: f32 },
    Thorns { damage: f32, magic: bool, chill: f32, tick: f32 },
}

/// Everything the equipped artifacts change, resolved once per map.
#[derive(Clone, Copy, Debug)]
struct Mods {
    burn_damage: f32,
    poison_damage: f32,
    poison_duration: f32,
    chilled_vulnerability: f32,
    cooldown_rate: f32,
    elite_damage: f32,
    heal_per_kill: f32,
    momentum_on_any_kill: bool,
    area: f32,
    barrier_on_dash: f32,
    extra_jumps: u8,
    extra_pierce: u8,
    ad: f32,
    ap: f32,
    armor: f32,
    mr: f32,
    max_hp: f32,
}

impl Mods {
    fn from(artifacts: &[ArtifactKind]) -> Self {
        let has = |a| artifacts.contains(&a);
        let pick = |a, yes: f32, no: f32| if has(a) { yes } else { no };
        Mods {
            burn_damage: pick(ArtifactKind::EmberHeart, 1.3, 1.0),
            poison_damage: pick(ArtifactKind::VenomGland, 1.2, 1.0),
            poison_duration: pick(ArtifactKind::VenomGland, 6.0, 4.0),
            chilled_vulnerability: pick(ArtifactKind::FrostSigil, 1.2, 1.0),
            cooldown_rate: pick(ArtifactKind::StormBattery, 1.15, 1.0),
            elite_damage: pick(ArtifactKind::HuntersLens, 1.3, 1.0),
            heal_per_kill: pick(ArtifactKind::SoulSiphon, 1.0, 0.0),
            momentum_on_any_kill: has(ArtifactKind::MomentumEngine),
            area: pick(ArtifactKind::WideLens, 1.25, 1.0),
            barrier_on_dash: pick(ArtifactKind::BarrierCharm, 25.0, 0.0),
            extra_jumps: u8::from(has(ArtifactKind::ChainLink)),
            extra_pierce: u8::from(has(ArtifactKind::ChainLink)),
            ad: pick(ArtifactKind::Warband, 15.0, 0.0),
            ap: pick(ArtifactKind::SagesRing, 15.0, 0.0),
            armor: pick(ArtifactKind::IronPlate, 25.0, 0.0),
            mr: pick(ArtifactKind::SpiritVeil, 25.0, 0.0),
            max_hp: pick(ArtifactKind::Bloodstone, 100.0, 0.0),
        }
    }
}

/// Damage after resistance: 100 / (100 + resistance).
fn mitigate(raw: f32, resist: f32) -> f32 {
    raw * 100.0 / (100.0 + resist.max(0.0))
}

pub struct World {
    pub seed: u64,
    pub tier: u32,
    rng: Rng,
    pub build: Build,
    slots: Vec<SlotState>,
    mods: Mods,
    timed: Vec<Timed>,
    pub player: Player,
    pub enemies: Vec<Enemy>,
    next_id: u32,
    grid: Grid,
    pub projectiles: Vec<Projectile>,
    pub enemy_shots: Vec<EnemyShot>,
    pub zones: Vec<Zone>,
    pub orbits: Vec<Orbit>,
    pub telegraphs: Vec<Telegraph>,
    pub time: f32,
    pub phase: Phase,
    pub progress: f32,
    pub momentum: u8,
    momentum_timer: f32,
    pub boss: Option<BossState>,
    pub stats: Vec<SpellStats>,
    pub kills: u32,
    kill_times: VecDeque<f32>,
    pub peak_kills_per_minute: f32,
    pub clear_time: Option<f32>,
    pub boss_start: Option<f32>,
    pub end_time: Option<f32>,
    pub events: Vec<Event>,
    death_queue: Vec<(usize, Family)>,
    moves: Vec<(usize, V2)>,
    near: Vec<usize>,
}

impl World {
    pub fn new(build: Build, seed: u64, tier: u32) -> Self {
        let slots = build
            .slots
            .iter()
            .take(MAX_SLOTS)
            .map(|s| SlotState { spell: s.spell, support: s.support, cooldown: 0.0, casts: 0, haste: 0.0, haste_time: 0.0 })
            .collect::<Vec<_>>();
        let mods = Mods::from(&build.artifacts);
        let quick = build.artifacts.contains(&ArtifactKind::QuickBoots);
        let stats = vec![SpellStats::default(); slots.len()];
        let mut orbits = Vec::new();
        for (i, s) in slots.iter().enumerate() {
            if s.spell.delivery() == Delivery::Orbit {
                let ring = orbits.len();
                orbits.push(Orbit {
                    slot: i,
                    family: s.spell.family(),
                    radius: (ORBIT_RADIUS + ring as f32 * ORBIT_SPACING) * area_mult(s.support) * mods.area,
                    angle: ring as f32 * 0.7,
                    speed: if ring % 2 == 0 { 3.2 } else { -2.6 },
                });
            }
        }
        let start = V2::new(MAP_W / 2.0, MAP_H / 2.0);
        let max_hp = BASE_HP + mods.max_hp;
        let mut world = World {
            seed,
            tier,
            rng: Rng::new(seed),
            build,
            slots,
            mods,
            timed: Vec::new(),
            player: Player {
                pos: start,
                hp: max_hp,
                barrier: 0.0,
                facing: V2::new(1.0, 0.0),
                dash_cooldown: 0.0,
                dash_cooldown_max: DASH_COOLDOWN - if quick { 0.5 } else { 0.0 },
                dash_time: 0.0,
                dash_dir: V2::ZERO,
                invulnerable: 0.0,
                speed: PLAYER_SPEED * if quick { 1.15 } else { 1.0 },
            },
            enemies: Vec::new(),
            next_id: 0,
            grid: Grid::new(MAP_W, MAP_H, 80.0),
            projectiles: Vec::new(),
            enemy_shots: Vec::new(),
            zones: Vec::new(),
            orbits,
            telegraphs: Vec::new(),
            time: 0.0,
            phase: Phase::Clearing,
            progress: 0.0,
            momentum: 0,
            momentum_timer: 0.0,
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
        let (armor, mr) = kind.resists();
        // Stagger first attacks so a pack does not fire in unison.
        let attack_timer = self.rng.range(1.0, CASTER_COOLDOWN);
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
            armor,
            mr,
            shred_armor: (0.0, 0.0),
            shred_mr: (0.0, 0.0),
            poison: 0,
            poison_time: 0.0,
            burn: 0,
            chill: 0.0,
            freeze: 0.0,
            bleed: 0.0,
            bleed_time: 0.0,
            weaken: 0.0,
            poison_amp: 0.0,
            dot_timer: 0.0,
            pending: 0.0,
            last_source: NO_SOURCE,
            blade_cooldown: [0.0; MAX_SLOTS],
            attack_timer,
            windup: 0.0,
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
                let roll = self.rng.f32();
                let kind = if roll < 0.1 {
                    EnemyKind::Caster
                } else if roll < 0.32 {
                    EnemyKind::Runner
                } else {
                    EnemyKind::Grunt
                };
                let offset = V2::from_angle(self.rng.range(0.0, TAU)) * self.rng.range(0.0, spread);
                self.spawn(kind, clamp_to_map(center + offset, 20.0), false);
            }
            if self.rng.chance(0.3) {
                self.spawn(EnemyKind::Brute, center, false);
            }
        }
    }

    // ---- stats and queries --------------------------------------------------

    pub fn stat(&self, stat: Stat) -> f32 {
        let base = match stat {
            Stat::Ad => BASE_AD + self.mods.ad,
            Stat::Ap => BASE_AP + self.mods.ap,
            Stat::Armor => BASE_ARMOR + self.mods.armor,
            Stat::Mr => BASE_MR + self.mods.mr,
            Stat::DamagePct => 0.0,
        };
        let buffs: f32 = self
            .timed
            .iter()
            .filter_map(|t| match t.kind {
                TimedKind::Buff { stat: s, amount } if s == stat => Some(amount),
                _ => None,
            })
            .sum();
        (base + buffs).max(0.0)
    }

    pub fn max_hp(&self) -> f32 {
        BASE_HP + self.mods.max_hp
    }

    fn eval(&self, s: Scale) -> f32 {
        s.flat
            + s.ad * self.stat(Stat::Ad)
            + s.ap * self.stat(Stat::Ap)
            + s.max_hp * self.max_hp()
            + s.armor * self.stat(Stat::Armor)
            + s.mr * self.stat(Stat::Mr)
    }

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
        (1.0 + MOMENTUM_RATE * self.momentum as f32) * self.mods.cooldown_rate
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

    pub fn thorns_active(&self) -> bool {
        self.timed.iter().any(|t| matches!(t.kind, TimedKind::Thorns { .. }))
    }

    pub fn blade_positions(&self, orbit: &Orbit) -> [V2; BLADES] {
        std::array::from_fn(|k| self.player.pos + V2::from_angle(orbit.angle + k as f32 * TAU / BLADES as f32) * orbit.radius)
    }

    fn support_active(&self, support: SupportKind) -> Option<usize> {
        self.slots.iter().position(|s| s.support == Some(support))
    }

    fn in_combat(&self) -> bool {
        let p = self.player.pos;
        self.enemies.iter().any(|e| e.alive && e.awake && e.pos.dist_sq(p) < COMBAT_RANGE * COMBAT_RANGE)
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

    /// Best targets in range by a targeting rule, skipping enemies that the
    /// damage already on its way will kill. Hunter puts bosses, then rares, first.
    fn pick_targets(&self, from: V2, range: f32, count: usize, rule: Targeting, hunter: bool) -> Vec<usize> {
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
            let score = match rule {
                Targeting::Nearest | Targeting::Densest => d2,
                // Percent first; distance breaks ties so packs of equals still pick the nearest.
                Targeting::LowestHealth => e.hp / e.max_hp * 1e7 + d2,
                Targeting::HighestHealth => -e.hp * 1e3 + d2 * 1e-3,
            };
            candidates.push((priority + score, id as usize));
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
        self.update_timed();
        self.move_enemies();
        self.rebuild_grid();
        self.contact_damage();
        self.enemy_attacks();
        self.update_enemy_shots();
        self.update_momentum();
        self.update_spells();
        self.update_projectiles();
        self.update_zones();
        self.update_orbits();
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
        let max_hp = self.max_hp();
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
            p.dash_cooldown = p.dash_cooldown_max;
            p.invulnerable = DASH_TIME + 0.1;
            p.barrier = (p.barrier + self.mods.barrier_on_dash).min(max_hp);
            self.events.push(Event::Dash { from: p.pos, dir: p.dash_dir });
        }
        if p.dash_time > 0.0 {
            p.pos += p.dash_dir * (DASH_DISTANCE / DASH_TIME * DT);
            p.dash_time -= DT;
        } else {
            p.pos += dir * (p.speed * DT);
        }
        p.pos = clamp_to_map(p.pos, PLAYER_RADIUS);
        p.hp = (p.hp + 3.0 * DT).min(max_hp);
    }

    /// Buffs, regeneration and thorns on you.
    fn update_timed(&mut self) {
        let mut regen = 0.0;
        let mut thorns = Vec::new();
        for t in &mut self.timed {
            t.time -= DT;
            match &mut t.kind {
                TimedKind::Regen { per_sec } => regen += *per_sec * DT,
                TimedKind::Thorns { damage, magic, chill, tick } => {
                    *tick -= DT;
                    if *tick <= 0.0 {
                        *tick += DOT_TICK;
                        thorns.push((t.key.0, *damage, *magic, *chill));
                    }
                }
                TimedKind::Buff { .. } => {}
            }
        }
        self.timed.retain(|t| t.time > 0.0);
        self.heal(regen);
        for (slot, damage, magic, chill) in thorns {
            self.collect_near(self.player.pos, PLAYER_RADIUS + 14.0);
            let near = self.near.clone();
            let family = self.slots[slot].spell.family();
            for i in near {
                if damage > 0.0 {
                    self.deal(i, damage, magic, 0.0, slot as u8, family);
                }
                if chill > 0.0 {
                    let e = &mut self.enemies[i];
                    e.chill = e.chill.max(chill);
                }
            }
        }
    }

    fn heal(&mut self, amount: f32) {
        if amount > 0.0 {
            self.player.hp = (self.player.hp + amount).min(self.max_hp());
        }
    }

    /// Barrier absorbs damage before Health; resistances apply first.
    fn hurt_player(&mut self, raw: f32, magic: bool) {
        let resist = self.stat(if magic { Stat::Mr } else { Stat::Armor });
        let amount = mitigate(raw, resist);
        let absorbed = amount.min(self.player.barrier);
        self.player.barrier -= absorbed;
        if absorbed > 0.0 {
            self.events.push(Event::BarrierHit);
        }
        let rest = amount - absorbed;
        if rest > 0.0 {
            self.player.hp -= rest;
            self.events.push(Event::PlayerHurt { amount: rest });
        }
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
            if e.freeze > 0.0 {
                continue;
            }
            let to_player = player - e.pos;
            let dist = to_player.len();
            let speed = e.kind.speed() * if e.chill > 0.0 { 0.5 } else { 1.0 };
            // Casters hold at range; everything stands still while winding up.
            let stop_at = if e.kind == EnemyKind::Caster { CASTER_RANGE } else { PLAYER_RADIUS + e.radius };
            let mut step = if e.windup <= 0.0 && dist > stop_at {
                to_player.norm() * (speed * DT)
            } else if e.kind == EnemyKind::Caster && dist < CASTER_RANGE * 0.7 {
                to_player.norm() * (-speed * 0.6 * DT)
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
        let dps: f32 = self
            .near
            .iter()
            .map(|&i| {
                let e = &self.enemies[i];
                if e.freeze > 0.0 {
                    0.0
                } else {
                    e.kind.contact_dps() * if e.weaken > 0.0 { 0.8 } else { 1.0 }
                }
            })
            .sum();
        if dps > 0.0 {
            self.hurt_player(dps * self.damage_mult() * DT, false);
        }
    }

    /// Casters fire orbs; Brutes wind up a slam. Both are telegraphed and dodgeable.
    fn enemy_attacks(&mut self) {
        let player = self.player.pos;
        let damage_mult = self.damage_mult();
        for i in 0..self.enemies.len() {
            let e = &mut self.enemies[i];
            if !e.alive || !e.awake || e.freeze > 0.0 || !matches!(e.kind, EnemyKind::Caster | EnemyKind::Brute) {
                continue;
            }
            let weak = if e.weaken > 0.0 { 0.8 } else { 1.0 };
            e.windup = (e.windup - DT).max(0.0);
            e.attack_timer -= DT * if e.chill > 0.0 { 0.5 } else { 1.0 };
            let dist = e.pos.dist(player);
            match e.kind {
                EnemyKind::Caster if e.attack_timer <= 0.0 && dist < AGGRO_RANGE => {
                    e.attack_timer = CASTER_COOLDOWN;
                    let dir = (player - e.pos).norm();
                    let pos = e.pos + dir * e.radius;
                    self.enemy_shots.push(EnemyShot {
                        pos,
                        vel: dir * SHOT_SPEED,
                        radius: 10.0,
                        damage: 18.0 * damage_mult * weak,
                        life: 3.0,
                    });
                    self.events.push(Event::EnemyFire { pos });
                }
                EnemyKind::Brute if e.attack_timer <= 0.0 && dist < SLAM_TRIGGER => {
                    e.attack_timer = SLAM_COOLDOWN;
                    e.windup = SLAM_WINDUP;
                    let pos = e.pos;
                    self.telegraphs.push(Telegraph {
                        pos,
                        radius: SLAM_RADIUS,
                        warning: SLAM_WINDUP,
                        elapsed: 0.0,
                        owner: EnemyKind::Brute,
                        damage: 35.0 * damage_mult * weak,
                        magic: false,
                    });
                }
                _ => {}
            }
        }
    }

    fn update_enemy_shots(&mut self) {
        let player = self.player.pos;
        let mut hurt = 0.0;
        let invulnerable = self.player.invulnerable > 0.0;
        for shot in &mut self.enemy_shots {
            shot.pos += shot.vel * DT;
            shot.life -= DT;
            if shot.pos.dist(player) < shot.radius + PLAYER_RADIUS {
                if !invulnerable {
                    hurt += shot.damage;
                }
                shot.life = 0.0;
            }
        }
        self.enemy_shots.retain(|s| s.life > 0.0 && inside_map(s.pos));
        if hurt > 0.0 {
            self.hurt_player(hurt, true);
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
            let s = &mut self.slots[i];
            s.haste_time = (s.haste_time - DT).max(0.0);
            if !spell.casts() {
                continue;
            }
            let haste = if s.haste_time > 0.0 { 1.0 + s.haste } else { 1.0 };
            s.cooldown -= DT * rate * haste;
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

    fn area(&self, support: Option<SupportKind>) -> f32 {
        area_mult(support) * self.mods.area
    }

    fn holds(&self, cond: SelfCond) -> bool {
        let hp = self.player.hp / self.max_hp();
        match cond {
            SelfCond::BelowHealth(p) => hp < p,
            SelfCond::AboveHealth(p) => hp > p,
            SelfCond::FullHealth => hp >= 0.999,
            SelfCond::NoBarrier => self.player.barrier <= 0.0,
        }
    }

    /// Rough damage one hit of this cast will deal, for anti-overkill.
    fn estimate(&self, ctx: CastCtx) -> f32 {
        let def = self.slots[ctx.slot].spell.def();
        let raw: f32 = def
            .hits
            .iter()
            .map(|h| match h {
                Hit::Damage { scale, .. } => self.eval(*scale),
                _ => 0.0,
            })
            .sum();
        (raw + ctx.growth) * ctx.mult
    }

    /// Casts a slot; an Echo on the right neighbor casts it too. Returns false when
    /// there was nothing to cast at.
    fn cast(&mut self, slot: usize) -> bool {
        let SlotState { spell, support, casts, .. } = self.slots[slot];
        let def = spell.def();
        let hunter = support == Some(SupportKind::Hunter);
        let multishot = support == Some(SupportKind::Multishot);
        let boost = def.boost.filter(|(c, _)| self.holds(*c)).map_or(1.0, |(_, m)| m);
        let ctx = CastCtx {
            slot,
            mult: boost * if multishot { 0.8 } else { 1.0 },
            growth: def.growth * casts as f32,
        };
        let family = def.family;
        let origin = self.player.pos;
        let cast = match def.delivery {
            Delivery::Projectile { count, pierce, target } => {
                let count = count as usize + if multishot { 2 } else { 0 };
                let targets = self.pick_targets(origin, PROJECTILE_RANGE, count, target, hunter);
                if targets.is_empty() {
                    false
                } else {
                    let estimate = self.estimate(ctx);
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
                            self.enemies[t].pending += estimate;
                        }
                        let pierce = pierce + if support == Some(SupportKind::Pierce) { 3 } else { 0 } + self.mods.extra_pierce;
                        self.projectiles.push(Projectile {
                            pos: origin,
                            vel: aim.norm() * 900.0,
                            family,
                            ctx,
                            estimate,
                            radius: 7.0,
                            pierce,
                            range_left: PROJECTILE_RANGE + 70.0,
                            target: target.map(|t| (t, self.enemies[t].id)),
                            hits: Vec::new(),
                            alive: true,
                        });
                    }
                    true
                }
            }
            Delivery::Chain { jumps, falloff } => {
                let jumps = jumps + if multishot { 2 } else { 0 } + self.mods.extra_jumps;
                match self.pick_targets(origin, TARGET_RANGE, 1, Targeting::Nearest, hunter).first() {
                    None => false,
                    Some(&first) => {
                        let mut points = vec![origin];
                        let mut hit_ids = Vec::new();
                        let mut current = first;
                        let mut mult = ctx.mult;
                        for _ in 0..=jumps {
                            let pos = self.enemies[current].pos;
                            points.push(pos);
                            hit_ids.push(self.enemies[current].id);
                            self.spell_hit(CastCtx { mult, ..ctx }, current);
                            mult *= falloff;
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
                        self.events.push(Event::Chain { points, family });
                        true
                    }
                }
            }
            Delivery::Nova { radius } => {
                let radius = radius * self.area(support);
                self.collect_near(origin, radius * 0.9);
                if self.near.is_empty() {
                    false
                } else {
                    self.collect_near(origin, radius);
                    for i in self.near.clone() {
                        self.spell_hit(ctx, i);
                    }
                    self.events.push(Event::Nova { pos: origin, radius, family });
                    true
                }
            }
            Delivery::Cone { range } => {
                let range = range * (1.0 + (self.area(support) - 1.0) * 0.7);
                let half_angle: f32 = 0.61;
                match self.pick_targets(origin, range * 1.1, 1, Targeting::Nearest, hunter).first() {
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
                            self.spell_hit(ctx, i);
                        }
                        self.events.push(Event::Cone { pos: origin, dir, range, half_angle, family });
                        true
                    }
                }
            }
            Delivery::Blast { radius, target } => {
                let radius = radius * self.area(support);
                match self.aim_point(origin, target, radius, hunter) {
                    None => false,
                    Some(pos) => {
                        self.collect_near(pos, radius);
                        for i in self.near.clone() {
                            self.spell_hit(ctx, i);
                        }
                        self.events.push(Event::Blast { pos, radius, family });
                        true
                    }
                }
            }
            Delivery::Zone { radius, ticks, interval, target } => {
                let radius = radius * self.area(support);
                match self.aim_point(origin, target, radius, hunter) {
                    None => false,
                    Some(pos) => {
                        self.zones.push(Zone { pos, radius, family, ticks_left: ticks, interval, timer: interval, ctx });
                        true
                    }
                }
            }
            Delivery::Orbit => false,
            Delivery::SelfCast => {
                if self.in_combat() {
                    self.events.push(Event::SelfCast { family });
                    true
                } else {
                    false
                }
            }
        };
        if cast {
            self.slots[slot].casts += 1;
            self.apply_own(slot, def.own, 0);
            self.events.push(Event::Cast { slot });
            if self.slots.get(slot + 1).is_some_and(|s| s.support == Some(SupportKind::Echo)) {
                self.cast(slot + 1);
            }
        }
        cast
    }

    /// Where a Blast or Zone lands.
    fn aim_point(&mut self, from: V2, rule: Targeting, radius: f32, hunter: bool) -> Option<V2> {
        if rule == Targeting::Densest && !hunter {
            return self.densest_point(from, TARGET_RANGE, radius);
        }
        self.pick_targets(from, TARGET_RANGE, 1, rule, hunter).first().map(|&t| self.enemies[t].pos)
    }

    fn densest_point(&mut self, from: V2, range: f32, radius: f32) -> Option<V2> {
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

    /// Effects on you and your bar. `depth` guards nested conditions.
    fn apply_own(&mut self, slot: usize, list: &'static [Own], depth: usize) {
        for (k, own) in list.iter().enumerate() {
            let key = (slot, k + depth * 16);
            match *own {
                Own::Barrier(s) => {
                    let amount = self.eval(s);
                    self.player.barrier = (self.player.barrier + amount).min(self.max_hp());
                }
                Own::Heal(s) => {
                    let amount = self.eval(s);
                    self.heal(amount);
                    self.events.push(Event::Heal);
                }
                Own::Regen { per_sec, secs } => {
                    let per_sec = self.eval(per_sec);
                    self.set_timed(key, TimedKind::Regen { per_sec }, secs);
                }
                Own::Buff { stat, amount, secs } => self.set_timed(key, TimedKind::Buff { stat, amount }, secs),
                Own::Haste { scope, rate, secs } => {
                    for i in self.scope(slot, scope) {
                        self.slots[i].haste = rate;
                        self.slots[i].haste_time = secs;
                    }
                }
                Own::Charge { scope, secs } => {
                    for i in self.scope(slot, scope) {
                        let s = &mut self.slots[i];
                        s.cooldown = (s.cooldown - secs * COOLDOWN_SCALE).max(0.0);
                    }
                }
                Own::PayHealth(p) => {
                    let pay = self.player.hp * p;
                    self.player.hp = (self.player.hp - pay).max(1.0);
                }
                Own::Thorns { scale, magic, chill, secs } => {
                    let damage = self.eval(scale);
                    self.set_timed(key, TimedKind::Thorns { damage, magic, chill, tick: 0.0 }, secs);
                }
                Own::If(cond, inner) => {
                    if depth < 3 && self.holds(cond) {
                        self.apply_own(slot, inner, depth + 1);
                    }
                }
            }
        }
    }

    fn set_timed(&mut self, key: (usize, usize), kind: TimedKind, time: f32) {
        self.timed.retain(|t| t.key != key);
        self.timed.push(Timed { key, kind, time });
    }

    fn scope(&self, slot: usize, scope: Scope) -> Vec<usize> {
        let n = self.slots.len();
        match scope {
            Scope::Left => (slot > 0).then(|| slot - 1).into_iter().collect(),
            Scope::Right => (slot + 1 < n).then_some(slot + 1).into_iter().collect(),
            Scope::Adjacent => [slot.checked_sub(1), (slot + 1 < n).then_some(slot + 1)].into_iter().flatten().collect(),
            Scope::Bar => (0..n).collect(),
        }
    }

    /// Runs a spell's hit effects on one enemy.
    fn spell_hit(&mut self, ctx: CastCtx, index: usize) {
        if !self.enemies[index].alive {
            return;
        }
        let slot = ctx.slot;
        let spell = self.slots[slot].spell;
        let support = self.slots[slot].support;
        let def = spell.def();
        let family = def.family;
        let source = slot as u8;
        let elite = self.enemies[index].kind.elite();
        let elite_bonus: f32 = def
            .hits
            .iter()
            .map(|h| match h {
                Hit::EliteBonus(m) if elite => *m,
                _ => 1.0,
            })
            .product();
        let mult = ctx.mult * elite_bonus * (1.0 + self.stat(Stat::DamagePct) / 100.0);
        // Conditions read the target as it was before this hit.
        let (was_chilled, was_burning, was_poisoned) = {
            let e = &self.enemies[index];
            (e.chill > 0.0, e.burn > 0, e.poison > 0)
        };
        let burn_immune = self.boss.as_ref().is_some_and(|b| b.index == index && b.burn_immunity > 0.0);
        let poison_duration = self.mods.poison_duration;
        let mut growth = ctx.growth;
        let mut dealt = 0.0;
        for hit in def.hits {
            match *hit {
                Hit::Damage { scale, magic, pen } => {
                    let raw = (self.eval(scale) + growth) * mult;
                    growth = 0.0;
                    dealt += self.deal(index, raw, magic, pen, source, family);
                }
                Hit::Burn(n) => self.add_burn(index, n, burn_immune),
                Hit::Poison(n) => self.add_poison(index, n, poison_duration),
                Hit::Chill(s) => {
                    let e = &mut self.enemies[index];
                    e.chill = e.chill.max(s);
                }
                Hit::Freeze(s) => {
                    let e = &mut self.enemies[index];
                    e.freeze = e.freeze.max(s);
                }
                Hit::Bleed { scale, secs } => {
                    let dps = self.eval(scale);
                    let e = &mut self.enemies[index];
                    e.bleed = e.bleed.max(dps);
                    e.bleed_time = secs;
                    e.last_source = source;
                }
                Hit::ShredArmor(x) => {
                    let e = &mut self.enemies[index];
                    e.shred_armor = (e.shred_armor.0.max(x), 4.0);
                }
                Hit::ShredMr(x) => {
                    let e = &mut self.enemies[index];
                    e.shred_mr = (e.shred_mr.0.max(x), 4.0);
                }
                Hit::Weaken(s) => {
                    let e = &mut self.enemies[index];
                    e.weaken = e.weaken.max(s);
                }
                Hit::PoisonAmp(s) => {
                    let e = &mut self.enemies[index];
                    e.poison_amp = e.poison_amp.max(s);
                }
                Hit::ConsumeChill(scale) => {
                    if was_chilled {
                        self.enemies[index].chill = 0.0;
                        let raw = self.eval(scale) * mult;
                        dealt += self.deal(index, raw, true, 0.0, source, family);
                    }
                }
                Hit::ConsumeBurn { scale, per_stack } => {
                    let stacks = std::mem::take(&mut self.enemies[index].burn) as f32;
                    let raw = (self.eval(scale) + per_stack * stacks) * mult;
                    dealt += self.deal(index, raw, true, 0.0, source, family);
                }
                Hit::ConsumePoison { scale, per_stack } => {
                    let stacks = std::mem::take(&mut self.enemies[index].poison) as f32;
                    let raw = (self.eval(scale) + per_stack * stacks) * mult;
                    dealt += self.deal(index, raw, true, 0.0, source, family);
                }
                Hit::SpreadBurn { radius } => {
                    let (burn, pos) = (self.enemies[index].burn, self.enemies[index].pos);
                    if burn == 0 {
                        self.add_burn(index, 3, burn_immune);
                    } else {
                        self.collect_near(pos, radius);
                        for i in self.near.clone() {
                            if i != index {
                                let e = &mut self.enemies[i];
                                e.burn = e.burn.max(burn);
                                e.last_source = source;
                                self.events.push(Event::Spread { from: pos, to: self.enemies[i].pos });
                            }
                        }
                    }
                }
                Hit::BurnIfBurning(n) => {
                    if was_burning {
                        self.add_burn(index, n, burn_immune);
                    }
                }
                Hit::IfPoisonedAndBurning(scale) => {
                    if was_poisoned && was_burning {
                        let raw = self.eval(scale) * mult;
                        dealt += self.deal(index, raw, true, 0.0, source, family);
                    }
                }
                Hit::Lifesteal(p) => self.heal(dealt * p),
                Hit::HealOnKill(p) => {
                    if !self.enemies[index].alive {
                        let amount = self.max_hp() * p;
                        self.heal(amount);
                        self.events.push(Event::Heal);
                    }
                }
                Hit::EliteBonus(_) => {}
            }
        }
        match support {
            Some(SupportKind::Contagion) => self.add_poison(index, 3, poison_duration),
            Some(SupportKind::Combustion) => self.add_burn(index, 3, burn_immune),
            _ => {}
        }
        let e = &mut self.enemies[index];
        e.last_source = source;
    }

    fn add_burn(&mut self, index: usize, n: u16, immune: bool) {
        if !immune {
            let e = &mut self.enemies[index];
            e.burn = e.burn.saturating_add(n).min(60);
        }
    }

    fn add_poison(&mut self, index: usize, n: u16, duration: f32) {
        let e = &mut self.enemies[index];
        e.poison = e.poison.saturating_add(n).min(80);
        e.poison_time = duration;
    }

    /// Direct spell damage: resistances, then the artifact multipliers.
    fn deal(&mut self, index: usize, raw: f32, magic: bool, pen: f32, source: u8, family: Family) -> f32 {
        let e = &self.enemies[index];
        let mut amount = mitigate(raw, e.resist(magic, pen));
        if e.chill > 0.0 {
            amount *= self.mods.chilled_vulnerability;
        }
        if e.kind.elite() {
            amount *= self.mods.elite_damage;
        }
        self.damage(index, amount, source, family)
    }

    /// Final damage (after resistances). Returns the Health actually removed.
    fn damage(&mut self, index: usize, amount: f32, source: u8, family: Family) -> f32 {
        let e = &mut self.enemies[index];
        if !e.alive || amount <= 0.0 {
            return 0.0;
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
        self.events.push(Event::Hit { enemy: index, family });
        if e.hp <= 0.0 {
            e.alive = false;
            self.death_queue.push((index, family));
        }
        dealt
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
            self.collect_near(p.pos, p.radius);
            for i in self.near.clone() {
                if !p.alive {
                    break;
                }
                let id = self.enemies[i].id;
                if p.hits.contains(&id) || !self.enemies[i].alive {
                    continue;
                }
                p.hits.push(id);
                if p.target.is_some_and(|(_, tid)| tid == id) {
                    self.release_pending(p.target.take(), p.estimate);
                }
                self.spell_hit(p.ctx, i);
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
                self.release_pending(p.target.take(), p.estimate);
            }
        }
        projectiles.retain(|p| p.alive);
        self.projectiles = projectiles;
    }

    fn update_zones(&mut self) {
        let mut zones = std::mem::take(&mut self.zones);
        for zone in &mut zones {
            zone.timer -= DT;
            if zone.timer <= 0.0 && zone.ticks_left > 0 {
                zone.timer += zone.interval;
                zone.ticks_left -= 1;
                self.collect_near(zone.pos, zone.radius);
                for i in self.near.clone() {
                    self.spell_hit(zone.ctx, i);
                }
                self.events.push(Event::Blast { pos: zone.pos, radius: zone.radius, family: zone.family });
            }
        }
        zones.retain(|z| z.ticks_left > 0);
        self.zones = zones;
    }

    fn update_orbits(&mut self) {
        for k in 0..self.orbits.len() {
            let orbit = {
                let o = &mut self.orbits[k];
                o.angle = (o.angle + o.speed * DT).rem_euclid(TAU);
                *o
            };
            let slot = orbit.slot;
            let ctx = CastCtx { slot, mult: 1.0, growth: 0.0 };
            for blade in self.blade_positions(&orbit) {
                self.collect_near(blade, BLADE_RADIUS);
                for i in self.near.clone() {
                    if self.enemies[i].blade_cooldown[slot] <= 0.0 {
                        self.enemies[i].blade_cooldown[slot] = 0.35;
                        self.spell_hit(ctx, i);
                    }
                }
            }
        }
    }

    fn update_statuses(&mut self) {
        let mods = self.mods;
        let orbits = !self.orbits.is_empty();
        for i in 0..self.enemies.len() {
            let e = &mut self.enemies[i];
            if !e.alive {
                continue;
            }
            let idle = e.poison == 0
                && e.burn == 0
                && e.chill <= 0.0
                && e.freeze <= 0.0
                && e.bleed_time <= 0.0
                && e.weaken <= 0.0
                && e.poison_amp <= 0.0
                && e.shred_armor.1 <= 0.0
                && e.shred_mr.1 <= 0.0;
            if idle && !orbits {
                continue;
            }
            e.chill = (e.chill - DT).max(0.0);
            e.freeze = (e.freeze - DT).max(0.0);
            e.weaken = (e.weaken - DT).max(0.0);
            e.poison_amp = (e.poison_amp - DT).max(0.0);
            for shred in [&mut e.shred_armor, &mut e.shred_mr] {
                shred.1 -= DT;
                if shred.1 <= 0.0 {
                    *shred = (0.0, 0.0);
                }
            }
            for cd in &mut e.blade_cooldown {
                *cd -= DT;
            }
            if e.poison > 0 {
                e.poison_time -= DT;
                if e.poison_time <= 0.0 {
                    e.poison = 0;
                }
            }
            e.bleed_time -= DT;
            if e.bleed_time <= 0.0 {
                e.bleed = 0.0;
            }
            e.dot_timer -= DT;
            if e.dot_timer <= 0.0 {
                e.dot_timer += DOT_TICK;
                // Per tick: Poison 1 per stack, Burn 1.5 per stack (then −1 stack),
                // Bleed half its damage per second.
                let amp = if e.poison_amp > 0.0 { 1.6 } else { 1.0 };
                let poison = e.poison as f32 * mods.poison_damage * amp;
                let burn = e.burn as f32 * 1.5 * mods.burn_damage;
                let bleed = e.bleed * DOT_TICK;
                let family = if bleed > poison.max(burn) {
                    Family::Blood
                } else if poison >= burn {
                    Family::Poison
                } else {
                    Family::Fire
                };
                e.burn = e.burn.saturating_sub(1);
                let source = e.last_source;
                if poison + burn + bleed > 0.0 {
                    self.damage(i, poison + burn + bleed, source, family);
                }
            }
        }
    }

    fn process_deaths(&mut self) {
        let contagion = self.support_active(SupportKind::Contagion).is_some();
        let combustion = self.support_active(SupportKind::Combustion);
        let frenzy = self.support_active(SupportKind::Frenzy);
        let mut guard = 0;
        while let Some((index, family)) = self.death_queue.pop() {
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
            if self.mods.momentum_on_any_kill || frenzy.is_some_and(|slot| slot as u8 == e.last_source) {
                self.momentum = (self.momentum + 1).min(MOMENTUM_MAX);
                self.momentum_timer = 0.0;
            }
            self.heal(self.mods.heal_per_kill);
            if self.phase == Phase::Clearing {
                self.progress = (self.progress + e.kind.progress()).min(PROGRESS_NEEDED);
            }
            self.events.push(Event::Death { pos: e.pos, kind: e.kind, family, radius: e.radius });
            if e.kind == EnemyKind::Boss {
                self.phase = Phase::Victory;
                self.end_time = Some(self.time);
                continue;
            }
            if contagion && e.poison > 0 {
                self.collect_near(e.pos, 170.0);
                let mut near = self.near.clone();
                near.truncate(6);
                let duration = self.mods.poison_duration;
                for i in near {
                    self.add_poison(i, e.poison.max(3), duration);
                    self.enemies[i].last_source = e.last_source;
                    self.events.push(Event::Spread { from: e.pos, to: self.enemies[i].pos });
                }
            }
            if let (Some(slot), true) = (combustion, e.burn > 0) {
                let radius = 110.0 * self.mods.area;
                let damage = 0.25 * e.max_hp + 6.0;
                self.collect_near(e.pos, radius);
                let immune_boss = self.boss.as_ref().filter(|b| b.burn_immunity > 0.0).map(|b| b.index);
                for i in self.near.clone() {
                    self.damage(i, damage, slot as u8, Family::Fire);
                    self.add_burn(i, 3, immune_boss == Some(i));
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
                        def: EMBER_WARDEN,
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

    /// Ember Warden: ember circles to dodge; at half health it becomes immune to
    /// Burn for 15 s and calls three waves of adds.
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
                let circle = |pos| Telegraph {
                    pos,
                    radius: 90.0,
                    warning: 1.3,
                    elapsed: 0.0,
                    owner: EnemyKind::Boss,
                    damage,
                    magic: true,
                };
                self.telegraphs.push(circle(target));
                for _ in 0..2 {
                    let offset = V2::from_angle(self.rng.range(0.0, TAU)) * self.rng.range(120.0, 220.0);
                    self.telegraphs.push(circle(clamp_to_map(target + offset, 90.0)));
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
        let mut hurt = Vec::new();
        for t in &mut self.telegraphs {
            t.elapsed += DT;
            if t.elapsed >= t.warning {
                if player.dist(t.pos) < t.radius + PLAYER_RADIUS && self.player.invulnerable <= 0.0 {
                    hurt.push((t.damage, t.magic));
                }
                self.events.push(Event::Detonate { pos: t.pos, radius: t.radius, owner: t.owner });
            }
        }
        self.telegraphs.retain(|t| t.elapsed < t.warning);
        for (damage, magic) in hurt {
            self.hurt_player(damage, magic);
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

/// A simple autopilot: dodge telegraphs and orbs, keep some distance from what
/// is close, otherwise walk to the nearest enemy. Used for benchmarks and demos.
pub fn autopilot(world: &World) -> Input {
    let p = world.player.pos;
    for t in &world.telegraphs {
        let d = p - t.pos;
        if d.len() < t.radius + PLAYER_RADIUS + 10.0 {
            let away = if d.len_sq() < 1.0 { V2::new(1.0, 0.0) } else { d.norm() };
            return Input { move_dir: away, dash: t.warning - t.elapsed < 0.3 };
        }
    }
    for s in &world.enemy_shots {
        let to_player = p - s.pos;
        if to_player.len() < 110.0 && to_player.dot(s.vel) > 0.0 {
            // Step sideways out of the orb's path.
            let side = V2::new(-s.vel.y, s.vel.x).norm();
            let dir = if side.dot(to_player) >= 0.0 { side } else { -side };
            return Input { move_dir: dir, dash: false };
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
    let low = world.player.hp < world.max_hp() * 0.35;
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
    use crate::content::{presets, Slot};

    fn run(build: Build, seed: u64, max_seconds: f32) -> World {
        let mut world = World::new(build, seed, 0);
        while world.time < max_seconds && !matches!(world.phase, Phase::Victory | Phase::Defeat) {
            let input = autopilot(&world);
            world.step(&input);
            world.events.clear();
        }
        world
    }

    fn preset(name: &str) -> Build {
        presets().into_iter().find(|(n, _)| *n == name).unwrap().1
    }

    fn single(name: &str) -> Build {
        Build::new(vec![Slot { spell: SpellId::named(name), support: None }], vec![])
    }

    /// A world with one awake enemy of `kind` next to the player.
    fn duel(build: Build, kind: EnemyKind) -> (World, usize) {
        let mut world = World::new(build, 1, 0);
        world.enemies.clear();
        let pos = world.player.pos + V2::new(80.0, 0.0);
        let index = world.spawn(kind, pos, true);
        world.rebuild_grid();
        (world, index)
    }

    #[test]
    fn same_seed_same_map() {
        let a = World::new(preset("Starter"), 9, 0);
        let b = World::new(preset("Starter"), 9, 0);
        assert_eq!(a.enemies.len(), b.enemies.len());
        assert!(a.enemies.iter().zip(&b.enemies).all(|(x, y)| x.pos == y.pos && x.kind == y.kind));
    }

    #[test]
    fn maps_have_every_pack_enemy_shape() {
        let world = World::new(preset("Starter"), 5, 0);
        for kind in [EnemyKind::Grunt, EnemyKind::Runner, EnemyKind::Caster, EnemyKind::Brute] {
            assert!(world.enemies.iter().any(|e| e.kind == kind), "{kind:?} missing");
        }
    }

    #[test]
    fn strong_builds_clear_the_map_and_kill_the_boss() {
        for name in ["Plague", "Inferno", "Steel"] {
            let world = run(preset(name), 3, 600.0);
            assert_eq!(world.phase, Phase::Victory, "{name}: time {} progress {}", world.time, world.progress);
        }
    }

    #[test]
    fn every_spell_can_be_cast_at_an_enemy() {
        for spell in SpellId::all().filter(|s| s.casts()) {
            let build = Build::new(vec![Slot { spell, support: None }], vec![]);
            let (mut world, _) = duel(build, EnemyKind::Brute);
            assert!(world.cast(0), "{} did not cast", spell.name());
        }
    }

    #[test]
    fn resistances_reduce_the_matching_damage() {
        // Hexagons resist physical, pentagons resist magic, squares resist nothing.
        let hit = |spell: &str, kind| {
            let (mut world, i) = duel(single(spell), kind);
            let before = world.enemies[i].hp;
            let ctx = CastCtx { slot: 0, mult: 1.0, growth: 0.0 };
            world.spell_hit(ctx, i);
            before - world.enemies[i].hp
        };
        let physical_vs_brute = hit("True Strike", EnemyKind::Brute);
        let physical_vs_caster = hit("True Strike", EnemyKind::Caster);
        let magic_vs_caster = hit("Arcane Bolt", EnemyKind::Caster);
        let magic_vs_brute = hit("Arcane Bolt", EnemyKind::Brute);
        assert!(physical_vs_brute < physical_vs_caster);
        assert!(magic_vs_caster < magic_vs_brute);
    }

    #[test]
    fn shatter_consumes_chill_for_more_damage() {
        let (mut world, i) = duel(single("Shatter"), EnemyKind::Brute);
        let ctx = CastCtx { slot: 0, mult: 1.0, growth: 0.0 };
        let hp = world.enemies[i].hp;
        world.spell_hit(ctx, i);
        let plain = hp - world.enemies[i].hp;
        world.enemies[i].chill = 2.0;
        let hp = world.enemies[i].hp;
        world.spell_hit(ctx, i);
        let chilled = hp - world.enemies[i].hp;
        assert!(chilled > plain * 1.5);
        assert_eq!(world.enemies[i].chill, 0.0);
    }

    #[test]
    fn buffs_refresh_instead_of_stacking() {
        let (mut world, _) = duel(single("Mana Lens"), EnemyKind::Grunt);
        world.cast(0);
        world.cast(0);
        assert_eq!(world.stat(Stat::Ap), BASE_AP + 20.0);
    }

    #[test]
    fn charge_advances_the_right_neighbor() {
        let build = Build::new(
            vec![
                Slot { spell: SpellId::named("Quickstep"), support: None },
                Slot { spell: SpellId::named("Supernova"), support: None },
            ],
            vec![],
        );
        let (mut world, _) = duel(build, EnemyKind::Grunt);
        world.slots[1].cooldown = 2.0;
        assert!(world.cast(0));
        assert!((world.slots[1].cooldown - (2.0 - COOLDOWN_SCALE)).abs() < 1e-4);
    }

    #[test]
    fn echo_casts_with_its_left_neighbor() {
        let build = Build::new(
            vec![
                Slot { spell: SpellId::named("Arcane Bolt"), support: None },
                Slot { spell: SpellId::named("Thunderclap"), support: Some(SupportKind::Echo) },
            ],
            vec![],
        );
        let (mut world, _) = duel(build, EnemyKind::Brute);
        world.slots[1].cooldown = 99.0;
        assert!(world.cast(0));
        let novas = world.events.iter().filter(|e| matches!(e, Event::Nova { .. })).count();
        assert_eq!(novas, 1);
    }

    #[test]
    fn momentum_caps_and_decays() {
        let mut world = World::new(preset("Starter"), 1, 0);
        world.momentum = MOMENTUM_MAX;
        world.momentum_timer = 0.0;
        for _ in 0..((MOMENTUM_DECAY / DT) as usize + 2) {
            world.update_momentum();
        }
        assert_eq!(world.momentum, 0);
    }

    #[test]
    fn artifacts_change_the_player() {
        let build = Build::new(vec![], vec![ArtifactKind::Bloodstone, ArtifactKind::QuickBoots, ArtifactKind::SagesRing]);
        let world = World::new(build, 1, 0);
        assert_eq!(world.max_hp(), BASE_HP + 100.0);
        assert!(world.player.dash_cooldown_max < DASH_COOLDOWN);
        assert_eq!(world.stat(Stat::Ap), BASE_AP + 15.0);
    }

    #[test]
    fn barrier_absorbs_after_resistances() {
        let mut world = World::new(Build::empty(), 1, 0);
        world.player.barrier = 1000.0;
        world.hurt_player(120.0, false);
        let expected = mitigate(120.0, BASE_ARMOR);
        assert!((1000.0 - world.player.barrier - expected).abs() < 1e-3);
        assert_eq!(world.player.hp, world.max_hp());
    }

    #[test]
    fn brute_slam_is_telegraphed_then_hits() {
        let (mut world, brute) = duel(Build::empty(), EnemyKind::Brute);
        world.enemies[brute].pos = world.player.pos + V2::new(60.0, 0.0);
        world.enemies[brute].attack_timer = 0.0;
        world.enemy_attacks();
        assert_eq!(world.telegraphs.len(), 1);
        let hp = world.player.hp;
        for _ in 0..((SLAM_WINDUP / DT) as usize + 2) {
            world.update_telegraphs();
        }
        assert!(world.player.hp < hp);
    }

    #[test]
    fn caster_orbs_can_be_dodged() {
        let (mut world, caster) = duel(Build::empty(), EnemyKind::Caster);
        world.enemies[caster].pos = world.player.pos + V2::new(300.0, 0.0);
        world.enemies[caster].attack_timer = 0.0;
        world.enemy_attacks();
        assert_eq!(world.enemy_shots.len(), 1);
        // Step out of the line of fire; the orb flies past.
        world.player.pos += V2::new(0.0, 80.0);
        let hp = world.player.hp;
        for _ in 0..200 {
            world.update_enemy_shots();
        }
        assert_eq!(world.player.hp, hp);
    }
}
