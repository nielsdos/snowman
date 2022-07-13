use crate::emulator_accessor::EmulatorAccessor;
use crate::registers::Registers;
use crate::EmulatorError;

pub struct EmulatedGdi {}

impl EmulatedGdi {
    pub fn new() -> Self {
        Self {}
    }

    fn add_font_resource(
        &self,
        mut emulator_accessor: EmulatorAccessor,
    ) -> Result<(), EmulatorError> {
        let pointer = emulator_accessor.pointer_argument(0)?;
        println!("ADD FONT RESOURCE {:x}", pointer);

        // TODO: this always indicates failure right now
        emulator_accessor
            .regs_mut()
            .write_gpr_16(Registers::REG_AX, 0);

        Ok(())
    }

    pub fn syscall(
        &self,
        nr: u16,
        emulator_accessor: EmulatorAccessor,
    ) -> Result<(), EmulatorError> {
        match nr {
            119 => self.add_font_resource(emulator_accessor),
            nr => {
                todo!("unimplemented gdi syscall {}", nr)
            }
        }
    }
}
