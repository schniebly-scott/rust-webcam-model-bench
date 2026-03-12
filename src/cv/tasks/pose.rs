mod constants;

use std::error::Error;
use crate::{config::{InferenceGenericConfig, PoseConfig}, cv::tasks::pose::constants::KPT_START};

use super::{VisionTask, TaskResult};
use raqote::{
    DrawOptions, DrawTarget, LineJoin, PathBuilder,
    SolidSource, Source, StrokeStyle,
};
use image::{DynamicImage, imageops::FilterType};
use ndarray::{Array4, Axis};
use constants::SKELETON;

pub type Keypoints = [Option<(f32, f32, f32)>; 17];

#[derive(Debug)]

pub struct PoseTask {
    config: PoseConfig,
    inf_width: usize,
    inf_height: usize,
    confidence_threshold: f32,
}

impl PoseTask {
    pub fn new(generics: &InferenceGenericConfig, pose_config: &PoseConfig) -> Self {
        Self { 
            config: pose_config.clone(),
            inf_width: generics.inf_width,
            inf_height: generics.inf_height,
            confidence_threshold: generics.confidence_threshold,
        }
    }

    fn decode_heatmap_pose(
        &self,
        heatmaps: &Array4<f32>,
        orig_w: u32,
        orig_h: u32,
    ) -> Keypoints {
        let mut keypoints: Keypoints = [None; 17];
        let maps = heatmaps.index_axis(Axis(0), 0);

        let hm_h = maps.len_of(Axis(1));
        let hm_w = maps.len_of(Axis(2));

        let scale_x = orig_w as f32 / self.inf_width as f32;
        let scale_y = orig_h as f32 / self.inf_height as f32;

        for &k in &self.config.keep_keypoints {
            let map = maps.index_axis(Axis(0), k);
            let slice = map.as_slice().unwrap();

            let (mut max_val, mut max_idx) = (f32::MIN, 0);
            for (i, &v) in slice.iter().enumerate() {
                if v > max_val {
                    max_val = v;
                    max_idx = i;
                }
            }

            let y = max_idx / hm_w;
            let x = max_idx % hm_w;

            let x_img = (x as f32 * self.inf_width as f32 / hm_w as f32) * scale_x;
            let y_img = (y as f32 * self.inf_height as f32 / hm_h as f32) * scale_y;

            keypoints[k] = Some((x_img, y_img, max_val));
        }

        keypoints
    }

    fn decode_yolo_pose(
        &self,
        preds: &ndarray::Array3<f32>,
        orig_w: u32,
        orig_h: u32,
    ) -> Result<Keypoints, Box<dyn Error>> {

        // shape: [1, 56, 8400]
        let preds = preds.index_axis(ndarray::Axis(0), 0);

        // transpose to [8400, 56]
        let preds = preds.permuted_axes([1, 0]);

        let mut best_conf = 0.0;
        let mut best_row = None;

        for row in preds.outer_iter() {
            let conf = row[4];

            if conf > best_conf {
                best_conf = conf;
                best_row = Some(row);
            }
        }

        let row = best_row.ok_or("No detections")?;

        let mut keypoints: Keypoints = [None; 17];

        let scale_x = orig_w as f32 / self.inf_width as f32;
        let scale_y = orig_h as f32 / self.inf_height as f32;
        let kpt_start = KPT_START; // after bbox + obj + class

        for &k in &self.config.keep_keypoints {
            let base = kpt_start + k * 3;

            let x = row[base] * scale_x as f32;
            let y = row[base + 1] * scale_y as f32;
            let conf = row[base + 2];

            keypoints[k] = Some((x, y, conf));
        }

        Ok(keypoints)
    }

    fn render_pose(&self, keypoints: &Keypoints, width: u32, height: u32) -> Vec<u8> {
        // ----- Draw -----
        let mut dt = DrawTarget::new(width as i32, height as i32);
        self.draw_skeleton(&mut dt, &keypoints);

        // ----- Extract RGBA back out -----
        let data = dt.get_data();
        let mut out = Vec::with_capacity((width * height * 4) as usize);

        for px in data {
            out.push((px >> 16) as u8); // R
            out.push((px >> 8) as u8);  // G
            out.push(*px as u8);        // B
            out.push((px >> 24) as u8); // A
        }
        out
    }

    fn draw_skeleton(&self, dt: &mut DrawTarget, keypoints: &Keypoints) {
        for &(i, j) in SKELETON {
            if let (Some((x1, y1, c1)), Some((x2, y2, c2))) =
                (keypoints[i], keypoints[j])
            {
                if c1 < self.confidence_threshold || c2 < self.confidence_threshold {
                    continue;
                }

                let mut pb = PathBuilder::new();
                pb.move_to(x1, y1);
                pb.line_to(x2, y2);

                dt.stroke(
                    &pb.finish(),
                    &Source::Solid(SolidSource { r: 255, g: 0, b: 0, a: 255 }),
                    &StrokeStyle {
                        width: 2.0,
                        join: LineJoin::Round,
                        ..Default::default()
                    },
                    &DrawOptions::new(),
                );
            }
        }

        for &(x, y, c) in keypoints.iter().flatten() {
            if c < self.confidence_threshold {
                continue;
            }

            let mut pb = PathBuilder::new();
            pb.arc(x, y, 4.0, 0.0, std::f32::consts::TAU);

            dt.fill(
                &pb.finish(),
                &Source::Solid(SolidSource { r: 0, g: 255, b: 0, a: 255 }),
                &DrawOptions::new(),
            );
        }
    }
}

impl VisionTask for PoseTask {
    fn preprocess(
        &self,
        rgba: &[u8],
        width: u32,
        height: u32,
    ) -> Array4<f32> {

        let w = self.inf_width;
        let h = self.inf_height;

        let mut input = Array4::<f32>::zeros((1, 3, h, w));
        let out = input.as_slice_mut().unwrap();

        let hw = h * w;
        let scale = 1.0 / 255.0;

        let x_ratio = width as f32 / w as f32;
        let y_ratio = height as f32 / h as f32;

        for y in 0..h {
            let src_y = (y as f32 * y_ratio) as usize;

            for x in 0..w {
                let src_x = (x as f32 * x_ratio) as usize;

                let src_i = ((src_y * width as usize + src_x) * 4) as usize;
                let dst_i = y * w + x;

                let r = rgba[src_i];
                let g = rgba[src_i + 1];
                let b = rgba[src_i + 2];

                out[dst_i] = r as f32 * scale;
                out[hw + dst_i] = g as f32 * scale;
                out[2 * hw + dst_i] = b as f32 * scale;
            }
        }

        input
    }

    fn postprocess(
        &self,
        outputs: &ort::session::SessionOutputs,
        output_name: &str,
        orig_w: u32,
        orig_h: u32,
    ) -> Result<TaskResult, Box<dyn Error>> {
        let tensor = outputs
            .get(output_name)
            .ok_or("Missing output tensor")?;

        let array = tensor
            .try_extract_array::<f32>()?
            .into_owned();

        match array.ndim() {
            // Heatmap model
            4 => {
                let heatmaps = array.into_dimensionality::<ndarray::Ix4>()?;
                let keypoints = self.decode_heatmap_pose(&heatmaps, orig_w, orig_h);
                Ok(TaskResult::Pose(keypoints))
            }

            // YOLO model
            3 => {
                let preds = array.into_dimensionality::<ndarray::Ix3>()?;
                let keypoints = self.decode_yolo_pose(&preds, orig_w, orig_h)?;
                Ok(TaskResult::Pose(keypoints))
            }

            _ => Err("Unsupported output shape".into()),
        }
    }

    fn render(
        &self,
        result: &TaskResult,
        width: u32,
        height: u32,
    ) -> Vec<u8> {
        match result {
            TaskResult::Pose(keypoints) => self.render_pose(keypoints, width, height),
            _ => unreachable!(),
        }
    }
}