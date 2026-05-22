import type { FileInfo } from './types';

function base(apiBase: string): string {
  return apiBase.endsWith('/') ? apiBase.slice(0, -1) : apiBase;
}

/** Fetch directory listing for the given path. */
export async function listFiles(apiBase: string, path: string): Promise<FileInfo[]> {
  const url = `${base(apiBase)}/api/list?path=${encodeURIComponent(path)}`;
  const res = await fetch(url);
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error((err as any).error || `HTTP ${res.status}`);
  }
  return res.json();
}

/** Build the URL for downloading / streaming a file. */
export function fileUrl(apiBase: string, path: string): string {
  return `${base(apiBase)}/api/file?path=${encodeURIComponent(path)}`;
}
