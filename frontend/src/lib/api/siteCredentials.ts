// API client functions for user-supplied site credentials (settings page).
// These talk to the AuthUser-gated endpoints; the caller must be logged in
// (token in localStorage) or the backend returns 401.

import { authHeaders } from './social';

export interface SiteCredential {
  domain: string;
  username: string;
  expires_at: string;
}

export interface SiteCredentialsListResponse {
  err: number;
  credentials: SiteCredential[];
}

export interface SiteCredentialSaveResponse {
  err: number;
  domain: string;
  username: string;
  expires_at: string;
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`/api${path}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...authHeaders(),
      ...options?.headers,
    },
  });
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`API error ${res.status}: ${text}`);
  }
  if (res.status === 204) return undefined as T;
  return (await res.json()) as T;
}

/** GET /api/user/site-credentials — list the caller's stored credentials. */
export async function listSiteCredentials(): Promise<SiteCredentialsListResponse> {
  return request<SiteCredentialsListResponse>('/user/site-credentials');
}

/** PUT /api/user/site-credentials — save credentials (encrypted server-side). */
export async function setSiteCredentials(
  domain: string,
  username: string,
  password: string,
): Promise<SiteCredentialSaveResponse> {
  return request<SiteCredentialSaveResponse>('/user/site-credentials', {
    method: 'PUT',
    body: JSON.stringify({ domain, username, password }),
  });
}

/** DELETE /api/user/site-credentials/{domain} — remove stored credentials. */
export async function deleteSiteCredentials(domain: string): Promise<void> {
  return request<void>(`/user/site-credentials/${encodeURIComponent(domain)}`, {
    method: 'DELETE',
  });
}
