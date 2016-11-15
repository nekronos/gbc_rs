
extern crate sdl2;

use sdl2::pixels::PixelFormatEnum;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::env;
use std::path::PathBuf;
use std::boxed::Box;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

mod gbc;

use gbc::cart::Cart;
use gbc::cpu::Cpu;
use gbc::ppu::Ppu;
use gbc::spu::Spu;
use gbc::gamepad::{Gamepad, Button, ButtonState, InputEvent};
use gbc::interconnect::Interconnect;

fn load_bin(path: &PathBuf) -> Box<[u8]> {
    let mut bytes = Vec::new();
    let mut file = File::open(path).unwrap();
    file.read_to_end(&mut bytes).unwrap();
    bytes.into_boxed_slice()
}

fn keycode_to_button(keycode: Keycode) -> Button {
    match keycode {
        Keycode::LAlt => Button::A,
        Keycode::LCtrl => Button::B,
        Keycode::Return => Button::Start,
        Keycode::RShift => Button::Select,
        Keycode::Up => Button::Up,
        Keycode::Down => Button::Down,
        Keycode::Left => Button::Left,
        Keycode::Right => Button::Right,
        _ => panic!("Keycode not supported: {:?}", keycode),
    }
}

fn main() {
    let rom_path = PathBuf::from(env::args().nth(1).unwrap());
    let rom_binary = load_bin(&rom_path);

    let cart = Cart::new(rom_binary);

    println!("{:?}", cart);

    // let gb_type = cart.gameboy_type();
    let gb_type = gbc::GameboyType::Dmg;

    let (tx, rx): (Sender<Box<[u8]>>, Receiver<Box<[u8]>>) = mpsc::channel();
    let (gamepad_tx, gamepad_rx): (Sender<InputEvent>, Receiver<InputEvent>) = mpsc::channel();

    let ppu = Ppu::new(tx.clone());
    let spu = Spu::new();
    let gamepad = Gamepad::new(gamepad_rx);
    let interconnect = Interconnect::new(gb_type, cart, ppu, spu, gamepad);

    let mut cpu = Cpu::new(gb_type, interconnect);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("GBC_RS", 160 * 4, 144 * 4)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut renderer = window.renderer().build().unwrap();
    let mut texture = renderer.create_texture_streaming(PixelFormatEnum::BGRA8888, 160, 144)
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let sleep_time = std::time::Duration::from_millis(16);

    'running: loop {

        let now = std::time::Instant::now();

        let mut cycle_count: u32 = 0;

        loop {
            cycle_count += cpu.step() as u32;
            if cycle_count >= 70224 {
                break;
            }
        }

        if let Ok(framebuffer) = rx.try_recv() {
            texture.update(None, &framebuffer, 160 * 4).unwrap();
        }

        renderer.clear();
        renderer.copy(&texture, None, None).unwrap();
        renderer.present();

        for event in event_pump.poll_iter() {
            use sdl2::keyboard::Keycode::*;
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    match keycode {
                        LAlt | LCtrl | Return | RShift | Up | Down | Left | Right => {
                            gamepad_tx.send(InputEvent::new(keycode_to_button(keycode),
                                                      ButtonState::Down))
                                .unwrap()
                        }
                        _ => {}
                    }
                }
                Event::KeyUp { keycode: Some(keycode), .. } => {
                    match keycode {
                        LAlt | LCtrl | Return | RShift | Up | Down | Left | Right => {
                            gamepad_tx.send(InputEvent::new(keycode_to_button(keycode),
                                                      ButtonState::Up))
                                .unwrap()
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        let elapsed = now.elapsed();
        if sleep_time > elapsed {
            let sleep = sleep_time - elapsed;
            std::thread::sleep(sleep)
        }

    }
}
