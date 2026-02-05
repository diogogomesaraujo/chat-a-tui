use crate::web_cam::WebCam;
use crate::{FILTER, screen_capture::Screen};
use async_rate_limiter::RateLimiter;
use image::imageops::colorops::{brighten_in_place, contrast_in_place};
use image::{DynamicImage, ImageBuffer, ImageReader, Luma, Rgb};
use std::error::Error;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use termcolor::{Buffer, BufferWriter, Color, ColorSpec, WriteColor};
use termion::terminal_size;
use tokio::sync::RwLock;

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
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            buffer_writer: BufferWriter::alternate_stdout(termcolor::ColorChoice::Auto)?,
        })
    }

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

        Ok(())
    }

    pub fn draw_frame_single_buffer(
        &mut self,
        rgb: ImageBuffer<Rgb<u8>, Vec<u8>>,
        encoding: &AsciiEncoding,
    ) -> Result<(), Box<dyn Error>> {
        let mut buffer = self.preprocess_frame_buffer(rgb, encoding)?;

        self.buffer_writer.print(&mut buffer)?;

        Ok(())
    }

    pub fn preprocess_frame_buffer(
        &mut self,
        rgb: ImageBuffer<Rgb<u8>, Vec<u8>>,
        encoding: &AsciiEncoding,
    ) -> Result<Buffer, Box<dyn Error>> {
        let (mut rgb, x, y) = match terminal_size() {
            Ok((x, y)) if x > 400 || y > 300 => return Ok(self.buffer_writer.buffer()),
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

    pub async fn show_webcam_feed_single_buffer(
        &mut self,
        encoding: &AsciiEncoding,
        end_flag: Arc<AtomicBool>,
        rate_limiter: RateLimiter,
    ) -> Result<(), Box<dyn Error>> {
        let mut cam = WebCam::new()?;

        while end_flag.load(std::sync::atomic::Ordering::Relaxed) == false {
            rate_limiter.acquire().await;

            let rgb = cam.get_frame_rgb()?;
            self.draw_frame_single_buffer(rgb, &encoding)?;
        }

        Ok(())
    }

    pub async fn show_screen_capture_feed_single_buffer(
        &mut self,
        encoding: &AsciiEncoding,
        end_flag: Arc<AtomicBool>,
        rate_limiter: RateLimiter,
    ) -> Result<(), Box<dyn Error>> {
        let mut screen = Screen::new()?;

        while end_flag.load(std::sync::atomic::Ordering::Relaxed) == false {
            rate_limiter.acquire().await;

            let rgb = screen.get_frame_rgb()?;
            self.draw_frame_single_buffer(rgb, &encoding)?;
        }

        Ok(())
    }

    pub async fn show_webcam_feed_triple_buffer(
        self,
        encoding: AsciiEncoding,
        end_flag: Arc<AtomicBool>,
        rate_limiter: RateLimiter,
    ) -> Result<(), Box<dyn Error>> {
        let mut cam = WebCam::new()?;

        let encoding = Arc::new(encoding);

        let triple_buffer = Arc::new(TripleBuffer::new(
            self.buffer_writer.buffer(),
            self.buffer_writer.buffer(),
            self.buffer_writer.buffer(),
        ));

        let window = Arc::new(RwLock::new(self));

        {
            let window = window.clone();
            let encoding = encoding.clone();

            let end_flag = end_flag.clone();
            let triple_buffer = triple_buffer.clone();

            tokio::spawn(async move {
                while end_flag.load(std::sync::atomic::Ordering::Relaxed) == false {
                    let rgb = cam.get_frame_rgb().unwrap();
                    *triple_buffer.back.write().await = window
                        .write()
                        .await
                        .preprocess_frame_buffer(rgb, &encoding)
                        .unwrap();

                    triple_buffer.swap_back_spare().await;
                    triple_buffer
                        .has_update
                        .swap(true, std::sync::atomic::Ordering::AcqRel);
                }
            });
        }

        while end_flag.load(std::sync::atomic::Ordering::Relaxed) == false {
            rate_limiter.acquire().await;

            if triple_buffer
                .has_update
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                triple_buffer.swap_front_spare().await;
                triple_buffer
                    .has_update
                    .swap(false, std::sync::atomic::Ordering::Relaxed);
            }

            window
                .write()
                .await
                .buffer_writer
                .print(&*triple_buffer.front.read().await)?;
        }

        Ok(())
    }
}

pub struct TripleBuffer<T> {
    front: RwLock<T>,
    spare: RwLock<T>,
    back: RwLock<T>,
    has_update: AtomicBool,
}

impl<T> TripleBuffer<T> {
    pub fn new(front: T, spare: T, back: T) -> Self {
        Self {
            front: RwLock::new(front),
            spare: RwLock::new(back),
            back: RwLock::new(spare),
            has_update: AtomicBool::new(false),
        }
    }

    pub async fn swap_front_spare(&self) {
        std::mem::swap(
            &mut *self.front.write().await,
            &mut *self.spare.write().await,
        );
    }

    pub async fn swap_back_spare(&self) {
        std::mem::swap(
            &mut *self.spare.write().await,
            &mut *self.back.write().await,
        );
    }
}
