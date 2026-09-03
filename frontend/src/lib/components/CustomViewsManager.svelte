<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { fetchViews, createView, deleteView, togglePin, type SavedView } from '$lib/api/views';

  let views = $state<SavedView[]>([]);
  let loading = $state(true);
  let error = $state('');
  let showCreate = $state(false);
  let newName = $state('');
  let newQueryJson = $state('{}');
  let saving = $state(false);

  onMount(async () => {
    if (!auth.isLoggedIn || auth.level < 7) return; // ui.views feature gate: rank 7+
    try {
      views = await fetchViews();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  async function handleCreate() {
    if (!newName.trim()) return;
    saving = true;
    error = '';
    try {
      let query: Record<string, unknown> = {};
      try {
        query = JSON.parse(newQueryJson);
      } catch {
        error = 'Invalid JSON in query field';
        saving = false;
        return;
      }
      const v = await createView(newName.trim(), query);
      views = [...views, v];
      newName = '';
      newQueryJson = '{}';
      showCreate = false;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  async function handleDelete(id: number) {
    try {
      await deleteView(id);
      views = views.filter((v) => v.id !== id);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleTogglePin(id: number) {
    try {
      const updated = await togglePin(id);
      views = views.map((v) => (v.id === id ? updated : v));
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function queryPreview(query: Record<string, unknown>): string {
    const keys = Object.keys(query);
    if (keys.length === 0) return '(empty)';
    return keys
      .slice(0, 3)
      .map((k) => `${k}: ${String(query[k]).slice(0, 30)}`)
      .join(', ') + (keys.length > 3 ? '…' : '');
  }

  function buildViewUrl(view: SavedView): string {
    return `/search?q=${encodeURIComponent(JSON.stringify(view.query))}`;
  }
</script>

{#if !auth.isLoggedIn || auth.level < 7}
  <p class="muted">Custom views require rank 7+ (ui.views feature).</p>
{:else}
  <div class="views-manager">
    <div class="views-header">
      <h3>📌 Saved Views</h3>
      <button class="btn btn-secondary btn-sm" onclick={() => (showCreate = !showCreate)}>
        {showCreate ? '✕ Cancel' : '+ New View'}
      </button>
    </div>

    {#if error}
      <div class="error-card">⚠️ {error}</div>
    {/if}

    {#if showCreate}
      <div class="card create-form">
        <label for="view-name">View Name</label>
        <input id="view-name" type="text" bind:value={newName} placeholder="e.g. My favourite HP fics" />
        <label for="view-query">Query JSON</label>
        <textarea id="view-query" bind:value={newQueryJson} rows="4" placeholder={'{"include_tags":"2:Harry Potter"}'}></textarea>
        <div class="actions">
          <button class="btn btn-primary btn-sm" onclick={handleCreate} disabled={saving || !newName.trim()}>
            {saving ? 'Saving…' : '💾 Save View'}
          </button>
        </div>
      </div>
    {/if}

    {#if loading}
      <p class="muted">Loading views…</p>
    {:else if views.length === 0}
      <p class="muted">No saved views yet. Use the search page and save a view here.</p>
    {:else}
      <ul class="views-list">
        {#each views as view (view.id)}
          <li class="view-row" class:pinned={view.pinned}>
            <a class="view-link" href={buildViewUrl(view)}>
              <span class="view-name">{view.pinned ? '📌 ' : ''}{view.name}</span>
              <span class="view-preview">{queryPreview(view.query)}</span>
            </a>
            <div class="view-actions">
              <button
                class="icon-btn"
                title={view.pinned ? 'Unpin' : 'Pin to top'}
                onclick={() => handleTogglePin(view.id)}
              >
                {view.pinned ? '📌' : '📍'}
              </button>
              <button class="icon-btn danger" title="Delete view" onclick={() => handleDelete(view.id)}>
                🗑️
              </button>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
{/if}

<style>
  .views-manager { margin: 1rem 0; }
  .views-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.75rem; }
  .views-header h3 { margin: 0; }
  .views-list { list-style: none; padding: 0; margin: 0.5rem 0; }
  .view-row {
    display: flex; align-items: center; justify-content: space-between;
    padding: 0.6rem 0.8rem; border: 1px solid var(--color-border, #ddd);
    border-radius: var(--radius-sm, 6px); margin-bottom: 0.4rem;
    transition: background 0.15s;
  }
  .view-row:hover { background: var(--color-surface-2, #f8f8f8); }
  .view-row.pinned { border-left: 3px solid var(--color-accent, #6366f1); }
  .view-link {
    text-decoration: none; color: inherit; flex: 1;
    display: flex; flex-direction: column; gap: 0.15rem;
  }
  .view-name { font-weight: 600; }
  .view-preview { font-size: 0.8rem; color: var(--color-text-muted, #888); }
  .view-actions { display: flex; gap: 0.3rem; margin-left: 0.5rem; }
  .icon-btn {
    background: none; border: none; cursor: pointer; font-size: 1rem;
    padding: 0.3rem; border-radius: 4px;
  }
  .icon-btn:hover { background: var(--color-surface-2, #eee); }
  .icon-btn.danger:hover { background: #fee2e2; }
  .create-form { display: flex; flex-direction: column; gap: 0.4rem; padding: 1rem; margin-bottom: 0.75rem; }
  .create-form label { font-weight: 700; font-size: 0.85rem; }
  .create-form input, .create-form textarea {
    padding: 0.5rem; border: 1px solid var(--color-border, #ddd); border-radius: 6px;
    width: 100%; box-sizing: border-box;
  }
  .actions { margin-top: 0.5rem; }
  .muted { color: var(--color-text-muted, #888); }
  .error-card { background: #fee2e2; color: #991b1b; padding: 0.5rem; border-radius: 6px; margin-bottom: 0.75rem; font-size: 0.9rem; }
  .card { background: var(--color-surface, #fff); border: 1px solid var(--color-border, #ddd); border-radius: 8px; }
  .btn { padding: 0.4rem 0.8rem; border-radius: 6px; border: 1px solid var(--color-border, #ccc); cursor: pointer; background: var(--color-surface, #fff); }
  .btn-primary { background: var(--color-accent, #6366f1); color: #fff; border-color: var(--color-accent, #6366f1); }
  .btn-secondary { background: var(--color-surface-2, #f3f4f6); }
  .btn-sm { font-size: 0.85rem; padding: 0.3rem 0.6rem; }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
