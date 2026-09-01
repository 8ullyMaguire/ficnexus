// API client for Fic Requests (prompt board).

export interface ApiResponse {
  err: number;
  msg?: string;
}

export interface RequestItem {
  id: number;
  title: string;
  body: string;
  seed_work_id: number | null;
  status: 'open' | 'answered' | 'closed';
  created_at: string;
  answer_count: number;
  upvotes?: number;
  username?: string;
}

export type AnswerKind = 'work' | 'llm' | 'search';

export interface RequestAnswer {
  id: number;
  user_id: number;
  username: string;
  work_id: number | null;
  fic_title: string;
  fic_author: string;
  pitch: string;
  source: string;
  created_at: string;
  score: number;
  my_vote: number | null;
  accepted: boolean;
  answer_kind: AnswerKind;
  search_query: string | null;
  payload: unknown;
}

export interface RequestDetail {
  id: number;
  user_id: number;
  username: string;
  title: string;
  body: string;
  seed_work_id: number | null;
  seed_fic: { title: string; author: string } | null;
  status: 'open' | 'answered' | 'closed';
  created_at: string;
  accepted_answer_id: number | null;
  upvotes?: number;
  my_upvote?: boolean;
}

export interface RequestDetailResponse {
  err: number;
  request: RequestDetail;
  answers: RequestAnswer[];
}

const headers = (): Record<string, string> => ({
  'Content-Type': 'application/json',
});

export async function listRequests(status = 'open', sort = 'new', page = 1): Promise<ApiResponse & { items?: RequestItem[] }> {
  const res = await fetch(`/api/requests?status=${status}&sort=${sort}&page=${page}`, { credentials: 'include' });
  return res.json();
}

export async function getRequest(id: number): Promise<RequestDetailResponse> {
  const res = await fetch(`/api/requests/${id}`, { credentials: 'include' });
  return res.json();
}

export async function createRequest(body: { title: string; body?: string; seed_work_id?: number | null }): Promise<ApiResponse & { id?: number }> {
  const res = await fetch('/api/requests', {
    method: 'POST',
    credentials: 'include',
    headers: headers(),
    body: JSON.stringify(body),
  });
  return res.json();
}

export interface AddAnswerPayload {
  work_id: number | null;
  url: string;
  pitch: string;
  url_id: string | null;
  search_query: string | null;
}

export async function addAnswer(
  requestId: number,
  workId: number | null,
  url: string,
  pitch: string,
  urlId: string | null,
  searchQuery: string | null,
): Promise<ApiResponse & { answer_id?: number }> {
  const body: AddAnswerPayload = {
    work_id: workId,
    url,
    pitch,
    url_id: urlId ?? null,
    search_query: searchQuery ?? null,
  };
  const res = await fetch(`/api/requests/${requestId}/answers`, {
    method: 'POST',
    credentials: 'include',
    headers: headers(),
    body: JSON.stringify(body),
  });
  return res.json();
}

export async function voteAnswer(requestId: number, answerId: number, vote: 1 | -1 | 0): Promise<ApiResponse & { score?: number; my_vote?: number | null }> {
  const res = await fetch(`/api/requests/${requestId}/answers/${answerId}/vote`, {
    method: 'POST',
    credentials: 'include',
    headers: headers(),
    body: JSON.stringify({ vote }),
  });
  return res.json();
}

export async function upvoteRequest(requestId: number, enabled: boolean): Promise<ApiResponse & { upvotes?: number; my_upvote?: boolean }> {
  const res = await fetch(`/api/requests/${requestId}/upvote`, {
    method: 'POST',
    credentials: 'include',
    headers: headers(),
    body: JSON.stringify({ enabled }),
  });
  return res.json();
}

export async function acceptAnswer(requestId: number, answerId: number): Promise<ApiResponse> {
  const res = await fetch(`/api/requests/${requestId}/accept/${answerId}`, {
    method: 'POST',
    credentials: 'include',
    headers: headers(),
    body: JSON.stringify({}),
  });
  return res.json();
}

export async function deleteRequest(id: number): Promise<ApiResponse> {
  const res = await fetch(`/api/requests/${id}`, { method: 'DELETE', credentials: 'include' });
  return res.json();
}

export async function deleteAnswer(requestId: number, answerId: number): Promise<ApiResponse> {
  const res = await fetch(`/api/requests/${requestId}/answers/${answerId}`, {
    method: 'DELETE',
    credentials: 'include',
  });
  return res.json();
}
