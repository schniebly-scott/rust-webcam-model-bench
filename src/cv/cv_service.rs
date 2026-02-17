use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{time::Instant, error::Error};
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;
use crate::SharedFrame;
use crate::config::ModelConfig;
use crate::cv::cv_worker::CVWorker;
use super::{Inference, cv_inference::Model};

#[derive(Debug)]
pub struct CVManager {
    config: ModelConfig,
    model: Arc<Mutex<Option<Model>>>,
    shared: SharedFrame,
    tx: broadcast::Sender<Inference>,
    running: Arc<AtomicBool>
}

impl CVManager {
    pub fn new(config: ModelConfig, shared: SharedFrame) -> Self {
        let (tx, _) = broadcast::channel::<Inference>(2);

        Self {
            config,
            model: Arc::new(Mutex::new(None)),
            shared: shared,
            tx,
            running: Arc::new(AtomicBool::new(false))
        }
    }

    pub fn load_model(&self) -> Result<Duration, Box<dyn Error>> {
        let now = Instant::now();
        let estimator = Model::new(&self.config.model_path)?;
        let elapsed = now.elapsed();

        let mut model_lock = self.model.lock().unwrap();
        *model_lock = Some(estimator);

        println!("Loading model took {:?}", elapsed);
        Ok(elapsed)
    }


    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        self.running.store(true, Ordering::SeqCst);

        CVWorker {
            config: self.config.clone(),
            model: self.model.clone(),
            shared: self.shared.clone(),
            tx: self.tx.clone(),
            running: self.running.clone()
        }.spawn()
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn spawn(config: ModelConfig, shared: SharedFrame) -> Result<Self, Box<dyn Error>> {
        let cv = Self::new(config, shared);
        cv.start()?;
        Ok(cv)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Inference> {
        self.tx.subscribe()
    }
}