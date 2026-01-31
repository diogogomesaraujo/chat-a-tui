use crate::FILTER;
use image::{DynamicImage, ImageReader};
use std::io::Write;
use std::{error::Error, io::stdout};
use termcolor::{BufferWriter, Color, ColorSpec, WriteColor};
use termion::terminal_size;

#[derive(Clone)]
pub struct FrameSize {
    pub x: u16,
    pub y: u16,
}

impl FrameSize {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

#[derive(Clone)]
pub struct Frame {
    pub image: DynamicImage,
    pub frame_size: FrameSize,
}

impl Frame {
    pub fn from_path(path: &str, frame_size: FrameSize) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            image: ImageReader::open(path)?.decode()?.resize(
                frame_size.x as u32,
                frame_size.y as u32,
                FILTER,
            ),
            frame_size,
        })
    }

    pub fn new(image: DynamicImage, size_x: u16, size_y: u16) -> Self {
        Self {
            image,
            frame_size: FrameSize {
                x: size_x,
                y: size_y,
            },
        }
    }

    pub fn clear_and_draw(
        &self,
        encoding: &AsciiEncoding,
        buffer_writer: &mut BufferWriter,
    ) -> Result<(), Box<dyn Error>> {
        let mut stdout = stdout();
        write!(stdout, "{}", termion::clear::All)?;
        stdout.flush()?;

        self.image
            .to_luma8()
            .enumerate_pixels()
            .zip(self.image.to_rgb8().enumerate_pixels())
            .try_for_each(
                |((luma_x, _, luma_pixel), (_, _, rgb_pixel))| -> Result<(), Box<dyn Error>> {
                    let mut buffer = buffer_writer.buffer();
                    buffer.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(
                        rgb_pixel[0],
                        rgb_pixel[1],
                        rgb_pixel[2],
                    ))))?;

                    if luma_x == 0 {
                        write!(buffer, "\n")?;
                    }

                    let char_to_print = encoding.from_greyscale_value8(luma_pixel[0]);

                    write!(buffer, "{char_to_print}{char_to_print}")?;
                    buffer_writer.print(&buffer)?;

                    Ok(())
                },
            )?;

        stdout.flush()?;

        Ok(())
    }
}

pub struct AsciiEncoding(pub Vec<char>);

impl AsciiEncoding {
    pub fn from_greyscale_value8(&self, value: u8) -> char {
        let encodings_len = self.0.len() as u8 - 1;
        let range = u8::MAX;

        self.0[value as usize * encodings_len as usize / range as usize]
    }
}

pub struct Window {
    pub buffer_writer: BufferWriter,
}

impl Window {
    pub fn draw(&mut self, frame: Frame, encoding: &AsciiEncoding) -> Result<(), Box<dyn Error>> {
        let frame = match terminal_size() {
            Ok((x, y)) => Frame::new(frame.image.resize(x as u32, y as u32, FILTER), x, y),
            Err(_) => frame,
        };
        frame.clear_and_draw(encoding, &mut self.buffer_writer)?;

        Ok(())
    }
}
