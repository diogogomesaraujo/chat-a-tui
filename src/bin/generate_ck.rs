use std::{error::Error, fs::File, io::Write};
use tui_video_chat::stream::generate_server_certificate;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (cert, key) = generate_server_certificate("localhost")?;
    let mut file = File::create("cert.pem")?;
    file.write_all(cert.as_bytes())?;
    let mut file = File::create("key.pem")?;
    file.write_all(key.as_bytes())?;
    Ok(())
}
