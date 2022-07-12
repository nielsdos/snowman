use crate::emulator_accessor::EmulatorAccessor;
use crate::registers::Registers;
use crate::EmulatorError;

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
        let pointer_argument = accessor.pointer_argument(0)?;
        println!("WSPRINTF [TODO]");
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
