use crate::atom_table::AtomTable;
use crate::emulator_accessor::EmulatorAccessor;
use crate::registers::Registers;
use crate::util::debug_print_null_terminated_string;
use crate::{debug, EmulatorError};

pub struct EmulatedUser {
    // TODO: do we need the table to be here, or globally available, and what protections need to exist in case of global availability?
    user_atom_table: AtomTable,
}

impl EmulatedUser {
    pub fn new() -> Self {
        Self {
            user_atom_table: AtomTable::new(),
        }
    }

    fn init_app(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        debug!("[user] INIT APP {:x}", accessor.word_argument(0)?);
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, 1);
        Ok(())
    }

    fn create_window(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
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
        println!(
            "CREATE WINDOW {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x}",
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
        debug_print_null_terminated_string(&accessor, window_name);

        // TODO: returns the window handle
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, 0x100);

        Ok(())
    }

    fn show_window(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let cmd_show = accessor.word_argument(0)?;
        let h_wnd = accessor.word_argument(1)?;
        debug!("[user] SHOW WINDOW {:x} {:x}", h_wnd, cmd_show);
        // TODO
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, 1);
        Ok(())
    }

    fn update_window(&self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let h_wnd = accessor.word_argument(0)?;
        debug!("[user] UPDATE WINDOW {:x}", h_wnd);
        // TODO
        accessor.regs_mut().write_gpr_16(Registers::REG_AX, 1);
        Ok(())
    }

    fn register_class(&mut self, mut accessor: EmulatorAccessor) -> Result<(), EmulatorError> {
        let wnd_class_ptr = accessor.pointer_argument(0)?;
        // TODO: register the other stuff too...
        let wnd_class_style = accessor.memory().read_16(wnd_class_ptr)?;
        let wnd_class_proc_segment = accessor.memory().read_16(wnd_class_ptr + 2)?;
        let wnd_class_proc_offset = accessor.memory().read_16(wnd_class_ptr + 4)?;
        let wnd_class_cls_extra = accessor.memory().read_16(wnd_class_ptr + 6)?;
        let wnd_class_wnd_extra = accessor.memory().read_16(wnd_class_ptr + 8)?;
        let wnd_class_h_instance = accessor.memory().read_16(wnd_class_ptr + 10)?;
        let wnd_class_h_icon = accessor.memory().read_16(wnd_class_ptr + 12)?;
        let wnd_class_h_cursor = accessor.memory().read_16(wnd_class_ptr + 14)?;
        let wnd_class_background = accessor.memory().read_16(wnd_class_ptr + 16)?;
        let wnd_class_menu_name = accessor.memory().flat_pointer_read(wnd_class_ptr + 18)?;
        let wnd_class_class_name = accessor.memory().flat_pointer_read(wnd_class_ptr + 22)?;

        let cloned_class_name = accessor.clone_string(wnd_class_class_name)?;
        if let Some(atom) = self.user_atom_table.register_atom(cloned_class_name) {
            accessor
                .regs_mut()
                .write_gpr_16(Registers::REG_AX, atom.as_u16());
        } else {
            accessor.regs_mut().write_gpr_16(Registers::REG_AX, 0);
        }

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
