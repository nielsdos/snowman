use std::fmt::{Debug, Formatter};

pub struct HeapByteString(Box<[u8]>);

impl HeapByteString {
    pub fn from(b: Box<[u8]>) -> Self {
        Self(b)
    }
}

impl Debug for HeapByteString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for &byte in &*self.0 {
            write!(f, "{}", byte as char)?;
        }
        Ok(())
    }
}
