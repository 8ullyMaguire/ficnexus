// API client for forum groups (Lane 5 — groups management).

import { authHeaders } from './social';

export interface ForumGroup {
  id: number;
  name: string;
  slug: string;
  description: string;
  cover_url: string | null;
  is_private: boolean;
  is_system: boolean;
  owner_id: number;
  owner_username: string;
  member_count: number;
  caller_is_member: boolean;
  created_at: string;
}

export interface GroupMember {
  user_id: number;
  username: string;
  role: string;
  joined_at: string;
}

export interface GroupDetail extends ForumGroup {
  members: GroupMember[];
}

/** List groups with optional filters. */
export async function listGroups(opts?: {
  type?: 'public' | 'private' | 'system';
  search?: string;
  limit?: number;
  cursor?: number;
}): Promise<{ err: number; items?: ForumGroup[]; next_cursor?: number | null; msg?: string }> {
  const params = new URLSearchParams();
  if (opts?.type) params.set('type', opts.type);
  if (opts?.search) params.set('search', opts.search);
  if (opts?.limit) params.set('limit', String(opts.limit));
  if (opts?.cursor) params.set('cursor', String(opts.cursor));
  const q = params.toString();
  const res = await fetch(`/api/forum/groups${q ? `?${q}` : ''}`, {
    headers: authHeaders(),
  });
  return res.json();
}

/** Get a single group with its members. */
export async function getGroup(
  groupId: number,
): Promise<{ err: number; group?: GroupDetail; msg?: string }> {
  const res = await fetch(`/api/forum/groups/${groupId}`, {
    headers: authHeaders(),
  });
  return res.json();
}

/** Create a new group. Returns the new group id. */
export async function createGroup(data: {
  name: string;
  description?: string;
  is_private?: boolean;
}): Promise<{ err: number; group_id?: number; msg?: string }> {
  const res = await fetch('/api/forum/groups', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', ...authHeaders() },
    body: JSON.stringify(data),
  });
  return res.json();
}

/** Update a group (owner only). */
export async function updateGroup(
  groupId: number,
  data: { name?: string; description?: string; is_private?: boolean },
): Promise<{ err: number; msg?: string }> {
  const res = await fetch(`/api/forum/groups/${groupId}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json', ...authHeaders() },
    body: JSON.stringify(data),
  });
  return res.json();
}

/** Delete a group (owner only). */
export async function deleteGroup(
  groupId: number,
): Promise<{ err: number; msg?: string }> {
  const res = await fetch(`/api/forum/groups/${groupId}`, {
    method: 'DELETE',
    headers: authHeaders(),
  });
  return res.json();
}

/** Join a group. */
export async function joinGroup(
  groupId: number,
): Promise<{ err: number; msg?: string }> {
  const res = await fetch(`/api/forum/groups/${groupId}/join`, {
    method: 'POST',
    headers: authHeaders(),
  });
  return res.json();
}

/** Leave a group. */
export async function leaveGroup(
  groupId: number,
): Promise<{ err: number; msg?: string }> {
  const res = await fetch(`/api/forum/groups/${groupId}/leave`, {
    method: 'POST',
    headers: authHeaders(),
  });
  return res.json();
}

/** Invite a user to a group (owner/moderator). */
export async function inviteToGroup(
  groupId: number,
  userId: number,
): Promise<{ err: number; msg?: string }> {
  const res = await fetch(`/api/forum/groups/${groupId}/invite`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', ...authHeaders() },
    body: JSON.stringify({ user_id: userId }),
  });
  return res.json();
}

/** Change a member's role (owner only). */
export async function changeMemberRole(
  groupId: number,
  userId: number,
  role: string,
): Promise<{ err: number; msg?: string }> {
  const res = await fetch(`/api/forum/groups/${groupId}/members/${userId}/role`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', ...authHeaders() },
    body: JSON.stringify({ role }),
  });
  return res.json();
}
