mod service;
mod subscription;

use crate::camera::Frame;
use tokio::sync::broadcast;

pub fn spawn(rx: broadcast::Receiver<Frame>) {
    unimplemented!()
}