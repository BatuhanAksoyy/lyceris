use event_emitter_rs::EventEmitter;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Default)]
pub struct Emitter {
    pub wrap: Arc<Mutex<EventEmitter>>,
}

#[derive(Debug)]
pub enum Event {
    MultipleDownloadProgress,
    SingleDownloadProgress,
    Console,
}

pub trait Emit {
    #[allow(async_fn_in_trait)]
    async fn emit<T: Serialize>(&self, event: Event, data: T);
}

impl Emit for Option<&Emitter> {
    async fn emit<T: Serialize>(&self, event: Event, data: T) {
        if let Some(emitter) = self {
            emitter
                .wrap
                .lock()
                .await
                .emit(&format!("{:?}", event), data);
        }
    }
}

impl Emitter {
    pub async fn emit<T: Serialize>(&self, event: Event, data: T) {
        self.wrap.lock().await.emit(&format!("{:?}", event), data);
    }

    pub async fn on<F, T>(&self, event: Event, listener: F)
    where
        F: Fn(T) + Send + Sync + 'static,
        T: for<'de> Deserialize<'de> + Serialize,
    {
        self.wrap.lock().await.on(&format!("{:?}", event), listener);
    }
}
