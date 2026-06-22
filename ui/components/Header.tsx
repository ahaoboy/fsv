import {
  AppBar,
  Toolbar,
  Typography,
  Box,
  Breadcrumbs,
  Link,
  Chip,
  IconButton,
  TextField,
  InputAdornment,
  Tooltip,
} from "@mui/material";
import {
  Search as SearchIcon,
  Refresh as RefreshIcon,
  Settings as SettingsIcon,
  PowerSettingsNew as PowerIcon,
  GitHub as GitHubIcon,
} from "@mui/icons-material";
import type { WsStatus } from "../types";

interface Breadcrumb {
  name: string;
  path: string;
}

/** Build breadcrumb segments from a path string. */
export function getBreadcrumbs(path: string): Breadcrumb[] {
  const segments = path.split("/").filter(Boolean);
  const crumbs: Breadcrumb[] = [{ name: "Root", path: "" }];
  let acc = "";
  for (const seg of segments) {
    acc = acc ? `${acc}/${seg}` : seg;
    crumbs.push({ name: seg, path: acc });
  }
  return crumbs;
}

/** Status dot colors mapped to MUI palette colors. */
const wsDotColor: Record<WsStatus, string> = {
  connected: "#22c55e",
  connecting: "#eab308",
  disconnected: "#6b7280",
};

interface HeaderProps {
  currentPath: string;
  search: string;
  wsStatus: WsStatus;
  shuttingDown: boolean;
  onSearchChange: (value: string) => void;
  onNavigate: (path: string) => void;
  onRefresh: () => void;
  onOpenSettings: () => void;
  onShutdown: () => void;
}

/** App header with brand, breadcrumbs, search bar, and controls. */
export function Header({
  currentPath,
  search,
  wsStatus,
  shuttingDown,
  onSearchChange,
  onNavigate,
  onRefresh,
  onOpenSettings,
  onShutdown,
}: HeaderProps) {
  const crumbs = getBreadcrumbs(currentPath);

  return (
    <AppBar
      position="sticky"
      color="default"
      elevation={0}
      sx={{
        borderBottom: 1,
        borderColor: "divider",
        bgcolor: "background.default",
      }}
    >
      <Toolbar sx={{ flexDirection: "column", alignItems: "stretch", gap: 1, py: 1 }}>
        {/* Top row: brand + controls */}
        <Box sx={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
            <Chip
              label="FSV"
              size="small"
              sx={{
                fontWeight: 800,
                fontSize: 11,
                background: "linear-gradient(135deg, #a855f7, #7c3aed)",
                color: "#fff",
                borderRadius: 1.5,
              }}
            />
            <Typography variant="subtitle1" sx={{ fontWeight: 600 }} noWrap>
              File Share Viewer
            </Typography>
            <IconButton
              size="small"
              component="a"
              href="https://github.com/ahaoboy/fsv"
              target="_blank"
              rel="noopener noreferrer"
              title="View on GitHub"
              sx={{ color: "text.secondary", opacity: 0.6, "&:hover": { opacity: 1 } }}
            >
              <GitHubIcon fontSize="small" />
            </IconButton>
          </Box>

          <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
            {/* WebSocket status dot */}
            <Tooltip title={shuttingDown ? "Server shutting down" : `WebSocket: ${wsStatus}`}>
              <Box
                sx={{
                  width: 8,
                  height: 8,
                  borderRadius: "50%",
                  bgcolor: shuttingDown ? wsDotColor.disconnected : wsDotColor[wsStatus],
                  boxShadow:
                    !shuttingDown && wsStatus !== "disconnected"
                      ? `0 0 6px ${wsDotColor[wsStatus]}`
                      : undefined,
                  animation:
                    !shuttingDown && wsStatus === "connecting" ? "pulse 1.4s infinite" : undefined,
                  "@keyframes pulse": {
                    "0%, 100%": { opacity: 1 },
                    "50%": { opacity: 0.4 },
                  },
                }}
              />
            </Tooltip>
            <Tooltip title={shuttingDown ? "Shutting down…" : "Shutdown server"}>
              <span>
                <IconButton
                  size="small"
                  onClick={onShutdown}
                  disabled={shuttingDown}
                  sx={{ color: "error.main" }}
                >
                  <PowerIcon fontSize="small" />
                </IconButton>
              </span>
            </Tooltip>
            <Tooltip title="Settings">
              <IconButton size="small" onClick={onOpenSettings}>
                <SettingsIcon fontSize="small" />
              </IconButton>
            </Tooltip>
          </Box>
        </Box>

        {/* Breadcrumbs */}
        <Breadcrumbs
          aria-label="Path navigation"
          separator="/"
          sx={{ "& .MuiBreadcrumbs-ol": { flexWrap: "wrap" } }}
        >
          {crumbs.map((crumb, i) =>
            i === crumbs.length - 1 ? (
              <Typography key={crumb.path} variant="body2" color="text.primary">
                {crumb.name}
              </Typography>
            ) : (
              <Link
                key={crumb.path}
                component="button"
                underline="hover"
                variant="body2"
                color="text.secondary"
                onClick={() => {
                  onNavigate(crumb.path);
                  onSearchChange("");
                }}
                sx={{ fontFamily: "inherit" }}
              >
                {crumb.name}
              </Link>
            ),
          )}
        </Breadcrumbs>

        {/* Search + Refresh */}
        <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
          <TextField
            size="small"
            fullWidth
            placeholder="Filter files…"
            value={search}
            onChange={(e) => onSearchChange(e.target.value)}
            slotProps={{
              input: {
                startAdornment: (
                  <InputAdornment position="start">
                    <SearchIcon fontSize="small" color="action" />
                  </InputAdornment>
                ),
              },
            }}
            aria-label="Filter files"
            sx={{
              "& .MuiOutlinedInput-root": { borderRadius: 2 },
            }}
          />
          <Tooltip title="Refresh">
            <IconButton onClick={onRefresh} size="small">
              <RefreshIcon fontSize="small" />
            </IconButton>
          </Tooltip>
        </Box>
      </Toolbar>
    </AppBar>
  );
}
