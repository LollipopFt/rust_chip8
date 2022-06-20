use std::fs::File;
use std::io::{Read, Result};
type u12 = u16;

use sdl2::rect::Rect;

pub struct Chip8 {
    memory: [u8; 4096],
    vreg: [u8; 16],
    index: u12,
    pc: u12,
    delay_timer: u8,
    sound_timer: u8,
    stack: Vec<u16>,
    graphic: [bool; 64 * 32],
    pub draw_flag: bool,
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
    pub fn romLoad(&mut self, filename: &str) -> Result<()> {
        let mut rom = File::open(filename)?;
        rom.read_exact(&mut self.memory[0x200..])?;
        Ok(())
    }

    pub fn fontLoad(&mut self, font: [u8; 5 * 16]) {
        self.memory[0x50..0x0A0].copy_from_slice(&font);
    }

    pub fn emuCycle(&mut self, black: &mut Vec<Rect>, white: &mut Vec<Rect>) {
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
                        self.pc = self.stack.pop().unwrap();
                    }
                }
            }
            0x1000 => {
                self.pc = opcode & 0x0FFF;
            }
            0x2000 => {
                self.stack.push(self.pc);
                self.pc = opcode & 0x0FFF;
            }
            0x3000 => {
                if self.vreg[((opcode & 0x0F00) >> 8) as usize]
                    == (opcode & 0x00FF) as u8
                {
                    self.pc += 2;
                }
            }
            0x4000 => {
                if self.vreg[((opcode & 0x0F00) >> 8) as usize]
                    != (opcode & 0x00FF) as u8
                {
                    self.pc += 2;
                }
            }
            0x5000 => {
                if self.vreg[((opcode & 0x0F00) >> 8) as usize]
                    == self.vreg[((opcode & 0x00F0) >> 4) as usize]
                {
                    self.pc += 2;
                }
            }
            0x6000 => {
                self.vreg[((opcode & 0x0F00) >> 8) as usize] = opcode as u8;
            }
            0x7000 => {
                let num = self.vreg[((opcode & 0x0F00) >> 8) as usize]
                    .wrapping_add(opcode as u8);
                self.vreg[((opcode & 0x0F00) >> 8) as usize] = num;
            }
            0x8000 => {
                let halfbyte2 = ((opcode & 0x0F00) >> 8) as usize;
                let halfbyte3 = ((opcode & 0x00F0) >> 4) as usize;
                match opcode & 0x000F {
                    0 => self.vreg[halfbyte2] = self.vreg[halfbyte3],
                    1 => self.vreg[halfbyte2] |= self.vreg[halfbyte3],
                    2 => self.vreg[halfbyte2] &= self.vreg[halfbyte3],
                    3 => self.vreg[halfbyte2] ^= self.vreg[halfbyte3],
                    4 => {
                        let (num, carry) = self.vreg[halfbyte2]
                            .overflowing_add(self.vreg[halfbyte3]);
                        self.vreg[halfbyte2] = num;
                        if carry {
                            self.vreg[15] = 1;
                        } else {
                            self.vreg[15] = 0;
                        }
                    }
                    5 => {
                        let (num, carry) = self.vreg[halfbyte2]
                            .overflowing_sub(self.vreg[halfbyte3]);
                        self.vreg[halfbyte2] = num;
                        if carry {
                            self.vreg[15] = 0;
                        } else {
                            self.vreg[15] = 1;
                        }
                    }
                    6 => {
                        self.vreg[15] = self.vreg[halfbyte2] & 1;
                        self.vreg[halfbyte2] >>= 1;
                    }
                    7 => {
                        let (num, carry) = self.vreg[halfbyte3]
                            .overflowing_sub(self.vreg[halfbyte2]);
                        self.vreg[halfbyte2] = num;
                        if carry {
                            self.vreg[15] = 0;
                        } else {
                            self.vreg[15] = 1;
                        }
                    }
                    0xE => {
                        self.vreg[15] = self.vreg[halfbyte2] & 8;
                        self.vreg[halfbyte2] <<= 1;
                    }
                    _ => {}
                }
            }
            0x9000 => {
                if self.vreg[((opcode & 0x0F00) >> 8) as usize]
                    != self.vreg[((opcode & 0x00F0) >> 4) as usize]
                {
                    self.pc += 2;
                }
            }
            0xA000 => {
                self.index = opcode & 0x0FFF;
            }
            #[allow(clippy::precedence)]
            0xB000 => {
                self.pc = opcode & 0x0FFF + self.vreg[0] as u12;
            }
            0xC000 => {
                self.vreg[((opcode & 0x0F00) >> 8) as usize] =
                    rand() & (opcode & 0x00FF) as u8
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
            0xE000 => match opcode & 0x000F {
                0xE => {}
            },
            _ => {
                println!("unknown opcode: {opcode:#x}");
            }
        }
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("\x07");
            }
            self.sound_timer -= 1;
        }
    }
}

fn newRect(x: u8, y: u8) -> Rect {
    Rect::new(x as i32 * 20, y as i32 * 20, 20, 20)
}

fn rand() -> u8 {
    rand::random::<u8>()
}
