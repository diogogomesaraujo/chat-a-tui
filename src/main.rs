use std::error::Error;
use termcolor::BufferWriter;
use tui_video_chat::image::{AsciiEncoding, Frame, FrameSize, Window};

const ENCODING: &[char] = &[':', '-', '=', '+', '*', '%', '@', '#'];
const _ENCODING_REVERSED: &[char] = &['#', '@', '%', '*', '+', '=', '-', ':'];

fn main() -> Result<(), Box<dyn Error>> {
    let puppy = Frame::from_path("assets/bitiz.png", FrameSize::new(35, 50))?;

    let encoding = AsciiEncoding(ENCODING.to_vec());

    let mut window = Window {
        buffer_writer: BufferWriter::stdout(termcolor::ColorChoice::Auto),
    };

    window.draw(puppy.clone(), &encoding)?;
    Ok(())
}
