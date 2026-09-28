//! A level-1 character: 7 skill points spent greedily near the Origin
//! (damage spells first), every learned spell that fits on the bar, no
//! supports or artifacts, then maps at tier 0 with the autopilot. Prints how
//! it goes and how much experience it earns.
//!
//! cargo run --release --example first_game
use arpg_machine::character::Character;
use arpg_machine::content::{Build, Slot};
use arpg_machine::tree::{SkillTree, ORIGIN};
use arpg_machine::world::{autopilot, Phase, World};

/// Spend points on the cheapest visible damaging nodes, then anything.
fn spend(tree: &mut SkillTree) {
    loop {
        let mut options: Vec<(usize, bool, usize)> = (1..tree.nodes.len())
            .filter(|&i| !tree.acquired[i] && tree.visible(i))
            .filter_map(|i| tree.route_to(i).map(|r| (i, tree.nodes[i].spell.unwrap().hits_enemies(), r.len())))
            .filter(|&(_, _, cost)| cost as u32 <= tree.remaining())
            .collect();
        options.sort_by_key(|&(_, damaging, cost)| (!damaging, cost));
        match options.first() {
            Some(&(id, _, _)) => {
                tree.acquire(id);
            }
            None => break,
        }
    }
}

fn main() {
    for seed in 1..=5u64 {
        let mut character = Character::default();
        let mut tree = SkillTree::generate(seed, character.skill_points());
        spend(&mut tree);
        let mut build = Build::empty();
        for spell in tree.acquired_spells() {
            let end = build.slots.len();
            if build.used() + spell.size() <= build.capacity {
                build.place_spell(spell, end);
            }
        }
        let names: Vec<&str> = build.slots.iter().map(|s: &Slot| s.spell.name()).collect();
        let exits = tree.nodes[ORIGIN].neighbors.len();
        println!("seed {seed}: {} spells from {exits} exits: {}", names.len(), names.join(", "));
        for map in 1..=2 {
            let mut world = World::new(build.clone(), seed * 100 + map, 1);
            while world.time < 900.0 && !matches!(world.phase, Phase::Victory | Phase::Defeat) {
                let input = autopilot(&world);
                world.step(&input);
                world.events.clear();
            }
            let levels = character.add_xp(world.xp);
            println!(
                "   map {map}: {:?} at {:.0}s (clear {:.0}s, boss {:.0}s), {} kills, {} XP -> level {} (+{levels})",
                world.phase,
                world.time,
                world.clear_time.unwrap_or(f32::NAN),
                world.boss_start.map_or(f32::NAN, |s| world.end_time.unwrap_or(world.time) - s),
                world.kills,
                world.xp,
                character.level
            );
        }
    }
}
