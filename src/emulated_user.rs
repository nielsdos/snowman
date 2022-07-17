use crate::atom_table::AtomTable;
use crate::byte_string::{ByteString, HeapByteString};
use crate::constants::MessageType;
use crate::emulator_accessor::EmulatorAccessor;
use crate::handle_table::{GenericHandle, Handle};
use crate::memory::SegmentAndOffset;
use crate::message_queue::MessageQueue;
use crate::object_environment::{GdiObject, ObjectEnvironment, UserObject, UserWindow};
use crate::registers::Registers;
use crate::util::debug_print_null_terminated_string;
use crate::window_manager::{ProcessId, WindowIdentifier};
use crate::{debug, EmulatorError};
use std::collections::HashMap;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use sdl2::keyboard::Keycode::Return;
use syscall::api_function;
use crate::api_helpers::{Pointer, ReturnValue};

#[allow(dead_code)]
#[derive(Debug)]
struct WindowClass<'a> {
    style: u16,
    proc: SegmentAndOffset,
    cls_extra: u16,
    wnd_extra: u16,
    h_icon: Handle,
    h_cursor: Handle,
    h_background: Handle,
    menu_class_name: Option<ByteString<'a>>,
}

// TODO: figure out which parts here need to be shared and in case of sharing, what needs to be protected
pub struct EmulatedUser<'a> {
    user_atom_table: AtomTable<'a>,
    window_classes: HashMap<ByteString<'a>, WindowClass<'a>>,
    objects: &'a RwLock<ObjectEnvironment<'a>>,
}

impl<'a> EmulatedUser<'a> {
    pub fn new(objects: &'a RwLock<ObjectEnvironment<'a>>) -> Self {
        let mut window_classes = HashMap::new();
        /*window_classes.insert(
            ByteString::from_slice(b"BUTTON"),
            WindowClass {
                style: 0, // TODO
                proc: SegmentAndOffset {
                    segment: 0x1234,
                    offset: 0,
                },
                cls_extra: 0,
                wnd_extra: 0,
                h_icon: Handle::null(),
                h_cursor: Handle::null(),
                h_background: Handle::null(), // TODO
                menu_class_name: None,
            },
        );*/
        Self {
            user_atom_table: AtomTable::new(),
            window_classes,
            objects,
        }
    }

    #[api_function]
    fn init_app(&self, _arg1: u16) -> Result<ReturnValue, EmulatorError> {
        Ok(ReturnValue::U16(1))
    }

    #[api_function]
    fn create_window(&mut self, mut accessor: EmulatorAccessor, class_name: Pointer, window_name: HeapByteString, style: u32, x: u16, y: u16, width: u16, height: u16, h_wnd_parent: Handle, h_menu: Handle, h_instance: Handle, param: Pointer) -> Result<ReturnValue, EmulatorError> {
        let class_name = accessor.static_string(class_name.0)?;
        println!("  > class name: {:?}", class_name);

        // TODO: support atom lookup here (that's the case if segment == 0)
        if let Some(class) = self
            .window_classes
            .get(&class_name)
        {
            let user_window = UserWindow {
                proc: class.proc,
                message_queue: MessageQueue::new(),
            };
            let proc = user_window.proc;
            let window_handle = self
                .write_objects()
                .user
                .register(UserObject::Window(user_window))
                .unwrap_or(Handle::null());
            if window_handle != Handle::null() {
                self.write_objects().window_manager().create_window(
                    WindowIdentifier {
                        window_handle,
                        process_id: self.process_id(),
                    },
                    x,
                    y,
                    width,
                    height,
                );

                // TODO: l_param should get a pointer to a CREATESTRUCT that contains info about the window being created
                self.call_wndproc_sync(
                    accessor,
                    proc,
                    window_handle,
                    MessageType::Create,
                    0,
                    0,
                )?;
                return Ok(ReturnValue::DelayedU16(window_handle.as_u16()));
            }
        }
        Ok(ReturnValue::U16(Handle::null().as_u16()))
    }

    fn read_objects(&self) -> RwLockReadGuard<'_, ObjectEnvironment<'a>> {
        self.objects.read().unwrap()
    }

    fn write_objects(&self) -> RwLockWriteGuard<'_, ObjectEnvironment<'a>> {
        self.objects.write().unwrap()
    }

    fn process_id(&self) -> ProcessId {
        // TODO
        ProcessId::null()
    }

    #[api_function]
    fn show_window(&self, h_wnd: Handle, cmd_show: u16) -> Result<ReturnValue, EmulatorError> {
        let objects = self.write_objects();
        let success = match objects.user.get(h_wnd) {
            Some(UserObject::Window(_)) => {
                // TODO: do something with cmd_show
                objects.window_manager().show_window(WindowIdentifier {
                    window_handle: h_wnd,
                    process_id: self.process_id(),
                });
                true
            }
            None => false,
        };
        Ok(ReturnValue::U16(success.into()))
    }

    #[api_function]
    fn update_window(&self, accessor: EmulatorAccessor, h_wnd: Handle) -> Result<ReturnValue, EmulatorError> {
        let success = match self.write_objects().user.get(h_wnd) {
            Some(UserObject::Window(user_window)) => {
                // TODO: only do this if update region is non-empty
                self.call_wndproc_sync(
                    accessor,
                    user_window.proc,
                    h_wnd,
                    MessageType::Paint,
                    0,
                    0,
                )?;
                true
            }
            None => false,
        };
        Ok(ReturnValue::DelayedU16(success.into()))
    }

    #[api_function]
    fn register_class(&mut self, mut accessor: EmulatorAccessor, wnd_class_ptr: Pointer) -> Result<ReturnValue, EmulatorError> {
        let wnd_class_style = accessor.memory().read_16(wnd_class_ptr.0)?;
        let wnd_class_proc_offset = accessor.memory().read_16(wnd_class_ptr.0 + 2)?;
        let wnd_class_proc_segment = accessor.memory().read_16(wnd_class_ptr.0 + 4)?;
        let wnd_class_cls_extra = accessor.memory().read_16(wnd_class_ptr.0 + 6)?;
        let wnd_class_wnd_extra = accessor.memory().read_16(wnd_class_ptr.0 + 8)?;
        let _wnd_class_h_instance = accessor.memory().read_16(wnd_class_ptr.0 + 10)?;
        let wnd_class_h_icon = accessor.memory().read_16(wnd_class_ptr.0 + 12)?;
        let wnd_class_h_cursor = accessor.memory().read_16(wnd_class_ptr.0 + 14)?;
        let wnd_class_h_background = accessor.memory().read_16(wnd_class_ptr.0 + 16)?;
        let wnd_class_menu_name = accessor.memory().flat_pointer_read(wnd_class_ptr.0 + 18)?;
        let wnd_class_class_name = accessor.memory().flat_pointer_read(wnd_class_ptr.0 + 22)?;

        let cloned_class_name = accessor.clone_string(wnd_class_class_name)?;
        if let Some(atom) = self
            .user_atom_table
            .register(cloned_class_name.clone().into())
        {
            let window_class = WindowClass {
                style: wnd_class_style,
                proc: SegmentAndOffset {
                    segment: wnd_class_proc_segment,
                    offset: wnd_class_proc_offset,
                },
                cls_extra: wnd_class_cls_extra,
                wnd_extra: wnd_class_wnd_extra,
                h_icon: wnd_class_h_icon.into(),
                h_cursor: wnd_class_h_cursor.into(),
                h_background: wnd_class_h_background.into(),
                menu_class_name: if wnd_class_menu_name != 0 {
                    Some(accessor.clone_string(wnd_class_menu_name)?.into())
                } else {
                    None
                },
            };

            debug!(
                "[user] REGISTER CLASS SUCCESS {:?} => {:#?}",
                cloned_class_name, window_class
            );
            if self
                .window_classes
                .insert(cloned_class_name.into(), window_class)
                .is_none()
            {
                return Ok(ReturnValue::U16(atom.as_u16()));
            }

            self.user_atom_table.deregister(atom);
        }

        Ok(ReturnValue::U16(0))
    }

    #[api_function]
    fn dialog_box(&self, accessor: EmulatorAccessor, h_instance: Handle, template: Pointer, h_wnd_parent: Handle, dialog_func: u32) -> Result<ReturnValue, EmulatorError> {
        println!(
            "DIALOG BOX {:?} {:x} {:?} {:x}",
            h_instance, template.0, h_wnd_parent, dialog_func
        );
        Ok(ReturnValue::U16(0))
    }

    #[api_function]
    fn get_message(&self, msg: u32, h_wnd: Handle, msg_filter_min: u16, msg_filer_max: u16) -> Result<ReturnValue, EmulatorError> {
        let message = match self.read_objects().user.get(h_wnd.into()) {
            Some(UserObject::Window(user_window)) => user_window.message_queue.receive(),
            _ => None,
        };

        // TODO: implement filters
        // TODO: support hwnd being null
        let return_value = if let Some(message) = message {
            // TODO: write message

            if message.message == MessageType::Quit {
                0
            } else {
                1
            }
        } else {
            println!("error");
            0xffff
        };
        Ok(ReturnValue::U16(return_value))
    }

    #[api_function]
    fn load_string(&self, h_instance: Handle, uid: u16, buffer: Pointer, buffer_max: u16) -> Result<ReturnValue, EmulatorError> {
        println!(
            "LOAD STRING {:?} {:x} {:x} {:x}",
            h_instance, uid, buffer.0, buffer_max
        );
        Ok(ReturnValue::U16(0))
    }

    #[api_function]
    fn load_cursor(&self, _h_instance: Handle, _cursor_name: u32) -> Result<ReturnValue, EmulatorError> {
        Ok(ReturnValue::U16(Handle::null().as_u16()))
    }

    fn call_wndproc_sync(
        &self,
        mut accessor: EmulatorAccessor,
        proc: SegmentAndOffset,
        h_wnd: Handle,
        message: MessageType,
        w_param: u16,
        l_param: u32,
    ) -> Result<(), EmulatorError> {
        accessor.far_call_into_proc_setup()?;
        accessor.push_16(h_wnd.as_u16())?;
        accessor.push_16(message.into())?;
        accessor.push_16(w_param)?;
        accessor.push_16((l_param >> 16) as u16)?;
        accessor.push_16(l_param as u16)?;
        accessor.far_call_into_proc_execute(proc.segment, proc.offset)
    }

    #[api_function]
    fn get_system_metrics(&self, metric: u16) -> Result<ReturnValue, EmulatorError> {
        if metric == 0x16 {
            // 1 if debug version is installed, 0 otherwise
            Ok(ReturnValue::U16(1))
        } else {
            // TODO: the others
            Ok(ReturnValue::U16(0))
        }
    }

    #[api_function]
    fn wsprintf(&self, mut accessor: EmulatorAccessor, format_string_ptr: Pointer, output_buffer_ptr: Pointer) -> Result<ReturnValue, EmulatorError> {
        print!("WSPRINTF FORMAT: ");
        debug_print_null_terminated_string(&accessor, format_string_ptr.0);
        // TODO: implement actual sprintf, now it just copies
        accessor.copy_string(format_string_ptr.0, output_buffer_ptr.0)?;
        print!("WSPRINTF OUTPUT: ");
        debug_print_null_terminated_string(&accessor, format_string_ptr.0);
        Ok(ReturnValue::U16(0))
    }

    #[api_function]
    fn def_window_proc(&self, mut accessor: EmulatorAccessor, h_wnd: Handle, msg: u16, w_param: u16, l_param: u32) -> Result<ReturnValue, EmulatorError> {
        debug!(
            "[user] DEF WINDOW PROC {:?} {:x} {:x} {:x}",
            h_wnd, msg, w_param, l_param
        );
        Ok(ReturnValue::U16(0))
    }

    #[api_function]
    fn begin_paint(&self, mut accessor: EmulatorAccessor, h_wnd: Handle, paint: Pointer) -> Result<ReturnValue, EmulatorError> {
        let mut objects = self.write_objects();
        let display_device_handle_for_window = match objects.user.get(h_wnd.into()) {
            Some(UserObject::Window(_)) => {
                let window_identifier = WindowIdentifier {
                    process_id: self.process_id(),
                    window_handle: h_wnd.into(),
                };
                if let Some(handle) = objects.gdi.register(GdiObject::DC(window_identifier)) {
                    accessor.memory_mut().write_16(paint.0, handle.as_u16())?;
                    accessor.memory_mut().write_8(paint.0.wrapping_add(2), 0)?; // TODO: fErase
                    accessor.memory_mut().write_16(paint.0.wrapping_add(2), 0)?;
                    accessor.memory_mut().write_16(paint.0.wrapping_add(2), 0)?;
                    accessor.memory_mut().write_16(paint.0.wrapping_add(2), 200)?; // TODO: rcPaint.right
                    accessor.memory_mut().write_16(paint.0.wrapping_add(2), 200)?; // TODO: rcPaint.bottom
                    handle.as_u16()
                } else {
                    0
                }
            }
            None => 0,
        };
        Ok(ReturnValue::U16(display_device_handle_for_window))
    }

    #[api_function]
    fn end_paint(&self, mut accessor: EmulatorAccessor, h_wnd: Handle, paint: Pointer) -> Result<ReturnValue, EmulatorError> {
        // TODO: this should probably cause a flip of the front and back bitmap for the given window
        let handle = accessor.memory().read_16(paint.0)?;
        self.write_objects().gdi.deregister(handle.into());
        Ok(ReturnValue::U16(1))
    }

    #[api_function]
    fn fill_rect(&self, mut accessor: EmulatorAccessor, h_dc: Handle, rect: Pointer, h_brush: Handle) -> Result<ReturnValue, EmulatorError> {
        let rect = accessor.read_rect(rect.0)?;
        let objects = self.read_objects();
        if let (Some(GdiObject::DC(window_identifier)), Some(GdiObject::SolidBrush(color))) = (
            objects.gdi.get(h_dc),
            objects.gdi.get(h_brush),
        ) {
            // TODO: wat als de DC een window identifier + clip rect geeft?
            // TODO: see also CS_PARENTDC & https://devblogs.microsoft.com/oldnewthing/20120604-00/?p=7463

            if let Some(bitmap) = objects
                .window_manager()
                .paint_bitmap_for(*window_identifier)
            {
                bitmap.fill_rectangle(rect, *color)
            }
        }
        Ok(ReturnValue::U16(1))
    }

    pub fn syscall(
        &mut self,
        nr: u16,
        emulator_accessor: EmulatorAccessor,
    ) -> Result<ReturnValue, EmulatorError> {
        match nr {
            5 => self.__api_init_app(emulator_accessor),
            39 => self.__api_begin_paint(emulator_accessor),
            40 => self.__api_end_paint(emulator_accessor),
            41 => self.__api_create_window(emulator_accessor),
            42 => self.__api_show_window(emulator_accessor),
            57 => self.__api_register_class(emulator_accessor),
            81 => self.__api_fill_rect(emulator_accessor),
            87 => self.__api_dialog_box(emulator_accessor),
            107 => self.__api_def_window_proc(emulator_accessor),
            108 => self.__api_get_message(emulator_accessor),
            124 => self.__api_update_window(emulator_accessor),
            173 => self.__api_load_cursor(emulator_accessor),
            176 => self.__api_load_string(emulator_accessor),
            179 => self.__api_get_system_metrics(emulator_accessor),
            420 => self.__api_wsprintf(emulator_accessor),
            nr => {
                todo!("unimplemented user syscall {}", nr)
            }
        }
    }
}
