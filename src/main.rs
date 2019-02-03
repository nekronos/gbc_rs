#[macro_use]
extern crate bitflags;
extern crate sdl2;

use std::env;
use std::path::PathBuf;
use std::boxed::Box;
use std::fs::File;
use std::io::{Read, Write};

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

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Texture;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

const SCALE: usize = 4;

const WINDOW_WIDTH: usize = WIDTH * SCALE;
const WINDOW_HEIGHT: usize = HEIGHT * SCALE;

mod gbc;

use gbc::console::{Console,Button,ButtonState,InputEvent,Cart};

fn make_events(current: &Vec<Keycode>, prev: &Vec<Keycode>) -> Vec<InputEvent> {

    let released: Vec<_> = prev.clone().into_iter().filter(|x| !current.contains(x)).collect();
    let pressed: Vec<_> = current.into_iter().filter(|x| !prev.contains(x)).collect();

    let mut events = Vec::new();

    for r in released {
        if let Some(button) = r.into_button() {
            events.push(InputEvent::new(button, ButtonState::Up))
        }
    }

    for p in pressed {
        if let Some(button) = p.into_button() {
            events.push(InputEvent::new(button, ButtonState::Down))
        }
    }
    events
}

impl<'a> gbc::console::VideoSink for Texture<'a> {
    fn frame_available(&mut self, frame: &Box<[u32]>) {
        unsafe {
            let size = frame.len() * 4;
            let frame = std::slice::from_raw_parts(frame.as_ptr() as *const u8, size);
            let _ = self.update(Rect::new(0, 0, WIDTH as _, HEIGHT as _), frame, (WIDTH * 4) as _).unwrap();
        }
    }
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("gbc_rs", WINDOW_WIDTH as _, WINDOW_HEIGHT as _)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())?;
    
    let texture_creator = canvas.texture_creator();

    let mut texture: Texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGBA32, WIDTH as _, HEIGHT as _)
        .map_err(|e| e.to_string())?;

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

    let mut event_pump = sdl_context.event_pump()?;

    let sleep_time = std::time::Duration::from_millis(16);

    let mut prev_keys: Vec<Keycode> = Vec::new();

    'running: loop {
        let now = std::time::Instant::now();

        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                break 'running
            }
        }

        let keys = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();
    
        make_events(&keys, &prev_keys)
            .into_iter()
            .for_each(|e| console.handle_event(e));
        prev_keys = keys;
    
        console.run_for_one_frame(&mut texture);

        canvas.clear();
        canvas.copy(&texture, None, Some(Rect::new(0, 0, WINDOW_WIDTH as _, WINDOW_HEIGHT as _)))?;
        canvas.present();

        let elapsed = now.elapsed();
        if sleep_time > elapsed {
            let sleep = sleep_time - elapsed;
            std::thread::sleep(sleep)
        }
    }

    Ok(())
}

trait IntoButton {
    fn into_button(self) -> Option<Button>;
}

impl IntoButton for Keycode {
    fn into_button(self) -> Option<Button> {
        match self {
            Keycode::Space => Some(Button::A),
            Keycode::LCtrl => Some(Button::B),
            Keycode::Return => Some(Button::Start),
            Keycode::RShift => Some(Button::Select),
            Keycode::Up => Some(Button::Up),
            Keycode::Down => Some(Button::Down),
            Keycode::Left => Some(Button::Left),
            Keycode::Right => Some(Button::Right),
            _ => None,
        }
    }
}