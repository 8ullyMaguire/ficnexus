<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { authHeaders } from '$lib/api/social';
  import { t } from '$lib/i18n/index.svelte';
  import DocLink from '$lib/components/DocLink.svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  interface Cluster {
    id: number;
    text: string;
    matches_played: number;
    elo_rating: number;
  }

  interface LeaderboardRow {
    id: number;
    text: string;
    elo_rating: number;
    matches_played: number;
    times_picked_best: number;
    times_picked_worst: number;
    suggestions: number;
    status: string;
  }

  interface ControversyRow {
    id: number;
    text: string;
    matches_played: number;
    times_picked_best: number;
    times_picked_worst: number;
    controversy: number;
    elo_rating: number;
  }

  let clusters = $state<Cluster[]>([]);
  let loading = $state(true);
  let error = $state('');
  let bestId = $state<number | null>(null);
  let worstId = $state<number | null>(null);
  let submitting = $state(false);
  let voted = $state(false);
  let message = $state('');
  let showSuggest = $state(false);
  let suggestText = $state('');
  let suggestMsg = $state('');
  let suggestErr = $state('');

  // Public consensus (leaderboard + controversy) — loaded lazily on first
  // toggle so the arena stays the focus. Endpoint is public: raw fetch,
  // no auth header needed (same as the arena).
  let showConsensus = $state(false);
  let consensusLoaded = $state(false);
  let consensusLoading = $state(false);
  let consensusErr = $state('');
  let leaderboard = $state<LeaderboardRow[]>([]);
  let controversy = $state<ControversyRow[]>([]);

  async function loadArena() {
    loading = true;
    error = '';
    bestId = null;
    worstId = null;
    try {
      const res = await fetch('/api/roadmap/arena', { credentials: 'include' });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      clusters = body.clusters ?? [];
      message = body.message ?? '';
    } catch (e) {
      error = `Failed to load arena: ${e instanceof Error ? e.message : e}`;
      clusters = [];
    } finally {
      loading = false;
    }
  }

  async function loadConsensus() {
    if (consensusLoaded || consensusLoading) return;
    consensusLoading = true;
    consensusErr = '';
    try {
      const res = await fetch('/api/roadmap/consensus', { credentials: 'include' });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      leaderboard = body.leaderboard ?? [];
      controversy = body.controversy ?? [];
      consensusLoaded = true;
    } catch (e) {
      consensusErr = `Failed to load consensus: ${e instanceof Error ? e.message : e}`;
    } finally {
      consensusLoading = false;
    }
  }

  function toggleConsensus() {
    showConsensus = !showConsensus;
    if (showConsensus) loadConsensus();
  }

  onMount(async () => {
    await auth.init();
    await loadArena();
  });

  function pick(id: number) {
    if (bestId === id) { bestId = null; return; }
    if (worstId === id) { worstId = null; return; }
    if (bestId === null) { bestId = id; return; }
    if (worstId === null && id !== bestId) { worstId = id; return; }
    // Toggle: clicking an already-picked one clears it (handled above);
    // clicking a third card replaces worst with it.
    if (id !== bestId) worstId = id;
  }

  function cardClass(id: number): string {
    if (bestId === id) return 'card best';
    if (worstId === id) return 'card worst';
    return 'card';
  }

  async function submitVote() {
    if (bestId === null || worstId === null) return;
    submitting = true;
    error = '';
    try {
      const res = await fetch('/api/roadmap/vote', {
        method: 'POST',
        credentials: 'include',
        headers: { 'content-type': 'application/json', ...authHeaders() },
        body: JSON.stringify({
          cluster_ids: clusters.map((c) => c.id),
          best_cluster_id: bestId,
          worst_cluster_id: worstId,
        }),
      });
      const body = await res.json();
      if (!res.ok || body.err !== 0) {
        error = body.msg ?? 'Vote failed';
        return;
      }
      voted = true;
      // Flip-to-impact: show new Elo briefly, then load the next set.
      const newElos = new Map((body.applied as { cluster_id: number; new_elo: number }[]).map((a) => [a.cluster_id, a.new_elo]));
      clusters = clusters.map((c) => ({ ...c, elo_rating: newElos.get(c.id) ?? c.elo_rating }));
      setTimeout(() => {
        voted = false;
        loadArena();
      }, 1200);
    } catch (e) {
      error = `Vote failed: ${e instanceof Error ? e.message : e}`;
    } finally {
      submitting = false;
    }
  }

  async function submitSuggestion() {
    const text = suggestText.trim();
    if (!text) return;
    suggestMsg = '';
    suggestErr = '';
    try {
      const res = await fetch('/api/roadmap/suggest', {
        method: 'POST',
        credentials: 'include',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ text }),
      });
      const body = await res.json();
      if (!res.ok || body.err !== 0) {
        suggestErr = body.msg ?? 'Suggestion failed';
        return;
      }
      suggestMsg = body.clustered
        ? t('roadmap.thanksJoined', { id: body.cluster_id })
        : t('roadmap.thanksSaved');
      suggestText = '';
    } catch (e) {
      suggestErr = `Suggestion failed: ${e instanceof Error ? e.message : e}`;
    }
  }

  function statusClass(status: string): string {
    if (status === 'open') return 'status open';
    if (status === 'implemented' || status === 'shipped') return 'status done';
    if (status === 'archived' || status === 'rejected') return 'status dead';
    return 'status';
  }

  function statusLabel(status: string): string {
    if (status === 'implemented') return 'shipped';
    if (status === 'archived') return 'rejected';
    return status;
  }

  // Most controversial first (best+worst picks), per the section heading.
  const sortedControversy = $derived(
    [...controversy].sort((a, b) => b.controversy - a.controversy),
  );
</script>

{#if uiMode === 'archive'}
<main class="archive-main">
  <div class="archive-content arc-roadmap">
    <header class="archive-header">
      <h1 class="archive-page-title">{t('roadmap.title')} <DocLink slug="features#roadmap-consensus" label="?" title="How the consensus arena works (MaxDiff/Elo)" /></h1>
      <p class="archive-summary">{@html t('roadmap.subtitle')}</p>
      <div class="arc-head-actions">
        <button class="archive-btn" type="button" onclick={() => (showSuggest = !showSuggest)}>
          {showSuggest ? t('roadmap.close') : t('roadmap.suggestFeature')}
        </button>
        <button class="archive-btn" type="button" onclick={toggleConsensus} data-testid="consensus-toggle">
          {showConsensus ? t('roadmap.hideConsensus') : t('roadmap.viewConsensus')}
        </button>
        {#if clusters.length > 0}
          <button class="archive-btn" type="button" onclick={loadArena} disabled={loading}>{t('roadmap.newSet')}</button>
        {/if}
      </div>
    </header>

    {#if showSuggest}
      <fieldset class="archive-fieldset arc-suggest">
        <legend>{t('roadmap.suggestTitle')}</legend>
        <p class="arc-hint">{t('roadmap.suggestHint')}</p>
        <textarea class="arc-suggest-input" bind:value={suggestText} maxlength="1000" rows="3" placeholder={t('roadmap.suggestPlaceholder')}></textarea>
        <div class="arc-suggest-actions">
          <button class="archive-btn" type="button" onclick={submitSuggestion} disabled={!suggestText.trim() || submitting}>
            {t('roadmap.submitIdea')}
          </button>
          {#if suggestMsg}<p class="arc-ok">{suggestMsg}</p>{/if}
          {#if suggestErr}<p class="arc-bad">{suggestErr}</p>{/if}
        </div>
      </fieldset>
    {/if}

    {#if loading}
      <blockquote class="arc-empty"><p>{t('roadmap.loadingArena')}</p></blockquote>
    {:else if error}
      <div class="archive-error"><strong>&#9888; {error}</strong></div>
    {:else if clusters.length === 0}
      <blockquote class="arc-empty"><p>{message || t('roadmap.noFeatures')}</p></blockquote>
    {:else}
      <fieldset class="archive-fieldset arc-arena-fieldset">
        <legend>{t('roadmap.pickBoth')}</legend>
        <div class="arena" class:impacting={voted}>
          {#each clusters as c (c.id)}
            <button class={cardClass(c.id)} data-testid="arena-card" onclick={() => pick(c.id)}>
              <span class="badge">{t('roadmap.votes', { count: c.matches_played })}</span>
              <span class="elo">{t('roadmap.elo', { score: Math.round(c.elo_rating) })}</span>
              <span class="text">{c.text}</span>
              {#if bestId === c.id}<span class="tag best-tag">{t('roadmap.mostImportant')}</span>{/if}
              {#if worstId === c.id}<span class="tag worst-tag">{t('roadmap.leastImportant')}</span>{/if}
            </button>
          {/each}
        </div>

        <div class="vote-bar">
          <p class="arc-hint">
            {#if bestId === null && worstId === null}
              {t('roadmap.pickBoth')}
            {:else if bestId !== null && worstId === null}
              {t('roadmap.pickLeast')}
            {:else}
              {t('roadmap.readyToSubmit')}
            {/if}
          </p>
          <button class="archive-btn" type="button" onclick={submitVote} disabled={bestId === null || worstId === null || submitting}>
            {voted ? t('roadmap.applied') : t('roadmap.submitVote')}
          </button>
        </div>
      </fieldset>
    {/if}

    {#if showConsensus}
      <section class="consensus arc-consensus" data-testid="consensus-section">
        <h2 class="arc-section-title">{t('roadmap.consensus')}</h2>
        <p class="arc-hint">
          {t('roadmap.consensusSub')}
        </p>

        {#if consensusLoading}
          <blockquote class="arc-empty"><p>{t('roadmap.loadingConsensus')}</p></blockquote>
        {:else if consensusErr}
          <div class="archive-error"><strong>&#9888; {consensusErr}</strong></div>
        {:else if leaderboard.length === 0}
          <blockquote class="arc-empty"><p>{t('roadmap.noFeaturesConsensus')}</p></blockquote>
        {:else}
          <div class="consensus-grid">
            <fieldset class="archive-fieldset arc-panel">
              <legend>{t('roadmap.leaderboard')}</legend>
              <div class="table-wrap">
                <table>
                  <thead>
                    <tr>
                      <th>#</th><th>{t('roadmap.feature')}</th><th>Elo</th><th>{t('roadmap.votesHeader')}</th>
                      <th>{t('roadmap.best')}</th><th>{t('roadmap.suggestions')}</th><th>{t('roadmap.status')}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each leaderboard as c, i (c.id)}
                      <tr>
                        <td>{i + 1}</td>
                        <td class="mono">{c.text}</td>
                        <td><span class="elo">{Math.round(c.elo_rating)}</span></td>
                        <td>{c.matches_played}</td>
                        <td class="best">{c.times_picked_best}</td>
                        <td>{c.suggestions}</td>
                        <td><span class={statusClass(c.status)}>{statusLabel(c.status)}</span></td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            </fieldset>

            <fieldset class="archive-fieldset arc-panel">
              <legend>{t('roadmap.mostControversial')}</legend>
              <ul class="controversy-list">
                {#each sortedControversy as c (c.id)}
                  <li class="controversy-item">
                    <span class="controversy-score">{c.controversy}</span>
                    <span class="controversy-text">
                      <span class="mono">{c.text}</span>
                      <span class="controversy-meta">
                        {t('roadmap.votes', { count: c.matches_played })} &middot; {t('roadmap.elo', { score: Math.round(c.elo_rating) })}
                      </span>
                    </span>
                  </li>
                {/each}
              </ul>
              <p class="arc-hint">{t('roadmap.scoreNote')}</p>
            </fieldset>
          </div>
        {/if}
      </section>
    {/if}
  </div>
</main>
{:else}
<div class="roadmap-page">
  <header class="page-head">
    <h1>🗺️ {t('roadmap.title')} <DocLink slug="features#roadmap-consensus" label="?" title="How the consensus arena works (MaxDiff/Elo)" /></h1>
    <p class="subtitle">
      {@html t('roadmap.subtitle')}
    </p>
    <div class="head-actions">
      <button class="btn btn-primary" onclick={() => (showSuggest = !showSuggest)}>
        {showSuggest ? t('roadmap.close') : t('roadmap.suggestFeature')}
      </button>
      <button class="btn" onclick={toggleConsensus} data-testid="consensus-toggle">
        {showConsensus ? t('roadmap.hideConsensus') : t('roadmap.viewConsensus')}
      </button>
      {#if clusters.length > 0}
        <button class="btn" onclick={loadArena} disabled={loading}>{t('roadmap.newSet')}</button>
      {/if}
    </div>
  </header>

  {#if showSuggest}
    <div class="suggest-box">
      <h3>{t('roadmap.suggestTitle')}</h3>
      <p class="hint">{t('roadmap.suggestHint')}</p>
      <textarea bind:value={suggestText} maxlength="1000" rows="3" placeholder={t('roadmap.suggestPlaceholder')}></textarea>
      <div class="suggest-actions">
        <button class="btn btn-primary" onclick={submitSuggestion} disabled={!suggestText.trim() || submitting}>
          {t('roadmap.submitIdea')}
        </button>
        {#if suggestMsg}<p class="ok">{suggestMsg}</p>{/if}
        {#if suggestErr}<p class="bad">{suggestErr}</p>{/if}
      </div>
    </div>
  {/if}

  {#if loading}
    <p class="empty">{t('roadmap.loadingArena')}</p>
  {:else if error}
    <div class="error-card"><strong>⚠️ {error}</strong></div>
  {:else if clusters.length === 0}
    <p class="empty">{message || t('roadmap.noFeatures')}</p>
  {:else}
    <div class="arena" class:impacting={voted}>
      {#each clusters as c (c.id)}
        <button class={cardClass(c.id)} data-testid="arena-card" onclick={() => pick(c.id)}>
          <span class="badge">{t('roadmap.votes', { count: c.matches_played })}</span>
          <span class="elo">{t('roadmap.elo', { score: Math.round(c.elo_rating) })}</span>
          <span class="text">{c.text}</span>
          {#if bestId === c.id}<span class="tag best-tag">{t('roadmap.mostImportant')}</span>{/if}
          {#if worstId === c.id}<span class="tag worst-tag">{t('roadmap.leastImportant')}</span>{/if}
        </button>
      {/each}
    </div>

    <div class="vote-bar">
      <p class="legend">
        {#if bestId === null && worstId === null}
          {t('roadmap.pickBoth')}
        {:else if bestId !== null && worstId === null}
          {t('roadmap.pickLeast')}
        {:else}
          {t('roadmap.readyToSubmit')}
        {/if}
      </p>
      <button class="btn btn-primary" onclick={submitVote} disabled={bestId === null || worstId === null || submitting}>
        {voted ? t('roadmap.applied') : t('roadmap.submitVote')}
      </button>
    </div>
  {/if}

  {#if showConsensus}
    <section class="consensus" data-testid="consensus-section">
      <h2>📊 {t('roadmap.consensus')}</h2>
      <p class="consensus-sub">
        {t('roadmap.consensusSub')}
      </p>

      {#if consensusLoading}
        <p class="empty">{t('roadmap.loadingConsensus')}</p>
      {:else if consensusErr}
        <div class="error-card"><strong>⚠️ {consensusErr}</strong></div>
      {:else if leaderboard.length === 0}
        <p class="empty">{t('roadmap.noFeaturesConsensus')}</p>
      {:else}
        <div class="consensus-grid">
          <div class="consensus-panel">
            <h3>{t('roadmap.leaderboard')}</h3>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>#</th><th>{t('roadmap.feature')}</th><th>Elo</th><th>{t('roadmap.votesHeader')}</th>
                    <th>{t('roadmap.best')}</th><th>{t('roadmap.suggestions')}</th><th>{t('roadmap.status')}</th>
                  </tr>
                </thead>
                <tbody>
                  {#each leaderboard as c, i (c.id)}
                    <tr>
                      <td>{i + 1}</td>
                      <td class="mono">{c.text}</td>
                      <td><span class="elo">{Math.round(c.elo_rating)}</span></td>
                      <td>{c.matches_played}</td>
                      <td class="best">{c.times_picked_best}</td>
                      <td>{c.suggestions}</td>
                      <td><span class={statusClass(c.status)}>{statusLabel(c.status)}</span></td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>

          <div class="consensus-panel">
            <h3>{t('roadmap.mostControversial')}</h3>
            <ul class="controversy-list">
              {#each sortedControversy as c (c.id)}
                <li class="controversy-item">
                  <span class="controversy-score">{c.controversy}</span>
                  <span class="controversy-text">
                    <span class="mono">{c.text}</span>
                    <span class="controversy-meta">
                      {t('roadmap.votes', { count: c.matches_played })} · {t('roadmap.elo', { score: Math.round(c.elo_rating) })}
                    </span>
                  </span>
                </li>
              {/each}
            </ul>
            <p class="consensus-note">{t('roadmap.scoreNote')}</p>
          </div>
        </div>
      {/if}
    </section>
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
    max-width: 960px;
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

  .archive-fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    padding: 1em 1.25em;
    margin: 0 0 1.25rem;
    background: var(--archive-bg, #ffffff);
  }
  .archive-fieldset legend {
    font-family: Georgia, 'Times New Roman', serif;
    font-weight: 700;
    font-size: 1.05em;
    color: var(--archive-text, #2a2a2a);
    padding: 0 0.4em;
  }

  .arc-head-actions { display: flex; gap: 0.5rem; margin-top: 0.75em; flex-wrap: wrap; }
  .arc-suggest-input {
    width: 100%;
    box-sizing: border-box;
    padding: 0.5em 0.6em;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.95em;
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
    resize: vertical;
  }
  .arc-suggest-actions { display: flex; align-items: center; gap: 0.75rem; margin-top: 0.6em; flex-wrap: wrap; }
  .arc-ok { color: #2e7d32; margin: 0; }
  .arc-bad { color: #c0392b; margin: 0; }
  .arc-hint { color: var(--archive-muted, #666666); font-size: 0.88em; margin: 0.3em 0; }
  .arc-empty {
    border-left: 3px solid var(--archive-border, #dddddd);
    padding: 0.6em 1em;
    color: var(--archive-muted, #666666);
    font-style: italic;
    background: var(--archive-bg-raised, #f5f5f5);
    margin: 1rem 0;
  }
  .arc-empty p { margin: 0.2em 0; }
  .arc-arena-fieldset .vote-bar { display: flex; align-items: center; justify-content: space-between; gap: 0.75rem; margin-top: 1em; flex-wrap: wrap; }
  /* Archive restyle of arena widgets — behaviour untouched */
  .arc-roadmap .arena { gap: 0.75rem; }
  .arc-roadmap .arena .card {
    border-radius: 0;
    background: var(--archive-bg-raised, #f5f5f5);
    border-color: var(--archive-border, #dddddd);
    font-family: Georgia, 'Times New Roman', serif;
  }
  .arc-roadmap .arena .card:hover { border-color: var(--archive-muted, #666666); }
  .arc-roadmap .arena .card.best { border-color: #2e7d32; }
  .arc-roadmap .arena .card.worst { border-color: #c0392b; }
  .arc-roadmap .tag { border-radius: 0; }
  .arc-roadmap .consensus { border-top: none; margin-top: 0; padding-top: 0; }
  .arc-section-title {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.35em;
    font-weight: 700;
    color: var(--archive-heading, #990000);
    margin: 0 0 0.2em;
  }
  .arc-roadmap .consensus-grid { align-items: start; }
  .arc-roadmap .arc-panel { margin: 0; }
  .arc-roadmap table { border: 1px solid var(--archive-border, #dddddd); }
  .arc-roadmap th, .arc-roadmap td { border-bottom-color: var(--archive-border, #dddddd); }
  .arc-roadmap .status { border-radius: 0; }
  .arc-roadmap .controversy-item { border-radius: 0; background: var(--archive-bg-raised, #f5f5f5); border-color: var(--archive-border, #dddddd); }
  .roadmap-page { max-width: 1100px; margin: 0 auto; padding: 1rem; }
  .page-head h1 { margin-bottom: 0.3rem; }
  .subtitle { color: var(--color-text-muted, #888); max-width: 75ch; }
  .head-actions { display: flex; gap: 0.75rem; margin: 1rem 0; flex-wrap: wrap; }
  .suggest-box { background: var(--color-surface-2, #f6f6f6); border: 1px solid var(--color-border, #ddd); border-radius: 12px; padding: 1rem; margin-bottom: 1.5rem; }
  .suggest-box textarea { width: 100%; padding: 0.6rem; border-radius: 8px; border: 1px solid var(--color-border, #ccc); font: inherit; color: var(--color-text, #222); background: var(--color-surface, #fff); }
  .suggest-box h3 { color: var(--color-text, #222); }
  .suggest-actions { display: flex; align-items: center; gap: 1rem; margin-top: 0.6rem; }
  .ok { color: var(--color-success, #2a6); }
  .bad { color: var(--color-danger, #d33); }
  .arena { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }
  .card { position: relative; min-height: 140px; display: flex; flex-direction: column; align-items: flex-start; gap: 0.3rem; padding: 1rem; border: 2px solid var(--color-border, #ddd); border-radius: 12px; background: var(--color-surface, #fff); cursor: pointer; text-align: left; font: inherit; color: var(--color-text, #222); transition: border-color 0.15s, transform 0.1s; }
  .card:hover { border-color: var(--color-primary, #38c); }
  .card.best { border-color: var(--color-success, #2a6); box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-success, #2a6) 15%, transparent); }
  .card.worst { border-color: var(--color-danger, #d33); box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-danger, #d33) 15%, transparent); }
  .badge { font-size: 0.75rem; color: var(--color-text-muted, #888); }
  .elo { font-size: 0.75rem; color: var(--color-text-muted, #888); }
  .text { font-size: 1rem; font-weight: 500; color: var(--color-text, #222); }
  .tag { font-size: 0.75rem; padding: 0.15rem 0.5rem; border-radius: 999px; }
  .best-tag { background: color-mix(in srgb, var(--color-success, #2a6) 15%, transparent); color: var(--color-success, #2a6); }
  .worst-tag { background: color-mix(in srgb, var(--color-danger, #d33) 15%, transparent); color: var(--color-danger, #d33); }
  .vote-bar { display: flex; align-items: center; justify-content: space-between; margin-top: 1.25rem; }
  .legend { color: var(--color-text-muted, #888); }
  .empty { padding: 3rem; text-align: center; color: var(--color-text-muted, #888); }
  .error-card { padding: 1rem; border: 1px solid var(--color-danger, #d33); border-radius: 8px; }
  .arena.impacting .card { transition: transform 0.3s; }

  .consensus { margin-top: 2.5rem; border-top: 1px solid var(--color-border, #ddd); padding-top: 1.5rem; }
  .consensus h2 { font-size: 1.25rem; margin-bottom: 0.3rem; }
  .consensus-sub { color: var(--color-text-muted, #888); font-size: 0.9rem; max-width: 80ch; margin-bottom: 1.25rem; }
  .consensus-grid { display: grid; grid-template-columns: 1.4fr 1fr; gap: 1.25rem; align-items: start; }
  .consensus-panel { background: var(--color-surface-2, #f6f6f6); border: 1px solid var(--color-border, #ddd); border-radius: 12px; padding: 1rem; }
  .consensus-panel h3 { font-size: 1rem; margin-bottom: 0.6rem; color: var(--color-text, #222); }
  .table-wrap { overflow-x: auto; }
  table { width: 100%; border-collapse: collapse; font-size: 0.9rem; }
  th, td { text-align: left; padding: 0.45rem 0.7rem; border-bottom: 1px solid var(--color-border, #eee); }
  th { font-weight: 600; color: var(--color-text-muted, #888); font-size: 0.78rem; text-transform: uppercase; }
  .mono { font-family: monospace; }
  .best { color: var(--color-success, #2a6); }
  .status { font-size: 0.72rem; padding: 0.15rem 0.5rem; border-radius: 999px; background: var(--color-surface, #fff); border: 1px solid var(--color-border, #ccc); color: var(--color-text-muted, #888); text-transform: capitalize; white-space: nowrap; }
  .status.open { border-color: var(--color-primary, #38c); color: var(--color-primary, #38c); background: color-mix(in srgb, var(--color-primary, #38c) 10%, transparent); }
  .status.done { border-color: var(--color-success, #2a6); color: var(--color-success, #2a6); background: color-mix(in srgb, var(--color-success, #2a6) 10%, transparent); }
  .status.dead { border-color: var(--color-danger, #d33); color: var(--color-danger, #d33); background: color-mix(in srgb, var(--color-danger, #d33) 10%, transparent); }
  .controversy-list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.5rem; }
  .controversy-item { display: flex; align-items: flex-start; gap: 0.6rem; padding: 0.5rem 0.6rem; background: var(--color-surface, #fff); border: 1px solid var(--color-border, #ddd); border-radius: 8px; }
  .controversy-score { flex-shrink: 0; font-weight: 700; color: var(--color-danger, #d33); min-width: 1.6rem; text-align: center; }
  .controversy-text { display: flex; flex-direction: column; gap: 0.15rem; }
  .controversy-meta { font-size: 0.75rem; color: var(--color-text-muted, #888); }
  .consensus-note { font-size: 0.78rem; color: var(--color-text-muted, #888); margin-top: 0.6rem; }

  @media (max-width: 640px) {
    .arena { grid-template-columns: 1fr; }
    .consensus-grid { grid-template-columns: 1fr; }
  }
</style>
