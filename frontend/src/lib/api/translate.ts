// Translation API client for the M1 frontend translation display layer.
//
// Endpoints (backend owns /api/translate/*):
//   POST /api/translate/batch?locale={locale}  — batch fetch translations
//     body: { items: [{type, id, field}, ...] }  (max 50)
//     → { err: 0, translations: { "type:id:field": { text, status } | null } }
//   POST /api/translate/enqueue                  — enqueue missing for LLM translation
//     body: { items: [{type, id, field}, ...], locale, website, form_opened_at }
//     → { err: 0, queued: N }
//   GET  /api/translate/status?target_type={type}&target_id={id} — coverage map
//     → { err: 0, coverage: { locale: { status } | null } }

import { authHeaders } from './social';

export interface TranslateItem {
  type: string;
  id: string;
  field: string;
}

export interface TranslateResult {
  text: string;
  status: 'machine' | 'approved' | null;
}

export interface BatchTranslateResponse {
  err: number;
  msg?: string;
  translations: Record<string, TranslateResult | null>;
}

export interface EnqueueResponse {
  err: number;
  msg?: string;
  queued: number;
}

export interface CoverageResponse {
  err: number;
  msg?: string;
  coverage: Record<string, { status: 'machine' | 'approved' } | null>;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(path, {
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

/** Per-locale batch coalescing for concurrent callers. */
const localeBatches = new Map<string, { items: TranslateItem[]; resolve: () => void; promise: Promise<void> }>();

/** Module-level result cache keyed by "type:id:field:locale". */
const resultCache = new Map<string, TranslateResult | null>();

/**
 * Batch-fetch translations for the given items and locale.
 * Concurrent calls within the same microtask tick are coalesced into ONE POST.
 * Results are cached and returned from cache on subsequent calls.
 */
export async function requestTranslations(
  items: TranslateItem[],
  locale: string,
): Promise<Record<string, TranslateResult | null>> {
  if (!items.length) return {};

  // Build cache keys and check cache first
  const cacheKeys = items.map((it) => `${it.type}:${it.id}:${it.field}:${locale}`);
  const results: Record<string, TranslateResult | null> = {};
  const missingItems: TranslateItem[] = [];

  for (let i = 0; i < items.length; i++) {
    const key = cacheKeys[i];
    const cached = resultCache.get(key);
    if (cached !== undefined) {
      results[key] = cached;
    } else {
      missingItems.push(items[i]);
    }
  }

  if (!missingItems.length) return results;

  // Coalesce all missing items for this locale into ONE POST, flushed on the
  // next microtask. Callers await a shared promise that resolves AFTER the
  // fetch completes (a plain queueMicrotask(r) here would resolve too early:
  // the flush callback's fetch is still in flight).
  let entry = localeBatches.get(locale);
  if (!entry) {
    let r!: () => void;
    const p2 = new Promise<void>((res) => (r = res));
    entry = { items: [], resolve: r, promise: p2 };
    localeBatches.set(locale, entry);
    queueMicrotask(async () => {
      const batch = localeBatches.get(locale);
      if (!batch) return;
      localeBatches.delete(locale);
      const batchItems = batch.items;
      try {
        const res = await request<BatchTranslateResponse>(
          `/api/translate/batch?locale=${encodeURIComponent(locale)}`,
          { method: 'POST', body: JSON.stringify({ items: batchItems }) },
        );
        if (res.err === 0 && res.translations) {
          for (const [k, v] of Object.entries(res.translations)) {
            resultCache.set(`${k}:${locale}`, v);
          }
        }
      } catch {
        // On fetch failure, cache null for all requested keys so we don't
        // hammer the backend endlessly; next mount retries.
        for (const item of batchItems) {
          resultCache.set(`${item.type}:${item.id}:${item.field}:${locale}`, null);
        }
      }
      // Register items with no translation for the Translate-this-page button.
      for (const item of batchItems) {
        const cached = resultCache.get(`${item.type}:${item.id}:${item.field}:${locale}`);
        if (!cached) markUntranslated(locale, item);
      }
      batch.resolve();
    });
  }
  entry.items.push(...missingItems);
  await entry.promise;

  // Results are in the cache now (flush promise already resolved).
  for (const item of missingItems) {
    const key = `${item.type}:${item.id}:${item.field}:${locale}`;
    results[key] = resultCache.get(key) ?? null;
  }
  return results;
}
/**
 * Enqueue items for LLM translation (fire-and-forget style).
 * Adds honeypot fields: website:'' and form_opened_at (Date.now() - 5000).
 * Mirrors the pattern from CommentSection.svelte and requests detail page.
 */
export async function enqueueTranslations(
  items: TranslateItem[],
  locale: string,
): Promise<number> {
  if (!items.length) return 0;

  // Honeypot: website empty string + form_opened_at ~5s ago (copied from CommentSection/requests detail)
  const formOpenedAt = Date.now() - 5000;
  const website = '';

  try {
    const res = await request<EnqueueResponse>('/api/translate/enqueue', {
      method: 'POST',
      body: JSON.stringify({ items, locale, website, form_opened_at: formOpenedAt }),
    });
    if (res.err === 0) return res.queued ?? 0;
  } catch {
    // Silently fail — translation is best-effort display enhancement
  }
  return 0;
}

/**
 * Untranslated-item registry (drives the TranslatePageButton):
 * TranslatedText calls markUntranslated(locale, item) whenever a resolved
 * batch fetch comes back with no translation, so the button can enqueue
 * "everything visible" without enumerating the page itself.
 */
const untranslated = new Map<string, TranslateItem[]>();

export function markUntranslated(locale: string, item: TranslateItem): void {
  const key = `${locale}|${item.type}|${item.id}|${item.field}`;
  const list = untranslated.get(locale) ?? [];
  if (!list.some((i) => `${locale}|${i.type}|${i.id}|${i.field}` === key)) {
    list.push(item);
    untranslated.set(locale, list);
  }
}

/** Drain the registered untranslated items for a locale. */
export function takePendingItems(locale: string): TranslateItem[] {
  const items = untranslated.get(locale) ?? [];
  untranslated.delete(locale);
  return items;
}

/**
 * Get translation coverage for a target (which locales have what status).
 * Returns a map of locale → status object or null.
 */
export async function coverageFor(
  type: string,
  id: string,
): Promise<Record<string, { status: 'machine' | 'approved' } | null>> {
  try {
    const res = await request<CoverageResponse>(
      `/api/translate/status?target_type=${encodeURIComponent(type)}&target_id=${encodeURIComponent(id)}`,
    );
    if (res.err === 0) return res.coverage ?? {};
  } catch {
    // Silently fail
  }
  return {};
}

/** Clear the module-level result cache (useful for testing or logout). */
export function clearTranslationCache(): void {
  resultCache.clear();
  localeBatches.clear();
}