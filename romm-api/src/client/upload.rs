use reqwest::header::HeaderValue;
use reqwest::multipart;
use serde_json::Value;
use std::path::Path;
use std::time::Instant;
use tokio::io::AsyncReadExt as _;

use crate::error::ApiError;

use super::response::{
    api_error_from_response, decode_json_response_body, read_error_response_text,
};
use super::{RommClient, SaveUploadOptions};

fn header_value(s: &str) -> Result<HeaderValue, ApiError> {
    HeaderValue::from_str(s).map_err(|_| ApiError::InvalidHeader(s.to_string()))
}

impl RommClient {
    /// Uploads a ROM file to the server using the RomM chunked upload API.
    pub async fn upload_rom<F>(
        &self,
        platform_id: u64,
        file_path: &Path,
        mut on_progress: F,
    ) -> Result<(), ApiError>
    where
        F: FnMut(u64, u64) + Send,
    {
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| ApiError::UnexpectedResponse("Invalid filename for upload".into()))?;

        let metadata = tokio::fs::metadata(file_path).await.map_err(|e| {
            ApiError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read file metadata {file_path:?}: {e}"),
            ))
        })?;
        let total_size = metadata.len();

        let chunk_size: u64 = 2 * 1024 * 1024;
        let total_chunks = if total_size == 0 {
            1
        } else {
            total_size.div_ceil(chunk_size)
        };

        let mut start_headers = self.build_headers()?;
        start_headers.insert(
            reqwest::header::HeaderName::from_static("x-upload-platform"),
            header_value(&platform_id.to_string())?,
        );
        start_headers.insert(
            reqwest::header::HeaderName::from_static("x-upload-filename"),
            header_value(filename)?,
        );
        start_headers.insert(
            reqwest::header::HeaderName::from_static("x-upload-total-size"),
            header_value(&total_size.to_string())?,
        );
        start_headers.insert(
            reqwest::header::HeaderName::from_static("x-upload-total-chunks"),
            header_value(&total_chunks.to_string())?,
        );

        let start_url = format!(
            "{}/api/roms/upload/start",
            self.base_url.trim_end_matches('/')
        );

        let t0 = Instant::now();
        let resp = self
            .http
            .post(&start_url)
            .headers(start_headers)
            .send()
            .await?;

        let status = resp.status();
        if self.verbose {
            tracing::info!(
                "[romm-cli] POST /api/roms/upload/start -> {} ({}ms)",
                status.as_u16(),
                t0.elapsed().as_millis()
            );
        }

        if !status.is_success() {
            let body = read_error_response_text(resp).await;
            return Err(api_error_from_response(status, &body));
        }

        let start_resp: Value = resp.json().await?;
        let upload_id = start_resp
            .get("upload_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ApiError::UnexpectedResponse(format!(
                    "Missing upload_id in start response: {start_resp}"
                ))
            })?
            .to_string();

        let mut file = tokio::fs::File::open(file_path).await?;
        let mut uploaded_bytes = 0;
        let mut buffer = vec![0u8; chunk_size as usize];

        for chunk_index in 0..total_chunks {
            let mut chunk_bytes = 0;
            let mut chunk_data = Vec::new();

            while chunk_bytes < chunk_size as usize {
                let n = file.read(&mut buffer[..]).await?;
                if n == 0 {
                    break;
                }
                chunk_data.extend_from_slice(&buffer[..n]);
                chunk_bytes += n;
            }

            let mut chunk_headers = self.build_headers()?;
            chunk_headers.insert(
                reqwest::header::HeaderName::from_static("x-chunk-index"),
                header_value(&chunk_index.to_string())?,
            );

            let chunk_url = format!(
                "{}/api/roms/upload/{}",
                self.base_url.trim_end_matches('/'),
                upload_id
            );

            let chunk_resp = self
                .http
                .put(&chunk_url)
                .headers(chunk_headers)
                .body(chunk_data.clone())
                .send()
                .await?;

            if !chunk_resp.status().is_success() {
                let body = read_error_response_text(chunk_resp).await;
                let cancel_url = format!(
                    "{}/api/roms/upload/{}/cancel",
                    self.base_url.trim_end_matches('/'),
                    upload_id
                );
                let _ = self
                    .http
                    .post(&cancel_url)
                    .headers(self.build_headers()?)
                    .send()
                    .await;

                return Err(ApiError::UnexpectedResponse(format!(
                    "Failed to upload chunk {chunk_index}: {body}"
                )));
            }

            uploaded_bytes += chunk_data.len() as u64;
            on_progress(uploaded_bytes, total_size);
        }

        let complete_url = format!(
            "{}/api/roms/upload/{}/complete",
            self.base_url.trim_end_matches('/'),
            upload_id
        );
        let complete_resp = self
            .http
            .post(&complete_url)
            .headers(self.build_headers()?)
            .send()
            .await?;

        if !complete_resp.status().is_success() {
            let body = read_error_response_text(complete_resp).await;
            return Err(ApiError::UnexpectedResponse(format!(
                "Failed to complete upload: {body}"
            )));
        }

        Ok(())
    }

    /// Uploads a game save file to the server.
    pub async fn upload_save_file(
        &self,
        rom_id: u64,
        emulator: Option<&str>,
        file_path: &Path,
    ) -> Result<Value, ApiError> {
        let options = SaveUploadOptions {
            emulator,
            ..Default::default()
        };
        self.upload_save_file_with_options(rom_id, file_path, &options)
            .await
    }

    /// Uploads a game save file with sync-specific options.
    pub async fn upload_save_file_with_options(
        &self,
        rom_id: u64,
        file_path: &Path,
        options: &SaveUploadOptions<'_>,
    ) -> Result<Value, ApiError> {
        let url = format!("{}/api/saves", self.base_url.trim_end_matches('/'));
        let bytes = tokio::fs::read(file_path).await.map_err(|e| {
            ApiError::Io(std::io::Error::new(
                e.kind(),
                format!("read {}: {e}", file_path.display()),
            ))
        })?;
        let fname = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                ApiError::UnexpectedResponse("upload path must have a unicode filename".into())
            })?;
        let part = multipart::Part::bytes(bytes).file_name(fname.to_string());
        let form = multipart::Form::new().part("saveFile", part);
        let mut query: Vec<(String, String)> = vec![("rom_id".into(), rom_id.to_string())];
        if let Some(em) = options.emulator {
            if !em.is_empty() {
                query.push(("emulator".into(), em.to_string()));
            }
        }
        if let Some(slot) = options.slot {
            if !slot.is_empty() {
                query.push(("slot".into(), slot.to_string()));
            }
        }
        if let Some(device_id) = options.device_id {
            if !device_id.is_empty() {
                query.push(("device_id".into(), device_id.to_string()));
            }
        }
        if let Some(session_id) = options.session_id {
            query.push(("session_id".into(), session_id.to_string()));
        }
        if options.overwrite {
            query.push(("overwrite".into(), "true".into()));
        }
        let query_refs: Vec<(&str, &str)> = query
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let headers = self.build_headers()?;
        let t0 = Instant::now();
        let resp = self
            .http
            .post(&url)
            .headers(headers)
            .query(&query_refs)
            .multipart(form)
            .send()
            .await?;
        let status = resp.status();
        if self.verbose {
            tracing::info!(
                "[romm-cli] POST /api/saves rom_id={rom_id} -> {} ({}ms)",
                status.as_u16(),
                t0.elapsed().as_millis()
            );
        }
        if !status.is_success() {
            let body = read_error_response_text(resp).await;
            return Err(api_error_from_response(status, &body));
        }
        let bytes = resp.bytes().await?;
        Ok(decode_json_response_body(&bytes))
    }

    /// Downloads save content from `GET /api/saves/{id}/content`.
    pub async fn download_save_content(
        &self,
        save_id: u64,
        device_id: Option<&str>,
        session_id: Option<u64>,
    ) -> Result<Vec<u8>, ApiError> {
        let path = format!("/api/saves/{save_id}/content");
        let mut query = Vec::new();
        if let Some(device_id) = device_id {
            if !device_id.is_empty() {
                query.push(("device_id".to_string(), device_id.to_string()));
            }
        }
        if let Some(session_id) = session_id {
            query.push(("session_id".to_string(), session_id.to_string()));
        }
        self.get_bytes(&path, &query).await
    }

    /// `POST /api/states` with multipart field `stateFile`.
    pub async fn upload_state_file(
        &self,
        rom_id: u64,
        emulator: Option<&str>,
        file_path: &Path,
    ) -> Result<Value, ApiError> {
        let url = format!("{}/api/states", self.base_url.trim_end_matches('/'));
        let bytes = tokio::fs::read(file_path).await.map_err(|e| {
            ApiError::Io(std::io::Error::new(
                e.kind(),
                format!("read {}: {e}", file_path.display()),
            ))
        })?;
        let fname = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                ApiError::UnexpectedResponse("upload path must have a unicode filename".into())
            })?;
        let part = multipart::Part::bytes(bytes).file_name(fname.to_string());
        let form = multipart::Form::new().part("stateFile", part);
        let mut query: Vec<(String, String)> = vec![("rom_id".into(), rom_id.to_string())];
        if let Some(em) = emulator {
            if !em.is_empty() {
                query.push(("emulator".into(), em.to_string()));
            }
        }
        let query_refs: Vec<(&str, &str)> = query
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let headers = self.build_headers()?;
        let resp = self
            .http
            .post(&url)
            .headers(headers)
            .query(&query_refs)
            .multipart(form)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = read_error_response_text(resp).await;
            return Err(api_error_from_response(status, &body));
        }
        let bytes = resp.bytes().await?;
        Ok(decode_json_response_body(&bytes))
    }

    /// `POST /api/screenshots` with multipart field `screenshotFile`.
    pub async fn upload_screenshot_file(
        &self,
        rom_id: u64,
        file_path: &Path,
    ) -> Result<Value, ApiError> {
        let url = format!("{}/api/screenshots", self.base_url.trim_end_matches('/'));
        let bytes = tokio::fs::read(file_path).await.map_err(|e| {
            ApiError::Io(std::io::Error::new(
                e.kind(),
                format!("read {}: {e}", file_path.display()),
            ))
        })?;
        let fname = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                ApiError::UnexpectedResponse("upload path must have a unicode filename".into())
            })?;
        let part = multipart::Part::bytes(bytes).file_name(fname.to_string());
        let form = multipart::Form::new().part("screenshotFile", part);
        let headers = self.build_headers()?;
        let resp = self
            .http
            .post(&url)
            .headers(headers)
            .query(&[("rom_id", rom_id.to_string().as_str())])
            .multipart(form)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = read_error_response_text(resp).await;
            return Err(api_error_from_response(status, &body));
        }
        let bytes = resp.bytes().await?;
        Ok(decode_json_response_body(&bytes))
    }

    /// `POST /api/firmware?platform_id=` with multipart `files`.
    pub async fn upload_firmware_file(
        &self,
        platform_id: u64,
        file_path: &Path,
    ) -> Result<Value, ApiError> {
        let url = format!("{}/api/firmware", self.base_url.trim_end_matches('/'));
        let bytes = tokio::fs::read(file_path).await.map_err(|e| {
            ApiError::Io(std::io::Error::new(
                e.kind(),
                format!("read {}: {e}", file_path.display()),
            ))
        })?;
        let fname = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                ApiError::UnexpectedResponse("upload path must have a unicode filename".into())
            })?;
        let part = multipart::Part::bytes(bytes).file_name(fname.to_string());
        let form = multipart::Form::new().part("files", part);
        let headers = self.build_headers()?;
        let resp = self
            .http
            .post(&url)
            .headers(headers)
            .query(&[("platform_id", platform_id.to_string())])
            .multipart(form)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = read_error_response_text(resp).await;
            return Err(api_error_from_response(status, &body));
        }
        let bytes = resp.bytes().await?;
        Ok(decode_json_response_body(&bytes))
    }

    /// `POST /api/roms/{id}/manuals` — raw file body with `x-upload-filename` header.
    pub async fn upload_rom_manual(
        &self,
        rom_id: u64,
        file_path: &Path,
    ) -> Result<Value, ApiError> {
        let fname = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                ApiError::UnexpectedResponse("manual path must have a unicode filename".into())
            })?
            .to_string();
        let url = format!(
            "{}/api/roms/{}/manuals",
            self.base_url.trim_end_matches('/'),
            rom_id
        );
        let bytes = tokio::fs::read(file_path).await.map_err(|e| {
            ApiError::Io(std::io::Error::new(
                e.kind(),
                format!("read {}: {e}", file_path.display()),
            ))
        })?;
        let mut headers = self.build_headers()?;
        headers.insert(
            reqwest::header::HeaderName::from_static("x-upload-filename"),
            header_value(&fname)?,
        );
        let resp = self
            .http
            .post(&url)
            .headers(headers)
            .body(bytes)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = read_error_response_text(resp).await;
            return Err(api_error_from_response(status, &body));
        }
        let out = resp.bytes().await?;
        Ok(decode_json_response_body(&out))
    }
}
