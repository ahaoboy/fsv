import { useState, useEffect, useRef } from 'preact/hooks';
import type { WsStatus } from '../types';

function resolveWsUrl(apiBase: string): string {
  if (apiBase === '/') {
    const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    return `${proto}//${window.location.host}/ws`;
  }
  return apiBase.replace(/^http/, 'ws').replace(/\/$/, '') + '/ws';
}

export function useWebSocket(
  apiBase: string,
  onMessage: (text: string) => void,
) {
  const [status, setStatus] = useState<WsStatus>('disconnected');
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    wsRef.current?.close();
    setStatus('connecting');

    let ws: WebSocket;
    try {
      ws = new WebSocket(resolveWsUrl(apiBase));
      wsRef.current = ws;
    } catch {
      setStatus('disconnected');
      return;
    }

    ws.onopen = () => setStatus('connected');
    ws.onmessage = (e) => onMessage(e.data as string);
    ws.onclose = () => setStatus('disconnected');
    ws.onerror = () => setStatus('disconnected');

    return () => ws.close();
  }, [apiBase]);

  return status;
}
