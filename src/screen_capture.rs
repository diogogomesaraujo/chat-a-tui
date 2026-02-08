use crate::{FILTER, feed::Feed};
use bincode::config::{self, Configuration};
use image::{DynamicImage, ImageBuffer, Rgb};
use std::error::Error;
use xcap::Monitor;

pub struct Screen {
    pub monitor: Monitor,
}

impl Feed for Screen {
    const FRAME_RATE: u32 = 200;
    const ENCODE_CONFIG: Configuration = config::standard();

    fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self {
            monitor: Monitor::all()?[0].clone(),
        })
    }

    fn get_frame_rgb(
        &mut self,
    ) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, Box<dyn Error + Send + Sync>> {
        let frame = image::imageops::resize(&self.monitor.capture_image()?, 640, 320, FILTER);
        let rgb_image = DynamicImage::ImageRgba8(frame).into_rgb8();

        Ok(rgb_image)
    }
}
