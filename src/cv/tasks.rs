use std::{error::Error, fmt::Debug};
use ndarray::Array4;

mod pose;
mod object;
mod segment;

use pose::Keypoints;
use object::Detections;

pub use pose::PoseTask;

pub trait VisionTask: Send + Sync + Debug {
    fn preprocess(&self,
        rgba: &[u8],
        width: u32,
        height: u32,) -> Array4<f32>;

    fn postprocess(
        &self,
        outputs: &ort::session::SessionOutputs,
        output_name: &str,
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
    Detections(Vec<Detections>),
    SegmentationMask(Vec<u8>),
}