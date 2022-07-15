use crate::two_d::Rect;

#[derive(Debug, Copy, Clone)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub fn from(color: u32) -> Self {
        Self(color as u8, (color >> 8) as u8, (color >> 16) as u8)
    }
}

pub struct Bitmap {
    // TODO: should probably not use a vec?
    pixels: Vec<Color>,
    width: u16,
    height: u16,
}

impl Bitmap {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            pixels: vec![Color(255, 255, 255); (width as usize) * (height as usize)],
            width,
            height,
        }
    }

    pub fn clip(&self, rect: Rect) -> Rect {
        Rect {
            left: rect.left.clamp(0, self.width),
            top: rect.top.clamp(0, self.height),
            right: rect.right.clamp(0, self.width),
            bottom: rect.bottom.clamp(0, self.height),
        }
    }

    #[inline]
    pub fn width(&self) -> u16 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u16 {
        self.height
    }

    fn index_for(&self, x: u16, y: u16) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }

    #[inline]
    pub fn pixel_at_no_checks(&self, x: u16, y: u16) -> Color {
        self.pixels[self.index_for(x, y)]
    }

    fn draw_horizontal_line_unclipped(&mut self, x_start: u16, y: u16, x_to: u16, color: Color) {
        let start_index = self.index_for(x_start, y);
        let end_index = self.index_for(x_to, y);
        self.pixels[start_index..end_index].fill(color);
    }

    pub fn draw_horizontal_line(&mut self, x_start: u16, y: u16, x_to: u16, color: Color) {
        self.draw_horizontal_line_unclipped(x_start.clamp(0, self.width), y.clamp(0, self.height), x_to.clamp(0, self.width), color)
    }

    pub fn fill_rectangle(&mut self, rect: Rect, color: Color) {
        let rect = self.clip(rect);
        for y in rect.top..rect.bottom {
            self.draw_horizontal_line_unclipped(rect.left, y, rect.right, color);
        }
    }
}
