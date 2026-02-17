use rscam::{Camera, Config};

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use std::error::Error;
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;

use yuv::{yuyv422_to_rgba, YuvPackedImage};

use crate::SharedFrame;
use crate::config::CameraConfig;

/// RGBA frame sent to the UI
/// (width, height, RgbaBuffer { frame, pool-pointer })
pub type Frame = (u32, u32, Arc<RgbaBuffer>);

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
        let mut camera = Camera::new(&self.config.device)?;

        camera.start(&Config {
            interval: (1, 30),
            resolution: (self.config.width, self.config.height),
            ..Default::default()
        })?;

        println!("Camera started successfully");

        let tx = self.tx.clone();
        let shared_clone = self.shared.clone();
        let pool: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
        let pool_clone = pool.clone();
        let width_clone = self.config.width;
        let height_clone = self.config.height;

        let running_clone = self.running.clone();
        running_clone.store(true, Ordering::SeqCst);

        thread::spawn(move || {
            let frame_len = (width_clone * height_clone * 4) as usize;

            while running_clone.load(Ordering::SeqCst) {
                match camera.capture() {
                    Ok(frame) => {
                        let mut rgba = {
                            let mut pool = pool_clone.lock().unwrap();
                            pool.pop().unwrap_or_else(|| vec![0u8; frame_len])
                        };

                        let yuv_image = YuvPackedImage {
                            yuy: &frame,
                            yuy_stride: width_clone * 2,
                            width: width_clone,
                            height: height_clone,
                        };

                        if yuyv422_to_rgba(
                            &yuv_image,
                            &mut rgba,
                            width_clone * 4,
                            yuv::YuvRange::Full,
                            yuv::YuvStandardMatrix::Bt601,
                        ).is_ok() {
                            let buf = RgbaBuffer {
                                data: rgba,
                                pool: pool_clone.clone(),
                            };

                            let captured_frame: Frame = (width_clone, height_clone, Arc::new(buf));

                            let mut slot = shared_clone.lock().unwrap();
                            *slot = Some(captured_frame.clone()); // overwrite old frame for ML

                            let _ = tx.send(captured_frame); // Send to UI
                        } else {
                            pool_clone.lock().unwrap().push(rgba);
                        }
                    }
                    Err(e) => {
                        eprintln!("Camera capture failed: {}", e);
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        });

        Ok(())
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