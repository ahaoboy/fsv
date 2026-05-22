import { useEffect, useRef } from 'preact/hooks';
import { CopyIcon, CloseIcon } from '../icons';
import { copyToClipboard } from '../api';

interface Props {
  message: string;
  onClose: () => void;
}

export function WsToast({ message, onClose }: Props) {
  const copyBtnRef = useRef<HTMLButtonElement>(null);

  // Auto-dismiss after 8 seconds
  useEffect(() => {
    const t = setTimeout(onClose, 8000);
    return () => clearTimeout(t);
  }, [message]);

  // Auto-focus copy button when toast appears
  useEffect(() => {
    copyBtnRef.current?.focus();
  }, []);

  return (
    <div class="ws-toast" role="alert">
      <div class="ws-toast-body">
        <span class="ws-toast-label">Server message</span>
        <p class="ws-toast-text">{message}</p>
      </div>
      <div class="ws-toast-actions">
        <button 
          ref={copyBtnRef}
          class="ws-toast-btn" 
          onClick={() => copyToClipboard(message)} 
          title="Copy to clipboard"
        >
          <CopyIcon size={15} />
        </button>
        <button class="ws-toast-btn" onClick={onClose} title="Dismiss">
          <CloseIcon size={15} />
        </button>
      </div>
    </div>
  );
}
