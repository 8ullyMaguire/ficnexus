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
    navigator.serviceWorker.register('/sw.js').then((reg) => {
      // Check for updates periodically.
      setInterval(() => reg.update(), 60 * 60 * 1000); // hourly

      // Listen for a new service worker installing.
      reg.addEventListener('updatefound', () => {
        const newWorker = reg.installing;
        if (!newWorker) return;

        newWorker.addEventListener('statechange', () => {
          if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
            // New content is available — dispatch event for UI.
            window.dispatchEvent(new CustomEvent('sw-update-available'));
          }
        });
      });
    }).catch((err) => {
      console.error('[pwa] service worker registration failed:', err);
    });

    // Reload once when a new SW takes control.
    let refreshing = false;
    navigator.serviceWorker.addEventListener('controllerchange', () => {
      if (refreshing) return;
      refreshing = true;
      window.location.reload();
    });
  });
}
