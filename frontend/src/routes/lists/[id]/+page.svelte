<script lang="ts">
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';
  import { getReadingListDetail, addReadingListItem, removeReadingListItem } from '$lib/api/social';
  import type { ReadingListDetail } from '$lib/api/social-types';
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';

  const uiMode = $derived(getPref('uiMode'));

  let { data } = $props();
  const listId = $derived(Number(data.id));

  let detail = $state<ReadingListDetail | null>(null);
  let loading = $state(true);
  let error = $state('');

  // Add-item form (owner only): work id + optional blurb.
  let showAddForm = $state(false);
  let newWorkId = $state('');
  let newBlurb = $state('');
  let adding = $state(false);
  let addError = $state('');

  onMount(async () => {
    await auth.init();
    await loadDetail();
  });

  async function loadDetail() {
    loading = true; error = '';
    try {
      const res = await getReadingListDetail(listId);
      if (res.err === 0) detail = res;
      else error = 'This list is private or does not exist.';
    } catch { error = 'Network error.'; }
    finally { loading = false; }
  }

  async function handleAdd() {
    const workId = Number(newWorkId.trim());
    if (!workId || !Number.isInteger(workId) || workId <= 0) { addError = 'Enter a valid work id.'; return; }
    adding = true; addError = '';
    try {
      const res = await addReadingListItem(listId, workId, newBlurb.trim() || undefined);
      if (res.err === 0) {
        newWorkId = ''; newBlurb = ''; showAddForm = false;
        await loadDetail();
      } else {
        addError = res.msg || 'Failed to add work.';
      }
    } catch { addError = 'Network error.'; }
    finally { adding = false; }
  }

  async function handleRemove(workId: number) {
    try {
      const res = await removeReadingListItem(listId, workId);
      if (res.err === 0) await loadDetail();
    } catch { error = 'Failed to remove work.'; }
  }
</script>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <a class="archive-link" href="/lists">← All lists</a>
      {#if loading}
        <p class="archive-muted"><span class="spinner"></span> Loading...</p>
      {:else if error || !detail}
        <h1 class="archive-page-title">Reading List</h1>
        <p class="archive-error">{error || 'List not found.'}</p>
      {:else}
        <h1 class="archive-page-title">{detail.list.title}</h1>
        {#if detail.list.description}
          <blockquote class="archive-summary">{detail.list.description}</blockquote>
        {/if}
        <p class="archive-muted detail-meta">
          {#if detail.list.is_public}Public{:else}Private{/if}
          · {detail.items.length} work{detail.items.length !== 1 ? 's' : ''}
          · Created {new Date(detail.list.created_at).toLocaleDateString()}
        </p>
      {/if}
    </header>

    {#if !loading && !error && detail}
      {#if detail.is_owner}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Manage</legend>
          <div class="archive-actions">
            <button class="archive-btn archive-btn-primary" onclick={() => { showAddForm = !showAddForm; addError = ''; }}>
              {showAddForm ? 'Cancel' : '+ Add Work'}
            </button>
          </div>

          {#if showAddForm}
            <form class="archive-form" onsubmit={(e) => { e.preventDefault(); handleAdd(); }}>
              <p class="archive-muted form-hint">Enter the numeric work id (from the work's page URL: /work/{'{'}id{'}'}).</p>
              <dl class="archive-dl">
                <div class="dl-row">
                  <dt><label for="add-work-id">Work id</label></dt>
                  <dd><input id="add-work-id" class="archive-input" type="text" inputmode="numeric" placeholder="Work id" bind:value={newWorkId} /></dd>
                </div>
                <div class="dl-row">
                  <dt><label for="add-blurb">Blurb</label></dt>
                  <dd><input id="add-blurb" class="archive-input" type="text" placeholder="One-line blurb (optional)" bind:value={newBlurb} maxlength="500" /></dd>
                </div>
              </dl>
              {#if addError}
                <p class="archive-error">{addError}</p>
              {/if}
              <button class="archive-btn archive-btn-primary" type="submit" disabled={adding}>
                {adding ? 'Adding...' : 'Add to list'}
              </button>
            </form>
          {/if}
        </fieldset>
      {/if}

      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Works in this list</legend>
        {#if detail.items.length === 0}
          <p class="archive-muted">This list is empty.</p>
        {:else}
          <ol class="archive-list">
            {#each detail.items as item (item.id)}
              <li class="archive-list-row">
                <span class="item-pos">{item.position}.</span>
                <span class="archive-list-main">
                  <a class="archive-link archive-list-title" href="/work/{item.work_id}">{item.title}</a>
                  <span class="archive-muted"> by {item.author}</span>
                  {#if item.blurb}
                    <span class="item-blurb">“{item.blurb}”</span>
                  {/if}
                </span>
                {#if detail.is_owner}
                  <button class="archive-btn" onclick={() => handleRemove(item.work_id)} title="Remove from list">Remove</button>
                {/if}
              </li>
            {/each}
          </ol>
        {/if}
      </fieldset>
    {/if}
  </div>
</main>
{:else}
<!-- Modern mode (unchanged) -->
<div class="list-detail-page">
  {#if loading}
    <div class="card"><p class="muted"><span class="spinner"></span> Loading...</p></div>
  {:else if error || !detail}
    <div class="card error-card"><strong class="error-text">⚠️ {error || 'List not found.'}</strong></div>
  {:else}
    <div class="detail-head">
      <a class="back" href="/lists">← All lists</a>
      <h1>{detail.list.title}</h1>
      {#if detail.list.description}
        <p class="muted detail-desc">{detail.list.description}</p>
      {/if}
      <p class="muted detail-meta">
        {#if detail.list.is_public}🌍 Public{:else}🔒 Private{/if}
        · {detail.items.length} work{detail.items.length !== 1 ? 's' : ''}
        · Created {new Date(detail.list.created_at).toLocaleDateString()}
      </p>
    </div>

    {#if detail.is_owner}
      <div class="actions">
        <button class="btn btn-primary" onclick={() => { showAddForm = !showAddForm; addError = ''; }}>
          {showAddForm ? 'Cancel' : '+ Add Work'}
        </button>
      </div>

      {#if showAddForm}
        <div class="card add-form">
          <h3>Add a Work</h3>
          <p class="muted form-hint">Enter the numeric work id (from the work's page URL: /work/{'{'}id{'}'}).</p>
          <input
            type="text"
            inputmode="numeric"
            placeholder="Work id"
            bind:value={newWorkId}
            class="input"
          />
          <input
            type="text"
            placeholder="One-line blurb (optional)"
            bind:value={newBlurb}
            maxlength="500"
            class="input"
          />
          {#if addError}
            <p class="error-text">{addError}</p>
          {/if}
          <button class="btn btn-primary" onclick={handleAdd} disabled={adding}>
            {adding ? 'Adding...' : 'Add to list'}
          </button>
        </div>
      {/if}
    {/if}

    {#if detail.items.length === 0}
      <div class="card empty">
        <p class="muted">This list is empty.</p>
      </div>
    {:else}
      <div class="item-list">
        {#each detail.items as item (item.id)}
          <div class="card item-row">
            <div class="item-pos">{item.position}</div>
            <div class="item-info">
              <h3><a href="/work/{item.work_id}">{item.title}</a></h3>
              <p class="muted">by {item.author}</p>
              {#if item.blurb}
                <p class="item-blurb">“{item.blurb}”</p>
              {/if}
            </div>
            {#if detail.is_owner}
              <button class="btn-danger-ghost" onclick={() => handleRemove(item.work_id)} title="Remove from list">Remove</button>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>
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
  .archive-content {
    max-width: var(--archive-max-width, 900px);
    margin: 0 auto;
    padding: 1rem;
  }
  .archive-header {
    padding: 0 0 0.6rem;
    border-bottom: 2px solid var(--archive-link, #990000);
    margin-bottom: 1rem;
  }
  .archive-page-title {
    font-size: 1.6em;
    font-weight: 700;
    margin: 0.2rem 0 0;
  }
  .detail-meta { font-size: 0.88rem; }
  .archive-muted {
    color: var(--archive-muted, #666666);
    font-style: italic;
  }
  .archive-error {
    color: var(--archive-link, #990000);
    font-weight: 700;
    font-size: 0.9rem;
  }
  .archive-link {
    color: var(--archive-link, #990000);
    text-decoration: none;
    font-weight: 600;
  }
  .archive-link:hover {
    text-decoration: underline;
  }
  .archive-summary {
    margin: 0.5rem 0;
    padding: 0.4rem 1rem;
    border-left: 3px solid var(--archive-border, #dddddd);
    color: var(--archive-muted, #666666);
    font-style: italic;
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .archive-fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.6rem 0.9rem 0.75rem;
    margin: 0 0 1rem;
    background: var(--archive-bg, #ffffff);
  }
  .archive-fieldset legend,
  .archive-legend {
    font-weight: 700;
    font-size: 0.9em;
    color: var(--archive-link, #990000);
    padding: 0 0.4rem;
  }
  .archive-actions { margin: 0.3rem 0 0.6rem; }
  .archive-form { margin: 0.4rem 0; }
  .form-hint { font-size: 0.85rem; }
  .archive-dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.4rem 1rem;
    margin: 0.4rem 0 0.6rem;
  }
  .archive-dl dt {
    font-weight: 600;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-dl dd { margin: 0; }
  .archive-input {
    padding: 0.3rem 0.5rem;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.9rem;
    width: 100%;
    box-sizing: border-box;
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
  }
  .archive-btn {
    padding: 0.28rem 0.7rem;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-text, #2a2a2a);
    font-family: inherit;
    font-size: 0.85rem;
    cursor: pointer;
  }
  .archive-btn:hover { background: var(--archive-border, #dddddd); }
  .archive-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .archive-btn-primary {
    color: var(--archive-bg, #ffffff);
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }
  .archive-btn-primary:hover { background: var(--archive-link-visited, #660066); border-color: var(--archive-link-visited, #660066); }
  .archive-list {
    list-style: none;
    margin: 0;
    padding: 0;
    border-top: 1px solid var(--archive-border, #dddddd);
  }
  .archive-list-row {
    display: flex;
    align-items: baseline;
    gap: 0.6rem;
    padding: 0.55rem 0.3rem;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    font-size: 0.92rem;
  }
  .archive-list-row:nth-child(odd) {
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .item-pos {
    min-width: 1.4rem;
    text-align: right;
    font-weight: 700;
    color: var(--archive-muted, #666666);
  }
  .archive-list-main {
    flex: 1;
    display: block;
  }
  .archive-list-title {
    font-weight: 700;
  }
  .item-blurb {
    display: block;
    font-style: italic;
    color: var(--archive-text, #2a2a2a);
    margin-top: 0.15rem;
    font-size: 0.88rem;
  }

  /* ── Modern mode (scoped — no bleed) ── */
  .list-detail-page { max-width: var(--max-width); margin: 0 auto; padding: 1rem; }
  .list-detail-page .detail-head { margin-bottom: 1rem; }
  .list-detail-page .back { color: var(--color-muted); text-decoration: none; font-size: 0.9rem; }
  .list-detail-page .back:hover { color: var(--color-primary); }
  .list-detail-page .detail-head h1 { margin: 0.3rem 0; }
  .list-detail-page .detail-desc { margin: 0.2rem 0; }
  .list-detail-page .detail-meta { margin: 0.2rem 0; font-size: 0.85rem; }
  .list-detail-page .actions { margin: 0.8rem 0; }
  .list-detail-page .add-form { display: flex; flex-direction: column; gap: 0.6rem; padding: 1rem; margin-bottom: 1rem; }
  .list-detail-page .form-hint { font-size: 0.85rem; }
  .list-detail-page .input { padding: 0.5rem; border: 1px solid var(--color-border); border-radius: var(--radius-sm); background: var(--color-surface); color: var(--color-text); }
  .list-detail-page .empty { text-align: center; padding: 2rem; }
  .list-detail-page .error-card { border-color: var(--color-error); }
  .list-detail-page .item-list { display: flex; flex-direction: column; gap: 0.5rem; }
  .list-detail-page .item-row { display: flex; align-items: flex-start; gap: 0.8rem; padding: 0.7rem 1rem; }
  .list-detail-page .item-pos { font-size: 1.1rem; font-weight: 700; color: var(--color-muted); min-width: 1.6rem; text-align: center; padding-top: 0.2rem; }
  .list-detail-page .item-info { flex: 1; }
  .list-detail-page .item-info h3 { margin: 0 0 0.15rem; font-size: 1rem; }
  .list-detail-page .item-info h3 a { color: var(--color-text); text-decoration: none; }
  .list-detail-page .item-info h3 a:hover { color: var(--color-primary); }
  .list-detail-page .item-blurb { margin: 0.25rem 0 0; font-size: 0.88rem; font-style: italic; color: var(--color-text); }
  .list-detail-page .btn-danger-ghost { padding: 0.3rem 0.6rem; font-size: 0.8rem; border-radius: var(--radius-sm); background: transparent; border: 1px solid var(--color-error); color: var(--color-error); cursor: pointer; margin-top: 0.2rem; }
  .list-detail-page .btn-danger-ghost:hover { background: var(--color-error); color: white; }
</style>
