use super::cart::Cart;

#[derive(Debug)]
pub struct Interconnect<'a> {
    cart: &'a Cart,
}

impl<'a> Interconnect<'a> {
    pub fn new(cart: &'a Cart) -> Interconnect {
        Interconnect { cart: cart }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {

            0x0000...0x3fff => self.cart.bytes[address as usize],

            // Speedswitch
            0xff4d => 0,

            _ => panic!("READ: address not in range: 0x{:x}", address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {

            // JOYPAD
            0xff00 => {
                println!("Write to JOYPAD: 0x{:x}", value);
            }

            // Speedswitch
            0xff4d => {
                // TODO
            },

            // Interrupt Enable
            0xffff => {
                // TODO
            },

            _ => panic!("WRITE: address not in range: 0x{:x}", address),
        }
    }
}
