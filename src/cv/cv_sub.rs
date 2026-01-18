use iced::advanced::subscription::Hasher;
use iced::futures::stream;
use iced::advanced::subscription as iced_subscription;
use iced::widget::image;
use iced::Subscription;

use tokio::sync::broadcast;

use super::Inference;

/* ============================
   Iced Subscription
   ============================ */

pub fn inference_subscription(rx: broadcast::Receiver<Inference>) -> Subscription<image::Handle> {
    iced_subscription::from_recipe(CVSubscription::new(rx))
}

struct CVSubscription {
    rx: broadcast::Receiver<Inference>,
}

impl CVSubscription {
    pub fn new(rx: broadcast::Receiver<Inference>) -> Self {
        Self { rx }
    }
}

impl iced_subscription::Recipe for CVSubscription {
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
                // yield <conversion logic here>;
            }
        };
        Box::pin(s)
    }
}