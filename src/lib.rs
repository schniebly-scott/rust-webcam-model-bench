pub mod app;
pub mod camera;
pub mod cv;

use std::sync::Arc;
use camera::{CameraManager, raw_frame_subscription};
use cv::{CVManager, inference_subscription, InfType};

pub fn run() -> iced::Result {
    let model_path = "model.onnx";
    let data_type: InfType = InfType::Pose;

    let camera = Arc::new(CameraManager::spawn("/dev/video0").expect("Error starting camera"));
    let cv_manager = Arc::new(CVManager::spawn(camera.subscribe(), model_path, data_type));
    
    let ui_camera = Arc::clone(&camera);
    
    app::run(move |_| {
        raw_frame_subscription(ui_camera.subscribe())
            .map(app::Message::NewFrame)
    })
}