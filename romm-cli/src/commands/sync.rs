use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{anyhow, Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::format_description::well_known::Rfc3339;
use time::{OffsetDateTime, UtcOffset};
use zip::ZipArchive;

use crate::cli_presentation::CliPresentation;
use crate::commands::OutputFormat;
use romm_api::client::{RommClient, SaveUploadOptions};
use romm_api::endpoints::device::{
    DeviceSchema, GetDevice, ListDevices, RegisterDevice, SyncMode as EndpointSyncMode,
};
use romm_api::endpoints::sync::{
    CompleteSyncSession, GetSyncSession, ListSyncSessions, NegotiateSync, SyncNegotiateResponse,
    TriggerPushPull,
};
use romm_api::feature_compat::{save_sync_compatibility, SAVE_SYNC_UNSUPPORTED_MESSAGE};
use romm_api::openapi::EndpointRegistry;

#[derive(Args, Debug)]
#[command(after_help = "Examples:\n  \
      romm-cli sync plan --device-id abc --manifest saves.json\n  \
      romm-cli sync run --device-id abc --manifest saves.json --download-dir ./saves")]
pub struct SyncCommand {
    /// Output as JSON (overrides global --json when set).
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub action: SyncAction,
}

#[derive(Subcommand, Debug)]
pub enum SyncAction {
    /// Manage sync devices.
    Device(SyncDeviceCommand),
    /// Negotiate sync operations without modifying files.
    Plan(SyncPlanArgs),
    /// Execute one-shot sync operations.
    Run(SyncRunArgs),
    /// Inspect sync sessions.
    Sessions(SyncSessionsCommand),
    /// Trigger push-pull mode on a registered device.
    PushPull {
        /// Device ID
        device_id: String,
    },
}

#[derive(Args, Debug)]
pub struct SyncDeviceCommand {
    #[command(subcommand)]
    pub action: SyncDeviceAction,
}

#[derive(Subcommand, Debug)]
pub enum SyncDeviceAction {
    /// Register a device (`POST /api/devices`).
    Register {
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        platform: Option<String>,
        #[arg(long, default_value = "romm-cli")]
        client: String,
        #[arg(long)]
        client_version: Option<String>,
        #[arg(long)]
        hostname: Option<String>,
        #[arg(long)]
        mac_address: Option<String>,
        #[arg(long)]
        ip_address: Option<String>,
        #[arg(long, value_enum, default_value_t = CliSyncMode::Api)]
        sync_mode: CliSyncMode,
        /// Optional JSON object string for `sync_config`.
        #[arg(long)]
        sync_config_json: Option<String>,
        #[arg(long)]
        allow_duplicate: bool,
        #[arg(long)]
        reset_syncs: bool,
    },
    /// List devices (`GET /api/devices`).
    List,
    /// Get one device (`GET /api/devices/{id}`).
    Get {
        /// Device ID
        device_id: String,
    },
}

#[derive(Args, Debug)]
pub struct SyncPlanArgs {
    #[arg(long)]
    pub device_id: String,
    #[arg(long)]
    pub manifest: PathBuf,
}

#[derive(Args, Debug)]
pub struct SyncRunArgs {
    #[arg(long)]
    pub device_id: String,
    #[arg(long)]
    pub manifest: PathBuf,
    /// Folder for downloaded saves (defaults to the manifest directory).
    #[arg(long)]
    pub download_dir: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = ConflictPolicy::Fail)]
    pub conflict: ConflictPolicy,
}

#[derive(Args, Debug)]
pub struct SyncSessionsCommand {
    #[command(subcommand)]
    pub action: SyncSessionsAction,
}

#[derive(Subcommand, Debug)]
pub enum SyncSessionsAction {
    /// List sessions (`GET /api/sync/sessions`).
    List {
        #[arg(long)]
        device_id: Option<String>,
        #[arg(long)]
        limit: Option<u32>,
    },
    /// Get one session (`GET /api/sync/sessions/{id}`).
    Get { session_id: u64 },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliSyncMode {
    Api,
    FileTransfer,
    PushPull,
}

impl From<CliSyncMode> for EndpointSyncMode {
    fn from(value: CliSyncMode) -> Self {
        match value {
            CliSyncMode::Api => EndpointSyncMode::Api,
            CliSyncMode::FileTransfer => EndpointSyncMode::FileTransfer,
            CliSyncMode::PushPull => EndpointSyncMode::PushPull,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum ConflictPolicy {
    Fail,
    Skip,
}

#[derive(Debug, Clone, Deserialize)]
struct SyncManifest {
    saves: Vec<ManifestSave>,
}

#[derive(Debug, Clone, Deserialize)]
struct ManifestSave {
    rom_id: u64,
    path: PathBuf,
    file_name: Option<String>,
    slot: Option<String>,
    emulator: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ClientSaveState {
    rom_id: u64,
    file_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    slot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emulator: Option<String>,
    content_hash: String,
    updated_at: String,
    file_size_bytes: u64,
}

#[derive(Debug, Clone)]
struct PreparedSave {
    path: PathBuf,
    client: ClientSaveState,
}

fn safe_download_file_name(input: &str, save_id: u64) -> String {
    let cleaned: String = input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.').trim();
    if trimmed.is_empty() {
        format!("save-{save_id}.sav")
    } else {
        trimmed.to_string()
    }
}

fn reserve_download_target(
    download_base: &Path,
    file_name: &str,
    save_id: u64,
    reserved: &mut HashSet<PathBuf>,
) -> PathBuf {
    let preferred = safe_download_file_name(file_name, save_id);
    let preferred_path = download_base.join(&preferred);
    if !reserved.contains(&preferred_path) && !preferred_path.exists() {
        reserved.insert(preferred_path.clone());
        return preferred_path;
    }

    let path = Path::new(&preferred);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("save");
    let extension = path.extension().and_then(|s| s.to_str());

    for attempt in 0u64.. {
        let suffix = if attempt == 0 {
            format!("-save-{save_id}")
        } else {
            format!("-save-{save_id}-{attempt}")
        };
        let candidate = match extension {
            Some(ext) => format!("{stem}{suffix}.{ext}"),
            None => format!("{stem}{suffix}"),
        };
        let candidate_path = download_base.join(candidate);
        if !reserved.contains(&candidate_path) && !candidate_path.exists() {
            reserved.insert(candidate_path.clone());
            return candidate_path;
        }
    }

    unreachable!("unbounded search for save download target should always find a free filename")
}

#[derive(Debug, Clone, Copy, Default)]
struct RunCounts {
    uploaded: u64,
    downloaded: u64,
    no_op: u64,
    conflicts_skipped: u64,
    completed: u64,
    failed: u64,
}

pub async fn handle(
    cmd: SyncCommand,
    client: &RommClient,
    presentation: CliPresentation,
) -> Result<()> {
    let format = presentation.format;
    preflight_save_sync_compatibility(client, format).await?;

    match cmd.action {
        SyncAction::Device(device_cmd) => handle_device(device_cmd, client, format).await,
        SyncAction::Plan(args) => handle_plan(args, client, format).await,
        SyncAction::Run(args) => handle_run(args, client, format).await,
        SyncAction::Sessions(cmd) => handle_sessions(cmd, client, format).await,
        SyncAction::PushPull { device_id } => {
            let out = client.call(&TriggerPushPull { device_id }).await?;
            print_output(format, &out)
        }
    }
}

async fn preflight_save_sync_compatibility(
    client: &RommClient,
    format: OutputFormat,
) -> Result<()> {
    let openapi = match client.fetch_openapi_json().await {
        Ok(body) => body,
        Err(e) => {
            tracing::warn!(
                "Skipping save-sync compatibility preflight: {}",
                e.redacted_for_log()
            );
            return Ok(());
        }
    };
    let registry = match EndpointRegistry::from_openapi_json(&openapi) {
        Ok(registry) => registry,
        Err(e) => {
            tracing::warn!(
                "Skipping save-sync compatibility preflight; OpenAPI parse failed: {e:#}"
            );
            return Ok(());
        }
    };
    let compat = save_sync_compatibility(&registry);
    if compat.supported {
        return Ok(());
    }

    if matches!(format, OutputFormat::Json) {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "error": "save_sync_unsupported",
                "message": SAVE_SYNC_UNSUPPORTED_MESSAGE,
                "missing_endpoints": compat
                    .missing
                    .iter()
                    .map(|ep| ep.label())
                    .collect::<Vec<_>>()
            }))?
        );
    }

    anyhow::bail!("{}", compat.unsupported_message())
}

async fn handle_device(
    cmd: SyncDeviceCommand,
    client: &RommClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd.action {
        SyncDeviceAction::Register {
            name,
            platform,
            client: client_name,
            client_version,
            hostname,
            mac_address,
            ip_address,
            sync_mode,
            sync_config_json,
            allow_duplicate,
            reset_syncs,
        } => {
            let sync_config = match sync_config_json {
                Some(raw) => Some(
                    serde_json::from_str::<serde_json::Value>(&raw)
                        .with_context(|| "invalid --sync-config-json (must be a JSON object)")?,
                ),
                None => None,
            };
            if let Some(v) = &sync_config {
                if !v.is_object() {
                    anyhow::bail!("--sync-config-json must decode to a JSON object");
                }
            }

            let body = json!({
                "name": name,
                "platform": platform,
                "client": client_name,
                "client_version": client_version,
                "hostname": hostname,
                "mac_address": mac_address,
                "ip_address": ip_address,
                "sync_mode": EndpointSyncMode::from(sync_mode),
                "sync_config": sync_config,
                "allow_existing": true,
                "allow_duplicate": allow_duplicate,
                "reset_syncs": reset_syncs
            });
            let created = client.call(&RegisterDevice { body }).await?;
            print_output(format, &created)
        }
        SyncDeviceAction::List => {
            let rows: Vec<DeviceSchema> = client.call(&ListDevices).await?;
            print_output(format, &rows)
        }
        SyncDeviceAction::Get { device_id } => {
            let row = client.call(&GetDevice { device_id }).await?;
            print_output(format, &row)
        }
    }
}

async fn handle_plan(args: SyncPlanArgs, client: &RommClient, format: OutputFormat) -> Result<()> {
    let prepared = load_manifest_and_prepare(&args.manifest)?;
    let negotiate = negotiate(client, &args.device_id, &prepared).await?;
    print_output(format, &negotiate)
}

async fn handle_run(args: SyncRunArgs, client: &RommClient, format: OutputFormat) -> Result<()> {
    let prepared = load_manifest_and_prepare(&args.manifest)?;
    let prepared_by_key = prepared_by_key(&prepared)?;
    let negotiate = negotiate(client, &args.device_id, &prepared).await?;

    let download_base = match args.download_dir {
        Some(dir) => dir,
        None => args
            .manifest
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(".")),
    };
    std::fs::create_dir_all(&download_base).with_context(|| {
        format!(
            "failed to create download directory {}",
            download_base.display()
        )
    })?;

    let mut counts = RunCounts::default();
    let mut hard_conflict = false;
    let mut reserved_download_targets = HashSet::new();
    for op in &negotiate.operations {
        match op.action.as_str() {
            "upload" => {
                let key = (op.rom_id, op.file_name.clone());
                let Some(local) = prepared_by_key.get(&key) else {
                    counts.failed += 1;
                    eprintln!(
                        "Missing local manifest entry for upload operation rom_id={} file_name={}",
                        op.rom_id, op.file_name
                    );
                    continue;
                };
                let options = SaveUploadOptions {
                    emulator: local.client.emulator.as_deref(),
                    slot: local.client.slot.as_deref(),
                    device_id: Some(args.device_id.as_str()),
                    session_id: Some(negotiate.session_id),
                    overwrite: false,
                };
                match client
                    .upload_save_file_with_options(local.client.rom_id, &local.path, &options)
                    .await
                {
                    Ok(_) => {
                        counts.uploaded += 1;
                        counts.completed += 1;
                    }
                    Err(err) => {
                        counts.failed += 1;
                        eprintln!(
                            "Upload failed for rom_id={} file_name={}: {:#}",
                            local.client.rom_id, local.client.file_name, err
                        );
                    }
                }
            }
            "download" => {
                let Some(save_id) = op.save_id else {
                    counts.failed += 1;
                    eprintln!(
                        "Download operation missing save_id for rom_id={} file_name={}",
                        op.rom_id, op.file_name
                    );
                    continue;
                };
                match client
                    .download_save_content(
                        save_id,
                        Some(args.device_id.as_str()),
                        Some(negotiate.session_id),
                    )
                    .await
                {
                    Ok(bytes) => {
                        let target = reserve_download_target(
                            &download_base,
                            &op.file_name,
                            save_id,
                            &mut reserved_download_targets,
                        );
                        if let Some(parent) = target.parent() {
                            std::fs::create_dir_all(parent).with_context(|| {
                                format!("failed to create parent folder {}", parent.display())
                            })?;
                        }
                        let mut f = File::create(&target).with_context(|| {
                            format!("failed to create download file {}", target.display())
                        })?;
                        f.write_all(&bytes).with_context(|| {
                            format!("failed to write download file {}", target.display())
                        })?;
                        counts.downloaded += 1;
                        counts.completed += 1;
                    }
                    Err(err) => {
                        counts.failed += 1;
                        eprintln!("Download failed for save_id={save_id}: {err:#}");
                    }
                }
            }
            "no_op" => {
                counts.no_op += 1;
            }
            "conflict" => match args.conflict {
                ConflictPolicy::Skip => {
                    counts.conflicts_skipped += 1;
                }
                ConflictPolicy::Fail => {
                    counts.failed += 1;
                    hard_conflict = true;
                    eprintln!(
                        "Conflict for rom_id={} file_name={}: {}",
                        op.rom_id, op.file_name, op.reason
                    );
                }
            },
            other => {
                counts.failed += 1;
                eprintln!(
                    "Unknown sync operation '{}' for rom_id={} file_name={}",
                    other, op.rom_id, op.file_name
                );
            }
        }
    }

    let completion = client
        .call(&CompleteSyncSession {
            session_id: negotiate.session_id,
            body: json!({
                "operations_completed": counts.completed,
                "operations_failed": counts.failed
            }),
        })
        .await?;

    if matches!(format, OutputFormat::Json) {
        let out = json!({
            "negotiate": negotiate,
            "counts": {
                "uploaded": counts.uploaded,
                "downloaded": counts.downloaded,
                "no_op": counts.no_op,
                "conflicts_skipped": counts.conflicts_skipped,
                "completed": counts.completed,
                "failed": counts.failed
            },
            "completion": completion
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!(
            "session={} uploaded={} downloaded={} no_op={} conflicts_skipped={} completed={} failed={}",
            completion.session.id,
            counts.uploaded,
            counts.downloaded,
            counts.no_op,
            counts.conflicts_skipped,
            counts.completed,
            counts.failed
        );
    }

    if hard_conflict || counts.failed > 0 {
        anyhow::bail!(
            "sync completed with {} failed operation(s); session {} marked complete",
            counts.failed,
            completion.session.id
        );
    }

    Ok(())
}

async fn handle_sessions(
    cmd: SyncSessionsCommand,
    client: &RommClient,
    format: OutputFormat,
) -> Result<()> {
    match cmd.action {
        SyncSessionsAction::List { device_id, limit } => {
            let out = client.call(&ListSyncSessions { device_id, limit }).await?;
            print_output(format, &out)
        }
        SyncSessionsAction::Get { session_id } => {
            let out = client.call(&GetSyncSession { session_id }).await?;
            print_output(format, &out)
        }
    }
}

async fn negotiate(
    client: &RommClient,
    device_id: &str,
    prepared: &[PreparedSave],
) -> Result<SyncNegotiateResponse> {
    let saves: Vec<ClientSaveState> = prepared.iter().map(|p| p.client.clone()).collect();
    client
        .call(&NegotiateSync {
            body: json!({
                "device_id": device_id,
                "saves": saves
            }),
        })
        .await
        .map_err(Into::into)
}

fn print_output<T: Serialize>(format: OutputFormat, value: &T) -> Result<()> {
    match format {
        OutputFormat::Json | OutputFormat::Text => {
            println!("{}", serde_json::to_string_pretty(value)?);
        }
    }
    Ok(())
}

fn prepared_by_key(prepared: &[PreparedSave]) -> Result<HashMap<(u64, String), PreparedSave>> {
    let mut map = HashMap::new();
    for item in prepared {
        let key = (item.client.rom_id, item.client.file_name.clone());
        if map.insert(key.clone(), item.clone()).is_some() {
            anyhow::bail!(
                "duplicate manifest mapping for rom_id={} file_name={}",
                key.0,
                key.1
            );
        }
    }
    Ok(map)
}

fn load_manifest_and_prepare(manifest_path: &Path) -> Result<Vec<PreparedSave>> {
    let raw = std::fs::read_to_string(manifest_path)
        .with_context(|| format!("read manifest {}", manifest_path.display()))?;
    let manifest: SyncManifest = serde_json::from_str(&raw)
        .with_context(|| format!("parse manifest {}", manifest_path.display()))?;
    let base_dir = manifest_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let mut out = Vec::new();
    for row in manifest.saves {
        let path = if row.path.is_absolute() {
            row.path.clone()
        } else {
            base_dir.join(&row.path)
        };
        if !path.is_file() {
            anyhow::bail!("manifest save path is not a file: {}", path.display());
        }
        let file_name = match row.file_name {
            Some(name) if !name.trim().is_empty() => name.trim().to_string(),
            _ => path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| {
                    anyhow!("save path must have a unicode filename: {}", path.display())
                })?
                .to_string(),
        };
        let meta = std::fs::metadata(&path)
            .with_context(|| format!("read metadata for {}", path.display()))?;
        let updated_at = format_system_time_utc_rfc3339(
            meta.modified()
                .with_context(|| format!("read modified timestamp for {}", path.display()))?,
        )?;
        let content_hash = compute_content_hash(&path)?;
        out.push(PreparedSave {
            path,
            client: ClientSaveState {
                rom_id: row.rom_id,
                file_name,
                slot: row.slot.filter(|s| !s.trim().is_empty()),
                emulator: row.emulator.filter(|s| !s.trim().is_empty()),
                content_hash,
                updated_at,
                file_size_bytes: meta.len(),
            },
        });
    }

    Ok(out)
}

fn format_system_time_utc_rfc3339(t: SystemTime) -> Result<String> {
    let odt: OffsetDateTime = t.into();
    let utc = odt.to_offset(UtcOffset::UTC);
    utc.format(&Rfc3339)
        .map_err(|e| anyhow!("format timestamp as RFC3339: {e}"))
}

fn compute_content_hash(path: &Path) -> Result<String> {
    if let Ok(file) = File::open(path) {
        if ZipArchive::new(file).is_ok() {
            return compute_zip_hash(path);
        }
    }
    compute_file_hash(path)
}

fn compute_file_hash(path: &Path) -> Result<String> {
    let mut file =
        File::open(path).with_context(|| format!("open file for hashing {}", path.display()))?;
    let mut ctx = md5::Context::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file
            .read(&mut buf)
            .with_context(|| format!("read file for hashing {}", path.display()))?;
        if n == 0 {
            break;
        }
        ctx.consume(&buf[..n]);
    }
    Ok(format!("{:x}", ctx.finalize()))
}

fn compute_zip_hash(path: &Path) -> Result<String> {
    let file = File::open(path)
        .with_context(|| format!("open zip file for hashing {}", path.display()))?;
    let mut archive = ZipArchive::new(file)
        .with_context(|| format!("read zip archive for hashing {}", path.display()))?;
    let mut row_hashes = BTreeMap::new();
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .with_context(|| format!("read zip entry {} in {}", i, path.display()))?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        let mut ctx = md5::Context::new();
        let mut buf = [0u8; 8192];
        loop {
            let n = entry
                .read(&mut buf)
                .with_context(|| format!("hash zip entry {} in {}", name, path.display()))?;
            if n == 0 {
                break;
            }
            ctx.consume(&buf[..n]);
        }
        row_hashes.insert(name, format!("{:x}", ctx.finalize()));
    }
    let combined = row_hashes
        .into_iter()
        .map(|(name, hash)| format!("{name}:{hash}"))
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!("{:x}", md5::compute(combined.as_bytes())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("romm-cli-sync-test-{nanos}-{name}"))
    }

    #[test]
    fn file_hash_matches_md5_hex() {
        let path = temp_path("plain.sav");
        fs::write(&path, b"abc123").expect("write");
        let got = compute_file_hash(&path).expect("hash");
        let expected = format!("{:x}", md5::compute(b"abc123"));
        assert_eq!(got, expected);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn zip_hash_matches_sorted_entry_scheme() {
        let path = temp_path("archive.zip");
        {
            let f = File::create(&path).expect("create");
            let mut writer = ZipWriter::new(f);
            writer
                .start_file("b.sav", SimpleFileOptions::default())
                .expect("start file");
            writer.write_all(b"bbb").expect("write b");
            writer
                .start_file("a.sav", SimpleFileOptions::default())
                .expect("start file");
            writer.write_all(b"aaa").expect("write a");
            writer.finish().expect("finish");
        }

        let hash = compute_zip_hash(&path).expect("hash zip");
        let a = format!("{:x}", md5::compute(b"aaa"));
        let b = format!("{:x}", md5::compute(b"bbb"));
        let combined = format!("a.sav:{a}\nb.sav:{b}");
        let expected = format!("{:x}", md5::compute(combined.as_bytes()));
        assert_eq!(hash, expected);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn duplicate_manifest_keys_fail() {
        let p = temp_path("dup.sav");
        fs::write(&p, b"x").expect("write");
        let prepared = vec![
            PreparedSave {
                path: p.clone(),
                client: ClientSaveState {
                    rom_id: 1,
                    file_name: "same.sav".into(),
                    slot: None,
                    emulator: None,
                    content_hash: "h1".into(),
                    updated_at: "2026-01-01T00:00:00Z".into(),
                    file_size_bytes: 1,
                },
            },
            PreparedSave {
                path: p.clone(),
                client: ClientSaveState {
                    rom_id: 1,
                    file_name: "same.sav".into(),
                    slot: None,
                    emulator: None,
                    content_hash: "h2".into(),
                    updated_at: "2026-01-01T00:00:01Z".into(),
                    file_size_bytes: 1,
                },
            },
        ];
        let err = prepared_by_key(&prepared).expect_err("duplicate should fail");
        assert!(err.to_string().contains("duplicate manifest mapping"));
        let _ = fs::remove_file(p);
    }

    #[test]
    fn safe_download_file_name_removes_path_separators() {
        assert_eq!(
            safe_download_file_name("../folder/evil.sav", 12),
            "_folder_evil.sav"
        );
        assert_eq!(
            safe_download_file_name(r"..\folder\evil.sav", 12),
            "_folder_evil.sav"
        );
    }

    #[test]
    fn safe_download_file_name_falls_back_when_empty() {
        assert_eq!(safe_download_file_name("...", 42), "save-42.sav");
    }

    #[test]
    fn reserve_download_target_avoids_existing_and_reserved_files() {
        let dir = temp_path("download-targets");
        fs::create_dir_all(&dir).expect("mkdir");
        fs::write(dir.join("shared.sav"), b"keep").expect("write existing");
        let mut reserved = HashSet::new();

        let first = reserve_download_target(&dir, "shared.sav", 55, &mut reserved);
        assert_eq!(
            first.file_name().and_then(|n| n.to_str()),
            Some("shared-save-55.sav")
        );
        fs::write(&first, b"first").expect("write reserved");

        let second = reserve_download_target(&dir, "shared.sav", 56, &mut reserved);
        assert_eq!(
            second.file_name().and_then(|n| n.to_str()),
            Some("shared-save-56.sav")
        );
        assert_eq!(
            fs::read(dir.join("shared.sav")).expect("read existing"),
            b"keep"
        );

        let _ = fs::remove_dir_all(dir);
    }
}
