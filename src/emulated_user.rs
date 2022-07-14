use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use crate::atom_table::AtomTable;
use crate::emulator_accessor::EmulatorAccessor;
use crate::handle_table::{GenericHandle, Handle, HandleTable};
use crate::registers::Registers;
use crate::util::debug_print_null_terminated_string;
use crate::{debug, EmulatorError};
use crate::byte_string::HeapByteString;

#[allow(dead_code)]
#[derive(Debug)]
struct WindowClass {
    style: u16,
    proc_segment: u16,
    proc_offset: u16,
    cls_extra: u16,
    wnd_extra: u16,
    h_icon: Handle,
    h_cursor: Handle,
    h_background: Handle,
    menu_class_name: Option<HeapByteString>,
}

enum UserObject {
    Window(),
}

// TODO: figure out which parts here need to be shared and in case of sharing, what needs to be protected
pub struct EmulatedUser {
    user_atom_table: AtomTable,
    window_classes: HashMap<HeapByteString, WindowClass>,
    objects: HandleTable<UserObject>,
}

impl EmulatedUser {
    pub fn new() -> Self {
        Self {
            user_atom_table: AtomTable::new(),
            window_classes: HashMap::new(),
            objects: HandleTable::new(),
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

        // TODO: support atom lookup here (that's the case if segment == 0)
        // TODO: avoid allocation in the future for just looking up strings
        if let Some(class) = self.window_classes.get(&accessor.clone_string(class_name)?) {
            let handle = self.objects.register(UserObject::Window() /* TODO */).unwrap_or(Handle::null());
            // TODO: send WM_CREATE
            accessor.regs_mut().write_gpr_16(Registers::REG_AX, handle.as_u16());
        } else {
            accessor.regs_mut().write_gpr_16(Registers::REG_AX, 0);
        }
        Ok(())
    }

    fn show_window(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let cmd_show = accessor.word_argument(0)?;
        let h_wnd = accessor.word_argument(1)?;
        debug!("[user] SHOW WINDOW {:x} {:x}", h_wnd, cmd_show);
        let success = match self.objects.get(h_wnd.into()) {
            Some(UserObject::Window()) => {
                // TODO: do something with cmd_show
                // TODO
                true
            }
            None => false,
        };
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, success.into());
        Ok(())
    }

    fn update_window(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let h_wnd = accessor.word_argument(0)?;
        debug!("[user] UPDATE WINDOW {:x}", h_wnd);
        let success = match self.objects.get(h_wnd.into()) {
            Some(UserObject::Window()) => {
                // TODO: update window
                true
            }
            None => false,
        };
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, success.into());
        Ok(())
    }

    fn register_class(&mut self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let wnd_class_ptr = accessor.pointer_argument(0)?;
        let wnd_class_style = accessor.memory().read_16(wnd_class_ptr)?;
        let wnd_class_proc_segment = accessor.memory().read_16(wnd_class_ptr + 2)?;
        let wnd_class_proc_offset = accessor.memory().read_16(wnd_class_ptr + 4)?;
        let wnd_class_cls_extra = accessor.memory().read_16(wnd_class_ptr + 6)?;
        let wnd_class_wnd_extra = accessor.memory().read_16(wnd_class_ptr + 8)?;
        let wnd_class_h_instance = accessor.memory().read_16(wnd_class_ptr + 10)?;
        let wnd_class_h_icon = accessor.memory().read_16(wnd_class_ptr + 12)?;
        let wnd_class_h_cursor = accessor.memory().read_16(wnd_class_ptr + 14)?;
        let wnd_class_h_background = accessor.memory().read_16(wnd_class_ptr + 16)?;
        let wnd_class_menu_name = accessor.memory().flat_pointer_read(wnd_class_ptr + 18)?;
        let wnd_class_class_name = accessor.memory().flat_pointer_read(wnd_class_ptr + 22)?;

        let cloned_class_name = accessor.clone_string(wnd_class_class_name)?;
        if let Some(atom) = self.user_atom_table.register(cloned_class_name.clone()) {
            let window_class = WindowClass {
                style: wnd_class_style,
                proc_segment: wnd_class_proc_segment,
                proc_offset: wnd_class_proc_offset,
                cls_extra: wnd_class_cls_extra,
                wnd_extra: wnd_class_wnd_extra,
                h_icon: wnd_class_h_icon.into(),
                h_cursor: wnd_class_h_cursor.into(),
                h_background: wnd_class_h_background.into(),
                menu_class_name: if wnd_class_menu_name != 0 {
                    Some(accessor.clone_string(wnd_class_menu_name)?)
                } else { None },
            };

            debug!("[user] REGISTER CLASS SUCCESS {:?} => {:#?}", cloned_class_name, window_class);
            if self.window_classes.insert(cloned_class_name, window_class).is_none() {
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
        // TODO: this is to prevent the application from exiting
        loop {
            thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }

        // TODO
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, 0);
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

    fn get_system_metrics(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let metric = accessor.word_argument(0)?;
        debug!("[user] GET SYSTEM METRICS {:x}", metric);
        // 0x16 = 1 if debug version is installed, 0 otherwise
        if metric == 0x16 {
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

    pub fn syscall(
        &mut self,
        nr: u16,
        emulator_accessor: EmulatorAccessor,
    ) -> Result<(), EmulatorError> {
        match nr {
            5 => self.init_app(emulator_accessor),
            41 => self.create_window(emulator_accessor),
            42 => self.show_window(emulator_accessor),
            57 => self.register_class(emulator_accessor),
            87 => self.dialog_box(emulator_accessor),
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
