import { useEffect } from 'preact/hooks';
import { CopyIcon, CloseIcon } from '../icons';

interface Props {
  message: string;
  onClose: () => void;
}

export function WsToast({ message, onClose }: Props) {
  // Auto-dismiss after 8 seconds
  useEffect(() => {
    const t = setTimeout(onClose, 8000);
    return () => clearTimeout(t);
  }, [message]);

  const copy = () => {
    navigator.clipboard.writeText(message).catch((e) => { console.error('Failed to copy WebSocket message:', e); });
  };

  useEffect(() => {
    if (message) {
      console.log('WebSocket message:', message);
      copy();
    }
  }, [message]);

  return (
    <div class="ws-toast" role="alert">
      <div class="ws-toast-body">
        <span class="ws-toast-label">Server message</span>
        <p class="ws-toast-text">{message}</p>
      </div>
      <div class="ws-toast-actions">
        <button class="ws-toast-btn" onClick={copy} title="Copy to clipboard">
          <CopyIcon size={15} />
        </button>
        <button class="ws-toast-btn" onClick={onClose} title="Dismiss">
          <CloseIcon size={15} />
        </button>
      </div>
    </div>
  );
}
