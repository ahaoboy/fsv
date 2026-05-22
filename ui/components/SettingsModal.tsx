import { useState } from 'preact/hooks';
import { CloseIcon } from '../icons';

interface Props {
  current: string;
  onSave: (url: string) => void;
  onClose: () => void;
}

export function SettingsModal({ current, onSave, onClose }: Props) {
  const [value, setValue] = useState(current);

  const save = () => {
    localStorage.setItem('fsv_api_base', value);
    onSave(value);
    onClose();
  };

  return (
    <div class="modal-overlay" onClick={onClose}>
      <div class="modal-card settings-modal" onClick={(e) => e.stopPropagation()}>
        <div class="modal-header">
          <span class="modal-title-text">Backend Settings</span>
          <button class="modal-close" onClick={onClose} aria-label="Close">
            <CloseIcon size={18} />
          </button>
        </div>
        <div class="modal-body">
          <div class="form-group">
            <label for="api-url">Server URL</label>
            <input
              id="api-url"
              type="text"
              placeholder="http://127.0.0.1:8888"
              value={value}
              onInput={(e) => setValue((e.target as HTMLInputElement).value)}
            />
            <p class="form-help">
              Use <code>/</code> to proxy via Vite in dev mode, or enter the full URL of your fsv instance.
            </p>
          </div>
          <div class="modal-actions">
            <button class="btn btn-secondary" onClick={onClose}>Cancel</button>
            <button class="btn btn-primary" onClick={save}>Save</button>
          </div>
        </div>
      </div>
    </div>
  );
}
