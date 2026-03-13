mod subscriptions;
mod helpers;

use std::time::Duration;

use iced::widget::{column, row, button, container, image, stack, text};
use iced::{Alignment, Element, Fill, Font, Subscription, Theme};
use crate::app::helpers::metric_row;
use crate::cv::TimeMetrics;
use crate::{Frame, Inference};
use crate::utils::ManagedService;

enum InferenceState {
    Unloaded,
    Stopped,
    Running,
}

pub fn run() -> iced::Result {
    iced::application(
            move || App::new(crate::new_pipelines()),
            App::update,
            App::view,
        )
        .subscription(App::subscription)
        .theme(App::theme)
        .run()
}

pub struct App {
    pipelines: crate::Pipelines,

    cam_frame: Option<image::Handle>,
    cv_frame: Option<image::Handle>,
    
    model_load_time: Option<Duration>,
    time_metrics: Option<TimeMetrics>,

    inference_state: InferenceState,
}

#[derive(Debug, Clone)]
pub enum Message {
    CamFrame(image::Handle),
    CvInference((image::Handle, TimeMetrics)),
    LoadModelPressed,
    StartInferencePressed,
    StopInferencePressed,
}

impl App {
    fn new(pipelines: crate::Pipelines) -> Self {
        Self {
            pipelines,
            cam_frame: None,
            cv_frame: None,
            model_load_time: None,
            time_metrics: None,
            inference_state: InferenceState::Unloaded,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::CamFrame(frame) => {
                self.cam_frame = Some(frame);
            }
            Message::CvInference((frame, inf_time)) => {
                self.cv_frame = Some(frame);
                self.time_metrics = Some(inf_time);
            }
            Message::LoadModelPressed => {
                match self.pipelines.cv_manager.load_model() {
                    Ok(elapsed) => {
                        self.model_load_time = Some(elapsed);
                    }
                    Err(e) => {
                        eprintln!("Unable to load model: {}", e)
                    }
                };
                self.inference_state = InferenceState::Stopped;
            }
            Message::StartInferencePressed => {
                self.pipelines.camera_manager.start().expect("Unable to start camera");
                self.pipelines.cv_manager.start().expect("Unable to start model");
                self.inference_state = InferenceState::Running;
            }
            Message::StopInferencePressed => {
                self.pipelines.camera_manager.stop();
                self.pipelines.cv_manager.stop();
                self.inference_state = InferenceState::Stopped;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let img: Element<_> = match (&self.cam_frame, &self.cv_frame) {
            (Some(cam), Some(cv)) => {
                stack![
                    image(cam.clone()).width(Fill).height(Fill),
                    image(cv.clone()).width(Fill).height(Fill),
                ]
                .into()
            }
            (Some(cam), None) => {
                image(cam.clone())
                    .width(Fill)
                    .height(Fill)
                    .into()
            }
            _ => container("-------- Camera not started --------").into(),
        };

        let load_button = match self.inference_state {
            InferenceState::Running => {
                button("Load Model")
            }
            InferenceState::Stopped | InferenceState::Unloaded => {
                button("Load Model")
                    .on_press(Message::LoadModelPressed)
            }
        };

        let control_button = match self.inference_state {
            InferenceState::Running => {
                button("Stop Model")
                    .on_press(Message::StopInferencePressed)
            } 
            InferenceState::Stopped => {
                button("Start Model")
                    .on_press(Message::StartInferencePressed)
            }
            InferenceState::Unloaded => {
                button("Start Model")
            }
        };

        let model_load_label = row![
            text("Model Load Time: ")
            .font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::DEFAULT
            }).size(16),
            text(
                self.model_load_time
                    .map(|t| format!("{:?}", t))
                    .unwrap_or_else(|| "Not loaded".to_string())
            )
            .size(16)
        ].spacing(5);

        let preprocess_time_label = metric_row(
            "Preprocess Time:",
            self.time_metrics.map(|t| format!("{:?}", t.preprocess)),
        );

        let inference_time_label = metric_row(
            "Inference Time:",
            self.time_metrics.map(|t| format!("{:?}", t.inference)),
        );

        let postprocess_time_label = metric_row(
            "Postprocess Time:",
            self.time_metrics.map(|t| format!("{:?}", t.postprocess)),
        );

        let render_time_label = metric_row(
            "Render Time:",
            self.time_metrics.map(|t| format!("{:?}", t.render)),
        );

        let content = column![
            img,
            row![
                load_button,
                control_button
            ].spacing(40),
            row![
                model_load_label,
                column![
                    preprocess_time_label,
                    inference_time_label,
                    postprocess_time_label,
                    render_time_label,
                ]
            ].spacing(40)
        ]
        .spacing(20)
        .padding(20)
        .align_x(Alignment::Center);

        container(content)
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            subscriptions::raw_frame_subscription(self.pipelines.camera_manager.clone()).map(Message::CamFrame),
            subscriptions::inference_subscription(self.pipelines.cv_manager.clone()).map(Message::CvInference),
        ])
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
