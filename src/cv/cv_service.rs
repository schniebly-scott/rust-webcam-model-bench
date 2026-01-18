use std::thread;
use std::time::Duration;
use std::error::Error;

use tokio::sync::broadcast;
use crate::camera::Frame;

#[derive(Clone, Debug)]
enum InfType {
    Pose,
    BoundingBox,
    Segment,
}

#[derive(Clone, Debug)]
pub struct Inference {
    datapoints: Vec<u32>,
    inf_type: InfType,
} 

pub struct CVManager {
    rx: broadcast::Receiver<Frame>,
    model_path: String,
    tx: broadcast::Sender<Inference>,
}

impl CVManager {
    pub fn new(rx: broadcast::Receiver<Frame>, model_path: String) -> Self {
        let (tx, _) = broadcast::channel::<Inference>(1);

        Self {
            rx,
            model_path,
            tx,
        }
    }

    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }

    pub fn spawn(rx: broadcast::Receiver<Frame>, model_path: String) -> Result<Self, Box<dyn Error>> {
        let cv = Self::new(rx, model_path);
        cv.start()?;
        Ok(cv)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Inference> {
        self.tx.subscribe()
    }
}