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

    pub fn argument(&self, nr: u32) -> Result<u16, EmulatorError> {
        let address = self.regs.flat_sp() + 4 + nr * 2;
        self.memory.read_16(address)
    }
}
