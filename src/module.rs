use crate::constants::{GDI_INT_VECTOR, KERNEL_INT_VECTOR, KEYBOARD_INT_VECTOR, USER_INT_VECTOR};
use crate::emulator_error::EmulatorError;
use crate::memory::SegmentAndOffset;
use crate::{Memory, Segment};
use std::cell::Cell;

pub struct BaseModule {
    flat_address: Cell<u32>,
    last_write_offset: Cell<u32>,
    int_vector: u8,
}

pub trait Module {
    fn argument_bytes_of_procedure(&self, procedure: u16) -> u16;
    fn base_module(&self) -> &BaseModule;
}

impl BaseModule {
    fn new(flat_address: u32, int_vector: u8) -> Self {
        Self {
            flat_address: Cell::new(flat_address),
            last_write_offset: Cell::new(0),
            int_vector,
        }
    }

    fn write_syscall_dispatch_byte(
        &self,
        memory: &mut Memory,
        data: u8,
    ) -> Result<(), EmulatorError> {
        let index = self.flat_address.get() + self.last_write_offset.get();
        self.last_write_offset.set(self.last_write_offset.get() + 1);
        memory.write_8(index, data)
    }

    fn write_syscall_dispatch(
        &self,
        memory: &mut Memory,
        ax: u16,
        argument_bytes: u16,
    ) -> Result<u32, EmulatorError> {
        let offset = self.flat_address.get() + self.last_write_offset.get();

        // mov ax, *value of ax*
        self.write_syscall_dispatch_byte(memory, 0xB8)?;
        self.write_syscall_dispatch_byte(memory, ax as u8)?;
        self.write_syscall_dispatch_byte(memory, (ax >> 8) as u8)?;
        // int 0xff
        self.write_syscall_dispatch_byte(memory, 0xCD)?;
        self.write_syscall_dispatch_byte(memory, self.int_vector)?;
        // return far
        if argument_bytes == 0 {
            self.write_syscall_dispatch_byte(memory, 0xCB)?;
        } else {
            self.write_syscall_dispatch_byte(memory, 0xCA)?;
            self.write_syscall_dispatch_byte(memory, argument_bytes as u8)?;
            self.write_syscall_dispatch_byte(memory, (argument_bytes >> 8) as u8)?;
        }

        Ok(offset)
    }

    pub fn procedure(
        &self,
        memory: &mut Memory,
        procedure: u16,
        argument_bytes: u16,
    ) -> Result<SegmentAndOffset, EmulatorError> {
        // TODO: deduplicate them?
        let flat_address = self.write_syscall_dispatch(memory, procedure, argument_bytes)?;
        Ok(memory.segment_and_offset(flat_address))
    }
}

pub struct KernelModule {
    base_module: BaseModule,
}

impl KernelModule {
    pub fn new(flat_address: u32) -> Self {
        Self {
            base_module: BaseModule::new(flat_address, KERNEL_INT_VECTOR),
        }
    }

    pub fn base(&self) -> &BaseModule {
        &self.base_module
    }
}

impl Module for KernelModule {
    fn argument_bytes_of_procedure(&self, procedure: u16) -> u16 {
        match procedure {
            23 | 24 | 30 => 2,
            51 => 6,
            _ => 0,
        }
    }

    fn base_module(&self) -> &BaseModule {
        &self.base_module
    }
}

pub struct UserModule {
    base_module: BaseModule,
}

impl UserModule {
    pub fn new(flat_address: u32) -> Self {
        Self {
            base_module: BaseModule::new(flat_address, USER_INT_VECTOR),
        }
    }
}

impl Module for UserModule {
    fn argument_bytes_of_procedure(&self, procedure: u16) -> u16 {
        match procedure {
            5 | 179 => 2,
            87 => 12,
            _ => 0,
        }
    }

    fn base_module(&self) -> &BaseModule {
        &self.base_module
    }
}

pub struct GdiModule {
    base_module: BaseModule,
}

impl GdiModule {
    pub fn new(flat_address: u32) -> Self {
        Self {
            base_module: BaseModule::new(flat_address, GDI_INT_VECTOR),
        }
    }
}

impl Module for GdiModule {
    fn argument_bytes_of_procedure(&self, procedure: u16) -> u16 {
        match procedure {
            _ => 0,
        }
    }

    fn base_module(&self) -> &BaseModule {
        &self.base_module
    }
}

pub struct KeyboardModule {
    base_module: BaseModule,
}

impl KeyboardModule {
    pub fn new(flat_address: u32) -> Self {
        Self {
            base_module: BaseModule::new(flat_address, KEYBOARD_INT_VECTOR),
        }
    }
}

impl Module for KeyboardModule {
    fn argument_bytes_of_procedure(&self, procedure: u16) -> u16 {
        match procedure {
            _ => 0,
        }
    }

    fn base_module(&self) -> &BaseModule {
        &self.base_module
    }
}
