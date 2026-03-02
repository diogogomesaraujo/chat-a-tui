//! Module that implements the higher level window where the feed will be displayed or that will stream the feed.

use ratatui::widgets::Widget;
use termcolor::{BufferWriter, ColorChoice};

use crate::feed::Feed;
use crate::feed::frame::AsciiEncoding;
use crate::stream::{join_connection_stream, open_connection_stream};
use std::error::Error;
use std::sync::{Arc, atomic::AtomicBool};

/// Struct that represents the window where the feed will be displayed or that will stream the feed.
pub struct Window<T: Feed + Send> {
    pub buffer_writer: BufferWriter,
    pub feed_source: T,
}

impl<T: Feed + Send> Window<T> {
    const COLOR_CHOICE: ColorChoice = ColorChoice::Auto;

    /// Function that creates a window from a coloured stdout.
    pub fn new(
        stdout: fn(ColorChoice) -> Result<BufferWriter, Box<dyn Error + Send + Sync>>,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self {
            buffer_writer: stdout(Self::COLOR_CHOICE)?,
            feed_source: T::new()?,
        })
    }

    /// Function that displays the feed from any source in the colored stdout.
    pub async fn show_feed(
        mut self,
        encoding: AsciiEncoding,
        end_flag: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.feed_source
            .display(self.buffer_writer, encoding, end_flag)
            .await
    }

    /// Function that streams the feed captured from any feed source.
    pub async fn stream_feed(
        mut self,
        port: u32,
        end_flag: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut stream = open_connection_stream(port).await?;
        self.feed_source.stream(&mut stream, end_flag).await
    }

    /// Function that shows the feed received from an UDP socket connection.
    pub async fn show_stream_feed(
        self,
        server_address: &str,
        encoding: AsciiEncoding,
        end_flag: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut stream = join_connection_stream(server_address).await?;
        T::display_stream(self.buffer_writer, &mut stream, &encoding, end_flag).await
    }
}

pub struct WindowWidget<T: Feed + Send> {
    pub window: Window<T>,
}

impl<T: Feed + Send> Widget for WindowWidget<T> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
    }
}
