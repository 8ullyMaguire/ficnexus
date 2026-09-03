/**
 * Register the FicNexus service worker (offline reading PWA).
 *
 * Guards: browser only, secure context (SW requires HTTPS or localhost),
 * and only registers in production builds — `vite dev` serves the raw
 * `static/sw.js` file rather than the built app shell, so a SW in dev
 * would cache a broken/partial shell. Call from `+layout.ts`.
 */
export function registerServiceWorker(): void {
  if (typeof window === 'undefined') return;
  if (!('serviceWorker' in navigator)) return;
  if (typeof window.isSecureContext === 'boolean' && !window.isSecureContext) return;
  if (import.meta.env.DEV) return;

  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/sw.js').catch((err) => {
      console.error('[pwa] service worker registration failed:', err);
    });
  });
}
