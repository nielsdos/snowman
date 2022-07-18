use crate::api_helpers::{Pointer, ReturnValue};
use crate::emulator_accessor::EmulatorAccessor;
use crate::handle_table::{GenericHandle, Handle};
use crate::object_environment::GdiObject;
use crate::{debug, EmulatorError, ObjectEnvironment};
use std::sync::{RwLock, RwLockWriteGuard};
use syscall::api_function;

pub struct EmulatedGdi<'a> {
    objects: &'a RwLock<ObjectEnvironment<'a>>,
}

impl<'a> EmulatedGdi<'a> {
    pub fn new(objects: &'a RwLock<ObjectEnvironment<'a>>) -> Self {
        Self { objects }
    }

    fn write_objects(&self) -> RwLockWriteGuard<'_, ObjectEnvironment<'a>> {
        self.objects.write().unwrap()
    }

    #[api_function]
    fn create_dc(
        &self,
        _driver: Pointer,
        _device: Pointer,
        _port: Pointer,
        _pdm: Pointer,
    ) -> Result<ReturnValue, EmulatorError> {
        // TODO: this always indicates failure right now
        Ok(ReturnValue::U16(0))
    }

    #[api_function]
    fn delete_dc(&self, hdc: Handle) -> Result<ReturnValue, EmulatorError> {
        debug!("[gdi] DELETE DC {:?}", hdc);
        // TODO: this always indicates success right now
        Ok(ReturnValue::U16(1))
    }

    #[api_function]
    fn get_device_caps(&self, _hdc: Handle, _index: u16) -> Result<ReturnValue, EmulatorError> {
        // TODO
        Ok(ReturnValue::U16(0))
    }

    #[api_function]
    fn add_font_resource(&self, _pointer: Pointer) -> Result<ReturnValue, EmulatorError> {
        // TODO: this always indicates failure right now
        Ok(ReturnValue::U16(0))
    }

    #[api_function]
    fn create_solid_brush(&self, color: u32) -> Result<ReturnValue, EmulatorError> {
        // TODO: do we have to take into account the alpha channel?
        let color = crate::bitmap::Color::from(color);
        let handle = self
            .write_objects()
            .gdi
            .register(GdiObject::SolidBrush(color))
            .unwrap_or(Handle::null());
        Ok(ReturnValue::U16(handle.as_u16()))
    }

    #[api_function]
    fn delete_object(&self, handle: Handle) -> Result<ReturnValue, EmulatorError> {
        // TODO: which objects may get deleted?
        // TODO: check if it is selected into a DC, in that case: fail ?
        Ok(ReturnValue::U16(
            self.write_objects().gdi.deregister(handle) as u16,
        ))
    }

    pub fn syscall(
        &self,
        nr: u16,
        emulator_accessor: EmulatorAccessor,
    ) -> Result<ReturnValue, EmulatorError> {
        match nr {
            53 => self.__api_create_dc(emulator_accessor),
            66 => self.__api_create_solid_brush(emulator_accessor),
            68 => self.__api_delete_dc(emulator_accessor),
            69 => self.__api_delete_object(emulator_accessor),
            80 => self.__api_get_device_caps(emulator_accessor),
            119 => self.__api_add_font_resource(emulator_accessor),
            nr => {
                todo!("unimplemented gdi syscall {}", nr)
            }
        }
    }
}
