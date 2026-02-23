//! Module that implements the higher level window where the feed will be displayed or that will stream the feed.

use termcolor::{BufferWriter, ColorChoice};

use crate::feed::Feed;
use crate::feed::frame::AsciiEncoding;
use crate::stream::{join_connection_stream, open_connection_stream};
use std::error::Error;
use std::sync::{Arc, atomic::AtomicBool};

/// Struct that represents the window where the feed will be displayed or that will stream the feed.
pub struct Window {
    buffer_writer: BufferWriter,
}

impl Window {
    const COLOR_CHOICE: ColorChoice = ColorChoice::Auto;

    /// Function that creates a window from a coloured stdout.
    pub fn new(
        stdout: fn(ColorChoice) -> Result<BufferWriter, Box<dyn Error + Send + Sync>>,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self {
            buffer_writer: stdout(Self::COLOR_CHOICE)?,
        })
    }

    /// Function that displays the feed from any source in the colored stdout.
    pub async fn show_feed<T: Feed + Send>(
        self,
        encoding: AsciiEncoding,
        end_flag: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        T::show(self.buffer_writer, encoding, end_flag).await
    }

    /// Function that streams the feed captured from any feed source.
    pub async fn stream_feed<T: Feed + Send>(
        self,
        port: u32,
        end_flag: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut stream = open_connection_stream(port).await?;
        T::stream(&mut stream, end_flag).await
    }

    /// Function that shows the feed received from an UDP socket connection.
    pub async fn show_stream_feed<T: Feed + Send>(
        self,
        server_address: &str,
        encoding: AsciiEncoding,
        end_flag: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut stream = match join_connection_stream(server_address).await? {
            Some(stream) => stream,
            None => return Err("Couldn't accept connection".into()),
        };

        T::show_stream(self.buffer_writer, &mut stream, &encoding, end_flag).await
    }
}
