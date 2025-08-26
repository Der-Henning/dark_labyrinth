use macroquad::prelude::*;
use macroquad::ui::widgets::{Checkbox, ComboBox};
use macroquad::ui::{Skin, hash, root_ui};

use crate::game::Game;
use crate::{GRID_SIZES, GameState, Settings, WINDOW_DIMENSIONS};

pub enum Menus {
    Main,
    Options,
    Pause,
    GameOver,
}

impl Menus {
    pub fn display(
        self,
        game: &mut Game,
        game_state: &mut GameState,
        settings: &mut Settings,
        display_options_menu: &mut bool,
    ) {
        match self {
            Menus::Main => {
                let window_size = vec2(370., 420.);
                root_ui().window(
                    hash!(),
                    (WINDOW_DIMENSIONS - window_size) * 0.5,
                    window_size,
                    |ui| {
                        ui.label(vec2(80.0, -34.0), "Main Menu");

                        if ui.button(vec2(65., 25.), "Play") {
                            *game = Game::new(
                                GRID_SIZES[settings.labyrinth_size],
                                settings.dropout,
                                settings.target_threshold,
                            );
                            game.timer.start();
                            *game_state = GameState::Playing;
                        }

                        if ui.button(vec2(20., 125.), "Options") {
                            *display_options_menu = true;
                        }

                        if ui.button(vec2(65.0, 225.0), "Quit") {
                            std::process::exit(0);
                        }
                    },
                );
            }
            Menus::Options => {
                let window_size = vec2(420., 375.);
                root_ui().window(
                    hash!(),
                    (WINDOW_DIMENSIONS - window_size) * 0.5,
                    window_size,
                    |ui| {
                        ui.label(vec2(80.0, -34.0), "Options Menu");

                        ComboBox::new(hash!(), &["small", "medium", "large"])
                            .label("Labyrinth Size")
                            .ui(ui, &mut settings.labyrinth_size);

                        Checkbox::new(hash!())
                            .pos(vec2(-110., 25.0))
                            .label("Display Labyrinth")
                            .ui(ui, &mut settings.draw_labyrinth);

                        Checkbox::new(hash!())
                            .pos(vec2(-110., 50.0))
                            .label("Display dt")
                            .ui(ui, &mut settings.draw_delta_time);

                        if ui.button(vec2(65., 175.), "Back") {
                            *display_options_menu = false;
                        }
                    },
                );
            }
            Menus::Pause => {
                let window_size = vec2(400., 420.);
                root_ui().window(
                    hash!(),
                    (WINDOW_DIMENSIONS - window_size) * 0.5,
                    window_size,
                    |ui| {
                        ui.label(vec2(80., -34.), "Pause Menu");

                        if ui.button(vec2(25., 25.), "Continue") {
                            game.timer.resume();
                            *game_state = GameState::Playing;
                        }

                        if ui.button(vec2(25., 125.), "New Game") {
                            *game = Game::new(
                                GRID_SIZES[settings.labyrinth_size],
                                settings.dropout,
                                settings.target_threshold,
                            );
                            game.timer.start();
                            *game_state = GameState::Playing;
                        }

                        if ui.button(vec2(5., 225.), "Quit Game") {
                            *game_state = GameState::MainMenu;
                            game.timer.stop();
                        }
                    },
                );
            }
            Menus::GameOver => {
                let window_size = vec2(400., 370.);
                root_ui().window(
                    hash!(),
                    (WINDOW_DIMENSIONS - window_size) * 0.5,
                    window_size,
                    |ui| {
                        ui.label(vec2(80., -34.), "Main Menu");

                        ui.label(
                            vec2(25., 25.),
                            format!("You Won! {:.2?}s", game.timer.result.unwrap()).as_str(),
                        );

                        if ui.button(vec2(25., 75.), "New Game") {
                            *game_state = GameState::Playing;
                            *game = Game::new(
                                GRID_SIZES[settings.labyrinth_size],
                                settings.dropout,
                                settings.target_threshold,
                            );
                            game.timer.start();
                        }

                        if ui.button(vec2(10., 175.), "Quit Game") {
                            *game_state = GameState::MainMenu;
                        }
                    },
                );
            }
        }
    }
}

pub async fn make_skin() -> Skin {
    let window_background = load_image("assets/window_background.png").await.unwrap();
    let button_background = load_image("assets/button_background.png").await.unwrap();
    let button_clicked_background = load_image("assets/button_clicked_background.png")
        .await
        .unwrap();
    let checkbox_background = load_image("assets/checkbox_background.png").await.unwrap();
    // let checkbox_background_selected = load_image("assets/checkbox_background_selected.png")
    //     .await
    //     .unwrap();
    let checkbox_clicked_background = load_image("assets/checkbox_clicked_background.png")
        .await
        .unwrap();
    let checkbox_hovered_background = load_image("assets/checkbox_hovered_background.png")
        .await
        .unwrap();
    let combobox_background = load_image("assets/combobox_background.png").await.unwrap();
    let font = load_file("assets/atari_games.ttf").await.unwrap();

    let window_style = root_ui()
        .style_builder()
        .background(window_background)
        .background_margin(RectOffset::new(32.0, 76.0, 44.0, 20.0))
        .margin(RectOffset::new(0.0, -40.0, 0.0, 0.0))
        .build();

    let button_style = root_ui()
        .style_builder()
        .background(button_background)
        .background_clicked(button_clicked_background)
        .background_margin(RectOffset::new(16.0, 16.0, 16.0, 16.0))
        .margin(RectOffset::new(16.0, 0.0, -8.0, -8.0))
        .font(&font)
        .unwrap()
        .text_color(WHITE)
        .font_size(64)
        .build();

    let label_style = root_ui()
        .style_builder()
        .font(&font)
        .unwrap()
        .text_color(WHITE)
        .font_size(28)
        .build();

    let checkbox_style = root_ui()
        .style_builder()
        .background(checkbox_background)
        .background_hovered(checkbox_hovered_background)
        .background_clicked(checkbox_clicked_background)
        .font(&font)
        .unwrap()
        .text_color(WHITE)
        .font_size(28)
        .build();

    let combobox_style = root_ui()
        .style_builder()
        .background(combobox_background)
        .background_margin(RectOffset::new(4., 25., 6., 6.))
        .font(&font)
        .unwrap()
        .text_color(WHITE)
        .color(Color::from_rgba(210, 210, 210, 255))
        .font_size(28)
        .build();

    Skin {
        window_style,
        button_style,
        label_style,
        checkbox_style,
        combobox_style,
        ..root_ui().default_skin()
    }
}
