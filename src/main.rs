#[macro_use]
extern crate bitflags;

extern crate minifb;

use minifb::{Key, WindowOptions, Window};

use std::env;
use std::path::PathBuf;
use std::boxed::Box;
use std::fs::File;
use std::io::{Read, Write};

mod gbc;

use gbc::console::{Console,Button,ButtonState,InputEvent,Cart};

fn load_bin(path: &PathBuf) -> Box<[u8]> {
    let mut bytes = Vec::new();
    let mut file = File::open(path).unwrap();
    file.read_to_end(&mut bytes).unwrap();
    bytes.into_boxed_slice()
}

fn save_bin(path: &PathBuf, bytes: Box<[u8]>) {
    let mut file = File::create(path).unwrap();
    file.write_all(&bytes).unwrap();
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

struct VideoSink<'a> {
    window: &'a mut Window
}

impl<'a> VideoSink<'a> {
    fn new(window: &'a mut Window) -> VideoSink<'a> {
        VideoSink {
            window
        }
    }
}

impl<'a> gbc::console::VideoSink for VideoSink<'a> {
    fn frame_available(&mut self, frame: &Box<[u32]>) {
        self.window.update_with_buffer(frame)
    }
}

fn main() {
    let rom_path = PathBuf::from(env::args().nth(1).unwrap());
    let rom_binary = load_bin(&rom_path);

    let save_ram_path = {
        let mut path = rom_path.clone();
        path.set_extension("sav");
        path
    };

    let ram = if save_ram_path.exists() {
        Some(load_bin(&save_ram_path))
    } else {
        None
    };

    let cart = Cart::new(rom_binary, ram);

    println!("{:?}", cart);

    let mut console = Console::new(cart);

    let mut window = Window::new("GBC_RS",
                                 160,
                                 144,
                                 WindowOptions { scale: minifb::Scale::X2, ..Default::default() })
        .unwrap_or_else(|e| panic!("{}", e));

    let sleep_time = std::time::Duration::from_millis(16);

    let mut prev_keys = Vec::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {

        let now = std::time::Instant::now();

        console.run_for_one_frame(&mut VideoSink::new(&mut window));

        if let Some(keys) = window.get_keys() {
            make_events(keys.clone(), prev_keys)
                .into_iter()
                .for_each(|e| console.handle_event(e));    
            prev_keys = keys
        }

        let elapsed = now.elapsed();
        if sleep_time > elapsed {
            let sleep = sleep_time - elapsed;
            std::thread::sleep(sleep)
        }
    }

    if let Some(ram) = console.copy_cart_ram() {
        save_bin(&save_ram_path, ram)
    }
}
