// Layout & widget registry API client.
// Backend: src/routes/me/routes.rs — GET/PUT /api/me/layout/{page},
//          src/routes/widgets/routes.rs — GET /api/widgets, GET /api/features

const BASE = '/api';

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const headers = new Headers(init?.headers);
  headers.set('Content-Type', 'application/json');
  const res = await fetch(`${BASE}${path}`, { ...init, headers });
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`API error ${res.status}: ${text}`);
  }
  return (await res.json()) as T;
}

// ── Types ────────────────────────────────────────────────────────────────

/** A single widget slot in a page layout. */
export interface LayoutItem {
  /** Widget slug, e.g. "recs", "trending", "download-input". */
  widget: string;
  /** Grid width: 'full' = 1 col, 'half' = 1/2, 'quarter' = 1/4. */
  size: 'full' | 'half' | 'quarter';
  /** Sort order (lower = earlier). */
  pos: number;
}

/** A widget returned by GET /api/widgets. */
export interface WidgetInfo {
  slug: string;
  name: string;
  description: string;
  /** Minimum user level required to see this widget. */
  min_rank: number;
  /** Emoji icon. */
  icon: string;
}

/** A feature flag returned by GET /api/features. */
export interface Feature {
  slug: string;
  enabled: boolean;
  /** Minimum level required for auto-enable (0 = always on). */
  min_level: number;
}

// ── API functions ────────────────────────────────────────────────────────

/** GET /api/me/layout/{page} — fetch user's saved layout for a page. */
export async function fetchLayout(page: string): Promise<LayoutItem[]> {
  try {
    const res = await request<{ layout?: LayoutItem[]; err?: number }>(
      `/me/layout/${encodeURIComponent(page)}`,
    );
    return res.layout ?? [];
  } catch {
    return [];
  }
}

/** PUT /api/me/layout/{page} — persist user's layout for a page. */
export async function saveLayout(
  page: string,
  layout: LayoutItem[],
): Promise<void> {
  await request(`/me/layout/${encodeURIComponent(page)}`, {
    method: 'PUT',
    body: JSON.stringify({ layout }),
  });
}

/** GET /api/widgets — list all available widgets in the registry. */
export async function fetchWidgets(): Promise<WidgetInfo[]> {
  try {
    const res = await request<{ widgets?: WidgetInfo[]; err?: number }>(
      '/widgets',
    );
    return res.widgets ?? [];
  } catch {
    return [];
  }
}

/** GET /api/features — list all features with enabled state. */
export async function fetchFeatures(): Promise<Feature[]> {
  try {
    const res = await request<{ features?: Feature[]; err?: number }>(
      '/features',
    );
    return res.features ?? [];
  } catch {
    return [];
  }
}
