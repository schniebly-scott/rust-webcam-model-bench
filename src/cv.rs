mod cv_inference;
mod cv_service;

use std::time::Duration;

use crate::camera::Frame;
pub use cv_service::CVManager;

#[derive(Clone, Debug)]
pub struct Inference { 
    pub frame: Frame,
    pub inf_time: Duration
}

#[derive(Clone, Debug, Copy)]
pub enum InfType {
    Pose,
    BoundingBox,
    Segment,
}