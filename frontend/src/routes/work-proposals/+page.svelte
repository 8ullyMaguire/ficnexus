<script lang="ts">
  import { onMount } from 'svelte';
  import { listProposals, voteProposal } from '$lib/api/social';
  import type { WorkProposal } from '$lib/api/social-types';
  import { auth } from '$lib/stores/auth.svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let proposals = $state<WorkProposal[]>([]);
  let loading = $state(true);
  let error = $state('');
  let votingId = $state<number | null>(null);
  let voteMsg = $state('');

  // Voting is open to every signed-in account (trusted+ = level ≥ 1) —
  // mirrors the backend gate used by the proposal vote endpoint.
  const canVote = $derived(auth.level >= 1);

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await listProposals();
      if (res.err === 0) {
        proposals = res.proposals ?? [];
      } else {
        error = 'Failed to load proposals';
      }
    } catch {
      error = 'Failed to load proposals';
    } finally {
      loading = false;
    }
  }

  async function handleVote(p: WorkProposal, vote: -1 | 0 | 1) {
    if (!canVote || votingId !== null) return;
    votingId = p.id;
    voteMsg = '';
    try {
      const res = await voteProposal(p.id, vote);
      p.vote_sum = res.vote_sum;
      p.voter_count = res.voter_count;
      voteMsg = res.msg;
    } catch (e) {
      voteMsg = e instanceof Error ? e.message : 'Vote failed';
    } finally {
      votingId = null;
    }
  }

  function describe(p: WorkProposal): string {
    if (p.action_type === 'merge') {
      return `Merge work #${p.source_work_id ?? '?'} into work #${p.target_work_id ?? '?'}`;
    }
    if (p.action_type === 'split') {
      return `Split work #${p.work_id ?? '?'}`;
    }
    return p.action_type;
  }

  onMount(async () => {
    await auth.init();
    await load();
  });
</script>

<svelte:head><title>Work Proposals — FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">Work Proposals</h1>
      <p class="archive-summary">
        Community-curated merge / split proposals for canonical works. Curators vote
        (<strong>+1</strong> approve, <strong>&minus;1</strong> reject); 3 net upvotes from 2+
        voters accepts a proposal, and a senior-curator veto rejects it instantly.
      </p>
    </header>

    {#if !canVote}
      <blockquote class="arc-empty">
        <p>
          Only curators can vote on proposals. If you're a curator, make sure you're
          signed in with the account holding the curator role.
        </p>
      </blockquote>
    {/if}

    {#if loading}
      <blockquote class="arc-empty"><p>Loading proposals&hellip;</p></blockquote>
    {:else if error}
      <div class="archive-error"><strong>&#9888; {error}</strong></div>
    {:else if proposals.length === 0}
      <blockquote class="arc-empty"><p>No pending proposals right now.</p></blockquote>
    {:else}
      <ul class="arc-proposal-list">
        {#each proposals as p (p.id)}
          <li class="arc-proposal">
            <div class="arc-proposal-main">
              <div class="arc-proposal-row">
                <span class="arc-chip">{p.action_type}</span>
                <span class="arc-chip">{p.status}</span>
                <span class="arc-chip arc-votes">+{p.vote_sum} / {p.voter_count} voter{p.voter_count === 1 ? '' : 's'}</span>
              </div>
              <strong class="arc-proposal-desc">{describe(p)}</strong>
              <span class="arc-muted">Proposed by user #{p.proposer_id} &middot; {new Date(p.created_at).toLocaleDateString()}</span>
              {#if p.details}
                <pre class="arc-details">{JSON.stringify(p.details, null, 2)}</pre>
              {/if}
            </div>

            {#if canVote}
              <div class="arc-vote-group">
                <button
                  class="archive-btn"
                  type="button"
                  onclick={() => handleVote(p, 1)}
                  disabled={votingId !== null}
                  aria-label={`Approve proposal ${p.id}`}
                >&#9650; +1</button>
                <button
                  class="archive-btn"
                  type="button"
                  onclick={() => handleVote(p, 0)}
                  disabled={votingId !== null}
                  aria-label={`Abstain on proposal ${p.id}`}
                >&mdash; 0</button>
                <button
                  class="archive-btn"
                  type="button"
                  onclick={() => handleVote(p, -1)}
                  disabled={votingId !== null}
                  aria-label={`Reject proposal ${p.id}`}
                >&#9660; &minus;1</button>
              </div>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}

    {#if voteMsg}
      <p class="arc-vote-msg" role="status" aria-live="polite">{voteMsg}</p>
    {/if}
  </div>
</main>
{:else}
<div class="proposals-page">
  <header class="page-head">
    <h1>🗳️ Work Proposals</h1>
    <p class="subtitle">
      Community-curated merge / split proposals for canonical works. Curators vote
      (<strong>+1</strong> approve, <strong>−1</strong> reject); 3 net upvotes from 2+
      voters accepts a proposal, and a senior-curator veto rejects it instantly.
    </p>
  </header>

  {#if !canVote}
    <div class="card notice">
      <p class="muted">
        Only curators can vote on proposals. If you're a curator, make sure you're
        signed in with the account holding the curator role.
      </p>
    </div>
  {/if}

  {#if loading}
    <p class="muted"><span class="spinner"></span> Loading proposals…</p>
  {:else if error}
    <div class="error-card"><strong>⚠️ {error}</strong></div>
  {:else if proposals.length === 0}
    <p class="empty">No pending proposals right now.</p>
  {:else}
    <ul class="proposal-list">
      {#each proposals as p (p.id)}
        <li class="card proposal-card">
          <div class="proposal-main">
            <div class="proposal-row">
              <span class="chip chip-type">{p.action_type}</span>
              <span class="chip chip-status">{p.status}</span>
              <span class="chip chip-votes">+{p.vote_sum} / {p.voter_count} voter{p.voter_count === 1 ? '' : 's'}</span>
            </div>
            <strong class="proposal-desc">{describe(p)}</strong>
            <span class="muted meta">Proposed by user #{p.proposer_id} · {new Date(p.created_at).toLocaleDateString()}</span>
            {#if p.details}
              <pre class="details">{JSON.stringify(p.details, null, 2)}</pre>
            {/if}
          </div>

          {#if canVote}
            <div class="vote-group">
              <button
                class="btn btn-vote up"
                onclick={() => handleVote(p, 1)}
                disabled={votingId !== null}
                aria-label={`Approve proposal ${p.id}`}
              >▲ +1</button>
              <button
                class="btn btn-vote neutral"
                onclick={() => handleVote(p, 0)}
                disabled={votingId !== null}
                aria-label={`Abstain on proposal ${p.id}`}
              >— 0</button>
              <button
                class="btn btn-vote down"
                onclick={() => handleVote(p, -1)}
                disabled={votingId !== null}
                aria-label={`Reject proposal ${p.id}`}
              >▼ −1</button>
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}

  {#if voteMsg}
    <p class="vote-msg" role="status" aria-live="polite">{voteMsg}</p>
  {/if}
</div>
{/if}

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
    max-width: 820px;
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
  }
  .archive-error {
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-link, #990000);
    padding: 0.8em 1em;
    font-weight: 700;
    margin: 0.5rem 0;
  }
  .archive-empty {
    border-left: 3px solid var(--archive-border, #dddddd);
    padding: 0.6em 1em;
    color: var(--archive-muted, #666666);
    font-style: italic;
    background: var(--archive-bg-raised, #f5f5f5);
    margin: 1rem 0;
  }
  .archive-empty p { margin: 0.2em 0; }

  .archive-btn {
    display: inline-block;
    padding: 0.25em 0.8em;
    font-size: 0.9em;
    font-family: inherit;
    font-weight: 600;
    line-height: 1.6;
    color: var(--archive-text, #2a2a2a);
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    cursor: pointer;
    text-decoration: none;
  }
  .archive-btn:hover:not(:disabled) {
    background: var(--archive-border, #dddddd);
    border-color: var(--archive-muted, #666666);
  }
  .archive-btn:disabled { opacity: 0.5; cursor: not-allowed; }

  .arc-empty {
    border-left: 3px solid var(--archive-border, #dddddd);
    padding: 0.6em 1em;
    color: var(--archive-muted, #666666);
    font-style: italic;
    background: var(--archive-bg-raised, #f5f5f5);
    margin: 1rem 0;
  }
  .arc-empty p { margin: 0.2em 0; font-style: normal; }
  .arc-muted { color: var(--archive-muted, #666666); font-size: 0.88em; }
  .arc-proposal-list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.75rem; }
  .arc-proposal {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.7em 1em;
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: flex-start;
    flex-wrap: wrap;
    background: var(--archive-bg, #ffffff);
  }
  .arc-proposal-main { display: flex; flex-direction: column; gap: 0.3rem; min-width: 0; flex: 1; }
  .arc-proposal-row { display: flex; gap: 0.4rem; flex-wrap: wrap; }
  .arc-chip { font-size: 0.78em; text-transform: uppercase; letter-spacing: 0.04em; border: 1px solid var(--archive-border, #dddddd); padding: 0.05em 0.4em; color: var(--archive-muted, #666666); align-self: flex-start; }
  .arc-votes { font-weight: 700; color: var(--archive-text, #2a2a2a); }
  .arc-proposal-desc { font-size: 1em; }
  .arc-details {
    font-family: ui-monospace, monospace;
    font-size: 0.8em;
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.5em;
    overflow-x: auto;
    max-width: 100%;
    margin: 0.2em 0 0;
  }
  .arc-vote-group { display: flex; gap: 0.4rem; align-items: center; }
  .arc-vote-msg { margin-top: 1rem; font-size: 0.88em; color: var(--archive-muted, #666666); }
  .proposals-page { max-width: 820px; margin: 0 auto; padding: 1.5rem; }
  .page-head { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 1.25rem; }
  .subtitle { color: var(--color-text-muted, #888); line-height: 1.55; }
  .notice { padding: 1rem; }
  .error-card { border-color: var(--color-error, #c0392b); padding: 1rem; }
  .proposal-list { list-style: none; display: flex; flex-direction: column; gap: 0.75rem; padding: 0; }
  .proposal-card { padding: 1rem; display: flex; justify-content: space-between; gap: 1rem; align-items: flex-start; flex-wrap: wrap; }
  .proposal-main { display: flex; flex-direction: column; gap: 0.35rem; min-width: 0; flex: 1; }
  .proposal-row { display: flex; gap: 0.4rem; flex-wrap: wrap; }
  .chip { align-self: flex-start; background: var(--color-chip-bg, #eef); padding: 0.15rem 0.5rem; border-radius: 999px; font-size: 0.75rem; }
  .chip-type { text-transform: uppercase; font-weight: 700; }
  .chip-status { background: var(--color-surface-2, #eee); }
  .chip-votes { background: var(--color-primary-alpha, #eef); color: var(--color-primary, #2b4bd7); font-weight: 700; }
  .proposal-desc { font-size: 1.02rem; }
  .meta { font-size: 0.82rem; }
  .details { font-size: 0.75rem; background: var(--color-surface-2, #f3f4f6); padding: 0.5rem; border-radius: var(--radius-sm, 8px); overflow-x: auto; max-width: 100%; }
  .vote-group { display: flex; gap: 0.4rem; align-items: center; }
  .btn-vote { padding: 0.3rem 0.7rem; font-size: 0.82rem; }
  .btn-vote.up:hover { border-color: #27ae60; color: #27ae60; }
  .btn-vote.down:hover { border-color: #c0392b; color: #c0392b; }
  .vote-msg { margin-top: 1rem; font-size: 0.85rem; color: var(--color-muted, #666); }
  .empty { text-align: center; color: var(--color-text-muted, #888); padding: 3rem 0; }
</style>
