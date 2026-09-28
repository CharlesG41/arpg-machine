//! Every preset against every boss at tier 1, with the autopilot: wins, boss
//! fight length and damage taken during the fight.
//!
//! cargo run --release --example boss_bench [seeds]
use arpg_machine::content::presets;
use arpg_machine::world::{autopilot, Phase, World, BOSSES};

fn main() {
    let seeds: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(5);
    for boss in BOSSES {
        println!("\n{} ({} HP, PR {}, MR {})", boss.name, boss.hp, boss.armor, boss.mr);
        println!("build         wins   boss s (median)   hp lost in fight");
        for (name, build) in presets() {
            let mut wins = 0;
            let mut times = Vec::new();
            let mut lost = Vec::new();
            for seed in 1..=seeds {
                let mut world = World::new(build.clone(), seed, 1).with_boss(boss.kind);
                let mut hp_at_boss = None;
                while world.time < 900.0 && !matches!(world.phase, Phase::Victory | Phase::Defeat) {
                    let input = autopilot(&world);
                    world.step(&input);
                    world.events.clear();
                    if world.boss_start.is_some() && hp_at_boss.is_none() {
                        hp_at_boss = Some(world.player.hp + world.player.barrier);
                    }
                }
                if world.phase == Phase::Victory {
                    wins += 1;
                }
                if let Some(start) = world.boss_start {
                    times.push(world.end_time.unwrap_or(world.time) - start);
                    lost.push(hp_at_boss.unwrap_or(0.0) - (world.player.hp.max(0.0) + world.player.barrier));
                }
            }
            times.sort_by(f32::total_cmp);
            let median = times.get(times.len() / 2).copied().unwrap_or(f32::NAN);
            let avg_lost = lost.iter().sum::<f32>() / lost.len().max(1) as f32;
            println!("{name:<12} {wins}/{seeds}   {median:>8.1}           {avg_lost:>6.0}");
        }
    }
}
