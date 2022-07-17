use crate::atom_table::AtomTable;
use crate::byte_string::ByteString;
use crate::constants::{WM_CREATE, WM_PAINT, WM_QUIT};
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
        window_classes.insert(
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
        );
        Self {
            user_atom_table: AtomTable::new(),
            window_classes,
            objects,
        }
    }

    fn init_app(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        debug!("[user] INIT APP {:x}", accessor.word_argument(0)?);
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, 1);
        Ok(())
    }

    fn create_window(&mut self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let param = accessor.pointer_argument(0)?;
        let h_instance = accessor.word_argument(2)?;
        let h_menu = accessor.word_argument(3)?;
        let h_wnd_parent = accessor.word_argument(4)?;
        let height = accessor.word_argument(5)?;
        let width = accessor.word_argument(6)?;
        let y = accessor.word_argument(7)?;
        let x = accessor.word_argument(8)?;
        let style = accessor.dword_argument(9)?;
        let window_name = accessor.pointer_argument(11)?;
        let class_name = accessor.pointer_argument(13)?;
        debug!(
            "[user] CREATE WINDOW {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x}",
            class_name,
            window_name,
            style,
            x,
            y,
            width,
            height,
            h_wnd_parent,
            h_menu,
            h_instance,
            param
        );
        debug_print_null_terminated_string(&accessor, class_name);

        // TODO: support atom lookup here (that's the case if segment == 0)
        if let Some(class) = self
            .window_classes
            .get(&accessor.static_string(class_name)?)
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
                accessor
                    .regs_mut()
                    .write_gpr_16(Registers::REG_AX, window_handle.as_u16());
                // TODO: l_param should get a pointer to a CREATESTRUCT that contains info about the window being created
                return self.call_wndproc_sync(&mut accessor, proc, window_handle, WM_CREATE, 0, 0);
            }
        }
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, 0);
        Ok(())
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

    fn show_window(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let cmd_show = accessor.word_argument(0)?;
        let h_wnd = accessor.word_argument(1)?;
        debug!("[user] SHOW WINDOW {:x} {:x}", h_wnd, cmd_show);
        let objects = self.write_objects();
        let success = match objects.user.get(h_wnd.into()) {
            Some(UserObject::Window(_)) => {
                // TODO: do something with cmd_show
                objects.window_manager().show_window(WindowIdentifier {
                    window_handle: h_wnd.into(),
                    process_id: self.process_id(),
                });
                true
            }
            None => false,
        };
        accessor
            .regs_mut()
            .write_gpr_16(Registers::REG_AX, success.into());
        Ok(())
    }

    fn update_window(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let h_wnd = accessor.word_argument(0)?;
        debug!("[user] UPDATE WINDOW {:x}", h_wnd);
        let success = match self.write_objects().user.get(h_wnd.into()) {
            Some(UserObject::Window(user_window)) => {
                // TODO: only do this if update region is non-empty
                self.call_wndproc_sync(
                    &mut accessor,
                    user_window.proc,
                    h_wnd.into(),
                    WM_PAINT,
                    0,
                    0,
                )?;
                true
            }
            None => false,
        };
        accessor
            .regs_mut()
            .write_gpr_16(Registers::REG_AX, success.into());
        Ok(())
    }

    fn register_class(&mut self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let wnd_class_ptr = accessor.pointer_argument(0)?;
        let wnd_class_style = accessor.memory().read_16(wnd_class_ptr)?;
        let wnd_class_proc_offset = accessor.memory().read_16(wnd_class_ptr + 2)?;
        let wnd_class_proc_segment = accessor.memory().read_16(wnd_class_ptr + 4)?;
        let wnd_class_cls_extra = accessor.memory().read_16(wnd_class_ptr + 6)?;
        let wnd_class_wnd_extra = accessor.memory().read_16(wnd_class_ptr + 8)?;
        let _wnd_class_h_instance = accessor.memory().read_16(wnd_class_ptr + 10)?;
        let wnd_class_h_icon = accessor.memory().read_16(wnd_class_ptr + 12)?;
        let wnd_class_h_cursor = accessor.memory().read_16(wnd_class_ptr + 14)?;
        let wnd_class_h_background = accessor.memory().read_16(wnd_class_ptr + 16)?;
        let wnd_class_menu_name = accessor.memory().flat_pointer_read(wnd_class_ptr + 18)?;
        let wnd_class_class_name = accessor.memory().flat_pointer_read(wnd_class_ptr + 22)?;

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
                accessor
                    .regs_mut()
                    .write_gpr_16(Registers::REG_AX, atom.as_u16());
                return Ok(());
            }

            self.user_atom_table.deregister(atom);
        }

        accessor.regs_mut().write_gpr_16(Registers::REG_AX, 0);

        Ok(())
    }

    fn dialog_box(&self, accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let dialog_func = accessor.pointer_argument(0)?;
        let hwnd_parent = accessor.word_argument(2)?;
        let template = accessor.pointer_argument(3)?;
        let h_instance = accessor.word_argument(5)?;
        println!(
            "DIALOG BOX {:x} {:x} {:x} {:x}",
            h_instance, template, hwnd_parent, dialog_func
        );
        // TODO
        Ok(())
    }

    fn get_message(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let msg_filter_max = accessor.word_argument(0)?;
        let msg_filter_min = accessor.word_argument(1)?;
        let h_wnd = accessor.word_argument(2)?;
        let msg = accessor.pointer_argument(3)?;
        debug!(
            "[user] GET MESSAGE {:x} {:x} {:x} {:x}",
            msg, h_wnd, msg_filter_min, msg_filter_max
        );

        let message = match self.read_objects().user.get(h_wnd.into()) {
            Some(UserObject::Window(user_window)) => user_window.message_queue.receive(),
            _ => None,
        };

        // TODO: implement filters
        // TODO: support hwnd being null
        let return_value = if let Some(message) = message {
            // TODO: write message

            if message.message == WM_QUIT {
                0
            } else {
                1
            }
        } else {
            println!("error");
            0xffff
        };
        accessor
            .regs_mut()
            .write_gpr_16(Registers::REG_AX, return_value);
        Ok(())
    }

    fn load_string(&self, accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let buffer_max = accessor.word_argument(0)?;
        let buffer = accessor.pointer_argument(1)?;
        let uid = accessor.word_argument(3)?;
        let h_instance = accessor.word_argument(4)?;
        println!(
            "LOAD STRING {:x} {:x} {:x} {:x}",
            h_instance, uid, buffer, buffer_max
        );
        // TODO
        Ok(())
    }

    fn load_cursor(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let cursor_name = accessor.pointer_argument(0)?;
        let h_instance = accessor.word_argument(2)?;
        debug!("[user] LOAD CURSOR {:x} {:x}", h_instance, cursor_name);

        // TODO: this now always returns NULL to indicate failure
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, 0);
        Ok(())
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

    fn get_system_metrics(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let metric = accessor.word_argument(0)?;
        debug!("[user] GET SYSTEM METRICS {:x}", metric);
        if metric == 0x16 {
            // 1 if debug version is installed, 0 otherwise
            accessor.regs_mut().write_gpr_16(Registers::REG_AX, 1);
        } else {
            // TODO: the others
            accessor.regs_mut().write_gpr_16(Registers::REG_AX, 0);
        }
        Ok(())
    }

    fn wsprintf(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let output_buffer_ptr = accessor.pointer_argument(0)?;
        let format_string_ptr = accessor.pointer_argument(2)?;
        print!("WSPRINTF FORMAT: ");
        debug_print_null_terminated_string(&accessor, format_string_ptr);
        // TODO: implement actual sprintf, now it just copies
        accessor.copy_string(format_string_ptr, output_buffer_ptr)?;
        print!("WSPRINTF OUTPUT: ");
        debug_print_null_terminated_string(&accessor, format_string_ptr);
        Ok(())
    }

    fn def_window_proc(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let l_param = accessor.dword_argument(0)?;
        let w_param = accessor.word_argument(2)?;
        let msg = accessor.word_argument(3)?;
        let h_wnd = accessor.word_argument(4)?;
        debug!(
            "[user] DEF WINDOW PROC {:x} {:x} {:x} {:x}",
            h_wnd, msg, w_param, l_param
        );
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, 0);
        Ok(())
    }

    fn begin_paint(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let paint = accessor.pointer_argument(0)?;
        let h_wnd = accessor.word_argument(2)?;
        debug!("[user] BEGIN PAINT {:x} {:x}", h_wnd, paint,);
        let mut objects = self.write_objects();
        let display_device_handle_for_window = match objects.user.get(h_wnd.into()) {
            Some(UserObject::Window(_)) => {
                let window_identifier = WindowIdentifier {
                    process_id: self.process_id(),
                    window_handle: h_wnd.into(),
                };
                if let Some(handle) = objects.gdi.register(GdiObject::DC(window_identifier)) {
                    accessor.memory_mut().write_16(paint, handle.as_u16())?;
                    accessor.memory_mut().write_8(paint.wrapping_add(2), 0)?; // TODO: fErase
                    accessor.memory_mut().write_16(paint.wrapping_add(2), 0)?;
                    accessor.memory_mut().write_16(paint.wrapping_add(2), 0)?;
                    accessor.memory_mut().write_16(paint.wrapping_add(2), 200)?; // TODO: rcPaint.right
                    accessor.memory_mut().write_16(paint.wrapping_add(2), 200)?; // TODO: rcPaint.bottom
                    handle.as_u16()
                } else {
                    0
                }
            }
            None => 0,
        };
        accessor
            .regs_mut()
            .write_gpr_16(Registers::REG_AX, display_device_handle_for_window);
        Ok(())
    }

    fn end_paint(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let paint = accessor.pointer_argument(0)?;
        let h_wnd = accessor.word_argument(2)?;
        debug!("[user] END PAINT {:x} {:x}", h_wnd, paint,);
        // TODO: this should probably cause a flip of the front and back bitmap for the given window
        let handle = accessor.memory().read_16(paint)?;
        self.write_objects().gdi.deregister(handle.into());
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, 1);
        Ok(())
    }

    fn fill_rect(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let h_brush = accessor.word_argument(0)?;
        let rect = accessor.pointer_argument(1)?;
        let h_dc = accessor.word_argument(3)?;
        debug!("[user] FILL RECT {:x} {:x} {:x}", h_dc, rect, h_brush);
        let rect = accessor.read_rect(rect)?;
        let objects = self.read_objects();
        if let (Some(GdiObject::DC(window_identifier)), Some(GdiObject::SolidBrush(color))) = (
            objects.gdi.get(h_dc.into()),
            objects.gdi.get(h_brush.into()),
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
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, 1);
        Ok(())
    }

    pub fn syscall(
        &mut self,
        nr: u16,
        emulator_accessor: EmulatorAccessor,
    ) -> Result<(), EmulatorError> {
        match nr {
            5 => self.init_app(emulator_accessor),
            39 => self.begin_paint(emulator_accessor),
            40 => self.end_paint(emulator_accessor),
            41 => self.create_window(emulator_accessor),
            42 => self.show_window(emulator_accessor),
            57 => self.register_class(emulator_accessor),
            81 => self.fill_rect(emulator_accessor),
            87 => self.dialog_box(emulator_accessor),
            107 => self.def_window_proc(emulator_accessor),
            108 => self.get_message(emulator_accessor),
            124 => self.update_window(emulator_accessor),
            173 => self.load_cursor(emulator_accessor),
            176 => self.load_string(emulator_accessor),
            179 => self.get_system_metrics(emulator_accessor),
            420 => self.wsprintf(emulator_accessor),
            nr => {
                todo!("unimplemented user syscall {}", nr)
            }
        }
    }
}
