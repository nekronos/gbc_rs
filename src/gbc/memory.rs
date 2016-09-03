use super::cart::Cart;

#[derive(Debug)]
pub struct Memory<'a> {
    cart: &'a Cart,
}

impl<'a> Memory<'a> {
    pub fn new(cart: &'a Cart) -> Memory {
        Memory { cart: cart }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000...0x3fff => self.cart.bytes[address as usize],
            _ => panic!("Address not in range: 0x{:x}", address),
        }
    }
}
