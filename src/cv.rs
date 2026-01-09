mod cv_service;
mod cv_sub;

use std::error::Error;
use crate::camera::Frame;
use tokio::sync::broadcast;

// Will likely implement with the same pattern as camera
pub fn spawn(rx: broadcast::Receiver<Frame>) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}