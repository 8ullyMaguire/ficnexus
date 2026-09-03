<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';
  import { adminFetch } from '$lib/api/admin';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  interface MarginaliaItem {
    id: number;
    work_title: string;
    url_id: string | null;
    chapter_index: number;
    chapter: number;
    excerpt: string;
    has_passage_text: boolean;
    topic_id: number;
    topic_slug: string | null;
    reply_count: number;
    created_at: string;
    read_url: string | null;
    topic_url: string;
  }

  let items = $state<MarginaliaItem[]>([]);
  let total = $state(0);
  let page = $state(0);
  const limit = 50;
  let loading = $state(true);
  let error = $state('');

  async function load(p = 0) {
    loading = true;
    error = '';
    try {
      const res = await adminFetch(`/api/curator/marginalia?page=${p}&limit=${limit}`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      items = body.items ?? [];
      total = body.total ?? 0;
      page = body.page ?? p;
    } catch (e) {
      error = `Failed to load marginalia: ${e instanceof Error ? e.message : e}`;
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn) {
      goto('/login');
      return;
    }
    load(0);
  });

  function fmtTime(iso: string): string {
    return new Date(iso).toLocaleString();
  }
</script>

{#if uiMode === 'archive'}
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Marginalia</h1>
        <p class="archive-summary">
          Per-passage forum discussions started from the reader. {total} total.
        </p>
      </header>
      {#if loading}<p>Loading…</p>{/if}
      {#if error}<p class="error">{error}</p>{/if}
      {#if !loading}
        <table class="mtable">
          <thead>
            <tr><th>Work</th><th>Ch</th><th>Passage</th><th>Replies</th><th>Created</th><th></th></tr>
          </thead>
          <tbody>
            {#each items as m (m.id)}
              <tr>
                <td>{m.work_title}</td>
                <td>{m.chapter}</td>
                <td><span class="excerpt" class:hash={!m.has_passage_text}>{m.excerpt}</span></td>
                <td>{m.reply_count}</td>
                <td>{fmtTime(m.created_at)}</td>
                <td>
                  {#if m.topic_url}<a href={m.topic_url}>forum</a>{/if}
                  {#if m.read_url}<a href={m.read_url}>read</a>{/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
        <nav class="pager">
          <button disabled={page === 0} onclick={() => load(page - 1)}>‹ Prev</button>
          <button disabled={(page + 1) * limit >= total} onclick={() => load(page + 1)}>Next ›</button>
        </nav>
      {/if}
    </div>
  </main>
{:else}
  <main class="modern">
    <h1>Marginalia</h1>
    <p class="summary">Per-passage forum discussions started from the reader. {total} total.</p>
    {#if loading}<p>Loading…</p>{/if}
    {#if error}<p class="error">{error}</p>{/if}
    {#if !loading}
      <table class="mtable">
        <thead>
          <tr><th>Work</th><th>Ch</th><th>Passage</th><th>Replies</th><th>Created</th><th></th></tr>
        </thead>
        <tbody>
          {#each items as m (m.id)}
            <tr>
              <td>{m.work_title}</td>
              <td>{m.chapter}</td>
              <td><span class="excerpt" class:hash={!m.has_passage_text}>{m.excerpt}</span></td>
              <td>{m.reply_count}</td>
              <td>{fmtTime(m.created_at)}</td>
              <td>
                {#if m.topic_url}<a href={m.topic_url}>forum</a>{/if}
                {#if m.read_url}<a href={m.read_url}>read</a>{/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
      <nav class="pager">
        <button disabled={page === 0} onclick={() => load(page - 1)}>‹ Prev</button>
        <button disabled={(page + 1) * limit >= total} onclick={() => load(page + 1)}>Next ›</button>
      </nav>
    {/if}
  </main>
{/if}

<style>
  .error { color: #c0392b; }
  .excerpt { font-style: italic; }
  .excerpt.hash { font-family: monospace; font-style: normal; font-size: 0.85em; opacity: 0.75; }
  .mtable { width: 100%; border-collapse: collapse; margin-top: 0.75rem; }
  .mtable th, .mtable td {
    text-align: left; padding: 0.4rem 0.6rem; border-bottom: 1px solid var(--color-border, #ddd);
    vertical-align: top;
  }
  .pager { display: flex; gap: 0.5rem; justify-content: center; margin: 1rem 0; }
  .pager button { padding: 0.35rem 0.8rem; cursor: pointer; font: inherit; }
  .pager button:disabled { opacity: 0.5; cursor: default; }
</style>
