
#[derive(Debug)]
pub struct SpeedSwitch {
    reg: u8,
}

impl SpeedSwitch {
    pub fn new() -> SpeedSwitch {
        SpeedSwitch { reg: 0 }
    }

    pub fn write(&mut self, value: u8) {
        self.reg = value
    }

    pub fn read(&self) -> u8 {
        self.reg
    }
}
