use super::display::Display;
use super::cart::Cart;
use super::ram::Ram;

const ZRAM_SIZE: usize = 0x7f;

pub struct Interconnect {
    cart: Cart,
    display: Display,
    ram: Ram,
    zram: [u8; ZRAM_SIZE],
}

impl Interconnect {
    pub fn new(cart: Cart, display: Display) -> Interconnect {
        Interconnect {
            cart: cart,
            display: display,
            ram: Ram::new(),
            zram: [0; ZRAM_SIZE],
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {

            0x0000...0x3fff => self.cart.bytes[address as usize],

            0xc000...0xdfff => self.ram.read(address),

            // Speedswitch
            0xff4d => 0,

            0xff40...0xff4b | 0xff51...0xff6b => self.display.read(address),

            0xff80...0xfffe => self.zram[(address - 0xff80) as usize],

            // Interrupt Enable
            0xffff => {
                // TODO
                0
            }

            _ => panic!("Read: address not in range: 0x{:x}", address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {

            0xc000...0xdfff => self.ram.write(address, value),

            // JOYPAD
            0xff00 => {
                // println!("Write to JOYPAD: 0x{:x}", value);
            }

            // Speedswitch
            0xff4d => {
                // TODO
            }

            0xff40...0xff4b | 0xff51...0xff6b => self.display.write(address, value),

            0xff80...0xfffe => self.zram[(address - 0xff80) as usize] = value,

            // Interrupt Enable
            0xffff => {
                // TODO
            }

            _ => {
                panic!("Write: address not in range: 0x{:x} - value: 0x{:x}",
                       address,
                       value)
            }
        }
    }
}
