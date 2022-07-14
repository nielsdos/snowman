#[derive(Copy, Clone)]
pub struct Pixel(pub u8, pub u8, pub u8);

pub struct Bitmap {
    // TODO: should probably not use a vec?
    pixels: Vec<Pixel>,
    width: u16,
    height: u16,
}

impl Bitmap {
    pub fn new(width: u16, height: u16) -> Self {
        let mut pixels = vec![Pixel(255, 0, 0); (width as usize) * (height as usize)];
        // TODO: this is a test
        for y in 0..height {
            for x in 0..width {
                pixels[(y as usize) * (width as usize) + (x as usize)] = Pixel(y as u8, 0, x as u8);
            }
        }
        Self {
            pixels,
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

    #[inline]
    pub fn pixel_at_no_checks(&self, x: u16, y: u16) -> Pixel {
        // TODO: the casting is ugly
        let index = (y as usize) * (self.width as usize) + (x as usize);
        self.pixels[index]
    }
}
