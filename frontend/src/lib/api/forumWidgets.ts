// Forum widgets API client.

import { authHeaders } from '$lib/api/social';

const BASE = '/api/forum';

async function request<T>(path: string): Promise<T> {
  const headers = { 'Content-Type': 'application/json', ...authHeaders() };
  const res = await fetch(`${BASE}${path}`, { headers });
  if (!res.ok) throw new Error(`API error ${res.status}`);
  return res.json() as Promise<T>;
}

export interface WidgetTopic {
  id: number;
  title: string;
  category?: { slug: string };
  author_id?: number;
  created_at: string;
  reply_count: number;
  view_count?: number;
  score?: number;
}

export interface ForumStats {
  total_topics: number;
  total_posts: number;
  total_users: number;
  newest_user: { username: string; created_at: string } | null;
}

export function fetchRecentTopics(): Promise<{ topics: WidgetTopic[] }> {
  return request('/widgets/recent');
}

export function fetchPopularTopics(): Promise<{ topics: WidgetTopic[] }> {
  return request('/widgets/popular');
}

export function fetchForumStats(): Promise<ForumStats> {
  return request('/widgets/stats');
}
