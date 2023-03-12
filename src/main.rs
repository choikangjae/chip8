use std::fs;

use rand::Rng;
use sdl2::{render::WindowCanvas, pixels::Color, rect::Rect, event::Event};
use sdl2::keyboard::Keycode;
use std::time::Duration;
use std::thread;

const WIDTH: u16 = 64;
const HEIGHT: u16 = 32;
const MAGNITUDE: u16 = 10;

pub fn get_rom(name: &str) -> &str {
    match name {
        "IBM" => {
            "./IBM_Logo.ch8"
        }
        "SPACE_INVADERS" => {
            "./Space_Invaders.ch8"
        }
        "TEST" => {
            "./chip8-test-suite.ch8"
        }
        _ => {
            println!("Not available");
            "qwe"
        }
    }
}

struct Processor {
    ram: [u8; 4096],
    registers: [u8; 16],
    i: usize,
    delay_timer: u8,
    sound_timer: u8,
    pc: usize,
    sp: usize,
    stack: [u16; 16],
    vram: [bool; (WIDTH * HEIGHT) as usize],
    key: [bool; 16],
}

impl Processor {
    fn new() -> Self {
        let mut ram = [0; 4096];
        let chip8_fontset: [u8; 80] = 
        [
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
        for i in 0..80 {
            ram[i] = chip8_fontset[i];
        };

        Self {
            ram,
            registers: [0; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            vram: [false; 64 * 32],
            key: [false; 16],
        }
    }
    pub fn load_binary(&mut self) {
        let binary = fs::read(get_rom("TEST"))
            .expect("ROM not found");
        for i in 0..binary.len() {
            self.ram[i + 0x200] = binary[i]
        }
    }

    pub fn fetch_opcode(&self) -> u16 {
        (self.ram[self.pc] as u16) << 8 | (self.ram[self.pc + 1] as u16)
    }

    pub fn execute_opcode(&mut self, opcode: u16) {
        let first = opcode >> 12;
        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;

        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        let n = (opcode & 0x000F) as u8;
        let nn = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;

        // println!("{:#x}", opcode);
        self.pc += 2;

        match first {
            0x0 => {
                match nn {
                    0xE0 => {
                        self.vram = [false; 64 * 32];
                    }
                    0xEE => {
                        self.sp -= 1;
                        self.pc = self.stack[self.sp] as usize;
                    }
                    _ => {}
                }
            }
            0x1 => {
                self.pc = nnn as usize;
            }
            0x2 => {
                self.stack[self.sp] = self.pc as u16;
                self.sp += 1;
                self.pc = nnn as usize;
            }
            0x3 => {
                if vx == nn {
                    self.pc += 2
                }
            }
            0x4 => {
                if vx != nn {
                    self.pc += 2
                }
            }
            0x5 => {
                if vx == vy {
                    self.pc += 2
                }
            }
            0x6 => {
                self.registers[x as usize] = nn;
            }
            0x7 => {
                // self.registers[x as usize] += nn;
                self.registers[x as usize] = self.registers[x as usize].overflowing_add(nn).0;
            }
            0x8 => {
                match n {
                    0x0 => {
                        self.registers[x as usize] = vy;
                    }
                    0x1 => {
                        self.registers[x as usize] = vx | vy;
                    }
                    0x2 => {
                        self.registers[x as usize] = vx & vy;
                    }
                    0x3 => {
                        self.registers[x as usize] = vx ^ vy;
                    }
                    0x4 => {
                        let (val, flag) = vx.overflowing_add(vy);
                        self.registers[x as usize] = val;
                        match flag {
                            true => self.registers[0xF] = 1,
                            false => self.registers[0xF] = 0,
                        }
                    }
                    0x5 => {
                        let (val, flag) = self.registers[x as usize].overflowing_sub(self.registers[y as usize]);
                        self.registers[x as usize] = val;
                        match flag {
                            true => self.registers[0xF] = 0,
                            false => self.registers[0xF] = 1,
                        }
                    }
                    0x6 => {
                        self.registers[0xF] = vx & 1;
                        self.registers[x as usize] /= 2;
                    }
                    0x7 => {
                        let (val, flag) = self.registers[y as usize].overflowing_sub(self.registers[x as usize]);
                        self.registers[x as usize] = val;
                        match flag {
                            true => self.registers[0xF] = 0,
                            false => self.registers[0xF] = 1,
                        }
                    }
                    0xE => {
                        if (vx >> 7) & 1 == 1 {
                            self.registers[0xF] = 1;
                        } else {
                            self.registers[0xF] = 0;
                        }
                        self.registers[x as usize] /= 2;
                    }
                    _ => {}
                }
            }
            0x9 => {
                if vx != vy {
                    self.pc += 2;
                }
            }
            0xA => {
                self.i = nnn as usize;
            }
            0xB => {
                self.pc = (nnn + self.registers[0x0] as u16) as usize;
            }
            0xC => {
                let rand = rand::thread_rng().gen_range(0..=255);
                self.registers[x as usize] = rand & nn;
            }
            //DXYN
            //It draws N pixels tall sprite from the memory location that the I index register is holding to the screen, at (vx, vy)
            0xD => {
                let vx = self.registers[x as usize] as u16;
                let vy = self.registers[y as usize] as u16;
                self.registers[0xF] = 0;
                for row in 0..n {
                    let row_data = self.ram[self.i + row as usize] as u16;
                    for pixel in 0..8 {
                        if (row_data & (0b1000_0000 >> pixel)) != 0 {
                            let x_width = (vx + pixel) % 64;
                            let y_height = (vy + row as u16) % 32;
                            if self.vram[(x_width + y_height * WIDTH) as usize] {
                                self.registers[0xF] = 1;
                            }
                            self.vram[(x_width + y_height * WIDTH) as usize] ^= true;
                        }
                    }
                }
            }
            0xE => {
                match nn {
                    0x9E => {
                        if self.key[vx as usize] {
                            self.pc += 2;
                        }
                    }
                    0xA1 => {
                        if !self.key[vx as usize] {
                            self.pc += 2;
                        }
                    }
                    _ => {}
                }
            }
            0xF => {
                match nn {
                    0x07 => {
                        self.registers[x as usize] = self.delay_timer;
                    }
                    0x0A => {
                        self.pc -= 2;
                        for (i, pressed) in self.key.iter().enumerate() {
                            // println!("{pressed}");
                            if *pressed {
                                self.registers[x as usize] = i as u8;
                                self.pc += 4;
                                break;
                            }
                        }
                    }
                    0x15 => {
                        self.delay_timer = vx;
                    }
                    0x18 => {
                        self.sound_timer = vx;
                    }
                    0x1E => {
                        self.i += vx as usize;
                    }
                    0x29 => {
                        self.i = vx as usize;
                    }
                    0x33 => {
                        self.ram[self.i] = vx / 100;
                        let vx = vx - vx / 100;
                        self.ram[self.i + 1] = vx / 10;
                        let vx = vx - vx / 10;
                        self.ram[self.i + 2] = vx;
                    }
                    0x55 => {
                        //todo
                        for num in 0..=x {
                            self.ram[self.i + num as usize] = self.registers[num as usize];
                        }
                    }
                    0x65 => {
                        for num in 0..=x {
                            self.registers[num as usize] = self.ram[self.i + num as usize];
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

    }

    pub fn key_down(&mut self, keycode :Keycode) {
        match keycode {
            Keycode::Num1 => self.key[0x01] = true,
            Keycode::Num2 => self.key[0x02] = true,
            Keycode::Num3 => self.key[0x03] = true,
            Keycode::Num4 => self.key[0x0C] = true,
            Keycode::Q => self.key[0x04] = true,
            Keycode::W => self.key[0x05] = true,
            Keycode::E => self.key[0x06] = true,
            Keycode::R => self.key[0x0D] = true,
            Keycode::A => self.key[0x07] = true,
            Keycode::S => self.key[0x08] = true,
            Keycode::D => self.key[0x09] = true,
            Keycode::F => self.key[0x0E] = true,
            Keycode::Z => self.key[0x0A] = true,
            Keycode::X => self.key[0x00] = true,
            Keycode::C => self.key[0x0B] = true,
            Keycode::V => self.key[0x0F] = true,
            _ => {}
        }
    }

    pub fn key_up(&mut self, keycode :Keycode) {
        match keycode {
            Keycode::Num1 => self.key[0x01] = false,
            Keycode::Num2 => self.key[0x02] = false,
            Keycode::Num3 => self.key[0x03] = false,
            Keycode::Num4 => self.key[0x0C] = false,
            Keycode::Q => self.key[0x04] = false,
            Keycode::W => self.key[0x05] = false,
            Keycode::E => self.key[0x06] = false,
            Keycode::R => self.key[0x0D] = false,
            Keycode::A => self.key[0x07] = false,
            Keycode::S => self.key[0x08] = false,
            Keycode::D => self.key[0x09] = false,
            Keycode::F => self.key[0x0E] = false,
            Keycode::Z => self.key[0x0A] = false,
            Keycode::X => self.key[0x00] = false,
            Keycode::C => self.key[0x0B] = false,
            Keycode::V => self.key[0x0F] = false,
            _ => {}
        }
    }

    pub fn set_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1
        }
    }
}

struct Display {
    canvas: WindowCanvas,
}

impl Display {
    pub fn new(sdl_context: &sdl2::Sdl) -> Result<Display, String> {
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window("Chip8", (WIDTH * MAGNITUDE) as u32, (HEIGHT * MAGNITUDE) as u32)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;
        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.present();

        Ok(Display { canvas })
    }
    pub fn render(&mut self, vram: &[bool; (WIDTH * HEIGHT) as usize]) {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
        self.canvas.set_draw_color(Color::WHITE);
        for row in 0..HEIGHT {
            for col in 0..WIDTH {
                if vram[(row * WIDTH + col) as usize] {
                    self.canvas.fill_rect(Rect::new(col as i32 * MAGNITUDE as i32, row as i32 * MAGNITUDE as i32, MAGNITUDE.into(), MAGNITUDE.into())).expect("Drawing failed");
                }
            }
        }
        self.canvas.present();
    }
}

fn main() {
    let mut processor = Processor::new();
    let sdl_context = sdl2::init().unwrap();
    let mut display = Display::new(&sdl_context).expect("Display init failed");
    let mut event_pump = sdl_context.event_pump().unwrap();

    processor.load_binary();

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                    | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {}
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => processor.key_down(keycode),
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => processor.key_up(keycode),
                _ => {}
            }
        }
        let opcode = processor.fetch_opcode();

        processor.execute_opcode(opcode);

        display.render(&processor.vram);

        processor.set_timers();

        thread::sleep(Duration::from_nanos(500));
    }
}
