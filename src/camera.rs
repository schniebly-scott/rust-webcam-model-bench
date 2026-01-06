use rscam::{Camera, Config};

use std::thread;
use std::time::Duration;
use std::error::Error;
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;

use iced::Subscription;
use iced::advanced::subscription;
use iced::advanced::subscription::Hasher;
use iced::futures::stream;
use iced::widget::image;

use yuv::{yuyv422_to_rgba, YuvPackedImage};

/// RGBA frame sent to the UI
/// (width, height, pixels)
type Frame = (u32, u32, Arc<RgbaBuffer>);

pub struct RgbaBuffer {
    data: Vec<u8>,
    pool: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl Drop for RgbaBuffer {
    fn drop(&mut self) {
        let mut pool = self.pool.lock().unwrap();
        pool.push(std::mem::take(&mut self.data));
    }
}

pub struct CameraManager {
    device: String,
}

impl CameraManager {
    pub const WIDTH: u32 = 640;
    pub const HEIGHT: u32 = 480;

    pub fn new(device: &str) -> Self {
        Self {
            device: device.to_string(),
        }
    }

    pub fn start(&self) -> Result<broadcast::Receiver<Frame>, Box<dyn Error>> {
        let mut camera = Camera::new(&self.device)?;

        camera.start(&Config {
            interval: (1, 30), // FPS
            resolution: (Self::WIDTH, Self::HEIGHT),
            ..Default::default() // YUYV format
        })?;

        eprintln!("Camera started successfully");

        let (tx, rx) = broadcast::channel::<Frame>(2);
        let pool: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));

        let pool_clone = pool.clone();

        thread::spawn(move || {
            let frame_len = (CameraManager::WIDTH * CameraManager::HEIGHT * 4) as usize;

            loop {
                match camera.capture() {
                    Ok(frame) => {
                        // Try to reuse a buffer
                        let mut rgba = {
                            let mut pool = pool_clone.lock().unwrap();
                            pool.pop().unwrap_or_else(|| vec![0u8; frame_len])
                        };

                        let yuv_image = YuvPackedImage {
                            yuy: &frame,
                            yuy_stride: CameraManager::WIDTH * 2,
                            width: CameraManager::WIDTH,
                            height: CameraManager::HEIGHT,
                        };

                        if yuyv422_to_rgba(
                            &yuv_image,
                            &mut rgba,
                            CameraManager::WIDTH * 4,
                            yuv::YuvRange::Full,
                            yuv::YuvStandardMatrix::Bt601,
                        ).is_ok() {
                            let buf = RgbaBuffer {
                                data: rgba,
                                pool: pool_clone.clone(),
                            };

                            let arc = Arc::new(buf);
                            let _ = tx.send((CameraManager::WIDTH, CameraManager::HEIGHT, arc));
                        } else {
                            // Conversion failed, return buffer to pool
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


        Ok(rx)
    }
}

/* ============================
   Iced Subscription
   ============================ */

pub fn subscription() -> Subscription<image::Handle> {
    subscription::from_recipe(CameraSubscription)
}

struct CameraSubscription;

impl subscription::Recipe for CameraSubscription {
    type Output = image::Handle;

    fn hash(&self, state: &mut Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: stream::BoxStream<subscription::Event>,
    ) -> stream::BoxStream<Self::Output> {
        let manager = CameraManager::new("/dev/video0");

        match manager.start() {
            Ok(mut rx) => {
                let s = async_stream::stream! {
                    while let Ok((w, h, buf)) = rx.recv().await {
                        yield image::Handle::from_rgba(w, h, buf.data.clone());
                    }
                };
                Box::pin(s)
            }
            Err(e) => {
                eprintln!("Failed to start camera: {}", e);
                Box::pin(stream::empty())
            }
        }
    }
}