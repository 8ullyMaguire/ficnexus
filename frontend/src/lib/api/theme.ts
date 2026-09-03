// Theme API client — persists theme tokens server-side.
// Backend: src/routes/me/routes.rs — GET/PUT /api/me/theme

import type { ThemeTokens } from '$lib/themes/presets.js';

const BASE = '/api/me';

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

/** GET /api/me/theme — fetch the user's saved theme tokens. */
export async function fetchTheme(): Promise<ThemeTokens | null> {
  try {
    const res = await request<{ theme?: ThemeTokens; err?: number }>(
      '/theme',
    );
    return res.theme ?? null;
  } catch {
    return null;
  }
}

/** PUT /api/me/theme — persist the user's theme tokens. */
export async function saveThemeToServer(tokens: ThemeTokens): Promise<void> {
  await request('/theme', {
    method: 'PUT',
    body: JSON.stringify({ theme: tokens }),
  });
}
