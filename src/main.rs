use std::fs;

use sdl2::{render::{Canvas, WindowCanvas}, video::{Window, self}, pixels::Color, rect::Rect};

const WIDTH: u16 = 64;
const HEIGHT: u16 = 32;
const MAGNITUDE: u16 = 10;
const IBM_Logo: &str = "./IBM_Logo.ch8";

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
    key: [u8; 16],
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
            key: [0; 16],
        }
    }
    pub fn load_binary(&mut self) {
        let binary = fs::read(IBM_Logo)
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

        let n = opcode & 0x000F;
        let nn = opcode & 0x00FF;
        let nnn = opcode & 0x0FFF;

        match first {
            0x0 => {
                self.vram = [false; 64 * 32];
                self.pc += 2;
            }
            0x1 => {
                self.pc = nnn as usize;
            }
            0x6 => {
                self.registers[x as usize] = nn as u8;
                self.pc += 2;
            }
            0x7 => {
                self.registers[x as usize] += nn as u8;
                self.pc += 2;
            }
            0xA => {
                self.i = nnn as usize;
                self.pc += 2;
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
                            if self.vram[(vx + pixel + (vy + row) * WIDTH) as usize] {
                                self.registers[0xF] = 1;
                            }
                            self.vram[(vx + pixel + (vy + row) * WIDTH) as usize] ^= true;
                        }
                    }
                }
                self.pc += 2;
            }
            _ => {

            }
        }

    }
}

struct Display {
    canvas: WindowCanvas,
}

impl Display {
    pub fn new() -> Result<Display, String> {
        let sdl_context = sdl2::init()?;
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
        self.canvas.set_draw_color(Color::GREEN);
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

//delta somewhere

fn main() {
    let mut processor = Processor::new();
    let mut display = Display::new().expect("Display init failed");

    processor.load_binary();

    loop {
        let opcode = processor.fetch_opcode();

        processor.execute_opcode(opcode);

        display.render(&processor.vram);
    }
}
