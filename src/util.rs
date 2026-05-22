use std::net::IpAddr;
use std::path::{Path, PathBuf};

use crate::error::FsvError;
use crate::types::FileInfo;

/// Returns all valid non-link-local IPv4 addresses on the machine.
/// Falls back to loopback if none are found.
pub fn get_local_ips() -> Vec<IpAddr> {
    let is_valid = |ip: &IpAddr| matches!(ip, IpAddr::V4(v4) if !v4.is_link_local());

    let mut ips: Vec<IpAddr> = local_ip_address::local_ip()
        .ok()
        .filter(|ip| is_valid(ip))
        .into_iter()
        .collect();

    if let Ok(interfaces) = local_ip_address::list_afinet_netifas() {
        for (_name, ip) in interfaces {
            if is_valid(&ip) && !ips.contains(&ip) {
                ips.push(ip);
            }
        }
    }

    if ips.is_empty() {
        ips.push(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
    }

    ips
}

/// Resolves a relative path safely under `root_path`, blocking directory traversal.
pub fn resolve_safe_path(root_path: &Path, relative: Option<&str>) -> Result<PathBuf, FsvError> {
    let canonical_root = root_path.canonicalize().map_err(|e| {
        FsvError::PathError(format!("Failed to canonicalize root path: {}", e))
    })?;

    let Some(rel) = relative else {
        return Ok(canonical_root);
    };

    // Strip leading slashes to prevent absolute path injection.
    let rel_cleaned = rel.trim_start_matches(['/', '\\']);
    let joined = canonical_root.join(rel_cleaned);

    let canonical_target = joined.canonicalize().map_err(|_| FsvError::NotFound)?;

    if canonical_target.starts_with(&canonical_root) {
        Ok(canonical_target)
    } else {
        Err(FsvError::AccessDenied)
    }
}

/// Builds a `FileInfo` for `target_path` relative to `canonical_root`.
pub fn get_file_info(canonical_root: &Path, target_path: &Path) -> Result<FileInfo, FsvError> {
    let metadata = target_path.metadata()?;

    let name = target_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    // Produce a forward-slash relative path for cross-platform consistency.
    let path = target_path
        .strip_prefix(canonical_root)
        .unwrap_or(Path::new(""))
        .to_string_lossy()
        .replace('\\', "/");

    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());

    Ok(FileInfo {
        name,
        path,
        is_dir: metadata.is_dir(),
        size: metadata.len(),
        modified,
    })
}
