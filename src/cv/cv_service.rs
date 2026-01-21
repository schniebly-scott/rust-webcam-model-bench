use std::time::Duration;
use std::{thread, time::Instant, error::Error};
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;
use crate::camera::RgbaBuffer;
use crate::SharedFrame;
use super::{Inference, cv_inference::PoseEstimator, InfType};

pub struct CVManager {
    model_path: String,
    data_type: InfType,
    shared: SharedFrame,
    tx: broadcast::Sender<Inference>,
}

impl CVManager {
    pub fn new(model_path: &str, data_type: InfType, shared: SharedFrame) -> Self {
        let (tx, _) = broadcast::channel::<Inference>(2);

        Self {
            model_path: String::from(model_path),
            data_type,
            shared: shared,
            tx,
        }
    }

    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        let model_path = self.model_path.clone();
        let tx_clone = self.tx.clone();
        let shared_clone = self.shared.clone();
        let data_type_clone = self.data_type;

        let pool: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
        let pool_clone = pool.clone();

        thread::spawn(move || {
            // ---------- Load model inside thread ----------
            let mut pose = match PoseEstimator::new(&model_path) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Failed to load model: {e}");
                    return;
                }
            };

            loop {
                let frame_opt = {
                    let mut slot = shared_clone.lock().unwrap();
                    slot.take() // take() = replace with None
                };

                if let Some(frame) = frame_opt {
                    // ---------- Extract RGBA ----------
                    let (width, height, rgba) = (frame.0, frame.1, frame.2.data.clone());

                    // ---------- Inference ----------
                    let now = Instant::now();
                    let output = match pose.process_rgba(&rgba, width, height, data_type_clone) {
                        Ok(o) => o,
                        Err(e) => {
                            eprintln!("Inference error: {e}");
                            continue;
                        }
                    };
                    println!("Inference took {:?}", now.elapsed());

                    // ---------- Publish result ----------
                    let buf = RgbaBuffer {
                        data: output,
                        pool: pool_clone.clone(),
                    };

                    let _ = tx_clone.send((width, height, Arc::new(buf))); // Frame
                } else {
                    eprintln!("No frame available, yield CPU");
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
        });

        Ok(())
    }

    pub fn spawn(model_path: &str, data_type: InfType, shared: SharedFrame) -> Result<Self, Box<dyn Error>> {
        let cv = Self::new(model_path, data_type, shared);
        cv.start()?;
        Ok(cv)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Inference> {
        self.tx.subscribe()
    }
}