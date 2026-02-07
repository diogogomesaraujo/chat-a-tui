use crate::FILTER;
use crate::feed::frame::{AsciiEncoding, Frame};
use async_rate_limiter::RateLimiter;
use async_trait::async_trait;
use image::imageops::colorops::{brighten_in_place, contrast_in_place};
use image::{DynamicImage, ImageBuffer, Rgb};
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use termcolor::{Buffer, BufferWriter};
use termion::terminal_size;

#[async_trait]
pub trait Feed {
    const FRAME_RATE: u32;

    fn new() -> Result<Self, Box<dyn Error + Send + Sync>>
    where
        Self: Sized;

    fn get_frame_rgb(
        &mut self,
    ) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, Box<dyn Error + Send + Sync>>;

    fn preprocess_frame_buffer(
        buffer_writer: &BufferWriter,
        rgb: ImageBuffer<Rgb<u8>, Vec<u8>>,
        encoding: &AsciiEncoding,
    ) -> Result<Buffer, Box<dyn Error + Send + Sync>> {
        let (mut rgb, x, y) = match terminal_size() {
            Ok((x, y)) if x > 400 || y > 300 => return Ok(buffer_writer.buffer()),
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

        let mut buffer = buffer_writer.buffer();
        frame.load_buffer(encoding, &mut buffer)?;

        Ok(buffer)
    }

    async fn show(
        buffer_writer_consumer: BufferWriter,
        buffer_writer_producer: BufferWriter,
        encoding: AsciiEncoding,
        end_flag: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut feed_source = Self::new()?;
        let rate_limiter = RateLimiter::new(Self::FRAME_RATE as usize);

        while end_flag.load(std::sync::atomic::Ordering::Acquire) == false {
            rate_limiter.acquire().await;

            let rgb = feed_source.get_frame_rgb()?;
            let buffer = Self::preprocess_frame_buffer(&buffer_writer_consumer, rgb, &encoding)?;
            buffer_writer_producer.print(&buffer)?;
        }
        Ok(())
    }
}

pub mod frame {
    use image::{ImageBuffer, Luma, Rgb};
    use std::error::Error;
    use std::io::Write;
    use termcolor::{Buffer, Color, ColorSpec, WriteColor};

    #[derive(Clone)]
    pub struct Size {
        pub x: u16,
        pub y: u16,
    }

    impl Size {
        pub fn new(x: u16, y: u16) -> Self {
            Self { x, y }
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

    pub struct Frame {
        pub luma: ImageBuffer<Luma<u8>, Vec<u8>>,
        pub rgb: ImageBuffer<Rgb<u8>, Vec<u8>>,
        pub frame_size: Size,
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
                frame_size: Size {
                    x: size_x,
                    y: size_y,
                },
            }
        }

        pub fn load_buffer(
            &self,
            encoding: &AsciiEncoding,
            buffer: &mut Buffer,
        ) -> Result<(), Box<dyn Error + Send + Sync>> {
            write!(buffer, "{}", termion::clear::AfterCursor)?;

            self.luma
                .enumerate_pixels()
                .zip(self.rgb.enumerate_pixels())
                .try_for_each(
                    |((luma_x, _, luma_pixel), (_, _, rgb_pixel))| -> Result<(), Box<dyn Error + Send + Sync>> {
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
}
