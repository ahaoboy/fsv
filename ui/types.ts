/** File or directory info returned by the server. */
export interface FileInfo {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  modified: number | null;
}

/** WebSocket connection status. */
export type WsStatus = 'connecting' | 'connected' | 'disconnected';

/** WebSocket broadcast message. */
export interface WsMessage {
  time: string;
  text: string;
}
