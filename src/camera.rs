use anyhow::Result;
use rscam::{Camera, Config};

use std::thread;
use std::time::Duration;
use std::io::Cursor;

use tokio::sync::broadcast;

use iced::Subscription;
use iced::advanced::subscription;
use iced::advanced::subscription::Hasher;
use iced::futures::stream;
use iced::widget::image;

use ::image::ImageReader;

/// RGBA frame sent to the UI
/// (width, height, pixels)
type Frame = (u32, u32, Vec<u8>);

pub struct CameraManager {
    device: String,
}

impl CameraManager {
    pub fn new(device: &str) -> Self {
        Self {
            device: device.to_string(),
        }
    }

    pub fn start(&self) -> Result<broadcast::Receiver<Frame>> {
        let mut camera = Camera::new(&self.device)?;

        camera.start(&Config {
            interval: (1, 30),
            resolution: (1280, 720),
            format: b"MJPG",
            ..Default::default()
        })?;

        eprintln!("Camera started successfully");

        let (tx, rx) = broadcast::channel::<Frame>(2);

        thread::spawn(move || {
            loop {
                match camera.capture() {
                    Ok(frame) => {
                        let decoded = match ImageReader::new(Cursor::new(&frame[..])).with_guessed_format() {
                            Ok(reader) => reader.decode(),
                            Err(e) => Err(e.into()),
                        };

                        match decoded {
                            Ok(img) => {
                                let rgba = img.to_rgba8();
                                let (w, h) = rgba.dimensions();
                                let _ = tx.send((w, h, rgba.into_raw()));
                            }
                            Err(e) => {
                                eprintln!("JPEG decode failed: {}", e);
                            }
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