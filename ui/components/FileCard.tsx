import { FolderIcon, FileIcon, CodeIcon, ImageIcon, VideoIcon, DownloadIcon, EyeIcon, QrIcon } from '../icons';
import { fileUrl } from '../api';
import type { FileInfo } from '../types';
import { isPreviewable } from './PreviewModal';

interface Props {
  file: FileInfo;
  apiBase: string;
  onNavigate: (file: FileInfo) => void;
  onPreview: (file: FileInfo) => void;
  onQr: (file: FileInfo) => void;
}

const CODE_EXTS = new Set(['rs', 'js', 'ts', 'tsx', 'jsx', 'html', 'htm', 'css', 'json', 'toml', 'yaml', 'yml', 'md', 'sh', 'py', 'go', 'c', 'cpp', 'h', 'java', 'rb', 'php', 'xml', 'lock']);
const IMAGE_EXTS = new Set(['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'ico', 'bmp', 'avif']);
const VIDEO_EXTS = new Set(['mp4', 'webm', 'ogg', 'mov']);

function getIcon(file: FileInfo) {
  if (file.is_dir) return <FolderIcon class="file-type-icon folder" size={22} />;
  const ext = file.name.split('.').pop()?.toLowerCase() ?? '';
  if (IMAGE_EXTS.has(ext)) return <ImageIcon class="file-type-icon image" size={22} />;
  if (VIDEO_EXTS.has(ext)) return <VideoIcon class="file-type-icon video" size={22} />;
  if (CODE_EXTS.has(ext)) return <CodeIcon class="file-type-icon code" size={22} />;
  return <FileIcon class="file-type-icon" size={22} />;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

function formatDate(epoch: number | null): string {
  if (!epoch) return '';
  const d = new Date(epoch * 1000);
  return d.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
}

export function FileCard({ file, apiBase, onNavigate, onPreview, onQr }: Props) {
  const downloadUrl = fileUrl(apiBase, file.path);

  const handleRowClick = () => {
    if (file.is_dir) onNavigate(file);
  };

  const meta = file.is_dir
    ? 'Folder'
    : [formatBytes(file.size), formatDate(file.modified)].filter(Boolean).join(' · ');

  return (
    <div class={`file-card ${file.is_dir ? 'dir-card' : ''}`} onClick={handleRowClick} role={file.is_dir ? 'button' : undefined} tabIndex={file.is_dir ? 0 : undefined} onKeyDown={(e) => e.key === 'Enter' && handleRowClick()}>
      <div class="file-icon-wrap">
        {getIcon(file)}
      </div>

      <div class="file-info">
        <span class="file-name" title={file.name}>{file.name}</span>
        <span class="file-meta">{meta}</span>
      </div>

      {!file.is_dir && (
        <div class="file-actions" onClick={(e) => e.stopPropagation()}>
          {isPreviewable(file.name) && (
            <button class="action-btn" title="Preview" onClick={() => onPreview(file)}>
              <EyeIcon size={16} />
            </button>
          )}
          <a class="action-btn" href={downloadUrl} download={file.name} title="Download" onClick={(e) => e.stopPropagation()}>
            <DownloadIcon size={16} />
          </a>
          <button class="action-btn" title="QR code link" onClick={() => onQr(file)}>
            <QrIcon size={16} />
          </button>
        </div>
      )}
    </div>
  );
}
