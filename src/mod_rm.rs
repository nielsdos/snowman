#[derive(Debug, Clone, Copy)]
pub struct ModRMByte(pub u8);

impl ModRMByte {
    #[inline]
    pub fn register_destination(&self) -> u8 {
        (self.0 >> 3) & 7
    }

    #[inline]
    pub fn addressing_mode(&self) -> u8 {
        self.0 >> 6
    }

    #[inline]
    pub fn rm(&self) -> u8 {
        self.0 & 7
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ModRM {
    pub mod_rm_byte: ModRMByte,
    pub computed: u16,
}
