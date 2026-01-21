pub mod app;
pub mod camera;
pub mod cv;

use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

use camera::{CameraManager, Frame};
use cv::{CVManager, InfType, Inference};

pub type SharedFrame = Arc<Mutex<Option<Frame>>>;

#[derive(Clone, Debug)]
pub struct Pipelines {
    pub camera_manager: Arc<CameraManager>,
    pub cv_manager: Arc<CVManager>
}

pub fn run() -> iced::Result {
    let model_path = "model.onnx";
    let data_type: InfType = InfType::Pose;

    let shared_frame: SharedFrame = Arc::new(Mutex::new(None));

    let camera_manager = Arc::new(CameraManager::spawn("/dev/video0", shared_frame.clone())
        .expect("Error starting camera"));
    let cv_manager = Arc::new(CVManager::spawn(model_path, data_type, shared_frame.clone())
        .expect("Error starting cv model"));

    let pipelines = Pipelines { camera_manager, cv_manager };

    app::run(pipelines)
}