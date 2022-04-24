use std::sync::{Arc, Mutex};

use anyhow::{Error, Result};

use std::task::Waker;

/// A struct that tracks the wakers associated with the monitor stream endpoint.
pub struct WakeWatcher {
    wakers: Mutex<Vec<Arc<Waker>>>,
}

impl WakeWatcher {
    pub fn new() -> Self {
        Self {
            wakers: Mutex::new(vec![]),
        }
    }

    /// Wake all of the wakers and clear them.
    pub fn wake(&self) -> Result<()> {
        let mut wakers = self
            .wakers
            .lock()
            .map_err(|err| Error::msg(err.to_string()))?;
        wakers.iter().for_each(|waker| {
            waker.wake_by_ref();
        });
        wakers.clear();

        Ok(())
    }

    /// Add a waker to the list.
    pub fn add_waker(&self, waker: Waker) -> Result<()> {
        let mut wakers = self
            .wakers
            .lock()
            .map_err(|err| Error::msg(err.to_string()))?;
        wakers.push(Arc::new(waker));
        Ok(())
    }
}
