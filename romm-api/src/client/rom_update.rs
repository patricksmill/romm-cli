use reqwest::multipart;
use reqwest::Url;

use crate::endpoints::roms::{PutRom, RomUpdateFields};
use crate::endpoints::Endpoint;
use crate::error::ApiError;
use crate::types::metadata::{RomMatchFields, RomUpdateResponse};

use super::response::{
    api_error_from_response, decode_json_response_body, read_error_response_text,
};
use super::RommClient;

/// Build `(field, value)` pairs for RomM `PUT /api/roms/{id}` multipart text fields.
pub(crate) fn rom_update_text_parts(fields: &RomUpdateFields) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if let Some(ref n) = fields.name {
        out.push(("name".into(), n.clone()));
    }
    if let Some(ref s) = fields.summary {
        out.push(("summary".into(), s.clone()));
    }
    if let Some(ref u) = fields.url_cover {
        out.push(("url_cover".into(), u.clone()));
    }
    push_match_fields(&mut out, &fields.match_fields);
    out
}

fn push_match_fields(out: &mut Vec<(String, String)>, m: &RomMatchFields) {
    push_i64(out, "igdb_id", m.igdb_id);
    push_i64(out, "moby_id", m.moby_id);
    push_i64(out, "ss_id", m.ss_id);
    push_i64(out, "launchbox_id", m.launchbox_id);
    push_i64(out, "sgdb_id", m.sgdb_id);
    push_i64(out, "ra_id", m.ra_id);
    push_i64(out, "hasheous_id", m.hasheous_id);
    push_i64(out, "tgdb_id", m.tgdb_id);
    push_i64(out, "hltb_id", m.hltb_id);
    if let Some(ref id) = m.flashpoint_id {
        out.push(("flashpoint_id".into(), id.clone()));
    }
    if let Some(ref id) = m.libretro_id {
        out.push(("libretro_id".into(), id.clone()));
    }
}

fn push_i64(out: &mut Vec<(String, String)>, key: &str, v: Option<i64>) {
    if let Some(n) = v {
        out.push((key.into(), n.to_string()));
    }
}

impl RommClient {
    /// `PUT /api/roms/{id}` with `multipart/form-data` (metadata match, edit, unmatch).
    pub async fn update_rom(&self, ep: &PutRom) -> Result<RomUpdateResponse, ApiError> {
        let path = ep.path();
        let mut url = Url::parse(&format!("{}{}", self.base_url.trim_end_matches('/'), path))
            .map_err(|e| ApiError::UnexpectedResponse(format!("invalid update_rom URL: {e}")))?;

        {
            let mut pairs = url.query_pairs_mut();
            if ep.remove_cover {
                pairs.append_pair("remove_cover", "true");
            }
            if ep.unmatch_metadata {
                pairs.append_pair("unmatch_metadata", "true");
            }
        }

        let mut form = multipart::Form::new();
        if !ep.unmatch_metadata {
            for (key, value) in rom_update_text_parts(&ep.fields) {
                form = form.text(key, value);
            }
            if let Some(ref artwork_path) = ep.artwork {
                let bytes = tokio::fs::read(artwork_path).await.map_err(|e| {
                    ApiError::Io(std::io::Error::new(
                        e.kind(),
                        format!("read artwork {}: {e}", artwork_path.display()),
                    ))
                })?;
                let fname = artwork_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("artwork.png");
                let part = multipart::Part::bytes(bytes).file_name(fname.to_string());
                form = form.part("artwork", part);
            }
        }

        let resp = self
            .http
            .put(url)
            .headers(self.build_headers()?)
            .multipart(form)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = read_error_response_text(resp).await;
            return Err(api_error_from_response(status, &body));
        }

        let bytes = resp.bytes().await?;
        let value = decode_json_response_body(&bytes);
        serde_json::from_value(value).map_err(|e| {
            ApiError::UnexpectedResponse(format!("failed to decode update_rom response: {e}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endpoints::roms::RomUpdateFields;

    #[test]
    fn form_parts_include_only_set_fields() {
        let fields = RomUpdateFields {
            name: Some("Foo".into()),
            summary: None,
            url_cover: None,
            match_fields: RomMatchFields {
                igdb_id: Some(99),
                ..Default::default()
            },
        };
        let parts = rom_update_text_parts(&fields);
        assert!(parts.contains(&("name".into(), "Foo".into())));
        assert!(parts.contains(&("igdb_id".into(), "99".into())));
        assert!(!parts.iter().any(|(k, _)| k == "summary"));
    }
}
