/**
 * Offline reading support for FicNexus.
 *
 * Cache strategy decision for a URL — a pure function so it can be unit
 * tested without a service worker environment.
 *
 * - 'precache':              app-shell assets (immutable build hashes, icons, manifest)
 * - 'network-first':         root navigation + public fic-content APIs
 * - 'stale-while-revalidate': public fic-content API responses (serve cache immediately,
 *                            update in background)
 * - 'cache-first':           fic reader content — the core offline-reading payload
 * - 'none':                  everything else (POSTs, auth, search, cross-origin, …)
 */

// Public, cacheable fic-content API endpoints. Everything else under /api/
// (auth, bookmarks, admin, social, ...) returns 'none' so per-user data is
// never written to the cache.
const FIC_API_PREFIXES = [
  '/api/reader/',
  '/api/epub',
  '/api/meta',
  '/api/search/similar/',
];

export function decideStrategy(url: string): 'precache' | 'network-first' | 'stale-while-revalidate' | 'cache-first' | 'none' {
  let pathname: string;
  try {
    pathname = new URL(url, 'http://fichub.local').pathname;
  } catch {
    return 'none';
  }

  // Public fic-content API: stale-while-revalidate (serve cache immediately,
  // update in background). Private endpoints return 'none' below.
  if (pathname.startsWith('/api/')) {
    return FIC_API_PREFIXES.some((prefix) => pathname.startsWith(prefix))
      ? 'stale-while-revalidate'
      : 'none';
  }

  // Fic content routes: cache-first for offline reading
  if (pathname.startsWith('/read/') || pathname.startsWith('/works/')) {
    return 'cache-first';
  }

  // Root navigation: network-first — never serve a stale cached shell when
  // online (a precached '/' is exactly what makes users see old builds after
  // a deploy). Offline falls back to the cached shell.
  if (pathname === '/') {
    return 'network-first';
  }

  // App-shell assets: precached (hashed build files, icons, manifest, SW)
  if (
    pathname.startsWith('/_app/') ||
    pathname.startsWith('/icon-') ||
    pathname === '/manifest.webmanifest' ||
    pathname === '/favicon.png' ||
    pathname === '/sw.js'
  ) {
    return 'precache';
  }

  return 'none';
}
