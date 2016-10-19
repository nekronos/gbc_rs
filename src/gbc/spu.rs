
#[derive(Debug)]
pub struct Spu;

impl Spu {
    pub fn new() -> Spu {
        Spu {}
    }

    #[allow(unused_variables)]
    pub fn write(&mut self, addr: u16, val: u8) {}

    #[allow(unused_variables)]
    pub fn read(&self, addr: u16) -> u8 {
        0
    }
}
