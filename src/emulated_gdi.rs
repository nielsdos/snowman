use crate::api_helpers::{Pointer, ReturnValue};
use crate::constants::DeviceCapRequest;
use crate::emulator_accessor::EmulatorAccessor;
use crate::handle_table::{GenericHandle, Handle};
use crate::object_environment::{GdiObject, GdiSelectionObjectType, Pen};
use crate::{debug, EmulatorError, ObjectEnvironment};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
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

    fn read_objects(&self) -> RwLockReadGuard<'_, ObjectEnvironment<'a>> {
        self.objects.read().unwrap()
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
    fn get_device_caps(&self, _hdc: Handle, index: u16) -> Result<ReturnValue, EmulatorError> {
        println!("Get caps: {}", index);
        /*
         For a 640x480 vbox screen:
         4 -> 00D0 = 208
         6 -> 009C = 156
         8 -> 0280 = 640
         A -> 01E0 = 480
        */
        let convert_to_unit = |number: u32| ((number * 1000 + 3077 / 2) / 3077) as u16;
        if index == DeviceCapRequest::HorzRes.into() {
            // TODO: screen width in pixels
            Ok(ReturnValue::U16(800))
        } else if index == DeviceCapRequest::HorzSize.into() {
            // TODO: screen width in some unit
            Ok(ReturnValue::U16(convert_to_unit(800)))
        } else if index == DeviceCapRequest::VertRes.into() {
            // TODO: screen height in pixels
            Ok(ReturnValue::U16(600))
        } else if index == DeviceCapRequest::VertSize.into() {
            // TODO: screen height in some unit
            Ok(ReturnValue::U16(convert_to_unit(600)))
        } else {
            // TODO
            Ok(ReturnValue::U16(0))
        }
    }

    #[api_function]
    fn get_stock_object(&self, index: u16) -> Result<ReturnValue, EmulatorError> {
        println!("Get stock object! {}", index);
        if index > 16 {
            Ok(ReturnValue::U16(0))
        } else {
            Ok(ReturnValue::U16(Handle::from_u16(index + 1).as_u16()))
        }
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
    fn create_pen(
        &self,
        _style: u16,
        width: u16,
        color: u32,
    ) -> Result<ReturnValue, EmulatorError> {
        let width = width.max(1);
        // TODO: validation of with wrt style
        // TODO: do we have to take into account the alpha channel?
        let color = crate::bitmap::Color::from(color);
        let handle = self
            .write_objects()
            .gdi
            .register(GdiObject::Pen(Pen { width, color }))
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

    #[api_function]
    fn muldiv(&self, a: u16, b: u16, c: u16) -> Result<ReturnValue, EmulatorError> {
        let mul = (a as u32) * (b as u32);
        // Add half the denominator for rounding
        let mul_with_half_denominator = mul.wrapping_add((c as u32) >> 1);
        let result = mul_with_half_denominator
            .checked_div(c as u32)
            .and_then(|result| u16::try_from(result).ok())
            .unwrap_or(0xffff);
        println!("MULDIV: {} {} {} = {}", a, b, c, result);
        Ok(ReturnValue::U16(result))
    }

    #[api_function]
    fn set_bk_mode(&self, hdc: Handle, mode: u16) -> Result<ReturnValue, EmulatorError> {
        println!("SET BK MODE: {:?} {}", hdc, mode);
        Ok(ReturnValue::U16(1)) // TODO: old bg mode
    }

    #[api_function]
    fn select_object(&self, hdc: Handle, object: Handle) -> Result<ReturnValue, EmulatorError> {
        let mut objects = self.write_objects();
        let selection_type = {
            match objects.gdi.get(object) {
                Some(GdiObject::SolidBrush(_)) => GdiSelectionObjectType::SolidBrush,
                Some(GdiObject::Pen(_)) => GdiSelectionObjectType::Pen,
                _ => GdiSelectionObjectType::Invalid,
            }
        };
        let return_value = match objects.gdi.get_mut(hdc) {
            Some(GdiObject::DC(dc)) => dc.select(selection_type, object),
            _ => Handle::null(),
        };
        Ok(ReturnValue::U16(return_value.as_u16()))
    }

    #[api_function]
    fn rectangle(&self, hdc: Handle, left: i16, top: i16, right: i16, bottom: i16) -> Result<ReturnValue, EmulatorError> {
        println!("{:?} {} {} {} {}", hdc, left, top, right, bottom);

        assert!(false);
        Ok(ReturnValue::U16(0))
    }

    pub fn syscall(
        &self,
        nr: u16,
        emulator_accessor: EmulatorAccessor,
    ) -> Result<ReturnValue, EmulatorError> {
        match nr {
            2 => self.__api_set_bk_mode(emulator_accessor),
            27 => self.__api_rectangle(emulator_accessor),
            45 => self.__api_select_object(emulator_accessor),
            53 => self.__api_create_dc(emulator_accessor),
            61 => self.__api_create_pen(emulator_accessor),
            66 => self.__api_create_solid_brush(emulator_accessor),
            68 => self.__api_delete_dc(emulator_accessor),
            69 => self.__api_delete_object(emulator_accessor),
            80 => self.__api_get_device_caps(emulator_accessor),
            87 => self.__api_get_stock_object(emulator_accessor),
            119 => self.__api_add_font_resource(emulator_accessor),
            128 => self.__api_muldiv(emulator_accessor),
            nr => {
                todo!("unimplemented gdi syscall {}", nr)
            }
        }
    }
}
