use super::Mbc;
use super::MbcInfo;

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
    ram_write_protected: bool,
    rom_bank: u8,
    ram_bank: u8,
    rtc_latch: u8,
    rtc: Rtc,
    latched_rtc: Rtc,
    rom_offset: usize,
    ram_offset: usize,
    ram: Box<[u8]>,
}

impl Mbc3 {
    pub fn new(mbc_info: MbcInfo, ram: Option<Box<[u8]>>) -> Mbc3 {
        let ram = if let Some(ram_info) = mbc_info.ram_info {
            ram_info.make_ram(ram)
        } else {
            vec![0; 0].into_boxed_slice()
        };
        let rtc = Rtc {
            rtc_seconds: 0,
            rtc_minutes: 0,
            rtc_hours: 0,
            rtc_days_low: 0,
            rtc_days_high: 0,
        };
        Mbc3 {
            ram_write_protected: true,
            rom_bank: 0,
            ram_bank: 0,
            rtc_latch: 0,
            rtc: rtc,
            latched_rtc: rtc,
            rom_offset: 0,
            ram_offset: 0,
            ram: ram,
        }
    }

    fn update_rom_offset(&mut self) {
        let bank = if self.rom_bank == 0 {
            1
        } else {
            self.rom_bank & 0x7f
        } as usize;
        self.rom_offset = bank * 16 * 1024
    }

    fn update_ram_offset(&mut self) {
        self.ram_offset = self.ram_bank as usize * 8 * 1024
    }
}

impl Mbc for Mbc3 {
    fn read(&self, rom: &Box<[u8]>, addr: u16) -> u8 {
        match addr {
            0x0000...0x3fff => rom[addr as usize],
            0x4000...0x7fff => rom[addr as usize - 0x4000 + self.rom_offset],
            _ => panic!("Address out of range 0x{:x}", addr),
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000...0x1fff => self.ram_write_protected = val != 0x0a,
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
        self.update_rom_offset();
        self.update_ram_offset()
    }

    fn read_ram(&self, addr: u16) -> u8 {
        match self.ram_bank {
            0...3 => self.ram[addr as usize - 0xa000 + self.ram_offset],
            0x08 => self.latched_rtc.rtc_seconds,
            0x09 => self.latched_rtc.rtc_minutes,
            0x0a => self.latched_rtc.rtc_hours,
            0x0b => self.latched_rtc.rtc_days_low,
            0x0c => self.latched_rtc.rtc_days_high,
            _ => panic!("Illegal ram bank: {:?}", self.ram_bank),
        }
    }

    fn write_ram(&mut self, addr: u16, val: u8) {
        if !self.ram_write_protected {
            match self.ram_bank {
                0...3 => self.ram[addr as usize - 0xa000 + self.ram_offset] = val,
                0x08 => self.rtc.rtc_seconds = val & 0x3f,
                0x09 => self.rtc.rtc_minutes = val & 0x3f,
                0x0a => self.rtc.rtc_hours = val & 0x1f,
                0x0b => self.rtc.rtc_days_low = val,
                0x0c => self.rtc.rtc_days_high = val & 0b1100_0001,
                _ => panic!("Illegal ram bank: {:?}", self.ram_bank),
            }
        }
    }

    fn copy_ram(&self) -> Option<Box<[u8]>> {
        if self.ram.len() > 0 {
            Some(self.ram.clone())
        } else {
            None
        }
    }
}
