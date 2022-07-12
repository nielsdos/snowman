use crate::emulator_accessor::EmulatorAccessor;
use crate::registers::Registers;
use crate::util::debug_print_null_terminated_string;
use crate::EmulatorError;

pub struct EmulatedGdi {}

impl EmulatedGdi {
    pub fn new() -> Self {
        Self {}
    }

    pub fn syscall(
        &self,
        nr: u16,
        emulator_accessor: EmulatorAccessor,
    ) -> Result<(), EmulatorError> {
        match nr {
            nr => {
                todo!("unimplemented gdi syscall {}", nr)
            }
        }
    }
}
