use crate::api_helpers::{Pointer, ReturnValue};
use crate::constants::WinFlags;
use crate::emulator_accessor::EmulatorAccessor;
use crate::handle_table::Handle;
use crate::registers::Registers;
use crate::{debug, debug_print_null_terminated_string, EmulatorError, ObjectEnvironment};
use std::sync::{RwLock, RwLockWriteGuard};
use syscall::api_function;

pub struct EmulatedKernel<'a> {
    objects: &'a RwLock<ObjectEnvironment<'a>>,
}

impl<'a> EmulatedKernel<'a> {
    pub fn new(objects: &'a RwLock<ObjectEnvironment<'a>>) -> Self {
        Self { objects }
    }

    fn write_objects(&self) -> RwLockWriteGuard<ObjectEnvironment<'a>> {
        self.objects.write().unwrap()
    }

    #[api_function]
    fn get_version(&self) -> Result<ReturnValue, EmulatorError> {
        // Report version Windows 3.10
        Ok(ReturnValue::U16(0x0A03))
    }

    #[api_function]
    fn local_alloc(
        &self,
        mut accessor: EmulatorAccessor,
        flags: u16,
        size: u16,
    ) -> Result<ReturnValue, EmulatorError> {
        let is_fixed = (flags & 0b10) == 0;
        let should_zero = (flags & 0x40) > 0;

        // TODO: should select a different local heap based on DS
        let result = self.write_objects().local_heap.allocate(is_fixed, size);
        if let Ok((return_value, pointer)) = result {
            if should_zero {
                let flat_address = accessor.regs().flat_address(Registers::REG_DS, pointer);
                accessor
                    .memory_mut()
                    .zero(flat_address, flat_address + (size as u32))?;
            }

            Ok(ReturnValue::U16(return_value))
        } else {
            Ok(ReturnValue::U16(0))
        }
    }

    #[api_function]
    fn local_free(&self, handle_or_pointer: u16) -> Result<ReturnValue, EmulatorError> {
        if handle_or_pointer == 0 {
            Ok(ReturnValue::U16(0))
        } else {
            // TODO: should select a different local heap based on DS
            Ok(ReturnValue::U16(
                self.write_objects()
                    .local_heap
                    .deallocate(handle_or_pointer),
            ))
        }
    }

    #[api_function]
    fn get_winflags(&self) -> Result<ReturnValue, EmulatorError> {
        let flags = (WinFlags::WF_80X87 | WinFlags::WF_PMODE | WinFlags::WF_ENHANCED).bits();
        Ok(ReturnValue::U32(flags))
    }

    #[api_function]
    fn init_task(&self, mut accessor: EmulatorAccessor) -> Result<ReturnValue, EmulatorError> {
        let regs = accessor.regs_mut();

        // TODO: hardcoded to inittask rn
        let es = regs.read_segment(Registers::REG_DS);
        regs.write_gpr_16(Registers::REG_BX, 0x1234); // TODO: offset into command line
        regs.write_gpr_16(Registers::REG_CX, 0); // TODO: stack limit
        regs.write_gpr_16(Registers::REG_DX, 0); // TODO: nCmdShow
        regs.write_gpr_16(Registers::REG_SI, 0); // TODO: previous instance handle
        regs.write_gpr_16(Registers::REG_DI, 0xBEEF); // TODO: instance handle
        regs.write_gpr_16(Registers::REG_BP, regs.read_gpr_16(Registers::REG_SP));
        // TODO: segments
        regs.write_segment(Registers::REG_ES, es); // TODO

        // TODO: must be = ES I believe
        Ok(ReturnValue::U16(es))
    }

    #[api_function]
    fn lock_segment(&self, _segment: u16) -> Result<ReturnValue, EmulatorError> {
        Ok(ReturnValue::None)
    }

    #[api_function]
    fn unlock_segment(&self, _segment: u16) -> Result<ReturnValue, EmulatorError> {
        Ok(ReturnValue::None)
    }

    #[api_function]
    fn wait_event(&self) -> Result<ReturnValue, EmulatorError> {
        Ok(ReturnValue::None)
    }

    #[api_function]
    fn get_proc_address(
        &self,
        accessor: EmulatorAccessor,
        _h_module: Handle,
        proc_name: Pointer,
    ) -> Result<ReturnValue, EmulatorError> {
        println!("GET PROC ADDRESS");
        debug_print_null_terminated_string(&accessor, proc_name.0);
        Ok(ReturnValue::U32(0))
    }

    #[api_function]
    fn make_proc_instance(
        &self,
        _h_instance: Handle,
        offset_of_function: u16,
        segment_of_function: u16,
    ) -> Result<ReturnValue, EmulatorError> {
        // As we don't share segments in the same way as a 16-bit Windows might do,
        // we don't need to set up any thunks. We just need to make sure the return value is
        // equal to the original function address.
        Ok(ReturnValue::U32(
            ((segment_of_function as u32) << 16) | (offset_of_function as u32),
        ))
    }

    #[api_function]
    fn get_profile_int(
        &self,
        accessor: EmulatorAccessor,
        app_name: Pointer,
        key_name: Pointer,
        default: u16,
    ) -> Result<ReturnValue, EmulatorError> {
        debug_print_null_terminated_string(&accessor, key_name.0);
        debug_print_null_terminated_string(&accessor, app_name.0);
        Ok(ReturnValue::U16(default))
    }

    #[api_function]
    fn find_resource(
        &self,
        accessor: EmulatorAccessor,
        module: Handle,
        name: Pointer,
        _type: Pointer,
    ) -> Result<ReturnValue, EmulatorError> {
        debug!(
            "[kernel] FIND RESOURCE {:x} {:x} {:?}",
            _type.0, name.0, module
        );
        debug_print_null_terminated_string(&accessor, name.0);
        debug_print_null_terminated_string(&accessor, _type.0);
        // TODO: this returns a hardcoded handle
        Ok(ReturnValue::U16(1))
    }

    #[api_function]
    fn load_resource(
        &self,
        module: Handle,
        res_info: Handle,
    ) -> Result<ReturnValue, EmulatorError> {
        debug!("[kernel] LOAD RESOURCE {:?} {:?}", module, res_info);
        //assert!(false);
        // TODO: this returns a hardcoded handle
        Ok(ReturnValue::U16(0xDEAD))
    }

    #[api_function]
    fn get_profile_string(
        &self,
        mut accessor: EmulatorAccessor,
        _app_name: Pointer,
        _key_name: Pointer,
        default: Pointer,
        returned_string: Pointer,
        size: u16,
    ) -> Result<ReturnValue, EmulatorError> {
        // TODO: arguments seem for some reason incorrect?
        debug!("[kernel] GET PROFILE STRING {}", size);
        debug_print_null_terminated_string(&accessor, _key_name.0);
        // TODO: honor size etc etc
        let number_of_bytes_copied = accessor.copy_string(default.0, returned_string.0)?;
        Ok(ReturnValue::U16(number_of_bytes_copied))
    }

    #[api_function]
    fn global_lock(
        &self,
        mut accessor: EmulatorAccessor,
        _h_mem: Handle,
    ) -> Result<ReturnValue, EmulatorError> {
        println!("{:?}", _h_mem);
        let segment = 0xF000;
        let offset = 0;

        // TODO: hack
        //for (i, entry) in data.iter().enumerate() {
        //    let entry = *entry;
        //    accessor.memory_mut().write_u16(0xF000 * 0x10 + (i as u32 * 2), (entry >> 8) | ((entry & 0xFF) << 8))?;
        //}

        // TODO
        Ok(ReturnValue::U32((segment << 16) | offset))
    }

    #[api_function]
    fn global_unlock(&self, _h_mem: Handle) -> Result<ReturnValue, EmulatorError> {
        // TODO
        Ok(ReturnValue::U16(1))
    }

    #[api_function]
    fn get_private_profile_int(
        &self,
        accessor: EmulatorAccessor,
        app_name: Pointer,
        key_name: Pointer,
        default: u16,
        file_name: Pointer,
    ) -> Result<ReturnValue, EmulatorError> {
        debug_print_null_terminated_string(&accessor, app_name.0);
        debug_print_null_terminated_string(&accessor, key_name.0);
        debug_print_null_terminated_string(&accessor, file_name.0);
        // TODO
        Ok(ReturnValue::U16(default))
    }

    #[api_function]
    fn get_private_profile_string(
        &self,
        mut accessor: EmulatorAccessor,
        app_name: Pointer,
        key_name: Pointer,
        default: Pointer,
        returned_string: Pointer,
        _size: u16,
        file_name: Pointer,
    ) -> Result<ReturnValue, EmulatorError> {
        debug_print_null_terminated_string(&accessor, app_name.0);
        debug_print_null_terminated_string(&accessor, key_name.0);
        debug_print_null_terminated_string(&accessor, file_name.0);
        debug_print_null_terminated_string(&accessor, default.0);
        // TODO: honor size etc etc
        let number_of_bytes_copied = accessor.copy_string(default.0, returned_string.0)?;
        // TODO: hack to force an option that causes the clock to run in analog mode
        if key_name.0 == 0x52a38 {
            for (i, c) in b"1,0,0,0,0,0".iter().enumerate() {
                accessor
                    .memory_mut()
                    .write_8(returned_string.0 + i as u32, *c)?;
            }
        }
        Ok(ReturnValue::U16(number_of_bytes_copied))
    }

    #[api_function]
    fn lstrcat(
        &self,
        mut accessor: EmulatorAccessor,
        str1: Pointer,
        str2: Pointer,
    ) -> Result<ReturnValue, EmulatorError> {
        let str1_length = accessor.strlen(str1.0)?;
        let str1_end = str1.advanced(str1_length as u32);
        println!("STR2 address: {:x}", str2.0);
        debug_print_null_terminated_string(&accessor, str2.0);
        accessor.copy_string(str2.0, str1_end.0)?;
        println!("LSTRCAT result: ");
        debug_print_null_terminated_string(&accessor, str1.0);
        Ok(ReturnValue::U32(str1.0))
    }

    #[api_function]
    fn strlen(
        &self,
        accessor: EmulatorAccessor,
        str: Pointer,
    ) -> Result<ReturnValue, EmulatorError> {
        Ok(ReturnValue::U16(accessor.strlen(str.0)?))
    }

    pub fn syscall(
        &self,
        nr: u16,
        emulator_accessor: EmulatorAccessor,
    ) -> Result<ReturnValue, EmulatorError> {
        match nr {
            3 => self.__api_get_version(emulator_accessor),
            5 => self.__api_local_alloc(emulator_accessor),
            7 => self.__api_local_free(emulator_accessor),
            18 => self.__api_global_lock(emulator_accessor),
            19 => self.__api_global_unlock(emulator_accessor),
            23 => self.__api_lock_segment(emulator_accessor),
            24 => self.__api_unlock_segment(emulator_accessor),
            30 => self.__api_wait_event(emulator_accessor),
            50 => self.__api_get_proc_address(emulator_accessor),
            51 => self.__api_make_proc_instance(emulator_accessor),
            57 => self.__api_get_profile_int(emulator_accessor),
            60 => self.__api_find_resource(emulator_accessor),
            61 => self.__api_load_resource(emulator_accessor),
            58 => self.__api_get_profile_string(emulator_accessor),
            89 => self.__api_lstrcat(emulator_accessor),
            90 => self.__api_strlen(emulator_accessor),
            91 => self.__api_init_task(emulator_accessor),
            127 => self.__api_get_private_profile_int(emulator_accessor),
            128 => self.__api_get_private_profile_string(emulator_accessor),
            132 => self.__api_get_winflags(emulator_accessor),
            nr => {
                todo!("unimplemented kernel syscall {}", nr)
            }
        }
    }
}
