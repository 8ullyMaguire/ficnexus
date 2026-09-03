<script lang="ts">
  import { getBlindDate, revealBlindDate } from '$lib/api/blind';
  import type { BlindDateFic } from '$lib/api/blind';
  import { formatWords, stripHtml } from '$lib/util';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  // The current fic under the veil (title & fandom hidden).
  let fic = $state<BlindDateFic | null>(null);
  // url_ids already shown in this session — passed as ?exclude= so the
  // server never hands back a repeat.
  let seen = $state<string[]>([]);
  let loading = $state(false);
  let revealing = $state(false);
  let error = $state('');
  // The revealed metadata (title + fandom), fetched from the signed
  // reveal endpoint only after the user clicks "Reveal".
  let revealedMeta = $state<{ title: string; author: string; source: string; fandom: string | null } | null>(null);

  async function draw() {
    loading = true;
    error = '';
    revealedMeta = null;
    try {
      const res = await getBlindDate(seen);
      fic = res.fic;
      if (fic) {
        seen = [...seen, fic.url_id];
      } else {
        error = 'No fics left in the hat — try again later!';
      }
    } catch (e) {
      fic = null;
      error = e instanceof Error ? e.message : 'Could not load a blind date.';
    } finally {
      loading = false;
    }
  }

  async function reveal() {
    if (!fic?.reveal) return;
    revealing = true;
    error = '';
    try {
      const res = await revealBlindDate(fic.url_id, fic.reveal);
      revealedMeta = res.fic;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Could not reveal this fic.';
    } finally {
      revealing = false;
    }
  }
</script>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<div class="bd-archive">
  {#if loading}
    <p class="bd-arc-loading">Drawing a fic…</p>
  {:else if error && !fic}
    <p class="bd-arc-muted">{error}</p>
  {:else if fic}
    <dl class="archive-dl bd-arc-dl">
      <div class="dl-row">
        <dt>Title</dt>
        <dd>
          {#if revealedMeta}
            <a href={`/works/${fic.url_id}`} class="bd-arc-reveal-link">{revealedMeta.title}</a>
          {:else}
            ???
          {/if}
        </dd>
      </div>
      {#if revealedMeta?.fandom}
        <div class="dl-row">
          <dt>Fandom</dt>
          <dd>{revealedMeta.fandom}</dd>
        </div>
      {/if}
      <div class="dl-row">
        <dt>Stats</dt>
        <dd>
          {formatWords(fic.words)} words · {fic.chapters} chapters · {fic.status}
        </dd>
      </div>
      <div class="dl-row">
        <dt>Summary</dt>
        <dd class="bd-arc-desc">{stripHtml(fic.description).slice(0, 400)}</dd>
      </div>
      {#if fic.tropes.length > 0}
        <div class="dl-row">
          <dt>Tropes</dt>
          <dd>{fic.tropes.join(', ')}</dd>
        </div>
      {/if}
    </dl>

    {#if error}
      <p class="bd-arc-muted bd-arc-err">{error}</p>
    {/if}

    <div class="bd-arc-actions">
      {#if revealedMeta}
        <a class="archive-btn" href={`/works/${fic.url_id}`} data-sveltekit-reload>
          Read “{revealedMeta.title}”
        </a>
      {:else}
        <button class="archive-btn" type="button" onclick={reveal} disabled={loading || revealing || !fic.reveal}>
          {#if revealing}Revealing…{:else}Reveal{/if}
        </button>
      {/if}
      <button class="archive-btn" type="button" onclick={draw} disabled={loading || revealing}>
        {#if loading}Drawing…{:else}Another!{/if}
      </button>
    </div>
  {:else}
    <button class="archive-btn" type="button" onclick={draw} disabled={loading}>
      {#if loading}Drawing…{:else}Draw a fic{/if}
    </button>
  {/if}
</div>
{:else}
<div class="blind-date card">
  <div class="bd-head">
    <h2 class="bd-title">🎲 Blind Date with a Fic</h2>
    <p class="muted bd-sub">
      Title &amp; fandom hidden — judge by summary, length and tropes alone.
    </p>
  </div>

  <div class="bd-body">
    {#if loading}
      <div class="bd-loading"><span class="spinner"></span> Drawing a fic…</div>
    {:else if error && !fic}
      <p class="muted">{error}</p>
    {:else if fic}
      <div class="bd-card">
        <!-- Title hidden until reveal -->
        <h3 class="bd-mystery" aria-label="Hidden title">
          {#if revealedMeta}
            <a href={`/works/${fic.url_id}`} class="bd-reveal-link">{revealedMeta.title}</a>
          {:else}
            ???
          {/if}
        </h3>
        {#if revealedMeta?.fandom}
          <p class="muted bd-fandom">📁 {revealedMeta.fandom}</p>
        {/if}
        <p class="muted bd-status">
          {formatWords(fic.words)} words · {fic.chapters} chapters · {fic.status}
        </p>
        <p class="bd-desc">{stripHtml(fic.description).slice(0, 400)}</p>
        {#if fic.tropes.length > 0}
          <div class="bd-tropes">
            {#each fic.tropes as trope (trope)}
              <span class="tag">{trope}</span>
            {/each}
          </div>
        {/if}
      </div>

      {#if error}
        <p class="muted bd-err">{error}</p>
      {/if}

      <div class="bd-actions">
        {#if revealedMeta}
          <a class="btn btn-secondary sm" href={`/works/${fic.url_id}`} data-sveltekit-reload>
            📖 Read “{revealedMeta.title}”
          </a>
        {:else}
          <button class="btn btn-secondary sm" type="button" onclick={reveal} disabled={loading || revealing || !fic.reveal}>
            {#if revealing}<span class="spinner"></span> Revealing…{:else}👀 Reveal{/if}
          </button>
        {/if}
        <button class="btn sm" type="button" onclick={draw} disabled={loading || revealing}>
          {#if loading}<span class="spinner"></span> Drawing…{:else}🎲 Another!{/if}
        </button>
      </div>
    {:else}
      <button class="btn" type="button" onclick={draw} disabled={loading}>
        {#if loading}<span class="spinner"></span> Drawing…{:else}🎲 Draw a fic{/if}
      </button>
    {/if}
  </div>
</div>
{/if}

<style>
  /* ── Archive mode ─────────────────────────────────────────── */
  .bd-archive {
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .bd-arc-loading,
  .bd-arc-muted {
    color: var(--archive-muted, #666666);
    font-style: italic;
  }
  .bd-arc-err {
    margin-top: 0.6em;
    color: var(--color-danger, #cc0000);
  }
  .archive-dl.bd-arc-dl {
    margin: 0;
  }
  .bd-arc-dl .dl-row {
    display: grid;
    grid-template-columns: 120px 1fr;
    gap: 0.3em 1em;
    padding: 0.45em 0;
    border-bottom: 1px solid var(--archive-border, #eeeeee);
  }
  .bd-arc-dl .dl-row:last-child { border-bottom: none; }
  .bd-arc-dl dt {
    font-weight: 700;
    font-size: 0.9em;
    color: var(--archive-link, #990000);
  }
  .bd-arc-dl dd {
    margin: 0;
    font-size: 0.92em;
  }
  .bd-arc-desc {
    font-style: italic;
  }
  .bd-arc-reveal-link {
    font-weight: 700;
    font-size: 1.05em;
    color: var(--archive-link, #990000);
    text-decoration: none;
  }
  .bd-arc-reveal-link:hover { text-decoration: underline; }
  .bd-arc-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    flex-wrap: wrap;
    margin-top: 0.8em;
  }
  .bd-arc-actions .archive-btn,
  .archive-btn {
    display: inline-block;
    padding: 0.3rem 0.85rem;
    font-size: 0.88em;
    font-family: inherit;
    font-weight: 600;
    color: var(--archive-text, #2a2a2a);
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    cursor: pointer;
    line-height: 1.5;
    text-decoration: none;
  }
  .archive-btn:hover:not(:disabled) {
    background: var(--archive-border, #dddddd);
    border-color: var(--archive-muted, #666666);
    text-decoration: none;
  }
  .archive-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  @media (max-width: 600px) {
    .bd-arc-dl .dl-row { grid-template-columns: 1fr; }
  }

  /* ── Modern mode (unchanged) ── */
  .blind-date { padding: 1rem; }

  .bd-head { margin-bottom: 0.5rem; }
  .bd-title { margin: 0 0 0.15rem; font-size: 1.05rem; }
  .bd-sub { margin: 0; font-size: 0.82rem; }
  .bd-body { display: flex; flex-direction: column; gap: 0.6rem; }
  .bd-loading { padding: 0.4rem 0; font-size: 0.9rem; }
  .bd-card {
    border: 1px dashed var(--color-border);
    border-radius: var(--radius-sm);
    padding: 0.8rem 1rem;
    background: var(--color-surface-2);
  }
  .bd-mystery { margin: 0 0 0.2rem; font-size: 1rem; letter-spacing: 0.15em; }
  .bd-reveal-link { color: var(--color-text); letter-spacing: normal; }
  .bd-fandom { margin: 0 0 0.3rem; font-size: 0.82rem; }
  .bd-status { margin: 0 0 0.4rem; font-size: 0.82rem; }
  .bd-desc { margin: 0 0 0.5rem; font-size: 0.88rem; line-height: 1.45; }
  .bd-tropes { display: flex; flex-wrap: wrap; gap: 0.35rem; }
  .bd-actions { display: flex; gap: 0.5rem; align-items: center; }
  .bd-err { font-size: 0.82rem; color: var(--color-danger); }
</style>
