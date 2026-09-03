<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { listBookmarks, removeBookmark, getWork, authHeaders } from '$lib/api/social';
  import type { Bookmark } from '$lib/api/social-types';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { formatWords, detectSite, relativeTime } from '$lib/util';
  import { getPref } from '$lib/prefs';
  import WorkBlurb from '$lib/ui/archive/WorkBlurb.svelte';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';

  const uiMode = $derived(getPref('uiMode'));

  let bookmarks = $state<Bookmark[]>([]);
  let loading = $state(true);
  let error = $state('');
  let showMode = $state<'list' | 'search'>('list');
  let metadata = $state<Record<number, { title: string; author: string; words: number; chapters: number; source: string }>>({});
  let removing = $state<number | null>(null);
  let exporting = $state(false);
  let importing = $state(false);
  let importQueued = $state(false);
  let importError = $state('');

  onMount(async () => {
    // Sync internal showMode from the URL ?tab= on initial load so deep
    // links (e.g. /bookmarks?tab=search) restore the correct tab.
    const tab = $page.url.searchParams.get('tab');
    if (tab === 'list' || tab === 'search') {
      showMode = tab;
    }
    await auth.init();
    if (!auth.isLoggedIn) {
      loading = false;
      return;
    }
    await loadBookmarks();
  });

  // Keep showMode in the URL (?tab=list|search) so tabs are bookmarkable.
  // Uses $effect (runes-aware) with history.replaceState to avoid a full
  // navigation reload — the SvelteKit page store already reflects the URL.
  $effect(() => {
    if (typeof history === 'undefined') return;
    const url = new URL(window.location.href);
    if (url.searchParams.get('tab') !== showMode) {
      url.searchParams.set('tab', showMode);
      history.replaceState(null, '', url.toString());
    }
  });

  // Reactive helper for tab links — replaces pushState navigation when the
  // user clicks a tab, so the page store updates and the URL stays in sync
  // without a full reload.
  function switchTab(mode: 'list' | 'search') {
    showMode = mode;
    goto(`?tab=${mode}`, { replaceState: true });
  }

  async function loadBookmarks() {
    loading = true;
    error = '';
    try {
      const res = await listBookmarks();
      if (res.err !== 0) {
        error = 'Failed to load bookmarks.';
        return;
      }
      bookmarks = res.bookmarks;
      // Fetch metadata for each bookmark in parallel via /api/works/{id}.
      const entries = await Promise.allSettled(
        bookmarks.map(async (b) => {
          const data = await getWork(b.work_id);
          return { work_id: b.work_id, work: data.work };
        }),
      );
      for (const entry of entries) {
        if (entry.status === 'fulfilled' && entry.value.work) {
          const w = entry.value.work;
          metadata[entry.value.work_id] = {
            title: w.canonical_title,
            author: w.canonical_author,
            words: w.sources[0]?.words ?? 0,
            chapters: w.sources[0]?.chapters ?? 0,
            source: w.sources[0]?.source ?? '',
          };
        }
      }
    } catch {
      error = 'Network error loading bookmarks.';
    } finally {
      loading = false;
    }
  }

  async function handleExport() {
    exporting = true;
    try {
      const headers: Record<string, string> = { ...authHeaders() };

      const res = await fetch('/api/bookmarks/export', { headers });
      if (!res.ok) throw new Error('Export failed');

      const blob = await res.blob();
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'ficnexus-bookmarks.csv';
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      error = 'Export failed.';
    } finally {
      exporting = false;
    }
  }

  let fileInput: HTMLInputElement | undefined = $state();

  function triggerImport() {
    fileInput?.click();
  }

  async function handleImportFile(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    importing = true;
    importQueued = false;
    importError = '';
    try {
      const headers: Record<string, string> = { ...authHeaders() };

      const formData = new FormData();
      formData.append('file', file);

      const res = await fetch('/api/bookmarks/import', {
        method: 'POST',
        headers,
        body: formData,
      });

      if (!res.ok) {
        error = 'Import failed.';
        return;
      }

      const data = await res.json();
      if (data.err !== 0) {
        importError = data.msg || 'Import failed.';
        return;
      }
      // Background import: the server queues the job and notifies the user
      // when it finishes; show queued status instead of a final count.
      importQueued = true;
      // Refresh bookmarks after a short delay so the notification bell
      // picks up the queued state; the list itself updates on next visit.
      await loadBookmarks();
    } catch (e) {
      error = 'Import failed.';
    } finally {
      importing = false;
      input.value = ''; // reset
    }
  }

  async function handleRemove(work_id: number) {
    removing = work_id;
    try {
      const res = await removeBookmark(work_id);
      if (res.err === 0) {
        bookmarks = bookmarks.filter((b) => b.work_id !== work_id);
      }
    } catch {
      error = 'Failed to remove bookmark.';
    } finally {
      removing = null;
    }
  }


</script>

<div class="bookmarks-page">
  {#if uiMode === 'archive'}
    <!-- ── Archive mode ─────────────────────────────────────────── -->
<header class="archive-header">
      <h1 class="archive-title">Bookmarks</h1>
      <p class="bookmark-subnav">
        <a
          class="bookmark-tab"
          class:active={showMode === 'list'}
          href="/bookmarks?tab=list"
          onclick={(e) => { e.preventDefault(); switchTab('list'); }}
        >Bookmarks</a>
        <a
          class="bookmark-tab"
          class:active={showMode === 'search'}
          href="/bookmarks?tab=search"
          onclick={(e) => { e.preventDefault(); switchTab('search'); }}
        >Search Bookmarks</a>
      </p>
    </header>

    {#if !auth.isLoggedIn}
      <div class="archive-empty">
        <p>Log in to see your bookmarks.</p>
        <a href="/login">Log In</a>
      </div>
    {:else if showMode === 'search'}
      <!-- Bookmark Search form (AO3-style separate search) -->
      <div class="bookmark-search-form">
        <h3 class="landmark heading">Bookmark Search</h3>
        <dl>
          <div class="dl-row">
            <dt><label for="bsearch-bookmark">Bookmark</label></dt>
            <dd><input id="bsearch-bookmark" type="search" placeholder="Search your bookmarks" aria-label="Search bookmarks" /></dd>
          </div>
          <div class="dl-row">
            <dt><label for="bsearch-bookmarker">Bookmarker</label></dt>
            <dd><input id="bsearch-bookmarker" type="search" placeholder="Who bookmarked it" aria-label="Search by bookmarker" /></dd>
          </div>
          <div class="dl-row">
            <dt><label for="bsearch-work">Work</label></dt>
            <dd><input id="bsearch-work" type="search" placeholder="Work title or author" aria-label="Search by work" /></dd>
          </div>
          <div class="dl-row">
            <dt><label for="bsearch-fandom">Fandom</label></dt>
            <dd><input id="bsearch-fandom" type="search" placeholder="Fandom name" aria-label="Search by fandom" /></dd>
          </div>
        </dl>
        <p class="submit actions">
          <ArchiveButton type="submit" onclick={() => {}}>Search</ArchiveButton>
        </p>
      </div>
    {:else if loading}
      <div class="archive-skeleton-list">
        {#each Array(3) as _, i}
          <article class="skeleton-blurb" aria-hidden="true">
            <div class="skel-header">
              <span class="skel-badge"></span>
              <span class="skel-title"></span>
            </div>
            <span class="skel-byline"></span>
            <div class="skel-stats"></div>
          </article>
        {/each}
      </div>
    {:else if error}
      <div class="archive-error">
        <p>{error}</p>
      </div>
    {:else if bookmarks.length === 0}
      <div class="archive-empty">
        <p>No bookmarks yet.</p>
      </div>
    {:else}
      <div class="archive-bookmark-list">
        {#each bookmarks as b (b.work_id)}
          {@const meta = metadata[b.work_id]}
          {#if meta}
            <WorkBlurb
              fic={{
                url_id: '',
                title: meta.title,
                author: meta.author,
                words: meta.words,
                chapters: meta.chapters,
                source: meta.source,
                status: '',
                description: '',
                updated: '',
                tags: [],
              }}
            />
          {:else}
            <article class="archive-fallback-card">
              <div class="fallback-header">
                <span class="fallback-id">Work #{b.work_id}</span>
                <button
                  class="archive-remove-btn"
                  onclick={() => handleRemove(b.work_id)}
                  disabled={removing === b.work_id}
                  aria-label="Remove bookmark"
                >
                  {#if removing === b.work_id}
                    <span class="skel-spinner"></span>
                  {:else}
                    Remove
                  {/if}
                </button>
              </div>
              {#if b.notes}
                <p class="fallback-notes">[{b.notes}]</p>
              {/if}
              <p class="fallback-hint">Work metadata unavailable.</p>
            </article>
          {/if}
        {/each}
      </div>
    {/if}

  {:else}
    <!-- ── Modern mode (existing) ────────────────────────────────── -->
    <h1>{t('bookmarks.title')}</h1>

    {#if auth.isLoggedIn}
      <div class="bookmark-toolbar">
        <button class="btn btn-secondary sm" onclick={handleExport} disabled={exporting}>
          {#if exporting}
            <span class="spinner"></span> {t('bookmarks.exporting')}
          {:else}
            {t('bookmarks.exportCsv')}
          {/if}
        </button>
        <button class="btn btn-secondary sm" onclick={triggerImport} disabled={importing}>
          {#if importing}
            <span class="spinner"></span> {t('bookmarks.importing')}
          {:else}
            {t('bookmarks.importCsv')}
          {/if}
        </button>
        <input
          type="file"
          accept=".csv"
          bind:this={fileInput}
          onchange={handleImportFile}
          style="display:none"
        />
      </div>
      {#if importQueued}
        <div class="card import-result">
          <p>{t('bookmarks.importQueued')}</p>
        </div>
      {/if}
      {#if importError}
        <div class="card import-result">
          <p>⚠️ {importError}</p>
        </div>
      {/if}
    {/if}

    {#if !auth.isLoggedIn}
      <div class="card empty">
        <p class="muted">{t('bookmarks.login')}</p>
      </div>
    {:else if loading}
      <div class="card">
        <p class="muted"><span class="spinner"></span> {t('bookmarks.loading')}</p>
      </div>
    {:else if error}
      <div class="card error-card">
        <strong class="error-text">⚠️ {error}</strong>
      </div>
    {:else if bookmarks.length === 0}
      <div class="card empty">
        <p class="muted">{t('bookmarks.empty')}</p>
      </div>
    {:else}
      <p class="muted count">{t(bookmarks.length === 1 ? 'bookmarks.count' : 'bookmarks.countPlural', { count: bookmarks.length })}</p>
      <div class="bookmark-list">
        {#each bookmarks as b (b.work_id)}
          {@const meta = metadata[b.work_id]}
          <div class="card bookmark-item">
            <div class="bookmark-main">
              {#if meta}
                <h3><a href="/work/{b.work_id}">{meta.title}</a></h3>
                <p class="muted">{t('bookmarks.by')} {meta.author} · <span class="tag">{detectSite(meta.source)}</span></p>
                <p class="meta-line">{formatWords(meta.words)} words · {meta.chapters} chapters</p>
              {:else}
                <h3>Work #{b.work_id}</h3>
                <p class="muted"><span class="tag">{t('bookmarks.loadingMeta')}</span></p>
              {/if}
              {#if b.notes}
                <p class="note muted">📝 {b.notes}</p>
              {/if}
              {#if b.is_private}
                <span class="private-badge">🔒 {t('bookmarks.private')}</span>
              {/if}
            </div>
            <div class="bookmark-actions">
              <span class="muted date" title={b.created_at}>
                {t('bookmarks.saved')} {relativeTime(b.created_at)}
              </span>
              <button
                class="btn btn-secondary sm remove-btn"
                onclick={() => handleRemove(b.work_id)}
                disabled={removing === b.work_id}
                aria-label={t('bookmarks.removeAria')}
              >
                {#if removing === b.work_id}
                  <span class="spinner"></span>
                {:else}
                  ✕
                {/if}
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<style>
  .bookmarks-page {
    max-width: var(--max-width);
    margin: 0 auto;
    padding: 1rem;
  }
  h1 {
    margin-top: 0;
    margin-bottom: 1rem;
  }
  .count {
    font-size: 0.9rem;
    margin-bottom: 0.8rem;
  }
  .empty {
    text-align: center;
    padding: 2rem;
  }
  .error-card {
    border-color: var(--color-error);
  }
  .bookmark-list {
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
  }
  .bookmark-item {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: flex-start;
  }
  .bookmark-main h3 {
    margin: 0 0 0.3rem;
    font-size: 1.1rem;
  }
  .bookmark-main h3 a {
    color: var(--color-text);
  }
  .meta-line {
    margin: 0.3rem 0;
    font-size: 0.9rem;
  }
  .note {
    margin: 0.4rem 0 0;
    font-size: 0.85rem;
  }
  .private-badge {
    display: inline-block;
    margin-top: 0.3rem;
    font-size: 0.75rem;
    color: var(--color-warning);
    background: var(--color-surface-2);
    border: 1px solid var(--color-warning);
    border-radius: 999px;
    padding: 0.1rem 0.5rem;
  }
  .bookmark-actions {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 0.4rem;
    flex-shrink: 0;
  }
  .date {
    font-size: 0.78rem;
    white-space: nowrap;
  }
  .remove-btn {
    padding: 0.3rem 0.5rem;
    font-size: 0.8rem;
    background: var(--color-surface-2);
    color: var(--color-muted);
    border: 1px solid var(--color-border);
  }
  .remove-btn:hover {
    color: var(--color-error);
    border-color: var(--color-error);
  }
  .btn.sm {
    padding: 0.35rem 0.7rem;
    font-size: 0.82rem;
  }
  .bookmark-toolbar {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
    flex-wrap: wrap;
  }
  .import-result {
    margin-bottom: 1rem;
    padding: 0.8rem;
  }
  .import-result p {
    margin: 0;
  }
  .error-list {
    margin: 0.3rem 0 0;
    padding-left: 1.2rem;
    font-size: 0.85rem;
  }
  .spinner {
    display: inline-block;
    width: 0.8rem;
    height: 0.8rem;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
    vertical-align: middle;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  /* ── Archive mode styles ──────────────────────────────────────── */
  .archive-header {
    padding-bottom: 0.6em;
    border-bottom: 2px solid var(--archive-link, #990000);
    margin-bottom: 1em;
  }

  .archive-title {
    font-size: 1.3em;
    font-weight: 700;
    margin: 0;
    color: var(--archive-text, #2a2a2a);
  }

  .archive-empty {
    padding: 2em 1em;
    text-align: center;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
  }

  .archive-empty p {
    font-size: 0.95em;
    color: var(--archive-muted, #666666);
    font-style: italic;
    margin: 0 0 0.5em;
  }

  .archive-empty a {
    color: var(--archive-link, #990000);
    text-decoration: none;
  }

  .archive-empty a:hover {
    text-decoration: underline;
  }

  .archive-error {
    padding: 1.5em 1em;
    text-align: center;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-text, #2a2a2a);
  }

  .archive-bookmark-list {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .archive-fallback-card {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.75em 1em;
    margin-bottom: 0.75em;
    background: var(--archive-bg, #ffffff);
  }

  .fallback-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.3em;
  }

  .fallback-id {
    font-size: 0.9em;
    font-weight: 700;
    color: var(--archive-text, #2a2a2a);
  }

  .fallback-notes {
    font-size: 0.85em;
    font-weight: 600;
    color: var(--archive-link, #990000);
    margin: 0.2em 0;
    font-style: italic;
  }

  .fallback-hint {
    font-size: 0.8em;
    color: var(--archive-muted, #666666);
    font-style: italic;
    margin: 0;
  }

  .archive-remove-btn {
    display: inline-block;
    padding: 0.2em 0.5em;
    font-size: 0.8em;
    font-family: inherit;
    font-weight: 600;
    line-height: 1.5;
    color: var(--archive-muted, #666666);
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    cursor: pointer;
    transition: background 0.15s, border-color 0.15s;
  }

  .archive-remove-btn:hover {
    color: #cc0000;
    border-color: #cc0000;
    background: var(--archive-border, #dddddd);
  }

  .archive-remove-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* ── Archive skeleton ─────────────────────────────────────────── */
  .archive-skeleton-list {
    display: flex;
    flex-direction: column;
  }

  .skeleton-blurb {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.75em 1em;
    margin-bottom: 0.75em;
    background: var(--archive-bg, #ffffff);
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .skel-header {
    display: flex;
    align-items: center;
    gap: 0.5em;
    margin-bottom: 0.4em;
  }

  .skel-badge {
    display: inline-block;
    width: 2em;
    height: 1.2em;
    background: var(--archive-border, #dddddd);
    border-radius: 2px;
  }

  .skel-title {
    display: inline-block;
    width: 50%;
    height: 1.2em;
    background: var(--archive-border, #dddddd);
    border-radius: 2px;
  }

  .skel-byline {
    display: block;
    width: 30%;
    height: 0.9em;
    background: var(--archive-border, #dddddd);
    border-radius: 2px;
    margin-bottom: 0.4em;
  }

  .skel-stats {
    width: 40%;
    height: 0.85em;
    background: var(--archive-border, #dddddd);
    border-radius: 2px;
  }

  .skel-spinner {
    display: inline-block;
    width: 0.8rem;
    height: 0.8rem;
    border: 2px solid var(--archive-border, #dddddd);
    border-top-color: var(--archive-link, #990000);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
    vertical-align: middle;
  }

  .archive-header {
    margin-bottom: 0.5em;
  }

  .bookmark-subnav {
    margin: 0;
    padding: 0;
    display: flex;
    gap: 0.3em;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    margin-bottom: 1em;
  }

  .bookmark-tab {
    display: inline-block;
    padding: 0.3em 0.9em;
    font-size: 0.88em;
    font-weight: 600;
    color: var(--archive-muted, #666666);
    text-decoration: none;
    border: 1px solid transparent;
    border-bottom: none;
    border-radius: 3px 3px 0 0;
    font-family: inherit;
  }

  .bookmark-tab.active {
    color: var(--archive-link, #990000);
    border-color: var(--archive-border, #dddddd);
    border-bottom-color: var(--archive-bg, #ffffff);
    background: var(--archive-bg, #ffffff);
  }

  .bookmark-tab:hover {
    color: var(--archive-link, #990000);
    border-color: var(--archive-border, #dddddd);
    border-bottom-color: var(--archive-bg, #ffffff);
    background: var(--archive-bg-raised, #f5f5f5);
  }

  .bookmark-search-form {
    margin: 1em 0;
  }

  .bookmark-search-form dl {
    margin: 0;
  }

  .bookmark-search-form .dl-row {
    display: flex;
    align-items: center;
    gap: 0.5em;
    margin-bottom: 0.4em;
  }

  .bookmark-search-form .dl-row dt {
    font-weight: 600;
    color: var(--archive-muted, #666666);
    margin: 0;
    width: 100px;
    min-width: 90px;
  }

  .bookmark-search-form .dl-row dd {
    margin: 0;
  }

  .bookmark-search-form input {
    width: 100%;
    max-width: 340px;
    padding: 0.3em 0.5em;
    font-size: 0.9em;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
    font-family: inherit;
  }

  .bookmark-search-form input:focus {
    outline: 1px solid var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }

  .bookmark-search-form .submit.actions {
    margin-top: 0.4em;
  }

  @media (max-width: 600px) {
    .bookmark-search-form .dl-row {
      flex-direction: column;
      align-items: flex-start;
      gap: 0.2em;
    }
    .bookmark-search-form .dl-row dt {
      width: 100%;
    }
    .bookmark-tab {
      font-size: 0.82em;
      padding: 0.25em 0.6em;
    }
  }
  @media (max-width: 600px) {
    .bookmark-item {
      flex-direction: column;
    }
    .bookmark-actions {
      flex-direction: row;
      align-items: center;
      width: 100%;
      justify-content: space-between;
    }
  }
</style>
