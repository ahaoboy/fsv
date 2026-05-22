import { useState, useEffect } from 'preact/hooks';
import { fileUrl } from '../api';
import { CloseIcon, DownloadIcon } from '../icons';
import type { FileInfo } from '../types';

interface Props {
  file: FileInfo;
  apiBase: string;
  onClose: () => void;
}

type PreviewKind = 'text' | 'image' | 'video' | 'audio' | 'unsupported';

const TEXT_EXTS = new Set(['rs', 'toml', 'json', 'md', 'txt', 'js', 'ts', 'tsx', 'jsx', 'css', 'html', 'htm', 'yaml', 'yml', 'sh', 'py', 'go', 'c', 'cpp', 'h', 'java', 'rb', 'php', 'xml', 'svg', 'lock', 'gitignore', 'env']);
const IMAGE_EXTS = new Set(['png', 'jpg', 'jpeg', 'gif', 'webp', 'ico', 'bmp', 'avif']);
const VIDEO_EXTS = new Set(['mp4', 'webm', 'ogg', 'mov']);
const AUDIO_EXTS = new Set(['mp3', 'wav', 'ogg', 'flac', 'aac', 'm4a']);

function getKind(name: string): PreviewKind {
  const ext = name.split('.').pop()?.toLowerCase() ?? '';
  if (TEXT_EXTS.has(ext)) return 'text';
  if (IMAGE_EXTS.has(ext)) return 'image';
  if (VIDEO_EXTS.has(ext)) return 'video';
  if (AUDIO_EXTS.has(ext)) return 'audio';
  return 'unsupported';
}

export function isPreviewable(name: string): boolean {
  return getKind(name) !== 'unsupported';
}

export function PreviewModal({ file, apiBase, onClose }: Props) {
  const [textContent, setTextContent] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const url = fileUrl(apiBase, file.path);
  const kind = getKind(file.name);

  useEffect(() => {
    if (kind !== 'text') return;
    setLoading(true);
    fetch(url)
      .then((r) => r.text())
      .then((t) => setTextContent(t))
      .catch((e) => setTextContent(`Error: ${e.message}`))
      .finally(() => setLoading(false));
  }, [url]);

  return (
    <div class="modal-overlay" onClick={onClose}>
      <div class="modal-card preview-modal" onClick={(e) => e.stopPropagation()}>
        <div class="modal-header">
          <span class="modal-title-text" title={file.name}>{file.name}</span>
          <div class="modal-header-actions">
            <a class="modal-action-btn" href={url} download={file.name} title="Download">
              <DownloadIcon size={16} />
            </a>
            <button class="modal-close" onClick={onClose} aria-label="Close">
              <CloseIcon size={18} />
            </button>
          </div>
        </div>

        <div class="modal-body preview-body">
          {kind === 'text' && (
            loading ? (
              <div class="preview-loading"><div class="spinner" /><p>Loading…</p></div>
            ) : (
              <pre class="code-block"><code>{textContent}</code></pre>
            )
          )}

          {kind === 'image' && (
            <div class="preview-media-wrap">
              <img src={url} alt={file.name} class="preview-image" />
            </div>
          )}

          {kind === 'video' && (
            <div class="preview-media-wrap">
              <video src={url} controls class="preview-video" />
            </div>
          )}

          {kind === 'audio' && (
            <div class="preview-media-wrap preview-audio-wrap">
              <audio src={url} controls class="preview-audio" />
            </div>
          )}

          {kind === 'unsupported' && (
            <div class="preview-unsupported">
              <p>Preview not available for this file type.</p>
              <a class="btn btn-primary" href={url} download={file.name}>Download instead</a>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
