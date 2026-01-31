use image::{DynamicImage, ImageReader};
use std::error::Error;
use std::io::Write;
use termcolor::{BufferWriter, Color, ColorSpec, WriteColor};

pub struct FrameSize {
    pub x: u16,
    pub y: u16,
}

impl FrameSize {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

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
                image::imageops::FilterType::Gaussian,
            ),
            frame_size,
        })
    }

    pub fn draw(
        &self,
        encoding: AsciiEncoding,
        buffer_writer: &mut BufferWriter,
    ) -> Result<(), Box<dyn Error>> {
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

                    write!(buffer, "{} ", char_to_print)?;
                    buffer_writer.print(&buffer)?;

                    Ok(())
                },
            )
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
