<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { listLocales } from '$lib/api/social';
  import type { Locale } from '$lib/api/social-types';
  import { i18n, setLocale, STORAGE_KEY } from '$lib/i18n/index.svelte';

  let locales = $state<Locale[]>([]);
  let selectedLocale = $state('en');
  let saving = $state(false);

  onMount(async () => {
    await auth.init();
    // Read from localStorage first, fall back to user profile / i18n store
    selectedLocale = localStorage.getItem(STORAGE_KEY) || auth.user?.locale || i18n.locale || 'en';
    try {
      const res = await listLocales();
      if (res.err === 0) {
        locales = res.locales;
      }
    } catch { /* silent */ }
  });

  async function changeLocale(e: Event) {
    const code = (e.target as HTMLSelectElement).value;
    // Switch the UI language immediately (reactive store), then persist.
    // setLocale() validates the code against the supported locales and
    // updates the reactive store + localStorage; the PUT below syncs the
    // server-side preference (best effort).
    if (setLocale(code)) {
      saving = true;
      try {
        const res = await fetch('/api/auth/locale', {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ locale: code }),
          credentials: 'include',
        });
        if (res.ok) {
          selectedLocale = code;
          localStorage.setItem(STORAGE_KEY, code);
        }
      } catch { /* silent */ }
      finally { saving = false; }
    }
  }</script>

<div class="locale-selector">
  <label for="locale-select">🌐</label>
  <select id="locale-select" onchange={changeLocale} disabled={saving}>
    {#each locales as loc}
      <option value={loc.code} selected={loc.code === selectedLocale}>{loc.name}</option>
    {/each}
  </select>
  {#if saving}<span class="saving">saving…</span>{/if}
</div>

<style>
  .locale-selector {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.2rem 0;
  }
  .locale-selector select {
    padding: 0.25rem 0.5rem;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface-2);
    color: var(--color-text);
    font-size: 0.82rem;
    max-width: 120px;
  }
  .saving {
    font-size: 0.75rem;
    color: var(--color-muted);
    animation: pulse 0.8s ease-in-out infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }
</style>
