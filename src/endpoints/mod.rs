use serde_json::Value;

pub mod collections;
pub mod platforms;
pub mod roms;

/// Generic description of a ROMM API endpoint.
pub trait Endpoint {
    type Output;

    /// HTTP method, e.g. "GET", "POST".
    fn method(&self) -> &'static str;

    /// Path relative to the base URL, e.g. "/api/roms".
    fn path(&self) -> String;

    /// Query parameters as key/value pairs.
    fn query(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// Optional JSON request body.
    fn body(&self) -> Option<Value> {
        None
    }
}

