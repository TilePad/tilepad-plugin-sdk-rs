use std::sync::Arc;

use parking_lot::Mutex;
use tokio::sync::oneshot;

use crate::protocol::ServerPluginMessage;

#[derive(Default, Clone)]
pub(crate) struct Subscriptions {
    subscribers: Arc<Mutex<Vec<Subscriber>>>,
}

impl Subscriptions {
    pub fn add(&self, subscriber: Subscriber) {
        self.subscribers.lock().push(subscriber);
    }

    pub fn apply(&self, msg: &ServerPluginMessage) {
        self.subscribers.lock().retain_mut(|subscriber| {
            if (subscriber.filter)(msg) {
                if let Some(tx) = subscriber.tx.take() {
                    _ = tx.send(msg.clone());
                }

                return false;
            }

            true
        });
    }
}

pub(crate) struct Subscriber {
    /// Function to filter for the desired plugin message type
    filter: Box<dyn Fn(&ServerPluginMessage) -> bool + Send + Sync + 'static>,

    /// Sender to send the matched message to
    tx: Option<oneshot::Sender<ServerPluginMessage>>,
}

impl Subscriber {
    pub fn new<F>(filter: F, tx: oneshot::Sender<ServerPluginMessage>) -> Self
    where
        F: Fn(&ServerPluginMessage) -> bool + Send + Sync + 'static,
    {
        Self {
            filter: Box::new(filter),
            tx: Some(tx),
        }
    }
}
