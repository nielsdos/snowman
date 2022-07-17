use std::mem::size_of;

pub struct BitVectorAllocator {
    bits: Box<[usize]>,
}

impl BitVectorAllocator {
    pub fn new(nr_bits: usize, is_zero_valid: bool) -> Self {
        let mut bits = vec![usize::MAX; (nr_bits + size_of::<usize>() - 1) / size_of::<usize>()]
            .into_boxed_slice();
        if !is_zero_valid {
            bits[0] &= !(1 << 0);
        }
        Self { bits }
    }

    pub fn allocate(&mut self) -> Option<usize> {
        for (index, entry) in self.bits.iter_mut().enumerate() {
            if *entry != 0 {
                let bit = entry.trailing_zeros();
                *entry &= !(1 << bit);
                return Some(index + bit as usize);
            }
        }

        None
    }

    pub fn deallocate(&mut self, bit: usize) {
        self.bits[bit / size_of::<usize>()] |= 1 << (bit % size_of::<usize>());
    }
}
