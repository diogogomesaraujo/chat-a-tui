use image::{
    ImageBuffer, Luma, Rgb,
    imageops::{
        colorops::{brighten_in_place, contrast_in_place},
        contrast,
    },
};
use nokhwa::{
    Camera, nokhwa_initialize,
    pixel_format::{LumaFormat, RgbFormat},
    query,
    utils::{ApiBackend, CameraFormat, RequestedFormat, RequestedFormatType, Resolution},
};
use std::error::Error;

pub struct WebCam {
    pub luma: Camera,
    pub rgb: Camera,
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
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(CameraFormat::new(
                Resolution::new(640, 480),
                nokhwa::utils::FrameFormat::YUYV,
                30,
            )));
        let luma_format =
            RequestedFormat::new::<LumaFormat>(RequestedFormatType::Exact(CameraFormat::new(
                Resolution::new(640, 480),
                nokhwa::utils::FrameFormat::YUYV,
                30,
            )));

        let first_camera = match cameras.first() {
            Some(c) => c,
            _ => return Err("Couldn't connect to the camera.".into()),
        };

        let mut luma_threaded = Camera::new(first_camera.index().clone(), luma_format)?;
        let mut rgb_threaded = Camera::new(first_camera.index().clone(), rgb_format)?;

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
        let luma_frame = self.luma.frame()?;
        let mut luma_image = luma_frame.decode_image::<LumaFormat>()?;
        brighten_in_place(&mut luma_image, 70);
        contrast_in_place(&mut luma_image, 10.);

        let rgb_frame = self.rgb.frame()?;
        let mut rgb_image = rgb_frame.decode_image::<RgbFormat>()?;
        brighten_in_place(&mut rgb_image, 40);

        Ok((luma_image, rgb_image))
    }
}
