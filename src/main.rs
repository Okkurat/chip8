extern crate raylib;

use raylib::prelude::*;
use std::process;
use std::env;
use std::time::{Duration, Instant};

const WIDTH: i32 = 64;
const HEIGHT: i32 = 32;
const SCALE: i32 = 10;

mod cpu;
use cpu::CPU;

fn main() -> std::io::Result<()> {
    let mut cpu = CPU::new();
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <game_path>", args[0]);
        process::exit(1);
    }
    let game_path = &args[1];
    cpu.load_game(game_path)?;
    let (mut rl, thread) = raylib::init()
        .size(WIDTH * SCALE, HEIGHT * SCALE)
        .title("Chip-8 Emulator")
        .build();

    let target_cycle_time = Duration::from_secs_f64(1.0 / 700.0);
    let mut last_cycle_time = Instant::now();
    let mut last_timer_update = Instant::now();

    let key_mapping: Vec<(u8, raylib::consts::KeyboardKey)> = vec![
        (0x0, raylib::consts::KeyboardKey::KEY_X),  // 0
        (0x1, raylib::consts::KeyboardKey::KEY_ONE),  // 1
        (0x2, raylib::consts::KeyboardKey::KEY_TWO),  // 2
        (0x3, raylib::consts::KeyboardKey::KEY_THREE),  // 3
        (0x4, raylib::consts::KeyboardKey::KEY_Q),  // 4
        (0x5, raylib::consts::KeyboardKey::KEY_W ),  // 5
        (0x6, raylib::consts::KeyboardKey::KEY_E),  // 6
        (0x7, raylib::consts::KeyboardKey::KEY_A),  // 7
        (0x8, raylib::consts::KeyboardKey::KEY_S),  // 8
        (0x9, raylib::consts::KeyboardKey::KEY_D), // 9
        (0xA, raylib::consts::KeyboardKey::KEY_Z), // A
        (0xB, raylib::consts::KeyboardKey::KEY_C), // B
        (0xC, raylib::consts::KeyboardKey::KEY_FOUR), // C
        (0xD, raylib::consts::KeyboardKey::KEY_R),  // D
        (0xE, raylib::consts::KeyboardKey::KEY_F), // E
        (0xF, raylib::consts::KeyboardKey::KEY_V), // F
    ];

    while !rl.window_should_close() {
        let current_time = Instant::now();
        
        if current_time.duration_since(last_cycle_time) >= target_cycle_time {
            cpu.execute_cycle();
            last_cycle_time = current_time;
        }
        
        if current_time.duration_since(last_timer_update) >= Duration::from_secs_f64(1.0 / 60.0) {
            cpu.decrement_timers();
            last_timer_update = current_time;
        }
        // TODO
        if cpu.sound_timer > 0 {
            println!("BEEP");
            // play resources_beep.wav
        } else {
            // dont play it
        }

        for (chip8_key, raylib_key) in &key_mapping {
            cpu.keys[*chip8_key as usize] = rl.is_key_down(*raylib_key);
        }
        for (chip8_key, raylib_key) in &key_mapping {
            if rl.is_key_released(*raylib_key) {
                cpu.key_pressed = (true, *chip8_key);
            }
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        for y in 0..HEIGHT as usize {
            for x in 0..WIDTH as usize {
                if cpu.screen_data[y][x] {
                    d.draw_rectangle(
                        x as i32 * SCALE,
                        y as i32 * SCALE,
                        SCALE,
                        SCALE,
                        Color::WHITE,
                    );
                }
            }
        }
    }

    Ok(())
}