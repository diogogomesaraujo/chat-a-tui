use std::{
    error::Error,
    io::{Write, stdout},
    process::exit,
};
use termion::screen::IntoAlternateScreen;
use tui_video_chat::image::{AsciiEncoding, Window};

const ENCODING: &[char] = &[':', '-', '=', '+', '*', '%', '@', '#'];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let encoding = AsciiEncoding(ENCODING.to_vec());

    ctrlc::set_handler(move || {
        stdout().into_alternate_screen().unwrap().flush().unwrap();
        exit(0);
    })?;

    let window = Window::new()?;
    window.show_webcam_feed_double_buffer(encoding).await?;

    Ok(())
}
