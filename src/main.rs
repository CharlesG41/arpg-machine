//! ARPG Machine prototype window.
//!
//! Debug environment variables (used for automated screenshots):
//! - `ARPG_PRESET=<name>`: skip planning and start a map with that preset.
//! - `ARPG_PLAN=<name>`: open the loadout with that preset loaded.
//! - `ARPG_TREE_DEMO=<n>`: open the skill tree with n routes already bought.
//! - `ARPG_AUTOPILOT=1`: let the autopilot move.
//! - `ARPG_SHOT=<path.png>` and `ARPG_SHOT_AT=<seconds>`: save a screenshot
//!   after that much time on the current screen, then quit.
mod planner;
mod render;
mod ui;

use arpg_machine::content::presets;
use arpg_machine::math::V2;
use arpg_machine::world::{autopilot, Input, Phase, World, DT};
use macroquad::prelude::*;
use render::Fx;
use planner::{PlanAction, Planner, Tab};
use ui::{ResultAction, Ui};

fn conf() -> Conf {
    Conf {
        window_title: "ARPG Machine — prototype".to_string(),
        window_width: 1600,
        window_height: 900,
        high_dpi: true,
        sample_count: 4,
        ..Default::default()
    }
}

enum Screen {
    Plan,
    Play,
    Result,
}

fn read_input() -> Input {
    let mut dir = V2::ZERO;
    let down = |keys: &[KeyCode]| keys.iter().any(|&k| is_key_down(k));
    if down(&[KeyCode::W, KeyCode::Z, KeyCode::Up]) {
        dir.y -= 1.0;
    }
    if down(&[KeyCode::S, KeyCode::Down]) {
        dir.y += 1.0;
    }
    if down(&[KeyCode::A, KeyCode::Q, KeyCode::Left]) {
        dir.x -= 1.0;
    }
    if down(&[KeyCode::D, KeyCode::Right]) {
        dir.x += 1.0;
    }
    Input { move_dir: dir, dash: is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::LeftShift) }
}

fn save_screenshot(path: &str) {
    if let Some(dir) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    get_screen_data().export_png(path);
}

#[macroquad::main(conf)]
async fn main() {
    let font = load_ttf_font_from_bytes(include_bytes!("../assets/Barlow-SemiBold.ttf")).expect("bundled Barlow font");
    let ui = Ui { font };
    let env = |k: &str| std::env::var(k).ok();
    // Debug runs (screenshots, presets) never touch your saved character.
    let debug_run = ["ARPG_SHOT", "ARPG_PRESET", "ARPG_PLAN", "ARPG_TREE_DEMO", "ARPG_AUTOPILOT", "ARPG_COLLECTION"].iter().any(|k| env(k).is_some());
    let mut planner = match (debug_run, arpg_machine::save::CharacterSave::load()) {
        (false, Some(save)) => Planner::from_save(&save),
        _ => Planner::new(1),
    };
    if debug_run {
        planner.detach_from_disk();
    }
    let mut saved_text = planner.save_text();
    let mut map_seed = 1u64;
    let mut screen = Screen::Plan;
    let mut world: Option<World> = None;
    let mut fx = Fx::new();
    let mut accumulator = 0.0f32;
    let mut show_stats = true;
    let mut use_autopilot = env("ARPG_AUTOPILOT").is_some();
    let mut dash_queued = false;
    let mut result_delay = 0.0f32;
    // Experience from the current map goes to the character once, when it ends.
    let mut xp_banked = false;
    // Artifacts the last map's loot fused up, for the result screen.
    let mut fusions = Vec::new();
    let shot_path = env("ARPG_SHOT");
    let shot_at: f32 = env("ARPG_SHOT_AT").and_then(|s| s.parse().ok()).unwrap_or(1.0);
    let mut screen_time = 0.0f32;

    let find_preset = |name: &str| presets().into_iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|p| p.1);
    if let Some(build) = env("ARPG_PRESET").and_then(|n| find_preset(&n)) {
        planner.load_preset(build);
        world = Some(planner.new_world(map_seed));
        screen = Screen::Play;
    }
    if let Some(build) = env("ARPG_PLAN").and_then(|n| find_preset(&n)) {
        planner.load_preset(build);
        planner.tab = Tab::Loadout;
    }
    if env("ARPG_COLLECTION").is_some() {
        planner.demo_collection();
    }
    if let Some(n) = env("ARPG_TREE_DEMO").and_then(|n| n.parse::<usize>().ok()) {
        // Buy a few routes toward far nodes, then put the learned spells on the bar.
        for k in 0..n {
            let target = (1..planner.tree.nodes.len()).rev().nth(k * 7).unwrap_or(1);
            let _ = planner.tree.acquire(target);
        }
        for spell in planner.tree.acquired_spells().into_iter().take(3) {
            let end = planner.build.slots.len();
            planner.build.place_spell(spell, end);
        }
    }

    loop {
        let frame = get_frame_time().min(0.1);
        screen_time += frame;
        clear_background(render::BACKGROUND);
        match screen {
            Screen::Plan => {
                if let PlanAction::Play = planner.draw(&ui) {
                    world = Some(planner.new_world(map_seed));
                    fx = Fx::new();
                    accumulator = 0.0;
                    result_delay = 0.0;
                    xp_banked = false;
                    screen = Screen::Play;
                    screen_time = 0.0;
                }
            }
            Screen::Play | Screen::Result => {
                let w = world.as_mut().expect("a map is running");
                if matches!(screen, Screen::Play) {
                    if is_key_pressed(KeyCode::Escape) {
                        // Leaving early keeps the experience (light death penalty, design §9).
                        if !xp_banked {
                            planner.gain_xp(w.xp);
                            fusions = planner.receive_loot(&w.loot());
                            xp_banked = true;
                        }
                        screen = Screen::Plan;
                        next_frame().await;
                        continue;
                    }
                    if is_key_pressed(KeyCode::Tab) {
                        show_stats = !show_stats;
                    }
                    if is_key_pressed(KeyCode::F2) {
                        use_autopilot = !use_autopilot;
                    }
                    let mut input = read_input();
                    // A tap can land between simulation steps; keep it until one runs.
                    dash_queued |= input.dash;
                    accumulator += frame;
                    while accumulator >= DT {
                        accumulator -= DT;
                        if use_autopilot {
                            input = autopilot(w);
                        }
                        input.dash = input.dash || dash_queued;
                        dash_queued = false;
                        w.step(&input);
                        fx.consume(w);
                        w.events.clear();
                    }
                    if matches!(w.phase, Phase::Victory | Phase::Defeat) {
                        result_delay += frame;
                        if result_delay > 1.2 {
                            screen = Screen::Result;
                            if !xp_banked {
                                planner.gain_xp(w.xp);
                                fusions = planner.receive_loot(&w.loot());
                                if w.phase == Phase::Victory {
                                    planner.record_victory(w.tier);
                                }
                                xp_banked = true;
                            }
                        }
                    }
                }
                fx.ambient(w, frame);
                fx.update(frame);
                let camera = render::fix_camera(fx.camera(w));
                render::draw_world(w, &fx, &camera);
                set_default_camera();
                let pending = if xp_banked { 0 } else { w.xp };
                ui::hud(&ui, w, &fx, show_stats, use_autopilot, planner.character, pending);
                if matches!(screen, Screen::Result) {
                    match ui::result_panel(&ui, w, &fusions) {
                        ResultAction::Replay => {
                            map_seed = map_seed.wrapping_add(1);
                            *w = planner.new_world(map_seed);
                            fx = Fx::new();
                            result_delay = 0.0;
                            xp_banked = false;
                            screen = Screen::Play;
                        }
                        ResultAction::Edit => screen = Screen::Plan,
                        ResultAction::None => {}
                    }
                }
            }
        }
        // Autosave whenever the character changes (level, tree, loadout, tier).
        if !debug_run {
            let text = planner.save_text();
            if text != saved_text {
                arpg_machine::save::CharacterSave::save_text(&text);
                saved_text = text;
            }
        }
        if let Some(path) = &shot_path {
            if screen_time >= shot_at {
                save_screenshot(path);
                break;
            }
        }
        next_frame().await;
    }
}
