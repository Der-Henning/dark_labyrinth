use macroquad::prelude::*;
use macroquad::ui::root_ui;
use std::collections::VecDeque;

mod game;
mod geometrie;
mod menu;

use game::{Game, GameSettings};
use geometrie::{Line, Point};
use menu::{Menus, make_skin};

const WINDOW_DIMENSIONS: (usize, usize) = (800, 1200);
const GRID_SIZES: [usize; 3] = [100, 50, 25];
const SEED: Option<u64> = None;
const RAYS: usize = 360;
const RAY_LENGTH: usize = 4;
const TARGET_THRESHOLD: usize = 3;
const CIRCLE_SIZE: usize = 5;
const FONT_SIZE: u16 = 50;
const TEXT_COLOR: Color = WHITE;
const DROPOUT: f32 = 0.01;

fn window_conf() -> Conf {
    Conf {
        window_title: "Dark Labyrinth".to_owned(),
        fullscreen: false,
        high_dpi: true,
        window_height: WINDOW_DIMENSIONS.0 as i32,
        window_width: WINDOW_DIMENSIONS.1 as i32,
        window_resizable: false,
        platform: miniquad::conf::Platform {
            linux_backend: miniquad::conf::LinuxBackend::WaylandOnly,
            ..Default::default()
        },
        ..Default::default()
    }
}

enum GameState {
    MainMenu,
    Playing,
    Paused,
    Won,
}

#[macroquad::main(window_conf)]
async fn main() {
    let skin = make_skin().await;
    root_ui().push_skin(&skin);

    match SEED {
        Some(seed) => rand::srand(seed),
        _ => rand::srand(macroquad::miniquad::date::now() as u64),
    }

    let mut game_settings = GameSettings::new(false, false, 1);
    let mut game = Game::new(game_settings.clone());

    let mut game_state = GameState::MainMenu;

    let mut frame_durations = DeltaTime::new();

    let mut display_new_game_menu = true;
    let mut display_options_menu = false;

    loop {
        let calculation_time = macroquad::miniquad::date::now();
        clear_background(BLACK);

        match game_state {
            GameState::MainMenu => {
                draw_labyrinth(&game);
                if display_options_menu {
                    Menus::Options.display(
                        &mut game,
                        &mut game_state,
                        &mut game_settings,
                        &mut display_options_menu,
                    );
                } else {
                    Menus::Main.display(
                        &mut game,
                        &mut game_state,
                        &mut game_settings,
                        &mut display_options_menu,
                    );
                }
            }
            GameState::Playing => {
                if game.settings.draw_labyrinth {
                    draw_labyrinth(&game);
                }
                game.update_position();
                draw_player(&game);
                draw_time(&game);

                if game.found_target() {
                    game.timer.stop();
                    game_state = GameState::Won;
                    display_new_game_menu = true;
                }

                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::Paused;
                    game.timer.pause();
                }
            }
            GameState::Paused => {
                if game.settings.draw_labyrinth {
                    draw_labyrinth(&game);
                }
                draw_player(&game);
                draw_time(&game);
                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::Playing;
                    game.timer.resume();
                }
                Menus::Pause.display(
                    &mut game,
                    &mut game_state,
                    &mut game_settings,
                    &mut display_options_menu,
                );
            }
            GameState::Won => {
                if is_key_pressed(KeyCode::Escape) {
                    display_new_game_menu = !display_new_game_menu;
                }
                if display_new_game_menu {
                    Menus::GameOver.display(
                        &mut game,
                        &mut game_state,
                        &mut game_settings,
                        &mut display_options_menu,
                    );
                }
                draw_labyrinth(&game);
                game.update_position();
                draw_player(&game);
                draw_time(&game);
            }
        }

        frame_durations.push(macroquad::miniquad::date::now() - calculation_time);
        if game.settings.draw_delta_time {
            // draw_fps();
            let delta_time = frame_durations.delta_time().unwrap_or(0.0) * 1000.0;
            draw_text(
                format!("dt {:.3}ms", delta_time).as_str(),
                5.0,
                FONT_SIZE as f32 / 2.0,
                FONT_SIZE as f32 / 2.0,
                TEXT_COLOR,
            );
        };

        next_frame().await
    }
}

fn draw_player(game: &Game) {
    game.get_rays()
        .iter()
        .for_each(|ray| draw_line(game.position.x, game.position.y, ray.x, ray.y, 1.0, GREEN));
    draw_circle(
        game.target.x,
        game.target.y,
        (game.grid_size / CIRCLE_SIZE) as f32,
        RED,
    );
    draw_circle(
        game.position.x,
        game.position.y,
        (game.grid_size / CIRCLE_SIZE) as f32,
        WHITE,
    );
}

fn draw_labyrinth(game: &Game) {
    game.walls.iter().for_each(|line| {
        draw_line(
            line.a.x.max(1.0),
            line.a.y.max(1.0),
            line.b.x.max(1.0),
            line.b.y.max(1.0),
            1.0,
            BLUE,
        );
    });
}

fn draw_time(game: &Game) {
    let timer_text = format!("{:.2?}", game.timer.current());
    let text_center = get_text_center(&timer_text, None, FONT_SIZE / 2, 1., 0.);
    draw_text(
        &timer_text,
        WINDOW_DIMENSIONS.1 as f32 - text_center.x * 2. - 5.,
        FONT_SIZE as f32 / 2.,
        FONT_SIZE as f32 / 2.,
        TEXT_COLOR,
    );
}

struct DeltaTime(VecDeque<f64>);

impl DeltaTime {
    fn new() -> Self {
        Self(VecDeque::with_capacity(120))
    }
    fn push(&mut self, time: f64) {
        while self.0.len() >= 120 {
            self.0.pop_front();
        }
        self.0.push_back(time);
    }
    fn delta_time(&self) -> Option<f64> {
        match self.0.len() {
            0 => None,
            _ => Some(self.0.iter().sum::<f64>() / self.0.len() as f64),
        }
    }
}
