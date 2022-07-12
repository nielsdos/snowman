use crate::registers::Registers;
use crate::{EmulatorError, Memory};

pub struct EmulatorAccessor<'a> {
    memory: &'a mut Memory,
    regs: &'a mut Registers,
}

impl<'a> EmulatorAccessor<'a> {
    pub fn new(memory: &'a mut Memory, regs: &'a mut Registers) -> Self {
        Self { memory, regs }
    }

    pub fn regs_mut(&mut self) -> &mut Registers {
        &mut self.regs
    }

    pub fn memory(&self) -> &Memory {
        self.memory
    }

    pub fn memory_mut(&mut self) -> &mut Memory {
        &mut self.memory
    }

    pub fn number_argument(&self, nr: u32) -> Result<u16, EmulatorError> {
        let address = self.regs.flat_sp() + 4 + nr * 2;
        self.memory.read_16(address)
    }

    pub fn pointer_argument(&self, nr: u32) -> Result<u32, EmulatorError> {
        let segment = self.number_argument(nr + 1)?;
        let offset = self.number_argument(nr)?;
        println!("{:x}:{:x}", segment, offset);
        let flat_address = ((segment as u32) << 4) + (offset as u32);
        println!(
            "flat address: {:x} {:x}",
            flat_address,
            self.memory.read_8(flat_address)?
        );
        Ok(flat_address)
    }
}
