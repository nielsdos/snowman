use crate::{bool_to_result, u16_from_slice, HeapByteString};

#[derive(Debug)]
pub enum ExecutableFormatError {
    HeaderSize,
    HeaderMagic,
    ApplicationFlags,
    OperatingSystem,
    SegmentIndex,
    SegmentOffset,
    Memory,
}

pub struct Executable<'a> {
    internal_data: &'a mut [u8],
    cursor: usize,
}

#[must_use]
pub struct OldCursor {
    cursor: usize,
}

impl<'a> Executable<'a> {
    pub fn new(internal_data: &'a mut [u8]) -> Self {
        Self {
            internal_data,
            cursor: 0,
        }
    }

    pub fn seek_from_start(&mut self, offset: usize) -> Result<OldCursor, ExecutableFormatError> {
        if offset >= self.internal_data.len() {
            Err(ExecutableFormatError::HeaderSize)
        } else {
            let old_cursor = self.cursor;
            self.cursor = offset;
            Ok(OldCursor { cursor: old_cursor })
        }
    }

    pub fn validate_magic_id(
        &self,
        offset: usize,
        expected_magic_id: &[u8; 2],
    ) -> Result<(), ExecutableFormatError> {
        bool_to_result(
            self.read_u8(offset)? == expected_magic_id[0]
                && self.read_u8(offset + 1)? == expected_magic_id[1],
            ExecutableFormatError::HeaderMagic,
        )
    }

    pub fn seek_from_here(&mut self, offset: usize) -> Result<OldCursor, ExecutableFormatError> {
        self.seek_from_start(self.cursor + offset)
    }

    pub fn restore_cursor(&mut self, cursor: OldCursor) {
        self.cursor = cursor.cursor;
    }

    pub fn read_u8(&self, offset: usize) -> Result<u8, ExecutableFormatError> {
        let index = self.cursor + offset;
        self.internal_data
            .get(index)
            .copied()
            .ok_or(ExecutableFormatError::HeaderSize)
    }

    pub fn read_u16(&self, offset: usize) -> Result<u16, ExecutableFormatError> {
        let index = self.cursor + offset;
        u16_from_slice(self.internal_data, index).ok_or(ExecutableFormatError::HeaderSize)
    }

    pub fn overwrite_u8(&mut self, offset: usize, data: u8) -> Result<(), ExecutableFormatError> {
        let index = self.cursor + offset;
        if index < self.internal_data.len() {
            self.internal_data[index] = data;
            Ok(())
        } else {
            Err(ExecutableFormatError::HeaderSize)
        }
    }

    pub fn slice(&self, offset: usize, len: usize) -> Result<&[u8], ExecutableFormatError> {
        if self.cursor + offset + len <= self.internal_data.len() {
            Ok(&self.internal_data[self.cursor + offset..self.cursor + offset + len])
        } else {
            Err(ExecutableFormatError::HeaderSize)
        }
    }

    pub fn read_string_helper(
        &self,
        offset: usize,
    ) -> Result<Option<&[u8]>, ExecutableFormatError> {
        let length = self.read_u8(offset)?;
        if length == 0 {
            Ok(None)
        } else {
            self.slice(offset + 1, length as usize).map(Some)
        }
    }

    pub fn read_string(
        &self,
        offset: usize,
    ) -> Result<Option<HeapByteString>, ExecutableFormatError> {
        self.read_string_helper(offset)
            .map(|data| data.map(|data| HeapByteString::from(data.into())))
    }

    pub fn read_string_to_lowercase(
        &self,
        offset: usize,
    ) -> Result<Option<HeapByteString>, ExecutableFormatError> {
        self.read_string_helper(offset)
            .map(|data| data.map(HeapByteString::from_to_lowercase))
    }
}
