use super::Interrupt;

bitflags! {
	flags LCDCtrl: u8 {
		const LCD_DISPLAY_ENABLE = 0b1000_0000,
		const WINDOW_TILE_MAP_DISPLAY_SELECT = 0b0100_0000,
		const WINDOW_DISPLAY_ENABLE = 0b0010_0000,
		const BG_WINDOW_TILE_DATE_SELECT = 0b0001_0000,
		const BG_TILE_MAP_DISPLAY_SELECT = 0b0000_1000,
		const OBJ_SIZE = 0b0000_0100,
		const OBJ_DISPLAY_ENABLE = 0b0000_0010,
		const BG_DISPLAY = 0b0000_0001,
	}
}

impl LCDCtrl {
    fn new() -> LCDCtrl {
        // Value at reset is 0x91
        LCD_DISPLAY_ENABLE | BG_WINDOW_TILE_DATE_SELECT | BG_DISPLAY
    }
}

#[allow(dead_code)]
const CLKS_SCREEN_REFRESH: u32 = 70224;
#[allow(dead_code)]
const HBLANK_CLKS: u32 = 456;
#[allow(dead_code)]
const VBLANK_CLKS: u32 = 4560;

const VRAM_SIZE: usize = 1024 * 16;
const OAM_SIZE: usize = 40 * 4; // 40 OBJs - 32 bits

pub struct Ppu {
    lcdc: LCDCtrl,
    scx: u8,
    scy: u8,
    ly: u8,
    bgp: u8, // Background palette data
    obp_0: u8, // Object palette 0 data
    obp_1: u8, // Object palette 1 data
    window_y: u8,
    window_x: u8,
    bgpi: u8,
    bgpd: u8,
    vbk: u8,
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            lcdc: LCDCtrl::new(),
            scx: 0,
            scy: 0,
            ly: 0,
            window_y: 0,
            window_x: 0,
            bgp: 0xfc,
            obp_0: 0xff,
            obp_1: 0xff,
            bgpi: 0x00,
            bgpd: 0x00,
            vbk: 0,
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x8000...0x9fff => {
                let addr = addr - 0x8000;
                let offset = self.vbk_offset();
                self.vram[(addr + offset) as usize] = val
            }
            0xfe00...0xfe9f => self.oam[(addr - 0xfe00) as usize] = val,
            0xff40 => self.lcdc.bits = val,
            0xff42 => self.scy = val,
            0xff43 => self.scx = val,
            0xff44 => self.ly = val,
            0xff47 => self.bgp = val,
            0xff48 => self.obp_0 = val,
            0xff49 => self.obp_1 = val,
            0xff4a => self.window_y = val,
            0xff4b => self.window_x = val,
            0xff4f => self.vbk = val,
            0xff68 => self.bgpi = val,
            0xff69 => self.bgpd = val,
            _ => panic!("Write not implmented for 0x{:x}", addr),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000...0x9fff => {
                let addr = addr - 0x8000;
                let offset = self.vbk_offset();
                self.vram[(addr + offset) as usize]
            }
            0xfe00...0xfe9f => self.oam[(addr - 0xfe00) as usize],
            0xff40 => self.lcdc.bits,
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.ly,
            0xff47 => self.bgp,
            0xff48 => self.obp_0,
            0xff49 => self.obp_1,
            0xff4a => self.window_y,
            0xff4b => self.window_x,
            0xff4f => self.vbk,
            0xff68 => self.bgpi,
            0xff69 => self.bgpd,
            _ => panic!("Read not implmented for 0x{:x}", addr),
        }
    }

    #[allow(unused_variables)]
    pub fn cycle_flush(&mut self, cycle_count: u32) -> Option<Interrupt> {
        None
    }

    fn vbk_offset(&self) -> u16 {
        (self.vbk | 0x01) as u16 * 0x2000
    }
}
