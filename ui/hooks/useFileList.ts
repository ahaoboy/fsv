import { useState, useEffect } from 'preact/hooks';
import { listFiles } from '../api';
import type { FileInfo } from '../types';

export function useFileList(path: string, apiBase: string) {
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchFiles = async (p: string, base: string) => {
    setLoading(true);
    setError(null);
    try {
      setFiles(await listFiles(base, p));
    } catch (e: any) {
      setError(e.message || 'Failed to load files');
      setFiles([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchFiles(path, apiBase);
  }, [path, apiBase]);

  const refresh = () => fetchFiles(path, apiBase);

  return { files, loading, error, refresh };
}
