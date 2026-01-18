mod cv_inference;
mod cv_service;
mod cv_sub;

use crate::camera::Frame;

pub use cv_service::{CVManager, InfType};
pub use cv_sub::inference_subscription;

pub type Inference = Frame;