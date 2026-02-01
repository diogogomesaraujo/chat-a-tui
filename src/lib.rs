use ::image::imageops::FilterType;

pub mod image;
pub mod screen_capture;
pub mod web_cam;

pub const FILTER: FilterType = FilterType::Nearest;
