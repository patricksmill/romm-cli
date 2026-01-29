use std::env;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use serde_json::Value;

/// Very small sketch of an OpenAPI-driven generator.
///
/// For now, this just:
/// - reads an OpenAPI JSON file
/// - lists available paths and methods
///
/// In the future, this could:
/// - emit endpoint structs following `endpoints::*` conventions
/// - emit type stubs into dedicated modules.
fn main() -> Result<()> {
    let path = env::args()
        .nth(1)
        .ok_or_else(|| anyhow!("usage: romm_openapi_gen <openapi.json>"))?;

    let path = Path::new(&path);
    let data = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&data)?;

    let paths = value
        .get("paths")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow!("openapi.json has no top-level \"paths\" object"))?;

    println!("Discovered endpoints:");
    for (p, methods) in paths {
        if let Some(methods_obj) = methods.as_object() {
            for (method, _meta) in methods_obj {
                println!("  {} {}", method.to_uppercase(), p);
            }
        }
    }

    Ok(())
}

