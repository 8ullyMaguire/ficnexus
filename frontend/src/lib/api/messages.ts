// API client for site-wide DMs (Lane 3 — /api/messages/*).

import { authHeaders } from './social';
import type { ApiResponse } from './forum';

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const headers = new Headers(init?.headers);
  Object.entries(authHeaders()).forEach(([k, v]) => headers.set(k, v));
  if (init?.body && !headers.has('content-type')) {
    headers.set('content-type', 'application/json');
  }
  const res = await fetch(`/api${path}`, { ...init, headers });
  return (await res.json()) as T;
}

export interface DMRoom {
  id: number;
  name: string | null;
  is_group: boolean;
  created_at: string;
  last_activity_at: string | null;
}

export interface RoomsResponse extends ApiResponse {
  rooms?: DMRoom[];
  count?: number;
}

export interface RoomResponse extends ApiResponse {
  room_id?: number;
  existing?: boolean;
}

export interface DMMessage {
  id: number;
  room_id?: number;
  sender_id: number;
  sender_username: string | null;
  body: string;
  created_at: string;
}

export interface MessagesResponse extends ApiResponse {
  messages?: DMMessage[];
  has_more?: boolean;
}

export interface SendResponse extends ApiResponse {
  message_id?: number;
}

/** GET /api/messages/rooms — my rooms, newest activity first. */
export async function listRooms(filter: 'all' | 'dm' | 'group' = 'all'): Promise<RoomsResponse> {
  return request(`/messages/rooms?filter=${filter}`);
}

/** POST /api/messages/rooms — find-or-create a DM room with a user. */
export async function createDMRoom(userId: number): Promise<RoomResponse> {
  return request('/messages/rooms', {
    method: 'POST',
    body: JSON.stringify({ user_id: userId }),
  });
}

/** GET /api/messages/rooms/{id}/messages — cursor pages, newest first. */
export async function getRoomMessages(
  roomId: number,
  opts: { cursor?: number; limit?: number } = {},
): Promise<MessagesResponse> {
  const params = new URLSearchParams();
  if (opts.cursor != null) params.set('cursor', String(opts.cursor));
  if (opts.limit != null) params.set('limit', String(opts.limit));
  const qs = params.toString();
  return request(`/messages/rooms/${roomId}/messages${qs ? `?${qs}` : ''}`);
}

/** POST /api/messages/rooms/{id}/messages — send. 403 when blocked. */
export async function sendRoomMessage(roomId: number, body: string): Promise<SendResponse> {
  return request(`/messages/rooms/${roomId}/messages`, {
    method: 'POST',
    body: JSON.stringify({ body }),
  });
}

/** PATCH /api/messages/rooms/{id} — mute / notify_level / leave. */
export async function updateRoomPrefs(
  roomId: number,
  prefs: { mute?: boolean; notify_level?: 'all' | 'mentions' | 'none'; leave?: boolean },
): Promise<ApiResponse & { room_id?: number }> {
  return request(`/messages/rooms/${roomId}`, {
    method: 'PATCH',
    body: JSON.stringify(prefs),
  });
}
