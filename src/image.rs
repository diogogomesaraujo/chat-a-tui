use crate::FILTER;
use image::{DynamicImage, ImageBuffer, ImageReader, Luma, Rgb};
use std::io::Write;
use std::{error::Error, io::stdout};
use termcolor::{BufferWriter, Color, ColorSpec, WriteColor};
use termion::terminal_size;

#[derive(Clone)]
pub struct ImageSize {
    pub x: u16,
    pub y: u16,
}

impl ImageSize {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

#[derive(Clone)]
pub struct Image {
    pub image: DynamicImage,
    pub size: ImageSize,
}

impl Image {
    pub fn from_path(path: &str, size: ImageSize) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            image: ImageReader::open(path)?
                .decode()?
                .resize(size.x as u32, size.y as u32, FILTER),
            size,
        })
    }

    pub fn new(image: DynamicImage, size_x: u16, size_y: u16) -> Self {
        Self {
            image,
            size: ImageSize {
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
        write!(stdout, "{}", termion::clear::AfterCursor)?;
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

pub struct Frame {
    pub luma: ImageBuffer<Luma<u8>, Vec<u8>>,
    pub rgb: ImageBuffer<Rgb<u8>, Vec<u8>>,
    pub frame_size: ImageSize,
}

impl Frame {
    pub fn new(
        luma: ImageBuffer<Luma<u8>, Vec<u8>>,
        rgb: ImageBuffer<Rgb<u8>, Vec<u8>>,
        size_x: u16,
        size_y: u16,
    ) -> Self {
        Self {
            luma,
            rgb,
            frame_size: ImageSize {
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
        write!(stdout, "{}", termion::clear::AfterCursor)?;
        stdout.flush()?;

        self.luma
            .enumerate_pixels()
            .zip(self.rgb.enumerate_pixels())
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
    pub fn draw_image(
        &mut self,
        image: Image,
        encoding: &AsciiEncoding,
    ) -> Result<(), Box<dyn Error>> {
        let frame = match terminal_size() {
            Ok((x, y)) => Image::new(image.image.resize(x as u32, y as u32, FILTER), x, y),
            Err(_) => image,
        };
        frame.clear_and_draw(encoding, &mut self.buffer_writer)?;

        Ok(())
    }
    pub fn draw_frame(
        &mut self,
        luma: ImageBuffer<Luma<u8>, Vec<u8>>,
        rgb: ImageBuffer<Rgb<u8>, Vec<u8>>,
        encoding: &AsciiEncoding,
    ) -> Result<(), Box<dyn Error>> {
        let frame = match terminal_size() {
            Ok((x, y)) => Frame::new(
                image::imageops::resize(&luma, x as u32 / 2, y as u32, FILTER),
                image::imageops::resize(&rgb, x as u32 / 2, y as u32, FILTER),
                x / 2,
                y,
            ),
            Err(_) => Frame::new(
                image::imageops::resize(&luma, 50, 50, FILTER),
                image::imageops::resize(&rgb, 50, 50, FILTER),
                50,
                50,
            ),
        };
        frame.clear_and_draw(encoding, &mut self.buffer_writer)?;

        Ok(())
    }
}
