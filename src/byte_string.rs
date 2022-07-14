use std::fmt::{Debug, Formatter};
use std::rc::Rc;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct HeapByteString(Rc<[u8]>);

impl HeapByteString {
    pub fn from(b: Rc<[u8]>) -> Self {
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
