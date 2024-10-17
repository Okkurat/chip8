extern crate raylib;

use raylib::prelude::*;
use std::io::{self, Read};
use std::fs::File;
use std::time::{Duration, Instant};
use rand::Rng;

const WIDTH: i32 = 64;
const HEIGHT: i32 = 32;
const SCALE: i32 = 10;

#[derive(Debug)]
struct CPU {
    memory: [u8; 4096],
    pc: usize,
    i_reg: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    v_reg: [u8; 16],
    screen_data: [[bool; WIDTH as usize]; HEIGHT as usize],
    keys: [bool; 16],
    key_pressed: (bool, u8),
}

impl CPU {
    fn new() -> Self {
        let mut cpu = CPU {
            memory: [0; 4096],
            pc: 0x200,
            i_reg: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            v_reg: [0; 16],
            screen_data: [[false; WIDTH as usize]; HEIGHT as usize],
            keys: [false; 16],
            key_pressed: (false, 0x0),
        };
        cpu.load_fontset();
        cpu
    }

    fn load_fontset(&mut self) {
        const FONTSET: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80  // F
        ];
        self.memory[0..80].copy_from_slice(&FONTSET);
    }

    fn load_game(&mut self, file_name: &str) -> io::Result<()> {
        let mut file = File::open(file_name)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        self.memory[0x200..0x200 + buffer.len()].copy_from_slice(&buffer);
        Ok(())
    }

    fn decrement_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn execute_cycle(&mut self) {
        let opcode = self.fetch_opcode();
        self.execute_opcode(opcode);
    }

    fn fetch_opcode(&mut self) -> u16 {
        let hi = self.memory[self.pc] as u16;
        let lo = self.memory[self.pc + 1] as u16;
        self.pc += 2;
        (hi << 8) | lo
    }

    fn execute_opcode(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let nnn = opcode & 0x0FFF;
        let nn = (opcode & 0x00FF) as u8;
        let n = (opcode & 0x000F) as u8;

        match (opcode & 0xF000) >> 12 {
            0x0 => match opcode & 0x00FF {
                0xE0 => self.op_00e0(),
                0xEE => self.op_00ee(),
                _ => println!("Unknown opcode: {:04X}", opcode),
            },
            0x1 => self.op_1nnn(nnn),
            0x2 => self.op_2nnn(nnn),
            0x3 => self.op_3xnn(x, nn),
            0x4 => self.op_4xnn(x, nn),
            0x5 => self.op_5xy0(x, y),
            0x6 => self.op_6xnn(x as u16, nn as u16),
            0x7 => self.op_7xnn(x as u16, nn as u16),
            0x8 => match opcode & 0x000F {
                0x0 => self.op_8xy0(x, y),
                0x1 => self.op_8xy1(x, y),
                0x2 => self.op_8xy2(x, y),
                0x3 => self.op_8xy3(x, y),
                0x4 => self.op_8xy4(x, y),
                0x5 => self.op_8xy5(x, y),
                0x6 => self.op_8xy6(y, x),
                0x7 => self.op_8xy7(x, y),
                0xE => self.op_8xye(y, x),
                _ => println!("Unknown opcode: {:04X}", opcode),
            }
            0x9 => self.op_9xy0(x, y),
            0xA => self.op_annn(nnn),
            0xB => self.op_bnnn(x, nnn, nn),
            0xC => self.op_cxnn(x, nn),
            0xD => self.op_dxyn(x as usize, y as usize, n as u8),
            0xE => match opcode & 0x00FF {
                0x9e => self.op_ex9e(x),
                0xa1 => self.op_exa1(x),
                _ => println!("Unknown opcode: {:04X}", opcode),
            }
            0xF => match opcode & 0x00FF {
                0x07 => self.op_fx07(x),
                0x15 => self.op_fx15(x),
                0x18 => self.op_fx18(x),
                0x1E => self.op_fx1e(x),
                0x0A => self.op_fx0a(x),
                0x29 => self.op_fx29(x),
                0x33 => self.op_fx33(x),
                0x55 => self.op_fx55(x),
                0x65 => self.op_f65(x),
                _ => println!("Unknown opcode: {:04X}", opcode),
            }
            _ => println!("Unknown opcode: {:04X}", opcode),
        }
    }


    fn op_f65(&mut self, x: usize) {
        if x != 0 {
            for i in 0..x {
                self.v_reg[i] = self.memory[self.i_reg as usize + i];
            }
        } else {
            self.v_reg[0] = self.memory[self.i_reg as usize];
        }
    }

    fn op_fx55(&mut self, x: usize) {
        if x != 0 {
            for i in 0..x {
                self.memory[self.i_reg as usize + i] = self.v_reg[i];
            }
        } else {
            self.memory[self.i_reg as usize] = self.v_reg[0];
        }
    }

    fn op_fx33(&mut self, x: usize) {
        self.memory[self.i_reg as usize] = self.v_reg[x] / 100;
        self.memory[(self.i_reg + 1)  as usize] = (self.v_reg[x] % 100) / 10;
        self.memory[(self.i_reg + 2) as usize] = self.v_reg[x] % 10;
    }

    fn op_fx29(&mut self, x: usize) {
        self.i_reg = (self.v_reg[x] as u16) * 5;
    }

    fn op_fx0a(&mut self, x: usize) {
        if !self.key_pressed.0 {
            self.pc -= 2;
        } else {
            self.v_reg[x] = self.key_pressed.1;
            self.key_pressed.0 = false;
        }
    }

    fn op_fx1e(&mut self, x: usize) {
        let old_i = self.i_reg;
        let new_i= self.i_reg.wrapping_add(self.v_reg[x] as u16);
        self.i_reg = new_i;
        self.v_reg[0xF] = if new_i > 0xFFF && old_i < 0xFFF { 1 } else { 0 };
    }

    fn op_fx07(&mut self, x: usize) {
        self.v_reg[x] = self.delay_timer;
    }

    fn op_fx15(&mut self, x: usize) {
        self.delay_timer = self.v_reg[x];
    }

    fn op_fx18(&mut self, x: usize) {
        self.sound_timer = self.v_reg[x];
    }

    fn op_exa1(&mut self, x: usize) {
        if !self.keys[self.v_reg[x] as usize] {
            self.pc += 2;
        }        
    }

    fn op_ex9e(&mut self, x: usize) {
        if self.keys[self.v_reg[x] as usize] {
            self.pc += 2;
        }   
    }

    fn op_cxnn(&mut self, x: usize, nn: u8) {
        let mut rng = rand::thread_rng();
        let random_value = rng.gen::<u8>();
        self.v_reg[x] = random_value & nn;
    }

    fn op_bnnn(&mut self, x: usize, nnn: u16, nn: u8) {
        // TODO implement CHIP-48 and SUPER-CHIP behaviour, where jump as xnn + vx
        self.pc = nnn as usize + self.v_reg[0x0] as usize;
    }

    fn op_8xy7(&mut self, x: usize, y: usize) {
        let vy = self.v_reg[y];
        let vx = self.v_reg[x];
        let (result, carry) = vy.overflowing_sub(vx);
        self.v_reg[x] = result;
        self.v_reg[0xF] = if carry { 0 } else { 1 };
    }

    fn op_8xy5(&mut self, x: usize, y: usize) {
        let vy = self.v_reg[y];
        let vx = self.v_reg[x];
        let (result, carry) = vx.overflowing_sub(vy);
        self.v_reg[x] = result;
        self.v_reg[0xF] = if carry { 0 } else { 1 };
    }

    fn op_8xye(&mut self, y: usize, x: usize) {
        // Add optional parametor for oroginal COSMAC VIP option
        // (Optional, or configurable) Set VX to the value of VY
        /*
        self.v_reg[x] = self.v_reg[y];
         */
        self.v_reg[0xF] = self.v_reg[x] & 1;
        self.v_reg[x] = self.v_reg[x] << 1;
    }   

    fn op_8xy6(&mut self, y: usize, x: usize) {
        // Add optional parametor for oroginal COSMAC VIP option
        // (Optional, or configurable) Set VX to the value of VY
        /*
        self.v_reg[x] = self.v_reg[y];
         */
        self.v_reg[0xF] = self.v_reg[x] & 1;
        self.v_reg[x] = self.v_reg[x] >> 1;
    }

    fn op_8xy4(&mut self, x: usize, y: usize) {
        let (sum, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
        self.v_reg[x] = sum;

        self.v_reg[0xF] = if carry { 1 } else { 0 };
    }

    fn op_8xy3(&mut self, x: usize, y: usize) {
        self.v_reg[x] = self.v_reg[x] ^ self.v_reg[y];
    }

    fn op_8xy2(&mut self, x: usize, y: usize) {
        self.v_reg[x] = self.v_reg[x] & self.v_reg[y];
    }
    fn op_8xy1(&mut self, x: usize, y: usize) {
        self.v_reg[x] = self.v_reg[x] | self.v_reg[y];
    }

    fn op_8xy0(&mut self, x: usize, y: usize) {
        self.v_reg[x] = self.v_reg[y];
    }

    fn op_9xy0(&mut self, x: usize, y: usize) {
        if self.v_reg[x] != self.v_reg[y] {
            self.pc += 2;
        }
    }
    fn op_5xy0(&mut self, x: usize, y: usize) {
        if self.v_reg[x] == self.v_reg[y] {
            self.pc += 2;
        }
    }
    fn op_4xnn(&mut self, x: usize, nn: u8) {
        if self.v_reg[x] != nn {
            self.pc += 2;
        }
    }
    fn op_3xnn(&mut self, x: usize, nn: u8) {
        if self.v_reg[x] == nn {
            self.pc += 2;
        }
    }
    fn op_00e0(&mut self) {
        self.screen_data = [[false; WIDTH as usize]; HEIGHT as usize];
    }
    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn as usize;
    }
    fn op_00ee(&mut self) {
        if let Some(value) = self.stack.pop() {
            self.pc = value as usize;
        } else {
            panic!("op_00ee failed");
        }
    }
    fn op_2nnn(&mut self, nnn: u16) {
        self.stack.push(self.pc as u16);
        self.pc = nnn as usize;
    }
    fn op_6xnn(&mut self, x: u16, nn: u16) {
        self.v_reg[x as usize] = nn as u8;
    }
    fn op_7xnn(&mut self, x: u16, nn: u16) {
        self.v_reg[x as usize] = self.v_reg[x as usize].wrapping_add(nn as u8);
    }
    fn op_annn(&mut self, nnn: u16) {
        self.i_reg = nnn;
    }


    fn op_dxyn(&mut self, x: usize, y: usize, n: u8) {
        let vx = self.v_reg[x] as usize;
        let vy = self.v_reg[y] as usize;
        self.v_reg[0xF] = 0;

        for row in 0..n {
            let y = (vy + row as usize) % HEIGHT as usize;
            let sprite_data = self.memory[(self.i_reg as usize + row as usize) as usize];
            for b in 0..8 {
                let x = (vx + b as usize) % WIDTH as usize;

                let sprite_pixel = (sprite_data >> (7 - b)) & 1;
                if sprite_pixel == 1{
                    if self.screen_data[y][x] {
                        self.v_reg[0xF] = 1;
                    }
                    self.screen_data[y][x] ^= true;
                }
            }
        }
    }

}

fn main() -> io::Result<()> {
    let mut cpu = CPU::new();
    cpu.load_game("petdog.ch8")?;
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