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
            _ => panic!("Address not in range: 0x{:x}", address),
        }
    }
}
