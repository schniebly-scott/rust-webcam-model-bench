use std::error::Error;
use ndarray::{Array4, Axis};
use image::DynamicImage;

mod pose_task;

pub trait VisionTask {
    fn preprocess(&self, img: &DynamicImage) -> Array4<f32>;

    fn postprocess(
        &self,
        outputs: &ort::session::SessionOutputs,
        orig_width: u32,
        orig_height: u32,
    ) -> Result<TaskResult, Box<dyn Error>>;

    fn render(
        &self,
        result: &TaskResult,
        width: u32,
        height: u32,
    ) -> Vec<u8>;
}

pub enum TaskResult {
    Pose(Keypoints),
    Detections(Vec<Detection>),
    SegmentationMask(Vec<u8>),
}