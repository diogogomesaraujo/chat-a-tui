use std::{
    error::Error,
    io::{Write, stdout},
    sync::{Arc, atomic::AtomicBool},
};
use termcolor::BufferWriter;
use tui_video_chat::{stream::connect, webcam::WebCam, window::Window};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let end_flag = Arc::new(AtomicBool::new(false));
    end_flag.load(std::sync::atomic::Ordering::SeqCst);

    let end_flag_ctrlc = end_flag.clone();
    ctrlc::set_handler(move || {
        end_flag_ctrlc.store(true, std::sync::atomic::Ordering::SeqCst);
    })?;

    let window = Window::new(BufferWriter::alternate_stdout)?;

    let connection = connect(3001, "localhost:3000").await?;

    window.stream_feed::<WebCam>(connection, end_flag).await?;

    print!("{}", termion::clear::All);
    stdout().flush()?;

    Ok(())
}
