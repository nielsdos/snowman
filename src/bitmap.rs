use crate::two_d::{Point, Rect};
use std::ops::{Deref, DerefMut};

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
    width: u16,
    height: u16,
}

impl<'a> BitmapView<'a> {
    pub fn new(bitmap: &'a mut Bitmap, translation: Point) -> Self {
        println!("Translated bitmap {:?}", translation);
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
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            pixels: vec![Color(255, 255, 255); (width as usize) * (height as usize)],
            translation: Point::origin(),
            width,
            height,
        }
    }

    fn clip_x(&self, x: u16) -> u16 {
        // TODO: edge cases?
        x.clamp(0, self.width.saturating_sub(self.translation.x))
    }

    fn clip_y(&self, y: u16) -> u16 {
        // TODO: edge cases?
        y.clamp(0, self.height.saturating_sub(self.translation.y))
    }

    pub fn clip(&self, rect: Rect) -> Rect {
        Rect {
            left: self.clip_x(rect.left),
            top: self.clip_y(rect.top),
            right: self.clip_x(rect.right),
            bottom: self.clip_y(rect.bottom),
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
        let x = x.wrapping_add(self.translation.x);
        let y = y.wrapping_add(self.translation.y);
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

    fn draw_vertical_line_unclipped(&mut self, x: u16, y_start: u16, y_to: u16, color: Color) {
        for y in y_start..y_to {
            let index = self.index_for(x, y);
            self.pixels[index] = color;
        }
    }

    pub fn draw_vertical_line(&mut self, x: u16, y_start: u16, y_to: u16, color: Color) {
        self.draw_vertical_line_unclipped(
            self.clip_x(x),
            self.clip_y(y_start),
            self.clip_y(y_to),
            color,
        )
    }

    pub fn draw_horizontal_line(&mut self, x_start: u16, y: u16, x_to: u16, color: Color) {
        self.draw_horizontal_line_unclipped(
            self.clip_x(x_start),
            self.clip_y(y),
            self.clip_x(x_to),
            color,
        )
    }

    pub fn fill_rectangle(&mut self, rect: Rect, color: Color) {
        let rect = self.clip(rect);
        // TODO: check validity of rect
        for y in rect.top..rect.bottom {
            self.draw_horizontal_line_unclipped(rect.left, y, rect.right, color);
        }
    }
}
