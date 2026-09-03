<script lang="ts">
  import { adminFetch } from '$lib/api/admin';
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let items = $state<any[]>([]);
  let loading = $state(true);
  let error = $state('');

  async function loadQueue() {
    loading = true; error = '';
    try {
      const res = await adminFetch('/api/admin/moderation/queue');
      if (res.ok) {
        const data = await res.json();
        items = Array.isArray(data.items) ? data.items : [];
      } else { error = 'Failed to load'; }
    } catch { error = 'Network error'; }
    finally { loading = false; }
  }

  async function approve(id: number) {
    const res = await adminFetch(`/api/admin/moderation/approve/${id}`, { method: 'POST', credentials: 'include' });
    if (res.ok) items = items.filter((i: any) => i.work_id !== id);
  }

  async function reject(id: number) {
    const res = await adminFetch(`/api/admin/moderation/reject/${id}`, { method: 'POST', credentials: 'include' });
    if (res.ok) items = items.filter((i: any) => i.work_id !== id);
  }

  onMount(loadQueue);
</script>

<svelte:head><title>Moderation | FicNexus Admin</title></svelte:head>

{#if uiMode === 'archive'}
  <!-- ── Archive mode ─────────────────────────────────────────── -->
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Moderation Queue</h1>
        <p class="archive-summary">Pending manual uploads waiting for approve / reject.</p>
      </header>

      <div class="archive-form-actions">
        <button class="archive-btn" type="button" onclick={loadQueue}>🔄 Refresh</button>
      </div>

      {#if loading}
        <p class="archive-note">Loading...</p>
      {:else if error}
        <p class="archive-error" role="alert">{error}</p>
      {:else if items.length === 0}
        <p class="archive-note">No pending uploads.</p>
      {:else}
        {#each items as item (item.work_id)}
          <dl class="archive-dl mod-item">
            <div class="dl-row"><dt>Title</dt><dd><strong>{item.title}</strong></dd></div>
            <div class="dl-row"><dt>Author</dt><dd>by {item.author}</dd></div>
            <div class="dl-row"><dt>Upload</dt><dd>Uploaded by <strong>{item.uploader}</strong> · {item.created} · {item.source_type}</dd></div>
            <div class="dl-row"><dt>Description</dt><dd>{item.description?.slice(0, 200)}...</dd></div>
            <div class="dl-row">
              <dt>Action</dt>
              <dd>
                <button class="archive-btn archive-btn-primary" type="button" onclick={() => approve(item.work_id)}>✅ Approve</button>
                {' '}
                <button class="archive-btn archive-btn-danger" type="button" onclick={() => reject(item.work_id)}>❌ Reject</button>
              </dd>
            </div>
          </dl>
        {/each}
      {/if}
    </div>
  </main>
{:else}
<h1>Moderation Queue</h1>
<button onclick={loadQueue}>🔄 Refresh</button>

{#if loading}
  <p>Loading...</p>
{:else if error}
  <p class="error">{error}</p>
{:else if items.length === 0}
  <p>No pending uploads.</p>
{:else}
  {#each items as item}
    <div class="mod-card">
      <div class="mod-info">
        <h3>{item.title}</h3>
        <p class="byline">by {item.author}</p>
        <p class="meta">Uploaded by <strong>{item.uploader}</strong> · {item.created} · {item.source_type}</p>
        <p class="desc">{item.description?.slice(0, 200)}...</p>
      </div>
      <div class="mod-actions">
        <button class="approve" onclick={() => approve(item.work_id)}>✅ Approve</button>
        <button class="reject" onclick={() => reject(item.work_id)}>❌ Reject</button>
      </div>
    </div>
  {/each}
{/if}
{/if}

<style>
  /* ── Archive mode ─────────────────────────────────────────── */
  .archive-main {
    max-width: var(--archive-max-width, 1100px);
    margin: 0 auto;
    padding: 0;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-content { max-width: 820px; margin: 0 auto; padding: 1rem; }
  .archive-header {
    padding: 0.8rem 1rem 0.6rem;
    border-bottom: 2px solid var(--archive-link, #990000);
    background: var(--archive-bg, #ffffff);
  }
  .archive-page-title { font-size: 1.6em; font-weight: 700; margin: 0; }
  .archive-summary { color: var(--archive-muted, #666666); font-size: 0.95em; margin: 0.4em 0 0; }
  .archive-form-actions { margin: 1rem 0; }
  .archive-dl.mod-item { border-bottom: 1px solid var(--archive-border, #dddddd); margin: 0; padding: 0.5em 0; }
  .archive-dl.mod-item:last-child { border-bottom: none; }
  .archive-dl .dl-row { display: flex; }
  .archive-dl .dl-row dt { font-weight: 700; font-size: 0.88em; flex: 0 0 7em; margin: 0.35em 0; }
  .archive-dl .dl-row dd { margin: 0.35em 0; min-width: 0; flex: 1; }
  .archive-btn {
    display: inline-block;
    padding: 0.3em 1em;
    font-size: 0.85em;
    font-weight: 700;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-btn-bg, #f5f5f5);
    color: var(--archive-text, #2a2a2a);
    cursor: pointer;
    font-family: inherit;
  }
  .archive-btn:hover:not(:disabled) { background: #eeeeee; }
  .archive-btn-primary {
    background: var(--archive-link, #990000);
    color: #ffffff;
    border-color: var(--archive-link, #990000);
  }
  .archive-btn-primary:hover:not(:disabled) { background: #7a0000; }
  .archive-btn-danger { color: var(--archive-link, #990000); border-color: var(--archive-link, #990000); }
  .archive-error { color: #990000; font-weight: 700; font-size: 0.92em; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.92em; }

  /* ── Modern mode ──────────────────────────────────────────── */
  .mod-card { display: flex; justify-content: space-between; align-items: flex-start; padding: 1rem; margin: 0.5rem 0; background: var(--color-surface); border-radius: var(--radius-md); border: 1px solid var(--color-border); }
  .mod-info h3 { margin: 0 0 0.2rem; font-size: 1rem; }
  .byline { font-size: 0.88rem; color: var(--color-muted); margin: 0; }
  .meta { font-size: 0.82rem; color: var(--color-muted); margin: 0.3rem 0; }
  .mod-actions { display: flex; gap: 0.5rem; flex-shrink: 0; }
  .mod-actions button { padding: 0.4rem 1rem; border-radius: var(--radius-sm); border: none; cursor: pointer; font-size: 0.88rem; }
  .approve { background: #22c55e; color: white; }
  .reject { background: #ef4444; color: white; }
  .error { color: var(--color-error); }
</style>
