use crate::emulator_error::EmulatorError;
use crate::{u16_from_slice, Segment};

pub struct Memory {
    bytes: Box<[u8]>,
}

#[derive(Debug)]
pub struct SegmentAndOffset {
    pub segment: u16,
    pub offset: u16,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            // TODO
            bytes: vec![0; 1024 * 1024].into_boxed_slice(),
        }
    }

    pub fn segment_and_offset(&self, address: u32) -> SegmentAndOffset {
        let offset = address as u16;
        let segment = ((address - (address & 0xffff)) >> 4) as u16;
        SegmentAndOffset { segment, offset }
    }

    pub fn write_16(&mut self, address: u32, data: u16) -> Result<(), EmulatorError> {
        // TODO: bounds checks
        self.bytes[address as usize] = data as u8;
        self.bytes[address as usize + 1] = (data >> 8) as u8;
        Ok(())
    }

    pub fn write_8(&mut self, address: u32, data: u8) -> Result<(), EmulatorError> {
        // TODO: bounds checks
        self.bytes[address as usize] = data;
        Ok(())
    }

    pub fn read_16(&self, address: u32) -> Result<u16, EmulatorError> {
        // TODO: bounds check
        Ok(u16_from_slice(&self.bytes, address as usize))
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
