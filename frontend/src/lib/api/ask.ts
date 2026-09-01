// Ask the Archive — natural-language search.
//
// POST /api/search/ask: sends the NL query to the backend, which translates
// it into a v2 search query string via Ollama (falling back to a plain
// keyword search when Ollama is unavailable) and runs the same search
// pipeline as /api/search. The response is the standard /api/search envelope
// plus:
//   translated: bool   — true when Ollama produced a v2 query
//   nl_query: string   — the original NL request (for the UI)
//   applied_params: string — the v2 query that was actually run (rendered
//                       as the "Interpreted as" chip; the same string works
//                       verbatim as /search?q=)

import type { SearchFacets, SearchResult } from './search';

/** The v2 query string Ollama produced from the natural-language request. */
export type AskAppliedParams = string;

/** Response from POST /api/search/ask. */
export interface AskResponse {
  total: number;
  page: number;
  per_page: number;
  results: SearchResult[];
  facets: SearchFacets;
  translated: boolean;
  nl_query: string;
  applied_params: AskAppliedParams;
}

/**
 * Ask the Archive: natural language → v2 search query → results.
 * Throws on non-OK so the page can show the error.
 */
export async function askArchive(nlQuery: string): Promise<AskResponse> {
  const res = await fetch('/api/search/ask', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ q: nlQuery }),
  });
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`Ask failed (${res.status}): ${text}`);
  }
  return (await res.json()) as AskResponse;
}

/** Human-readable rendering of the applied v2 query for the chip. */
export function describeAppliedParams(p: AskAppliedParams): string {
  return p;
}
