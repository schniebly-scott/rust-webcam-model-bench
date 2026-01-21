mod cv_inference;
mod cv_service;

use crate::camera::Frame;
pub use cv_service::CVManager;

pub type Inference = Frame;

#[derive(Clone, Debug, Copy)]
pub enum InfType {
    Pose,
    BoundingBox,
    Segment,
}