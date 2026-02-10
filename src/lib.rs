use ::image::imageops::FilterType;

pub mod feed;
pub mod screen_capture;
pub mod stream;
pub mod webcam;
pub mod window;

pub const FILTER: FilterType = FilterType::Nearest;
