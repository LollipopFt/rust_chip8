#![allow(nonstandard_style)]
use std::io::ErrorKind;

use sdl2::pixels::Color;
use sdl2::rect::Rect;

mod chip8;

use chip8::*;

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
    let filename = std::env::args()
        .last()
        .expect("\x1b[1;31mnot a file.\x1b[0m");
    if let Err(error) = chip8.romLoad(&filename) {
        if error.kind() == ErrorKind::PermissionDenied {
            println("file or directory permission lacking.");
            std::process::exit(0);
        } else if error.kind() == ErrorKind::NotFound {
            println("file or directory does not exist.");
            std::process::exit(0);
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
                        .expect("\x1b[1;31mdraw failed.\x1b[0m");
                }
                if !paint_white.is_empty() {
                    canvas.set_draw_color(Color::RGB(0xF0, 0xF6, 0xF0));
                    canvas
                        .fill_rects(paint_white.as_slice())
                        .expect("\x1b[1;31mdraw failed.\x1b[0m");
                }
            }
            canvas.present();
        }
    }
}

fn println(string: &str) {
    println!("\x1b[1;31m{string}\x1b[0m");
}
