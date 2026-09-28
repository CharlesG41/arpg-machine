//! When loot drops in a map (for picking screenshot moments and checking pacing).
//!
//! cargo run --release --example loot_times
use arpg_machine::content::presets;
use arpg_machine::world::{autopilot, Event, Phase, World};

fn main() {
    for (name, build) in presets() {
        let mut world = World::new(build, 1, 1);
        let mut times = Vec::new();
        while world.time < 600.0 && !matches!(world.phase, Phase::Victory | Phase::Defeat) {
            let input = autopilot(&world);
            world.step(&input);
            for e in &world.events {
                if let Event::LootDropped { loot, .. } = e {
                    times.push(format!("{:.0}s {}", world.time, loot.name()));
                }
            }
            world.events.clear();
        }
        println!("{name:<10} {:?} at {:.0}s: {}", world.phase, world.time, times.join(", "));
    }
}
