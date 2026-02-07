use termcolor::{BufferWriter, ColorChoice};

use crate::feed::Feed;
use crate::feed::frame::AsciiEncoding;
use std::error::Error;
use std::sync::{Arc, atomic::AtomicBool};

pub struct Window {
    buffer_writer_consumer: BufferWriter,
    buffer_writer_producer: BufferWriter,
}

impl Window {
    const COLOR_CHOICE: ColorChoice = ColorChoice::Auto;

    pub fn new(
        stdout: fn(ColorChoice) -> Result<BufferWriter, Box<dyn Error + Send + Sync>>,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self {
            buffer_writer_consumer: stdout(Self::COLOR_CHOICE)?,
            buffer_writer_producer: stdout(Self::COLOR_CHOICE)?,
        })
    }

    pub async fn show_feed<T: Feed + Send>(
        self,
        encoding: AsciiEncoding,
        end_flag: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        T::show(
            self.buffer_writer_consumer,
            self.buffer_writer_producer,
            encoding,
            end_flag,
        )
        .await
    }
}
