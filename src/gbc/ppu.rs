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

    fn is_set(self, flag: LCDCtrl) -> bool {
        self.intersects(flag)
    }
}

bitflags! {
    flags LCDStat: u8 {
        const LYC_LY_INTERRUPT = 0b0100_0000,
        const OAM_INTERRUPT = 0b0010_0000,
        const VBLANK_INTERRUPT = 0b0001_0000,
        const HBLANK_INTERRUPT = 0b0000_1000,
        const COINCIDENCE_FLAG = 0b0000_0100,
        const MODE_FLAG = 0b0000_0011,
    }
}

impl LCDStat {
    fn new() -> LCDStat {
        LCDStat { bits: 0 }
    }

    fn is_set(self, flag: LCDStat) -> bool {
        self.intersects(flag)
    }
}

#[derive(Debug)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

const WHITE: Color = Color {
    r: 175,
    g: 203,
    b: 70,
    a: 255,
};
const LIGHT_GRAY: Color = Color {
    r: 121,
    g: 170,
    b: 109,
    a: 255,
};
const DARK_GRAY: Color = Color {
    r: 34,
    g: 111,
    b: 95,
    a: 255,
};
const BLACK: Color = Color {
    r: 8,
    g: 41,
    b: 85,
    a: 255,
};

impl Color {
    fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r: r,
            g: g,
            b: b,
            a: a,
        }
    }
}

pub const OAM_SIZE: usize = 0x100; // 40 OBJs - 32 bits

#[allow(dead_code)]
const CLKS_SCREEN_REFRESH: u32 = 70224;
#[allow(dead_code)]
const DISPLAY_WIDTH: usize = 160;
#[allow(dead_code)]
const DISPLAY_HEIGHT: usize = 144;

pub const FRAMEBUFFER_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT * 4;

const VRAM_SIZE: usize = 1024 * 16;

const MODE_HBLANK: u32 = 0;
const MODE_VBLANK: u32 = 1;
const MODE_OAM: u32 = 2;
const MODE_VRAM: u32 = 3;

const HBLANK_CYCLES: u32 = 204;
const VBLANK_CYCLES: u32 = 456;
const OAM_CYCLES: u32 = 80;
const VRAM_CYCLES: u32 = 172;

pub struct Ppu {
    lcdc: LCDCtrl,
    lcdstat: LCDStat,
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
    vram: Box<[u8]>,
    oam: Box<[u8]>,
    framebuffer: Box<[u8]>,
    mode_cycles: u32,
    mode: u32,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            lcdc: LCDCtrl::new(),
            lcdstat: LCDStat::new(),
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
            vram: vec![0; VRAM_SIZE].into_boxed_slice(),
            oam: vec![0; OAM_SIZE].into_boxed_slice(),
            framebuffer: vec![0; FRAMEBUFFER_SIZE].into_boxed_slice(),
            mode_cycles: 0,
            mode: 0,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x8000...0x9fff => {
                let addr = addr - 0x8000;
                let offset = self.vbk_offset();
                self.vram[(addr + offset) as usize] = val
            }
            0xfe00...0xfeff => self.oam[(addr - 0xfe00) as usize] = val,
            0xff40 => self.lcdc.bits = val,
            0xff41 => self.lcdstat.bits = val,
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
            0xfe00...0xfeff => self.oam[(addr - 0xfe00) as usize],
            0xff40 => self.lcdc.bits,
            0xff41 => self.lcdstat.bits,
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

        self.mode_cycles = self.mode_cycles + cycle_count;

        if self.lcdc.is_set(LCD_DISPLAY_ENABLE) {

            let cycles = self.mode_cycles;
            let mode = self.mode;

            let mut int: Option<Interrupt> = None;

            match mode {
                MODE_HBLANK => {
                    if cycles >= HBLANK_CYCLES {
                        self.ly = self.ly + 1;
                        self.mode = if self.ly == 143 {
                            int = Some(Interrupt::VBlank);
                            MODE_VBLANK
                        } else {
                            MODE_OAM
                        };
                        self.mode_cycles = cycles - HBLANK_CYCLES
                    }
                }

                MODE_VBLANK => {
                    if cycles >= VBLANK_CYCLES {
                        self.ly = self.ly + 1;
                        if self.ly > 153 {
                            self.ly = 0;
                            self.mode = MODE_OAM
                        }
                        self.mode_cycles = cycles - VBLANK_CYCLES
                    }
                }

                MODE_OAM => {
                    if cycles >= OAM_CYCLES {
                        self.mode = MODE_VRAM;
                        self.mode_cycles = cycles - OAM_CYCLES
                    }
                }

                MODE_VRAM => {
                    if cycles >= VRAM_CYCLES {
                        self.mode = MODE_HBLANK;
                        self.mode_cycles = cycles - VRAM_CYCLES;
                        self.draw_scanline()
                    }
                }
                _ => panic!("Invalid PPU mode!"),
            }

            return int;
        }
        None
    }

    pub fn oam_dma_transfer(&mut self, oam: Box<[u8]>) {
        self.oam = oam
    }

    fn vbk_offset(&self) -> u16 {
        (self.vbk | 0x01) as u16 * 0x2000
    }

    fn draw_scanline(&mut self) {
        if self.lcdc.is_set(BG_DISPLAY) {
            self.render_tiles()
        }

        if self.lcdc.is_set(OBJ_DISPLAY_ENABLE) {
            self.render_sprites()
        }
    }

    fn render_tiles(&mut self) {

        let scanline = self.ly;

        let scroll_y = self.scy;
        let scroll_x = self.scx;
        let window_y = self.window_y;
        let window_x = self.window_x;

        // Is the window enabled and visible on the current scanline?
        let using_window = if self.lcdc.is_set(WINDOW_DISPLAY_ENABLE) {
            window_x <= scanline
        } else {
            false
        };

        // What region do we read tile data from, and is the tile identifier signed?
        let (tile_offset, signed_id): (u16, bool) = if self.lcdc
            .is_set(BG_WINDOW_TILE_DATE_SELECT) {
            (0x8000, false)
        } else {
            (0x8800, true)
        };

        // What background region to use?
        let background_offset: u16 = if using_window {
            if self.lcdc.is_set(WINDOW_TILE_MAP_DISPLAY_SELECT) {
                0x9c00
            } else {
                0x9800
            }
        } else {
            if self.lcdc.is_set(BG_TILE_MAP_DISPLAY_SELECT) {
                0x9c00
            } else {
                0x9800
            }
        };

        let y = if using_window {
            scanline - window_y
        } else {
            scroll_y + scanline
        };

        let tile_row: u16 = (y as u16 / 8) * 32;
        for i in 0..160 {

            let x = if using_window && i >= window_x {
                i - window_x
            } else {
                i + scroll_x
            };

            let tile_col: u16 = x as u16 / 8;

            let tile_address = background_offset + tile_row + tile_col;

            let tile_id = self.read(tile_address);

            let tile_location = if signed_id {
                tile_offset + (tile_id as u16 * 16)
            } else {
                tile_offset + ((tile_id as u16 + 128) * 16)
            };

            let line = ((y % 8) * 2) as u16;
            let t1 = self.read(tile_location + line);
            let t2 = self.read(tile_location + line + 1);

            let color_bit = (((x as i32) % 8) - 7) * -1;

            let color_id = ((t2 >> color_bit) & 0b1) << 1;
            let color_id = color_id | ((t1 >> color_bit) & 0b1);

            let color = self.get_color(color_id, 0xff47);
            self.set_pixel(i as u32, scanline as u32, color)
        }
    }

    fn render_sprites(&mut self) {}

    fn get_color(&self, color_id: u8, addr: u16) -> Color {

        let palette = self.read(addr);

        let (hi, lo) = match color_id {
            0 => (1, 0),
            1 => (3, 2),
            2 => (5, 3),
            3 => (7, 6),
            _ => panic!("Invalid color id: 0x{:x}", color_id),
        };

        let color = ((palette >> hi) & 0b1) << 1;
        let color = color | ((palette >> lo) & 0b1);

        match color {
            0 => WHITE,
            1 => LIGHT_GRAY,
            2 => DARK_GRAY,
            3 => BLACK,
            _ => panic!("Invalid color: 0x{:x}", color),
        }
    }

    fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        let offset = (((y * 160) + x) * 4) as usize;
        self.framebuffer[offset + 0] = color.r;
        self.framebuffer[offset + 1] = color.g;
        self.framebuffer[offset + 2] = color.b;
        self.framebuffer[offset + 3] = color.a;
    }
}
