pub mod app;
pub mod camera;
pub mod cv;

use std::sync::Arc;
use camera::{CameraManager, subscription};

pub fn run() -> iced::Result {
    let camera = Arc::new(CameraManager::new("/dev/video0"));
    camera.start().unwrap();

    // CV subscription
    let cv_rx = camera.subscribe();

    // UI subscription
    let ui_camera = Arc::clone(&camera);
    app::run(move |_| {
        subscription(ui_camera.subscribe())
            .map(app::Message::NewFrame)
    })
}