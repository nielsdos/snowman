use crate::api_helpers::{Pointer, ReturnValue};
use crate::atom_table::AtomTable;
use crate::bitmap::{BitmapView, Color};
use crate::byte_string::{ByteString, HeapByteString};
use crate::constants::{ClassStyles, MessageType, SystemColors};
use crate::emulator_accessor::EmulatorAccessor;
use crate::handle_table::{GenericHandle, Handle};
use crate::memory::SegmentAndOffset;
use crate::message_queue::{MessageQueue, WindowMessage};
use crate::object_environment::{
    DeviceContext, GdiObject, ObjectEnvironment, UserObject, UserWindow,
};
use crate::two_d::{Point, Rect};
use crate::util::debug_print_null_terminated_string;
use crate::window_manager::{ProcessId, WindowIdentifier};
use crate::{debug, EmulatorError, ResourceTable};
use num_traits::FromPrimitive;
use std::collections::HashMap;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use syscall::api_function;

#[allow(dead_code)]
#[derive(Debug)]
struct WindowClass<'a> {
    style: ClassStyles,
    proc: SegmentAndOffset,
    cls_extra: u16,
    wnd_extra: u16,
    h_icon: Handle,
    h_cursor: Handle,
    h_background: Handle,
    menu_class_name: Option<ByteString<'a>>,
}

struct Paint {
    hdc: Handle,
    f_erase: bool,
    rect: Rect,
}

// TODO: figure out which parts here need to be shared and in case of sharing, what needs to be protected
pub struct EmulatedUser<'a> {
    user_atom_table: AtomTable<'a>,
    resource_table: ResourceTable,
    window_classes: HashMap<ByteString<'a>, WindowClass<'a>>,
    objects: &'a RwLock<ObjectEnvironment<'a>>,
    message_queue: &'a MessageQueue,
}

impl<'a> EmulatedUser<'a> {
    pub fn new(
        objects: &'a RwLock<ObjectEnvironment<'a>>,
        message_queue: &'a MessageQueue,
        resource_table: ResourceTable,
        button_wnd_proc: SegmentAndOffset,
    ) -> Self {
        let mut window_classes = HashMap::new();
        window_classes.insert(
            ByteString::from_slice(b"BUTTON"),
            WindowClass {
                style: ClassStyles::PARENT_DC,
                proc: button_wnd_proc,
                cls_extra: 0,
                wnd_extra: 0,
                h_icon: Handle::null(),
                h_cursor: Handle::null(),
                h_background: Handle::null(),
                menu_class_name: None,
            },
        );
        Self {
            user_atom_table: AtomTable::new(),
            message_queue,
            resource_table,
            window_classes,
            objects,
        }
    }

    #[api_function]
    fn internal_get_sys_color(&self, index: u16) -> Result<ReturnValue, EmulatorError> {
        let system_color: Option<SystemColors> = FromPrimitive::from_u16(index);
        let color = system_color
            .map(|color| self.get_system_color(color))
            .unwrap_or(Color(0, 0, 0));
        Ok(ReturnValue::U32(color.as_u32()))
    }

    fn get_system_color(&self, color: SystemColors) -> Color {
        match color {
            SystemColors::Background => Color(192, 192, 192),
            SystemColors::AppWorkspace => Color(255, 255, 255),
            SystemColors::Window => Color(255, 255, 255),
            SystemColors::WindowText => Color(0, 0, 0),
            SystemColors::Menu => Color(255, 255, 255),
            SystemColors::MenuText => Color(0, 0, 0),
            SystemColors::ActiveCaption => Color(0, 0, 128),
            SystemColors::InactiveCaption => Color(255, 255, 255),
            SystemColors::WindowFrame => Color(0, 0, 0),
            SystemColors::Scrollbar => Color(192, 192, 192),
            SystemColors::ButtonFace => Color(192, 192, 192),
            SystemColors::ButtonShadow => Color(128, 128, 128),
            SystemColors::ButtonText => Color(0, 0, 0),
            SystemColors::GrayText => Color(192, 192, 192),
            SystemColors::Highlight => Color(0, 0, 128),
            SystemColors::HighlightText => Color(255, 255, 255),
            SystemColors::InactiveCaptionText => Color(0, 0, 0),
            SystemColors::ButtonHighlight => Color(255, 255, 255),
            SystemColors::CaptionText => Color(255, 255, 255),
            SystemColors::ActiveBorder => Color(192, 192, 192),
            SystemColors::InactiveBorder => Color(192, 192, 192),
        }
    }

    #[api_function]
    fn init_app(&self, _arg1: u16) -> Result<ReturnValue, EmulatorError> {
        Ok(ReturnValue::U16(1))
    }

    #[api_function]
    fn create_window(
        &mut self,
        mut accessor: EmulatorAccessor,
        class_name: Pointer,
        _window_name: HeapByteString,
        style: u32,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        h_wnd_parent: Handle,
        _h_menu: Handle,
        _h_instance: Handle,
        _param: Pointer,
    ) -> Result<ReturnValue, EmulatorError> {
        let class_name = accessor.static_string(class_name.0)?;

        println!(
            "CREATE WINDOW: {:?} {:x} {:x} {:x} {:x} {:x}",
            _window_name, style, x, y, width, height
        );

        // TODO: support atom lookup here (that's the case if segment == 0)
        if let Some(class) = self.window_classes.get(&class_name) {
            let parent_dc = class.style.contains(ClassStyles::PARENT_DC);
            let user_window = UserWindow::new(class.proc, parent_dc, h_wnd_parent);
            let proc = user_window.proc;
            let mut objects = self.write_objects();
            if let Some(window_handle) = objects.user.register(UserObject::Window(user_window)) {
                println!("window_handle = {:?}", window_handle);

                if h_wnd_parent != Handle::null() {
                    if let Some(UserObject::Window(parent_window)) =
                        objects.user.get_mut(h_wnd_parent)
                    {
                        parent_window.children.push(window_handle);
                    } else {
                        objects.user.deregister(window_handle);
                        return Ok(ReturnValue::U16(Handle::null().as_u16()));
                    }
                }

                objects.write_window_manager().create_window(
                    WindowIdentifier {
                        window_handle,
                        process_id: self.process_id(),
                    },
                    x,
                    y,
                    width,
                    height,
                    parent_dc,
                );

                // TODO: l_param should get a pointer to a CREATESTRUCT that contains info about the window being created
                self.call_wndproc_sync(
                    &mut accessor,
                    proc,
                    window_handle,
                    MessageType::Create.into(),
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
        println!("show window {:?} {:x}", h_wnd, cmd_show);

        let objects = self.write_objects();
        let success = match objects.user.get(h_wnd) {
            Some(UserObject::Window(_)) => {
                // TODO: do something with cmd_show
                objects
                    .write_window_manager()
                    .show_window(WindowIdentifier {
                        window_handle: h_wnd,
                        process_id: self.process_id(),
                    });
                true
            }
            None => false,
        };

        if success {
            // TODO: iirc this should only be done when the window was not visible yet and now became visible
            //       ... And if the message queue is empty? (not sure about this)
            self.message_queue.send(WindowMessage {
                h_wnd,
                message: MessageType::Paint,
                w_param: 0,
                l_param: 0,
                time: 0,
                point: Point::origin(),
            });
        }

        Ok(ReturnValue::U16(success.into()))
    }

    fn recursive_window_paint(
        &self,
        accessor: &mut EmulatorAccessor,
        objects: &RwLockReadGuard<ObjectEnvironment>,
        h_wnd: Handle,
    ) -> bool {
        println!("recursive window paint: {:?}", h_wnd);
        match objects.user.get(h_wnd) {
            Some(UserObject::Window(user_window)) => {
                for child in &user_window.children {
                    self.recursive_window_paint(accessor, objects, *child);
                }
                // TODO: only do this if update region is non-empty
                self.call_wndproc_sync(accessor, user_window.proc, h_wnd, MessageType::Paint.into(), 0, 0)
                    .is_ok()
            }
            _ => false,
        }
    }

    #[api_function]
    fn update_window(
        &self,
        mut accessor: EmulatorAccessor,
        h_wnd: Handle,
    ) -> Result<ReturnValue, EmulatorError> {
        let objects = self.read_objects();
        let success = self.recursive_window_paint(&mut accessor, &objects, h_wnd);
        Ok(ReturnValue::DelayedU16(success.into()))
    }

    #[api_function]
    fn register_class(
        &mut self,
        accessor: EmulatorAccessor,
        wnd_class_ptr: Pointer,
    ) -> Result<ReturnValue, EmulatorError> {
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
                style: ClassStyles::from_bits_truncate(wnd_class_style),
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
    fn dialog_box(
        &self,
        h_instance: Handle,
        template: Pointer,
        h_wnd_parent: Handle,
        dialog_func: u32,
    ) -> Result<ReturnValue, EmulatorError> {
        println!(
            "DIALOG BOX {:?} {:x} {:?} {:x}",
            h_instance, template.0, h_wnd_parent, dialog_func
        );
        Ok(ReturnValue::U16(0))
    }

    #[api_function]
    fn get_message(
        &self,
        mut accessor: EmulatorAccessor,
        msg: Pointer,
        h_wnd: Handle,
        _msg_filter_min: u16,
        _msg_filer_max: u16,
    ) -> Result<ReturnValue, EmulatorError> {
        // TODO: implement min & max filters
        let return_value = if let Some(message) = self.message_queue.receive(h_wnd) {
            let message_type = message.message;
            accessor.memory_mut().write_16(msg.0, message.h_wnd.as_u16())?;
            accessor.memory_mut().write_16(msg.0 + 2, message_type.into())?;
            accessor.memory_mut().write_16(msg.0 + 4, message.w_param)?;
            accessor.memory_mut().write_32(msg.0 + 6, message.l_param)?;
            accessor.memory_mut().write_32(msg.0 + 10, message.time)?;
            accessor.memory_mut().write_16(msg.0 + 14, message.point.x)?;
            accessor.memory_mut().write_16(msg.0 + 16, message.point.y)?;

            if message_type == MessageType::Quit {
                0
            } else {
                1
            }
        } else {
            0xFFFF
        };
        Ok(ReturnValue::U16(return_value))
    }

    #[api_function]
    fn translate_message(&self, _msg: Pointer) -> Result<ReturnValue, EmulatorError> {
        Ok(ReturnValue::U16(0))
    }

    #[api_function]
    fn dispatch_message(&self, mut accessor: EmulatorAccessor, msg: Pointer) -> Result<ReturnValue, EmulatorError> {
        let h_wnd: Handle = accessor.memory().read_16(msg.0)?.into();
        let message_type = accessor.memory().read_16(msg.0 + 2)?;
        let w_param = accessor.memory().read_16(msg.0 + 4)?;
        let l_param = accessor.memory().read_32(msg.0 + 6)?;

        match self.read_objects().user.get(h_wnd) {
            Some(UserObject::Window(user_window)) => {
                self.call_wndproc_sync(&mut accessor, user_window.proc, h_wnd, message_type, w_param, l_param)?;
            }
            _ => {},
        };

        // Return value will be set by the wnd proc
        Ok(ReturnValue::None)
    }

    #[api_function]
    fn load_string(
        &self,
        mut accessor: EmulatorAccessor,
        h_instance: Handle,
        uid: u16,
        buffer: Pointer,
        buffer_max: u16,
    ) -> Result<ReturnValue, EmulatorError> {
        println!(
            "LOAD STRING {:?} {:x} {:x} {:x}",
            h_instance, uid, buffer.0, buffer_max
        );

        // TODO: keep h_instance into account...
        if let Some(string) = self.resource_table.strings_resources.get(&uid) {
            let string = string.as_slice();
            let length = string.len();
            let amount_of_bytes_to_copy = if buffer_max == 0 {
                length
            } else {
                (buffer_max as usize).min(length)
            };

            accessor
                .memory_mut()
                .copy_from(&string[0..amount_of_bytes_to_copy], buffer.0 as usize)?;

            debug_print_null_terminated_string(&accessor, buffer.0);
            // String lengths from the resource table will fit in 16 bits, because their length
            // was originally 8 bits.
            Ok(ReturnValue::U16(amount_of_bytes_to_copy as u16))
        } else {
            Ok(ReturnValue::U16(0))
        }
    }

    #[api_function]
    fn load_cursor(
        &self,
        _h_instance: Handle,
        _cursor_name: u32,
    ) -> Result<ReturnValue, EmulatorError> {
        Ok(ReturnValue::U16(Handle::null().as_u16()))
    }

    fn call_wndproc_sync(
        &self,
        accessor: &mut EmulatorAccessor,
        proc: SegmentAndOffset,
        h_wnd: Handle,
        message: u16,
        w_param: u16,
        l_param: u32,
    ) -> Result<(), EmulatorError> {
        accessor.far_call_into_proc_setup()?;
        accessor.push_16(h_wnd.as_u16())?;
        accessor.push_16(message)?;
        accessor.push_16(w_param)?;
        accessor.push_16((l_param >> 16) as u16)?;
        accessor.push_16(l_param as u16)?;
        accessor.far_call_into_proc_execute(proc.segment, proc.offset)
    }

    #[api_function]
    fn get_system_metrics(&self, metric: u16) -> Result<ReturnValue, EmulatorError> {
        println!("metric {} requested", metric);
        // TODO: other metrics
        if metric == 0x16 {
            // 1 if debug version is installed, 0 otherwise
            Ok(ReturnValue::U16(1))
        } else if metric == 30 || metric == 31 {
            // TODO: this is just to let the system continue
            Ok(ReturnValue::U16(16))
        } else if metric == 32 || metric == 33 {
            // TODO: this is just to let the system continue
            Ok(ReturnValue::U16(4))
        } else if metric == 41 {
            // As the Windows for Pen computing extension is not installed, return 0
            Ok(ReturnValue::U16(0))
        } else {
            Ok(ReturnValue::U16(0))
        }
    }

    #[api_function]
    fn wsprintf(
        &self,
        mut accessor: EmulatorAccessor,
        format_string_ptr: Pointer,
        output_buffer_ptr: Pointer,
    ) -> Result<ReturnValue, EmulatorError> {
        print!("WSPRINTF FORMAT: ");
        debug_print_null_terminated_string(&accessor, format_string_ptr.0);
        // TODO: implement actual sprintf, now it just copies
        accessor.copy_string(format_string_ptr.0, output_buffer_ptr.0)?;
        print!("WSPRINTF OUTPUT: ");
        debug_print_null_terminated_string(&accessor, format_string_ptr.0);
        Ok(ReturnValue::U16(0))
    }

    #[api_function]
    fn def_window_proc(
        &self,
        h_wnd: Handle,
        msg: u16,
        w_param: u16,
        l_param: u32,
    ) -> Result<ReturnValue, EmulatorError> {
        debug!(
            "[user] DEF WINDOW PROC {:?} {:x} {:x} {:x}",
            h_wnd, msg, w_param, l_param
        );
        Ok(ReturnValue::U16(0))
    }

    #[api_function]
    fn button_window_proc(
        &self,
        h_wnd: Handle,
        msg: u16,
        w_param: u16,
        l_param: u32,
    ) -> Result<ReturnValue, EmulatorError> {
        debug!(
            "[user] BUTTON WINDOW PROC {:?} {:x} {:x} {:x}",
            h_wnd, msg, w_param, l_param
        );
        if msg == MessageType::Paint.into() {
            // Paint button
            if let Some(paint) = self.begin_paint(h_wnd) {
                let objects = self.read_objects();
                let containing_rect = self.get_client_rect(h_wnd, &objects);
                self.with_paint_bitmap_for(paint.hdc, &objects, &|mut bitmap| {
                    // Black rounded frame
                    bitmap.draw_horizontal_line(
                        1,
                        0,
                        containing_rect.right.wrapping_sub(1),
                        self.get_system_color(SystemColors::WindowFrame),
                    );
                    bitmap.draw_horizontal_line(
                        1,
                        containing_rect.bottom.saturating_sub(1),
                        containing_rect.right.saturating_sub(1),
                        self.get_system_color(SystemColors::WindowFrame),
                    );
                    bitmap.draw_vertical_line(
                        0,
                        1,
                        containing_rect.bottom.saturating_sub(1),
                        self.get_system_color(SystemColors::WindowFrame),
                    );
                    bitmap.draw_vertical_line(
                        containing_rect.right.saturating_sub(1),
                        1,
                        containing_rect.bottom.saturating_sub(1),
                        self.get_system_color(SystemColors::WindowFrame),
                    );

                    // Highlight top
                    bitmap.draw_horizontal_line(
                        1,
                        1,
                        containing_rect.right.saturating_sub(1),
                        self.get_system_color(SystemColors::ButtonHighlight),
                    );
                    bitmap.draw_horizontal_line(
                        1,
                        2,
                        containing_rect.right.saturating_sub(2),
                        self.get_system_color(SystemColors::ButtonHighlight),
                    );
                    // Highlight left
                    bitmap.draw_vertical_line(
                        1,
                        3,
                        containing_rect.bottom.saturating_sub(1),
                        self.get_system_color(SystemColors::ButtonHighlight),
                    );
                    bitmap.draw_vertical_line(
                        2,
                        3,
                        containing_rect.bottom.saturating_sub(2),
                        self.get_system_color(SystemColors::ButtonHighlight),
                    );

                    // Shadow right
                    bitmap.draw_vertical_line(
                        containing_rect.right.saturating_sub(2),
                        1,
                        containing_rect.bottom.saturating_sub(3),
                        self.get_system_color(SystemColors::ButtonShadow),
                    );
                    bitmap.draw_vertical_line(
                        containing_rect.right.saturating_sub(3),
                        2,
                        containing_rect.bottom.saturating_sub(3),
                        self.get_system_color(SystemColors::ButtonShadow),
                    );
                    // Shadow bottom
                    bitmap.draw_horizontal_line(
                        2,
                        containing_rect.bottom.saturating_sub(3),
                        containing_rect.right.saturating_sub(1),
                        self.get_system_color(SystemColors::ButtonShadow),
                    );
                    bitmap.draw_horizontal_line(
                        1,
                        containing_rect.bottom.saturating_sub(2),
                        containing_rect.right.saturating_sub(1),
                        self.get_system_color(SystemColors::ButtonShadow),
                    );

                    // Face
                    let bg_rect = containing_rect.shrink(3);
                    bitmap.fill_rectangle(bg_rect, self.get_system_color(SystemColors::ButtonFace));
                });
                drop(objects);
                self.end_paint(h_wnd, paint.hdc);
            }
        }
        Ok(ReturnValue::U16(0))
    }

    fn get_client_rect(&self, h_wnd: Handle, objects: &RwLockReadGuard<ObjectEnvironment>) -> Rect {
        objects
            .read_window_manager()
            .client_rect_of(WindowIdentifier {
                process_id: self.process_id(),
                window_handle: h_wnd,
            })
    }

    fn get_dc(&self, h_wnd: Handle) -> Option<Handle> {
        let mut objects = self.write_objects();
        match objects.user.get(h_wnd) {
            Some(UserObject::Window(user_window)) => {
                let window_identifier = WindowIdentifier {
                    process_id: self.process_id(),
                    window_handle: h_wnd,
                };
                let (bitmap_window_identifier, translation) = if user_window.parent_dc {
                    // TODO: nested CS_PARENTDC: how to handle them?
                    let translation = objects
                        .read_window_manager()
                        .position_of(window_identifier)
                        .unwrap_or_else(Point::origin);
                    let parent_window_identifier =
                        window_identifier.other_handle(user_window.parent_handle);
                    (parent_window_identifier, translation)
                } else {
                    (window_identifier, Point::origin())
                };
                let dc = DeviceContext {
                    bitmap_window_identifier,
                    translation,
                };
                objects.gdi.register(GdiObject::DC(dc))
            }
            None => None,
        }
    }

    #[api_function]
    fn internal_get_dc(&self, h_wnd: Handle) -> Result<ReturnValue, EmulatorError> {
        Ok(ReturnValue::U16(
            self.get_dc(h_wnd).unwrap_or(Handle::null()).as_u16(),
        ))
    }

    fn begin_paint(&self, h_wnd: Handle) -> Option<Paint> {
        self.get_dc(h_wnd).map(|hdc| Paint {
            hdc,
            f_erase: false,
            rect: Rect {
                left: 0,
                top: 0,
                right: 200,
                bottom: 200,
            },
        })
    }

    #[api_function]
    fn internal_begin_paint(
        &self,
        mut accessor: EmulatorAccessor,
        h_wnd: Handle,
        paint_ptr: Pointer,
    ) -> Result<ReturnValue, EmulatorError> {
        if let Some(paint) = self.begin_paint(h_wnd) {
            accessor
                .memory_mut()
                .write_16(paint_ptr.0, paint.hdc.as_u16())?;
            accessor
                .memory_mut()
                .write_8(paint_ptr.0.wrapping_add(2), paint.f_erase.into())?;
            accessor.write_rect(paint_ptr.0.wrapping_add(4), &paint.rect)?;
            Ok(ReturnValue::U16(paint.hdc.as_u16()))
        } else {
            Ok(ReturnValue::U16(0))
        }
    }

    fn release_dc(&self, _h_wnd: Handle, hdc: Handle) -> bool {
        // TODO: check whether the hdc belongs to the h_wnd ?
        // TODO: this should probably cause a flip of the front and back bitmap for the given window
        self.write_objects().gdi.deregister(hdc);
        true
    }

    #[api_function]
    fn internal_release_dc(
        &self,
        _h_wnd: Handle,
        hdc: Handle,
    ) -> Result<ReturnValue, EmulatorError> {
        Ok(ReturnValue::U16(self.release_dc(_h_wnd, hdc).into()))
    }

    fn end_paint(&self, _h_wnd: Handle, hdc: Handle) -> u16 {
        self.release_dc(_h_wnd, hdc).into()
    }

    #[api_function]
    fn internal_end_paint(
        &self,
        accessor: EmulatorAccessor,
        _h_wnd: Handle,
        paint: Pointer,
    ) -> Result<ReturnValue, EmulatorError> {
        // TODO: this should probably cause a flip of the front and back bitmap for the given window
        let handle = accessor.memory().read_16(paint.0)?;
        Ok(ReturnValue::U16(self.end_paint(_h_wnd, handle.into())))
    }

    fn with_paint_bitmap_for(
        &self,
        h_dc: Handle,
        objects: &RwLockReadGuard<ObjectEnvironment>,
        f: &dyn Fn(BitmapView),
    ) {
        if let Some(GdiObject::DC(device_context)) = objects.gdi.get(h_dc) {
            if let Some(bitmap) = objects
                .write_window_manager()
                .paint_bitmap_for_dc(device_context)
            {
                f(bitmap)
            }
        }
    }

    #[api_function]
    fn fill_rect(
        &self,
        accessor: EmulatorAccessor,
        h_dc: Handle,
        rect: Pointer,
        h_brush: Handle,
    ) -> Result<ReturnValue, EmulatorError> {
        let rect = accessor.read_rect(rect.0)?;
        let objects = self.read_objects();
        if let Some(GdiObject::SolidBrush(color)) = objects.gdi.get(h_brush) {
            self.with_paint_bitmap_for(h_dc, &objects, &|mut bitmap| {
                bitmap.fill_rectangle(rect, *color)
            });
        }
        Ok(ReturnValue::U16(1))
    }

    #[api_function]
    fn set_timer(
        &self,
        h_wnd: Handle,
        id_event: u16,
        elapse: u16,
        timer_proc_segment: u16,
        timer_proc_offset: u16,
    ) -> Result<ReturnValue, EmulatorError> {
        println!(
            "SET TIMER {:?}, {:x}, {}, {:x}, {:x}",
            h_wnd, id_event, elapse, timer_proc_segment, timer_proc_offset
        );
        // TODO: this fakes success
        Ok(ReturnValue::U16(1))
    }

    #[api_function]
    fn kill_timer(&self, _h_wnd: Handle, _u_id_event: u16) -> Result<ReturnValue, EmulatorError> {
        // TODO: this fakes success
        Ok(ReturnValue::U16(1))
    }

    #[api_function]
    fn get_window_rect(
        &self,
        mut accessor: EmulatorAccessor,
        h_wnd: Handle,
        rect_ptr: Pointer,
    ) -> Result<ReturnValue, EmulatorError> {
        println!("GET WINDOW RECT {:?}", h_wnd);
        if let Some(rect) =
            self.read_objects()
                .read_window_manager()
                .window_rect_of(WindowIdentifier {
                    process_id: self.process_id(),
                    window_handle: h_wnd,
                })
        {
            accessor.write_rect(rect_ptr.0, &rect)?;
            Ok(ReturnValue::U16(1))
        } else {
            Ok(ReturnValue::U16(0))
        }
    }

    #[api_function]
    fn set_window_text(
        &self,
        _h_wnd: Handle,
        text: HeapByteString,
    ) -> Result<ReturnValue, EmulatorError> {
        println!("SET WINDOW TEXT: {:?}", text);
        Ok(ReturnValue::U16(1))
    }

    #[api_function]
    fn message_box(
        &self,
        accessor: EmulatorAccessor,
        h_wnd: Handle,
        text: Pointer,
        caption: Pointer,
        _type: u16,
    ) -> Result<ReturnValue, EmulatorError> {
        println!(
            "MESSAGE BOX {:?}, {:x}, {:x}, {:x}",
            h_wnd, text.0, caption.0, _type
        );
        debug_print_null_terminated_string(&accessor, text.0);
        debug_print_null_terminated_string(&accessor, caption.0);
        Ok(ReturnValue::U16(0))
    }

    #[api_function]
    fn get_menu(&self, _h_wnd: Handle) -> Result<ReturnValue, EmulatorError> {
        // TODO
        Ok(ReturnValue::U16(Handle::null().as_u16()))
    }

    #[api_function]
    fn get_system_menu(&self, _h_wnd: Handle, _revert: u16) -> Result<ReturnValue, EmulatorError> {
        // TODO
        Ok(ReturnValue::U16(Handle::null().as_u16()))
    }

    #[api_function]
    fn check_menu_item(
        &self,
        _h_menu: Handle,
        _id_check_item: u16,
        _u_check: u16,
    ) -> Result<ReturnValue, EmulatorError> {
        // TODO
        Ok(ReturnValue::U16(0xFFFF))
    }

    #[api_function]
    fn enable_menu_item(
        &self,
        _h_menu: Handle,
        _id_enable_item: u16,
        _u_enable: u16,
    ) -> Result<ReturnValue, EmulatorError> {
        // TODO
        Ok(ReturnValue::U16(0xFFFF))
    }

    #[api_function]
    fn append_menu(
        &self,
        _h_menu: Handle,
        _flags: u16,
        _id_new_item: u16,
        _new_item_str: Pointer,
    ) -> Result<ReturnValue, EmulatorError> {
        // TODO
        Ok(ReturnValue::U16(1))
    }

    pub fn syscall(
        &mut self,
        nr: u16,
        emulator_accessor: EmulatorAccessor,
    ) -> Result<ReturnValue, EmulatorError> {
        match nr {
            1 => self.__api_message_box(emulator_accessor),
            5 => self.__api_init_app(emulator_accessor),
            10 => self.__api_set_timer(emulator_accessor),
            12 => self.__api_kill_timer(emulator_accessor),
            32 => self.__api_get_window_rect(emulator_accessor),
            37 => self.__api_set_window_text(emulator_accessor),
            39 => self.__api_internal_begin_paint(emulator_accessor),
            40 => self.__api_internal_end_paint(emulator_accessor),
            41 => self.__api_create_window(emulator_accessor),
            42 => self.__api_show_window(emulator_accessor),
            57 => self.__api_register_class(emulator_accessor),
            66 => self.__api_internal_get_dc(emulator_accessor),
            68 => self.__api_internal_release_dc(emulator_accessor),
            81 => self.__api_fill_rect(emulator_accessor),
            87 => self.__api_dialog_box(emulator_accessor),
            107 => self.__api_def_window_proc(emulator_accessor),
            108 => self.__api_get_message(emulator_accessor),
            113 => self.__api_translate_message(emulator_accessor),
            114 => self.__api_dispatch_message(emulator_accessor),
            124 => self.__api_update_window(emulator_accessor),
            154 => self.__api_check_menu_item(emulator_accessor),
            155 => self.__api_enable_menu_item(emulator_accessor),
            156 => self.__api_get_system_menu(emulator_accessor),
            157 => self.__api_get_menu(emulator_accessor),
            173 => self.__api_load_cursor(emulator_accessor),
            176 => self.__api_load_string(emulator_accessor),
            179 => self.__api_get_system_metrics(emulator_accessor),
            180 => self.__api_internal_get_sys_color(emulator_accessor),
            411 => self.__api_append_menu(emulator_accessor),
            420 => self.__api_wsprintf(emulator_accessor),
            0xffff => self.__api_button_window_proc(emulator_accessor),
            nr => {
                todo!("unimplemented user syscall {}", nr)
            }
        }
    }
}
