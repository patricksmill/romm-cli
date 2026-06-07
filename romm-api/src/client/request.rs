use reqwest::header::HeaderMap;
use reqwest::Method;
use serde_json::Value;
use std::time::Instant;

use crate::endpoints::Endpoint;
use crate::error::ApiError;

use super::response::{
    api_error_from_response, decode_json_response_body, read_error_response_text,
};
use super::RommClient;

impl RommClient {
    /// Executes a typed [`Endpoint`] and returns its deserialized output.
    pub async fn call<E>(&self, ep: &E) -> Result<E::Output, ApiError>
    where
        E: Endpoint,
        E::Output: serde::de::DeserializeOwned,
    {
        let method = ep.method();
        let path = ep.path();
        let query = ep.query();
        let body = ep.body();

        let value = self.request_json(method, &path, &query, body).await?;
        let output = serde_json::from_value(value).map_err(|e| {
            ApiError::UnexpectedResponse(format!(
                "failed to decode response for {method} {path}: {e}"
            ))
        })?;

        Ok(output)
    }

    /// Low-level helper that issues an HTTP request and returns a raw JSON [`Value`].
    pub async fn request_json(
        &self,
        method: &str,
        path: &str,
        query: &[(String, String)],
        body: Option<Value>,
    ) -> Result<Value, ApiError> {
        self.request_json_with_headers(method, path, query, body, self.build_headers()?)
            .await
    }

    pub async fn request_json_unauthenticated(
        &self,
        method: &str,
        path: &str,
        query: &[(String, String)],
        body: Option<Value>,
    ) -> Result<Value, ApiError> {
        self.request_json_with_headers(method, path, query, body, HeaderMap::new())
            .await
    }

    async fn request_json_with_headers(
        &self,
        method: &str,
        path: &str,
        query: &[(String, String)],
        body: Option<Value>,
        headers: HeaderMap,
    ) -> Result<Value, ApiError> {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );

        let http_method = Method::from_bytes(method.as_bytes())
            .map_err(|_| ApiError::InvalidMethod(method.to_string()))?;

        let query_refs: Vec<(&str, &str)> = query
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        let mut req = self
            .http
            .request(http_method, &url)
            .headers(headers)
            .query(&query_refs);

        if let Some(body) = body {
            req = req.json(&body);
        }

        let t0 = Instant::now();
        let resp = req.send().await?;

        let status = resp.status();
        if self.verbose {
            let keys: Vec<&str> = query.iter().map(|(k, _)| k.as_str()).collect();
            tracing::info!(
                "[romm-cli] {} {} query_keys={:?} -> {} ({}ms)",
                method,
                path,
                keys,
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

    /// Authenticated GET returning raw bytes.
    pub async fn get_bytes(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<Vec<u8>, ApiError> {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let headers = self.build_headers()?;
        let query_refs: Vec<(&str, &str)> = query
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let resp = self
            .http
            .get(&url)
            .headers(headers)
            .query(&query_refs)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = read_error_response_text(resp).await;
            return Err(api_error_from_response(status, &body));
        }
        Ok(resp.bytes().await?.to_vec())
    }

    /// POST returning raw bytes.
    pub async fn post_bytes(
        &self,
        path: &str,
        query: &[(String, String)],
        json_body: Option<Value>,
    ) -> Result<Vec<u8>, ApiError> {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let headers = self.build_headers()?;
        let query_refs: Vec<(&str, &str)> = query
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let mut req = self.http.post(&url).headers(headers).query(&query_refs);
        if let Some(b) = json_body {
            req = req.json(&b);
        }
        let resp = req.send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body = read_error_response_text(resp).await;
            return Err(api_error_from_response(status, &body));
        }
        Ok(resp.bytes().await?.to_vec())
    }
}
