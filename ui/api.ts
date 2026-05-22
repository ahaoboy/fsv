import type { FileInfo } from './types';

function base(apiBase: string): string {
  return apiBase.endsWith('/') ? apiBase.slice(0, -1) : apiBase;
}

/** Fetch directory listing for the given path. */
export async function listFiles(apiBase: string, path: string): Promise<FileInfo[]> {
  const url = `${base(apiBase)}/list?path=${encodeURIComponent(path)}`;
  const res = await fetch(url, { method: 'POST' });
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error((err as any).error || `HTTP ${res.status}`);
  }
  return res.json();
}

/** Build the URL for downloading / streaming a file. */
export function fileUrl(apiBase: string, path: string): string {
  return `${base(apiBase)}/${path}`;
}

export function copyToClipboard(text: string): boolean {
  // Modern async clipboard API (requires HTTPS or localhost)
  if (navigator.clipboard && window.isSecureContext) {
    navigator.clipboard.writeText(text).catch(() => { });
    return true;
  }

  // Fallback: legacy execCommand with a hidden textarea
  try {
    const textarea = document.createElement('textarea');
    textarea.value = text;
    textarea.style.position = 'fixed';
    textarea.style.left = '-9999px';
    textarea.style.top = '-9999px';
    document.body.appendChild(textarea);
    textarea.select();
    const success = document.execCommand('copy');
    document.body.removeChild(textarea);
    return success;
  } catch {
    return false;
  }
}

/** WebSocket connection info. */
export interface WsInfo {
  connected: number;
  broadcast_capacity: number;
}

/** Fetch WebSocket connection statistics. */
export async function getWsInfo(apiBase: string): Promise<WsInfo> {
  const url = `${base(apiBase)}/ws-info`;
  const res = await fetch(url, { method: 'POST' });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json();
}

/** Health check response. */
export interface HealthStatus {
  status: string;
  timestamp: number;
}

/** Check server health. */
export async function checkHealth(apiBase: string): Promise<HealthStatus> {
  const url = `${base(apiBase)}/health`;
  const res = await fetch(url, { method: 'POST' });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json();
}
