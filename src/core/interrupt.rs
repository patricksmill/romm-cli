use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use thiserror::Error;
use tokio::sync::Notify;

use crate::error::{DownloadError, RommError};

#[derive(Debug, Error)]
#[error("operation cancelled by user")]
pub struct CancelledByUser;

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

impl Default for InterruptContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn cancelled_download_error() -> DownloadError {
    DownloadError::Cancelled(CancelledByUser)
}

/// Legacy alias used during migration from `anyhow`.
pub fn cancelled_error() -> DownloadError {
    cancelled_download_error()
}

pub fn is_cancelled_download(err: &DownloadError) -> bool {
    matches!(err, DownloadError::Cancelled(_))
}

pub fn is_cancelled_error(err: &RommError) -> bool {
    err.is_cancelled()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancelled_error_is_classified() {
        let err = RommError::from(cancelled_download_error());
        assert!(is_cancelled_error(&err));
        assert!(is_cancelled_download(&cancelled_download_error()));
    }

    #[tokio::test]
    async fn context_cancel_sets_flag() {
        let ctx = InterruptContext::new();
        assert!(!ctx.is_cancelled());
        ctx.cancel();
        assert!(ctx.is_cancelled());
    }
}
