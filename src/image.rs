use crate::web_cam::WebCam;
use crate::{FILTER, screen_capture::Screen};
use image::imageops::colorops::{brighten_in_place, contrast_in_place};
use image::{DynamicImage, ImageBuffer, ImageReader, Luma, Rgb};
use std::error::Error;
use std::io::Write;
use termcolor::{Buffer, BufferWriter, Color, ColorSpec, WriteColor};
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

    pub fn load_buffer(
        &self,
        encoding: &AsciiEncoding,
        buffer: &mut Buffer,
    ) -> Result<(), Box<dyn Error>> {
        write!(buffer, "{}", termion::clear::AfterCursor)?;

        self.image
            .to_luma8()
            .enumerate_pixels()
            .zip(self.image.to_rgb8().enumerate_pixels())
            .try_for_each(
                |((luma_x, _, luma_pixel), (_, _, rgb_pixel))| -> Result<(), Box<dyn Error>> {
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

                    Ok(())
                },
            )?;

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

    pub fn load_buffer(
        &self,
        encoding: &AsciiEncoding,
        buffer: &mut Buffer,
    ) -> Result<(), Box<dyn Error>> {
        write!(buffer, "{}", termion::clear::AfterCursor)?;

        self.luma
            .enumerate_pixels()
            .zip(self.rgb.enumerate_pixels())
            .try_for_each(
                |((luma_x, _, luma_pixel), (_, _, rgb_pixel))| -> Result<(), Box<dyn Error>> {
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

                    Ok(())
                },
            )?;

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
        let mut buffer = self.buffer_writer.buffer();

        frame.load_buffer(encoding, &mut buffer)?;

        self.buffer_writer.print(&mut buffer)?;
        buffer.clear();

        Ok(())
    }

    pub fn draw_frame_single_buffer(
        &mut self,
        rgb: ImageBuffer<Rgb<u8>, Vec<u8>>,
        encoding: &AsciiEncoding,
    ) -> Result<(), Box<dyn Error>> {
        let mut buffer = self.preprocess_frame_buffer(rgb, encoding)?;

        self.buffer_writer.print(&mut buffer)?;
        buffer.clear();

        Ok(())
    }

    pub fn preprocess_frame_buffer(
        &mut self,
        rgb: ImageBuffer<Rgb<u8>, Vec<u8>>,
        encoding: &AsciiEncoding,
    ) -> Result<Buffer, Box<dyn Error>> {
        let (mut rgb, x, y) = match terminal_size() {
            Ok((x, y)) => (
                image::imageops::resize(&rgb, x as u32 / 2, y as u32, FILTER),
                x,
                y,
            ),
            Err(_) => (
                rgb.clone(),
                rgb.width().clone() as u16,
                rgb.height().clone() as u16,
            ),
        };
        brighten_in_place(&mut rgb, 40);

        let mut luma = DynamicImage::ImageRgb8(rgb.clone()).into_luma8();
        brighten_in_place(&mut luma, 20);
        contrast_in_place(&mut luma, 10.);

        let frame = Frame::new(luma, rgb, x, y);

        let mut buffer = self.buffer_writer.buffer();
        frame.load_buffer(encoding, &mut buffer)?;

        Ok(buffer)
    }

    pub fn show_webcam_feed_single_buffer(
        &mut self,
        encoding: &AsciiEncoding,
    ) -> Result<(), Box<dyn Error>> {
        let mut cam = WebCam::new()?;
        loop {
            let rgb = cam.get_frame_rgb()?;
            self.draw_frame_single_buffer(rgb, &encoding)?;
        }
    }

    pub fn show_screen_capture_feed_single_buffer(
        &mut self,
        encoding: &AsciiEncoding,
    ) -> Result<(), Box<dyn Error>> {
        let mut screen = Screen::new()?;
        loop {
            let rgb = screen.get_frame_rgb()?;
            self.draw_frame_single_buffer(rgb, &encoding)?;
        }
    }
}
