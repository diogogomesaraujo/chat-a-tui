use std::error::Error;
use termcolor::BufferWriter;
use tui_video_chat::image::{AsciiEncoding, Window};

const ENCODING: &[char] = &[':', '-', '=', '+', '*', '%', '@', '#'];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let encoding = AsciiEncoding(ENCODING.to_vec());

    let mut window = Window {
        buffer_writer: BufferWriter::stdout(termcolor::ColorChoice::Auto),
    };

    window.show_webcam_feed_single_buffer(&encoding)?;

    Ok(())
}
