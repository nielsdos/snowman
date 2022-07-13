use std::mem::size_of;

pub struct BitmapAllocator {
    bits: Box<[usize]>,
}

impl BitmapAllocator {
    pub fn new(nr_bits: usize) -> Self {
        Self {
            bits: vec![usize::MAX; (nr_bits + size_of::<usize>() - 1) / size_of::<usize>()].into_boxed_slice()
        }
    }

    pub fn claim(&mut self, bit: usize) {
        self.bits[bit / size_of::<usize>()] &= !(1 << (bit % size_of::<usize>()));
    }

    pub fn allocate(&mut self) -> Option<usize> {
        for (index, mut entry) in self.bits.iter_mut().enumerate() {
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
