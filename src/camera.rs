use rscam::{Camera, Config};

use std::thread;
use std::time::Duration;
use std::error::Error;

use tokio::sync::broadcast;

use iced::Subscription;
use iced::advanced::subscription;
use iced::advanced::subscription::Hasher;
use iced::futures::stream;
use iced::widget::image;

use yuv::{yuyv422_to_rgba, YuvPackedImage};

/// RGBA frame sent to the UI
/// (width, height, pixels)
type Frame = (u32, u32, Vec<u8>);

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

        thread::spawn(move || {
            loop {
                match camera.capture() {
                    Ok(frame) => {
                        let mut rgba = vec![0u8; (CameraManager::WIDTH * CameraManager::HEIGHT * 4) as usize];
                        let yuv_image = YuvPackedImage {
                            yuy: &frame,
                            yuy_stride: CameraManager::WIDTH * 2, // YUYV422 has 2 bytes per pixel
                            width: CameraManager::WIDTH,
                            height: CameraManager::HEIGHT,
                        };
                        if let Ok(_) = yuyv422_to_rgba(
                            &yuv_image,
                            &mut rgba,
                            CameraManager::WIDTH * 4, // RGBA stride
                            yuv::YuvRange::Full,
                            yuv::YuvStandardMatrix::Bt601,
                        ) {
                            let _ = tx.send((CameraManager::WIDTH, CameraManager::HEIGHT, rgba));
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
                    while let Ok((w, h, rgba)) = rx.recv().await {
                        yield image::Handle::from_rgba(w, h, rgba);
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