use crate::bitmap_allocator::BitmapAllocator;
use crate::heap_byte_string::HeapByteString;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Atom(u16);

pub struct AtomTable {
    internal_table: HashMap<Atom, HeapByteString>,
    allocator: BitmapAllocator,
}

impl Atom {
    #[inline]
    pub fn as_u16(self) -> u16 {
        self.0
    }
}

impl AtomTable {
    pub fn new() -> Self {
        let mut allocator = BitmapAllocator::new(10_000);
        // Note: atom 0 is invalid
        allocator.claim(0);
        Self {
            internal_table: HashMap::new(),
            // Note: we probably don't need all 65536 atoms to be available?
            //       I capped it at 10K for now...
            allocator,
        }
    }

    fn allocate_atom(&mut self) -> Option<Atom> {
        self.allocator.allocate().map(|value| Atom(value as u16))
    }

    pub fn register_atom(&mut self, string: HeapByteString) -> Option<Atom> {
        let atom = self.allocate_atom()?;
        println!("REGISTER ATOM: {:?} => {:?}", atom, string);
        self.internal_table.insert(atom, string);
        Some(atom)
    }

    pub fn deregister_atom(&mut self, atom: Atom) -> bool {
        match self.internal_table.remove(&atom) {
            Some(_) => true,
            None => false,
        }
    }
}
