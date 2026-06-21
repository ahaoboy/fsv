use std::net::IpAddr;
use std::path::{Path, PathBuf};
use tracing;

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
///
/// This function is designed to work reliably on Android devices where:
/// - Symlinks are common (e.g., /sdcard -> /storage/emulated/0)
/// - Permission issues may prevent canonicalization
/// - Case-sensitive filesystems are used
pub fn resolve_safe_path(root_path: &Path, relative: Option<&str>) -> Result<PathBuf, FsvError> {
    // Try to canonicalize root, but fall back to absolute path if it fails
    let canonical_root = root_path
        .canonicalize()
        .or_else(|_| {
            // On Android, canonicalize may fail due to permissions
            // Fall back to using the absolute path directly
            if root_path.is_absolute() {
                Ok(root_path.to_path_buf())
            } else {
                std::env::current_dir()
                    .map(|cwd| cwd.join(root_path))
                    .map_err(|e| FsvError::PathError(format!("Failed to resolve root path: {}", e)))
            }
        })?;

    let Some(rel) = relative else {
        return Ok(canonical_root);
    };

    // Strip leading slashes to prevent absolute path injection
    let rel_cleaned = rel.trim_start_matches(['/', '\\']);

    // Prevent directory traversal by checking for ".." components
    if rel_cleaned.split(['/', '\\']).any(|part| part == "..") {
        tracing::warn!(
            root = %canonical_root.display(),
            relative = %rel,
            "blocked directory traversal attempt"
        );
        return Err(FsvError::AccessDenied);
    }

    // When the root itself is a file, the only valid relative path is the
    // file's own name (e.g. `fsv ./foo.txt` → GET /foo.txt → serve the file).
    if canonical_root.is_file() {
        let root_name = canonical_root
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();
        if rel_cleaned == root_name {
            return Ok(canonical_root);
        }
        tracing::debug!(
            root = %canonical_root.display(),
            relative = %rel_cleaned,
            "relative path does not match single-file root name"
        );
        return Err(FsvError::NotFound);
    }

    let joined = canonical_root.join(rel_cleaned);

    // Try to canonicalize the target path
    let canonical_target = match joined.canonicalize() {
        Ok(path) => path,
        Err(_) => {
            // On Android, canonicalize may fail for valid paths due to:
            // - Symlinks in the path
            // - Permission restrictions
            // - Non-existent intermediate directories

            // Check if the path exists without canonicalizing
            if joined.exists() {
                // Use the joined path directly, but verify it's under root
                // by checking string prefix (less secure but works on Android)
                let joined_str = joined.to_string_lossy();
                let root_str = canonical_root.to_string_lossy();

                if joined_str.starts_with(root_str.as_ref()) {
                    tracing::debug!(
                        path = %joined_str,
                        "canonicalize failed, using joined path (Android fallback)"
                    );
                    joined
                } else {
                    tracing::warn!(
                        path = %joined_str,
                        root = %root_str,
                        "joined path is outside root (Android fallback)"
                    );
                    return Err(FsvError::AccessDenied);
                }
            } else {
                tracing::debug!(
                    path = %joined.display(),
                    "path not found"
                );
                return Err(FsvError::NotFound);
            }
        }
    };

    // Final security check: ensure the resolved path is under root
    // Use both canonicalized comparison and string prefix check for Android compatibility
    if canonical_target.starts_with(&canonical_root) {
        Ok(canonical_target)
    } else {
        // Fallback check for Android where symlinks might cause issues
        let target_str = canonical_target.to_string_lossy();
        let root_str = canonical_root.to_string_lossy();

        if target_str.starts_with(root_str.as_ref()) {
            Ok(canonical_target)
        } else {
            tracing::warn!(
                target = %target_str,
                root = %root_str,
                "resolved path is outside root — access denied"
            );
            Err(FsvError::AccessDenied)
        }
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

    // When sharing a single file, root and target are the same path,
    // so strip_prefix yields "". Use the filename as the URL path instead.
    let path = if path.is_empty() && target_path == canonical_root {
        name.clone()
    } else {
        path
    };

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
