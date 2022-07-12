#[derive(Debug, Clone, Copy)]
pub struct ModRM(pub u8);

impl ModRM {
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
