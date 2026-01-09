use iced::advanced::subscription::Hasher;
use iced::futures::stream;
use iced::advanced::subscription as iced_subscription;
use iced::widget::image;
use iced::Subscription;

use tokio::sync::broadcast;

use super::Frame;

/* ============================
   Iced Subscription
   ============================ */

pub fn subscription(rx: broadcast::Receiver<Frame>) -> Subscription<image::Handle> {
    iced_subscription::from_recipe(CameraSubscription::new(rx))
}

struct CameraSubscription {
    rx: broadcast::Receiver<Frame>,
}

impl CameraSubscription {
    pub fn new(rx: broadcast::Receiver<Frame>) -> Self {
        Self { rx }
    }
}

impl iced_subscription::Recipe for CameraSubscription {
    type Output = image::Handle;

    fn hash(&self, state: &mut Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: stream::BoxStream<iced_subscription::Event>,
    ) -> stream::BoxStream<Self::Output> {
        let mut rx = self.rx;

        let s = async_stream::stream! {
            while let Ok(frame) = rx.recv().await {
                yield image::Handle::from_rgba(frame.0, frame.1, frame.2.data.clone());
            }
        };
        Box::pin(s)
    }
}