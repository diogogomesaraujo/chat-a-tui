use async_rate_limiter::RateLimiter;
use std::{
    error::Error,
    io::{Write, stdout},
    sync::{Arc, atomic::AtomicBool},
};
use tui_video_chat::image::{AsciiEncoding, Window};

const ENCODING: &[char] = &[':', '-', '=', '+', '*', '%', '@', '#'];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let end_flag = Arc::new(AtomicBool::new(false));
    end_flag.load(std::sync::atomic::Ordering::SeqCst);

    let end_flag_ctrlc = end_flag.clone();
    ctrlc::set_handler(move || {
        end_flag_ctrlc.store(true, std::sync::atomic::Ordering::SeqCst);
    })?;

    let window = Window::new()?;
    let encoding = AsciiEncoding(ENCODING.to_vec());
    let rate_limiter = RateLimiter::new(30);

    window
        .show_webcam_feed_triple_buffer(encoding, end_flag, rate_limiter)
        .await?;

    print!("{}", termion::clear::All);
    stdout().flush()?;

    Ok(())
}
