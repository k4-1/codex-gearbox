use std::{
    fs::{self, OpenOptions},
    io::ErrorKind,
    path::{Path, PathBuf},
    process::Stdio,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::process::Command;

const UPDATE_WORKER_ARG: &str = "__codex_gearbox_update";
const RELEASES_API: &str = "https://api.github.com/repos/k4-1/codex-gearbox/releases/latest";
const ASSET_PREFIX: &str = "https://github.com/k4-1/codex-gearbox/releases/download/";
const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const LOCK_TIMEOUT: Duration = Duration::from_secs(60 * 60);

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    digest: Option<String>,
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct Version([u64; 3]);

pub fn is_update_worker() -> bool {
    std::env::args().nth(1).as_deref() == Some(UPDATE_WORKER_ARG)
}

pub async fn delegate_to_cached() -> Result<Option<i32>> {
    if updates_disabled() {
        return Ok(None);
    }
    let current = std::env::current_exe().context("failed to locate Gearbox executable")?;
    let Some(binary) = binary_name(&current) else {
        return Ok(None);
    };
    let current_version = parse_version(env!("CARGO_PKG_VERSION"))?;
    let root = update_root()?;
    if let Some(cached) = newest_cached(&root, &binary, current_version)? {
        let status = Command::new(cached)
            .args(std::env::args_os().skip(1))
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .context("failed to launch the updated Gearbox binary")?;
        return Ok(Some(status.code().unwrap_or(1)));
    }
    spawn_worker_if_due(&current, &root).await;
    Ok(None)
}

pub async fn download_latest() {
    let result = download_latest_inner().await;
    if let Err(error) = result {
        eprintln!("Gearbox update skipped: {error:#}");
    }
}

async fn download_latest_inner() -> Result<()> {
    if updates_disabled() {
        return Ok(());
    }
    let current = std::env::current_exe().context("failed to locate Gearbox executable")?;
    let Some(binary) = binary_name(&current) else {
        return Ok(());
    };
    let current_version = parse_version(env!("CARGO_PKG_VERSION"))?;
    let root = update_root()?;
    let Some(_lock) = acquire_lock(&root)? else {
        return Ok(());
    };
    mark_check(&root)?;

    let release: Release = fetch_json(RELEASES_API).await?;
    let release_version = parse_version(&release.tag_name)?;
    if release_version <= current_version {
        return Ok(());
    }
    let asset_name = asset_name(&binary)?;
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == asset_name)
        .context("latest release has no asset for this platform")?;
    let digest = parse_digest(
        asset
            .digest
            .as_deref()
            .context("release asset has no SHA-256 digest")?,
    )?;
    if !asset.browser_download_url.starts_with(ASSET_PREFIX) {
        bail!("release asset URL is outside the pinned GitHub repository");
    }

    let version_dir = root.join(format!("v{}", version_string(release_version)));
    fs::create_dir_all(&version_dir).with_context(|| {
        format!(
            "failed to create update directory {}",
            version_dir.display()
        )
    })?;
    let final_path = version_dir.join(binary_filename(&binary));
    if final_path.is_file() {
        return Ok(());
    }
    let temporary_path = version_dir.join(format!(
        "{}.download-{}",
        binary_filename(&binary),
        std::process::id()
    ));
    download_file(&asset.browser_download_url, &temporary_path).await?;
    let bytes = fs::read(&temporary_path).with_context(|| {
        format!(
            "failed to read downloaded update {}",
            temporary_path.display()
        )
    })?;
    if Sha256::digest(&bytes).as_slice() != digest.as_slice() {
        let _ = fs::remove_file(&temporary_path);
        bail!("downloaded update failed SHA-256 verification");
    }
    set_executable(&temporary_path)?;
    fs::rename(&temporary_path, &final_path).with_context(|| {
        format!(
            "failed to install downloaded update at {}",
            final_path.display()
        )
    })?;
    Ok(())
}

async fn spawn_worker_if_due(current: &Path, root: &Path) {
    if !check_due(root) || mark_check(root).is_err() {
        return;
    }
    let _ = Command::new(current)
        .arg(UPDATE_WORKER_ARG)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

async fn fetch_json<T: for<'de> Deserialize<'de>>(url: &str) -> Result<T> {
    let output = curl()
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--proto",
            "=https",
            "--tlsv1.2",
            "--max-time",
            "15",
            "--user-agent",
            "codex-gearbox-updater",
            "--output",
            "-",
            url,
        ])
        .output()
        .await
        .context("failed to start curl for update metadata")?;
    if !output.status.success() {
        bail!("update metadata request failed");
    }
    serde_json::from_slice(&output.stdout).context("invalid update metadata")
}

async fn download_file(url: &str, destination: &Path) -> Result<()> {
    let output = curl()
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--proto",
            "=https",
            "--tlsv1.2",
            "--max-time",
            "60",
            "--output",
            destination
                .to_str()
                .context("update path is not valid UTF-8")?,
            url,
        ])
        .output()
        .await
        .context("failed to start curl for update download")?;
    if !output.status.success() {
        let _ = fs::remove_file(destination);
        bail!("update download failed");
    }
    Ok(())
}

fn curl() -> Command {
    // ponytail: native curl keeps the updater dependency-free; replace it with
    // an embedded HTTPS client if curl is unavailable on a supported platform.
    Command::new(if cfg!(windows) { "curl.exe" } else { "curl" })
}

fn update_root() -> Result<PathBuf> {
    crate::Config::path()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .map(|path| path.join("gearbox-updates"))
        .ok_or_else(|| anyhow!("cannot determine a writable Gearbox update directory"))
}

fn binary_name(path: &Path) -> Option<String> {
    let name = path.file_stem()?.to_str()?;
    matches!(name, "shift" | "codex-gearbox").then(|| name.to_owned())
}

fn binary_filename(binary: &str) -> String {
    if cfg!(windows) {
        format!("{binary}.exe")
    } else {
        binary.to_owned()
    }
}

fn asset_name(binary: &str) -> Result<String> {
    let target = target_triple().context("unsupported platform for automatic updates")?;
    Ok(format!(
        "{binary}-{target}{}",
        if cfg!(windows) { ".exe" } else { "" }
    ))
}

fn target_triple() -> Option<&'static str> {
    if cfg!(all(target_arch = "aarch64", target_os = "macos")) {
        Some("aarch64-apple-darwin")
    } else if cfg!(all(target_arch = "x86_64", target_os = "macos")) {
        Some("x86_64-apple-darwin")
    } else if cfg!(all(target_arch = "x86_64", target_os = "linux")) {
        Some("x86_64-unknown-linux-gnu")
    } else if cfg!(all(target_arch = "x86_64", target_os = "windows")) {
        Some("x86_64-pc-windows-msvc")
    } else {
        None
    }
}

fn parse_version(value: &str) -> Result<Version> {
    let value = value.strip_prefix('v').unwrap_or(value);
    let parts = value
        .split('.')
        .map(str::parse::<u64>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if parts.len() != 3 {
        bail!("unsupported version {value}");
    }
    Ok(Version([parts[0], parts[1], parts[2]]))
}

fn version_string(version: Version) -> String {
    format!("{}.{}.{}", version.0[0], version.0[1], version.0[2])
}

fn parse_digest(value: &str) -> Result<[u8; 32]> {
    let hex = value
        .strip_prefix("sha256:")
        .context("unsupported asset digest")?;
    if hex.len() != 64 {
        bail!("invalid SHA-256 digest length");
    }
    let mut digest = [0u8; 32];
    for (index, pair) in hex.as_bytes().chunks_exact(2).enumerate() {
        digest[index] = (hex_digit(pair[0])? << 4) | hex_digit(pair[1])?;
    }
    Ok(digest)
}

fn hex_digit(value: u8) -> Result<u8> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => bail!("invalid hexadecimal digest"),
    }
}

fn newest_cached(root: &Path, binary: &str, current: Version) -> Result<Option<PathBuf>> {
    let Ok(entries) = fs::read_dir(root) else {
        return Ok(None);
    };
    let mut newest = None;
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(version_text) = file_name.to_str().and_then(|name| name.strip_prefix('v')) else {
            continue;
        };
        let Ok(version) = parse_version(version_text) else {
            continue;
        };
        let candidate = entry.path().join(binary_filename(binary));
        if version > current
            && candidate.is_file()
            && newest.as_ref().is_none_or(|(v, _)| version > *v)
        {
            newest = Some((version, candidate));
        }
    }
    Ok(newest.map(|(_, path)| path))
}

fn check_due(root: &Path) -> bool {
    let path = root.join("last-check");
    let Ok(value) = fs::read_to_string(path) else {
        return true;
    };
    let Ok(last) = value.trim().parse::<u64>() else {
        return true;
    };
    now().saturating_sub(last) >= CHECK_INTERVAL.as_secs()
}

fn mark_check(root: &Path) -> Result<()> {
    fs::create_dir_all(root)?;
    fs::write(root.join("last-check"), now().to_string())?;
    Ok(())
}

fn acquire_lock(root: &Path) -> Result<Option<UpdateLock>> {
    fs::create_dir_all(root)?;
    let path = root.join("update.lock");
    match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(_) => Ok(Some(UpdateLock { path })),
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            if fs::metadata(&path)
                .and_then(|metadata| metadata.modified())
                .ok()
                .and_then(|modified| modified.elapsed().ok())
                .is_some_and(|age| age > LOCK_TIMEOUT)
            {
                let _ = fs::remove_file(&path);
            }
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

struct UpdateLock {
    path: PathBuf,
}

impl Drop for UpdateLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn updates_disabled() -> bool {
    matches!(
        std::env::var("CODEX_GEARBOX_DISABLE_UPDATE")
            .ok()
            .as_deref(),
        Some("1" | "true" | "yes")
    )
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_release_versions_and_rejects_suffixes() {
        assert_eq!(parse_version("v1.2.3").unwrap(), Version([1, 2, 3]));
        assert!(parse_version("1.2").is_err());
        assert!(parse_version("1.2.3-rc1").is_err());
    }

    #[test]
    fn verifies_sha256_digest_format() {
        let digest = parse_digest(&format!("sha256:{}", "ab".repeat(32))).unwrap();
        assert_eq!(digest, [0xab; 32]);
        assert!(parse_digest("sha256:bad").is_err());
    }

    #[test]
    fn rejects_unknown_binary_names() {
        assert_eq!(binary_name(Path::new("/tmp/shift")), Some("shift".into()));
        assert_eq!(binary_name(Path::new("/tmp/other")), None);
    }
}
