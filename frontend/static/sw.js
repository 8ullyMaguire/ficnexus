/* FicNexus offline reading service worker (plain JS, served from /sw.js). */
'use strict';

const VERSION = 'fichub-v6';

// Caches
const PRECACHE = `${VERSION}-precache`;
const RUNTIME_FIC = `${VERSION}-fic`;
const RUNTIME_API = `${VERSION}-api`;

// App-shell assets to precache. Hashed immutable files are fingerprinted,
// so the list is discovered at install time and the cache is versioned.
// NOTE: '/' is intentionally NOT precached — the navigation shell must be
// network-first so users always get the latest build when online (a
// precached '/' previously served a stale shell for the life of the SW
// version, which looked exactly like a failed deploy).
const PRECACHE_URLS = ['/manifest.webmanifest', '/icon-192.png', '/icon-512.png', '/favicon.png'];

/** Decide the caching strategy for a request URL (mirrors src/lib/pwa/strategies.ts). */
function decideStrategy(url) {
  let pathname;
  try {
    pathname = new URL(url).pathname;
  } catch {
    return 'none';
  }
  // Public fic-content APIs only — auth, bookmarks, admin, social etc. are
  // never intercepted so per-user data is never written to the cache.
  const FIC_API_PREFIXES = ['/api/reader/', '/api/epub', '/api/meta', '/api/search/similar/'];
  if (pathname.startsWith('/api/')) {
    return FIC_API_PREFIXES.some((p) => pathname.startsWith(p)) ? 'stale-while-revalidate' : 'none';
  }
  if (pathname.startsWith('/read/') || pathname.startsWith('/works/')) return 'cache-first';
  // Root navigation: network-first — never serve a stale cached shell when
  // online; fall back to the cached shell only when offline.
  if (pathname === '/') return 'network-first';
  if (pathname.startsWith('/_app/') || pathname.startsWith('/icon-') ||
      pathname === '/manifest.webmanifest' || pathname === '/favicon.png' || pathname === '/sw.js') {
    return 'precache';
  }
  return 'none';
}

// stale-while-revalidate: serve the cached copy immediately (offline reading),
// then fetch a fresh copy in the background to keep the cache current.
async function staleWhileRevalidate(request, cacheName) {
  const cache = await caches.open(cacheName);
  const cached = await cache.match(request);
  const network = fetch(request)
    .then((response) => {
      if (response.ok) cache.put(request, response.clone());
      return response;
    })
    .catch(() => cached); // offline — fall back to whatever we have
  // Fresh responses go to the client; if the network fails, the cached copy
  // (or undefined → browser error) is served.
  return cached || network;
}

// Cache-first keeps previously opened works readable without a connection.
async function cacheFirst(request, cacheName) {
  const cached = await caches.match(request);
  if (cached) return cached;
  const response = await fetch(request);
  if (response.ok) {
    const cache = await caches.open(cacheName);
    await cache.put(request, response.clone());
  }
  return response;
}

// network-first: try the network; only when it fails (offline) serve the
// cached copy. Used for the root navigation so online users always get the
// current build.
async function networkFirst(request) {
  const cache = await caches.open(PRECACHE);
  try {
    const network = await fetch(request);
    if (network.ok) cache.put(request, network.clone());
    return network;
  } catch {
    return (await cache.match(request)) || Response.error();
  }
}

async function clearOldCaches() {
  const keys = await caches.keys();
  await Promise.all(keys.filter((k) => !k.startsWith(VERSION)).map((k) => caches.delete(k)));
}

// Discover the hashed SvelteKit build assets by fetching the shell and parsing
// the /_app/ URLs out of the HTML. Without these the SPA shell loads but the
// app's own JS/CSS is missing, so the page renders blank offline.
async function discoverBuildAssets(cache) {
  const response = await fetch('/');
  if (!response.ok) return;
  const html = await response.text();
  const assetUrls = [...html.matchAll(/(?:src|href)="(\/_app\/[^"]+)"/g)].map((m) => m[1]);
  const unique = [...new Set(assetUrls)];
  await Promise.all(
    unique.map(async (asset) => {
      try {
        const res = await fetch(asset);
        if (res.ok) await cache.put(asset, res);
      } catch {
        /* skip individual asset failures — precache fetch-fallback covers them */
      }
    }),
  );
}

// Install: precache the app shell + build assets, then remove old caches.
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches
      .open(PRECACHE)
      .then((cache) => cache.addAll(PRECACHE_URLS).then(() => discoverBuildAssets(cache)))
      .then(clearOldCaches)
      .then(() => self.skipWaiting()),
  );
});

// Activate: take control of open clients immediately.
self.addEventListener('activate', (event) => {
  event.waitUntil(self.clients.claim());
});

// Fetch: route by strategy.
self.addEventListener('fetch', (event) => {
  const request = event.request;
  const url = new URL(request.url);

  // Same-origin GETs only (skip cross-origin, skip non-GET like POST auth).
  if (request.method !== 'GET' || url.origin !== self.location.origin) return;

  const strategy = decideStrategy(request.url);

  if (strategy === 'precache') {
    // App-shell assets are immutable (hashed) → cache-first. Cache on first
    // fetch too, so assets missed during install (e.g. lazy chunks) get stored.
    event.respondWith(
      caches.match(request).then((cached) => {
        if (cached) return cached;
        return fetch(request).then((response) => {
          if (response.ok) {
            const copy = response.clone();
            caches.open(PRECACHE).then((cache) => cache.put(request, copy));
          }
          return response;
        });
      }),
    );
    return;
  }

  if (strategy === 'network-first') {
    event.respondWith(networkFirst(request));
    return;
  }

  if (strategy === 'cache-first') {
    event.respondWith(cacheFirst(request, RUNTIME_FIC));
    return;
  }

  if (strategy === 'stale-while-revalidate') {
    event.respondWith(staleWhileRevalidate(request, RUNTIME_API));
    return;
  }
});
