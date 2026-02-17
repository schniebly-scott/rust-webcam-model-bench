use std::sync::atomic::{AtomicBool, Ordering};
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

use crate::SharedFrame;
use crate::config::CameraConfig;

use super::{ Frame, cam_worker::CameraWorker };

#[derive(Debug)]
pub struct RgbaBuffer {
    pub data: Vec<u8>,
    pub pool: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl Drop for RgbaBuffer {
    fn drop(&mut self) {
        let mut pool = self.pool.lock().unwrap();
        pool.push(std::mem::take(&mut self.data));
    }
}

#[derive(Debug)]
pub struct CameraManager {
    config: CameraConfig,
    tx: broadcast::Sender<Frame>,
    shared: SharedFrame,
    running: Arc<AtomicBool>,
}

impl CameraManager {
    pub fn new(config: CameraConfig, shared: SharedFrame) -> Self {
        let (tx, _) = broadcast::channel::<Frame>(2);

        Self {
            config,
            tx,
            shared: shared,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        self.running.store(true, Ordering::SeqCst);

        CameraWorker {
            config: self.config.clone(),
            tx: self.tx.clone(),
            shared: self.shared.clone(),
            running: self.running.clone(),
        }
        .spawn()
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn spawn(config: CameraConfig, shared: SharedFrame) -> Result<Self, Box<dyn Error>> {
        let cam = Self::new(config, shared);
        cam.start()?;
        //TODO: add more error information above if it fails
        Ok(cam)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Frame> {
        self.tx.subscribe()
    }
}