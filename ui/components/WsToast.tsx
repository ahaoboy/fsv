import { useEffect, useRef } from 'react';
import { Snackbar, Alert, IconButton, Box } from '@mui/material';
import {
  Close as CloseIcon,
  ContentCopy as CopyIcon,
} from '@mui/icons-material';
import { copyToClipboard } from '../api';

interface Props {
  message: string;
  onClose: () => void;
}

/** Toast notification for WebSocket broadcast messages. */
export function WsToast({ message, onClose }: Props) {
  const copyBtnRef = useRef<HTMLButtonElement>(null);

  // Auto-dismiss after 8 seconds
  useEffect(() => {
    const t = setTimeout(onClose, 8000);
    return () => clearTimeout(t);
  }, [message, onClose]);

  // Auto-focus copy button when toast appears
  useEffect(() => {
    copyBtnRef.current?.focus();
  }, []);

  return (
    <Snackbar
      open
      anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
      sx={{ maxWidth: 420, width: 'calc(100vw - 32px)' }}
    >
      <Alert
        severity="info"
        variant="filled"
        sx={{
          width: '100%',
          alignItems: 'center',
          '& .MuiAlert-message': { flex: 1, minWidth: 0 },
        }}
        action={
          <Box sx={{ display: 'flex', gap: 0.25 }}>
            <IconButton
              ref={copyBtnRef}
              size="small"
              color="inherit"
              onClick={() => copyToClipboard(message)}
              title="Copy to clipboard"
            >
              <CopyIcon fontSize="small" />
            </IconButton>
            <IconButton
              size="small"
              color="inherit"
              onClick={onClose}
              title="Dismiss"
            >
              <CloseIcon fontSize="small" />
            </IconButton>
          </Box>
        }
      >
        <Box sx={{ fontSize: 13, wordBreak: 'break-word', lineHeight: 1.4 }}>
          {message}
        </Box>
      </Alert>
    </Snackbar>
  );
}

