use iced::widget::{container, image};
use iced::{Element, Fill, Subscription, Theme};

pub fn run<S>(subscription: S) -> iced::Result
where
    S: Fn(&App) -> Subscription<Message> + 'static,
{
    iced::application(App::new, App::update, App::view)
        .subscription(subscription)
        .theme(App::theme)
        .run()
}

pub struct App {
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

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
