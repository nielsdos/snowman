use crate::emulator_accessor::EmulatorAccessor;
use crate::registers::Registers;
use crate::EmulatorError;

pub struct EmulatedUser {}

impl EmulatedUser {
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
                todo!("unimplemented syscall {}", nr)
            }
        }
    }
}

