use crate::bitmap::Bitmap;
use crate::window_manager::WindowManager;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::WindowCanvas;
use sdl2::Sdl;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

pub struct ScreenCanvas {
    canvas: WindowCanvas,
}

pub struct Screen {
    sdl_context: Sdl,
    canvas: ScreenCanvas,
    window_manager: Arc<RwLock<WindowManager>>,
}

impl Screen {
    pub fn new(window_manager: Arc<RwLock<WindowManager>>) -> Result<Self, String> {
        // Setup window
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window("Emulator screen", 800, 600)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.clear();
        canvas.present();

        Ok(Self {
            sdl_context,
            canvas: ScreenCanvas { canvas },
            window_manager,
        })
    }

    pub fn window_loop(&mut self) {
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        // TODO: be more efficient than always redrawing everything
        'running: loop {
            self.canvas.canvas.set_draw_color(Color::RGB(0, 0, 0));
            self.canvas.canvas.clear();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }
            {
                self.window_manager.write().unwrap().paint(&mut self.canvas);
            }
            self.canvas.canvas.present();
            thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        }
    }
}

impl ScreenCanvas {
    pub fn blit_bitmap(&mut self, position: crate::two_d::Point, bitmap: &Bitmap) {
        let top_left = Point::new(position.x as i32, position.y as i32);

        for y in 0..bitmap.height() {
            for x in 0..bitmap.width() {
                let pixel = bitmap.pixel_at_no_checks(x, y);
                self.canvas
                    .set_draw_color(Color::RGB(pixel.0, pixel.1, pixel.2));
                self.canvas
                    .draw_point(top_left.offset(x as i32, y as i32))
                    .unwrap();
            }
        }
    }
}
