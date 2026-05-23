use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
};
use std::path::Path;
use tokio_util::io::ReaderStream;
use webdav_serde::{Multistatus, Response as DavResponse, PropStat, Prop, ResourceType, Collection};

use crate::error::FsvError;
use crate::types::AppState;
use crate::util::resolve_safe_path;

/// WebDAV handler - supports PROPFIND (list) and GET (download)
pub async fn webdav_handler(
    State(state): State<AppState>,
    method: Method,
    request: Request,
) -> Result<Response, FsvError> {
    let path = request.uri().path();

    // No need to strip prefix - unified routing handles all paths
    let rel_path = path.trim_start_matches('/');
    let rel_path = if rel_path.is_empty() {
        None
    } else {
        Some(rel_path)
    };

    // When sharing a single file, normalize empty paths to the file's name
    // so PROPFIND and GET use /{filename} instead of /
    let rel_path = match rel_path {
        None if state.root_path.is_file() => state
            .root_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.to_string()),
        other => other.map(|s| s.to_string()),
    };
    let rel_path = rel_path.as_deref();

    match method {
        Method::OPTIONS => Ok(webdav_options()),
        ref m if m.as_str() == "PROPFIND" => {
            webdav_propfind(&state, rel_path).await
        }
        Method::GET => webdav_get(&state, rel_path).await,
        Method::HEAD => webdav_head(&state, rel_path).await,
        _ => Ok((StatusCode::METHOD_NOT_ALLOWED, "Method not allowed").into_response()),
    }
}

/// OPTIONS response for WebDAV
fn webdav_options() -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(header::ALLOW, "OPTIONS, PROPFIND, GET, HEAD".parse().unwrap());
    headers.insert("DAV", "1, 2".parse().unwrap());
    headers.insert("MS-Author-Via", "DAV".parse().unwrap());

    (StatusCode::OK, headers).into_response()
}

/// PROPFIND - list directory contents or file properties
async fn webdav_propfind(
    state: &AppState,
    rel_path: Option<&str>,
) -> Result<Response, FsvError> {
    let target = resolve_safe_path(&state.root_path, rel_path)?;

    let xml = if target.is_dir() {
        propfind_directory(&target, rel_path.unwrap_or("")).await?
    } else {
        propfind_file(&target, rel_path.unwrap_or("")).await?
    };

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/xml; charset=utf-8".parse().unwrap());

    Ok((StatusCode::MULTI_STATUS, headers, xml).into_response())
}

/// Generate PROPFIND XML for a directory
async fn propfind_directory(dir: &Path, rel_path: &str) -> Result<String, FsvError> {
    let mut responses = vec![];

    // Add the directory itself
    let dir_meta = tokio::fs::metadata(dir).await?;
    let dir_href = if rel_path.is_empty() {
        "/".to_string()
    } else {
        format!("/{}/", rel_path.trim_start_matches('/'))
    };

    responses.push(create_dav_response(
        &dir_href,
        "",
        true,
        0,
        dir_meta.modified().ok(),
    ));

    // Add directory contents
    let mut read_dir = tokio::fs::read_dir(dir).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let metadata = entry.metadata().await?;
        let name = entry.file_name().to_string_lossy().to_string();

        let entry_path = if rel_path.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", rel_path, name)
        };

        let href = if metadata.is_dir() {
            format!("/{}/", entry_path.trim_start_matches('/'))
        } else {
            format!("/{}", entry_path.trim_start_matches('/'))
        };

        responses.push(create_dav_response(
            &href,
            &name,
            metadata.is_dir(),
            metadata.len(),
            metadata.modified().ok(),
        ));
    }

    let multistatus = Multistatus { response: responses };
    multistatus.to_xml().map_err(|e| FsvError::PathError(format!("XML serialization error: {}", e)))
}

/// Generate PROPFIND XML for a single file
async fn propfind_file(file: &Path, rel_path: &str) -> Result<String, FsvError> {
    let metadata = tokio::fs::metadata(file).await?;
    let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");

    let href = format!("/{}", rel_path.trim_start_matches('/'));
    let response = create_dav_response(
        &href,
        name,
        false,
        metadata.len(),
        metadata.modified().ok(),
    );

    let multistatus = Multistatus { response: vec![response] };
    multistatus.to_xml().map_err(|e| FsvError::PathError(format!("XML serialization error: {}", e)))
}

/// Create a WebDAV response structure
fn create_dav_response(
    href: &str,
    display_name: &str,
    is_dir: bool,
    size: u64,
    modified: Option<std::time::SystemTime>,
) -> DavResponse {
    // Only set resourcetype for directories (collections)
    // For files, resourcetype should be None (will be omitted from XML)
    let resource_type = if is_dir {
        Some(ResourceType {
            collection: Some(Collection {}),
        })
    } else {
        None
    };

    let modified_str = modified
        .map(httpdate::fmt_http_date);

    DavResponse {
        href: href.to_string(),
        propstat: PropStat {
            prop: Prop {
                displayname: if display_name.is_empty() { None } else { Some(display_name.to_string()) },
                creationdate: None,
                getcontentlength: Some(size),
                getcontenttype: None,
                getetag: None,
                getlastmodified: modified_str,
                lockdiscovery: None,
                resourcetype: resource_type,
                supportedlock: None,
            },
            status: "HTTP/1.1 200 OK".to_string(),
        },
    }
}

/// GET - download file
pub async fn webdav_get(
    state: &AppState,
    rel_path: Option<&str>,
) -> Result<Response, FsvError> {
    let target = resolve_safe_path(&state.root_path, rel_path)?;

    if target.is_dir() {
        return Err(FsvError::NotAFile);
    }

    let file = tokio::fs::File::open(&target).await?;
    let metadata = file.metadata().await?;
    let file_len = metadata.len();
    let file_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download");

    // Detect MIME type from extension
    let mime_type = get_mime_type(file_name);

    // Generate ETag from modified time and size
    let etag = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| format!("\"{}-{}\"", d.as_secs(), file_len))
        .unwrap_or_else(|| format!("\"{}\"", file_len));

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, mime_type.parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("inline; filename=\"{}\"", file_name).parse().unwrap(),
    );
    headers.insert(header::ACCEPT_RANGES, "bytes".parse().unwrap());
    headers.insert(header::ETAG, etag.parse().unwrap());
    headers.insert(header::CACHE_CONTROL, "no-cache".parse().unwrap());

    if let Ok(modified) = metadata.modified() {
        headers.insert(
            header::LAST_MODIFIED,
            httpdate::fmt_http_date(modified).parse().unwrap(),
        );
    }

    headers.insert(
        header::CONTENT_LENGTH,
        file_len.to_string().parse().unwrap(),
    );

    Ok((headers, Body::from_stream(ReaderStream::new(file))).into_response())
}

/// Detect MIME type from file extension
fn get_mime_type(filename: &str) -> &'static str {
    let ext = filename.split('.').next_back().unwrap_or("").to_lowercase();
    match ext.as_str() {
        // Video
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "ogg" | "ogv" => "video/ogg",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        "mkv" => "video/x-matroska",
        // Audio
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "flac" => "audio/flac",
        "aac" => "audio/aac",
        "m4a" => "audio/mp4",
        "oga" => "audio/ogg",
        // Image
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        "avif" => "image/avif",
        // Text
        "txt" => "text/plain; charset=utf-8",
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "md" => "text/markdown; charset=utf-8",
        // Application
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        "7z" => "application/x-7z-compressed",
        _ => "application/octet-stream",
    }
}

/// GET with Range support - download partial file content
pub async fn webdav_get_range(
    state: &AppState,
    rel_path: Option<&str>,
    range_header: &str,
) -> Result<Response, FsvError> {
    let target = resolve_safe_path(&state.root_path, rel_path)?;

    if target.is_dir() {
        return Err(FsvError::NotAFile);
    }

    let file = tokio::fs::File::open(&target).await?;
    let metadata = file.metadata().await?;
    let file_len = metadata.len();
    let file_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download");

    // Parse Range header: "bytes=start-end"
    let range_str = range_header.trim_start_matches("bytes=");
    let (start, end) = parse_range(range_str, file_len)?;

    // Calculate content length for this range
    let content_length = end - start + 1;

    // Read the requested byte range
    use tokio::io::{AsyncReadExt, AsyncSeekExt};
    let mut file = file;
    file.seek(std::io::SeekFrom::Start(start)).await?;

    let mut buffer = vec![0u8; content_length as usize];
    file.read_exact(&mut buffer).await?;

    // Detect MIME type from extension
    let mime_type = get_mime_type(file_name);

    // Generate ETag from modified time and size
    let etag = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| format!("\"{}-{}\"", d.as_secs(), file_len))
        .unwrap_or_else(|| format!("\"{}\"", file_len));

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, mime_type.parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("inline; filename=\"{}\"", file_name).parse().unwrap(),
    );
    headers.insert(header::ACCEPT_RANGES, "bytes".parse().unwrap());
    headers.insert(header::ETAG, etag.parse().unwrap());
    headers.insert(header::CACHE_CONTROL, "no-cache".parse().unwrap());

    if let Ok(modified) = metadata.modified() {
        headers.insert(
            header::LAST_MODIFIED,
            httpdate::fmt_http_date(modified).parse().unwrap(),
        );
    }

    // Content-Range: bytes start-end/total
    headers.insert(
        header::CONTENT_RANGE,
        format!("bytes {}-{}/{}", start, end, file_len).parse().unwrap(),
    );

    headers.insert(
        header::CONTENT_LENGTH,
        content_length.to_string().parse().unwrap(),
    );

    Ok((StatusCode::PARTIAL_CONTENT, headers, buffer).into_response())
}

/// Parse Range header value
/// Supports formats: "0-1023", "1024-", "-1024"
fn parse_range(range_str: &str, file_len: u64) -> Result<(u64, u64), FsvError> {
    let parts: Vec<&str> = range_str.split('-').collect();

    if parts.len() != 2 {
        return Err(FsvError::InvalidRange);
    }

    let start_str = parts[0].trim();
    let end_str = parts[1].trim();

    let (start, end) = if start_str.is_empty() {
        // Suffix range: "-1024" means last 1024 bytes
        let suffix_len: u64 = end_str.parse().map_err(|_| FsvError::InvalidRange)?;
        let start = file_len.saturating_sub(suffix_len);
        (start, file_len - 1)
    } else if end_str.is_empty() {
        // Open-ended range: "1024-" means from byte 1024 to end
        let start: u64 = start_str.parse().map_err(|_| FsvError::InvalidRange)?;
        (start, file_len - 1)
    } else {
        // Full range: "0-1023"
        let start: u64 = start_str.parse().map_err(|_| FsvError::InvalidRange)?;
        let end: u64 = end_str.parse().map_err(|_| FsvError::InvalidRange)?;
        (start, end)
    };

    // Validate range
    if start >= file_len || end >= file_len || start > end {
        return Err(FsvError::InvalidRange);
    }

    Ok((start, end))
}

/// HEAD - get file metadata without body
async fn webdav_head(
    state: &AppState,
    rel_path: Option<&str>,
) -> Result<Response, FsvError> {
    let target = resolve_safe_path(&state.root_path, rel_path)?;

    if target.is_dir() {
        return Ok((StatusCode::OK, "").into_response());
    }

    let metadata = tokio::fs::metadata(&target).await?;
    let file_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download");

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/octet-stream".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", file_name).parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_LENGTH,
        metadata.len().to_string().parse().unwrap(),
    );

    Ok((StatusCode::OK, headers, "").into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_range() {
        // Full range
        assert_eq!(parse_range("0-1023", 10000).unwrap(), (0, 1023));

        // Open-ended range
        assert_eq!(parse_range("1024-", 10000).unwrap(), (1024, 9999));

        // Suffix range
        assert_eq!(parse_range("-1024", 10000).unwrap(), (8976, 9999));

        // Invalid ranges
        assert!(parse_range("10000-", 10000).is_err());
        assert!(parse_range("100-50", 10000).is_err());
    }
}
