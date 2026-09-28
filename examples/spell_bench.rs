//! Each spell next to a basic Arcane Bolt, 60 s on the same map with the
//! autopilot and no incoming damage. Damage spells are compared with the
//! median of their delivery type to flag outliers for balancing. Self-cast
//! spells are listed apart: their value is survival and tempo, not damage.
//!
//! cargo run --release --example spell_bench
use arpg_machine::content::{Build, Delivery, Slot, SpellId};
use arpg_machine::world::{autopilot, World};

fn run(build: Build) -> (u32, f32) {
    let mut world = World::new(build, 7, 1);
    world.god_mode = true;
    while world.time < 60.0 {
        let input = autopilot(&world);
        world.step(&input);
        world.events.clear();
    }
    (world.kills, world.stats.iter().map(|s| s.damage).sum())
}

fn main() {
    let bolt = SpellId::named("Arcane Bolt");
    let slot = |spell| Slot { spell, support: None };
    let (_, base) = run(Build::new(vec![slot(bolt)], vec![]));
    println!("baseline: Arcane Bolt alone deals {base:.0} damage in 60 s\n");
    let mut rows: Vec<(f32, u32, SpellId)> = SpellId::all()
        .filter(|&s| s != bolt && s.delivery() != Delivery::SelfCast)
        .map(|spell| {
            let (kills, damage) = run(Build::new(vec![slot(spell), slot(bolt)], vec![]));
            (damage - base, kills, spell)
        })
        .collect();
    rows.sort_by(|a, b| b.0.total_cmp(&a.0));
    let median_of = |name: &str| {
        let mut v: Vec<f32> = rows.iter().filter(|r| r.2.delivery().name() == name).map(|r| r.0).collect();
        v.sort_by(f32::total_cmp);
        v.get(v.len() / 2).copied().unwrap_or(1.0).max(1.0)
    };
    println!("{:<22} {:<10} {:<10} {:>4} {:>11} {:>6}", "spell", "family", "delivery", "size", "added dmg", "kills");
    for (added, kills, spell) in &rows {
        let ratio = added / median_of(spell.delivery().name());
        let flag = if ratio > 2.0 {
            "  <- strong for its type"
        } else if ratio < 0.4 {
            "  <- weak for its type"
        } else {
            ""
        };
        println!(
            "{:<22} {:<10} {:<10} {:>4} {:>11.0} {:>6}{flag}",
            spell.name(),
            spell.family().name(),
            spell.delivery().name(),
            spell.size(),
            added,
            kills
        );
    }
}
