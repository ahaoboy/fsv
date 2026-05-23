import { createTheme } from '@mui/material/styles';

// Shared theme options for both light and dark modes
const sharedTheme = {
  typography: {
    fontFamily: [
      'system-ui',
      '-apple-system',
      'Segoe UI',
      'Roboto',
      'sans-serif',
    ].join(','),
  },
  shape: {
    borderRadius: 10,
  },
  components: {
    MuiCssBaseline: {
      styleOverrides: {
        body: {
          margin: 0,
          transition: 'background-color 0.3s, color 0.3s',
        },
      },
    },
  },
};

// Light theme
export const lightTheme = createTheme({
  ...sharedTheme,
  palette: {
    mode: 'light',
    primary: {
      main: '#7c3aed', // Violet accent
      light: '#a78bfa',
      dark: '#5b21b6',
    },
    background: {
      default: '#ffffff',
      paper: '#f4f3ec',
    },
    text: {
      primary: '#08060d',
      secondary: '#6b6375',
    },
    divider: '#e5e4e7',
  },
});

// Dark theme
export const darkTheme = createTheme({
  ...sharedTheme,
  palette: {
    mode: 'dark',
    primary: {
      main: '#c084fc', // Lighter violet for dark mode
      light: '#d8b4fe',
      dark: '#a855f7',
    },
    background: {
      default: '#16171d',
      paper: '#1f2028',
    },
    text: {
      primary: '#f3f4f6',
      secondary: '#9ca3af',
    },
    divider: '#2e303a',
  },
});
