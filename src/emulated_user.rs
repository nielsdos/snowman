use crate::emulator_accessor::EmulatorAccessor;
use crate::registers::Registers;
use crate::EmulatorError;
use crate::util::debug_print_null_terminated_string;

pub struct EmulatedUser {}

impl EmulatedUser {
    pub fn new() -> Self {
        Self {}
    }

    fn init_app(&self, accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        println!("INIT APP {:x}", accessor.number_argument(0)?);
        // TODO
        Ok(())
    }

    fn get_system_metrics(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let metric = accessor.number_argument(0)?;
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
        let mut output_buffer_ptr = accessor.pointer_argument(0)?;
        let mut format_string_ptr = accessor.pointer_argument(2)?;
        let format_string_ptr_start = format_string_ptr;
        print!("WSPRINTF FORMAT: ");
        debug_print_null_terminated_string(&accessor, format_string_ptr);
        // TODO: implement actual sprintf, now it just copies
        loop {
            let data = accessor.memory().read_8(format_string_ptr).unwrap_or(0);
            accessor.memory_mut().write_8(output_buffer_ptr, data);
            if data == 0 {
                break;
            }
            format_string_ptr += 1;
            output_buffer_ptr += 1;
        }
        print!("WSPRINTF OUTPUT: ");
        debug_print_null_terminated_string(&accessor, format_string_ptr_start);
        Ok(())
    }

    pub fn syscall(
        &self,
        nr: u16,
        emulator_accessor: EmulatorAccessor,
    ) -> Result<(), EmulatorError> {
        match nr {
            5 => self.init_app(emulator_accessor),
            179 => self.get_system_metrics(emulator_accessor),
            420 => self.wsprintf(emulator_accessor),
            nr => {
                todo!("unimplemented user syscall {}", nr)
            }
        }
    }
}
