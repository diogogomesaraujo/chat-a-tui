use bincode::config::{self, Configuration};
use image::{ImageBuffer, Rgb};
use nokhwa::{
    Camera, nokhwa_initialize,
    pixel_format::RgbFormat,
    query,
    utils::{ApiBackend, CameraFormat, RequestedFormat, RequestedFormatType, Resolution},
};
use std::{error::Error, time::Duration};

use crate::feed::Feed;

pub struct WebCam {
    pub camera: Camera,
}

impl Feed for WebCam {
    const FRAME_RATE: u32 = 600;
    const ENCODE_CONFIG: Configuration = config::standard();
    const TIMEOUT_DURATION: Duration = Duration::from_secs(1);

    fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        nokhwa_initialize(|granted| {
            println!("Access granted: {}.", granted);
        });

        let cameras = query(ApiBackend::Auto)?;

        let rgb_format =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(CameraFormat::new(
                Resolution::new(640, 480),
                nokhwa::utils::FrameFormat::MJPEG,
                30,
            )));

        let first_camera = match cameras.first() {
            Some(c) => c,
            _ => return Err("Couldn't connect to the camera.".into()),
        };

        let mut threaded = Camera::new(first_camera.index().clone(), rgb_format)?;
        threaded.open_stream()?;

        Ok(Self { camera: threaded })
    }

    fn get_frame_rgb(
        &mut self,
    ) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, Box<dyn Error + Send + Sync>> {
        let rgb_frame = self.camera.frame()?;
        let rgb_image = rgb_frame.decode_image::<RgbFormat>()?;

        Ok(rgb_image)
    }
}
