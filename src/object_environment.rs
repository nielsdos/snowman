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
    pub selected_brush: Handle,
    pub selected_pen: Handle,
}

pub struct Pen {
    // TODO: style
    pub width: u16,
    pub color: Color,
}

pub enum GdiSelectionObjectType {
    SolidBrush,
    Pen,
    Invalid,
}

pub enum GdiObject {
    DC(DeviceContext),
    SolidBrush(Color),
    Pen(Pen),
    // TODO: remove me once we have all types
    Placeholder
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
        let mut gdi = HandleTable::new();

        // Stock objects
        gdi.register(GdiObject::SolidBrush(Color(255, 255, 255)));
        gdi.register(GdiObject::Placeholder);
        gdi.register(GdiObject::Placeholder);
        gdi.register(GdiObject::Placeholder);
        gdi.register(GdiObject::SolidBrush(Color(0, 0, 0)));
        gdi.register(GdiObject::Placeholder);
        gdi.register(GdiObject::Pen(Pen {
            width: 1,
            color: Color(255, 255, 255),
        }));
        gdi.register(GdiObject::Pen(Pen {
            width: 1,
            color: Color(0, 0, 0),
        }));
        gdi.register(GdiObject::Placeholder);
        gdi.register(GdiObject::Placeholder);
        gdi.register(GdiObject::Placeholder);
        gdi.register(GdiObject::Placeholder);
        gdi.register(GdiObject::Placeholder);
        gdi.register(GdiObject::Placeholder);
        gdi.register(GdiObject::Placeholder);
        gdi.register(GdiObject::Placeholder);
        gdi.register(GdiObject::Placeholder);

        Self {
            user: HandleTable::new(),
            gdi,
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

impl DeviceContext {
    pub fn select(&mut self, selection_type: GdiSelectionObjectType, handle: Handle) -> Handle {
        match selection_type {
            GdiSelectionObjectType::SolidBrush => {
                let old = self.selected_brush;
                self.selected_brush = handle;
                old
            }
            GdiSelectionObjectType::Pen => {
                let old = self.selected_pen;
                self.selected_pen = handle;
                old
            },
            GdiSelectionObjectType::Invalid => Handle::null(),
        }
    }
}
