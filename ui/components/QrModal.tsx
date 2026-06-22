import { useState, useEffect } from "react";
import QRCodeLib from "qrcode";
import {
  Dialog,
  DialogTitle,
  DialogContent,
  IconButton,
  Box,
  Typography,
  CircularProgress,
} from "@mui/material";
import { Close as CloseIcon } from "@mui/icons-material";

interface Props {
  url: string;
  fileName: string;
  onClose: () => void;
}

/** Generate a QR code data URL using the qrcode library. */
async function generateQR(text: string): Promise<string> {
  return QRCodeLib.toDataURL(text, { width: 512 });
}

/** Modal dialog displaying a QR code for sharing a file URL. */
export function QrModal({ url, fileName, onClose }: Props) {
  const [dataUrl, setDataUrl] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setDataUrl(null);
    setError(null);
    generateQR(url)
      .then(setDataUrl)
      .catch((e: Error) => setError(e.message ?? "Failed to generate QR code"));
  }, [url]);

  return (
    <Dialog open onClose={onClose} maxWidth="xs" fullWidth>
      <DialogTitle sx={{ display: "flex", alignItems: "center", gap: 1 }}>
        <Typography variant="body1" sx={{ fontWeight: 500, flex: 1 }}>
          Scan to download
        </Typography>
        <IconButton size="small" onClick={onClose} aria-label="Close">
          <CloseIcon fontSize="small" />
        </IconButton>
      </DialogTitle>

      <DialogContent sx={{ textAlign: "center", py: 3 }}>
        {error ? (
          <Typography variant="body2" color="error">
            {error}
          </Typography>
        ) : dataUrl ? (
          <Box
            component="img"
            src={dataUrl}
            alt="QR code"
            width={256}
            height={256}
            sx={{ borderRadius: 1, display: "block", mx: "auto" }}
          />
        ) : (
          <Box sx={{ display: "flex", justifyContent: "center", py: 4 }}>
            <CircularProgress size={40} />
          </Box>
        )}

        <Typography variant="body2" sx={{ fontWeight: 500, mt: 2 }}>
          {fileName}
        </Typography>
        <Typography
          variant="caption"
          color="text.secondary"
          sx={{
            mt: 0.5,
            wordBreak: "break-all",
            fontFamily: "monospace",
            display: "block",
          }}
        >
          {url}
        </Typography>
      </DialogContent>
    </Dialog>
  );
}
