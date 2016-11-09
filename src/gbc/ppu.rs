use super::Interrupt;

use std::sync::mpsc::Sender;

bitflags! {
	flags LCDCtrl: u8 {
		const LCD_DISPLAY_ENABLE = 0b1000_0000,
		const WINDOW_TILE_MAP_DISPLAY_SELECT = 0b0100_0000,
		const WINDOW_DISPLAY_ENABLE = 0b0010_0000,
		const BG_WINDOW_TILE_DATA_SELECT = 0b0001_0000,
		const BG_TILE_MAP_DISPLAY_SELECT = 0b0000_1000,
		const OBJ_SIZE = 0b0000_0100,
		const OBJ_DISPLAY_ENABLE = 0b0000_0010,
		const BG_DISPLAY = 0b0000_0001,
	}
}

impl LCDCtrl {
    fn new() -> LCDCtrl {
        // Value at reset is 0x91
        LCD_DISPLAY_ENABLE | BG_WINDOW_TILE_DATA_SELECT | BG_DISPLAY
    }

    fn is_set(self, flag: LCDCtrl) -> bool {
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
    r: 248,
    g: 232,
    b: 200,
    a: 255,
};
const LIGHT_GRAY: Color = Color {
    r: 216,
    g: 144,
    b: 72,
    a: 255,
};
const DARK_GRAY: Color = Color {
    r: 168,
    g: 52,
    b: 32,
    a: 255,
};
const BLACK: Color = Color {
    r: 48,
    g: 24,
    b: 80,
    a: 255,
};

struct LCDStat {
    lyc_ly_interrupt: bool,
    oam_interrupt: bool,
    vblank_interrupt: bool,
    hblank_interrupt: bool,
    coincidence_flag: bool,
    mode: Mode,
}

impl LCDStat {
    fn new() -> LCDStat {
        LCDStat {
            lyc_ly_interrupt: false,
            oam_interrupt: false,
            vblank_interrupt: false,
            hblank_interrupt: false,
            coincidence_flag: false,
            mode: Mode::VBlank,
        }
    }

    fn get_flags(&self) -> u8 {
        let mut flags: u8 = 0;
        if self.lyc_ly_interrupt {
            flags |= 0b0100_0000
        }
        if self.oam_interrupt {
            flags |= 0b0010_0000
        }
        if self.vblank_interrupt {
            flags |= 0b0001_0000
        }
        if self.hblank_interrupt {
            flags |= 0b0000_1000
        }
        if self.coincidence_flag {
            flags |= 0b0000_0100
        }
        flags |= self.mode.get_flag();
        flags
    }

    fn set_flags(&mut self, flags: u8) {
        self.lyc_ly_interrupt = (flags & 0b0100_0000) != 0;
        self.oam_interrupt = (flags & 0b0010_0000) != 0;
        self.vblank_interrupt = (flags & 0b0001_0000) != 0;
        self.hblank_interrupt = (flags & 0b0000_1000) != 0;
        // These are readonly
        // self.coincidence_flag = (flags & 0b0000_0100) != 0;
        // self.mode = Mode::from_flags(flags)
        //
    }
}

#[derive(Debug,Clone,Copy)]
enum Mode {
    HBlank,
    VBlank,
    Oam,
    VRam,
}

impl Mode {
    fn get_flag(self) -> u8 {
        let f = match self {
            Mode::HBlank => MODE_HBLANK,
            Mode::VBlank => MODE_VBLANK,
            Mode::Oam => MODE_OAM,
            Mode::VRam => MODE_VRAM,
        };
        f as u8
    }
}

pub const OAM_SIZE: usize = 0x100; // 40 OBJs - 32 bits

const FRAMEBUFFER_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT * 4;

const CLKS_SCREEN_REFRESH: u32 = 70224;
const DISPLAY_WIDTH: usize = 160;
const DISPLAY_HEIGHT: usize = 144;

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
    lyc: u8,
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
    framebuffer_channel: Sender<Box<[u8]>>,
    cycles: u32,
}

impl Ppu {
    pub fn new(framebuffer_channel: Sender<Box<[u8]>>) -> Ppu {
        Ppu {
            lcdc: LCDCtrl::new(),
            lcdstat: LCDStat::new(),
            scx: 0,
            scy: 0,
            ly: 144,
            lyc: 0xff,
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
            framebuffer_channel: framebuffer_channel,
            cycles: 0,
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
            0xff41 => self.lcdstat.set_flags(val),
            0xff42 => self.scy = val,
            0xff43 => self.scx = val,
            0xff44 => self.ly = val,
            0xff45 => self.lyc = val,
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
            0xff41 => self.lcdstat.get_flags(),
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.ly,
            0xff45 => self.lyc,
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
    pub fn cycle_flush(&mut self, cycle_count: u32) -> u8 {
        self.mode_cycles += cycle_count;

        let mut interrupt = 0;

        if self.lcdc.is_set(LCD_DISPLAY_ENABLE) {

            self.cycles += cycle_count;
            let cycles = self.mode_cycles;

            match self.lcdstat.mode {
                Mode::HBlank => {
                    if cycles >= HBLANK_CYCLES {
                        self.mode_cycles -= HBLANK_CYCLES;

                        if self.lcdstat.lyc_ly_interrupt {
                            let cmp = self.ly == self.lyc;
                            self.lcdstat.coincidence_flag = cmp;
                            if cmp {
                                interrupt |= Interrupt::LCDStat.flag()
                            }
                        }

                        self.lcdstat.mode = if self.ly == 144 {

                            self.framebuffer_channel.send(self.framebuffer.clone()).unwrap();

                            interrupt |= Interrupt::VBlank.flag();

                            if self.lcdstat.vblank_interrupt {
                                interrupt |= Interrupt::LCDStat.flag()
                            }

                            self.cycles = 0;

                            Mode::VBlank
                        } else {

                            if self.lcdstat.hblank_interrupt {
                                interrupt |= Interrupt::LCDStat.flag()
                            }

                            self.draw_scanline();
                            Mode::Oam
                        };

                        self.ly = self.ly + 1;
                    }
                }

                Mode::VBlank => {
                    if cycles >= VBLANK_CYCLES {
                        self.mode_cycles -= VBLANK_CYCLES;

                        if self.lcdstat.lyc_ly_interrupt {
                            let cmp = self.ly == self.lyc;
                            self.lcdstat.coincidence_flag = cmp;
                            if cmp {
                                interrupt |= Interrupt::LCDStat.flag()
                            }
                        }

                        self.ly = self.ly + 1;

                        if self.ly == 154 {
                            self.lcdstat.mode = Mode::Oam;
                            self.ly = 0;

                            if self.lcdstat.oam_interrupt {
                                interrupt |= Interrupt::LCDStat.flag()
                            }

                        }
                    }
                }

                Mode::Oam => {
                    if cycles >= OAM_CYCLES {
                        self.mode_cycles -= OAM_CYCLES;
                        self.lcdstat.mode = Mode::VRam
                    }
                }

                Mode::VRam => {
                    if cycles >= VRAM_CYCLES {
                        self.mode_cycles -= VRAM_CYCLES;
                        self.lcdstat.mode = Mode::HBlank
                    }
                }
            }
        } else {
            if self.mode_cycles >= CLKS_SCREEN_REFRESH {
                self.mode_cycles -= CLKS_SCREEN_REFRESH
            }
        }
        interrupt
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
        let window_x = self.window_x.wrapping_sub(7);

        let using_window = if self.lcdc.is_set(WINDOW_DISPLAY_ENABLE) {
            window_y <= scanline
        } else {
            false
        };

        let (tile_data, unsigned): (u16, bool) = if self.lcdc.is_set(BG_WINDOW_TILE_DATA_SELECT) {
            (0x8000, true)
        } else {
            (0x8800, false)
        };

        let background_mem = if using_window {
            if self.lcdc.is_set(BG_TILE_MAP_DISPLAY_SELECT) {
                0x9c00
            } else {
                0x9800
            }
        } else {
            if self.lcdc.is_set(WINDOW_TILE_MAP_DISPLAY_SELECT) {
                0x9c00
            } else {
                0x9800
            }
        };

        let y_pos = if using_window {
            scanline.wrapping_sub(window_y)
        } else {
            scroll_y.wrapping_add(scanline)
        };

        let tile_row = (y_pos / 8) as u16 * 32;

        for pixel in 0..160 {

            let x_pos = if using_window && pixel >= window_x {
                pixel - window_x
            } else {
                pixel + scroll_x
            };

            let tile_col = (x_pos / 8) as u16;

            let tile_address = background_mem + tile_row + tile_col;

            let tile_num: i16 = if unsigned {
                self.read(tile_address) as i16
            } else {
                self.read(tile_address) as i8 as i16
            };

            let tile_location = tile_data +
                                if unsigned {
                (tile_num * 16) as u16
            } else {
                ((tile_num + 128) * 16) as u16
            };

            let line = (y_pos as u16 % 8) * 2;
            let data1 = self.read(tile_location + line);
            let data2 = self.read(tile_location + line + 1);

            let color_bit = ((x_pos as i32 % 8) - 7) * -1;

            let color_num = ((data2 >> color_bit) & 0b1) << 1;
            let color_num = color_num | ((data1 >> color_bit) & 0b1);

            let color = self.get_color(color_num, 0xff47);
            self.set_pixel(pixel as u32, scanline as u32, color)
        }


    }

    fn render_sprites(&mut self) {

        let use_8x16 = self.lcdc.is_set(OBJ_SIZE);

        for sprite in 0..40 {

            let index: u8 = sprite * 4;
            let y_pos = self.oam[index as usize].wrapping_sub(16);
            let x_pos = self.oam[(index + 1) as usize].wrapping_sub(8);
            let tile_location = self.oam[(index + 2) as usize];
            let attributes = self.oam[(index + 3) as usize];

            let y_flip = (attributes & 0x40) != 0;
            let x_flip = (attributes & 0x20) != 0;

            let scanline = self.ly;

            let y_size = if use_8x16 { 16 } else { 8 };

            if scanline >= y_pos && scanline < (y_pos + y_size) {
                let line = (scanline - y_pos) as i32;
                let line = if y_flip {
                    (line - y_size as i32) * -1
                } else {
                    line
                };

                let line = line * 2;

                let data_address = (0x8000 + (tile_location as u16 * 16)) as i32 + line;
                let data_address = data_address as u16;

                let data1 = self.read(data_address);
                let data2 = self.read(data_address + 1);

                for tile_pixel in 8..0 {

                    let color_bit = tile_pixel as i32;
                    let color_bit = if x_flip {
                        (color_bit - 7) * -1
                    } else {
                        color_bit
                    };

                    let color_num = ((data2 >> color_bit) & 0b1) << 1;
                    let color_num = color_num | ((data1 >> color_bit) & 0b1);

                    let color_address = if (attributes & 0x10) != 0 {
                        0xff49
                    } else {
                        0xff48
                    };

                    let color = self.get_color(color_num, color_address);


                    let x_pix = 0 - tile_pixel;
                    let x_pix = x_pix + 7;

                    let pixel = x_pos + x_pix;

                    self.set_pixel(pixel as u32, scanline as u32, color)

                }

            }

        }

    }

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
        self.framebuffer[offset + 0] = color.a;
        self.framebuffer[offset + 1] = color.r;
        self.framebuffer[offset + 2] = color.g;
        self.framebuffer[offset + 3] = color.b;
    }
}
