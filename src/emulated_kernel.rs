use crate::constants::{WF_80x87, WF_CPU386, WF_ENHANCED, WF_PAGING, WF_PMODE};
use crate::emulator_accessor::EmulatorAccessor;
use crate::module::Module;
use crate::registers::Registers;
use crate::{debug_print_null_terminated_string, EmulatorError};

pub struct EmulatedKernel {}

impl EmulatedKernel {
    pub fn new() -> Self {
        Self {}
    }

    fn get_version(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        println!("GET VERSION");
        // Report version Windows 3.10
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, 0x0A03);
        Ok(())
    }

    fn get_winflags(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        println!("GET WINFLAGS");
        accessor.regs_mut().write_gpr_16(
            Registers::REG_AX,
            WF_80x87 | WF_PAGING | WF_CPU386 | WF_PMODE | WF_ENHANCED,
        );
        accessor.regs_mut().write_gpr_16(Registers::REG_DX, 0);
        Ok(())
    }

    fn init_task(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        println!("INIT TASK");

        let regs = accessor.regs_mut();

        // TODO: hardcoded to inittask rn
        regs.write_gpr_16(Registers::REG_AX, 0x10); // TODO: must be = DS I believe
        regs.write_gpr_16(Registers::REG_BX, 0x1234); // TODO: offset into command line
        regs.write_gpr_16(Registers::REG_CX, 0); // TODO: stack limit
        regs.write_gpr_16(Registers::REG_DX, 0); // TODO: nCmdShow
        regs.write_gpr_16(Registers::REG_SI, 0); // TODO: previous instance handle
        regs.write_gpr_16(Registers::REG_DI, 0xBEEF); // TODO: instance handle
        regs.write_gpr_16(Registers::REG_BP, regs.read_gpr_16(Registers::REG_SP));
        // TODO: segments
        regs.write_segment(Registers::REG_ES, 0x10); // TODO

        Ok(())
    }

    fn lock_segment(&self, accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        println!("LOCK SEGMENT {:x}", accessor.word_argument(0)?);
        // TODO
        Ok(())
    }

    fn unlock_segment(&self, accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        println!("UNLOCK SEGMENT {:x}", accessor.word_argument(0)?);
        // TODO
        Ok(())
    }

    fn wait_event(&self) -> Result<(), EmulatorError> {
        println!("WAIT EVENT");

        Ok(())
        // TODO?
    }

    fn make_proc_instance(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        // As we don't share segments in the same way as a 16-bit Windows might do,
        // we don't need to set up any thunks. We just need to make sure the return value is
        // equal to the original function address.
        let segment_of_function = accessor.word_argument(2)?;
        let offset_of_function = accessor.word_argument(1)?;
        //let h_instance = accessor.number_argument(0)?;
        accessor
            .regs_mut()
            .write_gpr_16(Registers::REG_AX, offset_of_function);
        accessor
            .regs_mut()
            .write_gpr_16(Registers::REG_DX, segment_of_function);
        Ok(())
    }

    fn get_profile_int(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let default = accessor.word_argument(0)?;
        let key_name = accessor.pointer_argument(1)?;
        let app_name = accessor.pointer_argument(3)?;
        print!("GET PROFILE INT {}", default);
        debug_print_null_terminated_string(&accessor, key_name);
        debug_print_null_terminated_string(&accessor, app_name);
        // TODO
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, default);
        Ok(())
    }

    fn get_profile_string(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let size = accessor.dword_argument(0)?;
        let returned_string = accessor.pointer_argument(2)?;
        let default = accessor.pointer_argument(4)?;
        let key_name = accessor.pointer_argument(6)?;
        let app_name = accessor.pointer_argument(8)?;
        print!("GET PROFILE STRING {}", size);
        // TODO: honor size etc etc
        let number_of_bytes_copied = accessor.copy_string(default, returned_string)?;
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, (number_of_bytes_copied >> 16) as u16);
        accessor.regs_mut().write_gpr_16(Registers::REG_DX, number_of_bytes_copied as u16);
        Ok(())
    }

    pub fn syscall(
        &self,
        nr: u16,
        emulator_accessor: EmulatorAccessor,
    ) -> Result<(), EmulatorError> {
        match nr {
            3 => self.get_version(emulator_accessor),
            23 => self.lock_segment(emulator_accessor),
            24 => self.unlock_segment(emulator_accessor),
            30 => self.wait_event(),
            51 => self.make_proc_instance(emulator_accessor),
            57 => self.get_profile_int(emulator_accessor),
            58 => self.get_profile_string(emulator_accessor),
            91 => self.init_task(emulator_accessor),
            132 => self.get_winflags(emulator_accessor),
            nr => {
                todo!("unimplemented kernel syscall {}", nr)
            }
        }
    }
}
