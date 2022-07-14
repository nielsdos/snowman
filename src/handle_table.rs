use crate::bitvector_allocator::BitVectorAllocator;
use std::collections::HashMap;
use std::hash::Hash;
use crate::byte_string::HeapByteString;

pub trait GenericHandle: Copy + Clone + Eq + Hash + PartialEq + From<u16> {
    fn as_u16(self) -> u16;
}

pub struct GenericHandleTable<K: GenericHandle, V> {
    internal_table: HashMap<K, V>,
    allocator: BitVectorAllocator,
}

impl<K: GenericHandle, V> GenericHandleTable<K, V> {
    pub fn new() -> Self {
        Self {
            internal_table: HashMap::new(),
            // Note: we probably don't need all 65536 atoms to be available?
            //       I capped it at 10K for now...
            allocator: BitVectorAllocator::new(10_000, false),
        }
    }

    fn allocate_handle(&mut self) -> Option<K> {
        self.allocator.allocate().map(|value| K::from(value as u16))
    }

    pub fn register(&mut self, value: V) -> Option<K> {
        let handle = self.allocate_handle()?;
        self.internal_table.insert(handle, value);
        Some(handle)
    }

    pub fn deregister(&mut self, handle: K) -> bool {
        self.internal_table.remove(&handle).is_some()
    }

    pub fn get(&self, handle: K) -> Option<&V> {
        self.internal_table.get(&handle)
    }
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Handle(u16);

impl Handle {
    pub const fn null() -> Self {
        Self(0)
    }
}

impl From<u16> for Handle {
    fn from(id: u16) -> Self {
        Self(id)
    }
}

impl GenericHandle for Handle {
    fn as_u16(self) -> u16 {
        self.0
    }
}

pub type HandleTable<V> = GenericHandleTable<Handle, V>;
