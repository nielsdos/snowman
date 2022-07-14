use std::sync::{Mutex, MutexGuard};
use crate::bitmap::Color;
use crate::handle_table::HandleTable;
use crate::window_manager::WindowIdentifier;
use crate::WindowManager;

#[derive(Copy, Clone)]
pub struct UserWindow {
    pub proc_segment: u16,
    pub proc_offset: u16,
}

pub enum UserObject {
    Window(UserWindow),
}

pub enum GdiObject {
    DC(WindowIdentifier),
    SolidBrush(Color),
}

pub struct ObjectEnvironment<'a> {
    pub user: HandleTable<UserObject>,
    pub gdi: HandleTable<GdiObject>,
    pub window_manager: &'a Mutex<WindowManager>,
}

impl<'a> ObjectEnvironment<'a> {
    pub fn new(window_manager: &'a Mutex<WindowManager>) -> Self {
        Self {
            user: HandleTable::new(),
            gdi: HandleTable::new(),
            window_manager,
        }
    }

    pub fn window_manager(&self) -> MutexGuard<'_, WindowManager> {
        self.window_manager.lock().unwrap()
    }
}
