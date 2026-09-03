<script lang="ts">
  // CommandPalette — Ctrl+K / Cmd+K quick-jump.
  // Doc entries come from docs-map.json (fetched once, cached); opening one
  // shows the in-app help modal; page actions navigate directly.
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { openDoc, closeDoc } from '$lib/stores/doc-help.svelte';

  interface DocEntry {
    slug: string;
    page: string;
    anchor: string | null;
    title: string;
    page_title: string;
    summary: string;
    keywords: string[];
    feature: string;
    level: number;
  }

  const PAGE_ACTIONS = [
    { label: 'Home', hint: '/', goto: '/' },
    { label: 'Search', hint: '/search', goto: '/search' },
    { label: 'Requests', hint: '/requests', goto: '/requests' },
    { label: 'Roadmap', hint: '/roadmap', goto: '/roadmap' },
    { label: 'Bookmarks', hint: '/bookmarks', goto: '/bookmarks' },
    { label: 'Moderation Log', hint: '/modlog', goto: '/modlog' },
  ];

  let open = $state(false);
  let query = $state('');
  let docs: DocEntry[] | null = $state(null);
  let docsError = $state('');

  onMount(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        open = !open;
        query = '';
      } else if (e.key === 'Escape' && open) {
        open = false;
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  });

  async function loadDocs() {
    if (docs) return;
    try {
      const res = await fetch('/docs/docs-map.json');
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      docs = await res.json();
    } catch (e) {
      docsError = e instanceof Error ? e.message : String(e);
    }
  }

  $effect(() => {
    if (open) void loadDocs();
  });

  let sel = $state(0);
  let inputEl: HTMLInputElement | undefined = $state();

  $effect(() => {
    if (open) {
      requestAnimationFrame(() => inputEl?.focus());
    }
  });

  function matches(e: DocEntry): boolean {
    if (!query.trim()) return true;
    const q = query.toLowerCase();
    return (
      e.title.toLowerCase().includes(q) ||
      e.page_title.toLowerCase().includes(q) ||
      e.summary.toLowerCase().includes(q) ||
      e.keywords.some((k) => k.toLowerCase().includes(q)) ||
      e.slug.toLowerCase().includes(q)
    );
  }

  function results(): DocEntry[] {
    if (!docs) return [];
    return docs.filter(matches).slice(0, 12);
  }

  function pick(entry: DocEntry) {
    open = false;
    openDoc(entry.slug);
  }

  function pickAction(a: { goto: string }) {
    open = false;
    void goto(a.goto);
  }
</script>

{#if open}
  <div class="palette-backdrop" role="presentation" onclick={(e) => { if (e.target === e.currentTarget) open = false; }}>
    <div class="palette" role="dialog" aria-modal="true" aria-label="Command palette" tabindex="-1">
      <div class="palette-input-row">
        <span class="kbd-hint">⌘K</span>
        <input
          bind:this={inputEl}
          type="search"
          placeholder="Jump to a page or docs section…"
          bind:value={query}
          onkeydown={(e) => {
            if (e.key === 'Enter') {
              const r = results();
              if (r.length) pick(r[0]);
            }
            if (e.key === 'ArrowDown') { e.preventDefault(); sel = Math.min(sel + 1, results().length - 1); }
            if (e.key === 'ArrowUp') { e.preventDefault(); sel = Math.max(sel - 1, 0); }
          }}
        />
      </div>
      <div class="palette-body">
        {#if docsError}<p class="muted">Docs unavailable ({docsError}) — page actions still work.</p>{/if}
        <div class="group">
          <p class="group-label">Pages</p>
          {#each PAGE_ACTIONS as a (a.label)}
            <button class="row" onclick={() => pickAction(a)}>
              <span class="row-title">→ {a.label}</span>
              <span class="row-hint">{a.hint}</span>
            </button>
          {/each}
        </div>
        <div class="group">
          <p class="group-label">Docs</p>
          {#if !docs && !docsError}<p class="muted"><span class="spinner"></span> Loading docs…</p>{/if}
          {#each results() as r (r.slug)}
            <button class="row" class:sel={sel === results().indexOf(r)} onclick={() => pick(r)}>
              <span class="row-title">📖 {r.level === 0 ? r.title : `${r.page_title} · ${r.title}`}</span>
              <span class="row-hint">{r.summary.slice(0, 60)}</span>
            </button>
          {/each}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .palette-backdrop {
    position: fixed; inset: 0; background: rgba(0,0,0,0.4);
    display: flex; align-items: flex-start; justify-content: center;
    padding-top: 12vh; z-index: 1200;
  }
  .palette {
    background: var(--color-bg, #fff); border-radius: 12px;
    width: min(560px, 92vw); max-height: 60vh; display: flex; flex-direction: column;
    box-shadow: 0 16px 48px rgba(0,0,0,0.3); overflow: hidden;
  }
  .palette-input-row { display: flex; align-items: center; gap: 0.6rem; padding: 0.8rem 1rem; border-bottom: 1px solid var(--color-border, #eee); }
  .kbd-hint { font-size: 0.7rem; color: var(--color-text-muted, #888); border: 1px solid var(--color-border, #ddd); border-radius: 4px; padding: 0.1rem 0.3rem; }
  .palette-input-row input { flex: 1; border: none; outline: none; font-size: 1rem; background: transparent; color: inherit; }
  .palette-body { overflow-y: auto; padding: 0.5rem; }
  .group-label { font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.06em; color: var(--color-text-muted, #888); margin: 0.6rem 0.4rem 0.2rem; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: 0.75rem; width: 100%; padding: 0.45rem 0.6rem; border: none; background: transparent; border-radius: 8px; cursor: pointer; text-align: left; color: inherit; }
  .row:hover, .row.sel { background: var(--color-accent-bg, #eef1ff); }
  .row-title { font-weight: 600; font-size: 0.92rem; }
  .row-hint { font-size: 0.75rem; color: var(--color-text-muted, #888); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 50%; }
  .muted { color: var(--color-text-muted, #888); font-size: 0.85rem; padding: 0.4rem 0.6rem; }
</style>
