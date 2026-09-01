// API client for the FicNexus Rust backend (v0 API).
// All requests go to relative /api/* paths so they work behind the
// same-origin Rust server or any nginx proxy.

import type { ExportResponse, RecommendationsResponse, EntityRecsResponse, EntityKind } from './types';

const BASE = '/api';

// Client ID management for anonymous usage tracking
function getClientId(): string {
  const STORAGE_KEY = 'fichub_client_id';
  let clientId = localStorage.getItem(STORAGE_KEY);
  if (!clientId) {
    // Generate UUID v4
    clientId = 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
      const r = (Math.random() * 16) | 0;
      const v = c === 'x' ? r : (r & 0x3) | 0x8;
      return v.toString(16);
    });
    localStorage.setItem(STORAGE_KEY, clientId);
  }
  return clientId;
}

class ApiError extends Error {
  status: number;
  body: string;
  constructor(status: number, body: string) {
    super(`API error ${status}: ${body}`);
    this.name = 'ApiError';
    this.status = status;
    this.body = body;
  }
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  // Add client ID header to all requests
  const headers = new Headers(init?.headers);
  headers.set('X-Client-ID', getClientId());
  
  const res = await fetch(`${BASE}${path}`, { ...init, headers });
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new ApiError(res.status, text);
  }
  return (await res.json()) as T;
}

function buildQuery(params: Record<string, string | number | undefined>): string {
  const entries = Object.entries(params).filter(
    ([, v]) => v !== undefined && v !== null && v !== '',
  );
  if (entries.length === 0) return '';
  const qs = entries
    .map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(String(v))}`)
    .join('&');
  return `?${qs}`;
}

/** GET /api/epub?q=<url> — export a fic and return download URLs. */
export async function fetchExport(url: string): Promise<ExportResponse> {
  return request<ExportResponse>(`/epub${buildQuery({ q: url })}`);
}

/** Response from GET /api/epub/convert. */
export interface ConvertResponse {
  err: number;
  q?: string;
  msg?: string;
  url_id?: string;
  format?: string;
  hash?: string;
  url?: string;
  cached?: boolean;
  elapsed_ms?: number;
}

/**
 * GET /api/epub/convert?q=<url>&format=mobi|pdf|azw3 — convert a fic to a
 * Calibre-backed format on demand. Returns the download URL once ready.
 */
export async function convertFormat(
  url: string,
  format: string,
): Promise<ConvertResponse> {
  return request<ConvertResponse>(`/epub/convert${buildQuery({ q: url, format })}`);
}

/** GET /api/meta?q=<url> — fetch fic metadata without downloading. */
export async function fetchMeta(url: string): Promise<ExportResponse> {
  return request<ExportResponse>(`/meta${buildQuery({ q: url })}`);
}

/** GET /api/recommendations?q=<url>&n=<n> — get recommendations for a fic. */
export async function fetchRecommendations(
  q?: string,
  url_id?: string,
  n = 20,
): Promise<RecommendationsResponse> {
  return request<RecommendationsResponse>(
    `/recommendations${buildQuery({ q, url_id, n })}`,
  );
}

/**
 * GET /api/v0/recommendations/entities?kind=<kind>&seed=<seed>&limit=<limit>
 * Get entity recommendations (tags, fandoms, authors, collections, users).
 */
export async function fetchEntityRecommendations(
  kind: EntityKind,
  seed?: string,
  limit = 20,
): Promise<EntityRecsResponse> {
  return request<EntityRecsResponse>(
    `/recommendations/entities${buildQuery({ kind, seed, n: limit })}`,
  );
}

export { ApiError };
