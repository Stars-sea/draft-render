mod linalg;
mod buffer;
mod color;

use anyhow::Result;
use bytemuck::cast_slice;
use minifb::{Key, Window, WindowOptions};

use crate::buffer::RenderBuffer;
use crate::color::Color;

fn main() -> Result<()> {
    let (width, height) = (800, 600);

    let mut fb = RenderBuffer::new(width, height, Color::BLACK);
    let mut window = Window::new("renderer", width, height, WindowOptions::default())?;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        fb.clear(Color::BLACK);

        window.update_with_buffer(
            cast_slice(fb.as_slice()),
            fb.width(),
            fb.height(),
        )?;
    }

    Ok(())
}
