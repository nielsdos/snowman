use crate::emulator_accessor::EmulatorAccessor;
use crate::registers::Registers;
use crate::util::debug_print_null_terminated_string;
use crate::EmulatorError;

pub struct EmulatedUser {}

impl EmulatedUser {
    pub fn new() -> Self {
        Self {}
    }

    fn init_app(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        println!("INIT APP {:x}", accessor.word_argument(0)?);
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, 1);
        Ok(())
    }

    fn register_class(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        println!("REGISTER CLASS {:x}", accessor.pointer_argument(0)?);
        // TODO: should return atom number, 0 indicates failure
        // TODO: now it just fakes a success with atom 1
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, 1);
        Ok(())
    }

    fn dialog_box(&self, accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let dialog_func = accessor.pointer_argument(0)?;
        let hwnd_parent = accessor.word_argument(2)?;
        let template = accessor.pointer_argument(3)?;
        let h_instance = accessor.word_argument(5)?;
        println!(
            "DIALOG BOX {:x} {:x} {:x} {:x}",
            h_instance, template, hwnd_parent, dialog_func
        );
        // TODO
        Ok(())
    }

    fn load_string(&self, accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let buffer_max = accessor.word_argument(0)?;
        let buffer = accessor.pointer_argument(1)?;
        let uid = accessor.word_argument(3)?;
        let h_instance = accessor.word_argument(4)?;
        println!(
            "LOAD STRING {:x} {:x} {:x} {:x}",
            h_instance, uid, buffer, buffer_max
        );
        // TODO
        Ok(())
    }

    fn load_cursor(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let cursor_name = accessor.pointer_argument(0)?;
        let h_instance = accessor.word_argument(2)?;
        println!("LOAD CURSOR {:x} {:x}", h_instance, cursor_name);

        // TODO: this now always returns NULL to indicate failure
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, 0);
        Ok(())
    }

    fn get_system_metrics(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let metric = accessor.word_argument(0)?;
        println!("GET SYSTEM METRICS {:x}", metric);
        // 0x16 = 1 if debug version is installed, 0 otherwise
        if metric == 0x16 {
            accessor.regs_mut().write_gpr_16(Registers::REG_AX, 1);
        } else {
            // TODO: the others
            accessor.regs_mut().write_gpr_16(Registers::REG_AX, 0);
        }
        Ok(())
    }

    fn wsprintf(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let output_buffer_ptr = accessor.pointer_argument(0)?;
        let format_string_ptr = accessor.pointer_argument(2)?;
        print!("WSPRINTF FORMAT: ");
        debug_print_null_terminated_string(&accessor, format_string_ptr);
        // TODO: implement actual sprintf, now it just copies
        accessor.copy_string(format_string_ptr, output_buffer_ptr)?;
        print!("WSPRINTF OUTPUT: ");
        debug_print_null_terminated_string(&accessor, format_string_ptr);
        Ok(())
    }

    pub fn syscall(
        &self,
        nr: u16,
        emulator_accessor: EmulatorAccessor,
    ) -> Result<(), EmulatorError> {
        match nr {
            5 => self.init_app(emulator_accessor),
            57 => self.register_class(emulator_accessor),
            87 => self.dialog_box(emulator_accessor),
            173 => self.load_cursor(emulator_accessor),
            176 => self.load_string(emulator_accessor),
            179 => self.get_system_metrics(emulator_accessor),
            420 => self.wsprintf(emulator_accessor),
            nr => {
                todo!("unimplemented user syscall {}", nr)
            }
        }
    }
}
