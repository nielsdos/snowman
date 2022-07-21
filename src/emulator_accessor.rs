use crate::byte_string::{ByteString, HeapByteString};
use crate::registers::Registers;
use crate::two_d::Rect;
use crate::{EmulatorError, Memory};

pub struct EmulatorAccessor<'a> {
    memory: &'a mut Memory,
    regs: &'a mut Registers,
}

impl<'a> EmulatorAccessor<'a> {
    pub fn new(memory: &'a mut Memory, regs: &'a mut Registers) -> Self {
        Self { memory, regs }
    }

    pub fn regs(&self) -> &Registers {
        self.regs
    }

    pub fn regs_mut(&mut self) -> &mut Registers {
        self.regs
    }

    pub fn memory(&self) -> &Memory {
        self.memory
    }

    pub fn memory_mut(&mut self) -> &mut Memory {
        self.memory
    }

    pub fn word_argument(&self, nr: u32) -> Result<u16, EmulatorError> {
        let address = self.regs.flat_sp() + 4 + nr * 2;
        self.memory.read_u16(address)
    }

    pub fn push_16(&mut self, value: u16) -> Result<(), EmulatorError> {
        self.regs.dec_sp(2);
        self.memory.write_u16(
            self.regs.flat_reg(Registers::REG_SS, Registers::REG_SP),
            value,
        )
    }

    pub fn far_call_into_proc_setup(&mut self) -> Result<(), EmulatorError> {
        // The actual place to return to in the system call
        self.push_16(self.regs.ip)?;
        // Return value will be filled in later by `ReturnValue::DelayedU16`
        self.regs.dec_sp(2);
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
        //println!("{}, {:x}:{:x}", nr, segment, offset);
        let flat_address = ((segment as u32) << 4) + (offset as u32);
        Ok(flat_address)
    }

    pub fn strlen(&self, mut ptr: u32) -> Result<u16, EmulatorError> {
        let mut length = 0u16;
        loop {
            let data = self.memory.read_8(ptr)?;
            if data == 0 {
                break;
            }
            length = length.wrapping_add(1);
            ptr += 1;
        }
        Ok(length)
    }

    pub fn copy_string(
        &mut self,
        mut src_ptr: u32,
        mut dst_ptr: u32,
    ) -> Result<u16, EmulatorError> {
        let mut number_of_bytes_copied = 0u16;
        loop {
            let data = self.memory.read_8(src_ptr)?;
            self.memory.write_8(dst_ptr, data)?;
            if data == 0 {
                break;
            }
            number_of_bytes_copied = number_of_bytes_copied.wrapping_add(1);
            src_ptr += 1;
            dst_ptr += 1;
        }
        Ok(number_of_bytes_copied)
    }

    pub fn clone_string(
        &self,
        mut src_ptr: u32,
        convert_to_lowercase: bool,
    ) -> Result<HeapByteString, EmulatorError> {
        let mut output = Vec::new();
        loop {
            let data = self.memory.read_8(src_ptr)?;
            if data == 0 {
                break;
            }
            if convert_to_lowercase && data >= 65 && data <= 90 {
                output.push(data | 32);
            } else {
                output.push(data);
            }
            src_ptr += 1;
        }
        Ok(HeapByteString::from(output.into()))
    }

    pub fn static_string(&self, src_ptr: u32) -> Result<ByteString, EmulatorError> {
        let mut length = 0;
        loop {
            let current = src_ptr.saturating_add(length);
            let data = self.memory.read_8(current)?;
            if data == 0 {
                return self
                    .memory
                    .slice(src_ptr, current)
                    .map(ByteString::from_slice);
            }
            length += 1;
        }
    }

    pub fn read_rect(&self, src_ptr: u32) -> Result<Rect, EmulatorError> {
        let rect_left = self.memory.read_i16(src_ptr)?;
        let rect_top = self.memory.read_i16(src_ptr + 2)?;
        let rect_right = self.memory.read_i16(src_ptr + 4)?;
        let rect_bottom = self.memory.read_i16(src_ptr + 6)?;
        Ok(Rect {
            left: rect_left,
            top: rect_top,
            right: rect_right,
            bottom: rect_bottom,
        })
    }

    pub fn write_rect(&mut self, dst_ptr: u32, rect: &Rect) -> Result<(), EmulatorError> {
        self.memory.write_i16(dst_ptr, rect.left)?;
        self.memory.write_i16(dst_ptr + 2, rect.top)?;
        self.memory.write_i16(dst_ptr + 4, rect.right)?;
        self.memory.write_i16(dst_ptr + 6, rect.bottom)
    }
}
