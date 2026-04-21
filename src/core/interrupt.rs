use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Error;
use tokio::sync::Notify;

#[derive(Debug)]
pub struct CancelledByUser;

impl fmt::Display for CancelledByUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "operation cancelled by user")
    }
}

impl std::error::Error for CancelledByUser {}

#[derive(Clone, Debug)]
pub struct InterruptContext {
    cancelled: Arc<AtomicBool>,
    notify: Arc<Notify>,
}

impl InterruptContext {
    pub fn new() -> Self {
        let this = Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            notify: Arc::new(Notify::new()),
        };
        let watcher = this.clone();
        tokio::spawn(async move {
            if tokio::signal::ctrl_c().await.is_ok() {
                watcher.cancel();
            }
        });
        this
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub async fn cancelled(&self) {
        if self.is_cancelled() {
            return;
        }
        self.notify.notified().await;
    }
}

pub fn cancelled_error() -> Error {
    Error::new(CancelledByUser)
}

pub fn is_cancelled_error(err: &Error) -> bool {
    err.downcast_ref::<CancelledByUser>().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancelled_error_is_classified() {
        let err = cancelled_error();
        assert!(is_cancelled_error(&err));
    }

    #[tokio::test]
    async fn context_cancel_sets_flag() {
        let ctx = InterruptContext::new();
        assert!(!ctx.is_cancelled());
        ctx.cancel();
        assert!(ctx.is_cancelled());
    }
}

