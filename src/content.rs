//! Spells, supports, artifacts and the loadout.
//!
//! Spells are adapted from God Tower's catalogue. Each spell is data: a
//! delivery (how it reaches enemies in a map) plus effects on each enemy hit
//! and effects on yourself. Descriptions are generated from the same data, so
//! the text always matches the numbers.
use crate::spells::SPELLS;

/// God Tower cooldowns are tuned for a duel; an ARPG clears packs, so spells
/// cast 2.5× more often (charges scale the same way).
pub const COOLDOWN_SCALE: f32 = 0.4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Family {
    Arcane,
    Fire,
    Frost,
    Lightning,
    Poison,
    Blood,
    Shadow,
    Light,
    Nature,
    Steel,
    Earth,
    Time,
    Spirits,
    Alchemy,
    Astral,
}

impl Family {
    pub fn name(self) -> &'static str {
        match self {
            Family::Arcane => "Arcane",
            Family::Fire => "Fire",
            Family::Frost => "Frost",
            Family::Lightning => "Lightning",
            Family::Poison => "Poison",
            Family::Blood => "Blood",
            Family::Shadow => "Shadow",
            Family::Light => "Light",
            Family::Nature => "Nature",
            Family::Steel => "Steel",
            Family::Earth => "Earth",
            Family::Time => "Time",
            Family::Spirits => "Spirits",
            Family::Alchemy => "Alchemy",
            Family::Astral => "Astral",
        }
    }
}

/// Who a targeted delivery aims at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Targeting {
    Nearest,
    Densest,
    LowestHealth,
    HighestHealth,
}

impl Targeting {
    fn phrase(self) -> &'static str {
        match self {
            Targeting::Nearest => "the nearest enemy",
            Targeting::Densest => "the densest pack",
            Targeting::LowestHealth => "the enemy with the lowest Health %",
            Targeting::HighestHealth => "the enemy with the most Health",
        }
    }
}

/// How a spell reaches enemies in the map. This is the spatial layer God
/// Tower's spells did not need.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Delivery {
    Projectile { count: u8, pierce: u8, target: Targeting },
    /// Hits the nearest enemy, then jumps; each jump deals `falloff` times the last.
    Chain { jumps: u8, falloff: f32 },
    /// Everything around you.
    Nova { radius: f32 },
    /// A cone toward the nearest enemy.
    Cone { range: f32 },
    /// An instant area at a target.
    Blast { radius: f32, target: Targeting },
    /// Repeated strikes on a spot chosen when cast; the first lands after `interval`.
    Zone { radius: f32, ticks: u8, interval: f32, target: Targeting },
    /// Three orbs circling you, hitting what they touch.
    Orbit,
    /// No target: effects on yourself or your spell bar.
    SelfCast,
}

impl Delivery {
    pub fn name(self) -> &'static str {
        match self {
            Delivery::Projectile { .. } => "Projectile",
            Delivery::Chain { .. } => "Chain",
            Delivery::Nova { .. } => "Nova",
            Delivery::Cone { .. } => "Cone",
            Delivery::Blast { .. } => "Blast",
            Delivery::Zone { .. } => "Zone",
            Delivery::Orbit => "Orbit",
            Delivery::SelfCast => "Self",
        }
    }
}

/// A value computed from your stats when the spell resolves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scale {
    pub flat: f32,
    pub ad: f32,
    pub ap: f32,
    pub max_hp: f32,
    pub armor: f32,
    pub mr: f32,
    /// Fraction of your current Barrier.
    pub barrier: f32,
}

impl Scale {
    pub const ZERO: Scale = Scale { flat: 0.0, ad: 0.0, ap: 0.0, max_hp: 0.0, armor: 0.0, mr: 0.0, barrier: 0.0 };

    pub fn text(self) -> String {
        let pct = |x: f32| format!("{:.0}%", x * 100.0);
        let mut parts = Vec::new();
        if self.flat != 0.0 {
            parts.push(format!("{:.0}", self.flat));
        }
        if self.ad != 0.0 {
            parts.push(format!("{} AD", pct(self.ad)));
        }
        if self.ap != 0.0 {
            parts.push(format!("{} AP", pct(self.ap)));
        }
        if self.max_hp != 0.0 {
            parts.push(format!("{} of your max Health", pct(self.max_hp)));
        }
        if self.armor != 0.0 {
            parts.push(format!("{} of your PR", pct(self.armor)));
        }
        if self.mr != 0.0 {
            parts.push(format!("{} of your MR", pct(self.mr)));
        }
        if self.barrier != 0.0 {
            parts.push(format!("{} of your Barrier", pct(self.barrier)));
        }
        parts.join(" + ")
    }
}

pub const fn ap(x: f32) -> Scale {
    Scale { ap: x, ..Scale::ZERO }
}

pub const fn ad(x: f32) -> Scale {
    Scale { ad: x, ..Scale::ZERO }
}

pub const fn flat(x: f32) -> Scale {
    Scale { flat: x, ..Scale::ZERO }
}

pub const fn max_hp(x: f32) -> Scale {
    Scale { max_hp: x, ..Scale::ZERO }
}

/// Statuses applied together (thorns).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Status {
    pub poison: u16,
    pub burn: u16,
    pub chill: f32,
}

impl Status {
    pub const NONE: Status = Status { poison: 0, burn: 0, chill: 0.0 };
    pub const CHILL: Status = Status { poison: 0, burn: 0, chill: 1.0 };
}

/// What happens to each enemy a spell hits, in order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hit {
    Damage { scale: Scale, magic: bool, pen: f32 },
    Burn(u16),
    Poison(u16),
    Chill(f32),
    Freeze(f32),
    /// Damage per second (from your stats when applied) for `secs`.
    Bleed { scale: Scale, secs: f32 },
    ShredArmor(f32),
    ShredMr(f32),
    /// The enemy deals 20% less damage for `secs`.
    Weaken(f32),
    /// The enemy takes 60% more Poison damage for `secs`.
    PoisonAmp(f32),
    /// If Chilled: consume it and deal extra magic damage.
    ConsumeChill(Scale),
    /// Consume the target's Burn: magic damage plus `per_stack` per stack.
    ConsumeBurn { scale: Scale, per_stack: f32 },
    ConsumePoison { scale: Scale, per_stack: f32 },
    /// Copy the target's Burn to enemies around it (or add 3 Burn if it has none).
    SpreadBurn { radius: f32 },
    /// Extra Burn on enemies that are already Burning.
    BurnIfBurning(u16),
    /// Effects that depend on the target, read before this hit.
    IfTarget(TargetCond, &'static [Hit], &'static [Hit]),
    Mark(MarkKind, f32),
    /// Magic damage: `per` for each unit stored since this spell's last cast.
    Reserve { kind: ReserveKind, per: Scale, at_least_one: bool },
    /// Damage: `per` times a count.
    PerCount { count: Count, per: Scale, magic: bool },
    /// Magic damage from your Barrier, optionally consuming it.
    FromBarrier { ratio: f32, consume: bool },
    /// Heal for this fraction of the damage this cast deals.
    Lifesteal(f32),
    /// Heal this fraction of your max Health if the hit kills.
    HealOnKill(f32),
    /// Damage multiplier against rares and bosses.
    EliteBonus(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TargetCond {
    Chilled,
    Burning,
    Poisoned,
    PoisonAtLeast(u16),
    Bleeding,
    PoisonedAndBurning,
    BelowHealth(f32),
    AboveHealth(f32),
    /// Its PR is lower than its MR.
    LowerPrThanMr,
    Marked,
}

impl TargetCond {
    fn phrase(self) -> String {
        match self {
            TargetCond::Chilled => "it is Chilled".into(),
            TargetCond::Burning => "it is Burning".into(),
            TargetCond::Poisoned => "it is Poisoned".into(),
            TargetCond::PoisonAtLeast(n) => format!("it has at least {n} Poison"),
            TargetCond::Bleeding => "it is Bleeding".into(),
            TargetCond::PoisonedAndBurning => "it has Poison and Burn".into(),
            TargetCond::BelowHealth(p) => format!("it is below {:.0}% Health", p * 100.0),
            TargetCond::AboveHealth(p) => format!("it is above {:.0}% Health", p * 100.0),
            TargetCond::LowerPrThanMr => "its PR is lower than its MR".into(),
            TargetCond::Marked => "it has one of your marks".into(),
        }
    }

    fn singular(self) -> String {
        match self {
            TargetCond::Chilled => "Chilled enemy".into(),
            TargetCond::Burning => "Burning enemy".into(),
            TargetCond::Poisoned => "Poisoned enemy".into(),
            TargetCond::Bleeding => "Bleeding enemy".into(),
            TargetCond::BelowHealth(p) => format!("enemy below {:.0}% Health", p * 100.0),
            other => format!("enemy where {}", other.phrase()),
        }
    }
}

/// Marks last a few seconds on an enemy and change how your spells treat it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkKind {
    /// Your other spells' hits deal 20% AP more magic damage to it.
    Dusk,
    /// Its Burn ticks deal 15 more damage.
    Brand,
    /// While Chilled, its PR and MR are reduced by 18.
    Brittle,
    /// Your physical hits deal 25% AD more physical damage to it.
    Duel,
    /// Your hits deal 20% of your MR more magic damage to it.
    Vow,
    /// Your hits Chill it for 1 s.
    Gravity,
}

impl MarkKind {
    pub const ALL: [MarkKind; 6] = [MarkKind::Dusk, MarkKind::Brand, MarkKind::Brittle, MarkKind::Duel, MarkKind::Vow, MarkKind::Gravity];

    pub fn index(self) -> usize {
        MarkKind::ALL.iter().position(|&m| m == self).unwrap()
    }

    fn effect(self) -> &'static str {
        match self {
            MarkKind::Dusk => "your other spells' hits deal 20% AP more magic damage to it",
            MarkKind::Brand => "its Burn ticks deal 15 more damage",
            MarkKind::Brittle => "while Chilled, its PR and MR are reduced by 18",
            MarkKind::Duel => "your physical hits deal 25% AD more physical damage to it",
            MarkKind::Vow => "your hits deal 20% of your MR more magic damage to it",
            MarkKind::Gravity => "your hits Chill it for 1 s",
        }
    }
}

/// Amounts that pile up between two casts of the same spell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReserveKind {
    OtherCasts,
    SmallCasts,
    BurnConsumed,
    BarrierAbsorbed,
    BarrierGained,
    HealthPaid,
    Healing,
    PoisonDamage,
}

impl ReserveKind {
    pub const COUNT: usize = 8;

    pub fn index(self) -> usize {
        self as usize
    }

    fn phrase(self) -> &'static str {
        match self {
            ReserveKind::OtherCasts => "for each cast of your other spells",
            ReserveKind::SmallCasts => "for each cast of your other size-1 spells",
            ReserveKind::BurnConsumed => "for each time another spell consumed Burn",
            ReserveKind::BarrierAbsorbed => "of the damage your Barrier absorbed",
            ReserveKind::BarrierGained => "of the Barrier you gained",
            ReserveKind::HealthPaid => "of the Health your spells paid",
            ReserveKind::Healing => "of the healing you received",
            ReserveKind::PoisonDamage => "of the Poison damage you dealt",
        }
    }

    /// Counted events (as opposed to amounts, which scale by a fraction).
    fn counted(self) -> bool {
        matches!(self, ReserveKind::OtherCasts | ReserveKind::SmallCasts | ReserveKind::BurnConsumed)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Count {
    Artifacts,
    /// Distinct families among your other spells on the bar.
    DistinctFamilies,
    EmptySlots,
    MarksOnTarget,
    /// Poison plus Burn stacks on the target.
    TargetStacks,
    /// Enemies near you matching a condition.
    Nearby(TargetCond),
    /// Your other spells with more than 1 s of cooldown left.
    CoolingSpells,
}

impl Count {
    fn phrase(self) -> String {
        match self {
            Count::Artifacts => "per equipped artifact".into(),
            Count::DistinctFamilies => "per distinct family among your other spells".into(),
            Count::EmptySlots => "per empty slot on your bar".into(),
            Count::MarksOnTarget => "per mark of yours on it".into(),
            Count::TargetStacks => "per Poison and Burn stack on it".into(),
            Count::Nearby(cond) => format!("for each {} near you", cond.singular()),
            Count::CoolingSpells => "per other spell still on cooldown".into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stat {
    Ad,
    Ap,
    Armor,
    Mr,
    /// Percent more damage for your spells.
    DamagePct,
    /// Percent less damage taken (capped at 60%).
    DamageReduction,
    /// Percent more healing received.
    HealingPct,
    /// Heals also grant this percent of the healing as Barrier.
    HealToBarrier,
    MaxHp,
    /// Percent more movement speed.
    MoveSpeedPct,
    /// Percent more cooldown rate.
    CooldownPct,
}

impl Stat {
    fn name(self) -> &'static str {
        match self {
            Stat::Ad => "AD",
            Stat::Ap => "AP",
            Stat::Armor => "PR",
            Stat::Mr => "MR",
            Stat::DamagePct => "% spell damage",
            Stat::DamageReduction => "% damage reduction",
            Stat::HealingPct => "% healing received",
            Stat::HealToBarrier => "% of healing as Barrier",
            Stat::MaxHp => "max Health",
            Stat::MoveSpeedPct => "% movement speed",
            Stat::CooldownPct => "% cooldown rate",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Left,
    Right,
    Adjacent,
    Bar,
}

impl Scope {
    fn phrase(self) -> &'static str {
        match self {
            Scope::Left => "the spell to the left",
            Scope::Right => "the spell to the right",
            Scope::Adjacent => "both adjacent spells",
            Scope::Bar => "your whole spell bar",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SelfCond {
    BelowHealth(f32),
    AboveHealth(f32),
    FullHealth,
    NoBarrier,
    /// The spell to the left cast within this many seconds.
    LeftCastWithin(f32),
    /// Both neighbors cast since this spell's last cast.
    BothNeighborsCast,
    LeftSizeAtLeast(u8),
    NeighborsAllSize1,
    NeighborsDifferentSize,
    /// Your AD and AP are within 10 of each other.
    AdNearAp,
    AdAboveAp,
    KilledWithin(f32),
}

impl SelfCond {
    fn phrase(self) -> String {
        match self {
            SelfCond::BelowHealth(p) => format!("you are below {:.0}% Health", p * 100.0),
            SelfCond::AboveHealth(p) => format!("you are above {:.0}% Health", p * 100.0),
            SelfCond::FullHealth => "you are at full Health".to_string(),
            SelfCond::NoBarrier => "you have no Barrier".to_string(),
            SelfCond::LeftCastWithin(s) => format!("the spell to the left cast in the last {s:.0} s"),
            SelfCond::BothNeighborsCast => "both adjacent spells cast since this spell's last cast".to_string(),
            SelfCond::LeftSizeAtLeast(n) => format!("the spell to the left has size {n} or more"),
            SelfCond::NeighborsAllSize1 => "both adjacent spells are size 1".to_string(),
            SelfCond::NeighborsDifferentSize => "the adjacent spells have different sizes".to_string(),
            SelfCond::AdNearAp => "your AD and AP are within 10 of each other".to_string(),
            SelfCond::AdAboveAp => "your AD is higher than your AP".to_string(),
            SelfCond::KilledWithin(s) => format!("you killed an enemy in the last {s:.0} s"),
        }
    }
}

/// What a cast does to you or your spell bar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Own {
    Barrier(Scale),
    Heal(Scale),
    Regen { per_sec: Scale, secs: f32 },
    Buff { stat: Stat, amount: f32, secs: f32 },
    /// Cooldown rate +`rate` (1.0 = twice as fast) for `secs`.
    Haste { scope: Scope, rate: f32, secs: f32 },
    /// Advance cooldowns by `secs` (God Tower seconds, scaled like cooldowns).
    Charge { scope: Scope, secs: f32 },
    /// Pay this fraction of your current Health (never lethal).
    PayHealth(f32),
    /// Enemies touching you take damage and statuses every 0.5 s.
    Thorns { scale: Scale, magic: bool, status: Status, secs: f32 },
    If(SelfCond, &'static [Own]),
    IfElse(SelfCond, &'static [Own], &'static [Own]),
    /// Alternates between two effects on each cast.
    Alternate(&'static [Own], &'static [Own]),
    /// The targeted spells deal `pct` more damage for `secs`.
    Empower { scope: Scope, pct: f32, secs: f32 },
    /// The targeted spells add `stacks` to each Poison or Burn they apply.
    Catalyze { scope: Scope, stacks: u16, secs: f32 },
    /// Consume a fraction of your Barrier: heal for part of it, gain AP from part of it.
    ConsumeBarrier { fraction: f32, heal: f32, ap: f32, secs: f32 },
    /// Pay Health, then gain Barrier equal to `ratio` times the Health paid.
    PayForBarrier { pct: f32, ratio: f32 },
    /// A permanent stat gain for the rest of the map.
    Grow { stat: Stat, amount: f32 },
    BarrierPer { count: Count, per: f32 },
    HealPer { count: Count, per: f32 },
    /// Heal for `ratio` of a reserve stored since this spell's last cast.
    HealFromReserve { kind: ReserveKind, ratio: f32 },
    /// Heal for a fraction of the Health you lost in the last `secs`.
    HealFromRecent { fraction: f32, secs: f32 },
    BarrierFromRecent { fraction: f32, secs: f32 },
    /// Charge your other spell that is closest to ready.
    ChargeNearestReady(f32),
    /// Charge by a fraction of the target spell's remaining cooldown.
    ChargeFraction { scope: Scope, fraction: f32 },
}

pub struct SpellDef {
    pub name: &'static str,
    pub family: Family,
    pub size: u8,
    /// God Tower cooldown in seconds (before COOLDOWN_SCALE).
    pub cooldown: f32,
    pub delivery: Delivery,
    pub hits: &'static [Hit],
    pub own: &'static [Own],
    /// Damage multiplier when a condition on you holds.
    pub boost: Option<(SelfCond, f32)>,
    /// Flat damage added per previous cast this map.
    pub growth: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpellId(pub u16);

impl SpellId {
    pub fn count() -> usize {
        SPELLS.len()
    }

    pub fn all() -> impl Iterator<Item = SpellId> {
        (0..SPELLS.len() as u16).map(SpellId)
    }

    /// Looks a spell up by name. Panics on unknown names: only for authored data.
    pub fn named(name: &str) -> Self {
        SpellId::all().find(|s| s.name() == name).unwrap_or_else(|| panic!("unknown spell {name}"))
    }

    pub fn def(self) -> &'static SpellDef {
        &SPELLS[self.0 as usize]
    }

    pub fn name(self) -> &'static str {
        self.def().name
    }

    pub fn family(self) -> Family {
        self.def().family
    }

    pub fn size(self) -> u8 {
        self.def().size
    }

    pub fn delivery(self) -> Delivery {
        self.def().delivery
    }

    /// Seconds between casts in the map.
    pub fn cooldown(self) -> f32 {
        self.def().cooldown * COOLDOWN_SCALE
    }

    /// Whether the spell casts on a cooldown (Orbit is always on).
    pub fn casts(self) -> bool {
        self.delivery() != Delivery::Orbit
    }

    pub fn hits_enemies(self) -> bool {
        self.delivery() != Delivery::SelfCast
    }

    pub fn targeted(self) -> bool {
        matches!(
            self.delivery(),
            Delivery::Projectile { .. } | Delivery::Chain { .. } | Delivery::Cone { .. } | Delivery::Blast { .. } | Delivery::Zone { .. }
        )
    }

    /// Which neighbors on the bar this spell's effects reach (charge, Haste,
    /// Empower, Catalyze): (left, right). Whole-bar effects count as neither.
    pub fn affects_sides(self) -> (bool, bool) {
        fn walk(list: &[Own], sides: &mut (bool, bool)) {
            for own in list {
                match *own {
                    Own::Charge { scope, .. }
                    | Own::Haste { scope, .. }
                    | Own::Empower { scope, .. }
                    | Own::Catalyze { scope, .. }
                    | Own::ChargeFraction { scope, .. } => match scope {
                        Scope::Left => sides.0 = true,
                        Scope::Right => sides.1 = true,
                        Scope::Adjacent => *sides = (true, true),
                        Scope::Bar => {}
                    },
                    Own::If(_, inner) => walk(inner, sides),
                    Own::IfElse(_, a, b) | Own::Alternate(a, b) => {
                        walk(a, sides);
                        walk(b, sides);
                    }
                    _ => {}
                }
            }
        }
        let mut sides = (false, false);
        walk(self.def().own, &mut sides);
        sides
    }

    pub fn has_area(self) -> bool {
        matches!(
            self.delivery(),
            Delivery::Nova { .. } | Delivery::Cone { .. } | Delivery::Blast { .. } | Delivery::Zone { .. } | Delivery::Orbit
        )
    }

    pub fn description(self) -> String {
        let d = self.def();
        let mut out: Vec<String> = Vec::new();
        match d.delivery {
            Delivery::Projectile { count, pierce, target } => {
                let what = if count > 1 { format!("Fires {count} projectiles") } else { "Fires a projectile".to_string() };
                let pierce = if pierce > 0 { format!(", piercing {pierce} enemies") } else { String::new() };
                out.push(format!("{what} at {}{pierce}.", target.phrase()));
            }
            Delivery::Chain { jumps, falloff } => {
                let loss = if falloff < 1.0 { format!(" ({:.0}% less per jump)", (1.0 - falloff) * 100.0) } else { String::new() };
                out.push(format!("Strikes the nearest enemy, then jumps to {jumps} more{loss}."));
            }
            Delivery::Nova { .. } => out.push("Burst around you.".to_string()),
            Delivery::Cone { .. } => out.push("Cone toward the nearest enemy.".to_string()),
            Delivery::Blast { target, .. } => out.push(format!("Strikes an area at {}.", target.phrase())),
            Delivery::Zone { ticks, interval, target, .. } => {
                if ticks == 1 {
                    out.push(format!("After {interval:.1} s, strikes the spot where {} stood.", target.phrase()));
                } else {
                    out.push(format!("Strikes the spot at {} {ticks} times, {interval:.2} s apart.", target.phrase()));
                }
            }
            Delivery::Orbit => out.push("Three orbs circle you and hit what they touch.".to_string()),
            Delivery::SelfCast => {}
        }
        for hit in d.hits {
            out.push(hit_text(*hit));
        }
        if let Some((cond, mult)) = d.boost {
            out.push(format!("×{mult:.1} damage if {}.", cond.phrase()));
        }
        if d.growth > 0.0 {
            out.push(format!("+{:.0} damage for each previous cast this map.", d.growth));
        }
        for own in d.own {
            out.push(own_text(*own));
        }
        out.join(" ")
    }
}

fn hit_text(hit: Hit) -> String {
    match hit {
        Hit::Damage { scale, magic, pen } => {
            let kind = if magic { "magic" } else { "physical" };
            let pen = if pen > 0.0 {
                format!(", ignoring {:.0}% of its {}", pen * 100.0, if magic { "MR" } else { "PR" })
            } else {
                String::new()
            };
            format!("Deal {} {kind} damage{pen}.", scale.text())
        }
        Hit::Burn(n) => format!("Apply {n} Burn."),
        Hit::Poison(n) => format!("Apply {n} Poison."),
        Hit::Chill(s) => format!("Chill for {s:.1} s."),
        Hit::Freeze(s) => format!("Freeze for {s:.2} s."),
        Hit::Bleed { scale, secs } => format!("Apply Bleed: {} physical damage per second for {secs:.0} s.", scale.text()),
        Hit::ShredArmor(x) => format!("Reduce its PR by {x:.0} for 4 s."),
        Hit::ShredMr(x) => format!("Reduce its MR by {x:.0} for 4 s."),
        Hit::Weaken(s) => format!("It deals 20% less damage for {s:.0} s."),
        Hit::PoisonAmp(s) => format!("It takes 60% more Poison damage for {s:.0} s."),
        Hit::ConsumeChill(scale) => format!("If Chilled, consume it for {} more magic damage.", scale.text()),
        Hit::ConsumeBurn { scale, per_stack } => {
            format!("Consume its Burn: deal {} + {per_stack:.0} per stack magic damage.", scale.text())
        }
        Hit::ConsumePoison { scale, per_stack } => {
            format!("Consume its Poison: deal {} + {per_stack:.0} per stack magic damage.", scale.text())
        }
        Hit::SpreadBurn { .. } => "Copy its Burn to nearby enemies; if it has none, apply 3 Burn.".to_string(),
        Hit::BurnIfBurning(n) => format!("Burning enemies get {n} more Burn."),
        Hit::IfTarget(cond, yes, no) => {
            let yes: Vec<String> = yes.iter().map(|h| lower_first(&hit_text(*h))).collect();
            let no: Vec<String> = no.iter().map(|h| lower_first(&hit_text(*h))).collect();
            if no.is_empty() {
                format!("If {}: {}", cond.phrase(), yes.join(" "))
            } else {
                format!("If {}: {} Otherwise: {}", cond.phrase(), yes.join(" "), no.join(" "))
            }
        }
        Hit::Mark(kind, secs) => format!("Mark it for {secs:.0} s: {}.", kind.effect()),
        Hit::Reserve { kind, per, at_least_one } => {
            let min = if at_least_one { " (at least once)" } else { "" };
            if kind.counted() {
                format!("Deal {} magic damage {}{min} since this spell's last cast.", per.text(), kind.phrase())
            } else {
                format!("Deal magic damage equal to {:.0}% {} since this spell's last cast.", per.flat * 100.0, kind.phrase())
            }
        }
        Hit::PerCount { count, per, magic } => {
            format!("Deal {} {} damage {}.", per.text(), if magic { "magic" } else { "physical" }, count.phrase())
        }
        Hit::FromBarrier { ratio, consume } => {
            if consume {
                format!("Consume all your Barrier and deal {ratio:.1} magic damage per point.")
            } else {
                format!("Deal magic damage equal to {:.0}% of your Barrier.", ratio * 100.0)
            }
        }
        Hit::Lifesteal(p) => format!("Heal for {:.0}% of the damage dealt.", p * 100.0),
        Hit::HealOnKill(p) => format!("If it kills, heal {:.0}% of your max Health.", p * 100.0),
        Hit::EliteBonus(m) => format!("×{m:.1} against rares and bosses."),
    }
}

fn own_text(own: Own) -> String {
    match own {
        Own::Barrier(s) => format!("Gain {} Barrier.", s.text()),
        Own::Heal(s) => format!("Heal for {}.", s.text()),
        Own::Regen { per_sec, secs } => format!("Heal for {} per second for {secs:.0} s.", per_sec.text()),
        Own::Buff { stat: Stat::DamagePct, amount, secs } => {
            format!("Your spells deal {amount:.0}% more damage for {secs:.0} s.")
        }
        Own::Buff { stat, amount, secs } if amount < 0.0 => format!("Lose {:.0} {} for {secs:.0} s.", -amount, stat.name()),
        Own::Buff { stat, amount, secs } => format!("Gain +{amount:.0} {} for {secs:.0} s.", stat.name()),
        Own::Haste { scope, rate, secs } => {
            format!("{} casts {:.0}% faster for {secs:.0} s.", capitalize(scope.phrase()), rate * 100.0)
        }
        Own::Charge { scope, secs } => format!("Charge {} by {:.2} s.", scope.phrase(), secs * COOLDOWN_SCALE),
        Own::PayHealth(p) => format!("Pay {:.0}% of your current Health.", p * 100.0),
        Own::Thorns { scale, magic, status, secs } => {
            let mut parts = Vec::new();
            if scale != Scale::ZERO {
                parts.push(format!("take {} {} damage", scale.text(), if magic { "magic" } else { "physical" }));
            }
            if status.chill > 0.0 {
                parts.push("are Chilled".to_string());
            }
            if status.burn > 0 {
                parts.push(format!("get {} Burn", status.burn));
            }
            if status.poison > 0 {
                parts.push(format!("get {} Poison", status.poison));
            }
            format!("For {secs:.0} s, enemies touching you {} every 0.5 s.", parts.join(" and "))
        }
        Own::If(cond, list) => {
            let inner: Vec<String> = list.iter().map(|o| lower_first(&own_text(*o))).collect();
            format!("If {}: {}", cond.phrase(), inner.join(" "))
        }
        Own::IfElse(cond, yes, no) => {
            let yes: Vec<String> = yes.iter().map(|o| lower_first(&own_text(*o))).collect();
            let no: Vec<String> = no.iter().map(|o| lower_first(&own_text(*o))).collect();
            format!("If {}: {} Otherwise: {}", cond.phrase(), yes.join(" "), no.join(" "))
        }
        Own::Alternate(a, b) => {
            let a: Vec<String> = a.iter().map(|o| lower_first(&own_text(*o))).collect();
            let b: Vec<String> = b.iter().map(|o| lower_first(&own_text(*o))).collect();
            format!("Alternates each cast between: {} And: {}", a.join(" "), b.join(" "))
        }
        Own::Empower { scope, pct, secs } => format!("{} deals {pct:.0}% more damage for {secs:.0} s.", capitalize(scope.phrase())),
        Own::Catalyze { scope, stacks, secs } => {
            format!("For {secs:.0} s, {} adds {stacks} to each Poison or Burn it applies.", scope.phrase())
        }
        Own::ConsumeBarrier { fraction, heal, ap, secs } => {
            let mut out = format!("Consume {:.0}% of your Barrier", fraction * 100.0);
            if heal > 0.0 {
                out += &format!(" and heal for {:.0}% of it", heal * 100.0);
            }
            if ap > 0.0 {
                out += &format!(" and gain AP equal to {:.0}% of it for {secs:.0} s", ap * 100.0);
            }
            out + "."
        }
        Own::PayForBarrier { pct, ratio } => {
            format!("Pay {:.0}% of your current Health; gain {ratio:.1}× that as Barrier.", pct * 100.0)
        }
        Own::Grow { stat, amount } => format!("Gain +{amount:.0} {} for the rest of the map.", stat_name(stat)),
        Own::BarrierPer { count, per } => format!("Gain {per:.0} Barrier {}.", count.phrase()),
        Own::HealPer { count, per } => format!("Heal for {per:.0} {}.", count.phrase()),
        Own::HealFromReserve { kind, ratio } => {
            format!("Heal for {:.0}% {} since this spell's last cast.", ratio * 100.0, kind.phrase())
        }
        Own::HealFromRecent { fraction, secs } => {
            format!("Heal for {:.0}% of the Health you lost in the last {secs:.0} s.", fraction * 100.0)
        }
        Own::BarrierFromRecent { fraction, secs } => {
            format!("Gain Barrier equal to {:.0}% of the Health you lost in the last {secs:.0} s.", fraction * 100.0)
        }
        Own::ChargeNearestReady(secs) => {
            format!("Charge your other spell closest to ready by {:.2} s.", secs * COOLDOWN_SCALE)
        }
        Own::ChargeFraction { scope, fraction } => {
            format!("Charge {} by {:.0}% of its remaining cooldown.", scope.phrase(), fraction * 100.0)
        }
    }
}

fn stat_name(stat: Stat) -> &'static str {
    stat.name()
}

fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_lowercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SupportKind {
    Multishot,
    Pierce,
    Amplify,
    Contagion,
    Combustion,
    Frenzy,
    Hunter,
    Echo,
}

impl SupportKind {
    pub const ALL: [SupportKind; 8] = [
        SupportKind::Multishot,
        SupportKind::Pierce,
        SupportKind::Amplify,
        SupportKind::Contagion,
        SupportKind::Combustion,
        SupportKind::Frenzy,
        SupportKind::Hunter,
        SupportKind::Echo,
    ];

    pub fn name(self) -> &'static str {
        match self {
            SupportKind::Multishot => "Multishot",
            SupportKind::Pierce => "Pierce",
            SupportKind::Amplify => "Amplify",
            SupportKind::Contagion => "Contagion",
            SupportKind::Combustion => "Combustion",
            SupportKind::Frenzy => "Frenzy",
            SupportKind::Hunter => "Hunter",
            SupportKind::Echo => "Echo",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            SupportKind::Multishot => "+2 projectiles (or +2 jumps), each aimed at a different enemy. −20% damage. Projectile and Chain spells.",
            SupportKind::Pierce => "Projectiles pass through 3 more enemies. Projectile spells.",
            SupportKind::Amplify => "+50% area. Nova, Cone, Blast, Zone and Orbit spells.",
            SupportKind::Contagion => "This spell applies 3 Poison. A Poisoned enemy that dies spreads its Poison around it. Spells that hit enemies.",
            SupportKind::Combustion => "This spell applies 3 Burn. An enemy that dies while Burning explodes. Spells that hit enemies.",
            SupportKind::Frenzy => "Kills by this spell grant momentum: +3% cooldown rate per stack (max 20). It all drops after 2 s without a kill.",
            SupportKind::Hunter => "Targeting: bosses first, then rares. Spells that pick a target.",
            SupportKind::Echo => "Also casts whenever the spell to its left casts. Spells that cast.",
        }
    }

    /// Whether this support does something on this spell.
    pub fn fits(self, spell: SpellId) -> bool {
        match self {
            SupportKind::Multishot => matches!(spell.delivery(), Delivery::Projectile { .. } | Delivery::Chain { .. }),
            SupportKind::Pierce => matches!(spell.delivery(), Delivery::Projectile { .. }),
            SupportKind::Amplify => spell.has_area(),
            SupportKind::Hunter => spell.targeted(),
            SupportKind::Echo => spell.casts(),
            SupportKind::Contagion | SupportKind::Combustion | SupportKind::Frenzy => spell.hits_enemies(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ArtifactKind {
    SagesRing,
    Warband,
    IronPlate,
    SpiritVeil,
    EmberHeart,
    VenomGland,
    FrostSigil,
    StormBattery,
    HuntersLens,
    Bloodstone,
    SoulSiphon,
    MomentumEngine,
    WideLens,
    QuickBoots,
    BarrierCharm,
    ChainLink,
}

/// How much wider a card is than a size-1 card: ×1, ×1.25, ×1.5. Bigger
/// cards read as bigger without a size 3 taking three times the room; the
/// size itself shows as bars on top of the card.
pub fn size_scale(size: u8) -> f32 {
    1.0 + 0.25 * size.saturating_sub(1) as f32
}

/// Artifacts go from tier 1 to 5; two copies of a tier fuse into the next.
/// Tier 5 is legendary.
pub const MAX_ARTIFACT_TIER: u8 = 5;

/// How much each tier multiplies an artifact's numbers: ×1, ×1.5, ×2, ×2.5, ×3.
pub fn tier_mult(tier: u8) -> f32 {
    1.0 + 0.5 * (tier.clamp(1, MAX_ARTIFACT_TIER) - 1) as f32
}

/// Whole-number bonuses (extra jumps, momentum per kill): 1, 1, 2, 2, 3.
pub fn tier_count(tier: u8) -> u8 {
    (tier.clamp(1, MAX_ARTIFACT_TIER) + 1) / 2
}

pub fn tier_label(tier: u8) -> String {
    if tier >= MAX_ARTIFACT_TIER {
        "Legendary".into()
    } else {
        format!("Tier {tier}")
    }
}

impl ArtifactKind {
    pub const COUNT: usize = 16;
    pub const ALL: [ArtifactKind; 16] = [
        ArtifactKind::SagesRing,
        ArtifactKind::Warband,
        ArtifactKind::IronPlate,
        ArtifactKind::SpiritVeil,
        ArtifactKind::EmberHeart,
        ArtifactKind::VenomGland,
        ArtifactKind::FrostSigil,
        ArtifactKind::StormBattery,
        ArtifactKind::HuntersLens,
        ArtifactKind::Bloodstone,
        ArtifactKind::SoulSiphon,
        ArtifactKind::MomentumEngine,
        ArtifactKind::WideLens,
        ArtifactKind::QuickBoots,
        ArtifactKind::BarrierCharm,
        ArtifactKind::ChainLink,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ArtifactKind::SagesRing => "Sage's Ring",
            ArtifactKind::Warband => "Warband",
            ArtifactKind::IronPlate => "Iron Plate",
            ArtifactKind::SpiritVeil => "Spirit Veil",
            ArtifactKind::EmberHeart => "Ember Heart",
            ArtifactKind::VenomGland => "Venom Gland",
            ArtifactKind::FrostSigil => "Frost Sigil",
            ArtifactKind::StormBattery => "Storm Battery",
            ArtifactKind::HuntersLens => "Hunter's Lens",
            ArtifactKind::Bloodstone => "Bloodstone",
            ArtifactKind::SoulSiphon => "Soul Siphon",
            ArtifactKind::MomentumEngine => "Momentum Engine",
            ArtifactKind::WideLens => "Wide Lens",
            ArtifactKind::QuickBoots => "Quick Boots",
            ArtifactKind::BarrierCharm => "Barrier Charm",
            ArtifactKind::ChainLink => "Chain Link",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    /// The number the effect scales at `tier`: AP, percent bonus, Health...
    pub fn value(self, tier: u8) -> f32 {
        let base = match self {
            ArtifactKind::SagesRing | ArtifactKind::Warband | ArtifactKind::StormBattery | ArtifactKind::QuickBoots => 15.0,
            ArtifactKind::IronPlate | ArtifactKind::SpiritVeil | ArtifactKind::WideLens | ArtifactKind::BarrierCharm => 25.0,
            ArtifactKind::EmberHeart | ArtifactKind::HuntersLens => 30.0,
            ArtifactKind::VenomGland | ArtifactKind::FrostSigil => 20.0,
            ArtifactKind::Bloodstone => 100.0,
            ArtifactKind::SoulSiphon => 2.0,
            ArtifactKind::MomentumEngine | ArtifactKind::ChainLink => return tier_count(tier) as f32,
        };
        base * tier_mult(tier)
    }

    pub fn description(self, tier: u8) -> String {
        let v = self.value(tier);
        match self {
            ArtifactKind::SagesRing => format!("+{v:.0} AP."),
            ArtifactKind::Warband => format!("+{v:.0} AD."),
            ArtifactKind::IronPlate => format!("+{v:.0} PR (physical resistance)."),
            ArtifactKind::SpiritVeil => format!("+{v:.0} MR (magic resistance)."),
            ArtifactKind::EmberHeart => format!("Burn deals {v:.0}% more damage."),
            ArtifactKind::VenomGland => {
                format!("Poison lasts {:.0} s longer and deals {v:.0}% more damage.", 2.0 * tier_mult(tier))
            }
            ArtifactKind::FrostSigil => format!("Chilled enemies take {v:.0}% more damage."),
            ArtifactKind::StormBattery => format!("+{v:.0}% cooldown rate."),
            ArtifactKind::HuntersLens => format!("+{v:.0}% damage to rares and bosses."),
            ArtifactKind::Bloodstone => format!("+{v:.0} maximum Health."),
            ArtifactKind::SoulSiphon => format!("Heal {v:.0} Health per kill."),
            ArtifactKind::MomentumEngine => format!(
                "Every kill grants {v:.0} momentum, from any spell (max 20, drops after 2 s without a kill)."
            ),
            ArtifactKind::WideLens => format!("+{v:.0}% area for all spells."),
            ArtifactKind::QuickBoots => format!("+{v:.0}% movement speed. Dash cooldown −0.5 s."),
            ArtifactKind::BarrierCharm => format!("Gain {v:.0} Barrier when you dash. Barrier absorbs damage."),
            ArtifactKind::ChainLink => {
                let s = if v > 1.0 { "s" } else { "" };
                format!("Chains jump {v:.0} more time{s}. Projectiles pierce {v:.0} more enem{}.", if v > 1.0 { "ies" } else { "y" })
            }
        }
    }

    /// Supports a legendary (tier 5) copy can hold: the ones that share its
    /// theme. A support there works on every spell of the bar it fits.
    pub fn socket_fits(self, support: SupportKind) -> bool {
        use ArtifactKind::*;
        use SupportKind::*;
        matches!(
            (self, support),
            (ChainLink, Multishot | Pierce)
                | (WideLens, Amplify)
                | (EmberHeart, Combustion)
                | (VenomGland, Contagion)
                | (HuntersLens, Hunter)
                | (StormBattery, Frenzy)
        )
    }

    /// Whether the legendary tier comes with a support socket.
    pub fn has_legendary_socket(self) -> bool {
        SupportKind::ALL.into_iter().any(|s| self.socket_fits(s))
    }

    /// Artifact capacity it takes (God Tower sizes).
    pub fn size(self) -> u8 {
        match self {
            ArtifactKind::StormBattery | ArtifactKind::MomentumEngine | ArtifactKind::WideLens => 2,
            _ => 1,
        }
    }
}

/// Something that drops in a map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loot {
    Support(SupportKind),
    /// An artifact and the tier it dropped at.
    Artifact(ArtifactKind, u8),
}

impl Loot {
    pub fn name(self) -> String {
        match self {
            Loot::Support(s) => s.name().into(),
            Loot::Artifact(a, 1) => a.name().into(),
            Loot::Artifact(a, tier) if tier >= MAX_ARTIFACT_TIER => format!("Legendary {}", a.name()),
            Loot::Artifact(a, tier) => format!("{} T{tier}", a.name()),
        }
    }

    pub fn kind_name(self) -> &'static str {
        match self {
            Loot::Support(_) => "Support",
            Loot::Artifact(..) => "Artifact",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    pub spell: SpellId,
    pub support: Option<SupportKind>,
}

/// The loadout: spell bar (ordered left to right; adjacency reads the neighbors
/// in this order) and artifact bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Build {
    pub slots: Vec<Slot>,
    pub capacity: u8,
    pub artifacts: Vec<ArtifactKind>,
    pub artifact_capacity: u8,
    /// Tier of each artifact (by `ArtifactKind::index`), from the account.
    pub tiers: [u8; ArtifactKind::COUNT],
    /// Supports socketed into legendary artifacts.
    pub artifact_sockets: Vec<(ArtifactKind, SupportKind)>,
}

impl Build {
    pub const SPELL_CAPACITY: u8 = 8;
    pub const ARTIFACT_CAPACITY: u8 = 5;

    pub fn new(slots: Vec<Slot>, artifacts: Vec<ArtifactKind>) -> Self {
        Build {
            slots,
            capacity: Self::SPELL_CAPACITY,
            artifacts,
            artifact_capacity: Self::ARTIFACT_CAPACITY,
            tiers: [1; ArtifactKind::COUNT],
            artifact_sockets: Vec::new(),
        }
    }

    pub fn empty() -> Self {
        Self::new(Vec::new(), Vec::new())
    }

    pub fn used(&self) -> u8 {
        self.slots.iter().map(|s| s.spell.size()).sum()
    }

    pub fn artifacts_used(&self) -> u8 {
        self.artifacts.iter().map(|a| a.size()).sum()
    }

    pub fn contains(&self, spell: SpellId) -> bool {
        self.slots.iter().any(|s| s.spell == spell)
    }

    pub fn has_artifact(&self, artifact: ArtifactKind) -> bool {
        self.artifacts.contains(&artifact)
    }

    pub fn support_used(&self, support: SupportKind) -> bool {
        self.slots.iter().any(|s| s.support == Some(support)) || self.artifact_sockets.iter().any(|&(_, s)| s == support)
    }

    pub fn tier(&self, artifact: ArtifactKind) -> u8 {
        self.tiers[artifact.index()]
    }

    pub fn is_legendary(&self, artifact: ArtifactKind) -> bool {
        self.tier(artifact) >= MAX_ARTIFACT_TIER
    }

    pub fn artifact_socket(&self, artifact: ArtifactKind) -> Option<SupportKind> {
        self.artifact_sockets.iter().find(|&&(a, _)| a == artifact).map(|&(_, s)| s)
    }

    /// Sockets a support into an equipped legendary artifact that accepts it.
    pub fn socket_artifact(&mut self, artifact: ArtifactKind, support: SupportKind) -> bool {
        if !self.has_artifact(artifact) || !self.is_legendary(artifact) || !artifact.socket_fits(support) {
            return false;
        }
        self.unsocket(support);
        self.artifact_sockets.retain(|&(a, _)| a != artifact);
        self.artifact_sockets.push((artifact, support));
        true
    }

    /// Supports held by equipped legendary artifacts: each works on every
    /// spell of the bar it fits.
    pub fn granted_supports(&self) -> impl Iterator<Item = SupportKind> + '_ {
        self.artifact_sockets
            .iter()
            .filter(|&&(a, s)| self.has_artifact(a) && self.is_legendary(a) && a.socket_fits(s))
            .map(|&(_, s)| s)
    }

    /// Inserts a spell at `index`, or moves it there if it is already on the bar
    /// (keeping its support).
    pub fn place_spell(&mut self, spell: SpellId, index: usize) -> bool {
        if let Some(from) = self.slots.iter().position(|s| s.spell == spell) {
            let slot = self.slots.remove(from);
            let to = if index > from { index - 1 } else { index };
            self.slots.insert(to.min(self.slots.len()), slot);
            return true;
        }
        if self.used() + spell.size() > self.capacity {
            return false;
        }
        self.slots.insert(index.min(self.slots.len()), Slot { spell, support: None });
        true
    }

    pub fn remove_spell(&mut self, spell: SpellId) {
        self.slots.retain(|s| s.spell != spell);
    }

    /// Sockets a support; each support may appear once per build, so it moves
    /// from wherever it was.
    pub fn socket(&mut self, index: usize, support: SupportKind) -> bool {
        if index >= self.slots.len() || !support.fits(self.slots[index].spell) {
            return false;
        }
        self.unsocket(support);
        self.slots[index].support = Some(support);
        true
    }

    pub fn unsocket(&mut self, support: SupportKind) {
        for slot in &mut self.slots {
            if slot.support == Some(support) {
                slot.support = None;
            }
        }
        self.artifact_sockets.retain(|&(_, s)| s != support);
    }

    /// Inserts an artifact at `index`, or moves it there if already equipped.
    pub fn place_artifact(&mut self, artifact: ArtifactKind, index: usize) -> bool {
        if let Some(from) = self.artifacts.iter().position(|&a| a == artifact) {
            self.artifacts.remove(from);
            let to = if index > from { index - 1 } else { index };
            self.artifacts.insert(to.min(self.artifacts.len()), artifact);
            return true;
        }
        if self.artifacts_used() + artifact.size() > self.artifact_capacity {
            return false;
        }
        self.artifacts.insert(index.min(self.artifacts.len()), artifact);
        true
    }

    pub fn remove_artifact(&mut self, artifact: ArtifactKind) {
        self.artifacts.retain(|&a| a != artifact);
        self.artifact_sockets.retain(|&(a, _)| a != artifact);
    }

    pub fn is_valid(&self) -> bool {
        let unique_spells = self.slots.iter().enumerate().all(|(i, a)| self.slots[i + 1..].iter().all(|b| b.spell != a.spell));
        let unique_supports = self
            .slots
            .iter()
            .enumerate()
            .all(|(i, a)| a.support.is_none() || self.slots[i + 1..].iter().all(|b| b.support != a.support));
        let fitting = self.slots.iter().all(|s| s.support.is_none_or(|sup| sup.fits(s.spell)));
        let unique_artifacts = self.artifacts.iter().enumerate().all(|(i, a)| !self.artifacts[i + 1..].contains(a));
        let sockets_ok = self.artifact_sockets.iter().enumerate().all(|(i, &(a, s))| {
            a.socket_fits(s)
                && self.has_artifact(a)
                && self.is_legendary(a)
                && !self.slots.iter().any(|slot| slot.support == Some(s))
                && self.artifact_sockets[i + 1..].iter().all(|&(b, t)| b != a && t != s)
        });
        self.used() <= self.capacity
            && self.artifacts_used() <= self.artifact_capacity
            && unique_spells
            && unique_supports
            && fitting
            && unique_artifacts
            && sockets_ok
    }
}

/// Ready-made loadouts to compare how different machines clear. They ignore
/// the skill tree (sandbox).
pub fn presets() -> Vec<(&'static str, Build)> {
    use ArtifactKind::*;
    use SupportKind::*;
    let slot = |name, support| Slot { spell: SpellId::named(name), support };
    vec![
        (
            "Starter",
            Build::new(
                vec![slot("Arcane Bolt", None), slot("Chain Lightning", None), slot("Gravity Well", None), slot("Mending Ray", None)],
                vec![],
            ),
        ),
        (
            "Plague",
            Build::new(
                vec![
                    slot("Venom Fang", Some(Contagion)),
                    slot("Toxic Bloom", Some(Amplify)),
                    slot("Septic Burst", Some(Hunter)),
                    slot("Arcane Bolt", Some(Echo)),
                    slot("Living Barrier", None),
                ],
                vec![VenomGland, SagesRing, WideLens, Bloodstone],
            ),
        ),
        (
            "Inferno",
            Build::new(
                vec![
                    slot("Cinder Dart", Some(Multishot)),
                    slot("Ember Engine", Some(Amplify)),
                    slot("Flashover", Some(Hunter)),
                    slot("Wild Spark", Some(Combustion)),
                    slot("Mana Lens", None),
                ],
                vec![EmberHeart, SagesRing, StormBattery, SoulSiphon],
            ),
        ),
        (
            "Steel",
            Build::new(
                vec![
                    slot("Crimson Pact", None),
                    slot("True Strike", Some(Pierce)),
                    slot("Twin Daggers", Some(Multishot)),
                    slot("Cleave", Some(Amplify)),
                    slot("Ricochet Blade", Some(Frenzy)),
                    slot("Granite Skin", None),
                ],
                vec![Warband, MomentumEngine, IronPlate, QuickBoots],
            ),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_spell_has_a_distinct_name_and_description() {
        let names: std::collections::HashSet<_> = SpellId::all().map(|s| s.name()).collect();
        assert_eq!(names.len(), SpellId::count());
        for spell in SpellId::all() {
            assert!(!spell.description().is_empty(), "{} has no description", spell.name());
        }
    }

    #[test]
    fn every_family_has_spells() {
        use Family::*;
        for family in [Arcane, Fire, Frost, Lightning, Poison, Blood, Shadow, Light, Nature, Steel, Earth, Time, Spirits, Alchemy, Astral] {
            assert!(SpellId::all().any(|s| s.family() == family), "{family:?} has no spell");
        }
    }

    #[test]
    fn presets_are_valid() {
        for (name, build) in presets() {
            assert!(build.is_valid(), "preset {name} is invalid");
        }
    }

    #[test]
    fn support_moves_instead_of_duplicating() {
        let mut build = Build::empty();
        build.place_spell(SpellId::named("Arcane Bolt"), 0);
        build.place_spell(SpellId::named("Chain Lightning"), 1);
        assert!(build.socket(0, SupportKind::Hunter));
        assert!(build.socket(1, SupportKind::Hunter));
        assert_eq!(build.slots[0].support, None);
        assert_eq!(build.slots[1].support, Some(SupportKind::Hunter));
        assert!(build.is_valid());
    }

    #[test]
    fn placing_a_bar_spell_moves_it() {
        let mut build = Build::empty();
        let (a, b, c) = (SpellId::named("Arcane Bolt"), SpellId::named("Frost Needle"), SpellId::named("Volt Javelin"));
        for (i, s) in [a, b, c].into_iter().enumerate() {
            build.place_spell(s, i);
        }
        build.socket(0, SupportKind::Pierce);
        assert!(build.place_spell(a, 3));
        let order: Vec<_> = build.slots.iter().map(|s| s.spell).collect();
        assert_eq!(order, vec![b, c, a]);
        assert_eq!(build.slots[2].support, Some(SupportKind::Pierce));
    }

    #[test]
    fn capacities_count_sizes() {
        let mut build = Build::empty();
        assert!(build.place_spell(SpellId::named("Supernova"), 0));
        assert!(build.place_spell(SpellId::named("Thunderclap"), 1));
        assert!(build.place_spell(SpellId::named("Cleave"), 2));
        assert!(!build.place_spell(SpellId::named("Arcane Bolt"), 3));
        assert!(build.place_artifact(ArtifactKind::WideLens, 0));
        assert!(build.place_artifact(ArtifactKind::StormBattery, 1));
        assert!(build.place_artifact(ArtifactKind::Bloodstone, 2));
        assert!(!build.place_artifact(ArtifactKind::QuickBoots, 3));
    }

    #[test]
    fn legendary_artifacts_hold_a_themed_support() {
        let mut build = Build::empty();
        build.place_spell(SpellId::named("Arcane Bolt"), 0);
        build.place_artifact(ArtifactKind::ChainLink, 0);
        assert!(!build.socket_artifact(ArtifactKind::ChainLink, SupportKind::Pierce), "tier 1 has no socket");
        build.tiers[ArtifactKind::ChainLink.index()] = MAX_ARTIFACT_TIER;
        assert!(!build.socket_artifact(ArtifactKind::ChainLink, SupportKind::Amplify), "off-theme");
        build.socket(0, SupportKind::Pierce);
        assert!(build.socket_artifact(ArtifactKind::ChainLink, SupportKind::Pierce));
        assert_eq!(build.slots[0].support, None, "a support is used once per build");
        assert_eq!(build.granted_supports().collect::<Vec<_>>(), vec![SupportKind::Pierce]);
        assert!(build.is_valid());
        build.remove_artifact(ArtifactKind::ChainLink);
        assert!(build.artifact_sockets.is_empty());
    }

    #[test]
    fn tiers_scale_artifacts() {
        assert_eq!(ArtifactKind::SagesRing.value(1), 15.0);
        assert_eq!(ArtifactKind::SagesRing.value(5), 45.0);
        assert_eq!(ArtifactKind::ChainLink.value(3), 2.0);
        assert!(ArtifactKind::ChainLink.description(5).contains("3 more times"));
        assert!(ArtifactKind::ALL.iter().enumerate().all(|(i, a)| a.index() == i));
    }

    #[test]
    fn side_effects_point_the_right_way() {
        assert_eq!(SpellId::named("Quickstep").affects_sides(), (false, true));
        assert_eq!(SpellId::named("Quickstep Sigil").affects_sides(), (true, false));
        assert_eq!(SpellId::named("Pendulum").affects_sides(), (true, true));
        assert_eq!(SpellId::named("Acceleration").affects_sides(), (false, false));
        assert_eq!(SpellId::named("Arcane Bolt").affects_sides(), (false, false));
    }

    #[test]
    fn supports_follow_delivery() {
        assert!(!SupportKind::Pierce.fits(SpellId::named("Thunderclap")));
        assert!(!SupportKind::Hunter.fits(SpellId::named("Orbiting Blade")));
        assert!(!SupportKind::Contagion.fits(SpellId::named("Mana Lens")));
        assert!(SupportKind::Amplify.fits(SpellId::named("Hailstorm")));
    }
}
