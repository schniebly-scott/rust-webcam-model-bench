mod cam_service;

pub use cam_service::{CameraManager, RgbaBuffer};

pub use cam_service::Frame;

pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 480;