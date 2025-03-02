mod script;

use std::{cell::LazyCell, path::Path, time::Instant};

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

const MONASPACE_KRYPTON: &[u8] = include_bytes!("../assets/Monaspace-Krypton_Medium.ttf");

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

#[derive(PartialEq, Default)]
struct NovelState {
    background: Option<Texture2D>,
    switch_delay: Option<Instant>,
    script_delay: Option<Instant>,

    textbox: Option<TextBox>,
}

#[derive(PartialEq, Default)]
struct TextBox {
    dialogue: Vec<String>,
    name: String,
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

    let monaspace_krypton = load_ttf_font_from_bytes(MONASPACE_KRYPTON).unwrap();
    let monaspace_krypton_params = TextParams {
        font: Some(&monaspace_krypton),
        font_size: 26,
        font_scale: 0.7,
        ..Default::default()
    };

    loop {
        if is_key_pressed(KeyCode::Space) {
            script.next_instruction(&mut state);
        }

        if let Some(b) = &state.background {
            draw_texture_ex(b, 0., 0., Color::from_rgba(255, 255, 255, 255), DrawTextureParams {
                dest_size: Some(Vec2 { x: WINDOW_WIDTH as f32 / scale, y: WINDOW_HEIGHT as f32 / scale }),
                ..Default::default()
            });
        }

        if let Some(textbox) = &state.textbox {
            draw_texture_ex(&TEXTBOX, 149.572 / scale, 486.565 / scale, Color::from_rgba(255, 255, 255, 255),
                DrawTextureParams {
                    dest_size: Some(Vec2 { x: TEXTBOX.width() as f32 / scale, y: TEXTBOX.height() as f32 / scale }),
                    ..Default::default()
                }
            );

            let mut shift = 0.0;
            for line in textbox.dialogue.iter() {
                draw_text_ex(&line, 205.420 / scale, (552.270 + shift) / scale,
                    monaspace_krypton_params.clone()
                );

                shift += 47.;
            }

            draw_texture_ex(&CHARBOX, 165.384 / scale, 412.606 / scale, Color::from_rgba(255, 255, 255, 255),
                DrawTextureParams {
                    dest_size: Some(Vec2 { x: CHARBOX.width() as f32 / scale, y: CHARBOX.height() as f32 / scale }),
                    ..Default::default()
                }
            );
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
