#![allow(nonstandard_style)]
use std::fs::File;
use std::io::{ErrorKind, Read, Result};
type u12 = u16;

use sdl2::pixels::Color;
use sdl2::rect::Rect;

struct Chip8 {
    memory: [u8; 4096],
    vreg: [u8; 16],
    index: u12,
    pc: u12,
    delay_timer: u8,
    sound_timer: u8,
    stack: Vec<u16>,
    graphic: [bool; 64 * 32],
    draw_flag: bool,
}

impl Default for Chip8 {
    fn default() -> Self {
        Chip8 {
            memory: [0; 4096],
            vreg: [0; 16],
            index: 0,
            pc: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            stack: Vec::with_capacity(16),
            graphic: [false; 64 * 32],
            draw_flag: false,
        }
    }
}

impl Chip8 {
    fn romLoad(&mut self, filename: &str) -> Result<()> {
        let mut rom = File::open(filename)?;
        rom.read_exact(&mut self.memory[0x200..])?;
        Ok(())
    }

    fn fontLoad(&mut self, font: [u8; 5 * 16]) {
        self.memory[0x50..0x0A0].copy_from_slice(&font);
    }

    fn emuCycle(&mut self, black: &mut Vec<Rect>, white: &mut Vec<Rect>) {
        self.exOp(black, white);
    }

    fn fetchOp(&self) -> u16 {
        let op_array: [u8; 2] = self.memory
            [self.pc as usize..(self.pc + 2) as usize]
            .try_into()
            .unwrap();
        u16::from_be_bytes(op_array)
    }

    fn exOp(&mut self, black: &mut Vec<Rect>, white: &mut Vec<Rect>) {
        let opcode = self.fetchOp();
        self.pc += 2;
        match opcode & 0xF000 {
            0x0000 => {
                if (opcode & 0x00F0) == 0x00E0 {
                    if (opcode & 0x000F) != 0x000E {
                        self.draw_flag = true;
                    } else {
                        println!("unknown opcode: {opcode:#x}");
                    }
                }
            }
            0x1000 => {
                self.pc = opcode & 0x0FFF;
            }
            0x6000 => {
                self.vreg[((opcode & 0x0F00) >> 8) as usize] = opcode as u8;
            }
            0x7000 => {
                self.vreg[((opcode & 0x0F00) >> 8) as usize] += opcode as u8;
            }
            0xA000 => {
                self.index = opcode & 0x0FFF;
            }
            0xD000 => {
                self.draw_flag = true;
                let x = self.vreg[((opcode & 0x0F00) >> 8) as usize];
                let y = self.vreg[((opcode & 0x00F0) >> 4) as usize];
                let height = opcode & 0x000F;
                self.vreg[15] = 0;
                for yline in 0..height {
                    let pxl = self.memory[(self.index + yline) as usize];
                    for xline in 0..8 {
                        if (pxl & (0x80 >> xline)) != 0 {
                            let num = x as usize
                                + xline as usize
                                + (y as u16 + yline) as usize * 64;
                            let pixel = self.graphic[num];
                            if pixel {
                                self.vreg[15] = 1;
                                black.push(newRect(x + xline, y + yline as u8));
                            } else {
                                white.push(newRect(x + xline, y + yline as u8));
                            }
                            self.graphic[num] = !pixel;
                        }
                    }
                }
            }
            _ => {
                println!("unknown opcode: {opcode:#x}");
            }
        }
    }
}

fn newRect(x: u8, y: u8) -> Rect {
    Rect::new(x as i32 * 20, y as i32 * 20, 20, 20)
}

const font1: [u8; 5 * 16] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10,
    0xF0, 0x80, 0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10,
    0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0,
    0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80,
    0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];

fn main() {
    let mut chip8 = Chip8::default();

    // error handling for rom upload
    let filename = std::env::args().last().expect("not a file.");
    if let Err(error) = chip8.romLoad(&filename) {
        if error.kind() == ErrorKind::PermissionDenied {
            println!("file or directory permission lacking.");
            std::process::exit(1);
        }
    }
    chip8.fontLoad(font1);

    let sdl_context = sdl2::init().unwrap();
    let video_subsys = sdl_context.video().unwrap();

    let window = video_subsys
        .window("CHIP-8", 1280, 640)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    canvas.set_draw_color(Color::RGB(0x22, 0x23, 0x23));
    canvas.clear();
    canvas.present();

    let mut paint_black: Vec<Rect> = Vec::new();
    let mut paint_white: Vec<Rect> = Vec::new();

    /*'running: */
    loop {
        paint_white.clear();
        paint_black.clear();
        chip8.draw_flag = false;
        chip8.emuCycle(&mut paint_black, &mut paint_white);
        if chip8.draw_flag {
            if paint_black.is_empty() && paint_white.is_empty() {
                canvas.set_draw_color(Color::RGB(0x22, 0x23, 0x23));
                canvas.clear();
            } else {
                if !paint_black.is_empty() {
                    canvas.set_draw_color(Color::RGB(0x22, 0x23, 0x23));
                    canvas
                        .fill_rects(paint_black.as_slice())
                        .expect("draw failed.");
                }
                if !paint_white.is_empty() {
                    canvas.set_draw_color(Color::RGB(0xF0, 0xF6, 0xF0));
                    canvas
                        .fill_rects(paint_white.as_slice())
                        .expect("draw failed.");
                }
            }
            canvas.present();
        }
    }
}
