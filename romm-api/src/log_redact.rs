//! Redaction helpers for tracing and debug output (Gap 6: secrets never in logs).

use crate::error::ApiError;

/// Strip URL userinfo and query strings before verbose HTTP logging.
pub fn redact_url_for_log(url: &str) -> String {
    let (base, had_query) = match url.split_once('?') {
        Some((b, _)) => (b, true),
        None => (url, false),
    };

    let redacted_base = if let Some(at_idx) = base.find('@') {
        if let Some(scheme_end) = base.find("://") {
            format!(
                "{}<redacted>@{}",
                &base[..scheme_end + 3],
                &base[at_idx + 1..]
            )
        } else {
            format!("<redacted>@{}", &base[at_idx + 1..])
        }
    } else {
        base.to_string()
    };

    if had_query {
        format!("{redacted_base}?<redacted>")
    } else {
        redacted_base
    }
}

/// Format a [`RommError`] for logs, redacting API response bodies.
pub fn redact_romm_error_for_log(err: &crate::error::RommError) -> String {
    match err {
        crate::error::RommError::Api(api) => api.redacted_for_log(),
        other => other.to_string(),
    }
}

/// Format an `anyhow::Error` for logs, redacting [`ApiError`] response bodies in the chain.
pub fn redact_anyhow_for_log(err: &anyhow::Error) -> String {
    if let Some(api) = err.downcast_ref::<ApiError>() {
        return api.redacted_for_log();
    }
    if let Some(romm) = err.downcast_ref::<crate::error::RommError>() {
        return redact_romm_error_for_log(romm);
    }
    for cause in err.chain() {
        if let Some(api) = cause.downcast_ref::<ApiError>() {
            return api.redacted_for_log();
        }
        if let Some(romm) = cause.downcast_ref::<crate::error::RommError>() {
            return redact_romm_error_for_log(romm);
        }
    }
    err.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ApiError;
    use std::io::Write;
    use std::sync::{Arc, Mutex};
    struct CaptureWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for CaptureWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn capture_logs<F: FnOnce()>(f: F) -> String {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let writer_buf = buf.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(move || CaptureWriter(writer_buf.clone()))
            .with_ansi(false)
            .without_time()
            .finish();
        let _guard = tracing::subscriber::set_default(subscriber);
        f();
        let output = buf.lock().unwrap().clone();
        String::from_utf8_lossy(&output).into_owned()
    }

    #[test]
    fn redact_url_strips_userinfo() {
        let url = "https://user:secret@romm.example/api/roms";
        let out = redact_url_for_log(url);
        assert!(!out.contains("secret"));
        assert!(!out.contains("user"));
        assert!(out.contains("romm.example/api/roms"));
    }

    #[test]
    fn redact_url_strips_query_string() {
        let url = "https://romm.example/api?token=abc123";
        let out = redact_url_for_log(url);
        assert!(!out.contains("abc123"));
        assert!(out.ends_with("?<redacted>"));
        assert!(out.contains("romm.example/api"));
    }

    #[test]
    fn redact_url_leaves_clean_url_unchanged() {
        let url = "https://romm.example/api/roms";
        assert_eq!(redact_url_for_log(url), url);
    }

    #[test]
    fn api_error_redacted_for_log_omits_body() {
        let err = ApiError::Unauthorized {
            body: "token=abc123".to_string(),
        };
        let out = err.redacted_for_log();
        assert!(!out.contains("abc123"));
        assert!(out.contains("401"));
    }

    #[test]
    fn tracing_verbose_url_does_not_leak_credentials() {
        let url = redact_url_for_log("https://admin:sekrit@host.example/dl");
        let output = capture_logs(|| {
            tracing::info!("[romm-cli] GET {} -> 200", url);
        });
        assert!(!output.contains("sekrit"));
        assert!(!output.contains("admin"));
    }

    #[test]
    fn tracing_api_error_warn_does_not_leak_body() {
        let err = ApiError::Unauthorized {
            body: "bearer leaked-token-value".to_string(),
        };
        let msg = err.redacted_for_log();
        let output = capture_logs(|| {
            tracing::warn!("preflight failed: {msg}");
        });
        assert!(!output.contains("leaked-token-value"));
    }

    #[test]
    fn redact_anyhow_handles_api_error_in_chain() {
        let api = ApiError::ClientError {
            status: 400,
            body: "secret-body".to_string(),
        };
        let err = anyhow::Error::from(api);
        let out = redact_anyhow_for_log(&err);
        assert!(!out.contains("secret-body"));
        assert!(out.contains("400"));
    }
}
