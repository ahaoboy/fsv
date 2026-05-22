import { useState, useEffect } from 'preact/hooks';
import QRCodeLib from 'qrcode';
import { CloseIcon } from '../icons';

interface Props {
  url: string;
  fileName: string;
  onClose: () => void;
}

async function generateQR(text: string): Promise<string> {
  return QRCodeLib.toDataURL(text, {
    width: 512,
  });
}

export function QrModal({ url, fileName, onClose }: Props) {
  const [dataUrl, setDataUrl] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setDataUrl(null);
    setError(null);
    generateQR(url)
      .then(setDataUrl)
      .catch((e) => setError(e.message ?? 'Failed to generate QR code'));
  }, [url]);

  return (
    <div class="modal-overlay" onClick={onClose}>
      <div class="modal-card qr-modal" onClick={(e) => e.stopPropagation()}>
        <div class="modal-header">
          <span class="modal-title-text">Scan to download</span>
          <button class="modal-close" onClick={onClose} aria-label="Close">
            <CloseIcon size={18} />
          </button>
        </div>
        <div class="modal-body qr-body">
          {error ? (
            <p class="qr-error">{error}</p>
          ) : dataUrl ? (
            <img src={dataUrl} alt="QR code" class="qr-img" width={256} height={256} />
          ) : (
            <div class="spinner" style="width:40px;height:40px" />
          )}
          <p class="qr-filename">{fileName}</p>
          <p class="qr-url">{url}</p>
        </div>
      </div>
    </div>
  );
}
