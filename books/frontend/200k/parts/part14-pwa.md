# Part 14: Progressive Web Apps

*Making FicHub work offline and feel like a native app.*

---

## Chapter 53: What Is a Progressive Web App?

### The Best of Both Worlds

A Progressive Web App (PWA) combines the best features of websites and native apps:

**From websites:**
- No installation required (just visit the URL)
- Instant updates (no app store approval)
- Works on any device with a browser
- Shareable via URL

**From native apps:**
- Works offline
- Home screen icon
- Splash screen
- Push notifications
- Fast loading (cached resources)

### How PWAs Work

A PWA needs three things:
1. **HTTPS** — Must be served over a secure connection
2. **Web App Manifest** — Describes the app to the browser
3. **Service Worker** — Intercepts network requests and caches resources

### The Web App Manifest

The manifest tells the browser how your app should behave when installed:

```json
// static/manifest.json
{
  "name": "FicHub - Fanfiction Downloads",
  "short_name": "FicHub",
  "description": "Download fanfiction as EPUB, PDF, and more",
  "start_url": "/",
  "display": "standalone",
  "background_color": "#0f1117",
  "theme_color": "#3b82f6",
  "icons": [
    {
      "src": "/icons/icon-192.png",
      "sizes": "192x192",
      "type": "image/png"
    },
    {
      "src": "/icons/icon-512.png",
      "sizes": "512x512",
      "type": "image/png"
    },
    {
      "src": "/icons/icon-maskable.png",
      "sizes": "512x512",
      "type": "image/png",
      "purpose": "maskable"
    }
  ]
}
```

```html
<!-- In your app.html -->
<link rel="manifest" href="/manifest.json" />
<meta name="theme-color" content="#3b82f6" />
<link rel="apple-touch-icon" href="/icons/icon-192.png" />
```

### The Service Worker

A service worker is a background script that intercepts network requests:

```typescript
// static/sw.js
const CACHE_NAME = 'fichub-v1';
const STATIC_ASSETS = [
  '/',
  '/search',
  '/offline.html',
  '/manifest.json',
  '/icons/icon-192.png',
  '/icons/icon-512.png'
];

// Install: Cache static assets
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {
      return cache.addAll(STATIC_ASSETS);
    })
  );
  self.skipWaiting();
});

// Activate: Clean up old caches
self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) => {
      return Promise.all(
        keys
          .filter((key) => key !== CACHE_NAME)
          .map((key) => caches.delete(key))
      );
    })
  );
  self.clients.claim();
});

// Fetch: Cache-first for static assets, network-first for API
self.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);

  // API requests: network-first
  if (url.pathname.startsWith('/api/')) {
    event.respondWith(
      fetch(request)
        .then((response) => {
          const clone = response.clone();
          caches.open(CACHE_NAME).then((cache) => {
            cache.put(request, clone);
          });
          return response;
        })
        .catch(() => {
          return caches.match(request);
        })
    );
    return;
  }

  // Static assets: cache-first
  event.respondWith(
    caches.match(request).then((cached) => {
      return cached || fetch(request).then((response) => {
        const clone = response.clone();
        caches.open(CACHE_NAME).then((cache) => {
          cache.put(request, clone);
        });
        return response;
      });
    })
  );
});
```

### Registering the Service Worker

```svelte
<!-- +layout.svelte -->
<script>
  import { onMount } from 'svelte';

  onMount(async () => {
    if ('serviceWorker' in navigator) {
      try {
        const registration = await navigator.serviceWorker.register('/sw.js');
        console.log('Service Worker registered:', registration.scope);
      } catch (error) {
        console.error('Service Worker registration failed:', error);
      }
    }
  });
</script>
```

### Caching Strategies

Different resources need different strategies:

1. **Cache-First** — Check cache first, fall back to network. Good for static assets (images, fonts, CSS, JS).

2. **Network-First** — Try network first, fall back to cache. Good for API responses where freshness matters.

3. **Stale-While-Revalidate** — Serve from cache, update in background. Good for content that can be slightly stale.

4. **Network-Only** — Always go to network. Good for sensitive data.

5. **Cache-Only** — Always use cache. Good for offline-only content.

### Offline Support

Create an offline fallback page:

```html
<!-- static/offline.html -->
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>FicHub - Offline</title>
  <style>
    body {
      font-family: system-ui, sans-serif;
      background: #0f1117;
      color: #f3f4f6;
      display: flex;
      justify-content: center;
      align-items: center;
      min-height: 100vh;
      margin: 0;
    }
    .offline {
      text-align: center;
      padding: 2rem;
    }
    .offline h1 { font-size: 3rem; margin-bottom: 1rem; }
    .offline p { color: #9ca3af; margin-bottom: 2rem; }
    .offline a {
      color: #3b82f6;
      text-decoration: none;
    }
  </style>
</head>
<body>
  <div class="offline">
    <h1>📡</h1>
    <h2>You're Offline</h2>
    <p>FicHub needs an internet connection to download stories.</p>
    <p>Please check your connection and try again.</p>
    <a href="/" onclick="location.reload()">Try Again</a>
  </div>
</body>
</html>
```

### Practice Exercises

1. **PWA Setup:** Convert an existing SvelteKit app to a PWA. Add a manifest, service worker, and offline support.

2. **Caching Strategy:** Implement different caching strategies for different resource types. Test with Chrome DevTools Application tab.

3. **Offline Indicator:** Build an offline indicator component that shows when the user loses connection.

---

## Chapter 54: Advanced PWA Features

### Background Sync

Background Sync lets you defer actions until the user has connectivity:

```typescript
// In your service worker
self.addEventListener('sync', (event) => {
  if (event.tag === 'sync-suggestions') {
    event.waitUntil(syncSuggestions());
  }
});

async function syncSuggestions() {
  const db = await openDB();
  const pending = await db.getAll('pending-suggestions');
  
  for (const suggestion of pending) {
    try {
      await fetch('/api/v0/recommendations/suggest', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(suggestion)
      });
      await db.delete('pending-suggestions', suggestion.id);
    } catch (e) {
      console.error('Sync failed:', e);
    }
  }
}
```

```typescript
// In your app
async function submitSuggestionOffline(data: SuggestionData) {
  // Store locally
  const db = await openDB();
  await db.add('pending-suggestions', data);
  
  // Request background sync
  if ('serviceWorker' in navigator && 'SyncManager' in window) {
    const registration = await navigator.serviceWorker.ready;
    await registration.sync.register('sync-suggestions');
  }
}
```

### Push Notifications

Push notifications let you re-engage users even when they're not on your site:

```typescript
// Request permission
async function requestNotificationPermission() {
  const permission = await Notification.requestPermission();
  return permission === 'granted';
}

// Subscribe to push
async function subscribeToPush() {
  const registration = await navigator.serviceWorker.ready;
  const subscription = await registration.pushManager.subscribe({
    userVisibleOnly: true,
    applicationServerKey: urlBase64ToUint8Array(VAPID_PUBLIC_KEY)
  });
  
  // Send subscription to server
  await fetch('/api/v0/notifications/subscribe', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(subscription)
  });
}
```

### App Install Prompt

Handle the beforeinstallprompt event to show a custom install button:

```svelte
<script>
  let deferredPrompt: any = $state(null);
  let showInstallButton = $state(false);

  onMount(() => {
    window.addEventListener('beforeinstallprompt', (e) => {
      e.preventDefault();
      deferredPrompt = e;
      showInstallButton = true;
    });

    window.addEventListener('appinstalled', () => {
      showInstallButton = false;
      deferredPrompt = null;
    });
  });

  async function installApp() {
    if (!deferredPrompt) return;
    deferredPrompt.prompt();
    const { outcome } = await deferredPrompt.userChoice;
    console.log('Install outcome:', outcome);
    deferredPrompt = null;
    showInstallButton = false;
  }
</script>

{#if showInstallButton}
  <button onclick={installApp} class="install-btn">
    📲 Install FicHub
  </button>
{/if}
```

### Practice Exercises

1. **Background Sync:** Implement background sync for the suggestion form. Test by going offline, submitting a suggestion, and coming back online.

2. **Push Notifications:** Set up push notifications for when a subscribed story is updated.

3. **Install Prompt:** Build a custom install prompt that shows after the user has downloaded 3 stories.
