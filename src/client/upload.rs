use anyhow::{anyhow, Result};
use reqwest::header::HeaderValue;
use reqwest::multipart;
use serde_json::Value;
use std::path::Path;
use std::time::Instant;
use tokio::io::AsyncReadExt as _;

use super::response::{decode_json_response_body, read_error_response_text, romm_api_error};
use super::{RommClient, SaveUploadOptions};

impl RommClient {
    /// Uploads a ROM file to the server using the RomM chunked upload API.
    pub async fn upload_rom<F>(
        &self,
        platform_id: u64,
        file_path: &Path,
        mut on_progress: F,
    ) -> Result<()>
    where
        F: FnMut(u64, u64) + Send,
    {
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid filename for upload"))?;

        let metadata = tokio::fs::metadata(file_path)
            .await
            .map_err(|e| anyhow!("Failed to read file metadata {:?}: {}", file_path, e))?;
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
            reqwest::header::HeaderValue::from_str(&platform_id.to_string())?,
        );
        start_headers.insert(
            reqwest::header::HeaderName::from_static("x-upload-filename"),
            reqwest::header::HeaderValue::from_str(filename)?,
        );
        start_headers.insert(
            reqwest::header::HeaderName::from_static("x-upload-total-size"),
            reqwest::header::HeaderValue::from_str(&total_size.to_string())?,
        );
        start_headers.insert(
            reqwest::header::HeaderName::from_static("x-upload-total-chunks"),
            reqwest::header::HeaderValue::from_str(&total_chunks.to_string())?,
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
            .await
            .map_err(|e| anyhow!("upload start request error: {}", e))?;

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
            return Err(romm_api_error(status, &body));
        }

        let start_resp: Value = resp
            .json()
            .await
            .map_err(|e| anyhow!("failed to parse start upload response: {}", e))?;
        let upload_id = start_resp
            .get("upload_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing upload_id in start response: {}", start_resp))?
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
                reqwest::header::HeaderValue::from_str(&chunk_index.to_string())?,
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
                .await
                .map_err(|e| anyhow!("chunk upload request error: {}", e))?;

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

                return Err(anyhow!("Failed to upload chunk {}: {}", chunk_index, body));
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
            .await
            .map_err(|e| anyhow!("upload complete request error: {}", e))?;

        if !complete_resp.status().is_success() {
            let body = read_error_response_text(complete_resp).await;
            return Err(anyhow!("Failed to complete upload: {}", body));
        }

        Ok(())
    }

    /// Uploads a game save file to the server.
    pub async fn upload_save_file(
        &self,
        rom_id: u64,
        emulator: Option<&str>,
        file_path: &Path,
    ) -> Result<Value> {
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
    ) -> Result<Value> {
        let url = format!("{}/api/saves", self.base_url.trim_end_matches('/'));
        let bytes = tokio::fs::read(file_path)
            .await
            .map_err(|e| anyhow!("read {}: {e}", file_path.display()))?;
        let fname = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("upload path must have a unicode filename"))?;
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
            .await
            .map_err(|e| anyhow!("save upload request: {e}"))?;
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
            return Err(romm_api_error(status, &body));
        }
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| anyhow!("read save upload body: {e}"))?;
        Ok(decode_json_response_body(&bytes))
    }

    /// Downloads save content from `GET /api/saves/{id}/content`.
    pub async fn download_save_content(
        &self,
        save_id: u64,
        device_id: Option<&str>,
        session_id: Option<u64>,
    ) -> Result<Vec<u8>> {
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
    ) -> Result<Value> {
        let url = format!("{}/api/states", self.base_url.trim_end_matches('/'));
        let bytes = tokio::fs::read(file_path)
            .await
            .map_err(|e| anyhow!("read {}: {e}", file_path.display()))?;
        let fname = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("upload path must have a unicode filename"))?;
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
            .await
            .map_err(|e| anyhow!("state upload request: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            let body = read_error_response_text(resp).await;
            return Err(romm_api_error(status, &body));
        }
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| anyhow!("read state upload body: {e}"))?;
        Ok(decode_json_response_body(&bytes))
    }

    /// `POST /api/screenshots` with multipart field `screenshotFile`.
    pub async fn upload_screenshot_file(&self, rom_id: u64, file_path: &Path) -> Result<Value> {
        let url = format!("{}/api/screenshots", self.base_url.trim_end_matches('/'));
        let bytes = tokio::fs::read(file_path)
            .await
            .map_err(|e| anyhow!("read {}: {e}", file_path.display()))?;
        let fname = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("upload path must have a unicode filename"))?;
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
            .await
            .map_err(|e| anyhow!("screenshot upload: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            let body = read_error_response_text(resp).await;
            return Err(romm_api_error(status, &body));
        }
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| anyhow!("read screenshot body: {e}"))?;
        Ok(decode_json_response_body(&bytes))
    }

    /// `POST /api/firmware?platform_id=` with multipart `files`.
    pub async fn upload_firmware_file(&self, platform_id: u64, file_path: &Path) -> Result<Value> {
        let url = format!("{}/api/firmware", self.base_url.trim_end_matches('/'));
        let bytes = tokio::fs::read(file_path)
            .await
            .map_err(|e| anyhow!("read {}: {e}", file_path.display()))?;
        let fname = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("upload path must have a unicode filename"))?;
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
            .await
            .map_err(|e| anyhow!("firmware upload: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            let body = read_error_response_text(resp).await;
            return Err(romm_api_error(status, &body));
        }
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| anyhow!("read firmware body: {e}"))?;
        Ok(decode_json_response_body(&bytes))
    }

    /// `POST /api/roms/{id}/manuals` — raw file body with `x-upload-filename` header.
    pub async fn upload_rom_manual(&self, rom_id: u64, file_path: &Path) -> Result<Value> {
        let fname = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("manual path must have a unicode filename"))?
            .to_string();
        let url = format!(
            "{}/api/roms/{}/manuals",
            self.base_url.trim_end_matches('/'),
            rom_id
        );
        let bytes = tokio::fs::read(file_path)
            .await
            .map_err(|e| anyhow!("read {}: {e}", file_path.display()))?;
        let mut headers = self.build_headers()?;
        headers.insert(
            reqwest::header::HeaderName::from_static("x-upload-filename"),
            HeaderValue::from_str(&fname).map_err(|_| anyhow!("invalid x-upload-filename"))?,
        );
        let resp = self
            .http
            .post(&url)
            .headers(headers)
            .body(bytes)
            .send()
            .await
            .map_err(|e| anyhow!("manual upload: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            let body = read_error_response_text(resp).await;
            return Err(romm_api_error(status, &body));
        }
        let out = resp.bytes().await?;
        Ok(decode_json_response_body(&out))
    }
}
