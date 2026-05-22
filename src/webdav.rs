use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
};
use std::path::Path;
use tokio_util::io::ReaderStream;

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
    
    // Strip /webdav prefix
    let rel_path = path.strip_prefix("/webdav").unwrap_or(path);
    let rel_path = if rel_path.is_empty() || rel_path == "/" {
        None
    } else {
        Some(rel_path.trim_start_matches('/'))
    };

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
    let mut entries = vec![];
    
    // Add the directory itself
    let dir_meta = tokio::fs::metadata(dir).await?;
    let dir_name = if rel_path.is_empty() {
        "webdav"
    } else {
        dir.file_name().and_then(|n| n.to_str()).unwrap_or("")
    };
    
    entries.push(format_propfind_entry(
        rel_path,
        dir_name,
        true,
        0,
        dir_meta.modified().ok(),
    ));

    // Add directory contents
    let mut read_dir = tokio::fs::read_dir(dir).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let _path = entry.path();
        let metadata = entry.metadata().await?;
        let name = entry.file_name().to_string_lossy().to_string();
        
        let entry_path = if rel_path.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", rel_path, name)
        };

        entries.push(format_propfind_entry(
            &entry_path,
            &name,
            metadata.is_dir(),
            metadata.len(),
            metadata.modified().ok(),
        ));
    }

    Ok(format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:">
{}
</D:multistatus>"#,
        entries.join("\n")
    ))
}

/// Generate PROPFIND XML for a single file
async fn propfind_file(file: &Path, rel_path: &str) -> Result<String, FsvError> {
    let metadata = tokio::fs::metadata(file).await?;
    let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
    
    let entry = format_propfind_entry(
        rel_path,
        name,
        false,
        metadata.len(),
        metadata.modified().ok(),
    );

    Ok(format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:">
{}
</D:multistatus>"#,
        entry
    ))
}

/// Format a single PROPFIND entry
fn format_propfind_entry(
    href: &str,
    display_name: &str,
    is_dir: bool,
    size: u64,
    modified: Option<std::time::SystemTime>,
) -> String {
    // Build proper href path
    let href = if href.is_empty() {
        "/webdav/".to_string()
    } else {
        format!("/webdav/{}", href.trim_start_matches('/'))
    };
    
    // Add trailing slash for directories
    let href = if is_dir && !href.ends_with('/') {
        format!("{}/", href)
    } else {
        href
    };
    
    let resource_type = if is_dir {
        "<D:collection/>"
    } else {
        ""
    };
    
    let modified_str = modified
        .map(|t| httpdate::fmt_http_date(t))
        .unwrap_or_default();

    format!(
        r#"  <D:response>
    <D:href>{}</D:href>
    <D:propstat>
      <D:prop>
        <D:displayname>{}</D:displayname>
        <D:resourcetype>{}</D:resourcetype>
        <D:getcontentlength>{}</D:getcontentlength>
        <D:getlastmodified>{}</D:getlastmodified>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>"#,
        escape_xml(&href),
        escape_xml(display_name),
        resource_type,
        size,
        modified_str
    )
}

/// GET - download file
async fn webdav_get(
    state: &AppState,
    rel_path: Option<&str>,
) -> Result<Response, FsvError> {
    let target = resolve_safe_path(&state.root_path, rel_path)?;

    if target.is_dir() {
        return Err(FsvError::NotAFile);
    }

    let file = tokio::fs::File::open(&target).await?;
    let metadata = file.metadata().await?;
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

    Ok((headers, Body::from_stream(ReaderStream::new(file))).into_response())
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

/// Escape XML special characters
fn escape_xml<S: AsRef<str>>(s: S) -> String {
    // IMPORTANT: & must be replaced first to avoid double-escaping
    s.as_ref()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("test & <tag>"), "test &amp; &lt;tag&gt;");
        assert_eq!(escape_xml("a'b\"c"), "a&apos;b&quot;c");
    }
}
