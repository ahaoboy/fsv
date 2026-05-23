import { useState, useEffect } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  IconButton,
  Box,
  Typography,
  CircularProgress,
  Button,
  Link,
} from '@mui/material';
import {
  Close as CloseIcon,
  Download as DownloadIcon,
} from '@mui/icons-material';
import { fileUrl } from '../api';
import type { FileInfo } from '../types';

interface Props {
  file: FileInfo;
  apiBase: string;
  onClose: () => void;
}

type PreviewKind = 'text' | 'image' | 'video' | 'audio' | 'unsupported';

/** File extension sets for preview type detection. */
const TEXT_EXTS = new Set([
  'rs', 'toml', 'json', 'md', 'txt', 'js', 'ts', 'tsx', 'jsx',
  'css', 'html', 'htm', 'yaml', 'yml', 'sh', 'py', 'go', 'c',
  'cpp', 'h', 'java', 'rb', 'php', 'xml', 'svg', 'lock', 'gitignore', 'env',
]);
const IMAGE_EXTS = new Set([
  'png', 'jpg', 'jpeg', 'gif', 'webp', 'ico', 'bmp', 'avif',
]);
const VIDEO_EXTS = new Set(['mp4', 'webm', 'ogg', 'mov']);
const AUDIO_EXTS = new Set(['mp3', 'wav', 'ogg', 'flac', 'aac', 'm4a']);

/** Determine the preview kind from a filename. */
function getKind(name: string): PreviewKind {
  const ext = name.split('.').pop()?.toLowerCase() ?? '';
  if (TEXT_EXTS.has(ext)) return 'text';
  if (IMAGE_EXTS.has(ext)) return 'image';
  if (VIDEO_EXTS.has(ext)) return 'video';
  if (AUDIO_EXTS.has(ext)) return 'audio';
  return 'unsupported';
}

/** Check if a file can be previewed. */
export function isPreviewable(name: string): boolean {
  return getKind(name) !== 'unsupported';
}

/** Modal dialog for previewing file content (text, image, video, audio). */
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
      .catch((e: Error) => setTextContent(`Error: ${e.message}`))
      .finally(() => setLoading(false));
  }, [url, kind]);

  return (
    <Dialog
      open
      onClose={onClose}
      maxWidth="md"
      fullWidth
      slotProps={{
        paper: { sx: { height: { xs: '100%', sm: '80vh' }, m: { xs: 0, sm: 2 } } },
      }}
    >
      {/* Title bar */}
      <DialogTitle
        sx={{
          display: 'flex',
          alignItems: 'center',
          gap: 1,
          pr: 1,
        }}
      >
        <Typography variant="body1" sx={{ fontWeight: 500, flex: 1 }} noWrap>
          {file.name}
        </Typography>
        <Link href={url} download={file.name} sx={{ display: 'flex' }}>
          <IconButton size="small" component="span" title="Download">
            <DownloadIcon fontSize="small" />
          </IconButton>
        </Link>
        <IconButton size="small" onClick={onClose} aria-label="Close">
          <CloseIcon fontSize="small" />
        </IconButton>
      </DialogTitle>

      {/* Body */}
      <DialogContent
        dividers
        sx={{
          p: 0,
          bgcolor: 'background.paper',
          display: 'flex',
          flexDirection: 'column',
          overflow: 'hidden',
        }}
      >
        {/* Text preview */}
        {kind === 'text' &&
          (loading ? (
            <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 2, p: 3 }}>
              <CircularProgress size={32} />
              <Typography variant="body2">Loading…</Typography>
            </Box>
          ) : (
            <Box
              component="pre"
              sx={{
                flex: 1,
                m: 0,
                overflow: 'auto',
                p: 2,
                fontSize: 12,
                lineHeight: 1.6,
                fontFamily: 'monospace',
                whiteSpace: 'pre',
                color: 'text.primary',
              }}
            >
              <code>{textContent}</code>
            </Box>
          ))}

        {/* Image preview */}
        {kind === 'image' && (
          <Box
            sx={{
              flex: 1,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              p: 2,
              overflow: 'auto',
            }}
          >
            <Box
              component="img"
              src={url}
              alt={file.name}
              sx={{ maxWidth: '100%', maxHeight: '100%', objectFit: 'contain', borderRadius: 1 }}
            />
          </Box>
        )}

        {/* Video preview */}
        {kind === 'video' && (
          <Box
            sx={{
              flex: 1,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              p: 2,
              overflow: 'auto',
            }}
          >
            <Box
              component="video"
              src={url}
              controls
              sx={{ maxWidth: '100%', maxHeight: '100%', borderRadius: 1 }}
            />
          </Box>
        )}

        {/* Audio preview */}
        {kind === 'audio' && (
          <Box
            sx={{
              flex: 1,
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              p: 4,
            }}
          >
            <Box component="audio" src={url} controls sx={{ width: '100%' }} />
          </Box>
        )}

        {/* Unsupported type */}
        {kind === 'unsupported' && (
          <Box
            sx={{
              flex: 1,
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              gap: 2,
              p: 4,
              textAlign: 'center',
            }}
          >
            <Typography variant="body2">
              Preview not available for this file type.
            </Typography>
            <Button variant="contained" href={url} download={file.name}>
              Download instead
            </Button>
          </Box>
        )}
      </DialogContent>
    </Dialog>
  );
}

