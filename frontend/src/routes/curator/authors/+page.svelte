<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';
  import { getPendingMerges, approveMerge, rejectMerge, type MergeProposal } from '$lib/api/authors';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let proposals = $state<MergeProposal[]>([]);
  let loading = $state(true);
  let error = $state('');
  let actionMsg = $state('');
  let checking = $state(true);

  const isAdmin = $derived(auth.level >= 100);

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn || auth.level < 100) {
      goto('/');
      return;
    }
    checking = false;
    await load();
  });

  async function load() {
    loading = true;
    error = '';
    try {
      proposals = await getPendingMerges();
      if (proposals.length === 0) {
        error = 'No pending merge proposals. 🎉';
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Could not load merge proposals.';
    } finally {
      loading = false;
    }
  }

  async function handleApprove(id: number) {
    actionMsg = '';
    try {
      await approveMerge(id);
      proposals = proposals.filter((p) => p.id !== id);
      actionMsg = '✅ Proposal approved';
    } catch (e) {
      actionMsg = `⚠️ ${e instanceof Error ? e.message : 'Approve failed'}`;
    }
  }

  async function handleReject(id: number) {
    actionMsg = '';
    try {
      await rejectMerge(id);
      proposals = proposals.filter((p) => p.id !== id);
      actionMsg = '✅ Proposal rejected';
    } catch (e) {
      actionMsg = `⚠️ ${e instanceof Error ? e.message : 'Reject failed'}`;
    }
  }
</script>

{#if uiMode === 'archive'}
  {#if checking}
    <div class="loading"><p>Checking access...</p></div>
  {:else if isAdmin}
    <main class="archive-main">
      <div class="archive-content">
        <header class="archive-header">
          <h1 class="archive-page-title">Author Merge Queue</h1>
          <p class="archive-summary">Approve a proposal to link the source account to the target author profile; reject it to discard the proposal.</p>
        </header>
        {#if actionMsg}<p class="archive-msg" role="status">{actionMsg}</p>{/if}
        {#if loading}
          <p class="archive-note">Loading proposals…</p>
        {:else if error}
          <p class="archive-error" role="alert">{error}</p>
        {:else}
          {#each proposals as p (p.id)}
            <fieldset class="archive-fieldset">
              <legend class="archive-legend">{p.source_author} → {p.target_name}</legend>
              <dl class="archive-dl">
                <div class="dl-row"><dt>Source</dt><dd><a class="archive-link" href={p.source_url} target="_blank" rel="noopener noreferrer">{p.source_url}</a></dd></div>
                <div class="dl-row"><dt>Merge into</dt><dd><a class="archive-link" href={`/authors/${p.target_profile_id}`}>{p.target_name}</a>{#if p.proposed_by} · proposed by {p.proposed_by}{/if} · {p.created_at}</dd></div>
              </dl>
              <div class="archive-form-actions">
                <button class="archive-btn archive-btn-primary" type="button" onclick={() => handleApprove(p.id)}>✅ Approve</button>
                {' '}
                <button class="archive-btn" type="button" onclick={() => handleReject(p.id)}>❌ Reject</button>
              </div>
            </fieldset>
          {/each}
        {/if}
        <p class="archive-note"><a class="archive-link" href="/authors">← Back to authors</a></p>
      </div>
    </main>
  {/if}
{:else}
{#if checking}
  <div class="loading"><p>Checking access...</p></div>
{:else if isAdmin}
  <div class="merge-queue">
    <h1>🧬 Author Merge Queue</h1>
    <p class="muted">
      Approve a proposal to link the source account to the target author profile;
      reject it to discard the proposal.
    </p>

    {#if actionMsg}<p class="form-msg">{actionMsg}</p>{/if}

    {#if loading}
      <div class="card"><p class="muted"><span class="spinner"></span> Loading proposals…</p></div>
    {:else if error}
      <div class="card error-card"><strong class="error-text">⚠️ {error}</strong></div>
    {:else}
      <div class="proposal-list">
        {#each proposals as p (p.id)}
          <div class="card proposal">
            <div class="proposal-main">
              <strong>{p.source_author}</strong>
              <a href={p.source_url} target="_blank" rel="noopener noreferrer" class="muted">{p.source_url}</a>
              <p class="muted">
                → merge into <a href={`/authors/${p.target_profile_id}`}>{p.target_name}</a>
                {#if p.proposed_by}
                  · proposed by {p.proposed_by}
                {/if}
                · {p.created_at}
              </p>
            </div>
            <div class="proposal-actions">
              <button class="btn btn-secondary" onclick={() => handleApprove(p.id)}>✅ Approve</button>
              <button class="btn btn-secondary" onclick={() => handleReject(p.id)}>❌ Reject</button>
            </div>
          </div>
        {/each}
      </div>
    {/if}

    <p class="muted back-link"><a href="/authors">← Back to authors</a></p>
  </div>
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
  .archive-fieldset { border: 1px solid var(--archive-border, #dddddd); padding: 1em 1.25em; margin: 1.2rem 0; }
  .archive-legend { font-weight: 700; font-size: 1.05em; padding: 0 0.4em; }
  .archive-dl { margin: 0; padding: 0; }
  .archive-dl .dl-row { display: flex; }
  .archive-dl .dl-row dt { font-weight: 700; font-size: 0.88em; flex: 0 0 7em; margin: 0.45em 0; }
  .archive-dl .dl-row dd { margin: 0.45em 0; min-width: 0; flex: 1; overflow-wrap: anywhere; }
  .archive-form-actions { margin-top: 0.75em; }
  .archive-btn {
    display: inline-block; padding: 0.3em 1em; font-size: 0.85em; font-weight: 700;
    border: 1px solid var(--archive-border, #dddddd); background: var(--archive-btn-bg, #f5f5f5);
    color: var(--archive-text, #2a2a2a); cursor: pointer; font-family: inherit;
  }
  .archive-btn:hover:not(:disabled) { background: #eeeeee; }
  .archive-btn-primary { background: var(--archive-link, #990000); color: #ffffff; border-color: var(--archive-link, #990000); }
  .archive-btn-primary:hover:not(:disabled) { background: #7a0000; }
  .archive-link { color: var(--archive-link, #990000); }
  .archive-msg { font-size: 0.92em; }
  .archive-error { color: #990000; font-weight: 700; font-size: 0.92em; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.88em; }
  p.archive-note { margin: 1.2em 0 0; }

  /* ── Modern mode ──────────────────────────────────────────── */
  .merge-queue {
    max-width: var(--max-width);
    margin: 0 auto;
    padding: 1rem;
  }
  .loading {
    text-align: center;
    padding: 3rem;
  }
  .proposal-list {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    margin-top: 1rem;
  }
  .proposal {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
  }
  .proposal-main {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    min-width: 0;
  }
  .proposal-actions {
    display: flex;
    gap: 0.5rem;
  }
  .form-msg {
    font-size: 0.9rem;
  }
  .back-link {
    margin-top: 1.5rem;
  }
</style>
