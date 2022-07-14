use crate::bitmap_allocator::BitmapAllocator;
use std::collections::HashMap;
use std::hash::Hash;

pub trait GenericHandle: Copy + Clone + Eq + Hash + PartialEq {
    fn from(id: u16) -> Self;
    fn as_u16(self) -> u16;
}

pub struct GenericHandleTable<K: GenericHandle, V> {
    internal_table: HashMap<K, V>,
    allocator: BitmapAllocator,
}

impl<K: GenericHandle, V> GenericHandleTable<K, V> {
    pub fn new() -> Self {
        Self {
            internal_table: HashMap::new(),
            // Note: we probably don't need all 65536 atoms to be available?
            //       I capped it at 10K for now...
            allocator: BitmapAllocator::new(10_000, false),
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
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Handle(u16);

impl GenericHandle for Handle {
    fn from(id: u16) -> Self {
        Self(id)
    }

    fn as_u16(self) -> u16 {
        self.0
    }
}

pub type HandleTable<V> = GenericHandleTable<Handle, V>;
