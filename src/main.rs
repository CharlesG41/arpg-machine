//! ARPG Machine prototype window.
//!
//! Debug environment variables (used for automated screenshots):
//! - `ARPG_PRESET=<name>`: skip the build screen and start with that preset.
//! - `ARPG_AUTOPILOT=1`: let the autopilot move.
//! - `ARPG_SHOT=<path.png>` and `ARPG_SHOT_AT=<seconds>`: save a screenshot
//!   after that much time on the current screen, then quit.
mod render;
mod ui;

use arpg_machine::content::presets;
use arpg_machine::math::V2;
use arpg_machine::world::{autopilot, Input, Phase, World, DT};
use macroquad::prelude::*;
use render::Fx;
use ui::{BuildAction, BuildState, ResultAction, Ui};

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
    Build,
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
    let mut state = BuildState { build: presets()[1].1.clone(), selected: None, tier: 0, seed: 1 };
    let mut screen = Screen::Build;
    let mut world: Option<World> = None;
    let mut fx = Fx::new();
    let mut accumulator = 0.0f32;
    let mut show_stats = true;
    let mut use_autopilot = env("ARPG_AUTOPILOT").is_some();
    let mut dash_queued = false;
    let mut result_delay = 0.0f32;
    let shot_path = env("ARPG_SHOT");
    let shot_at: f32 = env("ARPG_SHOT_AT").and_then(|s| s.parse().ok()).unwrap_or(1.0);
    let mut screen_time = 0.0f32;

    if let Some(name) = env("ARPG_PRESET") {
        if let Some((_, build)) = presets().into_iter().find(|(n, _)| n.eq_ignore_ascii_case(&name)) {
            state.build = build;
            world = Some(World::new(state.build.clone(), state.seed, state.tier));
            screen = Screen::Play;
        }
    }

    loop {
        let frame = get_frame_time().min(0.1);
        screen_time += frame;
        clear_background(render::BACKGROUND);
        match screen {
            Screen::Build => {
                if let BuildAction::Play = ui::build_screen(&ui, &mut state) {
                    world = Some(World::new(state.build.clone(), state.seed, state.tier));
                    fx = Fx::new();
                    accumulator = 0.0;
                    result_delay = 0.0;
                    screen = Screen::Play;
                    screen_time = 0.0;
                }
            }
            Screen::Play | Screen::Result => {
                let w = world.as_mut().expect("a map is running");
                if matches!(screen, Screen::Play) {
                    if is_key_pressed(KeyCode::Escape) {
                        screen = Screen::Build;
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
                        }
                    }
                }
                fx.ambient(w, frame);
                fx.update(frame);
                let camera = render::fix_camera(fx.camera(w));
                render::draw_world(w, &fx, &camera);
                set_default_camera();
                ui::hud(&ui, w, &fx, show_stats, use_autopilot);
                if matches!(screen, Screen::Result) {
                    match ui::result_panel(&ui, w) {
                        ResultAction::Replay => {
                            state.seed = state.seed.wrapping_add(1);
                            *w = World::new(state.build.clone(), state.seed, state.tier);
                            fx = Fx::new();
                            result_delay = 0.0;
                            screen = Screen::Play;
                        }
                        ResultAction::Edit => screen = Screen::Build,
                        ResultAction::None => {}
                    }
                }
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
