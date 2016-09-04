use super::cart::Cart;
use super::ram::Ram;

#[derive(Debug)]
pub struct Interconnect<'a> {
    cart: &'a Cart,
    ram: Ram,
}

impl<'a> Interconnect<'a> {
    pub fn new(cart: &'a Cart) -> Interconnect {
        Interconnect {
            cart: cart,
            ram: Ram::new(),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {

            0x0000...0x3fff => self.cart.bytes[address as usize],

            0xc000...0xdfff => self.ram.read(address),

            // Speedswitch
            0xff4d => 0,

            _ => panic!("READ: address not in range: 0x{:x}", address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {

            0xc000...0xdfff => self.ram.write(address, value),

            // JOYPAD
            0xff00 => {
                println!("Write to JOYPAD: 0x{:x}", value);
            }

            // Speedswitch
            0xff4d => {
                // TODO
            }

            // Interrupt Enable
            0xffff => {
                // TODO
            }

            _ => panic!("WRITE: address not in range: 0x{:x}", address),
        }
    }
}
