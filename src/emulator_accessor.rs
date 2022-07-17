use crate::byte_string::{ByteString, HeapByteString};
use crate::registers::Registers;
use crate::{EmulatorError, Memory};
use crate::two_d::Rect;

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

    pub fn push_16(&mut self, value: u16) -> Result<(), EmulatorError> {
        self.regs.dec_sp(2);
        self.memory.write_16(
            self.regs.flat_reg(Registers::REG_SS, Registers::REG_SP),
            value,
        )
    }

    pub fn far_call_into_proc_setup(&mut self) -> Result<(), EmulatorError> {
        // The actual place to return to in the system call
        self.push_16(self.regs.ip)?;
        // Save return value
        self.push_16(self.regs.read_gpr_16(Registers::REG_AX))?;
        Ok(())
    }

    pub fn far_call_into_proc_execute(
        &mut self,
        segment: u16,
        offset: u16,
    ) -> Result<(), EmulatorError> {
        // Setup return far into the system call trampoline
        self.push_16(self.regs.read_segment(Registers::REG_CS))?;
        self.push_16(0)?;
        // Move into proc
        self.regs.ip = offset;
        self.regs.write_segment(Registers::REG_CS, segment);
        Ok(())
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

    pub fn clone_string(&mut self, mut src_ptr: u32) -> Result<HeapByteString, EmulatorError> {
        let mut output = Vec::new();
        loop {
            let data = self.memory.read_8(src_ptr)?;
            if data == 0 {
                break;
            }
            output.push(data);
            src_ptr += 1;
        }
        Ok(HeapByteString::from(output.into()))
    }

    pub fn static_string(&mut self, mut src_ptr: u32) -> Result<ByteString, EmulatorError> {
        let mut length = 0;
        loop {
            let current = src_ptr.saturating_add(length);
            let data = self.memory.read_8(current)?;
            if data == 0 {
                return self.memory.slice(src_ptr, current).map(ByteString::Static);
            }
            length += 1;
        }
        Err(EmulatorError::OutOfBounds)
    }

    pub fn read_rect(&self, src_ptr: u32) -> Result<Rect, EmulatorError> {
        let rect_left = self.memory.read_16(src_ptr)?;
        let rect_top = self.memory.read_16(src_ptr + 2)?;
        let rect_right = self.memory.read_16(src_ptr + 4)?;
        let rect_bottom = self.memory.read_16(src_ptr + 6)?;
        Ok(Rect {
            left: rect_left,
            top: rect_top,
            right: rect_right,
            bottom: rect_bottom,
        })
    }
}
