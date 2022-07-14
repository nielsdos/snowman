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

    pub fn draw_horizontal_line(&mut self, x_start: u16, y: u16, x_to: u16, color: Color) {
        // TODO: bounds checks to saturate the bounds?
        let start_index = self.index_for(x_start, y);
        let end_index = self.index_for(x_to, y);
        self.pixels[start_index..end_index].fill(color);
    }

    pub fn fill_rectangle(&mut self, x_from: u16, y_from: u16, x_to: u16, y_to: u16, color: Color) {
        // TODO: bounds checks to saturate the bounds?
        for y in y_from..y_to {
            self.draw_horizontal_line(x_from, y, x_to, color);
        }
    }
}
