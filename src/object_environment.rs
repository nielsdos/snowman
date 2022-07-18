use crate::bitmap::Color;
use crate::handle_table::{Handle, HandleTable};
use crate::memory::SegmentAndOffset;
use crate::message_queue::MessageQueue;
use crate::window_manager::WindowIdentifier;
use crate::WindowManager;
use std::sync::{Mutex, MutexGuard};

pub struct UserWindow {
    pub proc: SegmentAndOffset,
    pub parent_dc: bool,
    pub parent_handle: Handle,
    pub message_queue: MessageQueue,
    pub children: Vec<Handle>,
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

impl UserWindow {
    pub fn new(proc: SegmentAndOffset, parent_dc: bool, parent_handle: Handle) -> Self {
        Self {
            proc,
            message_queue: MessageQueue::new(),
            children: Vec::new(),
            parent_dc,
            parent_handle,
        }
    }
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
