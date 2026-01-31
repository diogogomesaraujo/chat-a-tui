use std::error::Error;

use image::ImageReader;
use termcolor::BufferWriter;
use tui_video_chat::encoder::{AsciiEncoding, Frame, FrameSize};

fn main() -> Result<(), Box<dyn Error>> {
    let puppy = Frame::from_path("assets/bitiz.png", FrameSize::new(35, 50))?;

    let encoding = AsciiEncoding(vec![':', '-', '=', '+', '*', '%', '@', '#']);
    let mut buffer_writer = BufferWriter::stdout(termcolor::ColorChoice::Auto);
    puppy.draw(encoding, &mut buffer_writer)?;
    Ok(())
}
