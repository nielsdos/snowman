use crate::api_helpers::{Pointer, ReturnValue};
use crate::constants::DeviceCapRequest;
use crate::emulator_accessor::EmulatorAccessor;
use crate::handle_table::{GenericHandle, Handle};
use crate::object_environment::{GdiObject, GdiSelectionObjectType, Pen};
use crate::two_d::{Point, Rect};
use crate::util::encode_u16_u16_to_u32;
use crate::{debug, EmulatorError, ObjectEnvironment};
use num_traits::FromPrimitive;
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
        } else if index == DeviceCapRequest::NumColors.into() {
            // 1 is for higher than 8bit color depths
            Ok(ReturnValue::U16(convert_to_unit(1)))
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
    fn create_pen(&self, style: u16, width: u16, color: u32) -> Result<ReturnValue, EmulatorError> {
        let width = width.max(1);
        // TODO: validation of with wrt style
        // TODO: do we have to take into account the alpha channel?
        let color = crate::bitmap::Color::from(color);
        let handle = self
            .write_objects()
            .gdi
            .register(GdiObject::Pen(Pen { width, color }))
            .unwrap_or(Handle::null());
        println!("PEN: {:?} {} {} {:?}", handle, style, width, color);
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
    fn muldiv(&self, a: i16, b: i16, c: i16) -> Result<ReturnValue, EmulatorError> {
        let mul = (a as i32) * (b as i32);
        // Add half the denominator for rounding
        let mul_with_half_denominator = if mul < 0 {
            mul.wrapping_sub((c as i32) / 2)
        } else {
            mul.wrapping_add((c as i32) / 2)
        };
        let result = mul_with_half_denominator
            .checked_div(c as i32)
            .and_then(|result| i16::try_from(result).ok())
            .unwrap_or(-1);
        println!("MULDIV: {} {} {} = {}", a, b, c, result);
        Ok(ReturnValue::U16(result as u16))
    }

    #[api_function]
    fn set_bk_mode(&self, hdc: Handle, mode: u16) -> Result<ReturnValue, EmulatorError> {
        println!("SET BK MODE: {:?} {}", hdc, mode);
        match self.read_objects().gdi.get(hdc) {
            Some(GdiObject::DC(_)) => {}
            _ => {
                println!("Not a DC???");
            }
        };
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
    fn rectangle(
        &self,
        h_dc: Handle,
        left: i16,
        top: i16,
        right: i16,
        bottom: i16,
    ) -> Result<ReturnValue, EmulatorError> {
        let objects = self.read_objects();
        objects.with_paint_bitmap_for(h_dc, &|mut bitmap, device_context| {
            if let (Some(GdiObject::SolidBrush(brush)), Some(GdiObject::Pen(pen))) = (
                objects.gdi.get(device_context.selected_brush),
                objects.gdi.get(device_context.selected_pen),
            ) {
                bitmap.fill_rectangle(
                    Rect {
                        top,
                        left,
                        bottom,
                        right,
                    },
                    *brush,
                );
                bitmap.outline_rectangle(
                    Rect {
                        top,
                        left,
                        bottom,
                        right,
                    },
                    pen,
                );
            }
        });

        // TODO
        Ok(ReturnValue::U16(1))
    }

    #[api_function]
    fn move_to(&self, hdc: Handle, x: i16, y: i16) -> Result<ReturnValue, EmulatorError> {
        match self.write_objects().gdi.get_mut(hdc) {
            Some(GdiObject::DC(dc)) => {
                let old_position = dc.position.get();
                dc.move_to(Point::new(x, y));
                Ok(ReturnValue::U32(encode_u16_u16_to_u32(
                    old_position.x as u16,
                    old_position.y as u16,
                )))
            }
            _ => Ok(ReturnValue::U32(0)),
        }
    }

    #[api_function]
    fn line_to(&self, hdc: Handle, x: i16, y: i16) -> Result<ReturnValue, EmulatorError> {
        let objects = self.read_objects();
        self.read_objects()
            .with_paint_bitmap_for(hdc, &|mut bitmap, dc| {
                if let Some(GdiObject::Pen(pen)) = objects.gdi.get(dc.selected_pen) {
                    let to = Point::new(x, y);
                    println!(
                        "OP: {:?}, color {:?}, {:?}",
                        dc.raster_op, pen.color, dc.selected_pen
                    );
                    bitmap.line_to(to, pen);
                    dc.position.set(to);
                }
            });
        // TODO
        Ok(ReturnValue::U16(1))
    }

    #[api_function]
    fn set_rop2(&self, hdc: Handle, rop2: u16) -> Result<ReturnValue, EmulatorError> {
        println!("SET ROP2 {:?} {:x}", hdc, rop2);

        if let Some(GdiObject::DC(dc)) = self.write_objects().gdi.get_mut(hdc) {
            if let Some(raster_op) = FromPrimitive::from_u16(rop2) {
                let old = dc.raster_op;
                dc.raster_op = raster_op;
                return Ok(ReturnValue::U16(old.into()));
            }
        }

        Ok(ReturnValue::U16(0))
    }

    pub fn syscall(
        &self,
        nr: u16,
        emulator_accessor: EmulatorAccessor,
    ) -> Result<ReturnValue, EmulatorError> {
        match nr {
            2 => self.__api_set_bk_mode(emulator_accessor),
            4 => self.__api_set_rop2(emulator_accessor),
            19 => self.__api_line_to(emulator_accessor),
            20 => self.__api_move_to(emulator_accessor),
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
