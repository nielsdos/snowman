use crate::byte_string::ByteString;
use crate::handle_table::{GenericHandle, GenericHandleTable};

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Atom(u16);

pub type AtomTable<'a> = GenericHandleTable<Atom, ByteString<'a>>;

impl GenericHandle for Atom {
    fn as_u16(self) -> u16 {
        self.0
    }
}

impl From<u16> for Atom {
    fn from(id: u16) -> Self {
        Self(id)
    }
}
