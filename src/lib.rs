use ::image::imageops::FilterType;
use bincode::config::{self, Configuration};

pub mod feed;
pub mod screen_capture;
pub mod stream;
pub mod webcam;
pub mod window;

/// Constant that represents the filter used to resize images.
pub const FILTER: FilterType = FilterType::Nearest;

/// Configuration used to encode frames into bytes.
const ENCODE_CONFIG: Configuration = config::standard();
