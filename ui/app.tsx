import { useState } from 'preact/hooks';
import { useFileList } from './hooks/useFileList';
import { useWebSocket } from './hooks/useWebSocket';
import { copyToClipboard, fileUrl } from './api';
import { FileCard } from './components/FileCard';
import { PreviewModal } from './components/PreviewModal';
import { QrModal } from './components/QrModal';
import { WsToast } from './components/WsToast';
import { SettingsModal } from './components/SettingsModal';
import { SearchIcon, RefreshIcon, SettingsIcon } from './icons';
import type { FileInfo } from './types';
import './app.css';

function getBreadcrumbs(path: string) {
  const segments = path.split('/').filter(Boolean);
  const crumbs = [{ name: 'Root', path: '' }];
  let acc = '';
  for (const seg of segments) {
    acc = acc ? `${acc}/${seg}` : seg;
    crumbs.push({ name: seg, path: acc });
  }
  return crumbs;
}

export function App() {
  const [currentPath, setCurrentPath] = useState('');
  const [search, setSearch] = useState('');
  const [apiBase, setApiBase] = useState(() => localStorage.getItem('fsv_api_base') ?? '/');

  const [previewFile, setPreviewFile] = useState<FileInfo | null>(null);
  const [qrFile, setQrFile] = useState<FileInfo | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [wsToast, setWsToast] = useState<string | null>(null);

  const { files, loading, error, refresh } = useFileList(currentPath, apiBase);

  const wsStatus = useWebSocket(apiBase, (msg) => {
    setWsToast(msg);
    copyToClipboard(msg)
  });

  const filtered = files.filter((f) =>
    f.name.toLowerCase().includes(search.toLowerCase())
  );

  const getQrUrl = (file: FileInfo) => fileUrl(apiBase, file.path);

  return (
    <div class="fsv-app">
      {/* ── Header ── */}
      <header class="app-header">
        <div class="header-top">
          <div class="brand">
            <span class="brand-badge">FSV</span>
            <span class="brand-name">File Share Viewer</span>
          </div>
          <div class="header-controls">
            <div class={`ws-dot ${wsStatus}`} title={`WebSocket: ${wsStatus}`} />
            <button class="icon-btn" title="Settings" onClick={() => setShowSettings(true)}>
              <SettingsIcon size={18} />
            </button>
          </div>
        </div>

        {/* Breadcrumb */}
        <nav class="breadcrumb" aria-label="Path navigation">
          {getBreadcrumbs(currentPath).map((crumb, i, arr) => (
            <span key={crumb.path} class="crumb-item">
              <button class="crumb-btn" onClick={() => { setCurrentPath(crumb.path); setSearch(''); }}>
                {crumb.name}
              </button>
              {i < arr.length - 1 && <span class="crumb-sep" aria-hidden="true">/</span>}
            </span>
          ))}
        </nav>

        {/* Search + Refresh */}
        <div class="search-row">
          <div class="search-box">
            <SearchIcon size={15} />
            <input
              type="search"
              placeholder="Filter files…"
              value={search}
              onInput={(e) => setSearch((e.target as HTMLInputElement).value)}
              aria-label="Filter files"
            />
          </div>
          <button class="icon-btn" title="Refresh" onClick={refresh}>
            <RefreshIcon size={16} />
          </button>
        </div>
      </header>

      {/* ── File List ── */}
      <main class="file-list">
        {loading && (
          <div class="state-view">
            <div class="spinner" />
            <p>Loading…</p>
          </div>
        )}

        {!loading && error && (
          <div class="state-view error-view">
            <span class="state-icon">⚠️</span>
            <p>{error}</p>
            <button class="btn btn-primary" onClick={refresh}>Retry</button>
          </div>
        )}

        {!loading && !error && filtered.length === 0 && (
          <div class="state-view">
            <span class="state-icon">📁</span>
            <p>{search ? 'No files match your search.' : 'This folder is empty.'}</p>
          </div>
        )}

        {!loading && !error && filtered.map((file) => (
          <FileCard
            key={file.path}
            file={file}
            apiBase={apiBase}
            onNavigate={(f) => { setCurrentPath(f.path); setSearch(''); }}
            onPreview={setPreviewFile}
            onQr={setQrFile}
          />
        ))}
      </main>

      {/* ── Modals ── */}
      {previewFile && (
        <PreviewModal
          file={previewFile}
          apiBase={apiBase}
          onClose={() => setPreviewFile(null)}
        />
      )}

      {qrFile && (
        <QrModal
          url={getQrUrl(qrFile)}
          fileName={qrFile.name}
          onClose={() => setQrFile(null)}
        />
      )}

      {showSettings && (
        <SettingsModal
          current={apiBase}
          onSave={setApiBase}
          onClose={() => setShowSettings(false)}
        />
      )}

      {/* ── WS Toast ── */}
      {wsToast && (
        <WsToast message={wsToast} onClose={() => setWsToast(null)} />
      )}
    </div>
  );
}
