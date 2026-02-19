use super::{VisionTask, TaskResult};

pub struct PoseTask {
    keep_keypoints: [usize; 5],
}

impl VisionTask for PoseTask {
    fn preprocess(&self, img: &DynamicImage) -> Array4<f32> {
        preprocess_image(img)
    }

    fn postprocess(
        &self,
        outputs: &ort::SessionOutputs,
        orig_w: u32,
        orig_h: u32,
    ) -> anyhow::Result<TaskResult> {

        let heatmaps = outputs["output"]
            .try_extract_array::<f32>()?
            .into_owned()
            .into_dimensionality::<ndarray::Ix4>()?;

        let keypoints = decode_pose_fast(&heatmaps, orig_w, orig_h);

        Ok(TaskResult::Pose(keypoints))
    }

    fn render(
        &self,
        result: &TaskResult,
        width: u32,
        height: u32,
    ) -> Vec<u8> {
        match result {
            TaskResult::Pose(keypoints) => render_pose(keypoints, width, height),
            _ => unreachable!(),
        }
    }
}
