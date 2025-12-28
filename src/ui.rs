use iced::widget::{container, image};
use iced::{Element, Fill, Subscription, Theme};

use crate::camera;

pub fn run() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .run()
}

struct App {
    frame: Option<image::Handle>,
}

#[derive(Debug, Clone)]
pub enum Message {
    NewFrame(image::Handle),
}

impl App {
    fn new() -> Self {
        Self { frame: None }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::NewFrame(handle) => {
                self.frame = Some(handle);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = if let Some(frame) = &self.frame {
            Element::from(
                image(frame.clone())
                    .width(Fill)
                    .height(Fill)
            )
        } else {
            Element::from(
                container("Waiting for frames...")
            )
        };

        container(content).padding(20).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        camera::subscription().map(Message::NewFrame)
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
