use rscam::{Camera, Config};

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use std::error::Error;
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;

use yuv::{yuyv422_to_rgba, YuvPackedImage};

use crate::SharedFrame;
use crate::camera::RgbaBuffer;
use crate::config::CameraConfig;

use super::Frame;

pub struct CameraWorker {
    pub config: CameraConfig,
    pub tx: broadcast::Sender<Frame>,
    pub shared: SharedFrame,
    pub running: Arc<AtomicBool>,
}

impl CameraWorker {
    pub fn spawn(self) -> Result<(), Box<dyn Error>> {
        let mut camera = Camera::new(&self.config.device)?;

        camera.start(&Config {
            interval: (1, 30),
            resolution: (self.config.width, self.config.height),
            ..Default::default()
        })?;

        println!("Camera started successfully");

        let pool: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));

        thread::spawn(move || {
            let frame_len = (self.config.width * self.config.height * 4) as usize;

            while self.running.load(Ordering::SeqCst) {
                match camera.capture() {
                    Ok(frame) => {
                        let mut rgba = {
                            let mut pool = pool.lock().unwrap();
                            pool.pop().unwrap_or_else(|| vec![0u8; frame_len])
                        };

                        let yuv_image = YuvPackedImage {
                            yuy: &frame,
                            yuy_stride: self.config.width * 2,
                            width: self.config.width,
                            height: self.config.height,
                        };

                        if yuyv422_to_rgba(
                            &yuv_image,
                            &mut rgba,
                            self.config.width * 4,
                            yuv::YuvRange::Full,
                            yuv::YuvStandardMatrix::Bt601,
                        ).is_ok() {
                            let buf = RgbaBuffer {
                                data: rgba,
                                pool: pool.clone(),
                            };

                            let captured_frame: Frame = (self.config.width, self.config.height, Arc::new(buf));

                            let mut slot = self.shared.lock().unwrap();
                            *slot = Some(captured_frame.clone()); // overwrite old frame for ML

                            let _ = self.tx.send(captured_frame); // Send to UI
                        } else {
                            pool.lock().unwrap().push(rgba);
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
}