use serde_json::Value;

pub mod client_tokens;
pub mod collections;
pub mod platforms;
pub mod roms;
pub mod system;
pub mod tasks;

/// Generic description of a RomM API endpoint.
///
/// Implementations of this trait define the structure and behavior of a specific
/// API call, including its HTTP method, path, query parameters, and body.
pub trait Endpoint {
    /// The expected output type of this endpoint, which must be deserializable from JSON.
    type Output;

    /// Returns the HTTP method (e.g., "GET", "POST", "PUT", "DELETE").
    fn method(&self) -> &'static str;

    /// Returns the path relative to the base URL (e.g., "/api/roms").
    fn path(&self) -> String;

    /// Returns the query parameters as a list of key/value pairs.
    /// Defaults to an empty list.
    fn query(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// Returns the optional JSON request body.
    /// Defaults to `None`.
    fn body(&self) -> Option<Value> {
        None
    }
}
