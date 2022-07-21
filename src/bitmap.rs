use crate::two_d::{Point, Rect};
use std::ops::{Deref, DerefMut};
use crate::object_environment::Pen;

#[derive(Debug, Copy, Clone)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub fn from(color: u32) -> Self {
        Self(color as u8, (color >> 8) as u8, (color >> 16) as u8)
    }

    pub fn as_u32(&self) -> u32 {
        let r = self.0 as u32;
        let g = self.0 as u32;
        let b = self.0 as u32;
        r | (g << 8) | (b << 16)
    }
}

pub struct BitmapView<'a> {
    bitmap: &'a mut Bitmap,
    translation: Point,
}

pub struct Bitmap {
    // TODO: should probably not use a vec?
    pixels: Vec<Color>,
    translation: Point,
    moved_to: Point,
    width: i16,
    height: i16,
}

impl<'a> BitmapView<'a> {
    pub fn new(bitmap: &'a mut Bitmap, translation: Point) -> Self {
        bitmap.translation += translation;
        Self {
            bitmap,
            translation,
        }
    }
}

impl<'a> Deref for BitmapView<'a> {
    type Target = &'a mut Bitmap;

    fn deref(&self) -> &Self::Target {
        &self.bitmap
    }
}

impl<'a> DerefMut for BitmapView<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bitmap
    }
}

impl<'a> Drop for BitmapView<'a> {
    fn drop(&mut self) {
        self.bitmap.translation -= self.translation;
    }
}

impl Bitmap {
    pub fn new(width: i16, height: i16) -> Self {
        // TODO: remove this size hack
        let width = 500;
        let height = 500;
        Self {
            pixels: vec![Color(100, 0, 100); (width as usize) * (height as usize)],
            translation: Point::origin(),
            moved_to: Point::origin(),
            width,
            height,
        }
    }

    pub fn move_to(&mut self, point: Point) {
        self.moved_to = point;
    }

    fn clip_and_translate_x(&self, x: i16) -> i16 {
        let x = x.wrapping_add(self.translation.x);
        x.clamp(0, self.width - 1)
    }

    fn clip_and_translate_y(&self, y: i16) -> i16 {
        let y = y.wrapping_add(self.translation.y);
        y.clamp(0, self.height - 1)
    }

    pub fn clip_and_translate_rect(&self, rect: Rect) -> Rect {
        Rect {
            left: self.clip_and_translate_x(rect.left),
            top: self.clip_and_translate_y(rect.top),
            right: self.clip_and_translate_x(rect.right),
            bottom: self.clip_and_translate_y(rect.bottom),
        }
    }

    #[inline]
    pub fn width(&self) -> i16 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> i16 {
        self.height
    }

    fn index_for(&self, x: i16, y: i16) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }

    #[inline]
    pub fn pixel_at_no_checks(&self, x: i16, y: i16) -> Color {
        self.pixels[self.index_for(x, y)]
    }

    pub fn set_pixel(&mut self, x: i16, y: i16, color: Color) {
        let x = self.clip_and_translate_x(x);
        let y = self.clip_and_translate_y(y);
        let index = self.index_for(x, y);
        self.pixels[index] = color;
    }

    fn draw_horizontal_line_unclipped_untranslated(&mut self, x_start: i16, y: i16, x_to: i16, color: Color) {
        let start_index = self.index_for(x_start, y);
        let end_index = self.index_for(x_to, y);
        self.pixels[start_index..end_index].fill(color);
    }

    fn draw_vertical_line_unclipped_untranslated(&mut self, x: i16, y_start: i16, y_to: i16, color: Color) {
        for y in y_start..y_to {
            let index = self.index_for(x, y);
            self.pixels[index] = color;
        }
    }

    pub fn draw_vertical_line(&mut self, x: i16, y_start: i16, y_to: i16, color: Color) {
        self.draw_vertical_line_unclipped_untranslated(
            self.clip_and_translate_x(x),
            self.clip_and_translate_y(y_start),
            self.clip_and_translate_y(y_to),
            color,
        )
    }

    pub fn draw_horizontal_line(&mut self, x_start: i16, y: i16, x_to: i16, color: Color) {
        self.draw_horizontal_line_unclipped_untranslated(
            self.clip_and_translate_x(x_start),
            self.clip_and_translate_y(y),
            self.clip_and_translate_x(x_to),
            color,
        )
    }

    pub fn fill_rectangle(&mut self, rect: Rect, color: Color) {
        let rect = self.clip_and_translate_rect(rect);
        if rect.left < rect.right && rect.top < rect.bottom {
            for y in rect.top..rect.bottom {
                self.draw_horizontal_line_unclipped_untranslated(rect.left, y, rect.right, color);
            }
        }
    }

    fn draw_line(&mut self, from: Point, to: Point, pen: &Pen) {
        // TODO: different pen styles
        let dx = to.x.wrapping_sub(from.x).abs();
        let dy = -to.y.wrapping_sub(from.y).abs();
        let mut err = dx.wrapping_add(dy);
        let sx = if from.x < to.x { 1 } else { -1 };
        let sy = if from.y < to.y { 1 } else { -1 };
        let mut current_x = from.x;
        let mut current_y = from.y;

        loop {
            self.set_pixel(current_x, current_y, pen.color);
            if current_x == to.x && current_y == to.y {
                break;
            }
            let err2 = err.wrapping_mul(2);
            if err2 >= dy {
                err = err.wrapping_add(dy);
                current_x = current_x.wrapping_add(sx);
            }
            if err2 <= dx {
                err = err.wrapping_add(dx);
                current_y = current_y.wrapping_add(sy);
            }
        }
    }

    pub fn line_to(&mut self, to: Point, pen: &Pen) {
        self.draw_line(self.moved_to, to, pen)
    }
}
