use super::cart::Cart;
use super::ram::Ram;

pub struct Interconnect<'a> {
    cart: &'a Cart,
    ram: Ram,
    high_ram: [u8; 126],
}

impl<'a> Interconnect<'a> {
    pub fn new(cart: &'a Cart) -> Interconnect {
        Interconnect {
            cart: cart,
            ram: Ram::new(),
            high_ram: [0; 126],
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {

            0x0000...0x3fff => self.cart.bytes[address as usize],

            0xc000...0xdfff => self.ram.read(address),

            // Speedswitch
            0xff4d => 0,

            0xff80...0xfffe => self.high_ram[(address - 0xff80) as usize],

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

            0xff80...0xfffe => self.high_ram[(address - 0xff80) as usize] = value,

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
