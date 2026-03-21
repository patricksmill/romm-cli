//! Parse a subset of OpenAPI 3.x JSON into a flat endpoint list for the TUI expert browser.
//!
//! Inline `parameters` only; `$ref` on parameters is not resolved.

use anyhow::{anyhow, Result};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use serde_json::Value;

/// Percent-encode set for OpenAPI path parameter values (conservative).
const PATH_PARAM_ENCODE: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'?')
    .add(b'{')
    .add(b'}')
    .add(b'/')
    .add(b'%');

/// Replace `{name}` segments in an OpenAPI path template using percent-encoded values.
/// Returns an error if any `{...}` placeholder remains after substitution.
pub fn resolve_path_template(template: &str, values: &[(String, String)]) -> Result<String> {
    let mut out = template.to_string();
    for (name, raw) in values {
        let token = format!("{{{}}}", name);
        if out.contains(&token) {
            let encoded = utf8_percent_encode(raw, PATH_PARAM_ENCODE).to_string();
            out = out.replace(&token, &encoded);
        }
    }
    if out.contains('{') {
        return Err(anyhow!(
            "unresolved path placeholders in {:?} (fill all path parameters)",
            template
        ));
    }
    Ok(out)
}

/// Lowercase OpenAPI path-item operation keys we treat as HTTP methods.
const OPENAPI_OPERATION_METHODS: &[&str] = &[
    "get", "post", "put", "delete", "patch", "options", "head", "trace",
];

/// Returns true if `key` is an OpenAPI operation method (e.g. `get`, `post`), not `parameters` or `summary`.
pub fn is_openapi_operation_method(key: &str) -> bool {
    OPENAPI_OPERATION_METHODS
        .iter()
        .any(|m| m.eq_ignore_ascii_case(key))
}

#[derive(Debug, Clone)]
pub struct ApiParameter {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub default: Option<String>,
    #[allow(dead_code)]
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApiEndpoint {
    pub method: String,
    pub path: String,
    pub summary: Option<String>,
    #[allow(dead_code)]
    pub description: Option<String>,
    pub query_params: Vec<ApiParameter>,
    pub path_params: Vec<ApiParameter>,
    pub has_body: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EndpointRegistry {
    pub endpoints: Vec<ApiEndpoint>,
}

/// Split OpenAPI `parameters` array into query vs path params. Skips `header`, `cookie`, etc.
fn parse_parameters_array(params: &[Value]) -> (Vec<ApiParameter>, Vec<ApiParameter>) {
    let mut query_params = Vec::new();
    let mut path_params = Vec::new();

    for param in params {
        let Some(param_obj) = param.as_object() else {
            continue;
        };
        // $ref not resolved — skip
        if param_obj.contains_key("$ref") {
            continue;
        }

        let name = param_obj
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let param_in = param_obj
            .get("in")
            .and_then(|v| v.as_str())
            .unwrap_or("query");

        let required = param_obj
            .get("required")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let schema = param_obj.get("schema");
        let param_type = schema
            .and_then(|s| s.get("type"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "string".to_string());

        let default = schema.and_then(|s| s.get("default")).and_then(|v| {
            if v.is_string() {
                v.as_str().map(|s| s.to_string())
            } else {
                Some(v.to_string())
            }
        });

        let description = param_obj
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let api_param = ApiParameter {
            name,
            param_type,
            required,
            default,
            description,
        };

        match param_in {
            "query" => query_params.push(api_param),
            "path" => path_params.push(api_param),
            _ => {}
        }
    }

    (query_params, path_params)
}

/// Path-item parameters first; operation parameters with the same name replace query/path entries respectively.
fn merge_parameter_lists(
    path_query: Vec<ApiParameter>,
    path_path: Vec<ApiParameter>,
    op_query: Vec<ApiParameter>,
    op_path: Vec<ApiParameter>,
) -> (Vec<ApiParameter>, Vec<ApiParameter>) {
    let mut query = path_query;
    let mut path = path_path;

    for p in op_query {
        query.retain(|x| x.name != p.name);
        query.push(p);
    }
    for p in op_path {
        path.retain(|x| x.name != p.name);
        path.push(p);
    }

    (query, path)
}

impl EndpointRegistry {
    pub fn from_openapi_json(json_str: &str) -> Result<Self> {
        let value: Value = serde_json::from_str(json_str)
            .map_err(|e| anyhow!("Failed to parse OpenAPI JSON: {}", e))?;

        let paths = value
            .get("paths")
            .and_then(|v| v.as_object())
            .ok_or_else(|| anyhow!("OpenAPI JSON missing 'paths' object"))?;

        let mut endpoints = Vec::new();

        for (path, path_item) in paths {
            let path_item = path_item
                .as_object()
                .ok_or_else(|| anyhow!("Invalid path definition for {}", path))?;

            let path_level = path_item
                .get("parameters")
                .and_then(|v| v.as_array())
                .map(|a| a.as_slice())
                .unwrap_or(&[]);
            let (path_q, path_p) = parse_parameters_array(path_level);

            for (method_key, operation) in path_item {
                if !is_openapi_operation_method(method_key) {
                    continue;
                }

                let operation = operation
                    .as_object()
                    .ok_or_else(|| anyhow!("Invalid operation for {} {}", method_key, path))?;

                let summary = operation
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let description = operation
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let tags = operation
                    .get("tags")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_default();

                let op_level = operation
                    .get("parameters")
                    .and_then(|v| v.as_array())
                    .map(|a| a.as_slice())
                    .unwrap_or(&[]);
                let (op_q, op_p) = parse_parameters_array(op_level);
                let (query_params, path_params) =
                    merge_parameter_lists(path_q.clone(), path_p.clone(), op_q, op_p);

                let has_body = operation.get("requestBody").is_some();

                endpoints.push(ApiEndpoint {
                    method: method_key.to_uppercase(),
                    path: path.clone(),
                    summary,
                    description,
                    query_params,
                    path_params,
                    has_body,
                    tags,
                });
            }
        }

        endpoints.sort_by(|a, b| a.path.cmp(&b.path).then_with(|| a.method.cmp(&b.method)));

        Ok(EndpointRegistry { endpoints })
    }

    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read OpenAPI file {}: {}", path, e))?;
        Self::from_openapi_json(&content)
    }

    #[allow(dead_code)]
    pub fn get_by_tag(&self, tag: &str) -> Vec<&ApiEndpoint> {
        self.endpoints
            .iter()
            .filter(|ep| ep.tags.contains(&tag.to_string()))
            .collect()
    }

    #[allow(dead_code)]
    pub fn get_by_path_prefix(&self, prefix: &str) -> Vec<&ApiEndpoint> {
        self.endpoints
            .iter()
            .filter(|ep| ep.path.starts_with(prefix))
            .collect()
    }

    #[allow(dead_code)]
    pub fn search(&self, query: &str) -> Vec<&ApiEndpoint> {
        let query_lower = query.to_lowercase();
        self.endpoints
            .iter()
            .filter(|ep| {
                ep.path.to_lowercase().contains(&query_lower)
                    || ep.method.to_lowercase().contains(&query_lower)
                    || ep
                        .summary
                        .as_ref()
                        .map(|s| s.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || ep
                        .description
                        .as_ref()
                        .map(|s| s.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_path_item_parameters_key_as_operation() {
        let json = r#"{
            "openapi": "3.0.0",
            "paths": {
                "/api/foo/{id}": {
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string" }
                        }
                    ],
                    "summary": "path summary",
                    "get": {
                        "summary": "get foo",
                        "responses": { "200": { "description": "ok" } }
                    }
                }
            }
        }"#;

        let reg = EndpointRegistry::from_openapi_json(json).expect("parse");
        assert_eq!(reg.endpoints.len(), 1);
        let ep = &reg.endpoints[0];
        assert_eq!(ep.method, "GET");
        assert_eq!(ep.path, "/api/foo/{id}");
        assert_eq!(ep.path_params.len(), 1);
        assert_eq!(ep.path_params[0].name, "id");
    }

    #[test]
    fn operation_parameters_override_path_level_same_name() {
        let json = r#"{
            "openapi": "3.0.0",
            "paths": {
                "/x": {
                    "parameters": [
                        {
                            "name": "q",
                            "in": "query",
                            "required": false,
                            "schema": { "type": "string", "default": "base" }
                        }
                    ],
                    "get": {
                        "parameters": [
                            {
                                "name": "q",
                                "in": "query",
                                "required": true,
                                "schema": { "type": "string", "default": "op" }
                            }
                        ],
                        "responses": { "200": { "description": "ok" } }
                    }
                }
            }
        }"#;

        let reg = EndpointRegistry::from_openapi_json(json).expect("parse");
        assert_eq!(reg.endpoints[0].query_params.len(), 1);
        assert_eq!(
            reg.endpoints[0].query_params[0].default.as_deref(),
            Some("op")
        );
        assert!(reg.endpoints[0].query_params[0].required);
    }

    #[test]
    fn is_openapi_operation_method_cases() {
        assert!(is_openapi_operation_method("get"));
        assert!(is_openapi_operation_method("GET"));
        assert!(!is_openapi_operation_method("parameters"));
        assert!(!is_openapi_operation_method("summary"));
    }

    #[test]
    fn resolve_path_template_substitutes_and_encodes() {
        let p = resolve_path_template("/api/roms/{id}/files", &[("id".into(), "42".into())])
            .expect("ok");
        assert_eq!(p, "/api/roms/42/files");
    }

    #[test]
    fn resolve_path_template_errors_on_missing_placeholder() {
        let e = resolve_path_template("/api/{x}", &[]).unwrap_err();
        assert!(e.to_string().contains("unresolved"));
    }
}
