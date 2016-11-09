use super::Interrupt;

use std::sync::mpsc::Receiver;

#[derive(Debug)]
pub enum ButtonState {
    Up,
    Down,
}

#[derive(Debug,Copy,Clone)]
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

impl Button {
    fn flag(self) -> u8 {
        use self::Button::*;
        match self {
            Right | A => 0b0001,
            Left | B => 0b0010,
            Up | Select => 0b0100,
            Down | Start => 0b1000,
        }
    }
}

#[derive(Debug)]
pub struct InputEvent {
    button: Button,
    state: ButtonState,
}

impl InputEvent {
    pub fn new(button: Button, state: ButtonState) -> InputEvent {
        InputEvent {
            button: button,
            state: state,
        }
    }
}

pub struct Gamepad {
    input_port_1: u8,
    input_port_2: u8,
    port: u8,
    input: Receiver<InputEvent>,
}

impl Gamepad {
    pub fn new(input: Receiver<InputEvent>) -> Gamepad {
        Gamepad {
            input_port_1: 0x0f,
            input_port_2: 0x0f,
            input: input,
            port: 0xf0,
        }
    }

    pub fn read(&mut self) -> u8 {

        while let Ok(event) = self.input.try_recv() {
            self.handle_event(event)
        }

        let mut input = self.port | 0b1100_0000;

        if (self.port & 0x10) != 0 {
            input |= self.input_port_2 & 0x0f
        }

        if (self.port & 0x20) != 0 {
            input |= self.input_port_1 & 0x0f
        }

        input
    }

    pub fn write(&mut self, val: u8) {
        self.port = val & 0b0011_0000
    }

    pub fn cycle_flush(&mut self, cycle_count: u32) -> Option<Interrupt> {
        None
    }

    fn handle_event(&mut self, event: InputEvent) {
        use self::Button::*;

        println!("Handle event: {:?}", event);

        match event.state {
            ButtonState::Down => {
                let mask = !event.button.flag();
                match event.button {
                    Up | Down | Left | Right => self.input_port_1 = self.input_port_1 & mask,
                    A | B | Start | Select => self.input_port_2 = self.input_port_2 & mask,
                }
            }

            ButtonState::Up => {
                let flag = event.button.flag();
                match event.button {
                    Up | Down | Left | Right => self.input_port_1 = self.input_port_1 | flag,
                    A | B | Start | Select => self.input_port_2 = self.input_port_2 | flag,
                }

            }
        }
        // println!("Handle event: {:?}", event);
    }
}
