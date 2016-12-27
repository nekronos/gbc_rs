#[macro_use]
extern crate bitflags;

extern crate minifb;

use minifb::{Key, WindowOptions, Window};

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

fn keycode_to_button(keycode: Key) -> Option<Button> {
    match keycode {
        Key::Space => Some(Button::A),
        Key::LeftCtrl => Some(Button::B),
        Key::Enter => Some(Button::Start),
        Key::RightShift => Some(Button::Select),
        Key::Up => Some(Button::Up),
        Key::Down => Some(Button::Down),
        Key::Left => Some(Button::Left),
        Key::Right => Some(Button::Right),
        _ => None,
    }
}

fn make_events(current: Vec<Key>, prev: Vec<Key>) -> Vec<InputEvent> {

    let released: Vec<_> = prev.clone().into_iter().filter(|x| !current.contains(x)).collect();
    let pressed: Vec<_> = current.into_iter().filter(|x| !prev.contains(x)).collect();

    let mut events = Vec::new();

    for r in released {
        if let Some(button) = keycode_to_button(r) {
            events.push(InputEvent::new(button, ButtonState::Up))
        }
    }

    for p in pressed {
        if let Some(button) = keycode_to_button(p) {
            events.push(InputEvent::new(button, ButtonState::Down))
        }
    }
    events
}

fn main() {
    let rom_path = PathBuf::from(env::args().nth(1).unwrap());
    let rom_binary = load_bin(&rom_path);

    let cart = Cart::new(rom_binary);

    println!("{:?}", cart);

    // let gb_type = cart.gameboy_type();
    let gb_type = gbc::GameboyType::Dmg;

    let (tx, rx): (Sender<Box<[u32]>>, Receiver<Box<[u32]>>) = mpsc::channel();
    let (gamepad_tx, gamepad_rx): (Sender<InputEvent>, Receiver<InputEvent>) = mpsc::channel();

    let ppu = Ppu::new(tx.clone());
    let spu = Spu::new();
    let gamepad = Gamepad::new(gamepad_rx);
    let interconnect = Interconnect::new(gb_type, cart, ppu, spu, gamepad);

    let mut cpu = Cpu::new(gb_type, interconnect);

    let mut window = Window::new("GBC_RS",
                                 160,
                                 144,
                                 WindowOptions { scale: minifb::Scale::X4, ..Default::default() })
        .unwrap_or_else(|e| panic!("{}", e));

    let sleep_time = std::time::Duration::from_millis(16);

    let mut prev_keys = Vec::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {

        let now = std::time::Instant::now();

        let mut cycle_count: u32 = 0;

        loop {
            cycle_count += cpu.step() as u32;
            if cycle_count >= 70224 {
                break;
            }
        }

        if let Ok(framebuffer) = rx.try_recv() {
            window.update_with_buffer(&framebuffer)
        } else {
            window.update()
        }

        if let Some(keys) = window.get_keys() {
            make_events(keys.clone(), prev_keys)
                .into_iter()
                .map(|e| gamepad_tx.send(e).unwrap())
                .collect::<Vec<_>>();
            prev_keys = keys
        }

        let elapsed = now.elapsed();
        if sleep_time > elapsed {
            let sleep = sleep_time - elapsed;
            std::thread::sleep(sleep)
        }
    }
}
