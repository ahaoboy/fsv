import {
  Card,
  CardActionArea,
  Box,
  Typography,
  IconButton,
  Tooltip,
  Link as MuiLink,
} from '@mui/material';
import {
  Folder as FolderIcon,
  InsertDriveFile as FileIcon,
  Code as CodeIcon,
  Image as ImageIcon,
  Videocam as VideoIcon,
  Download as DownloadIcon,
  Visibility as EyeIcon,
  QrCode as QrIcon,
  ContentCopy as CopyIcon,
} from '@mui/icons-material';
import { fileUrl, copyToClipboard } from '../api';
import type { FileInfo } from '../types';
import { isPreviewable } from './PreviewModal';
import { type JSX } from 'react';
import prettyBytes from 'pretty-bytes';

interface Props {
  file: FileInfo;
  apiBase: string;
  onNavigate: (file: FileInfo) => void;
  onPreview: (file: FileInfo) => void;
  onQr: (file: FileInfo) => void;
}

/** File extensions grouped by type for icon selection. */
const CODE_EXTS = new Set([
  'rs', 'js', 'ts', 'tsx', 'jsx', 'html', 'htm', 'css', 'json',
  'toml', 'yaml', 'yml', 'md', 'sh', 'py', 'go', 'c', 'cpp', 'h',
  'java', 'rb', 'php', 'xml', 'lock',
]);
const IMAGE_EXTS = new Set([
  'png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'ico', 'bmp', 'avif',
]);
const VIDEO_EXTS = new Set(['mp4', 'webm', 'ogg', 'mov']);

/** Pick the appropriate icon for a file based on its extension. */
function getIcon(file: FileInfo): JSX.Element {
  if (file.is_dir) {
    return <FolderIcon sx={{ color: '#a855f7' }} fontSize="small" />;
  }
  const ext = file.name.split('.').pop()?.toLowerCase() ?? '';
  if (IMAGE_EXTS.has(ext)) return <ImageIcon sx={{ color: '#10b981' }} fontSize="small" />;
  if (VIDEO_EXTS.has(ext)) return <VideoIcon sx={{ color: '#f59e0b' }} fontSize="small" />;
  if (CODE_EXTS.has(ext)) return <CodeIcon sx={{ color: '#3b82f6' }} fontSize="small" />;
  return <FileIcon color="action" fontSize="small" />;
}

/** Format a Unix timestamp into a fixed-width date string (YYYY-MM-DD). */
function formatDate(epoch: number | null): string {
  if (!epoch) return '';
  const d = new Date(epoch * 1000);
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

/** A single file or directory row in the file list. */
export function FileCard({ file, apiBase, onNavigate, onPreview, onQr }: Props) {
  const downloadUrl = fileUrl(apiBase, file.path);

  const handleRowClick = () => {
    if (file.is_dir) onNavigate(file);
  };

  const handleCopy = () => {
    let fullUrl = downloadUrl;
    if (downloadUrl.startsWith('/')) {
      fullUrl = `${window.location.origin}${downloadUrl}`;
    }
    copyToClipboard(fullUrl);
  };

  const meta = file.is_dir
    ? 'Folder'
    : [formatDate(file.modified), prettyBytes(file.size)].filter(Boolean).join(' · ');

  // Shared icon and name blocks used by both directory and file layouts
  const iconBox = (
    <Box
      sx={{
        width: 38,
        height: 38,
        borderRadius: 2.5,
        bgcolor: 'background.paper',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        flexShrink: 0,
      }}
    >
      {getIcon(file)}
    </Box>
  );

  const nameBox = (
    <Box sx={{ flex: 1, minWidth: 0 }}>
      <Typography
        variant="body2"
        sx={{ fontWeight: 500 }}
        noWrap
        title={file.name}
      >
        {file.name}
      </Typography>
      <Typography variant="caption" color="text.secondary" noWrap>
        {meta}
      </Typography>
    </Box>
  );

  return (
    <Card
      elevation={0}
      sx={{
        borderBottom: 1,
        borderColor: 'divider',
        borderRadius: 0,
        bgcolor: 'background.default',
        '&:last-child': { borderBottom: 'none' },
      }}
    >
      {/* Directories: entire row is clickable via CardActionArea */}
      {file.is_dir ? (
        <CardActionArea
          onClick={handleRowClick}
          sx={{
            display: 'flex',
            alignItems: 'center',
            gap: 1.5,
            px: 2,
            py: 1.25,
            justifyContent: 'flex-start',
          }}
        >
          {iconBox}
          {nameBox}
        </CardActionArea>
      ) : (
        /* Files: plain row with icon, name, and action buttons */
        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            gap: 1.5,
            px: 2,
            py: 1.25,
          }}
        >
          {iconBox}
          {nameBox}

          {/* Action buttons */}
          <Box
            sx={{ display: 'flex', alignItems: 'center', gap: 0.25, flexShrink: 0 }}
          >
            {isPreviewable(file.name) && (
              <Tooltip title="Preview">
                <IconButton size="small" onClick={() => onPreview(file)}>
                  <EyeIcon fontSize="small" />
                </IconButton>
              </Tooltip>
            )}
            <Tooltip title="Download">
              <MuiLink
                href={downloadUrl}
                download={file.name}
                sx={{ display: 'flex', color: 'text.secondary' }}
              >
                <IconButton size="small" component="span">
                  <DownloadIcon fontSize="small" />
                </IconButton>
              </MuiLink>
            </Tooltip>
            <Tooltip title="Copy link">
              <IconButton size="small" onClick={handleCopy}>
                <CopyIcon fontSize="small" />
              </IconButton>
            </Tooltip>
            <Tooltip title="QR code">
              <IconButton size="small" onClick={() => onQr(file)}>
                <QrIcon fontSize="small" />
              </IconButton>
            </Tooltip>
          </Box>
        </Box>
      )}
    </Card>
  );
}

