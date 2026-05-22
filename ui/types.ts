export interface FileInfo {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  modified: number | null;
}

export type WsStatus = 'connecting' | 'connected' | 'disconnected';

export interface WsMessage {
  time: string;
  text: string;
}
