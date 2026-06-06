use reqwest::Url;
use std::path::Path;
use std::time::Instant;
use tokio::io::AsyncWriteExt as _;

use crate::config::normalize_romm_origin;
use crate::core::interrupt::CancelledByUser;
use crate::error::{ApiError, DownloadError};

use super::response::{api_error_from_response, read_error_response_text};
use super::RommClient;

impl RommClient {
    /// Downloads a ROM (or multiple ROMs as a zip) to the specified path.
    pub async fn download_rom<F>(
        &self,
        rom_id: u64,
        save_path: &Path,
        mut on_progress: F,
    ) -> Result<(), DownloadError>
    where
        F: FnMut(u64, u64) + Send,
    {
        self.download_rom_with_cancel(rom_id, save_path, |_, _| false, &mut on_progress)
            .await
    }

    pub async fn download_rom_with_cancel<F, C>(
        &self,
        rom_id: u64,
        save_path: &Path,
        is_cancelled: C,
        on_progress: &mut F,
    ) -> Result<(), DownloadError>
    where
        F: FnMut(u64, u64) + Send,
        C: FnMut(u64, u64) -> bool + Send,
    {
        let filename = filename_hint(save_path);
        let query = vec![
            ("rom_ids".to_string(), rom_id.to_string()),
            ("filename".to_string(), filename),
        ];
        self.download_url_with_query_with_cancel(
            "/api/roms/download",
            &query,
            save_path,
            is_cancelled,
            on_progress,
        )
        .await
    }

    /// Downloads an arbitrary URL to `save_path`, supporting auth headers and resume.
    pub async fn download_url_with_cancel<F, C>(
        &self,
        url: &str,
        save_path: &Path,
        is_cancelled: C,
        on_progress: &mut F,
    ) -> Result<(), DownloadError>
    where
        F: FnMut(u64, u64) + Send,
        C: FnMut(u64, u64) -> bool + Send,
    {
        self.download_url_with_query_with_cancel(url, &[], save_path, is_cancelled, on_progress)
            .await
    }

    /// Downloads an arbitrary URL and query to `save_path`, supporting auth headers and resume.
    pub async fn download_url_with_query_with_cancel<F, C>(
        &self,
        url: &str,
        query: &[(String, String)],
        save_path: &Path,
        mut is_cancelled: C,
        on_progress: &mut F,
    ) -> Result<(), DownloadError>
    where
        F: FnMut(u64, u64) + Send,
        C: FnMut(u64, u64) -> bool + Send,
    {
        let url = self.resolve_download_url(url)?;
        let filename = filename_hint(save_path);
        let mut headers = self.build_headers()?;

        let existing_len = tokio::fs::metadata(save_path)
            .await
            .map(|m| m.len())
            .unwrap_or(0);

        if existing_len > 0 {
            let range = format!("bytes={existing_len}-");
            if let Ok(v) = reqwest::header::HeaderValue::from_str(&range) {
                headers.insert(reqwest::header::RANGE, v);
            }
        }

        if let Some(parent) = save_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                DownloadError::IoContext {
                    context: format!("create download parent dir {parent:?}"),
                    source: e,
                }
            })?;
        }

        let t0 = Instant::now();
        let mut resp = self.http.get(&url).headers(headers).query(query).send().await?;

        let status = resp.status();
        if self.verbose {
            tracing::info!(
                "[romm-cli] GET {} filename={:?} -> {} ({}ms)",
                url,
                filename,
                status.as_u16(),
                t0.elapsed().as_millis()
            );
        }
        if !status.is_success() {
            let body = read_error_response_text(resp).await;
            return Err(DownloadError::Api(api_error_from_response(status, &body)));
        }

        let (mut received, total, mut file) = if status == reqwest::StatusCode::PARTIAL_CONTENT {
            let remaining = resp.content_length().unwrap_or(0);
            let total = existing_len + remaining;
            let file = tokio::fs::OpenOptions::new()
                .append(true)
                .open(save_path)
                .await
                .map_err(|e| DownloadError::IoContext {
                    context: format!("open file for append {save_path:?}"),
                    source: e,
                })?;
            (existing_len, total, file)
        } else {
            let total = resp.content_length().unwrap_or(0);
            let file = tokio::fs::File::create(save_path).await.map_err(|e| {
                DownloadError::IoContext {
                    context: format!("create file {save_path:?}"),
                    source: e,
                }
            })?;
            (0u64, total, file)
        };

        if is_cancelled(received, total) {
            return Err(DownloadError::Cancelled(CancelledByUser));
        }

        while let Some(chunk) = resp.chunk().await? {
            if is_cancelled(received, total) {
                return Err(DownloadError::Cancelled(CancelledByUser));
            }
            file.write_all(&chunk).await.map_err(|e| DownloadError::IoContext {
                context: format!("write chunk {save_path:?}"),
                source: e,
            })?;
            received += chunk.len() as u64;
            on_progress(received, total);
        }

        Ok(())
    }

    fn resolve_download_url(&self, url: &str) -> Result<String, DownloadError> {
        let trimmed = url.trim();
        if trimmed.is_empty() {
            return Err(DownloadError::Api(ApiError::UnexpectedResponse(
                "download URL cannot be empty".into(),
            )));
        }
        if let Ok(parsed) = Url::parse(trimmed) {
            return Ok(parsed.to_string());
        }

        let base = Url::parse(&normalize_romm_origin(&self.base_url)).map_err(|e| {
            DownloadError::Api(ApiError::UnexpectedResponse(format!(
                "invalid RomM base URL: {e}"
            )))
        })?;
        let joined = base.join(trimmed).map_err(|e| {
            DownloadError::Api(ApiError::UnexpectedResponse(format!(
                "could not resolve download URL {trimmed:?}: {e}"
            )))
        })?;
        Ok(joined.to_string())
    }
}

fn filename_hint(save_path: &Path) -> String {
    save_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download.bin")
        .to_string()
}
