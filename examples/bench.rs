//! Headless benchmark: every preset on a few seeds, driven by the autopilot.
//!
//! cargo run --release --example bench [tier]
use arpg_machine::content::presets;
use arpg_machine::world::{autopilot, Phase, World};

fn main() {
    let tier: u32 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(1);
    let seeds = [1u64, 2, 3, 4, 5];
    println!("tier {tier}, seeds {seeds:?}\n");
    println!("{:<12} {:>8} {:>8} {:>8} {:>9} {:>7} {:>7}", "build", "clear s", "boss s", "total s", "peak k/m", "kills", "wins");
    for (name, build) in presets() {
        let (mut clear, mut boss, mut total, mut peak, mut kills, mut wins) = (0.0, 0.0, 0.0, 0.0, 0u32, 0);
        let mut runs = Vec::new();
        for &seed in &seeds {
            let mut world = World::new(build.clone(), seed, tier);
            while world.time < 900.0 && !matches!(world.phase, Phase::Victory | Phase::Defeat) {
                let input = autopilot(&world);
                world.step(&input);
                world.events.clear();
            }
            let c = world.clear_time.unwrap_or(world.time);
            let b = world.boss_start.map(|s| world.end_time.unwrap_or(world.time) - s).unwrap_or(0.0);
            clear += c;
            boss += b;
            total += world.time;
            peak += world.peak_kills_per_minute;
            kills += world.kills;
            if world.phase == Phase::Victory {
                wins += 1;
            }
            runs.push(format!("{:?}@{:.0}s", world.phase, world.time));
        }
        let n = seeds.len() as f32;
        println!(
            "{:<12} {:>8.1} {:>8.1} {:>8.1} {:>9.0} {:>7} {:>5}/{}",
            name,
            clear / n,
            boss / n,
            total / n,
            peak / n,
            kills / seeds.len() as u32,
            wins,
            seeds.len()
        );
        println!("             {}", runs.join("  "));
    }
}
