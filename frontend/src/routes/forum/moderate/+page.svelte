<script lang="ts">
  // Forum moderation queue (F5): lists posts awaiting moderator attention
  // with their score/mod_count badges. Clicking one of the nine reason
  // buttons applies it via POST /api/forum/posts/{id}/moderate and removes
  // the item from the list optimistically (the backend drops moderated posts
  // from the queue). Curators only (level ≥ 50) — the API enforces this.
  import { onMount } from 'svelte';
  import { getModerationQueue, moderatePost, getPostModerations, getModerationStatus, getModerationUserGrants, MODERATION_REASONS, type ForumModQueueItem, type ForumModerationEntry, type ForumUserGrant } from '$lib/api/forum';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let loading = $state(true);
  let error = $state('');
  let items = $state<ForumModQueueItem[]>([]);
  let total = $state(0);
  let workingPostId = $state<number | null>(null);
  let failurePostId = $state<number | null>(null);
  let pointsLeft = $state<number | null>(null);
  let unfairRate = $state<number | null>(null);
  let cooldownUntil = $state<string | null>(null);
  let history = $state<Record<number, ForumModerationEntry[]>>({});
  let historyOpen = $state<number | null>(null);
  let drawerUserId = $state<number | null>(null);
  let drawerGrants = $state<ForumUserGrant[]>([]);
  let drawerLoading = $state(false);
  let drawerError = $state('');
  async function openDrawer(userId: number) { if (drawerUserId===userId) { drawerUserId=null; return;} drawerUserId=userId; drawerLoading=true; drawerError=''; drawerGrants=[]; try { const r=await getModerationUserGrants(userId); if(r.err===0) drawerGrants=r.items??[]; else drawerError=r.msg??'Failed'; } catch { drawerError='Failed to load grants'; } finally { drawerLoading=false; } }

  async function toggleHistory(postId: number) {
    if (historyOpen === postId) { historyOpen = null; return; }
    historyOpen = postId;
    if (history[postId]) return;
    try {
      const res = await getPostModerations(postId);
      if (res.err === 0) history = { ...history, [postId]: res.items ?? [] };
    } catch { /* history is supplementary; keep queue usable */ }
  }

  function modCountLabel(count: number): string {
    return count === 1 ? t('forum.modCountOne', { count }) : t('forum.modCount', { count });
  }

  function queueCountLabel(count: number): string {
    return count === 1 ? t('forum.moderateCountOne', { count }) : t('forum.moderateCount', { count });
  }

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await getModerationQueue();
      if (res.err === 0) {
        items = res.items ?? [];
        total = res.count ?? items.length;
        if (pointsLeft === null) pointsLeft = res.points_left ?? null;
      } else {
        error = res.msg ?? t('forum.moderateError');
      }
    } catch {
      error = t('forum.moderateError');
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    await auth.init();
    try {
      const s = await getModerationStatus();
      if (s.err === 0) { unfairRate = s.unfair_rate ?? null; cooldownUntil = s.cooldown_until ?? null; }
    } catch {}
    await load();
  });

  // T037 drawer helper above
  async function apply(item: ForumModQueueItem, reasonKey: string) {
    if (workingPostId !== null) return;
    workingPostId = item.post_id;
    failurePostId = null;
    try {
      const res = await moderatePost(item.post_id, reasonKey);
      if (res.err === 0) {
        // Optimistic removal — the post is no longer in the queue.
        items = items.filter((it) => it.post_id !== item.post_id);
        total = Math.max(0, total - 1);
        if (pointsLeft !== null) pointsLeft = Math.max(0, pointsLeft - 1);
      } else if (res.err === 409) {
        // Already moderated (by me or anyone) — treat as resolved, drop it.
        items = items.filter((it) => it.post_id !== item.post_id);
        total = Math.max(0, total - 1);
      } else {
        failurePostId = item.post_id;
      }
    } catch {
      failurePostId = item.post_id;
    } finally {
      workingPostId = null;
    }
  }
</script>

<svelte:head><title>{t('forum.moderate')} — FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">{t('forum.moderate')}</h1>
      <p class="archive-summary">{t('forum.moderateSubtitle')}</p>
      {#if pointsLeft !== null}
        <p>{t('forum.modPointsLeft', { count: pointsLeft })}</p>
      {/if}
      {#if unfairRate !== null}
        <p class="archive-muted">Unfair rate: {(unfairRate*100).toFixed(1)}%{#if cooldownUntil} — cooldown until {cooldownUntil.slice(0,10)}{/if}</p>
      {/if}
      <p>
        <a class="archive-link" href="/forum/metamod">{t('forum.metamod')}</a>
        <span class="archive-muted">&mdash; {t('forum.metamodSubtitle')}</span>
      </p>
      <p class="archive-backlink"><a class="archive-link" href="/forum">&larr; {t('forum.back')}</a></p>
    </header>

    {#if loading}
      <blockquote class="archive-empty"><p>{t('forum.loading')}</p></blockquote>
    {:else if error}
      <div class="archive-error"><strong>{error}</strong></div>
    {:else if items.length === 0}
      <blockquote class="archive-empty"><p>{t('forum.moderateEmpty')}</p></blockquote>
    {:else}
      <p class="archive-count">{queueCountLabel(total)}</p>
      <ul class="archive-queue-list">
        {#each items as item (item.post_id)}
          <li class="archive-queue-item">
            <div class="archive-audit-meta">
              <a class="archive-link" href={item.topic_slug ? `/forum/board/${item.topic_slug}.${item.topic_id}` : `/forum/${item.category_slug ?? ''}/${item.topic_id}`}>
                <strong>{item.author_username ?? '&mdash;'}</strong>
              </a>
              <span class="archive-chip" class:neg={item.score < 0}>{item.score}</span>
              <span class="archive-chip">{modCountLabel(item.mod_count)}</span>
              <span class="archive-muted">{(item.created_at ?? '').slice(0, 10)}</span>
              {#if item.reason}
                <span class="archive-chip">{t(`forum.reason.${item.reason}`)}</span>
              {/if}
            </div>
            <p class="archive-excerpt">{item.body}</p>
            {#if failurePostId === item.post_id}
              <div class="archive-error"><strong>{t('forum.moderationFailed')}</strong></div>
            {/if}
            <div class="archive-reason-row" aria-label="moderation reasons">
              {#each MODERATION_REASONS as r (r.key)}
                <button
                  class="archive-btn"
                  class:pos={r.delta > 0}
                  class:neg={r.delta < 0}
                  type="button"
                  onclick={() => apply(item, r.key)}
                  disabled={workingPostId !== null}
                >
                  {r.delta > 0 ? '+' : ''}{r.delta} &middot; {t(r.labelKey)}
                </button>
              {/each}
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</main>
{:else}
<div class="moderate-page">
  <header class="page-head">
    <a class="back" href="/forum">{t('forum.back')}</a>
    <h1>{t('forum.moderate')}</h1>
    <p class="subtitle">{t('forum.moderateSubtitle')}</p>
    {#if pointsLeft !== null}
      <p class="points muted">{t('forum.modPointsLeft', { count: pointsLeft })}</p>
    {/if}
    {#if unfairRate !== null}
      <p class="muted">Unfair rate: {(unfairRate*100).toFixed(1)}%{#if cooldownUntil} — cooldown until {cooldownUntil.slice(0,10)}{/if}</p>
    {/if}
    <p class="entry">
      <a class="metamod-link" href="/forum/metamod">{t('forum.metamod')}</a>
      <span class="muted">— {t('forum.metamodSubtitle')}</span>
    </p>
  </header>

  {#if loading}
    <p class="muted"><span class="spinner"></span> {t('forum.loading')}</p>
  {:else if error}
    <div class="error-card"><strong>{error}</strong></div>
  {:else if items.length === 0}
    <p class="empty">{t('forum.moderateEmpty')}</p>
  {:else}
    <p class="count muted">{queueCountLabel(total)}</p>
    <ul class="queue-list">
      {#each items as item (item.post_id)}
        <li class="card queue-item">
          <div class="meta-row">
            <a class="author" href={item.topic_slug ? `/forum/board/${item.topic_slug}.${item.topic_id}` : `/forum/${item.category_slug ?? ''}/${item.topic_id}`}>
              <strong>{item.author_username ?? '—'}</strong>
            </a>
            <span class="chip score-badge" class:neg={item.score < 0}>{item.score}</span>
            <span class="chip">{modCountLabel(item.mod_count)}</span>
            <span class="muted date">{(item.created_at ?? '').slice(0, 10)}</span>
            {#if item.reason}
              <span class="chip reason-chip">{t(`forum.reason.${item.reason}`)}</span>
            {/if}
          </div>
          <p class="excerpt">{item.body}</p>
          <button class="btn-link" type="button" onclick={() => toggleHistory(item.post_id)} aria-expanded={historyOpen === item.post_id}>
            {historyOpen === item.post_id ? 'Hide moderation history' : 'View moderation history'}
          </button>
          {#if historyOpen === item.post_id && history[item.post_id]}
            <ul class="moderation-history" aria-label="moderation history">
              {#each history[item.post_id] as entry}
                <li>{t(`forum.reason.${entry.reason}`)} · {entry.delta > 0 ? '+' : ''}{entry.delta} · {entry.created_at.slice(0, 10)}</li>
              {:else}<li>No prior moderation actions.</li>{/each}
            </ul>
          {/if}
          {#if failurePostId === item.post_id}
            <p class="error-card"><strong>{t('forum.moderationFailed')}</strong></p>
          {/if}
          <div class="reason-row" aria-label="moderation reasons">
            {#each MODERATION_REASONS as r (r.key)}
              <button
                class="btn-link reason"
                class:pos={r.delta > 0}
                class:neg={r.delta < 0}
                type="button"
                onclick={() => apply(item, r.key)}
                disabled={workingPostId !== null}
              >
                {r.delta > 0 ? '+' : ''}{r.delta} · {t(r.labelKey)}
              </button>
            {/each}
          </div>
        </li>
      {/each}
    </ul>
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
  .archive-link { color: var(--archive-link, #990000); }
  .archive-muted { color: var(--archive-muted, #666666); }
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
  .archive-backlink { font-size: 0.9em; margin: 0.4em 0 0; }

  .archive-count { color: var(--archive-muted, #666666); font-size: 0.9em; margin: 0 0 0.75rem; }
  .archive-header > p:not(.archive-summary):not(.archive-backlink) { margin: 0.3em 0 0; font-size: 0.95em; }
  .archive-queue-list, .archive-audit-list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.75rem; }
  .archive-queue-item { border: 1px solid var(--archive-border, #dddddd); padding: 0.7em 1em; background: var(--archive-bg, #ffffff); }
  .archive-audit-meta { display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; font-size: 0.88em; }
  .archive-chip { font-size: 0.78em; text-transform: uppercase; letter-spacing: 0.04em; border: 1px solid var(--archive-border, #dddddd); padding: 0.05em 0.4em; color: var(--archive-muted, #666666); }
  .archive-chip.neg { color: #c0392b; border-color: #c0392b; }
  .archive-excerpt { margin: 0.4em 0 0.6em; line-height: 1.55; overflow-wrap: break-word; }
  .archive-reason-row { display: flex; flex-wrap: wrap; gap: 0.35rem; }
  .archive-btn.pos { color: #1e8e3e; }
  .archive-btn.neg { color: #c0392b; }
  .moderate-page { max-width: 820px; margin: 0 auto; padding: 1.5rem; }
  .page-head { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 1.25rem; }
  .back { color: var(--color-link, #2b4bd7); text-decoration: none; font-size: 0.9rem; }
  .subtitle { color: var(--color-text-muted, #888); }
  .points { font-size: 0.85rem; }
  .entry { font-size: 0.9rem; margin: 0; }
  .metamod-link { color: var(--color-link, #2b4bd7); font-weight: 600; text-decoration: none; }
  .metamod-link:hover { color: #999; }
  .count { font-size: 0.9rem; margin: 0 0 0.75rem; }
  .queue-list { list-style: none; display: flex; flex-direction: column; gap: 0.75rem; padding: 0; }
  .queue-item { padding: 1rem; display: flex; flex-direction: column; gap: 0.5rem; }
  .meta-row { display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; }
  .author { color: var(--color-link, #2b4bd7); text-decoration: none; }
  .chip { background: var(--color-chip-bg, #eef); padding: 0.15rem 0.5rem; border-radius: 999px; font-size: 0.75rem; }
  .score-badge { font-weight: 700; }
  .score-badge.neg { background: #fdecea; color: #c0392b; }
  .reason-chip { background: #fdecea; color: #c0392b; }
  .date { font-size: 0.8rem; }
  .excerpt { margin: 0; line-height: 1.5; overflow-wrap: break-word; }
  .reason-row { display: flex; flex-wrap: wrap; gap: 0.35rem; }
  .reason { font-size: 0.85rem; padding: 0.25rem 0.5rem; border: 1px solid var(--color-border, #ddd); border-radius: 999px; background: var(--color-surface, #fff); }
  .reason.pos { color: #1e8e3e; }
  .reason.neg { color: #c0392b; }
  .empty { text-align: center; color: var(--color-text-muted, #888); padding: 3rem 0; }
</style>
