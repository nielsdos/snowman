use crate::byte_string::HeapByteString;
use crate::handle_table::{GenericHandle, GenericHandleTable};

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Atom(u16);

pub type AtomTable = GenericHandleTable<Atom, HeapByteString>;

impl GenericHandle for Atom {
    fn from(id: u16) -> Self {
        Self(id)
    }

    fn as_u16(self) -> u16 {
        self.0
    }
}
