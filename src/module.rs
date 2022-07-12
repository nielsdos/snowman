use crate::emulator_error::EmulatorError;
use crate::memory::SegmentAndOffset;
use crate::{Memory, Segment};

pub struct EmulatedModule {
    flat_address: u32,
    last_write_offset: u32,
}

impl EmulatedModule {
    pub fn new(flat_address: u32) -> Self {
        Self {
            flat_address,
            last_write_offset: 0,
        }
    }

    fn write_syscall_dispatch_byte(
        &mut self,
        memory: &mut Memory,
        data: u8,
    ) -> Result<(), EmulatorError> {
        let index = self.flat_address + self.last_write_offset;
        self.last_write_offset += 1;
        memory.write_8(index, data)
    }

    pub fn write_syscall_dispatch(
        &mut self,
        memory: &mut Memory,
        ax: u16,
        argument_bytes: u16,
    ) -> Result<u32, EmulatorError> {
        let offset = self.flat_address + self.last_write_offset;

        // mov ax, *value of ax*
        self.write_syscall_dispatch_byte(memory, 0xB8)?;
        self.write_syscall_dispatch_byte(memory, ax as u8)?;
        self.write_syscall_dispatch_byte(memory, (ax >> 8) as u8)?;
        // int 0xff
        self.write_syscall_dispatch_byte(memory, 0xCD)?;
        self.write_syscall_dispatch_byte(memory, 0xFF)?;
        // return far
        //self.write_syscall_dispatch_byte(memory, 0xCB)?;
        self.write_syscall_dispatch_byte(memory, 0xCA)?;
        self.write_syscall_dispatch_byte(memory, argument_bytes as u8)?;
        self.write_syscall_dispatch_byte(memory, (argument_bytes >> 8) as u8)?;

        Ok(offset)
    }

    pub fn procedure(
        &mut self,
        memory: &mut Memory,
        procedure: u16,
        argument_bytes: u16,
    ) -> Result<SegmentAndOffset, EmulatorError> {
        // TODO: deduplicate them?
        let flat_address = self.write_syscall_dispatch(memory, procedure, argument_bytes)?;
        Ok(memory.segment_and_offset(flat_address))
    }
}
