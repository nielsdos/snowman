use crate::emulator_accessor::EmulatorAccessor;
use crate::registers::Registers;
use crate::EmulatorError;
use crate::module::Module;

pub struct EmulatedKernel {}

impl EmulatedKernel {
    pub fn new() -> Self {
        Self {}
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
        println!("LOCK SEGMENT {:x}", accessor.argument(0)?);
        // TODO
        Ok(())
    }

    fn unlock_segment(&self, accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        println!("UNLOCK SEGMENT {:x}", accessor.argument(0)?);
        // TODO
        Ok(())
    }

    fn wait_event(&self) -> Result<(), EmulatorError> {
        println!("WAIT EVENT");

        Ok(())
        // TODO?
    }

    pub fn syscall(
        &self,
        nr: u16,
        emulator_accessor: EmulatorAccessor,
    ) -> Result<(), EmulatorError> {
        match nr {
            23 => self.lock_segment(emulator_accessor),
            24 => self.unlock_segment(emulator_accessor),
            30 => self.wait_event(),
            91 => self.init_task(emulator_accessor),
            nr => {
                todo!("unimplemented syscall {}", nr)
            }
        }
    }
}
