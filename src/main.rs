use async_rate_limiter::RateLimiter;
use std::{
    error::Error,
    io::{Write, stdout},
    sync::{Arc, atomic::AtomicBool},
};
use tui_video_chat::image::{AsciiEncoding, Window};

const ENCODING: [char; 8] = [':', '-', '=', '+', '*', '%', '@', '#'];

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
    let print_rate_limiter = RateLimiter::new(200);

    window
        .show_screen_capture_feed_single_buffer(&encoding, end_flag, print_rate_limiter)
        .await?;

    print!("{}", termion::clear::All);
    stdout().flush()?;

    Ok(())
}
