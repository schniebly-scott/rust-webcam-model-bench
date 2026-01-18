mod cam_service;
mod cam_sub;

pub use cam_service::{CameraManager, RgbaBuffer};
pub use cam_sub::raw_frame_subscription;

pub use cam_service::Frame;

pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 480;