//! The map bosses. Each one combines bullet patterns to dodge with a
//! mechanic that tests the build (design §9): the Ember Warden turns immune to
//! Burn, the Iron Juggernaut raises a shield you must burst, the Hollow Oracle
//! only takes damage while you stand in its light and seals your best spell.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossKind {
    EmberWarden,
    IronJuggernaut,
    HollowOracle,
}

/// Each boss has its own stats and a short briefing shown before the map.
#[derive(Clone, Copy, Debug)]
pub struct BossDef {
    pub kind: BossKind,
    pub name: &'static str,
    pub hp: f32,
    pub armor: f32,
    pub mr: f32,
    pub speed: f32,
    pub contact_dps: f32,
    /// One line: what kind of fight it is.
    pub summary: &'static str,
    /// What it does, one mechanic per entry.
    pub mechanics: &'static [&'static str],
}

/// The first boss: a fight of 30 to 60 s for a fresh character at tier 1.
pub const EMBER_WARDEN: BossDef = BossDef {
    kind: BossKind::EmberWarden,
    name: "Ember Warden",
    hp: 1000.0,
    armor: 20.0,
    mr: 25.0,
    speed: 62.0,
    contact_dps: 40.0,
    summary: "Fire spirals and ember circles. Burn builds need a plan B.",
    mechanics: &[
        "Spins spirals of embers: weave through the arms.",
        "Drops ember circles on you: step out before they burst.",
        "At half health: immune to Burn for 15 s, three waves of adds, and rings of fire.",
    ],
};

pub const IRON_JUGGERNAUT: BossDef = BossDef {
    kind: BossKind::IronJuggernaut,
    name: "Iron Juggernaut",
    hp: 900.0,
    armor: 45.0,
    mr: 15.0,
    speed: 75.0,
    contact_dps: 45.0,
    summary: "Heavy armor, charges and shields. Bring magic damage and burst.",
    mechanics: &[
        "Charges along a marked line, then bursts into shrapnel. Dodge sideways: it is staggered after, taking 30% more damage.",
        "Fires fans of bolts at you.",
        "At 66% and 33% health: raises a Bulwark shield and spins gears. Break the shield within 8 s, or it repairs 15% of its health.",
    ],
};

pub const HOLLOW_ORACLE: BossDef = BossDef {
    kind: BossKind::HollowOracle,
    name: "Hollow Oracle",
    hp: 620.0,
    armor: 15.0,
    mr: 45.0,
    speed: 45.0,
    contact_dps: 30.0,
    summary: "Warded by light, rings of orbs, seals your spells. Bring physical damage and spare spells.",
    mechanics: &[
        "Takes damage only while you stand in its circle of light, which moves.",
        "Fires rings of orbs and curving flowers of void.",
        "Every 10 s, seals your most damaging spell for 5 s: a build that leans on one spell stalls.",
    ],
};

/// Bosses come in this order; beating one brings the next.
pub const BOSSES: [BossDef; 3] = [EMBER_WARDEN, IRON_JUGGERNAUT, HOLLOW_ORACLE];

impl BossKind {
    pub fn def(self) -> BossDef {
        match self {
            BossKind::EmberWarden => EMBER_WARDEN,
            BossKind::IronJuggernaut => IRON_JUGGERNAUT,
            BossKind::HollowOracle => HOLLOW_ORACLE,
        }
    }
}

/// How an enemy bullet looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShotStyle {
    Orb,
    Ember,
    Iron,
    Void,
}

/// Radius of the Oracle's circle of light.
pub const WARD_RADIUS: f32 = 150.0;
const CHARGE_WINDUP: f32 = 1.0;
const CHARGE_TRAVEL: f32 = 0.3;
const CHARGE_LENGTH: f32 = 620.0;
const STAGGER_BONUS: f32 = 1.3;

#[derive(Clone, Debug)]
pub struct BossState {
    pub index: usize,
    pub def: BossDef,
    /// Ember circles, charges or ring volleys, depending on the boss.
    circle_timer: f32,
    /// Streams of bullets (spirals, gears, flowers): time to the next one,
    /// time left in the current one, and time to its next bullet.
    pattern_timer: f32,
    pattern_left: f32,
    emit_timer: f32,
    /// Aimed fans, flare rings.
    volley_timer: f32,
    volleys: u32,
    /// The rotating emitter's angle.
    angle: f32,
    /// Seconds of Burn immunity left (Ember Warden, phase 2).
    pub burn_immunity: f32,
    pub phase_two: bool,
    add_waves_left: u8,
    add_timer: f32,
    /// Juggernaut charge: start, direction, seconds since the wind-up began.
    pub charge: Option<(V2, V2, f32)>,
    /// Seconds left staggered: it stands still and takes more damage.
    pub stagger: f32,
    pub shield: f32,
    pub shield_max: f32,
    pub shield_timer: f32,
    shields_used: u8,
    /// Oracle: the circle of light, and when it moves next.
    pub ward: Option<V2>,
    ward_timer: f32,
    seal_timer: f32,
}

impl BossState {
    pub fn new(index: usize, def: BossDef) -> Self {
        BossState {
            index,
            def,
            circle_timer: 2.0,
            pattern_timer: 1.5,
            pattern_left: 0.0,
            emit_timer: 0.0,
            volley_timer: 1.0,
            volleys: 0,
            angle: 0.0,
            burn_immunity: 0.0,
            phase_two: false,
            add_waves_left: 0,
            add_timer: 0.0,
            charge: None,
            stagger: 0.0,
            shield: 0.0,
            shield_max: 0.0,
            shield_timer: 0.0,
            shields_used: 0,
            ward: None,
            ward_timer: 0.0,
            seal_timer: 5.0,
        }
    }

    /// While true, the Oracle ignores your damage.
    pub fn warded_against(&self, player: V2) -> bool {
        self.ward.is_some_and(|w| w.dist(player) > WARD_RADIUS)
    }
}

struct Bullet {
    speed: f32,
    radius: f32,
    damage: f32,
    magic: bool,
    spin: f32,
    style: ShotStyle,
}

impl World {
    /// Spawns the map's boss and its state.
    pub(super) fn spawn_boss(&mut self, pos: V2) {
        let def = self.boss_def;
        let index = self.spawn(EnemyKind::Boss, pos, true);
        let hp = def.hp * self.hp_mult();
        let e = &mut self.enemies[index];
        e.hp = hp;
        e.max_hp = hp;
        e.armor = def.armor;
        e.mr = def.mr;
        let mut state = BossState::new(index, def);
        if def.kind == BossKind::HollowOracle {
            state.ward = Some(self.player.pos + (pos - self.player.pos) * 0.5);
            state.ward_timer = 8.0;
        }
        self.boss = Some(state);
    }

    fn fire(&mut self, from: V2, angle: f32, b: &Bullet) {
        let dir = V2::from_angle(angle);
        self.enemy_shots.push(EnemyShot {
            pos: from + dir * 30.0,
            vel: dir * b.speed,
            radius: b.radius,
            damage: b.damage * self.damage_mult(),
            life: 7.0,
            spin: b.spin,
            magic: b.magic,
            style: b.style,
        });
    }

    /// A ring of `count` bullets; `gap` bullets starting at `gap_at` are left out.
    fn ring(&mut self, from: V2, count: usize, offset: f32, gap: (usize, usize), b: &Bullet) {
        for k in 0..count {
            let skip = (k + count - gap.1) % count < gap.0;
            if !skip {
                self.fire(from, offset + k as f32 * TAU / count as f32, b);
            }
        }
    }

    /// A fan of `count` bullets centered on `angle`, `spread` radians wide.
    fn fan(&mut self, from: V2, angle: f32, count: usize, spread: f32, b: &Bullet) {
        for k in 0..count {
            let t = if count > 1 { k as f32 / (count - 1) as f32 - 0.5 } else { 0.0 };
            self.fire(from, angle + t * spread, b);
        }
    }

    fn alert(&mut self, pos: V2, text: impl Into<String>) {
        self.events.push(Event::BossAlert { pos, text: text.into() });
    }

    /// What your damage to the boss becomes: nothing outside the Oracle's
    /// light, more while staggered, and a shield soaks it first.
    pub(super) fn boss_filter(&mut self, index: usize, amount: f32) -> f32 {
        let player = self.player.pos;
        let Some(b) = self.boss.as_mut().filter(|b| b.index == index) else {
            return amount;
        };
        if b.warded_against(player) {
            return 0.0;
        }
        let mut amount = amount * if b.stagger > 0.0 { STAGGER_BONUS } else { 1.0 };
        if b.shield > 0.0 {
            let soaked = amount.min(b.shield);
            b.shield -= soaked;
            amount -= soaked;
            if b.shield <= 0.0 {
                b.shield = 0.0;
                b.shield_timer = 0.0;
                b.stagger = 3.0;
                let pos = self.enemies[index].pos;
                self.alert(pos, "Bulwark broken! Staggered.");
            }
        }
        amount
    }

    pub(super) fn update_boss(&mut self) {
        let Some(mut boss) = self.boss.take() else {
            return;
        };
        let e = &self.enemies[boss.index];
        if e.alive {
            boss.stagger = (boss.stagger - DT).max(0.0);
            boss.burn_immunity = (boss.burn_immunity - DT).max(0.0);
            match boss.def.kind {
                BossKind::EmberWarden => self.ember_warden(&mut boss),
                BossKind::IronJuggernaut => self.iron_juggernaut(&mut boss),
                BossKind::HollowOracle => self.hollow_oracle(&mut boss),
            }
            if boss.stagger > 0.0 || boss.charge.is_some() {
                let e = &mut self.enemies[boss.index];
                e.windup = e.windup.max(DT * 2.0);
            }
        }
        self.boss = Some(boss);
    }

    /// Spirals, ember circles; at half health immune to Burn, adds and flare rings.
    fn ember_warden(&mut self, boss: &mut BossState) {
        let e = &self.enemies[boss.index];
        let (pos, low) = (e.pos, e.hp < e.max_hp * 0.5);
        let ember = Bullet { speed: 200.0, radius: 9.0, damage: 11.0, magic: true, spin: 0.0, style: ShotStyle::Ember };
        boss.angle += if boss.phase_two { 1.5 } else { 1.1 } * DT;
        boss.circle_timer -= DT;
        if boss.circle_timer <= 0.0 {
            boss.circle_timer = 4.2;
            let damage = 50.0 * self.damage_mult();
            let target = self.player.pos;
            let circle = |pos| Telegraph {
                pos,
                radius: 90.0,
                warning: 1.3,
                elapsed: 0.0,
                owner: EnemyKind::Boss,
                damage,
                magic: true,
                line: None,
            };
            self.telegraphs.push(circle(target));
            for _ in 0..2 {
                let offset = V2::from_angle(self.rng.range(0.0, TAU)) * self.rng.range(120.0, 220.0);
                self.telegraphs.push(circle(target + offset));
            }
        }
        // Spiral streams.
        boss.pattern_timer -= DT;
        if boss.pattern_timer <= 0.0 {
            boss.pattern_timer = 6.5;
            boss.pattern_left = if boss.phase_two { 3.5 } else { 3.0 };
        }
        if boss.pattern_left > 0.0 {
            boss.pattern_left -= DT;
            boss.emit_timer -= DT;
            if boss.emit_timer <= 0.0 {
                boss.emit_timer = 0.12;
                let arms = if boss.phase_two { 5 } else { 3 };
                for k in 0..arms {
                    self.fire(pos, boss.angle + k as f32 * TAU / arms as f32, &ember);
                }
            }
        }
        if low && !boss.phase_two {
            boss.phase_two = true;
            boss.burn_immunity = 15.0;
            boss.add_waves_left = 3;
            boss.add_timer = 0.0;
            self.enemies[boss.index].burn = 0;
            self.alert(pos, "Immune to Burn! Adds incoming!");
        }
        if boss.phase_two {
            // Flare rings with a gap to slip through.
            boss.volley_timer -= DT;
            if boss.volley_timer <= 0.0 {
                boss.volley_timer = 4.0;
                let gap_at = self.rng.range_usize(0, 27);
                let flare = Bullet { speed: 160.0, radius: 10.0, ..ember };
                self.ring(pos, 28, boss.angle, (5, gap_at), &flare);
            }
        }
        if boss.add_waves_left > 0 {
            boss.add_timer -= DT;
            if boss.add_timer <= 0.0 {
                boss.add_timer = 4.0;
                boss.add_waves_left -= 1;
                for k in 0..10 {
                    let at = pos + V2::from_angle(k as f32 * TAU / 10.0) * 140.0;
                    self.spawn(EnemyKind::Add, at, true);
                }
            }
        }
    }

    /// Charges and shrapnel, fans of bolts; shields at 66% and 33% health.
    fn iron_juggernaut(&mut self, boss: &mut BossState) {
        let e = &self.enemies[boss.index];
        let (pos, frac, max_hp) = (e.pos, e.hp / e.max_hp, e.max_hp);
        let player = self.player.pos;
        let bolt = Bullet { speed: 320.0, radius: 8.0, damage: 12.0, magic: false, spin: 0.0, style: ShotStyle::Iron };
        let gear = Bullet { speed: 190.0, radius: 10.0, damage: 12.0, magic: false, spin: 0.0, style: ShotStyle::Iron };
        boss.angle += 1.6 * DT;
        // Bulwark.
        let thresholds = [0.66, 0.33];
        if (boss.shields_used as usize) < thresholds.len() && frac < thresholds[boss.shields_used as usize] && boss.charge.is_none() {
            boss.shields_used += 1;
            boss.shield_max = max_hp * 0.15;
            boss.shield = boss.shield_max;
            boss.shield_timer = 8.0;
            boss.phase_two = boss.shields_used == 2;
            self.alert(pos, "Bulwark! Break the shield in 8 s.");
        }
        if boss.shield > 0.0 {
            boss.shield_timer -= DT;
            // Gears: four streams turning while it hides.
            boss.emit_timer -= DT;
            if boss.emit_timer <= 0.0 {
                boss.emit_timer = 0.15;
                for k in 0..4 {
                    self.fire(pos, boss.angle + k as f32 * TAU / 4.0, &gear);
                }
            }
            if boss.shield_timer <= 0.0 {
                boss.shield = 0.0;
                let e = &mut self.enemies[boss.index];
                e.hp = (e.hp + max_hp * 0.15).min(e.max_hp);
                self.alert(pos, "The Juggernaut repairs itself!");
            }
            return;
        }
        // Charge: wind up along a marked line, rush it, burst into shrapnel.
        if let Some((from, dir, t)) = boss.charge {
            let t = t + DT;
            if t < CHARGE_WINDUP {
                boss.charge = Some((from, dir, t));
            } else if t < CHARGE_WINDUP + CHARGE_TRAVEL {
                let k = (t - CHARGE_WINDUP) / CHARGE_TRAVEL;
                self.enemies[boss.index].pos = from + dir * (CHARGE_LENGTH * k);
                boss.charge = Some((from, dir, t));
            } else {
                let end = from + dir * CHARGE_LENGTH;
                self.enemies[boss.index].pos = end;
                boss.charge = None;
                boss.stagger = 1.8;
                let shrapnel = Bullet { speed: 180.0, radius: 10.0, damage: 13.0, ..bolt };
                let count = if boss.phase_two { 26 } else { 20 };
                let offset = self.rng.range(0.0, TAU);
                self.ring(end, count, offset, (0, 0), &shrapnel);
                self.events.push(Event::Detonate { pos: end, radius: 80.0, owner: EnemyKind::Boss });
            }
            return;
        }
        if boss.stagger > 0.0 {
            return;
        }
        boss.circle_timer -= DT;
        if boss.circle_timer <= 0.0 {
            boss.circle_timer = if boss.phase_two { 4.5 } else { 5.5 };
            let dir = (player - pos).norm();
            let dir = if dir == V2::ZERO { V2::new(1.0, 0.0) } else { dir };
            boss.charge = Some((pos, dir, 0.0));
            self.telegraphs.push(Telegraph {
                pos,
                radius: 55.0,
                warning: CHARGE_WINDUP,
                elapsed: 0.0,
                owner: EnemyKind::Boss,
                damage: 60.0 * self.damage_mult(),
                magic: false,
                line: Some((dir, CHARGE_LENGTH)),
            });
            return;
        }
        boss.volley_timer -= DT;
        if boss.volley_timer <= 0.0 {
            boss.volley_timer = 2.1;
            let aim = (player - pos).angle();
            let count = if boss.phase_two { 9 } else { 7 };
            self.fan(pos, aim, count, 0.9, &bolt);
        }
    }

    /// Circle of light, ring volleys, curving flowers, seals.
    fn hollow_oracle(&mut self, boss: &mut BossState) {
        let e = &self.enemies[boss.index];
        let (pos, low) = (e.pos, e.hp < e.max_hp * 0.5);
        let orb = Bullet { speed: 145.0, radius: 10.0, damage: 12.0, magic: true, spin: 0.0, style: ShotStyle::Void };
        if low && !boss.phase_two {
            boss.phase_two = true;
            self.alert(pos, "The light quickens!");
        }
        boss.angle += 0.9 * DT;
        // The circle of light moves near the Oracle.
        boss.ward_timer -= DT;
        if boss.ward_timer <= 0.0 {
            boss.ward_timer = if boss.phase_two { 6.0 } else { 8.0 };
            let dir = V2::from_angle(self.rng.range(0.0, TAU));
            boss.ward = Some(pos + dir * self.rng.range(170.0, 280.0));
        }
        // Ring volleys, each one offset by half a step so the lanes shift.
        boss.circle_timer -= DT;
        if boss.circle_timer <= 0.0 {
            boss.circle_timer = if boss.phase_two { 1.8 } else { 2.4 };
            boss.volleys += 1;
            let count = if boss.phase_two { 28 } else { 22 };
            let offset = (boss.volleys % 2) as f32 * 0.5 * TAU / count as f32;
            self.ring(pos, count, offset, (0, 0), &orb);
        }
        // Flowers: arms that curve, alternating directions.
        boss.pattern_timer -= DT;
        if boss.pattern_timer <= 0.0 {
            boss.pattern_timer = 6.0;
            boss.pattern_left = 2.5;
        }
        if boss.pattern_left > 0.0 {
            boss.pattern_left -= DT;
            boss.emit_timer -= DT;
            if boss.emit_timer <= 0.0 {
                boss.emit_timer = 0.16;
                for k in 0..6 {
                    let spin = if k % 2 == 0 { 0.9 } else { -0.9 };
                    let petal = Bullet { speed: 175.0, radius: 8.0, spin, ..orb };
                    self.fire(pos, boss.angle + k as f32 * TAU / 6.0, &petal);
                }
            }
        }
        // Seal the spell that has dealt the most damage.
        boss.seal_timer -= DT;
        if boss.seal_timer <= 0.0 {
            boss.seal_timer = 10.0;
            let best = (0..self.slots.len())
                .filter(|&i| self.slots[i].spell.casts() && self.slots[i].sealed <= 0.0)
                .max_by(|&a, &b| self.stats[a].damage.total_cmp(&self.stats[b].damage));
            if let Some(i) = best {
                self.slots[i].sealed = 5.0;
                let name = self.slots[i].spell.name();
                self.alert(pos, format!("{name} is sealed!"));
            }
        }
    }
}
