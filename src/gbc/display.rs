#[derive(Debug)]
pub struct Display {
    
}

impl Display {
    pub fn new() -> Display {
        Display {}
    }

    pub fn read(&self, address: u16) -> u8 {
        0
    }

    pub fn write(&mut self, address: u16, value: u8) {}
}
