use crate::bitmap::Color;
use crate::handle_table::{Handle, HandleTable};
use crate::memory::SegmentAndOffset;
use crate::two_d::Point;
use crate::window_manager::WindowIdentifier;
use crate::WindowManager;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct UserWindow {
    pub proc: SegmentAndOffset,
    pub parent_dc: bool,
    pub parent_handle: Handle,
    pub children: Vec<Handle>,
}

pub enum UserObject {
    Window(UserWindow),
}

pub struct DeviceContext {
    pub bitmap_window_identifier: WindowIdentifier,
    pub translation: Point,
}

pub struct Pen {
    // TODO: style
    pub width: u16,
    pub color: Color,
}

pub enum GdiObject {
    DC(DeviceContext),
    SolidBrush(Color),
    Pen(Pen),
}

pub struct ObjectEnvironment<'a> {
    pub user: HandleTable<UserObject>,
    pub gdi: HandleTable<GdiObject>,
    pub window_manager: &'a RwLock<WindowManager>,
}

impl UserWindow {
    pub fn new(proc: SegmentAndOffset, parent_dc: bool, parent_handle: Handle) -> Self {
        Self {
            proc,
            children: Vec::new(),
            parent_dc,
            parent_handle,
        }
    }
}

impl<'a> ObjectEnvironment<'a> {
    pub fn new(window_manager: &'a RwLock<WindowManager>) -> Self {
        Self {
            user: HandleTable::new(),
            gdi: HandleTable::new(),
            window_manager,
        }
    }

    pub fn read_window_manager(&self) -> RwLockReadGuard<WindowManager> {
        self.window_manager.read().unwrap()
    }

    pub fn write_window_manager(&self) -> RwLockWriteGuard<WindowManager> {
        self.window_manager.write().unwrap()
    }
}
