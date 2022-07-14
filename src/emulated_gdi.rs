use std::sync::{Mutex, MutexGuard};
use crate::emulator_accessor::EmulatorAccessor;
use crate::registers::Registers;
use crate::{debug, EmulatorError, ObjectEnvironment};
use crate::handle_table::{GenericHandle, Handle};
use crate::object_environment::GdiObject;

pub struct EmulatedGdi<'a> {
    objects: &'a Mutex<ObjectEnvironment<'a>>,
}

impl<'a> EmulatedGdi<'a> {
    pub fn new(objects: &'a Mutex<ObjectEnvironment<'a>>) -> Self {
        Self {
            objects,
        }
    }

    fn objects(&self) -> MutexGuard<'_, ObjectEnvironment<'a>> {
        self.objects.lock().unwrap()
    }

    fn create_dc(&self, mut emulator_accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let pdm = emulator_accessor.pointer_argument(0)?;
        let port = emulator_accessor.pointer_argument(2)?;
        let device = emulator_accessor.pointer_argument(4)?;
        let driver = emulator_accessor.pointer_argument(6)?;
        debug!(
            "[gdi] CREATE DC {:x} {:x} {:x} {:x}",
            driver, device, port, pdm
        );

        // TODO: this always indicates failure right now
        emulator_accessor
            .regs_mut()
            .write_gpr_16(Registers::REG_AX, 0);

        Ok(())
    }

    fn delete_dc(&self, mut emulator_accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let hdc = emulator_accessor.word_argument(0)?;
        debug!("[gdi] DELETE DC {:x}", hdc);

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
        debug!("[gdi] GET DEVICE CAPS {:x} {:x}", hdc, index);

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
        debug!("[gdi] ADD FONT RESOURCE {:x}", pointer);

        // TODO: this always indicates failure right now
        emulator_accessor
            .regs_mut()
            .write_gpr_16(Registers::REG_AX, 0);

        Ok(())
    }

    fn create_solid_brush(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        // TODO: do we have to take into account the alpha channel?
        let color = accessor.dword_argument(0)?;
        debug!("CREATE SOLID BRUSH {:x} {:?}", color, crate::bitmap::Color::from(color));
        let color = crate::bitmap::Color::from(color);
        let handle = self.objects().gdi.register(GdiObject::SolidBrush(color)).unwrap_or(Handle::null());
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, handle.as_u16());
        Ok(())
    }

    fn delete_object(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        // TODO: which objects may get deleted?
        let handle = accessor.word_argument(0)?;
        // TODO: check if it is selected into a DC, in that case: fail
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, self.objects().gdi.deregister(handle.into()) as u16);
        Ok(())
    }

    pub fn syscall(
        &self,
        nr: u16,
        emulator_accessor: EmulatorAccessor,
    ) -> Result<(), EmulatorError> {
        match nr {
            53 => self.create_dc(emulator_accessor),
            66 => self.create_solid_brush(emulator_accessor),
            68 => self.delete_dc(emulator_accessor),
            69 => self.delete_object(emulator_accessor),
            80 => self.get_device_caps(emulator_accessor),
            119 => self.add_font_resource(emulator_accessor),
            nr => {
                todo!("unimplemented gdi syscall {}", nr)
            }
        }
    }
}
