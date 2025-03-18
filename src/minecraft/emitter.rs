use event_emitter_rs::EventEmitter;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// A struct that wraps an `EventEmitter` for handling events asynchronously.
#[derive(Clone, Default)]
pub struct Emitter {
    pub wrap: Arc<Mutex<EventEmitter>>,
}

/// Enum representing different types of events that can be emitted.
#[derive(Debug)]
pub enum Event {
    /// Event triggered for multiple download progress updates.
    MultipleDownloadProgress,
    /// Event triggered for a single download progress update.
    SingleDownloadProgress,
    /// Event triggered for console output.
    Console,
}

/// Trait for emitting events.
pub trait Emit {
    /// Emits an event with associated data.
    ///
    /// # Parameters
    /// - `event`: The event to emit.
    /// - `data`: The data associated with the event.
    #[allow(async_fn_in_trait)]
    async fn emit<T: Serialize>(&self, event: Event, data: T);
}

/// Implementation of the `Emit` trait for an optional reference to `Emitter`.
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
    /// Emits an event with associated data.
    ///
    /// # Parameters
    /// - `event`: The event to emit.
    /// - `data`: The data associated with the event.
    pub async fn emit<T: Serialize>(&self, event: Event, data: T) {
        self.wrap.lock().await.emit(&format!("{:?}", event), data);
    }

    /// Registers a listener for a specific event.
    ///
    /// # Parameters
    /// - `event`: The event to listen for.
    /// - `listener`: A function that will be called when the event is emitted.
    ///
    /// # Type Parameters
    /// - `F`: The type of the listener function.
    /// - `T`: The type of data that the listener will receive.
    pub async fn on<F, T>(&self, event: Event, listener: F)
    where
        F: Fn(T) + Send + Sync + 'static,
        T: for<'de> Deserialize<'de> + Serialize,
    {
        self.wrap.lock().await.on(&format!("{:?}", event), listener);
    }
}
