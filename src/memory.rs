use crate::emulator_error::EmulatorError;
use crate::util::{u16_from_array, u32_from_array};

const MEMORY_SIZE: usize = 1024 * 1024;

pub struct Memory {
    bytes: Box<[u8; MEMORY_SIZE]>,
}

#[derive(Debug, Copy, Clone)]
pub struct SegmentAndOffset {
    pub segment: u16,
    pub offset: u16,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            bytes: Box::new([0; MEMORY_SIZE]),
        }
    }

    pub fn slice(&self, start: u32, end: u32) -> Result<&[u8], EmulatorError> {
        self.bytes
            .get(start as usize..end as usize)
            .ok_or(EmulatorError::OutOfBounds)
    }

    pub fn segment_and_offset(&self, address: u32) -> SegmentAndOffset {
        let offset = address as u16;
        let segment = ((address - (address & 0xffff)) >> 4) as u16;
        SegmentAndOffset { segment, offset }
    }

    pub fn write_16(&mut self, address: u32, data: u16) -> Result<(), EmulatorError> {
        if ((address + 1) as usize) < MEMORY_SIZE {
            self.bytes[address as usize] = data as u8;
            self.bytes[address as usize + 1] = (data >> 8) as u8;
            Ok(())
        } else {
            Err(EmulatorError::OutOfBounds)
        }
    }

    pub fn write_8(&mut self, address: u32, data: u8) -> Result<(), EmulatorError> {
        *self
            .bytes
            .get_mut(address as usize)
            .ok_or(EmulatorError::OutOfBounds)? = data;
        Ok(())
    }

    pub fn write<const N: usize>(&mut self, address: u32, data: u16) -> Result<(), EmulatorError> {
        if N == 8 {
            self.write_8(address, data as u8)
        } else if N == 16 {
            self.write_16(address, data)
        } else {
            unreachable!()
        }
    }

    pub fn flat_pointer_read(&self, offset: u32) -> Result<u32, EmulatorError> {
        let segment = self.read_16(offset + 2)?;
        let offset = self.read_16(offset)?;
        Ok(((segment as u32) << 4) + (offset as u32))
    }

    pub fn read_32(&self, address: u32) -> Result<u32, EmulatorError> {
        u32_from_array::<MEMORY_SIZE>(&*self.bytes, address as usize)
            .ok_or(EmulatorError::OutOfBounds)
    }

    pub fn read_16(&self, address: u32) -> Result<u16, EmulatorError> {
        u16_from_array(&*self.bytes, address as usize).ok_or(EmulatorError::OutOfBounds)
    }

    pub fn read_8(&self, address: u32) -> Result<u8, EmulatorError> {
        self.bytes
            .get(address as usize)
            .copied()
            .ok_or(EmulatorError::OutOfBounds)
    }

    pub fn read<const N: usize>(&self, address: u32) -> Result<u16, EmulatorError> {
        if N == 8 {
            self.read_8(address).map(|data| data as u16)
        } else if N == 16 {
            self.read_16(address)
        } else {
            unreachable!()
        }
    }

    pub fn copy_from(&mut self, bytes: &[u8], offset: usize) -> Result<(), EmulatorError> {
        if offset + bytes.len() < self.bytes.len() {
            self.bytes[offset..offset + bytes.len()].copy_from_slice(bytes);
            Ok(())
        } else {
            Err(EmulatorError::OutOfBounds)
        }
    }
}
