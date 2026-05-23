import { useState, useEffect, useRef } from 'react';
import type { WsStatus } from '../types';

/** Resolve the WebSocket URL from the API base URL. */
function resolveWsUrl(apiBase: string): string {
  if (apiBase === '/') {
    const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    return `${proto}//${window.location.host}/ws`;
  }
  return apiBase.replace(/^http/, 'ws').replace(/\/$/, '') + '/ws';
}

/** Hook to manage a WebSocket connection and track its status. */
export function useWebSocket(
  apiBase: string,
  onMessage: (text: string) => void,
): WsStatus {
  const [status, setStatus] = useState<WsStatus>('disconnected');
  const wsRef = useRef<WebSocket | null>(null);
  const onMessageRef = useRef(onMessage);
  onMessageRef.current = onMessage;

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
    ws.onmessage = (e: MessageEvent<string>) => onMessageRef.current(e.data);
    ws.onclose = () => setStatus('disconnected');
    ws.onerror = () => setStatus('disconnected');

    return () => {
      ws.close();
    };
  }, [apiBase]);

  return status;
}
