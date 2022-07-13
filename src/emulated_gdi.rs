use crate::emulator_accessor::EmulatorAccessor;
use crate::registers::Registers;
use crate::EmulatorError;

pub struct EmulatedGdi {}

impl EmulatedGdi {
    pub fn new() -> Self {
        Self {}
    }

    fn create_dc(&self, mut emulator_accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let pdm = emulator_accessor.pointer_argument(0)?;
        let port = emulator_accessor.pointer_argument(2)?;
        let device = emulator_accessor.pointer_argument(4)?;
        let driver = emulator_accessor.pointer_argument(6)?;
        println!("CREATE DC {:x} {:x} {:x} {:x}", driver, device, port, pdm);

        // TODO: this always indicates failure right now
        emulator_accessor
            .regs_mut()
            .write_gpr_16(Registers::REG_AX, 0);

        Ok(())
    }

    fn delete_dc(&self, mut emulator_accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let hdc = emulator_accessor.word_argument(0)?;
        println!("DELETE DC {:x}", hdc);

        // TODO: this always indicates success right now
        emulator_accessor
            .regs_mut()
            .write_gpr_16(Registers::REG_AX, 1);

        Ok(())
    }

    fn get_device_caps(
        &self,
        mut emulator_accessor: EmulatorAccessor,
    ) -> Result<(), EmulatorError> {
        let index = emulator_accessor.word_argument(0)?;
        let hdc = emulator_accessor.word_argument(1)?;
        println!("GET DEVICE CAPS {:x} {:x}", hdc, index);

        // TODO
        emulator_accessor
            .regs_mut()
            .write_gpr_16(Registers::REG_AX, 0);

        Ok(())
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
            53 => self.create_dc(emulator_accessor),
            68 => self.delete_dc(emulator_accessor),
            80 => self.get_device_caps(emulator_accessor),
            119 => self.add_font_resource(emulator_accessor),
            nr => {
                todo!("unimplemented gdi syscall {}", nr)
            }
        }
    }
}
