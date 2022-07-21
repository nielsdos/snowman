pub struct SegmentBumpAllocator {
    pointer: usize,
}

impl SegmentBumpAllocator {
    pub fn new() -> Self {
        Self {
            pointer: 0,
        }
    }

    pub fn allocate(&mut self, size: usize) -> Option<u16> {
        let current_pointer = self.pointer;
        // Round size up to the closest multiple of 16, as each segment moves 16 bytes
        let size = (size + 16 - 1) & !(16 - 1);
        self.pointer += size;
        u16::try_from(current_pointer >> 4).ok()
    }
}
