use std::u8;
use super::CpuClock;
use super::Interrupt;

const DIV_INC_RATE_0: u32 = 16384;
const DIV_INC_RATE_1: u32 = 32768;

const CLOCKS: [u32; 4] = [1024, 16, 64, 256];

#[derive(Debug)]
pub struct Timer {
    div: u8,
    div_cycles: u8,
    tima: u8,
    tima_cycles: u32,
    tma: u8,
    enabled: bool,
    clock_select: u8,
    cpu_clock: CpuClock,
}

impl Timer {
    pub fn new() -> Timer {
        let tima_inc_rate = CpuClock::Normal.value() / CLOCKS[0];
        Timer {
            div: 0,
            div_cycles: 0,
            tima: 0,
            tima_cycles: 0,
            tma: 0,
            enabled: false,
            clock_select: 0,
            cpu_clock: CpuClock::Normal,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff04 => self.div,
            0xff05 => self.tima,
            0xff06 => self.tma,
            0xff07 => (self.clock_select & 0b11) | if self.enabled { 0b100 } else { 0 },

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
                self.enabled = (val & 0b100) != 0
            }

            _ => panic!("Address not in range 0x{:x}", addr),
        }
    }

    pub fn cycle_flush(&mut self, cycle_count: u32) -> Option<Interrupt> {
        self.flush_div(cycle_count);

        if self.flush_tima(cycle_count) {
            Some(Interrupt::TimerOverflow)
        } else {
            None
        }
    }

    pub fn set_cpu_clock(&mut self, clock: CpuClock) {}

    fn flush_tima(&mut self, cycle_count: u32) -> bool {
        self.tima_cycles = self.tima_cycles + cycle_count;

        let cycles = self.tima_cycles;
        let rate = CLOCKS[self.clock_select as usize];

        let tick = cycles >= rate;

        if tick {
            self.tima_cycles = cycles - rate;
        }

        if self.enabled && tick {
            let (tima, overflow) = self.tima.overflowing_add(1);
            self.tima = if overflow { self.tma } else { tima };
            overflow
        } else {
            false
        }
    }

    fn flush_div(&mut self, cycle_count: u32) {
        let div_ticks = cycle_count >> 8;

        self.div = self.div.wrapping_add(div_ticks as u8);

        let div_inc_ticks = (cycle_count - (div_ticks << 8)) as u8;
        let (div_cycles, overflow) = self.div_cycles.overflowing_add(div_inc_ticks);

        self.div_cycles = div_cycles;

        if overflow {
            self.div = self.div.wrapping_add(1)
        }
    }
}
