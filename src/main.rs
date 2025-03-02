mod script;

use macroquad::prelude::*;
use script::Script;

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;

fn window_conf() -> Conf {
    Conf {
        window_title: "VN Engine".to_owned(),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        window_resizable: false,

        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    colog::default_builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let mut script = Script::load_script("./scripts/SCRIPT01.SCR");

    loop {
        /*
        draw_texture_ex(&texture, 0., 0., WHITE, DrawTextureParams {
            dest_size: Some(Vec2 { x: WINDOW_WIDTH as f32, y: WINDOW_HEIGHT as f32 }),
            ..Default::default()
        });
        */

        if is_key_pressed(KeyCode::Space) {
            script.next_instruction();
        }


        next_frame().await
    }
}

/*
async fn load_image() -> Option<Texture2D> {
    let image = image::load_from_memory(TEXTURE_1)
        .unwrap()
        .to_rgba8();

    Some(Texture2D::from_rgba8(
        image.width() as u16,
        image.height() as u16,
        &image.into_raw(),
    ))
}
*/
