pub mod app;
pub mod camera;
pub mod cv;

use std::sync::{Arc, Mutex};
use camera::{CameraManager, raw_frame_subscription, Frame};
use cv::{CVManager, inference_subscription, InfType};

pub type SharedFrame = Arc<Mutex<Option<Frame>>>;

pub fn run() -> iced::Result {
    let model_path = "model.onnx";
    let data_type: InfType = InfType::Pose;

    let shared_frame: SharedFrame = Arc::new(Mutex::new(None));

    let camera = Arc::new(CameraManager::spawn("/dev/video0", shared_frame.clone())
        .expect("Error starting camera"));
    let cv_manager = Arc::new(CVManager::spawn(model_path, data_type, shared_frame.clone())
        .expect("Error starting cv model"));
    
    app::run(move |_| {
        raw_frame_subscription(camera.subscribe())
            .map(app::Message::NewFrame)
    })
}