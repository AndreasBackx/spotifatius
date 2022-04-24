use std::pin::Pin;
use std::sync::Arc;

use super::api::MonitorResponse;
use anyhow::Result;
use futures::Stream;

use tokio::sync::broadcast::error::TryRecvError;
use tokio::sync::broadcast::Receiver;
use tonic::Status;

use std::task::{Context, Poll};

use super::wake_watcher::WakeWatcher;

pub struct MonitorClient {
    pub rx: Receiver<MonitorResponse>,
    pub wake_watcher: Arc<WakeWatcher>,
}

impl Stream for MonitorClient {
    type Item = Result<MonitorResponse, Status>;

    fn poll_next(
        self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let s = self.get_mut();
        let response = s.rx.try_recv();

        let poll = match response {
            Err(err) => {
                match err {
                    TryRecvError::Closed => {
                        Poll::Ready(Some(Err(Status::not_found(
                            "The monitoring server closed.",
                        ))))
                    }
                    TryRecvError::Empty => {
                        match s.wake_watcher.add_waker(ctx.waker().to_owned()) {
                            Ok(_) => Poll::Pending,
                            Err(err) => {
                                Poll::Ready(Some(Err(Status::internal(
                                    err.to_string(),
                                ))))
                            }
                        }
                    }
                    TryRecvError::Lagged(_) => {
                        Poll::Ready(Some(Err(Status::internal(
                            "The receiver lagged behind for some reason.",
                        ))))
                    }
                }
            }
            Ok(resp) => Poll::Ready(Some(Ok(resp))),
        };

        poll
    }
}
