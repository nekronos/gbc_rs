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
pub struct Display {
    
}

impl Display {
    pub fn new() -> Display {
        Display {}
    }

    pub fn read(&self, address: u16) -> u8 {
        panic!("Display read not implemented");
    }

    pub fn write(&mut self, address: u16, value: u8) {
        panic!("Display write not implemented");
    }
}
