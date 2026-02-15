use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{thread, time::Instant, error::Error};
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;
use crate::camera::RgbaBuffer;
use crate::SharedFrame;
use super::{Inference, cv_inference::Model, InfType};

#[derive(Debug)]
pub struct CVManager {
    model_path: String,
    data_type: InfType,
    model: Arc<Mutex<Option<Model>>>,
    shared: SharedFrame,
    tx: broadcast::Sender<Inference>,
    running: Arc<AtomicBool>
}

impl CVManager {
    pub fn new(model_path: &str, data_type: InfType, shared: SharedFrame) -> Self {
        let (tx, _) = broadcast::channel::<Inference>(2);

        Self {
            model_path: String::from(model_path),
            data_type,
            model: Arc::new(Mutex::new(None)),
            shared: shared,
            tx,
            running: Arc::new(AtomicBool::new(false))
        }
    }

    pub fn load_model(&self) -> Result<Duration, Box<dyn Error>> {
        let now = Instant::now();
        let estimator = Model::new(&self.model_path)?;
        let elapsed = now.elapsed();

        let mut model_lock = self.model.lock().unwrap();
        *model_lock = Some(estimator);

        println!("Loading model took {:?}", elapsed);
        Ok(elapsed)
    }


    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        let tx_clone = self.tx.clone();
        let shared_clone = self.shared.clone();
        let data_type_clone = self.data_type;
        let model_clone = self.model.clone();
        
        let running_clone = self.running.clone();
        running_clone.store(true, Ordering::SeqCst);

        let pool: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
        let pool_clone = pool.clone();

        thread::spawn(move || {
            // ---------- Get reference to Model inside thread ----------
            let mut model_lock = model_clone.lock().unwrap();

            let model = match model_lock.as_mut() {
                Some(p) => p,
                None => {
                    eprintln!("Model not loaded!");
                    return;
                }
            };

            while running_clone.load(Ordering::SeqCst) {
                let frame_opt = {
                    let mut slot = shared_clone.lock().unwrap();
                    slot.take() // take() = replace with None
                };

                if let Some(frame) = frame_opt {
                    // ---------- Extract RGBA ----------
                    let (width, height, rgba) = (frame.0, frame.1, frame.2.data.clone());

                    // ---------- Inference ----------
                    let now = Instant::now();
                    let output = match model.process_rgba(&rgba, width, height, data_type_clone) {
                        Ok(o) => o,
                        Err(e) => {
                            eprintln!("Inference error: {e}");
                            continue;
                        }
                    };
                    let elapsed = now.elapsed();

                    // ---------- Publish result ----------
                    let buf = RgbaBuffer {
                        data: output,
                        pool: pool_clone.clone(),
                    };

                    let _ = tx_clone.send(Inference { frame: (width, height, Arc::new(buf)), inf_time: elapsed });
                } else {
                    //No frame available, yield CPU
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
        });

        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
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