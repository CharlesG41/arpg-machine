//! The spell catalogue, adapted from God Tower (design/spells-150 and
//! design/spells-250 in the auto-battler repo). Names, families, sizes,
//! cooldowns and ratios come from there; each spell gained a delivery so it
//! works in a map. Duel-only rules (enemy spell bars, enemy Barriers, contracts,
//! Grievous Wounds) were dropped or reworded; see docs/spell-adaptation.md.
use crate::content::Delivery::*;
use crate::content::Family::*;
use crate::content::Scope::*;
use crate::content::Targeting::*;
use crate::content::*;

const fn magic(scale: Scale) -> Hit {
    Hit::Damage { scale, magic: true, pen: 0.0 }
}

const fn physical(scale: Scale) -> Hit {
    Hit::Damage { scale, magic: false, pen: 0.0 }
}

const fn physical_pen(scale: Scale, pen: f32) -> Hit {
    Hit::Damage { scale, magic: false, pen }
}

const fn sc(flat: f32, ad: f32, ap: f32) -> Scale {
    Scale { flat, ad, ap, ..Scale::ZERO }
}

const fn mr(x: f32) -> Scale {
    Scale { mr: x, ..Scale::ZERO }
}

const fn if_target(cond: TargetCond, yes: &'static [Hit]) -> Hit {
    Hit::IfTarget(cond, yes, &[])
}

const fn buff(stat: Stat, amount: f32, secs: f32) -> Own {
    Own::Buff { stat, amount, secs }
}

const fn charge(scope: Scope, secs: f32) -> Own {
    Own::Charge { scope, secs }
}

const fn haste(scope: Scope, rate: f32, secs: f32) -> Own {
    Own::Haste { scope, rate, secs }
}

const fn barrier(scale: Scale) -> Own {
    Own::Barrier(scale)
}

const fn heal(scale: Scale) -> Own {
    Own::Heal(scale)
}

const fn bleed(flat: f32, ad: f32) -> Hit {
    Hit::Bleed { scale: sc(flat, ad, 0.0), secs: 3.0 }
}

const PROJECTILE: Delivery = Projectile { count: 1, pierce: 0, target: Nearest };
const NOVA_M: Delivery = Nova { radius: 190.0 };
const NOVA_L: Delivery = Nova { radius: 240.0 };

const fn spell(name: &'static str, family: Family, size: u8, cooldown: f32, delivery: Delivery) -> SpellDef {
    SpellDef { name, family, size, cooldown, delivery, hits: &[], own: &[], boost: None, growth: 0.0 }
}

pub static SPELLS: &[SpellDef] = &[
    // ---------------------------------------------------------------- Arcane
    SpellDef { hits: &[magic(ap(0.9))], ..spell("Arcane Bolt", Arcane, 1, 2.5, PROJECTILE) },
    SpellDef { hits: &[magic(sc(18.0, 0.0, 1.1))], growth: 3.0, ..spell("Event Horizon", Arcane, 1, 4.0, PROJECTILE) },
    SpellDef {
        hits: &[Hit::Damage { scale: ap(1.5), magic: true, pen: 0.3 }],
        ..spell("Prismatic Lance", Arcane, 2, 3.2, Projectile { count: 1, pierce: 1, target: Nearest })
    },
    SpellDef {
        hits: &[Hit::Reserve { kind: ReserveKind::OtherCasts, per: ap(0.4), at_least_one: true }],
        ..spell("Arcane Reservoir", Arcane, 2, 4.5, PROJECTILE)
    },
    SpellDef { own: &[charge(Right, 0.75), Own::Empower { scope: Right, pct: 25.0, secs: 5.0 }], ..spell("Spellweave", Arcane, 1, 5.0, SelfCast) },
    SpellDef {
        hits: &[magic(ap(0.8))],
        boost: Some((SelfCond::LeftCastWithin(2.0), 2.0)),
        ..spell("Resonant Orb", Arcane, 2, 3.4, PROJECTILE)
    },
    SpellDef { hits: &[magic(ap(1.0))], ..spell("Null Needle", Arcane, 1, 3.4, PROJECTILE) },
    SpellDef { own: &[buff(Stat::Ap, 20.0, 4.0)], ..spell("Mana Lens", Arcane, 1, 5.5, SelfCast) },
    SpellDef {
        hits: &[magic(ap(0.7)), Hit::PerCount { count: Count::DistinctFamilies, per: ap(0.15), magic: true }],
        ..spell("Arcane Census", Arcane, 2, 3.6, PROJECTILE)
    },
    SpellDef {
        hits: &[Hit::Reserve { kind: ReserveKind::SmallCasts, per: ap(0.9), at_least_one: true }],
        ..spell("Convergence", Arcane, 3, 4.5, PROJECTILE)
    },
    SpellDef {
        hits: &[magic(ap(0.65))],
        own: &[Own::If(SelfCond::LeftSizeAtLeast(2), &[charge(Left, 0.6)])],
        ..spell("Runic Relay", Arcane, 1, 3.6, PROJECTILE)
    },
    SpellDef { own: &[Own::ConsumeBarrier { fraction: 0.3, heal: 0.0, ap: 0.1, secs: 4.0 }], ..spell("Mana Crucible", Arcane, 2, 5.0, SelfCast) },
    SpellDef { hits: &[magic(Scale { ap: 1.2, mr: 0.8, ..Scale::ZERO })], ..spell("Runebreaker", Arcane, 2, 4.2, PROJECTILE) },
    SpellDef {
        hits: &[magic(ap(0.95))],
        own: &[Own::If(SelfCond::NoBarrier, &[barrier(sc(20.0, 0.0, 0.2))])],
        ..spell("Empty Vessel", Arcane, 1, 4.0, PROJECTILE)
    },
    SpellDef { own: &[Own::Empower { scope: Adjacent, pct: 40.0, secs: 4.0 }], ..spell("Arcane Conduit", Arcane, 3, 6.5, SelfCast) },
    // ------------------------------------------------------------------ Fire
    SpellDef { hits: &[Hit::Burn(5)], ..spell("Ember Engine", Fire, 2, 5.0, NOVA_M) },
    SpellDef { hits: &[magic(ap(0.45)), Hit::Burn(3)], ..spell("Cinder Dart", Fire, 1, 2.6, PROJECTILE) },
    SpellDef { hits: &[Hit::ConsumeBurn { scale: ap(0.5), per_stack: 10.0 }], ..spell("Flashover", Fire, 2, 4.0, PROJECTILE) },
    SpellDef { hits: &[Hit::SpreadBurn { radius: 220.0 }], ..spell("Wild Spark", Fire, 1, 4.0, PROJECTILE) },
    SpellDef {
        own: &[barrier(max_hp(0.08)), Own::BarrierPer { count: Count::Nearby(TargetCond::Burning), per: 4.0 }],
        ..spell("Furnace Heart", Fire, 2, 4.5, SelfCast)
    },
    SpellDef { hits: &[magic(ap(0.6))], own: &[charge(Left, 0.5)], ..spell("Firebreak", Fire, 1, 4.0, PROJECTILE) },
    SpellDef {
        own: &[heal(max_hp(0.06)), Own::If(SelfCond::BelowHealth(0.3), &[heal(max_hp(0.12))])],
        ..spell("Phoenix Wick", Fire, 2, 6.0, SelfCast)
    },
    SpellDef { hits: &[Hit::Mark(MarkKind::Brand, 4.0), Hit::Burn(2)], ..spell("Searing Brand", Fire, 1, 4.5, PROJECTILE) },
    SpellDef { hits: &[magic(ap(1.0)), Hit::BurnIfBurning(6)], ..spell("Ashfall", Fire, 3, 4.8, Blast { radius: 180.0, target: Densest }) },
    SpellDef {
        hits: &[magic(ap(0.8)), Hit::Reserve { kind: ReserveKind::BurnConsumed, per: ap(1.0), at_least_one: false }],
        ..spell("Backdraft", Fire, 2, 3.8, PROJECTILE)
    },
    SpellDef {
        hits: &[physical(ad(0.65)), Hit::Burn(2), if_target(TargetCond::Bleeding, &[Hit::Burn(2)])],
        ..spell("Branding Iron", Fire, 1, 3.4, PROJECTILE)
    },
    SpellDef {
        own: &[
            barrier(Scale { flat: 40.0, armor: 0.6, ..Scale::ZERO }),
            Own::Thorns { scale: Scale::ZERO, magic: true, status: Status { poison: 0, burn: 2, chill: 0.0 }, secs: 4.0 },
        ],
        ..spell("Coalheart", Fire, 2, 5.5, SelfCast)
    },
    SpellDef { hits: &[if_target(TargetCond::Burning, &[Hit::Burn(2)])], own: &[charge(Right, 0.5)], ..spell("Bellows", Fire, 1, 4.2, PROJECTILE) },
    SpellDef {
        hits: &[physical(ad(1.3)), if_target(TargetCond::Burning, &[physical(ad(0.3))])],
        ..spell("Kiln Hammer", Fire, 2, 4.6, PROJECTILE)
    },
    SpellDef {
        hits: &[Hit::Burn(7)],
        own: &[Own::Regen { per_sec: max_hp(0.02), secs: 3.0 }],
        ..spell("Ashen Procession", Fire, 3, 6.2, NOVA_L)
    },
    // ----------------------------------------------------------------- Frost
    SpellDef { hits: &[magic(ap(0.65)), Hit::Chill(1.0)], ..spell("Frost Needle", Frost, 1, 2.8, PROJECTILE) },
    SpellDef {
        own: &[
            barrier(sc(35.0, 0.0, 0.4)),
            Own::Thorns { scale: Scale::ZERO, magic: true, status: Status { poison: 0, burn: 0, chill: 1.5 }, secs: 4.0 },
        ],
        ..spell("Ice Ward", Frost, 1, 4.6, SelfCast)
    },
    SpellDef { hits: &[magic(ap(1.1)), Hit::ConsumeChill(ap(0.9))], ..spell("Shatter", Frost, 2, 3.8, PROJECTILE) },
    SpellDef { hits: &[magic(ap(0.5)), Hit::Freeze(0.75)], ..spell("Glacial Prison", Frost, 2, 5.5, PROJECTILE) },
    SpellDef { own: &[buff(Stat::Armor, 25.0, 4.0), buff(Stat::Mr, 25.0, 4.0)], ..spell("Winter Mantle", Frost, 2, 5.5, SelfCast) },
    SpellDef {
        hits: &[magic(ap(0.3)), Hit::Chill(1.0)],
        ..spell("Hailstorm", Frost, 3, 4.8, Zone { radius: 150.0, ticks: 4, interval: 0.25, target: Densest })
    },
    SpellDef { own: &[barrier(Scale { barrier: 0.5, ..Scale::ZERO }), charge(Right, 0.75)], ..spell("Cold Storage", Frost, 1, 4.5, SelfCast) },
    SpellDef {
        own: &[Own::Thorns { scale: ap(0.4), magic: true, status: Status { poison: 0, burn: 0, chill: 0.75 }, secs: 4.0 }],
        ..spell("Rime Mirror", Frost, 2, 6.5, SelfCast)
    },
    SpellDef { hits: &[magic(ap(0.7)), if_target(TargetCond::Chilled, &[Hit::Chill(2.0)])], ..spell("Permafrost", Frost, 2, 4.0, NOVA_M) },
    SpellDef { own: &[heal(max_hp(0.08))], ..spell("Thawing Grace", Frost, 1, 5.5, SelfCast) },
    SpellDef {
        hits: &[physical(ad(0.85)), Hit::IfTarget(TargetCond::Chilled, &[physical(ad(0.25))], &[Hit::Chill(0.75)])],
        ..spell("Rime Edge", Frost, 1, 3.2, PROJECTILE)
    },
    SpellDef {
        own: &[barrier(Scale { flat: 40.0, mr: 0.7, ..Scale::ZERO }), Own::BarrierPer { count: Count::Nearby(TargetCond::Chilled), per: 20.0 }],
        ..spell("Glacial Treasury", Frost, 2, 5.0, SelfCast)
    },
    SpellDef { hits: &[Hit::Mark(MarkKind::Brittle, 4.0), Hit::Chill(1.0)], ..spell("Brittle Sigil", Frost, 2, 5.6, PROJECTILE) },
    SpellDef {
        hits: &[magic(ap(1.3)), Hit::IfTarget(TargetCond::Chilled, &[Hit::Chill(3.0)], &[Hit::Chill(1.5)])],
        ..spell("Whiteout", Frost, 3, 6.5, NOVA_L)
    },
    SpellDef { own: &[heal(sc(20.0, 0.0, 0.3))], ..spell("Thawwater", Frost, 1, 4.8, SelfCast) },
    SpellDef { own: &[barrier(max_hp(0.06)), charge(Left, 0.5)], ..spell("Frostspring", Frost, 2, 5.8, SelfCast) },
    // ------------------------------------------------------------- Lightning
    SpellDef { hits: &[magic(ap(0.9))], ..spell("Volt Javelin", Lightning, 1, 2.4, PROJECTILE) },
    SpellDef { hits: &[magic(ap(1.0))], ..spell("Chain Lightning", Lightning, 2, 3.6, Chain { jumps: 3, falloff: 0.7 }) },
    SpellDef { hits: &[magic(ap(1.5))], ..spell("Thunderclap", Lightning, 3, 5.0, NOVA_L) },
    SpellDef { own: &[Own::Empower { scope: Adjacent, pct: 30.0, secs: 4.0 }], ..spell("Live Wire", Lightning, 1, 6.0, SelfCast) },
    SpellDef {
        hits: &[magic(ap(0.6)), Hit::Reserve { kind: ReserveKind::BarrierAbsorbed, per: flat(0.15), at_least_one: false }],
        ..spell("Capacitor", Lightning, 2, 4.8, PROJECTILE)
    },
    SpellDef { own: &[buff(Stat::Mr, 15.0, 3.0), charge(Right, 0.75)], ..spell("Grounding Rod", Lightning, 1, 5.5, SelfCast) },
    SpellDef {
        hits: &[magic(ap(0.45))],
        ..spell("Ball Lightning", Lightning, 2, 3.8, Zone { radius: 100.0, ticks: 3, interval: 0.7, target: Densest })
    },
    SpellDef { hits: &[magic(ap(0.7))], boost: Some((SelfCond::BothNeighborsCast, 2.7)), ..spell("Storm Circuit", Lightning, 2, 3.8, PROJECTILE) },
    SpellDef { own: &[buff(Stat::DamagePct, 45.0, 5.0)], ..spell("Overvoltage", Lightning, 3, 6.0, SelfCast) },
    SpellDef {
        hits: &[physical(ad(0.55)), magic(ap(0.35))],
        boost: Some((SelfCond::AdNearAp, 1.25)),
        ..spell("Copper Fang", Lightning, 1, 3.0, PROJECTILE)
    },
    SpellDef { own: &[charge(Left, 1.0)], ..spell("Groundwire", Lightning, 1, 4.5, SelfCast) },
    SpellDef {
        hits: &[magic(ap(0.8)), Hit::Reserve { kind: ReserveKind::BarrierAbsorbed, per: flat(0.4), at_least_one: false }],
        ..spell("Stormglass", Lightning, 2, 5.0, PROJECTILE)
    },
    SpellDef {
        hits: &[magic(ap(0.85))],
        own: &[Own::If(SelfCond::NeighborsAllSize1, &[charge(Adjacent, 0.4)])],
        ..spell("Forked Current", Lightning, 2, 4.0, Chain { jumps: 3, falloff: 0.55 })
    },
    SpellDef {
        hits: &[magic(ap(1.4)), Hit::Reserve { kind: ReserveKind::BarrierGained, per: flat(0.2), at_least_one: false }],
        ..spell("Thunder Vault", Lightning, 3, 6.0, PROJECTILE)
    },
    // ---------------------------------------------------------------- Poison
    SpellDef { hits: &[physical(ad(0.4)), Hit::Poison(3)], ..spell("Venom Fang", Poison, 1, 2.7, PROJECTILE) },
    SpellDef { hits: &[Hit::Poison(5)], ..spell("Toxic Bloom", Poison, 2, 4.5, NOVA_M) },
    SpellDef { hits: &[Hit::ConsumePoison { scale: ap(0.4), per_stack: 15.0 }], ..spell("Septic Burst", Poison, 2, 4.5, PROJECTILE) },
    SpellDef {
        hits: &[Hit::Poison(2), if_target(TargetCond::PoisonAtLeast(8), &[Hit::Poison(3)])],
        ..spell("Virulent Seed", Poison, 1, 3.5, PROJECTILE)
    },
    SpellDef { own: &[heal(max_hp(0.05)), charge(Left, 0.5)], ..spell("Antidote", Poison, 1, 5.5, SelfCast) },
    SpellDef { hits: &[magic(ap(0.55)), Hit::ShredArmor(15.0)], ..spell("Corrosive Spit", Poison, 1, 4.2, PROJECTILE) },
    SpellDef {
        own: &[heal(max_hp(0.05)), Own::HealPer { count: Count::Nearby(TargetCond::Poisoned), per: 5.0 }],
        ..spell("Plague Doctor", Poison, 2, 5.0, SelfCast)
    },
    SpellDef { hits: &[Hit::Poison(8)], own: &[Own::PayHealth(0.04)], ..spell("Noxious Pact", Poison, 2, 4.8, PROJECTILE) },
    SpellDef { hits: &[Hit::PoisonAmp(5.0)], ..spell("Pandemic", Poison, 3, 5.5, NOVA_L) },
    SpellDef { hits: &[physical(ad(0.5)), Hit::Poison(2), bleed(8.0, 0.1)], ..spell("Viper Stitch", Poison, 1, 3.6, PROJECTILE) },
    SpellDef {
        own: &[
            barrier(max_hp(0.07)),
            Own::Thorns { scale: Scale::ZERO, magic: true, status: Status { poison: 2, burn: 0, chill: 0.0 }, secs: 4.0 },
        ],
        ..spell("Rotting Aegis", Poison, 2, 5.0, SelfCast)
    },
    SpellDef {
        hits: &[magic(ap(0.7)), Hit::Reserve { kind: ReserveKind::PoisonDamage, per: flat(0.18), at_least_one: false }],
        ..spell("Blight Ledger", Poison, 2, 5.5, PROJECTILE)
    },
    SpellDef { hits: &[Hit::Poison(5), Hit::Weaken(4.0)], ..spell("Withering Court", Poison, 3, 6.5, NOVA_L) },
    // ----------------------------------------------------------------- Blood
    SpellDef { hits: &[physical(ad(0.8)), Hit::Lifesteal(0.2)], ..spell("Blood Needle", Blood, 1, 2.5, PROJECTILE) },
    SpellDef { own: &[Own::PayHealth(0.05), buff(Stat::Ad, 25.0, 4.0)], ..spell("Crimson Pact", Blood, 1, 6.0, SelfCast) },
    SpellDef { hits: &[bleed(20.0, 0.3)], ..spell("Hemorrhage", Blood, 2, 4.0, PROJECTILE) },
    SpellDef {
        own: &[heal(max_hp(0.05)), Own::HealFromReserve { kind: ReserveKind::HealthPaid, ratio: 0.25 }],
        ..spell("Bloodwell", Blood, 2, 5.0, SelfCast)
    },
    SpellDef {
        own: &[Own::IfElse(SelfCond::BelowHealth(0.35), &[heal(max_hp(0.12))], &[heal(max_hp(0.03))])],
        ..spell("Last Vein", Blood, 1, 4.0, SelfCast)
    },
    SpellDef { own: &[Own::PayForBarrier { pct: 0.06, ratio: 2.5 }], ..spell("Sanguine Armor", Blood, 2, 4.5, SelfCast) },
    SpellDef { hits: &[physical(ad(1.2))], boost: Some((SelfCond::BelowHealth(0.5), 1.4)), ..spell("Heartseeker", Blood, 2, 3.5, PROJECTILE) },
    SpellDef { hits: &[magic(ap(0.6)), Hit::Lifesteal(0.3)], ..spell("Transfusion", Blood, 2, 4.5, NOVA_M) },
    SpellDef {
        hits: &[physical(ad(2.6)), Hit::HealOnKill(0.2)],
        ..spell("Red Harvest", Blood, 3, 4.5, Projectile { count: 1, pierce: 0, target: LowestHealth })
    },
    SpellDef { hits: &[physical(ad(0.55)), bleed(10.0, 0.15)], ..spell("Razor Petal", Blood, 1, 3.3, PROJECTILE) },
    SpellDef { own: &[buff(Stat::Armor, 15.0, 4.0)], ..spell("Scar Tissue", Blood, 1, 5.0, SelfCast) },
    SpellDef { hits: &[physical(ad(0.7)), bleed(12.0, 0.2)], ..spell("Open Wound", Blood, 2, 5.0, PROJECTILE) },
    SpellDef {
        own: &[Own::PayHealth(0.05), buff(Stat::Ad, 15.0, 4.0), buff(Stat::Ap, 15.0, 4.0)],
        ..spell("Sanguine Mirror", Blood, 2, 5.5, SelfCast)
    },
    SpellDef { own: &[Own::PayHealth(0.07), haste(Bar, 0.5, 2.5)], ..spell("Arterial Rush", Blood, 2, 6.5, SelfCast) },
    SpellDef {
        hits: &[bleed(12.0, 0.15)],
        own: &[Own::HealFromReserve { kind: ReserveKind::HealthPaid, ratio: 0.2 }],
        ..spell("Red Communion", Blood, 2, 5.5, NOVA_M)
    },
    SpellDef {
        hits: &[physical(ad(1.2)), if_target(TargetCond::Bleeding, &[physical(ad(0.9))])],
        ..spell("Crimson Deluge", Blood, 3, 6.2, NOVA_L)
    },
    SpellDef { own: &[Own::PayForBarrier { pct: 0.1, ratio: 2.0 }, buff(Stat::Ad, 20.0, 4.0)], ..spell("Heartforge", Blood, 3, 6.5, SelfCast) },
    // ---------------------------------------------------------------- Shadow
    SpellDef {
        hits: &[magic(sc(15.0, 0.0, 2.5))],
        boost: Some((SelfCond::BelowHealth(0.5), 1.5)),
        ..spell("Execution Spell", Shadow, 3, 3.0, PROJECTILE)
    },
    SpellDef { hits: &[physical_pen(ad(0.85), 0.2)], ..spell("Shadow Knife", Shadow, 1, 2.8, PROJECTILE) },
    SpellDef { hits: &[Hit::Mark(MarkKind::Dusk, 4.0)], ..spell("Dusk Mark", Shadow, 1, 6.5, PROJECTILE) },
    SpellDef { hits: &[magic(ap(0.9))], ..spell("Soul Tax", Shadow, 2, 4.0, PROJECTILE) },
    SpellDef { hits: &[magic(ap(0.9)), Hit::Weaken(3.0)], ..spell("Nightfall", Shadow, 3, 5.5, NOVA_L) },
    SpellDef {
        hits: &[magic(ap(1.5))],
        ..spell("Umbral Debt", Shadow, 2, 4.0, Zone { radius: 70.0, ticks: 1, interval: 1.5, target: Nearest })
    },
    SpellDef {
        hits: &[magic(ap(0.7))],
        own: &[charge(Left, 0.5)],
        boost: Some((SelfCond::KilledWithin(3.0), 1.7)),
        ..spell("Grave Whisper", Shadow, 1, 3.2, PROJECTILE)
    },
    SpellDef {
        hits: &[physical(ad(0.7)), magic(ap(0.7)), if_target(TargetCond::AboveHealth(0.5), &[physical(ad(0.18)), magic(ap(0.18))])],
        ..spell("Eclipse Blade", Shadow, 2, 3.8, PROJECTILE)
    },
    SpellDef { own: &[buff(Stat::DamageReduction, 25.0, 3.0)], ..spell("Soul Anchor", Shadow, 1, 5.5, SelfCast) },
    SpellDef { hits: &[physical(ad(0.75))], own: &[charge(Left, 0.3)], ..spell("Duskstep", Shadow, 1, 3.8, PROJECTILE) },
    SpellDef {
        hits: &[Hit::ShredMr(15.0)],
        own: &[Own::PayHealth(0.05), buff(Stat::Ap, 20.0, 4.0)],
        ..spell("Hollow Covenant", Shadow, 2, 5.5, PROJECTILE)
    },
    SpellDef { hits: &[magic(ap(0.65))], own: &[barrier(sc(15.0, 0.0, 0.15))], ..spell("Night Tithe", Shadow, 1, 4.5, PROJECTILE) },
    SpellDef {
        hits: &[physical(ad(1.35)), Hit::PerCount { count: Count::MarksOnTarget, per: ad(0.2), magic: false }],
        ..spell("Waning Blade", Shadow, 2, 4.3, PROJECTILE)
    },
    SpellDef { hits: &[magic(ap(1.35))], ..spell("Black Procession", Shadow, 3, 6.4, NOVA_L) },
    // ----------------------------------------------------------------- Light
    SpellDef { own: &[heal(max_hp(0.09))], ..spell("Sanctuary", Light, 1, 6.0, SelfCast) },
    SpellDef { own: &[heal(sc(20.0, 0.0, 0.35))], ..spell("Mending Ray", Light, 1, 2.6, SelfCast) },
    SpellDef {
        own: &[barrier(sc(60.0, 0.0, 0.7)), Own::If(SelfCond::FullHealth, &[barrier(flat(40.0))])],
        ..spell("Dawn Shield", Light, 2, 4.5, SelfCast)
    },
    SpellDef { own: &[heal(max_hp(0.07))], ..spell("Purifying Bell", Light, 2, 5.5, SelfCast) },
    SpellDef { hits: &[magic(ap(1.3))], boost: Some((SelfCond::AboveHealth(0.8), 1.3)), ..spell("Judgment", Light, 2, 5.5, PROJECTILE) },
    SpellDef { own: &[buff(Stat::HealingPct, 20.0, 4.0)], ..spell("Beacon", Light, 1, 6.0, SelfCast) },
    SpellDef { own: &[buff(Stat::HealToBarrier, 40.0, 4.0)], ..spell("Mercy Seal", Light, 2, 5.0, SelfCast) },
    SpellDef { hits: &[magic(ap(1.0))], own: &[heal(max_hp(0.07))], ..spell("Sunburst", Light, 3, 5.0, NOVA_L) },
    SpellDef {
        own: &[buff(Stat::Mr, 20.0, 4.0), Own::If(SelfCond::AboveHealth(0.7), &[buff(Stat::Armor, 20.0, 4.0)])],
        ..spell("Oath of Dawn", Light, 1, 5.5, SelfCast)
    },
    SpellDef {
        hits: &[magic(ap(0.6)), Hit::Reserve { kind: ReserveKind::Healing, per: flat(0.35), at_least_one: false }],
        ..spell("Radiant Reckoning", Light, 2, 4.0, PROJECTILE)
    },
    SpellDef { hits: &[magic(ap(0.7))], own: &[heal(sc(12.0, 0.0, 0.15))], ..spell("Dawn Needle", Light, 1, 3.2, PROJECTILE) },
    SpellDef {
        own: &[Own::ConsumeBarrier { fraction: 0.25, heal: 1.0, ap: 0.0, secs: 0.0 }, heal(sc(35.0, 0.0, 0.3))],
        ..spell("Merciful Exchange", Light, 2, 5.5, SelfCast)
    },
    SpellDef { own: &[buff(Stat::Armor, 12.0, 3.0), buff(Stat::Mr, 12.0, 3.0)], ..spell("Sunlit Guard", Light, 1, 5.0, SelfCast) },
    SpellDef {
        hits: &[magic(ap(1.3))],
        own: &[Own::IfElse(SelfCond::AboveHealth(0.75), &[charge(Right, 2.0)], &[charge(Right, 1.0)])],
        ..spell("Radiant Relay", Light, 2, 5.8, PROJECTILE)
    },
    SpellDef { own: &[buff(Stat::HealToBarrier, 30.0, 4.0)], ..spell("Vow of Shelter", Light, 2, 6.0, SelfCast) },
    SpellDef { hits: &[magic(ap(1.45)), Hit::Lifesteal(0.25)], ..spell("Daybreak Chorus", Light, 3, 7.0, NOVA_L) },
    // ---------------------------------------------------------------- Nature
    SpellDef { own: &[barrier(max_hp(0.22))], ..spell("Living Barrier", Nature, 2, 6.5, SelfCast) },
    SpellDef { hits: &[physical(Scale { ad: 0.7, max_hp: 0.02, ..Scale::ZERO })], ..spell("Bramble Lash", Nature, 1, 3.0, PROJECTILE) },
    SpellDef { own: &[Own::Regen { per_sec: max_hp(0.02), secs: 4.0 }], ..spell("Rejuvenation", Nature, 1, 5.0, SelfCast) },
    SpellDef {
        own: &[Own::Thorns { scale: sc(20.0, 0.2, 0.0), magic: false, status: Status::NONE, secs: 4.0 }],
        ..spell("Thorncoat", Nature, 2, 5.0, SelfCast)
    },
    SpellDef { own: &[Own::Grow { stat: Stat::MaxHp, amount: 10.0 }, heal(max_hp(0.05))], ..spell("Deep Roots", Nature, 2, 4.5, SelfCast) },
    SpellDef { hits: &[Hit::Poison(3)], own: &[barrier(sc(30.0, 0.0, 0.3))], ..spell("Spore Cloud", Nature, 2, 5.5, NOVA_M) },
    SpellDef {
        own: &[Own::ConsumeBarrier { fraction: 1.0, heal: 0.75, ap: 0.0, secs: 0.0 }, charge(Right, 0.75)],
        ..spell("Verdant Exchange", Nature, 1, 4.5, SelfCast)
    },
    SpellDef {
        own: &[buff(Stat::Armor, 15.0, 5.0), Own::Regen { per_sec: max_hp(0.02), secs: 5.0 }],
        ..spell("Ancient Grove", Nature, 3, 6.0, SelfCast)
    },
    SpellDef { own: &[buff(Stat::HealToBarrier, 50.0, 5.0)], ..spell("Bloomguard", Nature, 2, 7.5, SelfCast) },
    SpellDef {
        own: &[Own::Regen { per_sec: max_hp(0.025), secs: 4.0 }, Own::If(SelfCond::BelowHealth(0.4), &[heal(max_hp(0.1))])],
        ..spell("Seed of Renewal", Nature, 1, 5.0, SelfCast)
    },
    SpellDef { hits: &[physical(ad(0.65)), Hit::Poison(2)], ..spell("Briar Needle", Nature, 1, 3.0, PROJECTILE) },
    SpellDef { own: &[buff(Stat::Armor, 18.0, 4.0)], ..spell("Root Reservoir", Nature, 2, 6.0, SelfCast) },
    SpellDef { own: &[heal(sc(20.0, 0.0, 0.2)), charge(Left, 0.7)], ..spell("Bloom Relay", Nature, 1, 4.8, SelfCast) },
    SpellDef { hits: &[physical(Scale { ad: 1.1, ap: 0.165, ..Scale::ZERO })], ..spell("Wild Graft", Nature, 2, 5.5, PROJECTILE) },
    SpellDef {
        own: &[buff(Stat::Armor, 20.0, 4.0), Own::Thorns { scale: ad(0.4), magic: false, status: Status::NONE, secs: 4.0 }],
        ..spell("Thorn Orchard", Nature, 3, 6.5, SelfCast)
    },
    SpellDef { own: &[heal(sc(30.0, 0.0, 0.2))], ..spell("Verdant Reprieve", Nature, 2, 6.0, SelfCast) },
    // ----------------------------------------------------------------- Steel
    SpellDef { hits: &[physical(ad(1.2))], ..spell("True Strike", Steel, 1, 3.0, PROJECTILE) },
    SpellDef { hits: &[physical(ad(0.55))], ..spell("Twin Daggers", Steel, 1, 3.2, Projectile { count: 2, pierce: 0, target: Nearest }) },
    SpellDef { hits: &[physical(ad(1.0)), Hit::ShredArmor(25.0)], ..spell("Armorbreaker", Steel, 2, 4.0, PROJECTILE) },
    SpellDef { hits: &[physical(ad(1.1))], ..spell("Cleave", Steel, 2, 3.6, Cone { range: 280.0 }) },
    SpellDef { own: &[Own::Thorns { scale: ad(0.6), magic: false, status: Status::NONE, secs: 3.0 }], ..spell("Riposte", Steel, 1, 6.0, SelfCast) },
    SpellDef { own: &[Own::Empower { scope: Right, pct: 20.0, secs: 5.0 }, charge(Right, 1.0)], ..spell("Sharpen", Steel, 1, 6.0, SelfCast) },
    SpellDef { hits: &[physical(ad(3.0)), Hit::EliteBonus(1.5)], ..spell("Siegebreaker", Steel, 3, 4.5, PROJECTILE) },
    SpellDef { own: &[buff(Stat::Ad, 25.0, 4.0), buff(Stat::Armor, 10.0, 4.0)], ..spell("Duelist Stance", Steel, 2, 5.0, SelfCast) },
    SpellDef { hits: &[physical(ad(0.6))], ..spell("Ricochet Blade", Steel, 2, 4.0, Chain { jumps: 2, falloff: 1.0 }) },
    SpellDef {
        hits: &[Hit::IfTarget(TargetCond::BelowHealth(0.25), &[physical(ad(1.8))], &[physical(ad(1.1))])],
        ..spell("Finishing Thrust", Steel, 2, 3.8, Projectile { count: 1, pierce: 0, target: LowestHealth })
    },
    SpellDef { hits: &[physical(Scale { ad: 0.75, armor: 0.3, ..Scale::ZERO })], ..spell("Tempered Jab", Steel, 1, 2.8, PROJECTILE) },
    SpellDef { hits: &[physical(ad(0.6)), bleed(8.0, 0.1)], own: &[charge(Right, 0.5)], ..spell("Serrated Relay", Steel, 1, 4.0, PROJECTILE) },
    SpellDef { hits: &[physical_pen(ad(1.45), 0.3)], ..spell("Guardbreaker", Steel, 2, 4.5, PROJECTILE) },
    SpellDef { own: &[barrier(sc(20.0, 0.35, 0.0))], ..spell("Crossguard", Steel, 1, 4.5, SelfCast) },
    SpellDef { hits: &[physical(ad(1.0)), bleed(16.0, 0.2)], ..spell("Reaping Cut", Steel, 2, 5.0, Cone { range: 280.0 }) },
    SpellDef { hits: &[physical(ad(1.05))], own: &[haste(Adjacent, 0.25, 3.0)], ..spell("Iron Cadence", Steel, 2, 6.0, PROJECTILE) },
    SpellDef { hits: &[physical(Scale { ad: 1.25, armor: 0.9, ..Scale::ZERO })], ..spell("Weighted Edge", Steel, 2, 4.8, PROJECTILE) },
    SpellDef { hits: &[Hit::Mark(MarkKind::Duel, 4.0), physical(ad(0.5))], ..spell("Dueling Standard", Steel, 3, 6.5, PROJECTILE) },
    SpellDef { own: &[barrier(sc(35.0, 0.35, 0.0))], ..spell("Steel Rebound", Steel, 2, 5.5, SelfCast) },
    SpellDef {
        hits: &[physical(ad(2.1)), if_target(TargetCond::BelowHealth(0.5), &[physical(ad(0.7))])],
        ..spell("Final Measure", Steel, 3, 6.8, PROJECTILE)
    },
    // ----------------------------------------------------------------- Earth
    SpellDef { hits: &[physical(Scale { ad: 0.8, armor: 0.6, ..Scale::ZERO })], ..spell("Stone Fist", Earth, 1, 3.2, PROJECTILE) },
    SpellDef { own: &[buff(Stat::Armor, 30.0, 3.0)], ..spell("Granite Skin", Earth, 1, 5.0, SelfCast) },
    SpellDef {
        own: &[barrier(Scale { flat: 40.0, mr: 0.8, ..Scale::ZERO }), buff(Stat::Mr, 15.0, 3.0)],
        ..spell("Crystal Bastion", Earth, 2, 4.5, SelfCast)
    },
    SpellDef { hits: &[physical(ad(0.7)), Hit::ShredArmor(10.0)], ..spell("Fault Line", Earth, 2, 4.0, Cone { range: 320.0 }) },
    SpellDef { own: &[buff(Stat::DamageReduction, 20.0, 4.0)], ..spell("Obsidian Heart", Earth, 2, 5.5, SelfCast) },
    SpellDef {
        hits: &[physical(Scale { ad: 1.5, armor: 1.5, ..Scale::ZERO })],
        ..spell("Boulderfall", Earth, 3, 5.0, Blast { radius: 130.0, target: Nearest })
    },
    SpellDef {
        own: &[buff(Stat::Armor, -15.0, 3.0), barrier(flat(60.0)), charge(Left, 0.75)],
        ..spell("Earthen Exchange", Earth, 1, 4.5, SelfCast)
    },
    SpellDef { own: &[Own::BarrierPer { count: Count::Artifacts, per: 30.0 }], ..spell("Magnetic Shell", Earth, 2, 4.0, SelfCast) },
    SpellDef {
        own: &[Own::Grow { stat: Stat::Armor, amount: 5.0 }, Own::Grow { stat: Stat::Mr, amount: 5.0 }, heal(flat(20.0))],
        ..spell("Mountain Oath", Earth, 2, 5.0, SelfCast)
    },
    SpellDef { hits: &[magic(ap(0.6)), Hit::FromBarrier { ratio: 1.5, consume: true }], ..spell("Geode Burst", Earth, 2, 4.5, PROJECTILE) },
    SpellDef {
        hits: &[physical(Scale { ad: 0.25, armor: 0.1, ..Scale::ZERO })],
        ..spell("Pebble Volley", Earth, 1, 3.5, Projectile { count: 3, pierce: 0, target: Nearest })
    },
    SpellDef {
        hits: &[Hit::FromBarrier { ratio: 0.25, consume: false }],
        own: &[barrier(sc(35.0, 0.0, 0.35))],
        ..spell("Buried Pulse", Earth, 2, 5.0, PROJECTILE)
    },
    SpellDef { own: &[buff(Stat::Armor, 10.0, 3.0), charge(Right, 0.8)], ..spell("Stonewheel", Earth, 1, 4.2, SelfCast) },
    SpellDef {
        hits: &[physical(Scale { ad: 1.2, armor: 1.0, mr: 1.0, ..Scale::ZERO })],
        ..spell("Tectonic Verdict", Earth, 3, 7.0, NOVA_L)
    },
    SpellDef {
        own: &[Own::BarrierFromRecent { fraction: 0.45, secs: 3.0 }, barrier(flat(30.0))],
        ..spell("Granite Memory", Earth, 2, 6.0, SelfCast)
    },
    // ------------------------------------------------------------------ Time
    SpellDef { own: &[haste(Bar, 1.0, 2.0)], ..spell("Acceleration", Time, 1, 9.0, SelfCast) },
    SpellDef { own: &[charge(Right, 1.0)], ..spell("Quickstep", Time, 1, 3.5, SelfCast) },
    SpellDef { own: &[Own::Alternate(&[charge(Left, 1.5)], &[charge(Right, 1.5)])], ..spell("Pendulum", Time, 2, 4.0, SelfCast) },
    SpellDef { own: &[haste(Bar, 0.5, 3.0)], ..spell("Borrowed Time", Time, 2, 7.0, SelfCast) },
    SpellDef { own: &[Own::HealFromRecent { fraction: 0.35, secs: 3.0 }], ..spell("Rewind Flesh", Time, 2, 5.0, SelfCast) },
    SpellDef {
        own: &[barrier(flat(30.0)), Own::BarrierPer { count: Count::CoolingSpells, per: 10.0 }],
        ..spell("Hourglass Ward", Time, 1, 4.5, SelfCast)
    },
    SpellDef {
        hits: &[magic(ap(0.5))],
        ..spell("Delayed Verdict", Time, 2, 4.0, Zone { radius: 70.0, ticks: 3, interval: 1.0, target: Nearest })
    },
    SpellDef { hits: &[Hit::Freeze(0.5)], own: &[barrier(sc(40.0, 0.0, 0.4))], ..spell("Still Moment", Time, 2, 6.0, NOVA_M) },
    SpellDef { own: &[Own::ChargeNearestReady(1.25)], ..spell("Perfect Timing", Time, 1, 4.0, SelfCast) },
    SpellDef { own: &[charge(Left, 1.2)], ..spell("Quickstep Sigil", Time, 1, 4.0, SelfCast) },
    SpellDef { own: &[barrier(sc(20.0, 0.0, 0.2)), haste(Right, 0.2, 3.0)], ..spell("Minute Ward", Time, 1, 5.0, SelfCast) },
    SpellDef { own: &[haste(Left, 0.35, 3.0), charge(Right, 1.0)], ..spell("Split Tempo", Time, 2, 6.0, SelfCast) },
    SpellDef { hits: &[physical(ad(1.2))], own: &[charge(Left, 0.5)], ..spell("Borrowed Momentum", Time, 2, 5.0, PROJECTILE) },
    SpellDef {
        own: &[Own::HealFromRecent { fraction: 0.25, secs: 3.0 }, barrier(sc(25.0, 0.0, 0.25))],
        ..spell("Reversal Ward", Time, 2, 6.0, SelfCast)
    },
    SpellDef { hits: &[magic(ap(0.8)), Hit::Chill(1.5)], ..spell("Slowglass Needle", Time, 1, 3.8, PROJECTILE) },
    SpellDef { hits: &[physical(ad(0.75)), magic(ap(0.75))], ..spell("Pendulum Strike", Time, 2, 4.8, PROJECTILE) },
    SpellDef {
        hits: &[magic(ap(1.25)), if_target(TargetCond::Chilled, &[Hit::Lifesteal(0.35)])],
        ..spell("Clockwork Leech", Time, 2, 5.5, PROJECTILE)
    },
    SpellDef { hits: &[magic(ap(0.7))], own: &[Own::ChargeFraction { scope: Right, fraction: 0.2 }], ..spell("Patient Spark", Time, 1, 4.5, PROJECTILE) },
    SpellDef { own: &[charge(Adjacent, 2.5)], ..spell("Time Dividend", Time, 3, 7.0, SelfCast) },
    // --------------------------------------------------------------- Spirits
    SpellDef {
        hits: &[magic(ap(0.9))],
        own: &[charge(Left, 0.5)],
        ..spell("Spirit Arrow", Spirits, 1, 3.0, Projectile { count: 1, pierce: 0, target: HighestHealth })
    },
    SpellDef { own: &[buff(Stat::DamageReduction, 15.0, 4.0)], ..spell("Guardian Wisp", Spirits, 1, 6.0, SelfCast) },
    SpellDef { own: &[Own::Regen { per_sec: sc(15.0, 0.0, 0.2), secs: 3.0 }], ..spell("Ancestral Choir", Spirits, 2, 4.5, SelfCast) },
    SpellDef {
        own: &[barrier(flat(25.0)), Own::BarrierPer { count: Count::Nearby(TargetCond::BelowHealth(0.5)), per: 15.0 }, charge(Right, 0.75)],
        ..spell("Soul Lantern", Spirits, 1, 4.5, SelfCast)
    },
    SpellDef {
        hits: &[magic(ap(1.1))],
        ..spell("Spectral Pack", Spirits, 3, 4.5, Zone { radius: 70.0, ticks: 3, interval: 0.5, target: Nearest })
    },
    SpellDef { hits: &[Hit::Burn(6)], ..spell("Funeral Pyre", Spirits, 2, 4.5, NOVA_M) },
    SpellDef {
        own: &[barrier(max_hp(0.12)), Own::Regen { per_sec: max_hp(0.015), secs: 4.0 }],
        ..spell("Ancestral Aegis", Spirits, 3, 6.5, SelfCast)
    },
    SpellDef { hits: &[magic(Scale { ap: 0.65, mr: 0.35, ..Scale::ZERO })], ..spell("Wisp Lance", Spirits, 1, 3.3, PROJECTILE) },
    SpellDef {
        own: &[buff(Stat::Mr, 18.0, 4.0), barrier(Scale { flat: 25.0, mr: 0.4, ..Scale::ZERO })],
        ..spell("Ancestral Guard", Spirits, 2, 5.8, SelfCast)
    },
    SpellDef { own: &[heal(sc(18.0, 0.0, 0.15))], ..spell("Soul Stitch", Spirits, 1, 4.5, SelfCast) },
    SpellDef { hits: &[Hit::Mark(MarkKind::Vow, 4.0), magic(mr(0.2))], ..spell("Echoing Vow", Spirits, 2, 6.0, PROJECTILE) },
    SpellDef {
        own: &[barrier(sc(30.0, 0.0, 0.3)), Own::If(SelfCond::BelowHealth(0.5), &[charge(Left, 1.5)])],
        ..spell("Spirit Ferry", Spirits, 2, 5.5, SelfCast)
    },
    SpellDef {
        hits: &[magic(ap(1.1)), Hit::Burn(2)],
        own: &[Own::HealPer { count: Count::Nearby(TargetCond::Burning), per: 5.0 }],
        ..spell("Choir of Embers", Spirits, 3, 6.5, NOVA_L)
    },
    // --------------------------------------------------------------- Alchemy
    SpellDef { hits: &[magic(ap(0.6)), Hit::ShredMr(10.0)], ..spell("Acid Flask", Alchemy, 1, 3.5, PROJECTILE) },
    SpellDef {
        hits: &[magic(ap(0.9)), if_target(TargetCond::PoisonedAndBurning, &[magic(ap(0.9))])],
        ..spell("Volatile Mixture", Alchemy, 2, 4.0, PROJECTILE)
    },
    SpellDef { own: &[barrier(flat(20.0)), charge(Left, 0.5)], ..spell("Distillation", Alchemy, 1, 5.0, SelfCast) },
    SpellDef { own: &[Own::Catalyze { scope: Right, stacks: 3, secs: 4.0 }], ..spell("Catalyst", Alchemy, 1, 5.0, SelfCast) },
    SpellDef {
        hits: &[Hit::IfTarget(TargetCond::LowerPrThanMr, &[physical(ad(1.3))], &[magic(ap(1.3))])],
        ..spell("Mercury Blade", Alchemy, 2, 3.4, PROJECTILE)
    },
    SpellDef { own: &[heal(max_hp(0.07)), buff(Stat::Ad, 15.0, 4.0)], ..spell("Elixir of Vigor", Alchemy, 2, 5.0, SelfCast) },
    SpellDef {
        hits: &[magic(ap(2.2))],
        own: &[buff(Stat::Armor, -20.0, 3.0), buff(Stat::Mr, -20.0, 3.0)],
        ..spell("Glass Cannon", Alchemy, 3, 4.5, PROJECTILE)
    },
    SpellDef {
        own: &[Own::IfElse(SelfCond::BelowHealth(0.6), &[heal(max_hp(0.07))], &[barrier(max_hp(0.05))])],
        ..spell("Balanced Tonic", Alchemy, 1, 5.8, SelfCast)
    },
    SpellDef {
        hits: &[magic(ap(0.7)), Hit::PerCount { count: Count::Artifacts, per: ap(0.2), magic: true }],
        ..spell("Philosopher's Spark", Alchemy, 2, 4.5, PROJECTILE)
    },
    SpellDef { hits: &[physical_pen(ad(0.65), 0.15)], ..spell("Rust Tonic", Alchemy, 1, 4.0, PROJECTILE) },
    SpellDef { hits: &[Hit::Poison(4)], own: &[haste(Left, 0.25, 3.0)], ..spell("Catalytic Serum", Alchemy, 2, 5.5, PROJECTILE) },
    SpellDef { hits: &[bleed(12.0, 0.15)], own: &[barrier(flat(30.0))], ..spell("Coagulation Flask", Alchemy, 2, 5.0, PROJECTILE) },
    SpellDef {
        hits: &[magic(ap(0.9)), Hit::PerCount { count: Count::TargetStacks, per: ap(0.08), magic: true }],
        ..spell("Reactive Mixture", Alchemy, 2, 5.8, PROJECTILE)
    },
    SpellDef { own: &[heal(sc(45.0, 0.0, 0.45)), charge(Adjacent, 0.6)], ..spell("Panacea Engine", Alchemy, 3, 7.0, SelfCast) },
    // ---------------------------------------------------------------- Astral
    SpellDef { hits: &[magic(ap(1.4))], ..spell("Starfall", Astral, 2, 3.4, Blast { radius: 90.0, target: HighestHealth }) },
    SpellDef {
        own: &[Own::Alternate(&[barrier(sc(50.0, 0.0, 0.25))], &[heal(sc(30.0, 0.0, 0.25))])],
        ..spell("Lunar Ward", Astral, 1, 5.2, SelfCast)
    },
    SpellDef {
        hits: &[magic(ap(0.8)), if_target(TargetCond::AboveHealth(0.5), &[Hit::Burn(6)])],
        ..spell("Solar Flare", Astral, 2, 4.0, PROJECTILE)
    },
    SpellDef { hits: &[magic(ap(0.7))], ..spell("Gravity Well", Astral, 2, 4.5, NOVA_M) },
    SpellDef {
        hits: &[magic(ap(1.0)), Hit::PerCount { count: Count::EmptySlots, per: ap(0.8), magic: true }],
        ..spell("Constellation", Astral, 3, 5.0, PROJECTILE)
    },
    SpellDef { hits: &[magic(ap(1.0))], ..spell("Comet Tail", Astral, 2, 4.0, Chain { jumps: 1, falloff: 0.6 }) },
    SpellDef {
        own: &[Own::IfElse(SelfCond::AdAboveAp, &[buff(Stat::Ap, 20.0, 4.0)], &[buff(Stat::Ad, 20.0, 4.0)]), charge(Right, 0.5)],
        ..spell("Astral Balance", Astral, 1, 5.0, SelfCast)
    },
    SpellDef { hits: &[magic(ap(3.6))], ..spell("Supernova", Astral, 3, 7.0, Nova { radius: 260.0 }) },
    SpellDef {
        own: &[Own::IfElse(SelfCond::NeighborsDifferentSize, &[charge(Adjacent, 1.0)], &[barrier(sc(50.0, 0.0, 0.3))])],
        ..spell("Celestial Alignment", Astral, 2, 4.5, SelfCast)
    },
    SpellDef { hits: &[physical(ad(0.45)), magic(ap(0.45))], ..spell("Orbiting Blade", Astral, 1, 0.0, Orbit) },
    SpellDef { hits: &[magic(ap(0.9)), Hit::Mark(MarkKind::Gravity, 3.0)], ..spell("Gravity Brand", Astral, 2, 5.2, PROJECTILE) },
    SpellDef {
        own: &[barrier(Scale { flat: 30.0, armor: 0.35, mr: 0.35, ..Scale::ZERO })],
        ..spell("Prismatic Shelter", Astral, 2, 5.5, SelfCast)
    },
    SpellDef { hits: &[magic(ap(1.1))], own: &[charge(Adjacent, 0.7)], ..spell("Stellar Relay", Astral, 2, 6.0, PROJECTILE) },
    SpellDef { hits: &[physical(ad(0.95)), magic(ap(0.95))], ..spell("Convergence Engine", Astral, 3, 7.2, NOVA_L) },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delivery_and_effects_are_consistent() {
        for s in SPELLS {
            match s.delivery {
                SelfCast => assert!(s.hits.is_empty() && !s.own.is_empty(), "{} self-cast needs own effects only", s.name),
                _ => assert!(!s.hits.is_empty(), "{} hits nothing", s.name),
            }
            assert!((1..=3).contains(&s.size), "{} size", s.name);
            assert!(s.delivery == Orbit || s.cooldown > 0.0, "{} cooldown", s.name);
        }
    }

    #[test]
    fn catalogue_has_at_least_150_spells() {
        assert!(SPELLS.len() >= 150, "only {} spells", SPELLS.len());
    }
}
