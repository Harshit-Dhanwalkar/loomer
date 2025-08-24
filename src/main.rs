use libwayshot::{CaptureRegion, WayshotConnection};
use raylib::prelude::*;
use std::{env, process};

const SPOTLIGHT_TINT: Color = Color::new(0x00, 0x00, 0x00, 190);

fn main() {
    let mut args = env::args();
    let bin = args.next().unwrap();

    let mut monitor_name: Option<String> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--monitor" => {
                monitor_name = args.next().or_else(|| {
                    eprintln!("--monitor needs a value");
                    process::exit(1);
                })
            }
            _other => print_help_and_exit(&bin),
        }
    }

    let wayshot_connection = WayshotConnection::new().expect("Failed to connect to wayshot");
    let outputs = wayshot_connection.get_all_outputs();

    if outputs.is_empty() {
        eprintln!("No Wayland outputs found.");
        process::exit(1);
    }

    let selected_output = match monitor_name {
        None => &outputs[0],
        Some(ref name) => outputs
            .iter()
            .find(|out| &out.name == name)
            .unwrap_or_else(|| {
                eprintln!("Output '{}' not found.", name);
                process::exit(1);
            }),
    };

    let geometry = &selected_output.dimensions;
    let capture_region = CaptureRegion {
        x_coordinate: geometry.x,
        y_coordinate: geometry.y,
        width: geometry.width,
        height: geometry.height,
    };

    let width = geometry.width;
    let height = geometry.height;

    let (mut rl, thread) = raylib::init()
        .title(env!("CARGO_BIN_NAME"))
        .size(width, height)
        .transparent()
        .undecorated()
        .vsync()
        .build();

    let mut magnifier_shader = rl.load_shader(&thread, None, Some("shader/magnifier.fs"));

    let center_loc = magnifier_shader.get_shader_location("center");
    let radius_loc = magnifier_shader.get_shader_location("radius");
    let texture_width_loc = magnifier_shader.get_shader_location("textureWidth");
    let texture_height_loc = magnifier_shader.get_shader_location("textureHeight");
    let col_diffuse_loc = magnifier_shader.get_shader_location("colDiffuse");
    let magnification_loc = magnifier_shader.get_shader_location("magnification");

    let initial_image = Image::gen_image_color(width, height, Color::BLACK);
    let mut screenshot_texture = rl
        .load_texture_from_image(&thread, &initial_image)
        .expect("Failed to create initial texture");

    let mut rl_camera = Camera2D::default();
    rl_camera.zoom = 1.0;
    rl_camera.target = Vector2::new(0.0, 0.0);

    let mut delta_scale = 0f64;
    let mut scale_pivot = rl.get_mouse_position();
    let mut velocity = Vector2::default();
    let mut magnification = 2.0;
    let mut radius = 100.0;
    let mut should_exit = false;

    while !rl.window_should_close() && !should_exit {
        let screenshot_buffer = wayshot_connection
            .screenshot(capture_region, false)
            .expect("failed to take a screenshot");
        let rgba_data = screenshot_buffer.into_raw();

        screenshot_texture.update_texture(&rgba_data);

        if rl.is_key_pressed(KeyboardKey::KEY_Q) {
            should_exit = true;
        }
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
            break;
        }

        let mouse_pos = rl.get_mouse_position();
        let scrolled_amount = rl.get_mouse_wheel_move_v().y;

        if scrolled_amount != 0.0 {
            if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT)
            {
                magnification = (magnification + scrolled_amount as f32 * 0.1).clamp(1.0, 5.0);
            } else if rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
                || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL)
            {
                radius = (radius + scrolled_amount as f32 * 10.0).clamp(20.0, 300.0);
            } else {
                delta_scale += scrolled_amount as f64;
                scale_pivot = mouse_pos;
            }
        }

        if delta_scale.abs() > 0.5 {
            let p0 = scale_pivot / rl_camera.zoom;
            rl_camera.zoom = (rl_camera.zoom as f64 + delta_scale * rl.get_frame_time() as f64)
                .clamp(0.1, 10.0) as f32;
            let p1 = scale_pivot / rl_camera.zoom;
            rl_camera.target += p0 - p1;
            delta_scale -= delta_scale * rl.get_frame_time() as f64 * 4.0
        }

        const VELOCITY_THRESHOLD: f32 = 15.0;
        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            let delta = rl
                .get_screen_to_world2D(rl.get_mouse_position() - rl.get_mouse_delta(), rl_camera)
                - rl.get_screen_to_world2D(rl.get_mouse_position(), rl_camera);
            rl_camera.target += delta;
            velocity = delta * rl.get_fps().as_f32();
        } else if velocity.length_sqr() > VELOCITY_THRESHOLD * VELOCITY_THRESHOLD {
            rl_camera.target += velocity * rl.get_frame_time();
            velocity -= velocity * rl.get_frame_time() * 6.0;
        }

        let world_mouse_pos = rl.get_screen_to_world2D(mouse_pos, rl_camera);
        magnifier_shader.set_shader_value(center_loc, [world_mouse_pos.x, world_mouse_pos.y]);
        magnifier_shader.set_shader_value(radius_loc, radius);
        magnifier_shader.set_shader_value(texture_width_loc, width as f32);
        magnifier_shader.set_shader_value(texture_height_loc, height as f32);
        magnifier_shader.set_shader_value(col_diffuse_loc, [1.0, 1.0, 1.0, 1.0]);
        magnifier_shader.set_shader_value(magnification_loc, magnification);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(SPOTLIGHT_TINT);

        let mut mode2d = d.begin_mode2D(rl_camera);

        {
            let mut _shader_mode = mode2d.begin_shader_mode(&mut magnifier_shader);
            _shader_mode.draw_texture_pro(
                &screenshot_texture,
                Rectangle::new(0.0, 0.0, width as f32, height as f32),
                Rectangle::new(0.0, 0.0, width as f32, height as f32),
                Vector2::new(0.0, 0.0),
                0.0,
                Color::WHITE,
            );
        }
    }
}

fn print_help_and_exit(bin: &str) -> ! {
    eprintln!(
        "\
    {bin}  – Wayland screen-zoom tool

    USAGE:
        {bin} [--monitor <name>]

    OPTIONS:
        --monitor <name>   Target monitor (Wayland output name); defaults to primary if flag is not provided.",
        bin = bin
    );
    process::exit(0);
}
