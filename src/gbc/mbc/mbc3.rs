use super::Mbc;
use super::MbcInfo;

const MBC3_RAM_WRITE_ENABLE: u8 = 0x0a;

#[derive(Debug,Copy,Clone)]
struct Rtc {
    rtc_seconds: u8,
    rtc_minutes: u8,
    rtc_hours: u8,
    rtc_days_low: u8,
    rtc_days_high: u8,
}

#[derive(Debug)]
pub struct Mbc3 {
    ram_w: u8,
    rom_bank: u8,
    ram_bank: u8,
    ram: Option<Box<[u8]>>,
    rtc: Rtc,
    latched_rtc: Rtc,
    rtc_latch: u8,
}

impl Mbc3 {
    pub fn new(mbc_info: MbcInfo) -> Mbc3 {
        let rtc = Rtc {
            rtc_seconds: 0,
            rtc_minutes: 0,
            rtc_hours: 0,
            rtc_days_low: 0,
            rtc_days_high: 0,
        };
        Mbc3 {
            ram_w: 0,
            rom_bank: 0,
            ram_bank: 0,
            ram: if let Some(ram_info) = mbc_info.ram_info {
                Some(vec![0; ram_info.size as usize].into_boxed_slice())
            } else {
                None
            },
            rtc: rtc,
            latched_rtc: rtc,
            rtc_latch: 0,
        }
    }

    fn ram_offset(&self) -> u16 {
        self.ram_bank as u16 * 1024 * 8
    }
}

impl Mbc for Mbc3 {
    fn read(&self, rom: &Box<[u8]>, addr: u16) -> u8 {
        match addr {
            0x0000...0x3fff => rom[addr as usize],
            0x4000...0x7fff => {
                let addr = addr as u32 - 0x4000 + (self.rom_bank as u32) * 0x4000;
                rom[addr as usize]
            }
            _ => panic!("Address out of range 0x{:x}", addr),
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000...0x1fff => self.ram_w = val,
            0x2000...0x3fff => self.rom_bank = val,
            0x4000...0x5fff => self.ram_bank = val,
            0x6000...0x7fff => {
                if self.rtc_latch == 0 && val == 1 {
                    self.latched_rtc = self.rtc.clone()
                }
                self.rtc_latch = val
            }
            _ => panic!("Illegal address 0x{:x}", addr),
        }
    }

    fn read_ram(&self, addr: u16) -> u8 {
        if self.ram_w == MBC3_RAM_WRITE_ENABLE {
            match self.ram_bank {
                0...3 => {
                    if let Some(ref ram) = self.ram {
                        ram[(addr - 0xa000 + self.ram_offset()) as usize]
                    } else {
                        0
                    }
                }
                0x08 => self.latched_rtc.rtc_seconds,
                0x09 => self.latched_rtc.rtc_minutes,
                0x0a => self.latched_rtc.rtc_hours,
                0x0b => self.latched_rtc.rtc_days_low,
                0x0c => self.latched_rtc.rtc_days_high,
                _ => panic!("Illegal ram bank: {:?}", self.ram_bank),
            }
        } else {
            0
        }
    }

    fn write_ram(&mut self, addr: u16, val: u8) {
        if self.ram_w == MBC3_RAM_WRITE_ENABLE {
            match self.ram_bank {
                0...3 => {
                    let ram_bank_offset = self.ram_offset();
                    if let Some(ref mut ram) = self.ram {
                        ram[(addr - 0xa000 + ram_bank_offset) as usize] = val
                    }
                }
                0x08 => self.rtc.rtc_seconds = val & 0x3f,
                0x09 => self.rtc.rtc_minutes = val & 0x3f,
                0x0a => self.rtc.rtc_hours = val & 0x1f,
                0x0b => self.rtc.rtc_days_low = val,
                0x0c => self.rtc.rtc_days_high = val & 0b1100_0001,
                _ => panic!("Illegal ram bank: {:?}", self.ram_bank),
            }
        }
    }
}
