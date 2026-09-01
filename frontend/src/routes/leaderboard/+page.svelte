<script lang="ts">
  import { onMount } from 'svelte';
  import { getWeeklyLeaderboard, getMonthlyLeaderboard } from '$lib/api/social';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let entries = $state<{ user_id: number; username: string; score: number; rank: number }[]>([]);
  let loading = $state(true);
  let error = $state('');
  let period = $state<'weekly' | 'monthly'>('weekly');

  onMount(async () => { await loadLeaderboard(); });

  async function loadLeaderboard() {
    loading = true; error = '';
    try {
      const res = period === 'weekly' ? await getWeeklyLeaderboard() : await getMonthlyLeaderboard();
      if (res.err !== 0) { error = 'Failed to load leaderboard.'; return; }
      entries = res.leaderboard;
    } catch { error = 'Network error loading leaderboard.'; }
    finally { loading = false; }
  }

  function rankBadge(rank: number): string {
    if (rank === 1) return '🥇';
    if (rank === 2) return '🥈';
    if (rank === 3) return '🥉';
    return `#${rank}`;
  }
</script>

<svelte:head><title>Rankings — FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">Rankings</h1>
      <p class="archive-summary">Top curators by reputation points earned.</p>
    </header>

    <div class="arc-period-tabs" role="tablist" aria-label="Leaderboard period">
      <button
        class="archive-btn arc-period-btn"
        class:current={period === 'weekly'}
        type="button"
        aria-pressed={period === 'weekly'}
        onclick={() => { period = 'weekly'; loadLeaderboard(); }}
      >Weekly</button>
      <button
        class="archive-btn arc-period-btn"
        class:current={period === 'monthly'}
        type="button"
        aria-pressed={period === 'monthly'}
        onclick={() => { period = 'monthly'; loadLeaderboard(); }}
      >Monthly</button>
    </div>

    {#if loading}
      <blockquote class="archive-empty"><p>Loading leaderboard...</p></blockquote>
    {:else if error}
      <div class="archive-error">
        <strong>&#9888; {error}</strong>
        <div><button class="archive-btn" type="button" onclick={() => loadLeaderboard()}>&#8635; Retry</button></div>
        <p>You can also try the other period above.</p>
      </div>
    {:else if entries.length === 0}
      <blockquote class="archive-empty"><p>No entries yet. Be the first to contribute!</p></blockquote>
    {:else}
      <ol class="arc-rank-list">
        {#each entries as entry (entry.user_id)}
          <li class="arc-rank-item" class:podium={entry.rank <= 3}>
            <span class="arc-rank">{rankBadge(entry.rank)}</span>
            <span class="arc-rank-user">{entry.username}</span>
            <span class="arc-rank-score">{entry.score.toLocaleString()}</span>
            <span class="arc-rank-unit">pts</span>
          </li>
        {/each}
      </ol>
    {/if}
  </div>
</main>
{:else}
<div class="leaderboard-page">
  <h1>🏆 Rankings</h1>
  <p class="muted intro">Top curators by reputation points earned.</p>

  <div class="period-tabs">
    <button class="period-tab" class:active={period === 'weekly'} onclick={() => { period = 'weekly'; loadLeaderboard(); }}>Weekly</button>
    <button class="period-tab" class:active={period === 'monthly'} onclick={() => { period = 'monthly'; loadLeaderboard(); }}>Monthly</button>
  </div>

  {#if loading}
    <div class="card"><p class="muted"><span class="spinner"></span> Loading leaderboard...</p></div>
  {:else if error}
    <div class="card error-card">
      <strong class="error-text">⚠️ {error}</strong>
      <button class="btn retry-btn" type="button" onclick={() => loadLeaderboard()}>↻ Retry</button>
      <p class="muted error-hint">You can also try the other period above.</p>
    </div>
  {:else if entries.length === 0}
    <div class="card empty"><p class="muted">No entries yet. Be the first to contribute!</p></div>
  {:else}
    <div class="leaderboard-list">
      {#each entries as entry, i (entry.user_id)}
        <div class="card entry" class:podium={entry.rank <= 3}>
          <div class="rank-col">
            <span class="rank" class:gold={entry.rank === 1} class:silver={entry.rank === 2} class:bronze={entry.rank === 3}>
              {rankBadge(entry.rank)}
            </span>
          </div>
          <div class="entry-main">
            <span class="username">{entry.username}</span>
          </div>
          <div class="score-col">
            <span class="score">{entry.score.toLocaleString()}</span>
            <span class="rep-label muted">pts</span>
          </div>
        </div>
      {/each}
    </div>
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

  .arc-period-tabs { display: flex; gap: 0.4rem; margin-bottom: 1.25rem; flex-wrap: wrap; }
  .arc-period-btn.current {
    color: var(--archive-bg, #ffffff);
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }
  .arc-rank-list { list-style: none; margin: 0; padding: 0; border-top: 1px solid var(--archive-border, #dddddd); counter-reset: rank; }
  .arc-rank-item {
    display: flex;
    align-items: baseline;
    gap: 0.8rem;
    padding: 0.55rem 0.6rem;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    font-size: 0.95em;
  }
  .arc-rank-item:nth-child(odd) { background: var(--archive-bg-raised, #f5f5f5); }
  .arc-rank { min-width: 2.2rem; text-align: center; font-weight: 700; color: var(--archive-muted, #666666); }
  .arc-rank-user { font-weight: 700; flex: 1; min-width: 0; }
  .arc-rank-score { font-family: ui-monospace, monospace; font-weight: 700; }
  .arc-rank-unit { color: var(--archive-muted, #666666); font-size: 0.85em; }
  .leaderboard-page { max-width: var(--max-width); margin: 0 auto; padding: 1rem; }
  h1 { margin-top: 0; margin-bottom: 0.3rem; }
  .intro { margin-bottom: 1.2rem; }
  .period-tabs { display: flex; gap: 0.3rem; margin-bottom: 1rem; }
  .period-tab { padding: 0.4rem 0.8rem; font-size: 0.85rem; border-radius: var(--radius-sm); background: var(--color-surface-2); border: 1px solid var(--color-border); cursor: pointer; color: var(--color-muted); }
  .period-tab.active { background: var(--color-primary); color: white; }
  .empty { text-align: center; padding: 2rem; }
  .error-card { padding: 1rem; display: flex; flex-direction: column; gap: 0.5rem; align-items: flex-start; }
  .retry-btn { margin-top: 0.2rem; }
  .error-hint { font-size: 0.85rem; margin: 0; }
  .error-card { border-color: var(--color-error); }
  .leaderboard-list { display: flex; flex-direction: column; gap: 0.5rem; }
  .entry { display: flex; align-items: center; gap: 1rem; padding: 0.9rem 1.2rem; }
  .entry.podium { border-color: var(--color-primary); border-width: 1px 1px 1px 4px; }
  .rank-col { width: 3rem; text-align: center; flex-shrink: 0; }
  .rank { font-size: 1.1rem; font-weight: 700; color: var(--color-muted); }
  .rank.gold { font-size: 1.4rem; }
  .entry-main { flex: 1; min-width: 0; }
  .username { font-weight: 600; font-size: 1rem; }
  .score-col { display: flex; align-items: baseline; gap: 0.3rem; flex-shrink: 0; }
  .score { font-family: var(--mono); font-weight: 700; font-size: 1.1rem; color: var(--color-primary-hover); }
  .rep-label { font-size: 0.78rem; }
  @media (max-width: 500px) { .entry { gap: 0.6rem; padding: 0.7rem 0.8rem; } .rank-col { width: 2.2rem; } }
</style>
