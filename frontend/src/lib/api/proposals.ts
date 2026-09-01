// Curator proposals API client — the universal consensus queue.
// Contract: src/routes/proposals.rs (M2). Envelope {err:0,...} like the rest
// of the API family; request helper mirrors src/lib/api/forum.ts.

import { authHeaders } from './social';

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`/api${path}`, {
    credentials: 'include',
    ...init,
    headers: {
      'Content-Type': 'application/json',
      ...authHeaders(),
      ...init?.headers,
    },
  });
  return (await res.json()) as T;
}

export type ProposalKind =
  | 'translate'
  | 'ui_string'
  | 'content_fix'
  | 'metadata_fix'
  | 'post_edit'
  | 'request_edit'
  | 'doc_edit'
  | 'work_deletion'
  | 'collection_add';

export type ProposalStatus = 'pending' | 'approved' | 'dismissed' | 'superseded';

export interface Proposal {
  id: number;
  kind: ProposalKind;
  target_type: string;
  target_id: string;
  payload: Record<string, unknown>;
  proposer_id: number | null;
  proposer: string | null;
  source: 'user' | 'llm' | 'moderator' | 'system';
  status: ProposalStatus;
  supersedes: number | null;
  note: string | null;
  created_at: string;
  approves?: number;
  dismisses?: number;
  quorum?: number;
}

export interface ProposalVote {
  curator: string;
  decision: 'approve' | 'dismiss';
  created_at: string;
}

export interface ProposalDetail extends Proposal {
  votes: ProposalVote[];
  competing: Proposal[];
}

export async function createProposal(input: {
  kind: ProposalKind;
  target_type: string;
  target_id: string;
  payload: Record<string, unknown>;
  // honeypot (silent-reject trap): the real form always sends these.
  website?: string;
  form_opened_at?: string;
}): Promise<{ err: number; id?: number; msg?: string }> {
  return request('/proposals', {
    method: 'POST',
    body: JSON.stringify({ website: '', ...input }),
  });
}

export async function getProposal(id: number): Promise<{ err: number; proposal?: ProposalDetail; msg?: string }> {
  return request(`/proposals/${id}`);
}

export async function voteProposal(
  id: number,
  decision: 'approve' | 'dismiss',
): Promise<{ err: number; status?: ProposalStatus; approves?: number; dismisses?: number; resolved?: boolean; msg?: string }> {
  return request(`/proposals/${id}/vote`, {
    method: 'POST',
    body: JSON.stringify({ decision }),
  });
}

export async function decideProposal(
  id: number,
  decision: 'approve' | 'dismiss',
  note?: string,
): Promise<{ err: number; status?: ProposalStatus; msg?: string }> {
  return request(`/proposals/${id}/decide`, {
    method: 'POST',
    body: JSON.stringify({ decision, note }),
  });
}

export async function listProposals(params: {
  kind?: string;
  status?: string;
  locale?: string;
  page?: number;
}): Promise<{ err: number; items: Proposal[]; page: number }> {
  const q = new URLSearchParams();
  if (params.kind) q.set('kind', params.kind);
  if (params.status) q.set('status', params.status);
  if (params.locale) q.set('locale', params.locale);
  q.set('page', String(params.page ?? 1));
  return request(`/curator/proposals?${q.toString()}`);
}

export async function flagTranslation(input: {
  target_type: string;
  target_id: string;
  field: string;
  locale: string;
  reason?: string;
}): Promise<{ err: number; dup?: boolean; msg?: string }> {
  return request('/translate/flag', { method: 'POST', body: JSON.stringify(input) });
}
