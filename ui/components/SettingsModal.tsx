import { useState } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  IconButton,
  TextField,
  Typography,
  Button,
} from '@mui/material';
import { Close as CloseIcon } from '@mui/icons-material';

interface Props {
  current: string;
  onSave: (url: string) => void;
  onClose: () => void;
}

/** Settings modal for configuring the backend API base URL. */
export function SettingsModal({ current, onSave, onClose }: Props) {
  const [value, setValue] = useState(current);

  const handleSave = () => {
    localStorage.setItem('fsv_api_base', value);
    onSave(value);
    onClose();
  };

  return (
    <Dialog open onClose={onClose} maxWidth="xs" fullWidth>
      <DialogTitle sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
        <Typography variant="body1" sx={{ fontWeight: 500, flex: 1 }}>
          Backend Settings
        </Typography>
        <IconButton size="small" onClick={onClose} aria-label="Close">
          <CloseIcon fontSize="small" />
        </IconButton>
      </DialogTitle>

      <DialogContent>
        <TextField
          autoFocus
          fullWidth
          label="Server URL"
          placeholder="http://127.0.0.1:8888"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          size="small"
          margin="dense"
          slotProps={{ htmlInput: { id: 'api-url' } }}
        />
        <Typography variant="caption" color="text.secondary" sx={{ mt: 0.5, display: 'block' }}>
          Use <code>/</code> to proxy via Vite in dev mode, or enter the full URL of your fsv instance.
        </Typography>
      </DialogContent>

      <DialogActions sx={{ px: 3, pb: 2 }}>
        <Button variant="outlined" onClick={onClose}>
          Cancel
        </Button>
        <Button variant="contained" onClick={handleSave}>
          Save
        </Button>
      </DialogActions>
    </Dialog>
  );
}

