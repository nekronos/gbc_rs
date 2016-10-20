use super::Interrupt;

#[derive(Debug)]
pub enum ButtonState {
    Pressed,
    Released,
}

#[derive(Debug)]
pub enum Button {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
    Select,
}

#[derive(Debug)]
pub struct InputEvent {
    button: Button,
    state: ButtonState,
}

pub struct Gamepad {
    p1: u8,
}

impl Gamepad {
    pub fn new() -> Gamepad {
        Gamepad { p1: 0 }
    }

    pub fn read(&self) -> u8 {
        self.p1
    }

    pub fn write(&mut self, val: u8) {}

    pub fn cycle_flush(&mut self, cycle_count: u32) -> Option<Interrupt> {
        None
    }
}
