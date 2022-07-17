use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct HeapByteString {
    data: Rc<[u8]>,
}

#[derive(Clone)]
pub enum ByteString<'a> {
    Heaped(HeapByteString),
    Static(&'a [u8]),
}

impl HeapByteString {
    pub fn from(data: Rc<[u8]>) -> Self {
        Self { data }
    }

    pub fn as_slice(&self) -> &[u8] {
        &*self.data
    }
}

impl<'a> ByteString<'a> {
    pub fn from_rc_slice(b: Rc<[u8]>) -> Self {
        Self::Heaped(HeapByteString::from(b))
    }

    pub fn from_slice(s: &'a [u8]) -> Self {
        Self::Static(s)
    }

    pub fn as_slice(&self) -> &[u8] {
        match self {
            Self::Heaped(hbs) => hbs.as_slice(),
            Self::Static(slice) => slice,
        }
    }
}

impl From<HeapByteString> for ByteString<'_> {
    fn from(hbs: HeapByteString) -> Self {
        ByteString::Heaped(hbs)
    }
}

impl Debug for HeapByteString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for &byte in self.as_slice() {
            write!(f, "{}", byte as char)?;
        }
        Ok(())
    }
}

impl Debug for ByteString<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for &byte in self.as_slice() {
            write!(f, "{}", byte as char)?;
        }
        Ok(())
    }
}

impl Hash for ByteString<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state)
    }
}

impl PartialEq<Self> for ByteString<'_> {
    fn eq(&self, other: &Self) -> bool {
        let my_slice = self.as_slice();
        let other_slice = other.as_slice();
        my_slice.eq(other_slice)
    }
}

impl Eq for ByteString<'_> {}
