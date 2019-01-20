use super::ppu::{VideoSink,Ppu};
use super::spu::Spu;
use super::cpu::Cpu;
use super::gamepad::{InputEvent,Gamepad};
use super::cart::Cart;
use super::GameboyType;
use super::interconnect::Interconnect;

pub struct Console {
    cpu: Cpu,
}

impl Console {
    pub fn new(cart: Cart) -> Console {
        let gb_type = GameboyType::Dmg;
        let interconnect = Interconnect::new(
            gb_type,
            cart,
            Ppu::new(),
            Spu::new(),
            Gamepad::new());
        Console {
            cpu: Cpu::new(gb_type, interconnect)
        }
    }

    pub fn run_for_one_frame(&mut self, video_sink: &mut dyn VideoSink) {
        let mut frame_handler = FrameHandler::new(video_sink);
        while !frame_handler.frame_available {
            self.cpu.step(&mut frame_handler);
        }
    }

    pub fn handle_event(&mut self, input_event: InputEvent) {
        self.cpu.interconnect.gamepad.handle_event(input_event)
    }

    pub fn copy_cart_ram(&self) -> Option<Box<[u8]>> {
        self.cpu.interconnect.cart.copy_ram()
    }
}

struct FrameHandler<'a> {
    frame_available: bool,
    video_sink: &'a mut dyn VideoSink,
}

impl<'a> FrameHandler<'a> {
    fn new(video_sink: &'a mut dyn VideoSink) -> FrameHandler<'a> {
        FrameHandler {
            frame_available: false,
            video_sink,
        }
    }
}

impl<'a> VideoSink for FrameHandler<'a> {
    fn frame_available(&mut self, frame: &Box<[u32]>) {
        self.video_sink.frame_available(frame);
        self.frame_available = true
    }
}