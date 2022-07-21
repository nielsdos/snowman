use crate::constants::{GDI_INT_VECTOR, KERNEL_INT_VECTOR, KEYBOARD_INT_VECTOR, USER_INT_VECTOR};
use crate::emulator_error::EmulatorError;
use crate::memory::SegmentAndOffset;
use crate::Memory;
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
            // TODO: make sure the written bytes all stay within the same segment!
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

    pub fn write_syscall_proc_return_trampoline(
        &self,
        memory: &mut Memory,
    ) -> Result<(), EmulatorError> {
        // pop ax
        self.write_syscall_dispatch_byte(memory, 0x58)?;
        // ret
        self.write_syscall_dispatch_byte(memory, 0xC3)
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
}

impl Module for KernelModule {
    fn argument_bytes_of_procedure(&self, procedure: u16) -> u16 {
        match procedure {
            91 | 102 => 0,
            7 | 18 | 19 | 23 | 24 | 30 => 2,
            5 | 61 | 90 => 4,
            50 | 51 => 6,
            54 | 89 => 8,
            57 | 60 => 10,
            127 => 14,
            129 => 16,
            58 => 18,
            128 => 22,
            178 => 0, /* TODO: don't know! */
            _ => unimplemented!("procedure {}", procedure),
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
            19 | 243 => 0,
            5 | 6 | 18 | 31 | 59 | 66 | 69 | 124 | 157 | 179 | 180 | 249 | 272 | 287 => 2,
            12 | 42 | 57 | 68 | 113 | 114 | 135 | 156 => 4,
            32 | 33 | 37 | 39 | 40 | 134 | 154 | 155 | 173 | 174 => 6,
            77 | 78 | 81 | 125 | 136 => 8,
            0xFFFF | 10 | 107 | 108 | 110 | 176 | 411 => 10,
            1 | 72 | 87 => 12,
            232 => 14,
            41 => 30,
            420 => 0, // WSPRINTF's caller cleans up the arguments
            _ => unimplemented!("procedure {}", procedure),
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
            52 | 68 | 69 | 87 => 2,
            2 | 4 | 45 | 57 | 66 | 80 | 119 | 346 => 4,
            1 | 9 | 19 | 20 | 128 | 154 => 6,
            61 | 91 => 8,
            27 => 10,
            156 => 12,
            29 => 14,
            53 => 16,
            34 => 20,
            351 => 22,
            _ => unimplemented!("procedure {}", procedure),
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

pub struct DummyModule {
    base_module: BaseModule,
}

impl DummyModule {
    pub fn new(flat_address: u32) -> Self {
        Self {
            base_module: BaseModule::new(flat_address, KEYBOARD_INT_VECTOR),
        }
    }
}

impl Module for DummyModule {
    fn argument_bytes_of_procedure(&self, procedure: u16) -> u16 {
        match procedure {
            _ => 0,
        }
    }

    fn base_module(&self) -> &BaseModule {
        &self.base_module
    }
}
