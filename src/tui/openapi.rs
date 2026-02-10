use anyhow::{anyhow, Result};
use serde_json::Value;

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
    #[allow(dead_code)]
    pub path_params: Vec<ApiParameter>,
    pub has_body: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EndpointRegistry {
    pub endpoints: Vec<ApiEndpoint>,
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

        for (path, methods_obj) in paths {
            let methods_obj = methods_obj
                .as_object()
                .ok_or_else(|| anyhow!("Invalid path definition for {}", path))?;

            for (method, operation) in methods_obj {
                let method_upper = method.to_uppercase();
                let operation = operation
                    .as_object()
                    .ok_or_else(|| anyhow!("Invalid operation for {} {}", method, path))?;

                let summary = operation.get("summary").and_then(|v| v.as_str()).map(|s| s.to_string());
                let description = operation.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());

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

                let mut query_params = Vec::new();
                let mut path_params = Vec::new();

                if let Some(params) = operation.get("parameters").and_then(|v| v.as_array()) {
                    for param in params {
                        if let Some(param_obj) = param.as_object() {
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

                            let default = schema
                                .and_then(|s| s.get("default"))
                                .and_then(|v| {
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
                    }
                }

                let has_body = operation.get("requestBody").is_some();

                endpoints.push(ApiEndpoint {
                    method: method_upper,
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

        endpoints.sort_by(|a, b| {
            a.path.cmp(&b.path).then_with(|| a.method.cmp(&b.method))
        });

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
                    || ep.summary.as_ref().map(|s| s.to_lowercase().contains(&query_lower)).unwrap_or(false)
                    || ep.description.as_ref().map(|s| s.to_lowercase().contains(&query_lower)).unwrap_or(false)
            })
            .collect()
    }
}
