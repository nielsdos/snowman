use crate::bitmap::{Bitmap, BitmapView};
use crate::handle_table::Handle;
use crate::object_environment::DeviceContext;
use crate::screen::ScreenCanvas;
use crate::two_d::{Point, Rect};
use std::collections::HashMap;

struct Window {
    position: Point,
    width: i16,
    height: i16,
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
        x: i16,
        y: i16,
        width: i16,
        height: i16,
        use_parent_bitmap: bool,
    ) -> (i16, i16) {
        // TODO: set sane limits for arguments?
        let number_or_default = |number: i16, default: i16| {
            if number == -32768 {
                default
            } else {
                number
            }
        };

        let width = number_or_default(width, 400);
        let height = number_or_default(height, 300);
        self.windows.insert(
            identifier,
            Window {
                position: Point {
                    x: number_or_default(x, 0),
                    y: number_or_default(y, 0),
                },
                width,
                height,
                front_bitmap: if use_parent_bitmap {
                    None
                } else {
                    Some(Bitmap::new(width, height))
                },
            },
        );

        (width, height)
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
                    screen.blit_bitmap(window.position, bitmap);
                }
            }
        }
    }

    pub fn paint_bitmap_for(&mut self, identifier: WindowIdentifier) -> Option<&mut Bitmap> {
        self.windows
            .get_mut(&identifier)
            .and_then(|window| window.front_bitmap.as_mut())
    }

    pub fn paint_bitmap_for_dc(&mut self, dc: &DeviceContext) -> Option<BitmapView> {
        self.paint_bitmap_for(dc.bitmap_window_identifier)
            .map(|bitmap| BitmapView::new(bitmap, dc.translation))
    }

    pub fn position_of(&self, identifier: WindowIdentifier) -> Option<Point> {
        self.windows.get(&identifier).map(|window| window.position)
    }

    pub fn client_rect_of(&self, identifier: WindowIdentifier) -> Option<Rect> {
        self.windows
            .get(&identifier)
            .map(|window| Rect {
                top: 0,
                left: 0,
                right: window.width,
                bottom: window.height,
            })
    }

    pub fn window_rect_of(&self, identifier: WindowIdentifier) -> Option<Rect> {
        // TODO: what about nested windows?
        self.windows.get(&identifier).map(|window| Rect {
            top: window.position.y,
            left: window.position.x,
            right: window.width,
            bottom: window.height,
        })
    }
}
