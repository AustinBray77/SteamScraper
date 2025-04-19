use std::{error::Error, future::Future};

use crate::error::SteamError;

struct AsyncProcessRunner<T, Fut: Future<Output = T>> {
    processes: Vec<fn(T) -> Fut>,
    last: T,
}

impl<T, Fut: Future<Output = T>> AsyncProcessRunner<T, Fut> {
    fn new(init: T) -> Self {
        AsyncProcessRunner {
            processes: Vec::new(),
            last: init,
        }
    }

    async fn run_next(mut self) -> Result<(), Box<dyn Error>> {
        if let Some(func) = self.processes.first() {
            self.last = func(self.last).await;
            Ok(())
        } else {
            Err(SteamError::boxed_new("No process available"))
        }
    }
}
