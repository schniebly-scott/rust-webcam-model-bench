use anyhow::Result;
use bytes::Bytes;
use rscam::{Camera, Config};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::task;
use iced::Subscription;
use iced::advanced::subscription;
use iced::widget::image;
use iced::futures::stream;
use iced::advanced::subscription::Hasher;


pub struct CameraManager {
    device: String,
}

impl CameraManager {
    pub fn new(device: &str) -> Self {
        Self {
            device: device.to_string(),
        }
    }

    pub fn start(&self) -> Result<broadcast::Receiver<Bytes>> {
        let mut camera = Camera::new(&self.device)?;

        camera.start(&Config {
            interval: (1, 30), // 30 fps
            resolution: (1280, 720),
            format: b"MJPG",
            ..Default::default()
        })?;
        eprintln!("Camera started successfully");

        let (tx, rx) = broadcast::channel(30); // Channel capacity

        let _device = self.device.clone();
        std::thread::spawn(move || {
            loop {
                match camera.capture() {
                    Ok(frame) => {
                        eprintln!("Captured frame: {} bytes", frame.len()); 
                        let bytes = Bytes::copy_from_slice(&frame[..]);
                        if tx.send(bytes).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to capture frame: {}", e);
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
                
            }
        });

        Ok(rx)
    }
}


pub fn subscription() -> Subscription<image::Handle> {
    subscription::from_recipe(CameraSubscription)
}

pub struct CameraSubscription;

impl subscription::Recipe for CameraSubscription
{
    type Output = image::Handle;

    fn hash(&self, state: &mut Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(self: Box<Self>, _input: stream::BoxStream<subscription::Event>) -> stream::BoxStream<Self::Output> {
        let camera_manager = CameraManager::new("/dev/video0");
        match camera_manager.start() {
            Ok(mut rx) => {
                eprintln!("Received frame from broadcast");
                let stream = async_stream::stream! {
                    while let Ok(frame_data) = rx.recv().await {
                        match ::image::load_from_memory(&frame_data) {
                            Ok(img) => {
                                let rgba_img = img.to_rgba8();
                                let (width, height) = rgba_img.dimensions();
                                let handle = image::Handle::from_rgba(width, height, rgba_img.into_raw());
                                yield handle;
                            }
                            Err(e) => {
                                eprintln!("Failed to decode frame: {}", e);
                            }
                        }
                    }
                };
                Box::pin(stream)
            }
            Err(e) => {
                eprintln!("Failed to start camera: {}", e);
                // Return an empty stream on error
                Box::pin(stream::empty())
            }
        }
    }
}