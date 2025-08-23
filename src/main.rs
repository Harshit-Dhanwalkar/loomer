use image::{ImageBuffer, Rgba};
use libwayshot::{CaptureRegion, WayshotConnection};
use raylib::prelude::*;
use std::{env, fs::File, io::Write, process};
// use raylib::{
//     ffi::{Image as FfiImage, SetWindowMonitor, ToggleFullscreen},
//     prelude::*,
// };

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

    let screenshot_buffer = wayshot_connection
        .screenshot(capture_region, false)
        .expect("failed to take a screenshot");

    let width = screenshot_buffer.width();
    let height = screenshot_buffer.height();

    // let rgba_data = screenshot_buffer.as_raw();
    let rgba_data = screenshot_buffer.into_raw();

    let temp_path = "/tmp/screenshot.raw";
    let mut file = File::create(&temp_path).expect("Failed to create temp file");
    file.write_all(&rgba_data).expect("Failed to write data");

    // let screenshot_image = unsafe {
    //     let colors: Vec<Color> = rgba_data
    //         .chunks_exact(4)
    //         .map(|chunk| Color::new(chunk[0], chunk[1], chunk[2], chunk[3]))
    //         .collect();
    //
    //     // Leak the vector to prevent it from being freed
    //     let colors_box = colors.into_boxed_slice();
    //     let colors_ptr = Box::into_raw(colors_box);
    //
    //     Image::from_raw(
    //         colors_ptr as *mut raylib::ffi::Color,
    //         width as i32,
    //         height as i32,
    //     )
    // };
    let img = ImageBuffer::<Rgba<u8>, _>::from_raw(width as u32, height as u32, rgba_data)
        .expect("Failed to create image buffer");

    // Save as PNG
    img.save("/tmp/screenshot.png").expect("Failed to save PNG");

    // Load with raylib
    let screenshot_image =
        Image::load_image("/tmp/screenshot.png").expect("Failed to load screenshot");

    let (mut rl, thread) = raylib::init()
        .title(env!("CARGO_BIN_NAME"))
        .size(width as i32, height as i32)
        .transparent()
        .undecorated()
        .vsync()
        .build();

    let screenshot_texture = rl
        .load_texture_from_image(&thread, &screenshot_image)
        .expect("failed to load screenshot into a texture");

    std::fs::remove_file(temp_path).ok();

    let mut rl_camera = Camera2D::default();
    rl_camera.zoom = 1.0;
    rl_camera.target = Vector2::new(0.0, 0.0); // Start at top-left corner

    let mut delta_scale = 0f64;
    let mut scale_pivot = rl.get_mouse_position();
    let mut velocity = Vector2::default();
    let mut spotlight_radius_multiplier = 1.0;
    let mut spotlight_radius_multiplier_delta = 0.0;

    let mut should_exit = false;
    while !rl.window_should_close() && !should_exit {
        if rl.is_key_pressed(KeyboardKey::KEY_Q) {
            should_exit = true;
        }
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
            break;
        }

        let enable_spotlight = rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
            || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL);

        let scrolled_amount = rl.get_mouse_wheel_move_v().y;

        if rl.is_key_pressed(KeyboardKey::KEY_LEFT_CONTROL)
            || rl.is_key_pressed(KeyboardKey::KEY_RIGHT_CONTROL)
        {
            spotlight_radius_multiplier = 5.0;
            spotlight_radius_multiplier_delta = -15.0;
        }

        if scrolled_amount != 0.0 {
            match (
                enable_spotlight,
                rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                    || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT),
            ) {
                (_, false) => {
                    delta_scale += scrolled_amount as f64;
                }
                (true, true) => {
                    spotlight_radius_multiplier_delta -= scrolled_amount as f64;
                }
                _ => {}
            }
            scale_pivot = rl.get_mouse_position();
        }

        if delta_scale.abs() > 0.5 {
            let p0 = scale_pivot / rl_camera.zoom;
            rl_camera.zoom = (rl_camera.zoom as f64 + delta_scale * rl.get_frame_time() as f64)
                .clamp(0.1, 10.0) as f32;
            let p1 = scale_pivot / rl_camera.zoom;
            rl_camera.target += p0 - p1;
            delta_scale -= delta_scale * rl.get_frame_time() as f64 * 4.0
        }

        spotlight_radius_multiplier = (spotlight_radius_multiplier as f64
            + spotlight_radius_multiplier_delta * rl.get_frame_time() as f64)
            .clamp(0.3, 10.0) as f32;

        spotlight_radius_multiplier_delta -=
            spotlight_radius_multiplier_delta * rl.get_frame_time() as f64 * 4.0;

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

        let mut d = rl.begin_drawing(&thread);
        let mut mode2d = d.begin_mode2D(rl_camera);

        if enable_spotlight {
            mode2d.clear_background(SPOTLIGHT_TINT);
            mode2d.draw_texture(&screenshot_texture, 0, 0, Color::WHITE);
        } else {
            mode2d.clear_background(Color::BLACK);
            mode2d.draw_texture(&screenshot_texture, 0, 0, Color::WHITE);
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
