import { useState, useMemo, useEffect, useCallback } from "react";
import { Box, Container, Typography, CircularProgress, Button } from "@mui/material";
import { useFileList } from "./hooks/useFileList";
import { useWebSocket } from "./hooks/useWebSocket";
import { copyToClipboard, fileUrl, shutdownServer } from "./api";
import { Header } from "./components/Header";
import { FileCard } from "./components/FileCard";
import { PreviewModal } from "./components/PreviewModal";
import { QrModal } from "./components/QrModal";
import { SettingsModal } from "./components/SettingsModal";
import { WsToast } from "./components/WsToast";
import type { FileInfo, SortKey } from "./types";
import { pathFromHash, setHashPath } from "./hashRoute";

/** Sort a file list in place: dirs first, then by key, then by name as tiebreaker. */
function sortFiles(list: FileInfo[], key: SortKey, asc: boolean): FileInfo[] {
  const dirs = list.filter((f) => f.is_dir);
  const files = list.filter((f) => !f.is_dir);

  // Sort directories by name only (always ascending).
  dirs.sort((a, b) => a.name.localeCompare(b.name));

  const cmp = (a: FileInfo, b: FileInfo): number => {
    let r: number;
    switch (key) {
      case "name":
        r = a.name.localeCompare(b.name);
        break;
      case "modified": {
        const ma = a.modified ?? 0;
        const mb = b.modified ?? 0;
        r = ma - mb;
        break;
      }
      case "size":
        r = a.size - b.size;
        break;
    }
    // Tiebreaker: always sort by name if same value.
    return r !== 0 ? r : a.name.localeCompare(b.name);
  };

  files.sort(cmp);
  if (!asc) files.reverse();

  return [...dirs, ...files];
}

/** Main application component. */
export function App() {
  const [currentPath, setCurrentPath] = useState(pathFromHash);
  const [search, setSearch] = useState("");
  const [apiBase, setApiBase] = useState(() => localStorage.getItem("fsv_api_base") ?? "/");
  const [sortKey, setSortKey] = useState<SortKey>("name");
  const [sortAsc, setSortAsc] = useState(true);

  const [previewFile, setPreviewFile] = useState<FileInfo | null>(null);
  const [qrFile, setQrFile] = useState<FileInfo | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [wsToast, setWsToast] = useState<string | null>(null);
  const [shuttingDown, setShuttingDown] = useState(false);

  const { files, loading, error, refresh } = useFileList(currentPath, apiBase);

  const wsStatus = useWebSocket(apiBase, (msg) => {
    setWsToast(msg);
    copyToClipboard(msg);
  });

  const handleSortChange = useCallback((key: SortKey) => {
    setSortKey((prev) => {
      if (prev === key) {
        setSortAsc((v) => !v);
      } else {
        setSortAsc(true);
      }
      return key;
    });
  }, []);

  const filtered = useMemo(() => {
    const matched = files.filter((f) => f.name.toLowerCase().includes(search.toLowerCase()));
    return sortFiles(matched, sortKey, sortAsc);
  }, [files, search, sortKey, sortAsc]);

  const getQrUrl = (file: FileInfo) => {
    const url = fileUrl(apiBase, file.path);
    // If apiBase is relative (starts with /), prepend the origin to make a full URL
    if (url.startsWith("/")) {
      return window.location.origin + url;
    }
    return url;
  };

  // Navigate to a path and update the URL hash
  const navigateTo = useCallback((path: string) => {
    setCurrentPath(path);
    setSearch("");
    setHashPath(path);
  }, []);

  // Listen to browser back/forward (hashchange)
  useEffect(() => {
    const handler = () => {
      setCurrentPath(pathFromHash());
      setSearch("");
    };
    window.addEventListener("hashchange", handler);
    return () => window.removeEventListener("hashchange", handler);
  }, []);

  // Shutdown the server
  const handleShutdown = useCallback(async () => {
    setShuttingDown(true);
    try {
      await shutdownServer(apiBase);
    } catch {
      // Server may close the connection before responding; that's expected.
    }
  }, [apiBase]);

  // Normalize the URL hash on first load (decode percent-encoded paths)
  useEffect(() => {
    const decoded = pathFromHash();
    if (decoded) {
      setHashPath(decoded);
    }
  }, []);

  return (
    <Container
      disableGutters
      maxWidth="sm"
      sx={{
        minHeight: "100svh",
        display: "flex",
        flexDirection: "column",
        borderLeft: 1,
        borderRight: 1,
        borderColor: "divider",
        bgcolor: "background.default",
      }}
    >
      {/* ── Header ── */}
      <Header
        currentPath={currentPath}
        search={search}
        wsStatus={wsStatus}
        sortKey={sortKey}
        sortAsc={sortAsc}
        shuttingDown={shuttingDown}
        onSearchChange={setSearch}
        onNavigate={(path) => {
          navigateTo(path);
        }}
        onRefresh={refresh}
        onOpenSettings={() => setShowSettings(true)}
        onShutdown={handleShutdown}
        onSortKeyChange={handleSortChange}
      />

      {/* ── File List ── */}
      <Box component="main" sx={{ flex: 1, display: "flex", flexDirection: "column" }}>
        {/* Loading state */}
        {loading && (
          <Box
            sx={{
              flex: 1,
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              justifyContent: "center",
              gap: 2,
              py: 6,
            }}
          >
            <CircularProgress size={32} />
            <Typography variant="body2" color="text.secondary">
              Loading…
            </Typography>
          </Box>
        )}

        {/* Error state */}
        {!loading && error && (
          <Box
            sx={{
              flex: 1,
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              justifyContent: "center",
              gap: 1.5,
              py: 6,
              px: 3,
              textAlign: "center",
            }}
          >
            <Typography variant="h4" sx={{ opacity: 0.5 }}>
              ⚠️
            </Typography>
            <Typography variant="body2" color="error">
              {error}
            </Typography>
            <Button variant="contained" size="small" onClick={refresh}>
              Retry
            </Button>
          </Box>
        )}

        {/* Empty state */}
        {!loading && !error && filtered.length === 0 && (
          <Box
            sx={{
              flex: 1,
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              justifyContent: "center",
              gap: 1.5,
              py: 6,
              px: 3,
              textAlign: "center",
            }}
          >
            <Typography variant="h4" sx={{ opacity: 0.5 }}>
              📁
            </Typography>
            <Typography variant="body2" color="text.secondary">
              {search ? "No files match your search." : "This folder is empty."}
            </Typography>
          </Box>
        )}

        {/* File list */}
        {!loading &&
          !error &&
          filtered.map((file) => (
            <FileCard
              key={file.path}
              file={file}
              apiBase={apiBase}
              onNavigate={(f) => {
                navigateTo(f.path);
              }}
              onPreview={setPreviewFile}
              onQr={setQrFile}
            />
          ))}
      </Box>

      {/* ── Modals ── */}
      {previewFile && (
        <PreviewModal
          file={previewFile}
          files={files}
          apiBase={apiBase}
          onClose={() => setPreviewFile(null)}
          onPreviewFile={setPreviewFile}
        />
      )}

      {qrFile && (
        <QrModal url={getQrUrl(qrFile)} fileName={qrFile.name} onClose={() => setQrFile(null)} />
      )}

      {showSettings && (
        <SettingsModal
          current={apiBase}
          onSave={setApiBase}
          onClose={() => setShowSettings(false)}
        />
      )}

      {/* ── WS Toast ── */}
      {wsToast && <WsToast message={wsToast} onClose={() => setWsToast(null)} />}
    </Container>
  );
}
