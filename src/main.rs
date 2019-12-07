extern crate piston_window;

mod chip8;

use piston_window::*;
use std::io::prelude::*;
use std::fs::File;

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Chip8", [64 * 8, 32 * 8])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut input: [bool; 16] = [false; 16];
    let mut chip = chip8::new_chip8();
    chip.init();

    chip.load(&read_file("astro.ch8"));

    while let Some(e) = window.next() {
        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::Insert => input[0x0] = true,
                Key::End => input[0x1] = true,
                Key::Down => input[0x2] = true,
                Key::PageDown => input[0x3] = true,
                Key::Left => input[0x4] = true,
                Key::D5 => input[0x5] = true,
                Key::Right => input[0x6] = true,
                Key::Home => input[0x7] = true,
                Key::Up => input[0x8] = true,
                Key::PageUp => input[0x9] = true,
                Key::A => input[0xa] = true,
                Key::B => input[0xb] = true,
                Key::C => input[0xc] = true,
                Key::D => input[0xd] = true,
                Key::E => input[0xe] = true,
                Key::F => input[0xf] = true,
                _ => {}
            }
            chip.update_input(input);
        }
        if let Some(Button::Keyboard(key)) = e.release_args() {
            match key {
                Key::Insert => input[0x0] = false,
                Key::End => input[0x1] = false,
                Key::Down => input[0x2] = false,
                Key::PageDown => input[0x3] = false,
                Key::Left => input[0x4] = false,
                Key::D5 => input[0x5] = false,
                Key::Right => input[0x6] = false,
                Key::Home => input[0x7] = false,
                Key::Up => input[0x8] = false,
                Key::PageUp => input[0x9] = false,
                Key::A => input[0xa] = false,
                Key::B => input[0xb] = false,
                Key::C => input[0xc] = false,
                Key::D => input[0xd] = false,
                Key::E => input[0xe] = false,
                Key::F => input[0xf] = false,
                _ => {}
            }
            chip.update_input(input);
        }

        if let Some(_) = e.update_args() {
            chip.cycle();
        }

        if let Some(_) = e.render_args() {
            window.draw_2d(&e, |c, g, _d| {
                clear([0.0, 0.0, 0.0, 0.0], g);
                for y in 0..32 {
                    for x in 0..64 {
                        if (chip.display[y] >> x) & 1 == 1 {
                            rectangle(
                                [1.0, 1.0, 1.0, 1.0],
                                [(63 - x) as f64 * 8.0, y as f64 * 8.0, 8.0, 8.0],
                                c.transform,
                                g
                            );
                        }
                    }
                }
            });
        }
    }
}

fn read_file(path: &str) -> [u8; 3584] {
    let mut f = File::open(path).unwrap();
    let mut buffer = [0u8; 3584];
    
    f.read(&mut buffer).unwrap();
    
    buffer
}
