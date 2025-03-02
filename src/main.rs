mod script;

use std::{cell::LazyCell, path::Path};

use macroquad::prelude::*;
use script::Script;

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;

const TEXTBOX: LazyCell<Texture2D> = LazyCell::new(|| {
    let image = image::load_from_memory(include_bytes!("../assets/textbox.png"))
        .unwrap()
        .to_rgba8();

    Texture2D::from_rgba8(
        image.width() as u16,
        image.height() as u16,
        &image.into_raw(),
    )
});

const CHARBOX: LazyCell<Texture2D> = LazyCell::new(|| {
    let image = image::load_from_memory(include_bytes!("../assets/characterbox.png"))
        .unwrap()
        .to_rgba8();

    Texture2D::from_rgba8(
        image.width() as u16,
        image.height() as u16,
        &image.into_raw(),
    )
});

const SELECTION_BOX: LazyCell<Texture2D> = LazyCell::new(|| {
    let image = image::load_from_memory(include_bytes!("../assets/selectionbox.png"))
        .unwrap()
        .to_rgba8();

    Texture2D::from_rgba8(
        image.width() as u16,
        image.height() as u16,
        &image.into_raw(),
    )
});

const SELECTION_BOX_ACTIVE: LazyCell<Texture2D> = LazyCell::new(|| {
    let image = image::load_from_memory(include_bytes!("../assets/selectionbox-active.png"))
        .unwrap()
        .to_rgba8();

    Texture2D::from_rgba8(
        image.width() as u16,
        image.height() as u16,
        &image.into_raw(),
    )
});

const CHARACTER_TEXT: &[u8] = include_bytes!("../assets/Monaspace-Krypton_Medium.ttf");
const CHARACTER_NAME: &[u8] = include_bytes!("../assets/Monaspace-Krypton_Bold.ttf");

fn window_conf() -> Conf {
    Conf {
        window_title: "VN Engine".to_owned(),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        window_resizable: false,
        high_dpi: true,

        ..Default::default()
    }
}

#[derive(Default)]
struct NovelState {
    background: Option<Texture2D>,

    characters: Vec<CharacterSprite>,

    textbox: Option<TextBox>,

    select_menu: Option<SelectMenu>,
}

#[derive(Default)]
struct TextBox {
    dialogue: Vec<String>,
    name: String,
    current_line: usize,
    current_char: usize,
}

struct CharacterSprite {
    texture: Texture2D,
    position: Vec2,
    saturation: f32,
    flip: bool,
}

struct SelectMenu {
    variable: String,
    selected: usize,
    options: Vec<(String, String)>,
}

#[macroquad::main(window_conf)]
async fn main() {
    colog::default_builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let mut script = Script::load_script("./scripts/SCRIPT01.SCR");

    // Set the state
    let mut state = NovelState::default();

    // Step through the script until the first non-automatic instruction
    script.next_instruction(&mut state);

    let scale = screen_dpi_scale();

    let character_text = load_ttf_font_from_bytes(CHARACTER_TEXT).unwrap();
    let character_text = TextParams {
        font: Some(&character_text),
        font_size: 26,
        font_scale: 0.9,
        font_scale_aspect: 0.9,
        ..Default::default()
    };

    let character_name = load_ttf_font_from_bytes(CHARACTER_NAME).unwrap();
    let character_name = TextParams {
        font: Some(&character_name),
        font_size: 26,
        font_scale: 0.8,
        font_scale_aspect: 0.9,
        ..Default::default()
    };

    loop {
        if let Some(select) = state.select_menu.as_mut() {
            if is_key_pressed(KeyCode::Up) {
                if select.selected == 0 {
                    select.selected = select.options.len() - 1;
                } else {
                    select.selected -= 1;
                }
            }

            if is_key_pressed(KeyCode::Down) {
                if select.selected == select.options.len() - 1 {
                    select.selected = 0;
                } else {
                    select.selected += 1;
                }
            }

            if is_key_pressed(KeyCode::Enter) {
                script.insert_variable(
                    select.variable.clone(),
                    select.options.get(select.selected).unwrap().1.clone()
                );

                state.select_menu = None;
                script.next_instruction(&mut state);
            }
        } else if is_key_pressed(KeyCode::Enter) {
            // Non-option control mode... very simple!
            script.next_instruction(&mut state);
        }

        // Draw background
        if let Some(b) = &state.background {
            draw_texture_ex(b, 0., 0., WHITE, DrawTextureParams {
                dest_size: Some(Vec2 { x: WINDOW_WIDTH as f32 / scale, y: WINDOW_HEIGHT as f32 / scale }),
                ..Default::default()
            });
        }

        // Draw characters
        for character in &state.characters {
            draw_texture_ex(
                &character.texture,
                character.position.x / scale,
                character.position.y / scale,
                Color::from_rgba((255.0 * character.saturation) as u8, (255.0 * character.saturation) as u8, (255.0 * character.saturation) as u8, 255),
                DrawTextureParams {
                    dest_size: Some(Vec2::new(
                        character.texture.width() as f32 / scale,
                        character.texture.height() as f32 / scale
                    )),
                    flip_x: character.flip,
                    ..Default::default()
                }
            );
        }

        if let Some(o) = &state.select_menu {
            let mut vert = 0.0;
            for (i, option) in o.options.iter().enumerate() {
                if o.selected == i {
                    draw_texture_ex(&SELECTION_BOX_ACTIVE, 171. / scale, (100. + vert) / scale, WHITE,
                        DrawTextureParams {
                            dest_size: Some(Vec2 { x: SELECTION_BOX_ACTIVE.width() as f32 / scale, y: SELECTION_BOX_ACTIVE.height() as f32 / scale }),
                            ..Default::default()
                        }
                    );
                } else {
                    draw_texture_ex(&SELECTION_BOX, 171. / scale, (100. + vert) / scale, WHITE,
                        DrawTextureParams {
                            dest_size: Some(Vec2 { x: SELECTION_BOX.width() as f32 / scale, y: SELECTION_BOX.height() as f32 / scale }),
                            ..Default::default()
                        }
                    );
                }

                let mut text = character_text.clone();
                text.color = BLACK;

                draw_text_ex(&option.0, 275. / scale, (172. + vert) / scale, text);

                vert += 163.;
            }
        }

        // Draw textbox and stuff
        if let Some(textbox) = state.textbox.as_mut() {
            draw_texture_ex(&TEXTBOX, 150. / scale, 487. / scale, WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2 { x: TEXTBOX.width() as f32 / scale, y: TEXTBOX.height() as f32 / scale }),
                    ..Default::default()
                }
            );

            if textbox.name != "Narrator" {
                draw_texture_ex(&CHARBOX, 165. / scale, 413. / scale, WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2 { x: CHARBOX.width() as f32 / scale, y: CHARBOX.height() as f32 / scale }),
                        ..Default::default()
                    }
                );

                draw_text_ex(&textbox.name, 190. / scale, 459. / scale,
                    character_name.clone()
                );
            }

            let mut shift = 0.0;
            for line in textbox.dialogue.iter().enumerate() {
                let pos = if line.0 == textbox.current_line && line.1.len() > textbox.current_char / 3 {
                    textbox.current_char / 3
                } else {
                    line.1.len()
                };

                if line.0 <= textbox.current_line {
                    draw_text_ex(&line.1[..pos], 205. / scale, (552. + shift) / scale,
                        character_text.clone()
                    );
                }

                if textbox.dialogue.len() >= textbox.current_line {
                    if (textbox.current_char / 3) >= 38 {
                        textbox.current_char = 0;
                        textbox.current_line += 1;
                    }
                    textbox.current_char += 1;
                }
                shift += 56.;
            }
        }

        next_frame().await
    }
}

fn load_image<P: AsRef<Path>>(path: P) -> Option<Texture2D> {
    let image = image::open(path)
        .unwrap()
        .to_rgba8();

    Some(Texture2D::from_rgba8(
        image.width() as u16,
        image.height() as u16,
        &image.into_raw(),
    ))
}
