use std::{sync::mpsc, thread};

/// An asynchronous garbage collector that manages the lifecycle of objects in a background task.
/// It allows dropping items asynchronously without blocking the main thread.
#[derive(Debug)]
pub struct AsyncDropper<T> {
    _sender: mpsc::Sender<T>,
    _handle: thread::JoinHandle<()>,
}

impl<T: Send + 'static> Default for AsyncDropper<T> {
    fn default() -> Self {
        let (_sender, receiver) = mpsc::channel();
        Self {
            _sender,
            _handle: std::thread::spawn(move || receiver.into_iter().for_each(drop)),
        }
    }
}

impl<T> AsyncDropper<T> {
    #[inline]
    pub fn drop(&self, t: T) {
        let _ = self._sender.send(t);
    }
}
