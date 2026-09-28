//! The spell catalogue, adapted from God Tower (design/spells-150 and
//! design/spells-250 in the auto-battler repo). Names, families, sizes,
//! cooldowns and ratios come from there; each spell gained a delivery so it
//! works in a map. Duel-only rules (enemy spell bars, enemy Barriers, contracts)
//! were dropped or reworded.
use crate::content::Delivery::*;
use crate::content::Family::*;
use crate::content::Targeting::*;
use crate::content::*;

const fn magic(scale: Scale) -> Hit {
    Hit::Damage { scale, magic: true, pen: 0.0 }
}

const fn physical(scale: Scale) -> Hit {
    Hit::Damage { scale, magic: false, pen: 0.0 }
}

const PROJECTILE: Delivery = Projectile { count: 1, pierce: 0, target: Nearest };
const NOVA_M: Delivery = Nova { radius: 190.0 };
const NOVA_L: Delivery = Nova { radius: 240.0 };

const fn spell(name: &'static str, family: Family, size: u8, cooldown: f32, delivery: Delivery) -> SpellDef {
    SpellDef { name, family, size, cooldown, delivery, hits: &[], own: &[], boost: None, growth: 0.0 }
}

pub static SPELLS: &[SpellDef] = &[
    // ---- Arcane ----
    SpellDef { hits: &[magic(ap(0.9))], ..spell("Arcane Bolt", Arcane, 1, 2.5, PROJECTILE) },
    SpellDef {
        hits: &[magic(Scale { flat: 18.0, ap: 1.1, ..Scale::ZERO })],
        growth: 3.0,
        ..spell("Event Horizon", Arcane, 1, 4.0, PROJECTILE)
    },
    SpellDef {
        hits: &[Hit::Damage { scale: ap(1.5), magic: true, pen: 0.3 }],
        ..spell("Prismatic Lance", Arcane, 2, 3.2, Projectile { count: 1, pierce: 2, target: Nearest })
    },
    SpellDef { own: &[Own::Buff { stat: Stat::Ap, amount: 20.0, secs: 4.0 }], ..spell("Mana Lens", Arcane, 1, 5.5, SelfCast) },
    SpellDef {
        hits: &[magic(ap(0.95))],
        own: &[Own::If(SelfCond::NoBarrier, &[Own::Barrier(Scale { flat: 20.0, ap: 0.2, ..Scale::ZERO })])],
        ..spell("Empty Vessel", Arcane, 1, 4.0, PROJECTILE)
    },
    // ---- Fire ----
    SpellDef { hits: &[magic(ap(0.45)), Hit::Burn(3)], ..spell("Cinder Dart", Fire, 1, 2.6, PROJECTILE) },
    SpellDef { hits: &[Hit::Burn(5)], ..spell("Ember Engine", Fire, 2, 5.0, NOVA_M) },
    SpellDef {
        hits: &[Hit::ConsumeBurn { scale: ap(0.5), per_stack: 10.0 }],
        ..spell("Flashover", Fire, 2, 4.0, PROJECTILE)
    },
    SpellDef { hits: &[Hit::SpreadBurn { radius: 220.0 }], ..spell("Wild Spark", Fire, 1, 4.0, PROJECTILE) },
    SpellDef {
        hits: &[magic(ap(1.0)), Hit::BurnIfBurning(6)],
        ..spell("Ashfall", Fire, 3, 4.8, Blast { radius: 180.0, target: Densest })
    },
    // ---- Frost ----
    SpellDef { hits: &[magic(ap(0.65)), Hit::Chill(1.0)], ..spell("Frost Needle", Frost, 1, 2.8, PROJECTILE) },
    SpellDef { hits: &[magic(ap(1.1)), Hit::ConsumeChill(ap(0.9))], ..spell("Shatter", Frost, 2, 3.8, PROJECTILE) },
    SpellDef { hits: &[magic(ap(0.5)), Hit::Freeze(0.75)], ..spell("Glacial Prison", Frost, 2, 5.5, PROJECTILE) },
    SpellDef {
        hits: &[magic(ap(0.3)), Hit::Chill(1.0)],
        ..spell("Hailstorm", Frost, 3, 4.8, Zone { radius: 150.0, ticks: 4, interval: 0.25, target: Densest })
    },
    SpellDef {
        own: &[
            Own::Barrier(Scale { flat: 35.0, ap: 0.4, ..Scale::ZERO }),
            Own::Thorns { scale: Scale::ZERO, magic: true, chill: 1.5, secs: 4.0 },
        ],
        ..spell("Ice Ward", Frost, 1, 4.6, SelfCast)
    },
    // ---- Lightning ----
    SpellDef { hits: &[magic(ap(0.9))], ..spell("Volt Javelin", Lightning, 1, 2.4, PROJECTILE) },
    SpellDef { hits: &[magic(ap(1.0))], ..spell("Chain Lightning", Lightning, 2, 3.6, Chain { jumps: 3, falloff: 0.7 }) },
    SpellDef { hits: &[magic(ap(1.5))], ..spell("Thunderclap", Lightning, 3, 5.0, NOVA_L) },
    SpellDef {
        hits: &[magic(ap(0.45))],
        ..spell("Ball Lightning", Lightning, 2, 3.8, Zone { radius: 100.0, ticks: 3, interval: 0.7, target: Densest })
    },
    SpellDef {
        own: &[Own::Buff { stat: Stat::DamagePct, amount: 45.0, secs: 5.0 }],
        ..spell("Overvoltage", Lightning, 3, 6.0, SelfCast)
    },
    // ---- Poison ----
    SpellDef { hits: &[physical(ad(0.4)), Hit::Poison(3)], ..spell("Venom Fang", Poison, 1, 2.7, PROJECTILE) },
    SpellDef { hits: &[Hit::Poison(5)], ..spell("Toxic Bloom", Poison, 2, 4.5, NOVA_M) },
    SpellDef {
        hits: &[Hit::ConsumePoison { scale: ap(0.4), per_stack: 15.0 }],
        ..spell("Septic Burst", Poison, 2, 4.5, PROJECTILE)
    },
    SpellDef { hits: &[Hit::Poison(8)], own: &[Own::PayHealth(0.04)], ..spell("Noxious Pact", Poison, 2, 4.8, PROJECTILE) },
    SpellDef { hits: &[Hit::PoisonAmp(5.0)], ..spell("Pandemic", Poison, 3, 5.5, NOVA_L) },
    // ---- Blood ----
    SpellDef { hits: &[physical(ad(0.8)), Hit::Lifesteal(0.2)], ..spell("Blood Needle", Blood, 1, 2.5, PROJECTILE) },
    SpellDef {
        own: &[Own::PayHealth(0.05), Own::Buff { stat: Stat::Ad, amount: 25.0, secs: 4.0 }],
        ..spell("Crimson Pact", Blood, 1, 6.0, SelfCast)
    },
    SpellDef {
        hits: &[Hit::Bleed { scale: Scale { flat: 20.0, ad: 0.3, ..Scale::ZERO }, secs: 3.0 }],
        ..spell("Hemorrhage", Blood, 2, 4.0, PROJECTILE)
    },
    SpellDef {
        hits: &[physical(ad(1.2))],
        boost: Some((SelfCond::BelowHealth(0.5), 1.4)),
        ..spell("Heartseeker", Blood, 2, 3.5, PROJECTILE)
    },
    SpellDef {
        hits: &[physical(ad(2.6)), Hit::HealOnKill(0.2)],
        ..spell("Red Harvest", Blood, 3, 4.5, Projectile { count: 1, pierce: 0, target: LowestHealth })
    },
    // ---- Shadow ----
    SpellDef { hits: &[Hit::Damage { scale: ad(0.85), magic: false, pen: 0.2 }], ..spell("Shadow Knife", Shadow, 1, 2.8, PROJECTILE) },
    SpellDef {
        hits: &[magic(Scale { flat: 15.0, ap: 2.5, ..Scale::ZERO })],
        boost: Some((SelfCond::BelowHealth(0.5), 1.5)),
        ..spell("Execution Spell", Shadow, 3, 3.0, PROJECTILE)
    },
    SpellDef {
        hits: &[magic(ap(1.5))],
        ..spell("Umbral Debt", Shadow, 2, 4.0, Zone { radius: 70.0, ticks: 1, interval: 1.5, target: Nearest })
    },
    SpellDef { hits: &[magic(ap(0.9)), Hit::Weaken(3.0)], ..spell("Nightfall", Shadow, 3, 5.5, NOVA_L) },
    // ---- Light ----
    SpellDef { own: &[Own::Heal(Scale { flat: 20.0, ap: 0.35, ..Scale::ZERO })], ..spell("Mending Ray", Light, 1, 2.6, SelfCast) },
    SpellDef {
        own: &[
            Own::Barrier(Scale { flat: 60.0, ap: 0.7, ..Scale::ZERO }),
            Own::If(SelfCond::FullHealth, &[Own::Barrier(flat(40.0))]),
        ],
        ..spell("Dawn Shield", Light, 2, 4.5, SelfCast)
    },
    SpellDef {
        hits: &[magic(ap(1.3))],
        boost: Some((SelfCond::AboveHealth(0.8), 1.3)),
        ..spell("Judgment", Light, 2, 5.5, PROJECTILE)
    },
    SpellDef { hits: &[magic(ap(1.0))], own: &[Own::Heal(max_hp(0.07))], ..spell("Sunburst", Light, 3, 5.0, NOVA_L) },
    // ---- Nature ----
    SpellDef { own: &[Own::Barrier(max_hp(0.22))], ..spell("Living Barrier", Nature, 2, 6.5, SelfCast) },
    SpellDef {
        hits: &[physical(Scale { ad: 0.7, max_hp: 0.02, ..Scale::ZERO })],
        ..spell("Bramble Lash", Nature, 1, 3.0, PROJECTILE)
    },
    SpellDef {
        own: &[Own::Thorns { scale: Scale { flat: 20.0, ad: 0.2, ..Scale::ZERO }, magic: false, chill: 0.0, secs: 4.0 }],
        ..spell("Thorncoat", Nature, 2, 5.0, SelfCast)
    },
    SpellDef { own: &[Own::Regen { per_sec: max_hp(0.02), secs: 4.0 }], ..spell("Rejuvenation", Nature, 1, 5.0, SelfCast) },
    // ---- Steel ----
    SpellDef { hits: &[physical(ad(1.2))], ..spell("True Strike", Steel, 1, 3.0, PROJECTILE) },
    SpellDef {
        hits: &[physical(ad(0.55))],
        ..spell("Twin Daggers", Steel, 1, 3.2, Projectile { count: 2, pierce: 0, target: Nearest })
    },
    SpellDef { hits: &[physical(ad(1.1))], ..spell("Cleave", Steel, 2, 3.6, Cone { range: 280.0 }) },
    SpellDef { hits: &[physical(ad(0.6))], ..spell("Ricochet Blade", Steel, 2, 4.0, Chain { jumps: 2, falloff: 1.0 }) },
    SpellDef { hits: &[physical(ad(3.0)), Hit::EliteBonus(1.5)], ..spell("Siegebreaker", Steel, 3, 4.5, PROJECTILE) },
    // ---- Earth ----
    SpellDef {
        hits: &[physical(Scale { ad: 0.8, armor: 0.6, ..Scale::ZERO })],
        ..spell("Stone Fist", Earth, 1, 3.2, PROJECTILE)
    },
    SpellDef { own: &[Own::Buff { stat: Stat::Armor, amount: 30.0, secs: 3.0 }], ..spell("Granite Skin", Earth, 1, 5.0, SelfCast) },
    SpellDef { hits: &[physical(ad(0.7)), Hit::ShredArmor(10.0)], ..spell("Fault Line", Earth, 2, 4.0, Cone { range: 320.0 }) },
    SpellDef {
        hits: &[physical(Scale { ad: 1.5, armor: 1.5, ..Scale::ZERO })],
        ..spell("Boulderfall", Earth, 3, 5.0, Blast { radius: 130.0, target: Nearest })
    },
    // ---- Time ----
    SpellDef { own: &[Own::Haste { scope: Scope::Bar, rate: 1.0, secs: 2.0 }], ..spell("Acceleration", Time, 1, 9.0, SelfCast) },
    SpellDef { own: &[Own::Charge { scope: Scope::Right, secs: 1.0 }], ..spell("Quickstep", Time, 1, 3.5, SelfCast) },
    SpellDef {
        hits: &[magic(ap(0.5))],
        ..spell("Delayed Verdict", Time, 2, 4.0, Zone { radius: 70.0, ticks: 3, interval: 1.0, target: Nearest })
    },
    SpellDef {
        hits: &[Hit::Freeze(0.5)],
        own: &[Own::Barrier(Scale { flat: 40.0, ap: 0.4, ..Scale::ZERO })],
        ..spell("Still Moment", Time, 2, 6.0, NOVA_M)
    },
    // ---- Spirits ----
    SpellDef {
        hits: &[magic(ap(0.9))],
        own: &[Own::Charge { scope: Scope::Left, secs: 0.5 }],
        ..spell("Spirit Arrow", Spirits, 1, 3.0, Projectile { count: 1, pierce: 0, target: HighestHealth })
    },
    SpellDef {
        hits: &[magic(ap(1.1))],
        ..spell("Spectral Pack", Spirits, 3, 4.5, Zone { radius: 70.0, ticks: 3, interval: 0.5, target: Nearest })
    },
    SpellDef {
        own: &[Own::Regen { per_sec: Scale { flat: 15.0, ap: 0.2, ..Scale::ZERO }, secs: 3.0 }],
        ..spell("Ancestral Choir", Spirits, 2, 4.5, SelfCast)
    },
    // ---- Alchemy ----
    SpellDef { hits: &[magic(ap(0.6)), Hit::ShredMr(10.0)], ..spell("Acid Flask", Alchemy, 1, 3.5, PROJECTILE) },
    SpellDef {
        hits: &[magic(ap(0.9)), Hit::IfPoisonedAndBurning(ap(0.9))],
        ..spell("Volatile Mixture", Alchemy, 2, 4.0, PROJECTILE)
    },
    SpellDef {
        hits: &[magic(ap(2.2))],
        own: &[Own::Buff { stat: Stat::Armor, amount: -20.0, secs: 3.0 }, Own::Buff { stat: Stat::Mr, amount: -20.0, secs: 3.0 }],
        ..spell("Glass Cannon", Alchemy, 3, 4.5, PROJECTILE)
    },
    // ---- Astral ----
    SpellDef { hits: &[magic(ap(1.4))], ..spell("Starfall", Astral, 2, 3.4, Blast { radius: 90.0, target: HighestHealth }) },
    SpellDef { hits: &[magic(ap(3.6))], ..spell("Supernova", Astral, 3, 7.0, Nova { radius: 260.0 }) },
    SpellDef { hits: &[physical(ad(0.45)), magic(ap(0.45))], ..spell("Orbiting Blade", Astral, 1, 0.0, Orbit) },
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
}
