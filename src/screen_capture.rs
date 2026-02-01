use image::{
    DynamicImage, ImageBuffer, Luma, Rgb,
    imageops::colorops::{brighten_in_place, contrast_in_place},
};
use std::error::Error;
use xcap::Monitor;

pub struct Screen {
    pub monitor: Monitor,
}

impl Screen {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            monitor: Monitor::all()?[0].clone(),
        })
    }

    pub fn get_frame_luma_rgb(
        &mut self,
    ) -> Result<
        (
            ImageBuffer<Luma<u8>, Vec<u8>>,
            ImageBuffer<Rgb<u8>, Vec<u8>>,
        ),
        Box<dyn Error>,
    > {
        let frame = self.monitor.capture_image()?;

        let mut luma_image = DynamicImage::ImageRgba8(frame.clone()).into_luma8();
        brighten_in_place(&mut luma_image, 20);
        contrast_in_place(&mut luma_image, 10.);

        let mut rgb_image = DynamicImage::ImageRgba8(frame).into_rgb8();
        brighten_in_place(&mut rgb_image, 40);

        Ok((luma_image, rgb_image))
    }
}
