use ::image::imageops::FilterType;

pub mod feed;
pub mod screen_capture;
pub mod stream;
pub mod web_cam;
pub mod window;

pub const FILTER: FilterType = FilterType::Nearest;
