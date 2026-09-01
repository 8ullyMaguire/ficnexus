# Expansion Material: Complete Project Walkthroughs and Advanced Patterns

*Step-by-step project builds, advanced patterns, and comprehensive exercises.*

---

## Complete Project: Building a URL Preview System

### Overview

FicHub needs to show previews of fanfiction stories when users paste a URL. Let's build a complete URL preview system that fetches metadata and displays it beautifully.

### Step 1: Define the Preview Types

```typescript
// lib/types/preview.ts
export interface UrlPreview {
  url: string;
  title: string;
  author: string;
  site: string;
  description: string;
  wordCount: number;
  chapters: number;
  status: 'complete' | 'in-progress' | 'abandoned';
  rating: string;
  tags: string[];
  lastUpdated: string;
  coverImage?: string;
}

export interface PreviewState {
  status: 'idle' | 'loading' | 'success' | 'error';
  preview: UrlPreview | null;
  error: string | null;
}
```

### Step 2: Build the Preview Fetcher

```typescript
// lib/api/preview.ts
import type { UrlPreview } from '$lib/types/preview';

const PREVIEW_CACHE = new Map<string, { data: UrlPreview; timestamp: number }>();
const CACHE_TTL = 5 * 60 * 1000; // 5 minutes

export async function fetchPreview(url: string): Promise<UrlPreview> {
  // Check cache first
  const cached = PREVIEW_CACHE.get(url);
  if (cached && Date.now() - cached.timestamp < CACHE_TTL) {
    return cached.data;
  }

  const response = await fetch(`/api/v0/meta?q=${encodeURIComponent(url)}`);
  
  if (!response.ok) {
    throw new Error(`Failed to fetch preview: ${response.status}`);
  }
  
  const data = await response.json();
  
  if (data.err !== 0) {
    throw new Error(data.msg || 'Failed to fetch preview');
  }
  
  const preview: UrlPreview = {
    url: data.q,
    title: data.meta?.title || 'Unknown Title',
    author: data.meta?.author || 'Unknown Author',
    site: detectSite(data.meta?.source || ''),
    description: stripHtml(data.meta?.description || ''),
    wordCount: data.meta?.words || 0,
    chapters: data.meta?.chapters || 0,
    status: data.meta?.status || 'unknown',
    rating: data.meta?.rating || 'unknown',
    tags: data.meta?.tags || [],
    lastUpdated: data.meta?.updated || '',
    coverImage: data.meta?.cover_image
  };
  
  // Cache the result
  PREVIEW_CACHE.set(url, { data: preview, timestamp: Date.now() });
  
  return preview;
}

function detectSite(source: string): string {
  if (source.includes('archiveofourown.org')) return 'AO3';
  if (source.includes('fanfiction.net')) return 'FFN';
  if (source.includes('fictionpress.com')) return 'FP';
  if (source.includes('spacebattles.com')) return 'SB';
  if (source.includes('sufficientvelocity.com')) return 'SV';
  return 'Unknown';
}

function stripHtml(html: string): string {
  return html
    .replace(/<br\s*\/?>/gi, ' ')
    .replace(/<\/(p|div)>/gi, ' ')
    .replace(/<[^>]+>/g, '')
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/\s+/g, ' ')
    .trim();
}
```

### Step 3: Build the Preview Component

```svelte
<!-- UrlPreview.svelte -->
<script lang="ts">
  import { fly } from 'svelte/transition';
  import { fetchPreview } from '$lib/api/preview';
  import type { UrlPreview as PreviewType } from '$lib/types/preview';
  
  let { url, onDownload } = $props();
  
  let state = $state<{
    status: 'idle' | 'loading' | 'success' | 'error';
    preview: PreviewType | null;
    error: string | null;
  }>({
    status: 'idle',
    preview: null,
    error: null
  });
  
  let debounceTimer: ReturnType<typeof setTimeout>;
  
  // Watch for URL changes
  $effect(() => {
    if (url) {
      clearTimeout(debounceTimer);
      debounceTimer = setTimeout(() => loadPreview(url), 500);
    } else {
      state = { status: 'idle', preview: null, error: null };
    }
    
    return () => clearTimeout(debounceTimer);
  });
  
  async function loadPreview(url: string) {
    state = { status: 'loading', preview: null, error: null };
    
    try {
      const preview = await fetchPreview(url);
      state = { status: 'success', preview, error: null };
    } catch (e) {
      state = { 
        status: 'error', 
        preview: null, 
        error: e instanceof Error ? e.message : 'Failed to load preview' 
      };
    }
  }
  
  function formatWords(words: number): string {
    return words.toLocaleString();
  }
  
  function getStatusColor(status: string): string {
    switch (status) {
      case 'complete': return 'var(--color-success)';
      case 'in-progress': return 'var(--color-warning)';
      case 'abandoned': return 'var(--color-error)';
      default: return 'var(--color-muted)';
    }
  }
</script>

{#if state.status === 'loading'}
  <div class="preview-card loading" transition:fly={{ y: 10, duration: 200 }}>
    <div class="skeleton skeleton-title"></div>
    <div class="skeleton skeleton-text"></div>
    <div class="skeleton skeleton-text short"></div>
  </div>
{:else if state.status === 'error'}
  <div class="preview-card error" transition:fly={{ y: 10, duration: 200 }}>
    <p class="error-text">⚠️ {state.error}</p>
    <p class="error-hint">Please check the URL and try again.</p>
  </div>
{:else if state.status === 'success' && state.preview}
  {@const p = state.preview}
  <div class="preview-card" transition:fly={{ y: 10, duration: 200 }}>
    <div class="preview-header">
      <h3 class="preview-title">{p.title}</h3>
      <span class="preview-site">{p.site}</span>
    </div>
    
    <p class="preview-author">by {p.author}</p>
    
    <div class="preview-meta">
      <span>{formatWords(p.wordCount)} words</span>
      <span class="separator">·</span>
      <span>{p.chapters} chapters</span>
      <span class="separator">·</span>
      <span style="color: {getStatusColor(p.status)}">{p.status}</span>
    </div>
    
    {#if p.description}
      <p class="preview-desc">{p.description.slice(0, 250)}{p.description.length > 250 ? '...' : ''}</p>
    {/if}
    
    {#if p.tags.length > 0}
      <div class="preview-tags">
        {#each p.tags.slice(0, 5) as tag}
          <span class="tag">{tag}</span>
        {/each}
        {#if p.tags.length > 5}
          <span class="tag-more">+{p.tags.length - 5}</span>
        {/if}
      </div>
    {/if}
    
    <div class="preview-actions">
      <button class="btn btn-primary" onclick={() => onDownload?.(p.url)}>
        Download
      </button>
      <a class="btn btn-secondary" href={p.url} target="_blank" rel="noopener">
        View Original
      </a>
    </div>
  </div>
{/if}

<style>
  .preview-card {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: 1.5rem;
    transition: border-color 0.2s ease;
  }
  
  .preview-card:hover {
    border-color: var(--color-primary);
  }
  
  .preview-card.loading {
    opacity: 0.7;
  }
  
  .preview-card.error {
    border-color: var(--color-error);
  }
  
  .skeleton {
    background: var(--color-surface-2);
    border-radius: var(--radius);
    animation: pulse 1.5s ease-in-out infinite;
  }
  
  .skeleton-title {
    height: 1.5rem;
    width: 60%;
    margin-bottom: 0.75rem;
  }
  
  .skeleton-text {
    height: 1rem;
    width: 100%;
    margin-bottom: 0.5rem;
  }
  
  .skeleton-text.short {
    width: 40%;
  }
  
  .preview-header {
    display: flex;
    justify-content: space-between;
    align-items: start;
    gap: 1rem;
    margin-bottom: 0.5rem;
  }
  
  .preview-title {
    margin: 0;
    font-size: 1.25rem;
    color: var(--color-text);
  }
  
  .preview-site {
    font-size: 0.75rem;
    padding: 0.2rem 0.5rem;
    background: var(--color-surface-2);
    border-radius: var(--radius-full);
    color: var(--color-muted);
    white-space: nowrap;
  }
  
  .preview-author {
    color: var(--color-muted);
    font-size: 0.9rem;
    margin: 0 0 0.75rem;
  }
  
  .preview-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    font-size: 0.85rem;
    color: var(--color-muted);
    margin-bottom: 0.75rem;
  }
  
  .separator {
    opacity: 0.5;
  }
  
  .preview-desc {
    color: var(--color-muted);
    font-size: 0.9rem;
    line-height: 1.5;
    margin: 0 0 0.75rem;
  }
  
  .preview-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    margin-bottom: 1rem;
  }
  
  .tag {
    font-size: 0.75rem;
    padding: 0.15rem 0.5rem;
    background: var(--color-surface-2);
    border-radius: var(--radius-full);
    color: var(--color-muted);
  }
  
  .tag-more {
    font-size: 0.75rem;
    color: var(--color-primary);
  }
  
  .preview-actions {
    display: flex;
    gap: 0.75rem;
  }
  
  .btn {
    padding: 0.5rem 1rem;
    border-radius: var(--radius);
    font-weight: 600;
    font-size: 0.9rem;
    text-decoration: none;
    cursor: pointer;
    border: none;
    transition: all 0.2s ease;
  }
  
  .btn-primary {
    background: var(--color-primary);
    color: white;
  }
  
  .btn-primary:hover {
    background: var(--color-primary-hover);
  }
  
  .btn-secondary {
    background: var(--color-surface-2);
    color: var(--color-text);
    border: 1px solid var(--color-border);
  }
  
  .btn-secondary:hover {
    background: var(--color-surface-3);
  }
  
  .error-text {
    color: var(--color-error);
    font-weight: 500;
    margin: 0 0 0.25rem;
  }
  
  .error-hint {
    color: var(--color-muted);
    font-size: 0.85rem;
    margin: 0;
  }
  
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
</style>
```

### Step 4: Use in DownloadTab

```svelte
<!-- Updated DownloadTab.svelte -->
<script lang="ts">
  import UrlPreview from './UrlPreview.svelte';
  
  let url = $state('');
  let loading = $state(false);
  let error = $state('');
  let result = $state(null);

  async function handleDownload() {
    if (!url.trim()) {
      error = 'Please paste a fanfiction URL first.';
      return;
    }
    loading = true;
    error = '';
    result = null;
    try {
      const res = await fetchExport(url.trim());
      if (res.err !== 0) {
        error = res.msg || 'Something went wrong.';
        return;
      }
      result = res;
    } catch (e) {
      error = e instanceof ApiError 
        ? `Server error ${e.status}` 
        : 'Network error.';
    } finally {
      loading = false;
    }
  }
</script>

<div class="download-tab">
  <div class="search-row">
    <input
      type="url"
      placeholder="Paste a fanfiction URL..."
      bind:value={url}
      onkeydown={(e) => e.key === 'Enter' && handleDownload()}
      disabled={loading}
      aria-label="Fanfiction URL"
    />
    <button class="btn" onclick={handleDownload} disabled={loading}>
      {loading ? 'Working...' : 'Download'}
    </button>
  </div>

  <!-- Live preview as user types -->
  <UrlPreview {url} onDownload={(u) => { url = u; handleDownload(); }} />

  {#if error}
    <div class="error-card">
      <strong>⚠️ {error}</strong>
    </div>
  {/if}

  {#if result && result.meta}
    <!-- Existing result display -->
  {/if}
</div>
```

---

## Complete Project: Building a Notification System

### Overview

Build a complete notification system with different notification types, auto-dismiss, stacking, and persistence.

### Step 1: Define Notification Types

```typescript
// lib/types/notification.ts
export interface Notification {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message: string;
  duration?: number;
  dismissible?: boolean;
  action?: {
    label: string;
    onclick: () => void;
  };
  timestamp: number;
}
```

### Step 2: Build the Notification Store

```typescript
// lib/stores/notifications.ts
import type { Notification } from '$lib/types/notification';

let notifications = $state<Notification[]>([]);
let nextId = 0;

function generateId(): string {
  return `notification-${++nextId}-${Date.now()}`;
}

export const notificationStore = {
  get all() { return notifications; },
  get count() { return notifications.length; },
  
  add(notification: Omit<Notification, 'id' | 'timestamp'>): string {
    const id = generateId();
    const newNotification: Notification = {
      ...notification,
      id,
      timestamp: Date.now(),
      dismissible: notification.dismissible ?? true,
      duration: notification.duration ?? 5000
    };
    
    notifications = [...notifications, newNotification];
    
    // Auto-dismiss
    if (newNotification.duration && newNotification.duration > 0) {
      setTimeout(() => this.dismiss(id), newNotification.duration);
    }
    
    return id;
  },
  
  dismiss(id: string) {
    notifications = notifications.filter(n => n.id !== id);
  },
  
  dismissAll() {
    notifications = [];
  },
  
  // Convenience methods
  success(title: string, message: string = '') {
    return this.add({ type: 'success', title, message });
  },
  
  error(title: string, message: string = '') {
    return this.add({ type: 'error', title, message, duration: 0 });
  },
  
  warning(title: string, message: string = '') {
    return this.add({ type: 'warning', title, message });
  },
  
  info(title: string, message: string = '') {
    return this.add({ type: 'info', title, message });
  }
};
```

### Step 3: Build the Notification Container

```svelte
<!-- NotificationContainer.svelte -->
<script lang="ts">
  import { flip } from 'svelte/animate';
  import { fly, fade } from 'svelte/transition';
  import { notificationStore } from '$lib/stores/notifications';
  
  let notifications = $derived(notificationStore.all);
</script>

{#if notifications.length > 0}
  <div class="notification-container" aria-live="polite" aria-atomic="false">
    {#each notifications as notification (notification.id)}
      <div
        class="notification notification-{notification.type}"
        animate:flip={{ duration: 200 }}
        in:fly={{ x: 100, duration: 200 }}
        out:fade={{ duration: 150 }}
        role="alert"
      >
        <div class="notification-icon">
          {#if notification.type === 'success'}✓
          {:else if notification.type === 'error'}✕
          {:else if notification.type === 'warning'}⚠
          {:else}ℹ
          {/if}
        </div>
        
        <div class="notification-content">
          <p class="notification-title">{notification.title}</p>
          {#if notification.message}
            <p class="notification-message">{notification.message}</p>
          {/if}
          {#if notification.action}
            <button 
              class="notification-action"
              onclick={() => notification.action?.onclick()}
            >
              {notification.action.label}
            </button>
          {/if}
        </div>
        
        {#if notification.dismissible}
          <button 
            class="notification-close"
            onclick={() => notificationStore.dismiss(notification.id)}
            aria-label="Dismiss notification"
          >
            ×
          </button>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  .notification-container {
    position: fixed;
    top: 1rem;
    right: 1rem;
    z-index: 1000;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-width: 400px;
    pointer-events: none;
  }
  
  .notification {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 1rem 1.25rem;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    box-shadow: var(--shadow-lg);
    pointer-events: auto;
    animation: slideIn 0.2s ease;
  }
  
  .notification-success { border-left: 3px solid var(--color-success); }
  .notification-error { border-left: 3px solid var(--color-error); }
  .notification-warning { border-left: 3px solid var(--color-warning); }
  .notification-info { border-left: 3px solid var(--color-info); }
  
  .notification-icon {
    font-size: 1.2rem;
    flex-shrink: 0;
    line-height: 1;
  }
  
  .notification-success .notification-icon { color: var(--color-success); }
  .notification-error .notification-icon { color: var(--color-error); }
  .notification-warning .notification-icon { color: var(--color-warning); }
  .notification-info .notification-icon { color: var(--color-info); }
  
  .notification-content {
    flex: 1;
    min-width: 0;
  }
  
  .notification-title {
    margin: 0 0 0.25rem;
    font-weight: 600;
    font-size: 0.9rem;
  }
  
  .notification-message {
    margin: 0;
    font-size: 0.85rem;
    color: var(--color-muted);
  }
  
  .notification-action {
    margin-top: 0.5rem;
    padding: 0.35rem 0.75rem;
    background: transparent;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text);
    font-size: 0.8rem;
    cursor: pointer;
    transition: background 0.15s ease;
  }
  
  .notification-action:hover {
    background: var(--color-surface-2);
  }
  
  .notification-close {
    background: none;
    border: none;
    color: var(--color-muted);
    cursor: pointer;
    font-size: 1.4rem;
    line-height: 1;
    padding: 0;
    flex-shrink: 0;
    transition: color 0.15s ease;
  }
  
  .notification-close:hover {
    color: var(--color-text);
  }
  
  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateX(100%);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }
</style>
```

### Step 4: Usage

```svelte
<!-- AnyComponent.svelte -->
<script>
  import { notificationStore } from '$lib/stores/notifications';
  
  function handleSuccess() {
    notificationStore.success(
      'Download Complete',
      'Your story has been downloaded as EPUB.'
    );
  }
  
  function handleError() {
    notificationStore.error(
      'Download Failed',
      'Could not fetch the story. Please check the URL.'
    );
  }
  
  function handleWithAction() {
    notificationStore.add({
      type: 'info',
      title: 'Update Available',
      message: 'A new version of FicHub is available.',
      duration: 0, // Don't auto-dismiss
      action: {
        label: 'Update Now',
        onclick: () => console.log('Updating...')
      }
    });
  }
</script>

<button onclick={handleSuccess}>Download Success</button>
<button onclick={handleError}>Download Error</button>
<button onclick={handleWithAction}>Update Available</button>
```

---

## Complete Project: Building a Keyboard Shortcut System

### Overview

Power users love keyboard shortcuts. Let's build a system that registers, displays, and handles keyboard shortcuts.

### Step 1: Define Shortcut Types

```typescript
// lib/types/shortcuts.ts
export interface KeyboardShortcut {
  id: string;
  keys: string[];
  description: string;
  category: string;
  action: () => void;
  enabled?: boolean;
  global?: boolean;
}
```

### Step 2: Build the Shortcut Manager

```typescript
// lib/shortcuts/manager.ts
import type { KeyboardShortcut } from '$lib/types/shortcuts';

let shortcuts = $state<KeyboardShortcut[]>([]);
let isEnabled = $state(true);

export const shortcutManager = {
  get all() { return shortcuts; },
  get enabled() { return isEnabled; },
  
  register(shortcut: KeyboardShortcut) {
    shortcuts = [...shortcuts, { ...shortcut, enabled: shortcut.enabled ?? true }];
  },
  
  unregister(id: string) {
    shortcuts = shortcuts.filter(s => s.id !== id);
  },
  
  enable() { isEnabled = true; },
  disable() { isEnabled = false; },
  
  getByCategory(category: string) {
    return shortcuts.filter(s => s.category === category);
  },
  
  formatKeys(keys: string[]): string {
    return keys.map(key => {
      switch (key) {
        case 'Mod':
        case 'Ctrl': return navigator.platform.includes('Mac') ? '⌘' : 'Ctrl';
        case 'Alt': return navigator.platform.includes('Mac') ? '⌥' : 'Alt';
        case 'Shift': return navigator.platform.includes('Mac') ? '⇧' : 'Shift';
        case 'Enter': return '↵';
        case 'Escape': return 'Esc';
        case 'ArrowUp': return '↑';
        case 'ArrowDown': return '↓';
        case 'ArrowLeft': return '←';
        case 'ArrowRight': return '→';
        default: return key.length === 1 ? key.toUpperCase() : key;
      }
    }).join(' + ');
  }
};

// Initialize keyboard listener
if (typeof window !== 'undefined') {
  window.addEventListener('keydown', (e: KeyboardEvent) => {
    if (!isEnabled) return;
    
    const pressedKeys: string[] = [];
    
    if (e.ctrlKey || e.metaKey) pressedKeys.push('Mod');
    if (e.altKey) pressedKeys.push('Alt');
    if (e.shiftKey) pressedKeys.push('Shift');
    
    if (e.key !== 'Control' && e.key !== 'Alt' && e.key !== 'Shift' && e.key !== 'Meta') {
      pressedKeys.push(e.key);
    }
    
    for (const shortcut of shortcuts) {
      if (!shortcut.enabled) continue;
      if (shortcut.keys.length !== pressedKeys.length) continue;
      
      const match = shortcut.keys.every((key, i) => {
        if (key === 'Mod') return pressedKeys.includes('Mod');
        if (key === 'Alt') return pressedKeys.includes('Alt');
        if (key === 'Shift') return pressedKeys.includes('Shift');
        return pressedKeys[i]?.toLowerCase() === key.toLowerCase();
      });
      
      if (match) {
        e.preventDefault();
        shortcut.action();
        break;
      }
    }
  });
}
```

### Step 3: Build the Shortcuts Panel

```svelte
<!-- ShortcutsPanel.svelte -->
<script lang="ts">
  import { shortcutManager } from '$lib/shortcuts/manager';
  
  let { open = $bindable(false) } = $props();
  
  let shortcuts = $derived(shortcutManager.all);
  let categories = $derived([...new Set(shortcuts.map(s => s.category))]);
</script>

{#if open}
  <div class="panel-backdrop" onclick={() => open = false}>
    <div class="panel" onclick={(e) => e.stopPropagation()}>
      <div class="panel-header">
        <h2>Keyboard Shortcuts</h2>
        <button class="close-btn" onclick={() => open = false}>×</button>
      </div>
      
      <div class="panel-body">
        {#each categories as category}
          <div class="shortcut-group">
            <h3>{category}</h3>
            {#each shortcuts.filter(s => s.category === category) as shortcut}
              <div class="shortcut-row">
                <span class="shortcut-desc">{shortcut.description}</span>
                <div class="shortcut-keys">
                  {#each shortcut.keys as key, i}
                    {#if i > 0}<span class="separator">+</span>{/if}
                    <kbd>{shortcutManager.formatKeys([key])}</kbd>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  .panel-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 100;
    padding: 1rem;
  }
  
  .panel {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    width: 100%;
    max-width: 500px;
    max-height: 80vh;
    overflow-y: auto;
    box-shadow: var(--shadow-xl);
  }
  
  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.25rem 1.5rem;
    border-bottom: 1px solid var(--color-border);
  }
  
  .panel-header h2 {
    margin: 0;
    font-size: 1.25rem;
  }
  
  .close-btn {
    background: none;
    border: none;
    font-size: 1.5rem;
    color: var(--color-muted);
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }
  
  .panel-body {
    padding: 1.5rem;
  }
  
  .shortcut-group {
    margin-bottom: 1.5rem;
  }
  
  .shortcut-group:last-child {
    margin-bottom: 0;
  }
  
  .shortcut-group h3 {
    margin: 0 0 0.75rem;
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-muted);
  }
  
  .shortcut-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0;
  }
  
  .shortcut-desc {
    font-size: 0.9rem;
  }
  
  .shortcut-keys {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
  
  kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 1.75rem;
    height: 1.75rem;
    padding: 0 0.4rem;
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: 0.75rem;
    color: var(--color-text);
  }
  
  .separator {
    color: var(--color-muted);
    font-size: 0.75rem;
  }
</style>
```

### Step 4: Register Shortcuts

```svelte
<!-- +layout.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { shortcutManager } from '$lib/shortcuts/manager';
  import ShortcutsPanel from '$lib/components/ShortcutsPanel.svelte';
  
  let showShortcuts = $state(false);
  
  onMount(() => {
    // Register shortcuts
    shortcutManager.register({
      id: 'search',
      keys: ['Mod', 'k'],
      description: 'Open search',
      category: 'Navigation',
      action: () => document.querySelector<HTMLInputElement>('[type="search"]')?.focus()
    });
    
    shortcutManager.register({
      id: 'shortcuts',
      keys: ['Mod', '/'],
      description: 'Show keyboard shortcuts',
      category: 'Help',
      action: () => showShortcuts = !showShortcuts
    });
    
    shortcutManager.register({
      id: 'escape',
      keys: ['Escape'],
      description: 'Close dialog / Cancel',
      category: 'General',
      action: () => showShortcuts = false,
      global: true
    });
  });
</script>

<div class="app">
  <slot />
</div>

<ShortcutsPanel bind:open={showShortcuts} />
```

---

## Complete Exercise Set: Beginner to Expert

### Beginner Exercises (1-10)

1. **Counter App:** Build a counter with increment, decrement, and reset. Display the count with a large font.

2. **Temperature Converter:** Build a form that converts between Celsius and Fahrenheit. Update in real-time.

3. **Tip Calculator:** Build a tip calculator with bill amount, tip percentage, and number of people. Show total and per-person amount.

4. **Color Palette Generator:** Generate random color palettes. Click to copy hex codes.

5. **Markdown Preview:** Build a split-pane editor with markdown on the left and preview on the right.

6. **Quiz App:** Build a multiple-choice quiz with score tracking and explanations.

7. **Stopwatch:** Build a stopwatch with start, stop, reset, and lap functionality.

8. **Random Quote Generator:** Display random quotes with a "New Quote" button.

9. **BMI Calculator:** Build a BMI calculator with height/weight inputs and result interpretation.

10. **Password Generator:** Build a password generator with length, uppercase, lowercase, numbers, and symbols options.

### Intermediate Exercises (11-20)

11. **Search Autocomplete:** Build a search autocomplete with debounced input and dropdown results.

12. **Drag and Drop List:** Build a sortable list with drag and drop reordering.

13. **Infinite Scroll:** Build an infinite scroll list that loads more items from an API.

14. **Multi-Step Form:** Build a multi-step form with validation and progress indicator.

15. **Data Table:** Build a sortable, filterable, paginated data table.

16. **Modal System:** Build a modal dialog with focus trap, escape key, and backdrop click.

17. **Toast Notifications:** Build a toast notification system with different types and auto-dismiss.

18. **Theme Switcher:** Build a light/dark theme switcher with CSS variables and persistence.

19. **Rich Text Editor:** Build a simple rich text editor with bold, italic, and underline.

20. **File Upload:** Build a file upload component with drag-and-drop, preview, and progress.

### Advanced Exercises (21-30)

21. **Real-Time Chat:** Build a chat application with message history and typing indicators.

22. **Kanban Board:** Build a Kanban board with columns, cards, and drag-and-drop.

23. **Dashboard:** Build a dashboard with widgets, charts, and real-time updates.

24. **E-Commerce:** Build an e-commerce frontend with products, cart, and checkout.

25. **Blog Platform:** Build a blog with markdown support, comments, and RSS.

26. **Code Editor:** Build a code editor with syntax highlighting and line numbers.

27. **Video Player:** Build a custom video player with controls, playlist, and quality selection.

28. **Music Player:** Build a music player with playlist, shuffle, repeat, and progress bar.

29. **Calendar:** Build a calendar with event creation, editing, and drag-and-drop.

30. **Graph Visualizer:** Build a graph visualizer with nodes, edges, and layout algorithms.

### Expert Exercises (31-40)

31. **Build a Compiler:** Build a simple expression compiler that parses and evaluates math.

32. **Build a State Machine:** Build a visual state machine editor with simulation.

33. **Build a Virtual DOM:** Implement a simple virtual DOM diffing algorithm.

34. **Build a Router:** Implement a client-side router with nested routes and guards.

35. **Build a Testing Framework:** Implement a testing framework with assertions and mocking.

36. **Build a Database:** Implement a simple in-memory database with queries.

37. **Build a HTTP Server:** Build a simple HTTP server in JavaScript.

38. **Build a WebSocket Server:** Build a real-time WebSocket server for chat.

39. **Build a CSS Framework:** Build a minimal CSS framework with grid and utilities.

40. **Build a Design System:** Build a complete design system with tokens, components, and docs.
