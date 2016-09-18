// const LCD_DISPLAY_ENABLE: u8 = 0b1000_0000;
// const WINDOW_TILE_MAP_DISPLAY_SELECT: u8 = 0b0100_0000;
// const WINDOW_DISPLAY_ENABLE: u8 = 0b0010_0000;
// const BG_WINDOW_TILE_DATE_SELECT: u8 = 0b0001_0000;
// const BG_TILE_MAP_DISPLAY_SELECT: u8 = 0b0000_1000;
// const OBJ_SIZE: u8 = 0b0000_0100;
// const OBJ_DISPLAY_ENABLE: u8 = 0b0000_0010;
// const BG_DISPLAY: u8 = 0b0000_0001;
//
#[derive(Debug)]
pub struct Ppu {
    scx: u8,
    scy: u8,
    window_y: u8,
    window_x: u8,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            scx: 0,
            scy: 0,
            window_y: 0,
            window_x: 0,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xff42 => self.scy = val,
            0xff43 => self.scx = val,
            0xff4a => self.window_y = val,
            0xff4b => self.window_x = val,
            _ => panic!("Write not implmented for 0x{:x}", addr),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff4a => self.window_y,
            0xff4b => self.window_x,
            _ => panic!("Read not implmented for 0x{:x}", addr),
        }
    }
}
