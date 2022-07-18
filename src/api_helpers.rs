pub struct Pointer(pub u32);

pub enum ReturnValue {
    U16(u16),
    U32(u32),
    DelayedU16(u16),
    None,
}

impl From<u32> for Pointer {
    fn from(data: u32) -> Pointer {
        Pointer(data)
    }
}
