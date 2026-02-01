use image::{DynamicImage, ImageBuffer, Rgb};
use std::error::Error;
use xcap::Monitor;

use crate::FILTER;

pub struct Screen {
    pub monitor: Monitor,
}

impl Screen {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            monitor: Monitor::all()?[0].clone(),
        })
    }

    pub fn get_frame_rgb(&mut self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, Box<dyn Error>> {
        let frame = image::imageops::resize(&self.monitor.capture_image()?, 640, 320, FILTER);
        let rgb_image = DynamicImage::ImageRgba8(frame).into_rgb8();

        Ok(rgb_image)
    }
}
