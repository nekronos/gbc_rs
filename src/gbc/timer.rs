use std::u8;
use super::Interrupts;
use super::INT_TIMEROVERFLOW;

#[allow(dead_code)]
const DIV_INC_RATE_0: u32 = 16384;

#[allow(dead_code)]
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
    clock_rate: u32,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            div: 0,
            div_cycles: 0,
            tima: 0,
            tima_cycles: 0,
            tma: 0,
            enabled: false,
            clock_select: 0,
            clock_rate: CLOCKS[0],
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
                self.enabled = (val & 0b100) != 0;
                self.clock_rate = CLOCKS[self.clock_select as usize]
            }

            _ => panic!("Address not in range 0x{:x}", addr),
        }
    }

    pub fn cycle_flush(&mut self, cycle_count: u32) -> Interrupts {
        self.flush_div(cycle_count);

        if self.flush_tima(cycle_count) {
            INT_TIMEROVERFLOW
        } else {
            Interrupts::empty()
        }
    }

    fn flush_tima(&mut self, cycle_count: u32) -> bool {
        let tima_cycles = self.tima_cycles + cycle_count;
        let rate = self.clock_rate;
        let ticks = tima_cycles / rate;

        self.tima_cycles = tima_cycles - rate * ticks;

        if self.enabled {
            let (tima, overflow) = self.tima.overflowing_add(ticks as u8);
            self.tima = if overflow {
                self.tma.wrapping_add(tima)
            } else {
                tima
            };
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
