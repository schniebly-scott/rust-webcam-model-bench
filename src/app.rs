mod subscriptions;

use iced::widget::{container, image, stack};
use iced::{Element, Fill, Subscription, Theme};

use crate::{Pipelines, Inference, Frame};

pub fn run(pipelines: Pipelines) -> iced::Result {
    iced::application(
            move || App::new(pipelines.clone()),
            App::update,
            App::view,
        )
        .subscription(App::subscription)
        .theme(App::theme)
        .run()
}

pub struct App {
    pipelines: Pipelines,

    cam_frame: Option<image::Handle>,
    cv_frame: Option<image::Handle>,
}

#[derive(Debug, Clone)]
pub enum Message {
    CamFrame(image::Handle),
    CvFrame(image::Handle),
}

impl App {
    fn new(pipelines: Pipelines) -> Self {
        Self {
            pipelines,
            cam_frame: None,
            cv_frame: None,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::CamFrame(frame) => {
                self.cam_frame = Some(frame);
            }
            Message::CvFrame(frame) => {
                self.cv_frame = Some(frame);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content: Element<_> = match (&self.cam_frame, &self.cv_frame) {
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
            _ => container("Waiting for frames...").into(),
        };

        container(content)
            .width(Fill)
            .height(Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            subscriptions::raw_frame_subscription(self.pipelines.camera_manager.clone()).map(Message::CamFrame),
            subscriptions::inference_subscription(self.pipelines.cv_manager.clone()).map(Message::CvFrame),
        ])
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
