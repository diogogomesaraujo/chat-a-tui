use ::image::imageops::FilterType;
use bincode::config::{self, Configuration};

pub mod feed;
pub mod screen_capture;
pub mod stream;
pub mod webcam;
pub mod window;

pub const FILTER: FilterType = FilterType::Nearest;
const ENCODE_CONFIG: Configuration = config::standard();
