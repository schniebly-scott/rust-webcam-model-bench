mod cam_service;
mod cam_sub;

pub use cam_service::{CameraManager, RgbaBuffer};
pub use cam_sub::raw_frame_subscription;

use std::sync::Arc;

/// RGBA frame sent to the UI
/// (width, height, RgbaBugger { frame, pool-pointer })
pub type Frame = (u32, u32, Arc<RgbaBuffer>);

pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 480;