use crate::api_helpers::ReturnValue;
use crate::emulator_accessor::EmulatorAccessor;
use crate::EmulatorError;

pub struct EmulatedKeyboard {}

impl EmulatedKeyboard {
    pub fn new() -> Self {
        Self {}
    }

    pub fn syscall(
        &self,
        nr: u16,
        _emulator_accessor: EmulatorAccessor,
    ) -> Result<ReturnValue, EmulatorError> {
        match nr {
            nr => {
                todo!("unimplemented keyboard syscall {}", nr)
            }
        }
    }
}
