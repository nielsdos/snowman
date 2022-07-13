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

    pub fn word_argument(&self, nr: u32) -> Result<u16, EmulatorError> {
        let address = self.regs.flat_sp() + 4 + nr * 2;
        self.memory.read_16(address)
    }

    pub fn dword_argument(&self, nr: u32) -> Result<u32, EmulatorError> {
        let address = self.regs.flat_sp() + 4 + nr * 2;
        self.memory.read_32(address)
    }

    pub fn pointer_argument(&self, nr: u32) -> Result<u32, EmulatorError> {
        let segment = self.word_argument(nr + 1)?;
        let offset = self.word_argument(nr)?;
        let flat_address = ((segment as u32) << 4) + (offset as u32);
        //println!(
        //    "flat address: {:x}:{:x} = {:x}",
        //    segment, offset, flat_address,
        //);
        Ok(flat_address)
    }

    pub fn copy_string(
        &mut self,
        mut src_ptr: u32,
        mut dst_ptr: u32,
    ) -> Result<u32, EmulatorError> {
        let mut number_of_bytes_copied = 0;
        loop {
            let data = self.memory.read_8(src_ptr)?;
            self.memory.write_8(dst_ptr, data)?;
            if data == 0 {
                break;
            }
            number_of_bytes_copied += 1;
            src_ptr += 1;
            dst_ptr += 1;
        }
        Ok(number_of_bytes_copied)
    }
}
