use crate::emulated::EmulatedComponentInformationProvider;
use crate::registers::Registers;

pub struct EmulatedKernel {}

impl EmulatedKernel {
    pub fn new() -> Self {
        Self {}
    }

    fn init_task(&self, regs: &mut Registers) {
        println!("INIT TASK");

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
    }

    fn lock_segment(&self, regs: &mut Registers) {
        // TODO: read arguments from stack somehow
        println!("LOCK SEGMENT");
    }

    fn unlock_segment(&self) {
        println!("UNLOCK SEGMENT");
        // TODO?
    }

    fn wait_event(&self) {
        println!("WAIT EVENT");
        // TODO?
    }

    pub fn syscall(&self, regs: &mut Registers) {
        match regs.read_gpr_16(Registers::REG_AX) {
            23 => {
                self.lock_segment(regs);
            }
            24 => {
                self.unlock_segment();
            }
            30 => {
                self.wait_event();
            }
            91 => {
                self.init_task(regs);
            }
            nr => {
                // TODO
                println!("unimplemented system call {}", nr);
                todo!();
            }
        }
    }
}

impl EmulatedComponentInformationProvider for EmulatedKernel {
    fn argument_bytes_of_procedure(&self, procedure: u16) -> u16 {
        match procedure {
            23 | 24 | 30 => 2,
            _ => 0,
        }
    }
}
