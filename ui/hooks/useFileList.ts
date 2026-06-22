import { useState, useEffect, useCallback } from "react";
import { listFiles } from "../api";
import type { FileInfo } from "../types";

/** Hook to fetch and manage the file list for a given path. */
export function useFileList(path: string, apiBase: string) {
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchFiles = useCallback(async (p: string, base: string) => {
    setLoading(true);
    setError(null);
    try {
      const result = await listFiles(base, p);
      setFiles(result);
    } catch (e: unknown) {
      const message = e instanceof Error ? e.message : "Failed to load files";
      setError(message);
      setFiles([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchFiles(path, apiBase);
  }, [path, apiBase, fetchFiles]);

  const refresh = useCallback(() => fetchFiles(path, apiBase), [path, apiBase, fetchFiles]);

  return { files, loading, error, refresh };
}
