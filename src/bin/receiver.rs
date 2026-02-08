use std::{
    error::Error,
    io::{Write, stdout},
    sync::{Arc, atomic::AtomicBool},
};
use termcolor::BufferWriter;
use tui_video_chat::{
    feed::frame::AsciiEncoding, stream::connect, web_cam::WebCam, window::Window,
};

const ENCODING: [char; 8] = [':', '-', '=', '+', '*', '%', '@', '#'];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let end_flag = Arc::new(AtomicBool::new(false));
    end_flag.load(std::sync::atomic::Ordering::SeqCst);

    let end_flag_ctrlc = end_flag.clone();
    ctrlc::set_handler(move || {
        end_flag_ctrlc.store(true, std::sync::atomic::Ordering::SeqCst);
    })?;

    let window = Window::new(BufferWriter::alternate_stdout)?;
    let encoding = AsciiEncoding(ENCODING.to_vec());

    let connection = connect(3000, "localhost:3001").await?;

    window
        .show_stream_feed::<WebCam>(connection, encoding, end_flag)
        .await?;

    print!("{}", termion::clear::All);
    stdout().flush()?;

    Ok(())
}
