use termcolor::{BufferWriter, ColorChoice};
use tokio::net::UdpSocket;

use crate::feed::Feed;
use crate::feed::frame::AsciiEncoding;
use std::error::Error;
use std::sync::{Arc, atomic::AtomicBool};

pub struct Window {
    buffer_writer: BufferWriter,
}

impl Window {
    const COLOR_CHOICE: ColorChoice = ColorChoice::Auto;

    pub fn new(
        stdout: fn(ColorChoice) -> Result<BufferWriter, Box<dyn Error + Send + Sync>>,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self {
            buffer_writer: stdout(Self::COLOR_CHOICE)?,
        })
    }

    pub async fn show_feed<T: Feed + Send>(
        self,
        encoding: AsciiEncoding,
        end_flag: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        T::show(self.buffer_writer, encoding, end_flag).await
    }

    pub async fn stream_feed<T: Feed + Send>(
        self,
        connection: UdpSocket,
        end_flag: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        T::stream(connection, end_flag).await
    }

    pub async fn show_stream_feed<T: Feed + Send>(
        self,
        connection: UdpSocket,
        encoding: AsciiEncoding,
        end_flag: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        T::show_stream(self.buffer_writer, connection, &encoding, end_flag).await
    }
}
