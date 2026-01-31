use std::error::Error;
use termcolor::BufferWriter;
use tui_video_chat::{
    image::{AsciiEncoding, Window},
    web_cam::WebCam,
};

const ENCODING: &[char] = &[':', '-', '=', '+', '*', '%', '@', '#'];
const _ENCODING_REVERSED: &[char] = &['#', '@', '%', '*', '+', '=', '-', ':'];

fn main() -> Result<(), Box<dyn Error>> {
    let encoding = AsciiEncoding(ENCODING.to_vec());
    let mut window = Window {
        buffer_writer: BufferWriter::stdout(termcolor::ColorChoice::Auto),
    };

    let mut cam = WebCam::new()?;
    loop {
        let (luma, rgb) = cam.get_frame_luma_rgb()?;
        window.draw_frame(luma, rgb, &encoding)?;
    }
}
