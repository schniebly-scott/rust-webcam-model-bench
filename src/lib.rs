pub mod app;
pub mod camera;
pub mod cv;

use std::sync::Arc;
use camera::{CameraManager, raw_frame_subscription};

pub fn run() -> iced::Result {
    let camera = Arc::new(CameraManager::new("/dev/video0"));
    camera.start().unwrap();

    // CV subscription
    // let cv_service = cv::spawn(camera.subscribe());

    // UI subscription
    let ui_camera = Arc::clone(&camera);
    app::run(move |_| {
        raw_frame_subscription(ui_camera.subscribe())
            .map(app::Message::NewFrame)
    })
}