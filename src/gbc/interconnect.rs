use super::cart::Cart;
use super::speed_switch::SpeedSwitch;

#[derive(Debug)]
pub struct Interconnect<'a> {
    cart: &'a Cart,
    speed_switch: &'a SpeedSwitch,
}

impl<'a> Interconnect<'a> {
    pub fn new(cart: &'a Cart, speed_switch: &'a SpeedSwitch) -> Interconnect<'a> {
        Interconnect {
            cart: cart,
            speed_switch: speed_switch,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {

            0x0000...0x3fff => self.cart.bytes[address as usize],

            0xff4d => self.speed_switch.read(),

            _ => panic!("Address not in range: 0x{:x}", address),
        }
    }
}
