use crate::bitmap::Bitmap;
use crate::Screen;

struct Window {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    front_bitmap: Bitmap,
}

pub struct WindowManager {
    windows: Vec<Window>,
}

impl WindowManager {
    pub fn new() -> Self {
        let mut wm = Self {
            windows: Vec::new(),
        };
        // TODO: this is a test window
        wm.create_window(10, 10, 200, 200);
        wm
    }

    pub fn create_window(&mut self, x: u16, y: u16, width: u16, height: u16) {
        // TODO: handle default values
        self.windows.push(Window {
            x, y, width, height,
            front_bitmap: Bitmap::new(width, height),
        });
    }

    pub fn paint(&self, screen: &mut Screen) {
        // TODO: be more efficient than always redrawing everything
        for window in &self.windows {
            screen.blit_bitmap(window.x, window.y, &window.front_bitmap);
        }
    }
}
