use rscam::{Camera, Config};

use std::thread;
use std::time::Duration;
use std::error::Error;
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;

use yuv::{yuyv422_to_rgba, YuvPackedImage};

use super::{WIDTH, HEIGHT};

/// RGBA frame sent to the UI
/// (width, height, RgbaBuffer { frame, pool-pointer })
pub type Frame = (u32, u32, Arc<RgbaBuffer>);
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

pub struct CameraManager {
    device: String,
    tx: broadcast::Sender<Frame>,
}

impl CameraManager {
    pub fn new(device: &str) -> Self {
        let (tx, _) = broadcast::channel::<Frame>(2);

        Self {
            device: device.to_string(),
            tx,
        }
    }

    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        let mut camera = Camera::new(&self.device)?;

        camera.start(&Config {
            interval: (1, 30),
            resolution: (WIDTH, HEIGHT),
            ..Default::default()
        })?;

        eprintln!("Camera started successfully");

        let tx = self.tx.clone();
        let pool: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
        let pool_clone = pool.clone();

        thread::spawn(move || {
            let frame_len = (WIDTH * HEIGHT * 4) as usize;

            loop {
                match camera.capture() {
                    Ok(frame) => {
                        let mut rgba = {
                            let mut pool = pool_clone.lock().unwrap();
                            pool.pop().unwrap_or_else(|| vec![0u8; frame_len])
                        };

                        let yuv_image = YuvPackedImage {
                            yuy: &frame,
                            yuy_stride: WIDTH * 2,
                            width: WIDTH,
                            height: HEIGHT,
                        };
                        //TODO: Check if this can be done in the subscription depending on how ONNX model needs image data

                        if yuyv422_to_rgba(
                            &yuv_image,
                            &mut rgba,
                            WIDTH * 4,
                            yuv::YuvRange::Full,
                            yuv::YuvStandardMatrix::Bt601,
                        ).is_ok() {
                            let buf = RgbaBuffer {
                                data: rgba,
                                pool: pool_clone.clone(),
                            };

                            let _ = tx.send((WIDTH, HEIGHT, Arc::new(buf)));
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

    pub fn spawn(device: &str) -> Result<Self, Box<dyn Error>> {
        let cam = Self::new(device);
        cam.start()?;
        //TODO: add more error information above if it fails
        Ok(cam)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Frame> {
        self.tx.subscribe()
    }
}