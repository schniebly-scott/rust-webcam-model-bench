use std::error::Error;

use image::{DynamicImage, ImageBuffer, Rgba};
use ort::{inputs, session::Session, value::TensorRef};

use crate::{config::ModelConfig, cv::{InfType, tasks::{PoseTask, VisionTask}}};

#[derive(Debug)]
pub struct Model {
    session: Session,
    task: Box<dyn VisionTask + Send + Sync>,
    input_name: String,
    output_name: String,
}

impl Model {
    pub fn from_config(config: &ModelConfig) -> ort::Result<Self> {
        let session = Session::builder()?
            .commit_from_file(&config.model_path)?;

        let task: Box<dyn VisionTask + Send + Sync> =
            match config.inference_type {
                InfType::Pose => Box::new(PoseTask::new(&config.generics, config.pose.as_ref().unwrap())),
                InfType::BoundingBox => todo!(),
                InfType::Segment => todo!(),
            };

        let input_name = session.inputs()[0].name().to_string();
        let output_name = session.outputs()[0].name().to_string();

        Ok(Self { 
            session,
            task,
            input_name,
            output_name,
        })
    }

    pub fn process_rgba(
        &mut self,
        rgba: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let img = wrap_rgba(rgba, width, height);

        let input = self.task.preprocess(&img);

        let outputs = self.session.run(
            inputs![&self.input_name => TensorRef::from_array_view(&input)?]
        )?;

        let result = self.task.postprocess(&outputs, &self.output_name, width, height)?;

        Ok(self.task.render(&result, width, height))
    }
}

fn wrap_rgba(rgba: &[u8], width: u32, height: u32) -> DynamicImage {
    let img = DynamicImage::ImageRgba8(
        ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, rgba.to_vec())
            .expect("Invalid RGBA buffer"),
    );
    img
}