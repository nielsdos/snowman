use crate::bitmap::Bitmap;
use crate::handle_table::Handle;
use crate::screen::ScreenCanvas;
use std::collections::HashMap;

struct Window {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    front_bitmap: Option<Bitmap>,
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct ProcessId(u16);

impl ProcessId {
    pub const fn null() -> Self {
        Self(0)
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct WindowIdentifier {
    pub process_id: ProcessId,
    pub window_handle: Handle,
}

pub struct WindowManager {
    window_stack: Vec<WindowIdentifier>,
    windows: HashMap<WindowIdentifier, Window>,
}

impl WindowIdentifier {
    pub fn other_handle(&self, child_handle: Handle) -> Self {
        Self {
            process_id: self.process_id,
            window_handle: child_handle,
        }
    }
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            window_stack: Vec::new(),
            windows: HashMap::new(),
        }
    }

    pub fn create_window(
        &mut self,
        identifier: WindowIdentifier,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        use_parent_bitmap: bool,
    ) {
        // TODO: set sane limits for arguments?
        let number_or_default = |number: u16| {
            if number == 0x8000 {
                200
            } else {
                number
            }
        };

        // TODO: handle default values for x,y,w,h
        let width = number_or_default(width);
        let height = number_or_default(height);
        self.windows.insert(
            identifier,
            Window {
                x: number_or_default(x),
                y: number_or_default(y),
                width,
                height,
                front_bitmap: if use_parent_bitmap {
                    None
                } else {
                    Some(Bitmap::new(width, height))
                },
            },
        );
    }

    pub fn show_window(&mut self, identifier: WindowIdentifier) {
        if let Some(index) = self.window_stack.iter().position(|&w| w == identifier) {
            self.window_stack.remove(index);
        }
        self.window_stack.push(identifier);
    }

    pub fn paint(&mut self, screen: &mut ScreenCanvas) {
        // TODO: be more efficient than always redrawing everything
        for identifier in &self.window_stack {
            if let Some(window) = self.windows.get(identifier) {
                if let Some(bitmap) = &window.front_bitmap {
                    screen.blit_bitmap(window.x, window.y, bitmap);
                }
            }
        }
    }

    pub fn paint_bitmap_for(&mut self, identifier: WindowIdentifier) -> Option<&mut Bitmap> {
        self.windows.get_mut(&identifier).and_then(|window| window.front_bitmap.as_mut())
    }
}
