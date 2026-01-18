use std::{time::Instant, error::Error};
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;
use crate::camera::{Frame, RgbaBuffer};
use super::{Inference, cv_inference::PoseEstimator};

#[derive(Clone, Debug)]
pub enum InfType {
    Pose,
    BoundingBox,
    Segment,
}

pub struct CVManager {
    rx: broadcast::Receiver<Frame>,
    model_path: String,
    data_type: InfType,
    tx: broadcast::Sender<Inference>,
}

impl CVManager {
    pub fn new(rx: broadcast::Receiver<Frame>, model_path: &str, data_type: InfType) -> Self {
        let (tx, _) = broadcast::channel::<Inference>(1);

        Self {
            rx,
            model_path: String::from(model_path),
            data_type,
            tx,
        }
    }

    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        let mut rx = self.rx.resubscribe();
        let model_path = self.model_path.clone();
        let tx = self.tx.clone();
        let pool: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));

        tokio::spawn(async move {
            // Load model once
            let pose_arc = Arc::new(Mutex::new(PoseEstimator::new(&model_path)
                .expect("Failed to load model")));

            while let Ok(frame) = rx.recv().await {
                let pose_clone = pose_arc.clone();
                let tx = tx.clone();
                let pool_clone = pool.clone();
                let rgba = frame.2.clone(); // clone Arc, cheap
                let width = frame.0;
                let height = frame.1;

                tokio::task::spawn_blocking(move || {
                    let mut pose = pose_clone.lock().unwrap();
                    let now = Instant::now();

                    // Inference
                    let output = match pose.process_rgba(&rgba.data, width, height) {
                        Ok(o) => o,
                        Err(e) => {
                            eprintln!("Inference error: {e}");
                            return;
                        }
                    };
                    println!("Inference took {:?}", now.elapsed());

                    // Publish
                    let buf = RgbaBuffer {
                        data: output,
                        pool: pool_clone,
                    };
                    let _ = tx.send((width, height, Arc::new(buf)));
                });
            }

            eprintln!("Receiver closed, stopping cv_service");
        });

        Ok(())
    }

    pub fn spawn(rx: broadcast::Receiver<Frame>, model_path: &str, data_type: InfType) -> Result<Self, Box<dyn Error>> {
        let cv = Self::new(rx, model_path, data_type);
        cv.start()?;
        Ok(cv)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Inference> {
        self.tx.subscribe()
    }
}