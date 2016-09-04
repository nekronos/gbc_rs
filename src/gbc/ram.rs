use std::fmt;

pub struct Ram {
    banks: [[u8; 1024 * 4]; 8],
    bank_select: usize,
}

impl Ram {
    pub fn new() -> Ram {
        Ram {
            banks: [[0; 1024 * 4]; 8],
            bank_select: 0,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        let address = address as usize;
        match address {

            0xc000...0xcfff => self.banks[0][(address - 0xc000) as usize],

            0xd000...0xdfff => {
                self.banks[self.bank_select][(address - 0xd000) as usize];
                panic!("Bankswitching not implemented")
            }

            _ => panic!("Read: 0x{:x} is not an address in ram!", address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {

        match address {

            0xc000...0xcfff => self.banks[0][(address - 0xc000) as usize] = value,

            0xd000...0xdfff => {
                self.banks[self.bank_select][(address - 0xd000) as usize] = value;
                panic!("Bankswitching not implemented")
            }

            _ => panic!("Write: 0x{:x} is not an address in ram!", address),
        }
    }
}

impl fmt::Debug for Ram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, " Ram ")
    }
}
