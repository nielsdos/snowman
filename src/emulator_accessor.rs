use crate::registers::Registers;
use crate::{EmulatorError, Memory};

pub struct EmulatorAccessor<'a> {
    memory: &'a Memory,
    regs: &'a mut Registers,
}

impl<'a> EmulatorAccessor<'a> {
    pub fn new(memory: &'a Memory, regs: &'a mut Registers) -> Self {
        Self { memory, regs }
    }

    pub fn regs_mut(&mut self) -> &mut Registers {
        &mut self.regs
    }

    pub fn number_argument(&self, nr: u32) -> Result<u16, EmulatorError> {
        let address = self.regs.flat_sp() + 4 + nr * 2;
        self.memory.read_16(address)
    }

    pub fn pointer_argument(&self, nr: u32) -> Result<u32, EmulatorError> {
        let segment = self.number_argument(nr + 1)?;
        let offset = self.number_argument(nr)?;
        println!("{:x}:{:x}", self.number_argument(nr + 1)?, self.number_argument(nr)?);
        let flat_address = ((segment as u32) << 4) + (offset as u32);
        println!("flat address: {:x} {}", flat_address, self.memory.read_8(flat_address)?);
        todo!()
    }
}
