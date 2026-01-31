use image::{ImageBuffer, Luma, Rgb};
use nokhwa::{
    CallbackCamera, nokhwa_initialize,
    pixel_format::{LumaFormat, RgbFormat},
    query,
    utils::{ApiBackend, RequestedFormat, RequestedFormatType},
};
use std::error::Error;

pub struct WebCam {
    pub luma: CallbackCamera,
    pub rgb: CallbackCamera,
}

impl WebCam {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // only needs to be run on OSX
        nokhwa_initialize(|granted| {
            println!("Access granted: {}.", granted);
        });
        let cameras = query(ApiBackend::Auto)?;
        cameras.iter().for_each(|cam| println!("{:?}", cam));

        let rgb_format =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
        let luma_format =
            RequestedFormat::new::<LumaFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

        let first_camera = match cameras.first() {
            Some(c) => c,
            _ => return Err("Couldn't connect to the camera.".into()),
        };

        let mut luma_threaded =
            CallbackCamera::new(first_camera.index().clone(), luma_format, |buffer| {
                let _luma_image = buffer.decode_image::<LumaFormat>().unwrap();
            })?;
        let mut rgb_threaded =
            CallbackCamera::new(first_camera.index().clone(), rgb_format, |buffer| {
                let _rgb_image = buffer.decode_image::<RgbFormat>().unwrap();
            })?;

        luma_threaded.open_stream()?;
        rgb_threaded.open_stream()?;

        Ok(Self {
            luma: luma_threaded,
            rgb: rgb_threaded,
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
        let luma_frame = self.luma.poll_frame()?;
        let luma_image = luma_frame.decode_image::<LumaFormat>()?;

        let rgb_frame = self.rgb.poll_frame()?;
        let rgb_image = rgb_frame.decode_image::<RgbFormat>()?;

        Ok((luma_image, rgb_image))
    }
}
