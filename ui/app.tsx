import { useState, useEffect, useRef } from 'preact/hooks';
import './app.css';

interface FileInfo {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  modified: number | null;
}

// Inline SVG icons for premium look and zero external dependencies
const FolderIcon = () => (
  <svg class="file-icon folder" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
  </svg>
);

const FileIcon = () => (
  <svg class="file-icon file" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
    <polyline points="14 2 14 8 20 8"></polyline>
  </svg>
);

const CodeIcon = () => (
  <svg class="file-icon code" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <polyline points="16 18 22 12 16 6"></polyline>
    <polyline points="8 6 2 12 8 18"></polyline>
  </svg>
);

const ImageIcon = () => (
  <svg class="file-icon image" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
    <circle cx="8.5" cy="8.5" r="1.5"></circle>
    <polyline points="21 15 16 10 5 21"></polyline>
  </svg>
);

const DownloadIcon = () => (
  <svg class="action-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
    <polyline points="7 10 12 15 17 10"></polyline>
    <line x1="12" y1="15" x2="12" y2="3"></line>
  </svg>
);

const EyeIcon = () => (
  <svg class="action-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
    <circle cx="12" cy="12" r="3"></circle>
  </svg>
);

const CloseIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="18" height="18">
    <line x1="18" y1="6" x2="6" y2="18"></line>
    <line x1="6" y1="6" x2="18" y2="18"></line>
  </svg>
);

const RefreshIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="16" height="16">
    <polyline points="23 4 23 10 17 10"></polyline>
    <polyline points="1 20 1 14 7 14"></polyline>
    <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
  </svg>
);

export function App() {
  const [currentPath, setCurrentPath] = useState<string>('');
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState<string>('');
  
  // Real-time server broadcast logs
  const [wsMessages, setWsMessages] = useState<Array<{ time: string; msg: string }>>([]);
  const [wsStatus, setWsStatus] = useState<'connecting' | 'connected' | 'disconnected'>('disconnected');
  
  // Custom API configuration
  const [apiBase, setApiBase] = useState<string>('/');
  const [showSettings, setShowSettings] = useState<boolean>(false);
  const [settingsInput, setSettingsInput] = useState<string>('http://127.0.0.1:8888');

  // File preview modal
  const [previewFile, setPreviewFile] = useState<FileInfo | null>(null);
  const [previewLoading, setPreviewLoading] = useState<boolean>(false);
  const [previewContent, setPreviewContent] = useState<string | null>(null);

  const wsRef = useRef<WebSocket | null>(null);

  // Load API base from localStorage if saved
  useEffect(() => {
    const savedApi = localStorage.getItem('fsv_api_base');
    if (savedApi) {
      setApiBase(savedApi);
      setSettingsInput(savedApi);
    }
  }, []);

  // Fetch file list
  const fetchFiles = async (path: string, base: string = apiBase) => {
    setLoading(true);
    setError(null);
    try {
      const cleanBase = base.endsWith('/') ? base.slice(0, -1) : base;
      const url = `${cleanBase}/api/files?path=${encodeURIComponent(path)}`;
      const response = await fetch(url);
      if (!response.ok) {
        const errData = await response.json().catch(() => ({}));
        throw new Error(errData.error || `HTTP Error ${response.status}`);
      }
      const data = await response.json();
      setFiles(data);
    } catch (e: any) {
      setError(e.message || 'Failed to load files');
      setFiles([]);
    } finally {
      setLoading(false);
    }
  };

  // Re-fetch when path or apiBase changes
  useEffect(() => {
    fetchFiles(currentPath);
  }, [currentPath, apiBase]);

  // Connect WebSocket for live logs
  useEffect(() => {
    if (wsRef.current) {
      wsRef.current.close();
    }

    setWsStatus('connecting');

    // Resolve WS address based on API Base URL
    let wsUrl = '';
    if (apiBase === '/') {
      const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      wsUrl = `${proto}//${window.location.host}/ws`;
    } else {
      // Convert http:// to ws://
      wsUrl = apiBase.replace(/^http/, 'ws').replace(/\/$/, '') + '/ws';
    }

    try {
      const ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        setWsStatus('connected');
        setWsMessages(prev => [{ time: new Date().toLocaleTimeString(), msg: 'Connected to server WebSocket' }, ...prev]);
      };

      ws.onmessage = (event) => {
        setWsMessages(prev => [
          { time: new Date().toLocaleTimeString(), msg: event.data },
          ...prev.slice(0, 49) // Keep last 50 logs
        ]);
      };

      ws.onclose = () => {
        setWsStatus('disconnected');
      };

      ws.onerror = () => {
        setWsStatus('disconnected');
      };
    } catch (err) {
      setWsStatus('disconnected');
    }

    return () => {
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [apiBase]);

  // Handle settings save
  const saveSettings = () => {
    localStorage.setItem('fsv_api_base', settingsInput);
    setApiBase(settingsInput);
    setShowSettings(false);
  };

  // Formats file sizes nicely
  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  // Format epoch seconds to readable date
  const formatDate = (epochSecs: number | null) => {
    if (!epochSecs) return '-';
    const d = new Date(epochSecs * 1000);
    return d.toLocaleString();
  };

  // Decide which file icon to use
  const getFileIcon = (file: FileInfo) => {
    if (file.is_dir) return <FolderIcon />;
    
    const ext = file.name.split('.').pop()?.toLowerCase();
    if (['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'ico'].includes(ext || '')) {
      return <ImageIcon />;
    }
    if (['rs', 'js', 'ts', 'tsx', 'jsx', 'html', 'css', 'json', 'toml', 'yaml', 'yml', 'md', 'sh', 'py', 'go'].includes(ext || '')) {
      return <CodeIcon />;
    }
    return <FileIcon />;
  };

  // Check if file is previewable (simple text files)
  const isPreviewable = (fileName: string) => {
    const ext = fileName.split('.').pop()?.toLowerCase();
    return ['rs', 'toml', 'json', 'md', 'txt', 'js', 'ts', 'tsx', 'css', 'html', 'yaml', 'yml', 'sh', 'py'].includes(ext || '');
  };

  // Enter folder
  const handleFolderClick = (file: FileInfo) => {
    if (file.is_dir) {
      setCurrentPath(file.path);
    }
  };

  // Open download link
  const triggerDownload = (file: FileInfo) => {
    const cleanBase = apiBase.endsWith('/') ? apiBase.slice(0, -1) : apiBase;
    const url = `${cleanBase}/api/download?path=${encodeURIComponent(file.path)}`;
    window.open(url, '_blank');
  };

  // Fetch and display preview
  const openPreview = async (file: FileInfo) => {
    setPreviewFile(file);
    setPreviewLoading(true);
    setPreviewContent(null);
    try {
      const cleanBase = apiBase.endsWith('/') ? apiBase.slice(0, -1) : apiBase;
      const url = `${cleanBase}/api/download?path=${encodeURIComponent(file.path)}`;
      const response = await fetch(url);
      if (!response.ok) throw new Error('Failed to load preview');
      const text = await response.text();
      setPreviewContent(text);
    } catch (e: any) {
      setPreviewContent(`Error loading file preview: ${e.message}`);
    } finally {
      setPreviewLoading(false);
    }
  };

  // Segment breadcrumbs
  const getBreadcrumbs = () => {
    const segments = currentPath.split('/').filter(x => x);
    const crumbs = [{ name: 'Root', path: '' }];
    
    let pathAcc = '';
    segments.forEach(seg => {
      pathAcc = pathAcc ? `${pathAcc}/${seg}` : seg;
      crumbs.push({ name: seg, path: pathAcc });
    });
    
    return crumbs;
  };

  // Filter local file list by search input
  const filteredFiles = files.filter(f => 
    f.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div class="fsv-layout">
      {/* Sidebar for settings and real-time logs */}
      <aside class="sidebar">
        <div class="brand">
          <div class="logo-badge">FSV</div>
          <h2>File Share Viewer</h2>
        </div>

        {/* Server status indicator */}
        <div class="connection-status">
          <div class={`indicator ${wsStatus}`}></div>
          <span class="status-text">
            {wsStatus === 'connected' ? 'Connected to WebSocket' : 
             wsStatus === 'connecting' ? 'Connecting to Server...' : 'Backend Disconnected'}
          </span>
          <button class="settings-trigger" onClick={() => { setSettingsInput(apiBase); setShowSettings(true); }}>
            ⚙️
          </button>
        </div>

        {/* WebSocket broadcast listener feed */}
        <div class="ws-feed">
          <h3>Real-time Server Feed</h3>
          <p class="feed-sub">Logs broadcasts from Rust CLI</p>
          <div class="log-container">
            {wsMessages.length === 0 ? (
              <div class="empty-logs">No broadcast messages yet. Use the Rust command-line 'broadcast &lt;msg&gt;' to push messages here live.</div>
            ) : (
              wsMessages.map((item, idx) => (
                <div key={idx} class="log-item">
                  <span class="log-time">[{item.time}]</span>
                  <span class="log-text">{item.msg}</span>
                </div>
              ))
            )}
          </div>
        </div>
      </aside>

      {/* Main File Browser Area */}
      <main class="main-content">
        {/* Top Control Bar */}
        <header class="top-bar">
          <div class="breadcrumb-container">
            {getBreadcrumbs().map((crumb, idx, arr) => (
              <span key={idx} class="breadcrumb-item">
                <button class="crumb-btn" onClick={() => setCurrentPath(crumb.path)}>
                  {crumb.name}
                </button>
                {idx < arr.length - 1 && <span class="crumb-separator">/</span>}
              </span>
            ))}
          </div>

          <div class="actions-group">
            <div class="search-box">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
                <circle cx="11" cy="11" r="8"></circle>
                <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
              </svg>
              <input 
                type="text" 
                placeholder="Search current folder..." 
                value={searchQuery}
                onInput={(e) => setSearchQuery((e.target as HTMLInputElement).value)}
              />
            </div>
            
            <button class="icon-btn" title="Refresh files" onClick={() => fetchFiles(currentPath)}>
              <RefreshIcon />
            </button>
          </div>
        </header>

        {/* File Browser Grid */}
        <div class="browser-body">
          {loading ? (
            <div class="state-container">
              <div class="spinner"></div>
              <p>Scanning directory contents...</p>
            </div>
          ) : error ? (
            <div class="state-container error-state">
              <div class="error-icon">⚠️</div>
              <h3>API Connection Error</h3>
              <p>{error}</p>
              <button class="btn btn-primary" onClick={() => fetchFiles(currentPath)}>Try Again</button>
            </div>
          ) : filteredFiles.length === 0 ? (
            <div class="state-container">
              <div class="empty-icon">📁</div>
              <h3>No items found</h3>
              <p>{searchQuery ? 'No files match your search criteria.' : 'This directory is empty.'}</p>
            </div>
          ) : (
            <div class="file-grid">
              {filteredFiles.map((file) => (
                <div 
                  key={file.path} 
                  class={`file-card ${file.is_dir ? 'dir-card' : ''}`}
                  onClick={() => file.is_dir && handleFolderClick(file)}
                >
                  <div class="file-icon-wrap">
                    {getFileIcon(file)}
                  </div>
                  <div class="file-info-wrap">
                    <span class="file-name" title={file.name}>{file.name}</span>
                    <span class="file-meta">
                      {file.is_dir ? 'Folder' : formatBytes(file.size)}
                      {!file.is_dir && file.modified && ` • ${formatDate(file.modified).split(',')[0]}`}
                    </span>
                  </div>
                  <div class="file-actions" onClick={(e) => e.stopPropagation()}>
                    {!file.is_dir && isPreviewable(file.name) && (
                      <button class="action-btn" title="Preview File" onClick={() => openPreview(file)}>
                        <EyeIcon />
                      </button>
                    )}
                    {!file.is_dir && (
                      <button class="action-btn" title="Download File" onClick={() => triggerDownload(file)}>
                        <DownloadIcon />
                      </button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </main>

      {/* Code / Text File Preview Modal */}
      {previewFile && (
        <div class="modal-overlay" onClick={() => setPreviewFile(null)}>
          <div class="modal-card preview-modal" onClick={(e) => e.stopPropagation()}>
            <div class="modal-header">
              <div class="modal-title">
                {getFileIcon(previewFile)}
                <h3>Preview: {previewFile.name}</h3>
              </div>
              <button class="modal-close" onClick={() => setPreviewFile(null)}>
                <CloseIcon />
              </button>
            </div>
            <div class="modal-body preview-body">
              {previewLoading ? (
                <div class="preview-loading">
                  <div class="spinner"></div>
                  <p>Fetching content...</p>
                </div>
              ) : (
                <pre class="code-block">
                  <code>{previewContent}</code>
                </pre>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Connection Settings Modal */}
      {showSettings && (
        <div class="modal-overlay" onClick={() => setShowSettings(false)}>
          <div class="modal-card settings-modal" onClick={(e) => e.stopPropagation()}>
            <div class="modal-header">
              <h3>Backend API Settings</h3>
              <button class="modal-close" onClick={() => setShowSettings(false)}>
                <CloseIcon />
              </button>
            </div>
            <div class="modal-body">
              <div class="form-group">
                <label>Rust Server URL Base</label>
                <input 
                  type="text" 
                  placeholder="e.g. http://127.0.0.1:8888" 
                  value={settingsInput}
                  onInput={(e) => setSettingsInput((e.target as HTMLInputElement).value)}
                />
                <p class="form-help">Use "/" (default) to proxy requests via Vite in development, or set the explicit URL of your running fsv instance.</p>
              </div>
              <div class="modal-actions">
                <button class="btn btn-secondary" onClick={() => setShowSettings(false)}>Cancel</button>
                <button class="btn btn-primary" onClick={saveSettings}>Save Configuration</button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
