use std::collections::HashMap;
use crate::handle_table::{GenericHandle, Handle, HandleTable};

pub struct Heap {
    max_size: u16,
    allocation_base: u16,
    next_allocation: u16,
    handle_to_pointer: HandleTable<u16>,
}

pub enum HeapAllocationError {
    AllocationTooLarge,
    OutOfMemory,
}

// TODO: for now this is just a simple bump allocator, which should be replaced with something smarter
//       but at least this gets things going
impl Heap {
    pub fn new(max_size: u16, allocation_base: u16) -> Self {
        Self {
            max_size,
            allocation_base,
            next_allocation: 2, // Avoid 0
            handle_to_pointer: HandleTable::new(),
        }
    }

    pub fn allocate(&mut self, is_fixed: bool, size: u16) -> Result<(u16, u16), HeapAllocationError> {
        // Ceil size to a multiple of 2
        let size = if size == u16::MAX {
            return Err(HeapAllocationError::AllocationTooLarge)
        } else {
            (size + 1) & !1
        };
        // TODO: ensure this does not overflow
        let allocation = self.next_allocation + self.allocation_base;
        let (next_allocation, did_overflow) = self.next_allocation.overflowing_add(size);
        if did_overflow || next_allocation > self.max_size {
            Err(HeapAllocationError::AllocationTooLarge)
        } else {
            self.next_allocation = next_allocation;
            if is_fixed {
                Ok((allocation, allocation))
            } else {
                // The de-allocation has to know the difference between a handle and a pointer.
                // As pointers are aligned to multiple of 2's, we can use odd numbers to indicate
                // the handles. Any handle can be returned by the register method, so we map them
                // to odd numbers by using N * 2 - 1. This works because handles are at least 1
                // (so 1 is mapped to 1), and they have a reasonable limit of roughly 10K.
                self.handle_to_pointer.register(allocation)
                    .ok_or(HeapAllocationError::OutOfMemory)
                    .map(|data| (data.as_u16() * 2 - 1, allocation))
            }
        }
    }

    pub fn deallocate(&mut self, what: u16) -> u16 {
        // TODO
        what
    }
}
