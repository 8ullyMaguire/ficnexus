<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  interface ChangelogEntry {
    id: number;
    feature_id: number | null;
    title: string;
    body: string;
    kind: 'new' | 'improved' | 'fixed';
    author_id: number;
    published_at: string;
  }

  let entries = $state<ChangelogEntry[]>([]);
  let loading = $state(true);
  let error = $state('');
  let activeKind = $state('all');
  let featureFilter = $state<number | null>(null);

  const KINDS = [
    { id: 'all', label: 'All', icon: '' },
    { id: 'new', label: 'New', icon: '' },
    { id: 'improved', label: 'Improved', icon: '🔧' },
    { id: 'fixed', label: 'Fixed', icon: '🐛' },
  ] as const;

  async function loadChangelog() {
    loading = true;
    error = '';
    try {
      const params = new URLSearchParams();
      if (activeKind !== 'all') params.set('kind', activeKind);
      if (featureFilter) params.set('feature_id', String(featureFilter));
      const res = await fetch(`/api/roadmap/changelog?${params.toString()}`, { credentials: 'include' });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      entries = body.changelog ?? [];
    } catch (e) {
      error = `Failed to load changelog: ${e instanceof Error ? e.message : e}`;
      entries = [];
    } finally {
      loading = false;
    }
  }

  function kindLabel(kind: string): string {
    const k = KINDS.find(x => x.id === kind);
    return k ? `${k.icon} ${k.label}` : kind;
  }

  function formatDate(iso: string): string {
    const d = new Date(iso);
    return d.toLocaleDateString(undefined, { year: 'numeric', month: 'long', day: 'numeric' });
  }

  onMount(async () => {
    await loadChangelog();
  });
</script>

<div class="changelog-page" class:archive={uiMode === 'archive'}>
  {#if uiMode === 'archive'}
    <main class="archive-main" role="main">
      <div class="archive-content">
        <header class="archive-header">
          <h1 class="archive-page-title">{t('roadmap.changelogTitle')}</h1>
          <p class="archive-summary">{t('roadmap.changelogSub')}</p>
        </header>
        {@render content()}
      </div>
    </main>
  {:else}
    <div class="modern-changelog">
      <header class="changelog-header">
        <h1>{t('roadmap.changelogTitle')}</h1>
        <p class="changelog-sub">{t('roadmap.changelogSub')}</p>
      </header>
      {@render content()}
    </div>
  {/if}
</div>

{#snippet content()}
  <div class="changelog-toolbar">
    <div class="toolbar-kinds">
      {#each KINDS as kind}
        <button
          class="kind-chip {kind.id === activeKind ? 'active' : ''}"
          onclick={() => { activeKind = kind.id; loadChangelog(); }}
        >
          {kindLabel(kind.id)}
        </button>
      {/each}
    </div>
  </div>

  {#if loading}
    <div class="changelog-loading">{t('roadmap.loading')}</div>
  {:else if error}
    <div class="error-card"><strong>⚠️ {error}</strong></div>
  {:else if entries.length === 0}
    <div class="changelog-empty">{t('roadmap.changelogEmpty')}</div>
  {:else}
    <article class="changelog-list">
      {#each entries as entry (entry.id)}
        <section class="changelog-entry">
          <header class="entry-header">
            <span class="entry-kind">{kindLabel(entry.kind)}</span>
            <time class="entry-date" datetime={entry.published_at}>{formatDate(entry.published_at)}</time>
          </header>
          <h2 class="entry-title">{entry.title}</h2>
          <div class="entry-body">{entry.body}</div>
          {#if entry.feature_id}
            <footer class="entry-footer">
              <a href="/roadmap/board" class="feature-link">
                {t('roadmap.relatedFeature')} #{entry.feature_id}
              </a>
            </footer>
          {/if}
        </section>
      {/each}
    </article>
  {/if}
{/snippet}

<style>
  /* ── Archive mode ─────────────────────────────────────────────── */
  .archive-main {
    max-width: var(--archive-max-width, 1100px);
    margin: 0 auto;
    padding: 0;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-content {
    max-width: 800px;
    margin: 0 auto;
    padding: 1rem;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-header {
    margin-bottom: 1.25rem;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    padding-bottom: 0.6em;
  }
  .archive-page-title {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.8em;
    font-weight: 700;
    color: var(--archive-heading, #990000);
    margin: 0 0 0.15em;
  }
  .archive-summary {
    color: var(--archive-muted, #666666);
    font-size: 0.95em;
    margin: 0.2em 0 0;
    max-width: 80ch;
  }

  /* ── Modern mode ──────────────────────────────────────────────── */
  .modern-changelog {
    max-width: 800px;
    margin: 0 auto;
    padding: 1.5rem;
  }
  .changelog-header {
    margin-bottom: 1.5rem;
  }
  .changelog-header h1 {
    font-size: 1.75rem;
    font-weight: 700;
    margin: 0 0 0.25rem;
  }
  .changelog-sub {
    color: var(--text-muted, #6b7280);
    margin: 0;
  }

  /* ── Toolbar ──────────────────────────────────────────────────── */
  .changelog-toolbar {
    margin-bottom: 1.5rem;
  }
  .toolbar-kinds {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  .kind-chip {
    padding: 0.375rem 0.875rem;
    font-size: 0.85rem;
    font-weight: 500;
    border: 1px solid var(--border, #e5e7eb);
    border-radius: 9999px;
    background: var(--bg, #ffffff);
    color: var(--text, #1f2937);
    cursor: pointer;
    transition: all 0.15s;
  }
  .kind-chip:hover { background: var(--bg-raised, #f9fafb); }
  .kind-chip.active {
    background: var(--primary, #2563eb);
    border-color: var(--primary, #2563eb);
    color: white;
  }

  /* ── Changelog list ───────────────────────────────────────────── */
  .changelog-list {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  .changelog-entry {
    background: var(--bg, #ffffff);
    border: 1px solid var(--border, #e5e7eb);
    border-radius: 0.5rem;
    padding: 1.25rem;
  }
  .entry-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.5rem;
  }
  .entry-kind {
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
    background: var(--bg-raised, #f9fafb);
    border: 1px solid var(--border, #e5e7eb);
  }
  .entry-date {
    font-size: 0.8rem;
    color: var(--text-muted, #6b7280);
  }
  .entry-title {
    font-size: 1.125rem;
    font-weight: 600;
    margin: 0 0 0.5rem;
  }
  .entry-body {
    color: var(--text, #1f2937);
    line-height: 1.6;
  }
  .entry-footer {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border, #e5e7eb);
  }
  .feature-link {
    font-size: 0.85rem;
    color: var(--primary, #2563eb);
    text-decoration: none;
  }
  .feature-link:hover { text-decoration: underline; }

  .changelog-loading,
  .changelog-empty {
    padding: 2rem;
    text-align: center;
    color: var(--text-muted, #6b7280);
  }
  .error-card {
    padding: 1rem;
    background: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 0.5rem;
    color: #991b1b;
  }
</style>