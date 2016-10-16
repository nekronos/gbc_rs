use std::u8;
use super::CpuSpeed;
use super::Interrupt;

const DIV_INC_RATE_0: u32 = 16384;
const DIV_INC_RATE_1: u32 = 32768;

const CLOCKS: [u32; 4] = [1024, 16, 64, 256];

#[derive(Debug)]
pub struct Timer {
    div: u8,
    div_inc: u8,
    tima: u8,
    tma: u8,
    timer_enable: bool,
    clock_select: u8,
    tick_rate: u32,
    cpu_speed: CpuSpeed,
    tima_ticks: u32,
}

impl Timer {
    pub fn new() -> Timer {
        let tick_rate = CpuSpeed::Normal.value() / CLOCKS[0];
        Timer {
            div: 0,
            div_inc: 0,
            tima: 0,
            tma: 0,
            timer_enable: false,
            clock_select: 0,
            tick_rate: tick_rate,
            cpu_speed: CpuSpeed::Normal,
            tima_ticks: 0,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff04 => self.div,
            0xff05 => self.tima,
            0xff06 => self.tma,
            0xff07 => self.clock_select | if self.timer_enable { 0b100 } else { 0 },

            _ => panic!("Address not in range 0x{:x}", addr),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xff04 => self.div = 0,
            0xff05 => self.tima = val,
            0xff06 => self.tma = val,
            0xff07 => {
                self.clock_select = val & 0b11;
                self.timer_enable = (val & 0b100) != 0;

                let cpu_speed = self.cpu_speed.value();
                let clock = CLOCKS[self.clock_select as usize];
                let tick_rate = cpu_speed / clock;

                self.tick_rate = tick_rate
            }

            _ => panic!("Address not in range 0x{:x}", addr),
        }
    }

    pub fn cycle_flush(&mut self, cycle_count: u32) -> Option<Interrupt> {
        self.flush_div(cycle_count);
        if self.timer_enable {
            if self.flush_tima(cycle_count) {

            }
        }
        None
    }

    pub fn set_cpu_speed(&mut self, speed: CpuSpeed) {}

    fn flush_tima(&mut self, cycle_count: u32) -> bool {
        false
    }

    fn flush_div(&mut self, cycle_count: u32) {
        let div_ticks = cycle_count >> 8;

        self.div = self.div.wrapping_add(div_ticks as u8);

        let div_inc_ticks = (cycle_count - (div_ticks << 8)) as u8;
        let (div_inc, overflow) = self.div_inc.overflowing_add(div_inc_ticks);

        self.div_inc = div_inc;

        if overflow {
            self.div = self.div.wrapping_add(1)
        }
    }
}
