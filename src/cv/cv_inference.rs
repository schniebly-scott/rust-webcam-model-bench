use std::{error::Error, time::Instant};

use ort::{inputs, session::Session, value::TensorRef};

use crate::{config::ModelConfig, cv::{InfType, TimeMetrics, tasks::{PoseTask, VisionTask}}};

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
    ) -> Result<(Vec<u8>, TimeMetrics), Box<dyn Error>> {
        let t0 = Instant::now();
        let input = self.task.preprocess(rgba, width, height);
        let preprocess = t0.elapsed();

        let t1 = Instant::now();
        let outputs = self.session.run(
            inputs![&self.input_name => TensorRef::from_array_view(&input)?]
        )?;
        let inference = t1.elapsed();

        let t2 = Instant::now();
        let result = self.task.postprocess(&outputs, &self.output_name, width, height)?;
        let postprocess = t2.elapsed();

        let t3 = Instant::now();
        let img = self.task.render(&result, width, height);
        let render = t3.elapsed();

        Ok((img, TimeMetrics {
            preprocess,
            postprocess,
            inference,
            render
        }))
    }
}