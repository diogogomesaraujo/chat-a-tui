use crate::FILTER;
use crate::feed::frame::{AsciiEncoding, Frame};
use async_rate_limiter::RateLimiter;
use async_trait::async_trait;
use bincode::config::Configuration;
use image::imageops::colorops::{brighten_in_place, contrast_in_place};
use image::{DynamicImage, ImageBuffer, Rgb};
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use termcolor::BufferWriter;
use termion::terminal_size;
use tokio::net::UdpSocket;

#[async_trait]
pub trait Feed: 'static {
    const FRAME_RATE: u32;
    const ENCODE_CONFIG: Configuration;

    fn new() -> Result<Self, Box<dyn Error + Send + Sync>>
    where
        Self: Sized;

    fn get_frame_rgb(
        &mut self,
    ) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, Box<dyn Error + Send + Sync>>;

    fn preprocess_frame(
        rgb: ImageBuffer<Rgb<u8>, Vec<u8>>,
    ) -> Result<Frame, Box<dyn Error + Send + Sync>> {
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

        Ok(frame)
    }

    async fn show(
        buffer_writer: BufferWriter,
        encoding: AsciiEncoding,
        end_flag: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut feed_source = Self::new()?;
        let rate_limiter = RateLimiter::new(Self::FRAME_RATE as usize);

        let (mut input_buffer, mut output_buffer) =
            triple_buffer::triple_buffer(&buffer_writer.buffer());

        while end_flag.load(std::sync::atomic::Ordering::Acquire) == false {
            rate_limiter.acquire().await;

            let rgb = feed_source.get_frame_rgb()?;
            let frame = Self::preprocess_frame(rgb)?;

            let mut buffer = buffer_writer.buffer();
            frame.load_buffer(&encoding, &mut buffer)?;
            input_buffer.write(buffer);

            buffer_writer.print(&output_buffer.read())?;
        }

        Ok(())
    }

    fn encode_frame(frame: Frame) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        Ok(bincode::encode_to_vec(frame, Self::ENCODE_CONFIG)?)
    }

    fn decode_frame(bytes: &[u8]) -> Result<Frame, Box<dyn Error + Send + Sync>> {
        let (decoded, _): (Frame, usize) =
            bincode::decode_from_slice(&bytes[..], Self::ENCODE_CONFIG)?;

        Ok(decoded)
    }

    async fn stream(
        connection: UdpSocket,
        end_flag: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut feed_source = Self::new()?;

        let (mut input_buffer, mut output_buffer) = triple_buffer::triple_buffer(&Vec::new());

        while end_flag.load(std::sync::atomic::Ordering::Acquire) == false {
            let rgb = feed_source.get_frame_rgb()?;
            let frame = Self::preprocess_frame(rgb)?;
            input_buffer.write(Self::encode_frame(frame)?);

            connection.send(&output_buffer.read()).await?;
        }

        Ok(())
    }

    async fn show_stream(
        buffer_writer: BufferWriter,
        connection: UdpSocket,
        encoding: &AsciiEncoding,
        end_flag: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        Self: Sized,
    {
        let rate_limiter = RateLimiter::new(Self::FRAME_RATE as usize);

        let (mut input_buffer, mut output_buffer) =
            triple_buffer::triple_buffer(&buffer_writer.buffer());

        let mut buffer_temp = vec![0u8; 65_507];

        while end_flag.load(std::sync::atomic::Ordering::Acquire) == false {
            rate_limiter.acquire().await;

            connection.recv(&mut buffer_temp).await?;
            let frame = Self::decode_frame(&buffer_temp)?;

            let mut buffer = buffer_writer.buffer();
            frame.load_buffer(&encoding, &mut buffer)?;
            input_buffer.write(buffer);

            buffer_writer.print(&output_buffer.read())?;
        }

        Ok(())
    }
}

pub mod frame {
    use bincode::{Decode, Encode};
    use image::{ImageBuffer, Luma, Rgb};
    use std::error::Error;
    use std::io::Write;
    use termcolor::{Buffer, Color, ColorSpec, WriteColor};

    #[derive(Clone, Encode, Decode)]
    pub struct Size {
        pub x: u16,
        pub y: u16,
    }

    impl Size {
        pub fn new(x: u16, y: u16) -> Self {
            Self { x, y }
        }
    }

    #[derive(Encode, Decode)]
    pub struct Pixel {
        red: u8,
        green: u8,
        blue: u8,
        grey_scale: u8,
    }

    #[derive(Encode, Decode)]
    pub struct Frame {
        pub pixels: Vec<Pixel>,
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
                pixels: luma
                    .pixels()
                    .zip(rgb.pixels())
                    .map(|(luma_pixel, rgb_pixel)| Pixel {
                        red: rgb_pixel[0],
                        green: rgb_pixel[1],
                        blue: rgb_pixel[2],
                        grey_scale: luma_pixel[0],
                    })
                    .collect::<Vec<Pixel>>(),
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

            self.pixels.iter().enumerate().try_for_each(
                |(i, pixel)| -> Result<(), Box<dyn Error + Send + Sync>> {
                    buffer.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(
                        pixel.red,
                        pixel.green,
                        pixel.blue,
                    ))))?;

                    if i % self.frame_size.x as usize == 0 {
                        write!(buffer, "\n")?;
                    }

                    let char_to_print = encoding.from_greyscale_value8(pixel.grey_scale);

                    write!(buffer, "{char_to_print}{char_to_print}")?;

                    Ok(())
                },
            )?;

            Ok(())
        }

        pub fn to_image_buffer(&self) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
            let mut image = ImageBuffer::new(self.frame_size.x as u32, self.frame_size.y as u32);
            self.pixels.iter().enumerate().for_each(|(i, p)| {
                let x = i as u32 % self.frame_size.x as u32;
                let y = i as u32 / self.frame_size.x as u32;
                image.put_pixel(
                    x,
                    y,
                    Rgb {
                        0: [p.red, p.green, p.blue],
                    },
                );
            });

            image
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
}
