<script lang="ts">
  import { adminFetch } from '$lib/api/admin';
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  interface TriageRow {
    comment_id: number;
    body: string;
    category: string;
    reason: string;
    confidence: number;
    work_id: number | null;
    title: string | null;
    user_id: number | null;
  }

  let items = $state<TriageRow[]>([]);
  let loading = $state(true);
  let error = $state('');
  let actingId = $state<number | null>(null);
  let actionMsg = $state('');
  let actionErr = $state('');

  // Category badge colors, matching the LLM triage taxonomy in
  // src/routes/admin.rs (moderation_comments). Advisory only — nothing is
  // auto-hidden; the queue exists for human review.
  const CATEGORY_STYLE: Record<string, string> = {
    'fine': 'var(--color-success, #2a6)',
    'constructive': 'var(--color-primary, #38c)',
    'non-constructive': 'var(--color-warning, #d80)',
    'toxic': 'var(--color-error, #d33)',
    'spam': 'var(--color-error, #d33)',
  };

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await adminFetch('/api/admin/moderation/comments');
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      items = body.items ?? [];
    } catch (e) {
      error = `Failed to load comment triage: ${e instanceof Error ? e.message : e}`;
      items = [];
    } finally {
      loading = false;
    }
  }

  onMount(load);

  async function act(comment_id: number, action: 'hide' | 'delete') {
    actingId = comment_id;
    actionMsg = '';
    actionErr = '';
    try {
      const res = await adminFetch(`/api/admin/moderation/comments/${comment_id}/${action}`, {
        method: 'POST',
        credentials: 'include',
      });
      const body = await res.json().catch(() => ({}));
      if (!res.ok || body.err !== 0) {
        actionErr = `${action} failed: ${body.msg ?? `HTTP ${res.status}`}`;
        return;
      }
      // Both actions clear the triage row server-side, so remove the card
      // from the queue without a refetch.
      items = items.filter((it) => it.comment_id !== comment_id);
      actionMsg = `${action === 'hide' ? '🙈 Hidden' : '🗑️ Deleted'} comment #${comment_id}`;
    } catch (e) {
      actionErr = `${action} failed: ${e instanceof Error ? e.message : e}`;
    } finally {
      actingId = null;
    }
  }

  function confPct(c: number): string {
    if (c == null || Number.isNaN(c)) return '—';
    return `${Math.round(c * 100)}%`;
  }
</script>

{#if uiMode === 'archive'}
  <!-- ── Archive mode ─────────────────────────────────────────── -->
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Comment Triage</h1>
        <p class="archive-summary">
          Comments the LLM triage flagged as <strong>non-constructive</strong>,
          <strong>toxic</strong>, or <strong>spam</strong>, with the reason + confidence so a
          curator can decide without opening another page. Advisory only — nothing is
          auto-hidden. <strong>Hide</strong> flips the comment to hidden;
          <strong>Delete</strong> soft-deletes it; both clear it from this queue.
        </p>
      </header>

      <div class="archive-form-actions">
        <button class="archive-btn" type="button" onclick={load}>↻ Refresh</button>
      </div>

      {#if actionMsg}<p class="archive-msg archive-ok">{actionMsg}</p>{/if}
      {#if actionErr}<p class="archive-msg archive-err">{actionErr}</p>{/if}

      {#if loading}
        <p class="archive-note">Loading…</p>
      {:else if error}
        <p class="archive-error" role="alert">{error}</p>
      {:else if items.length === 0}
        <p class="archive-note">No flagged comments.</p>
      {:else}
        {#each items as item (item.comment_id)}
          {@const acting = actingId === item.comment_id}
          <fieldset class="archive-fieldset triage-item" class:dimmed={acting}>
            <legend class="archive-legend">
              <span class="cat">{item.category}</span>
              · confidence {confPct(item.confidence)}
            </legend>
            <dl class="archive-dl">
              <div class="dl-row">
                <dt>Comment</dt>
                <dd><blockquote class="body">{item.body}</blockquote></dd>
              </div>
              <div class="dl-row">
                <dt>Fic</dt>
                <dd>
                  <strong>{item.title ?? 'unknown fic'}</strong>
                  {#if item.work_id != null}
                    (<a class="archive-link" href={`/works/${item.work_id}`}>#{item.work_id}</a>)
                  {/if}
                </dd>
              </div>
              <div class="dl-row">
                <dt>User</dt>
                <dd>#{item.user_id ?? '—'}</dd>
              </div>
              {#if item.reason}
                <div class="dl-row">
                  <dt>Triage reason</dt>
                  <dd>{item.reason}</dd>
                </div>
              {/if}
            </dl>
            <div class="archive-form-actions">
              <button
                class="archive-btn"
                type="button"
                disabled={actingId !== null && !acting}
                onclick={() => act(item.comment_id, 'hide')}
              >
                {acting ? '…' : '🙈 Hide'}
              </button>
              {' '}
              <button
                class="archive-btn archive-btn-danger"
                type="button"
                disabled={actingId !== null && !acting}
                onclick={() => act(item.comment_id, 'delete')}
              >
                {acting ? '…' : '🗑️ Delete'}
              </button>
            </div>
          </fieldset>
        {/each}
      {/if}
    </div>
  </main>
{:else}
  <div class="triage-page">
  <header class="page-head">
    <h1>💬 Comment Triage</h1>
    <p class="subtitle">
      Comments the LLM triage flagged as <strong>non-constructive</strong>, <strong>toxic</strong>, or
      <strong>spam</strong>, with the reason + confidence so a curator can decide without opening
      another page. Advisory only — nothing is auto-hidden. <strong>Hide</strong> flips the comment to
      hidden; <strong>Delete</strong> soft-deletes it; both clear it from this queue.
    </p>
    <div class="controls">
      <button class="btn" onclick={load}>↻ Refresh</button>
    </div>
  </header>

  {#if actionMsg}<p class="msg ok">{actionMsg}</p>{/if}
  {#if actionErr}<p class="msg err">{actionErr}</p>{/if}

  {#if loading}
    <p class="empty">Loading…</p>
  {:else if error}
    <div class="error-card"><strong>⚠️ {error}</strong></div>
  {:else if items.length === 0}
    <p class="empty">No flagged comments. 🎉</p>
  {:else}
    <div class="comment-list">
      {#each items as item (item.comment_id)}
        {@const acting = actingId === item.comment_id}
        <div class="comment-card" class:dimmed={acting}>
          <div class="comment-head">
            <span class="badge" style="background: {CATEGORY_STYLE[item.category] ?? 'var(--color-surface-2, #f0f0f0)'}">
              {item.category}
            </span>
            <span class="conf">confidence {confPct(item.confidence)}</span>
            <span class="meta">
              on <strong>{item.title ?? 'unknown fic'}</strong>
              {#if item.work_id != null}
                (<a href={`/works/${item.work_id}`}>#{item.work_id}</a>)
              {/if}
              · user #{item.user_id ?? '—'}
            </span>
          </div>
          <blockquote class="body">{item.body}</blockquote>
          {#if item.reason}
            <p class="reason">💡 {item.reason}</p>
          {/if}
          <div class="actions">
            <button
              class="act hide"
              disabled={actingId !== null && !acting}
              onclick={() => act(item.comment_id, 'hide')}
            >
              {acting ? '…' : '🙈 Hide'}
            </button>
            <button
              class="act delete"
              disabled={actingId !== null && !acting}
              onclick={() => act(item.comment_id, 'delete')}
            >
              {acting ? '…' : '🗑️ Delete'}
            </button>
          </div>
        </div>
      {/each}
    </div>
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
  .archive-content { max-width: 900px; margin: 0 auto; padding: 1rem; }
  .archive-header {
    padding: 0.8rem 1rem 0.6rem;
    border-bottom: 2px solid var(--archive-link, #990000);
    background: var(--archive-bg, #ffffff);
  }
  .archive-page-title { font-size: 1.6em; font-weight: 700; margin: 0; }
  .archive-summary { color: var(--archive-muted, #666666); font-size: 0.95em; margin: 0.4em 0 0; }
  .archive-form-actions { margin-top: 0.9em; }
  div.archive-form-actions:first-of-type { margin-top: 1rem; margin-bottom: 1rem; }
  .archive-fieldset.triage-item {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 1em 1.25em;
    margin: 1.2rem 0;
  }
  .triage-item.dimmed { opacity: 0.55; }
  .archive-legend .cat { text-transform: uppercase; letter-spacing: 0.04em; }
  .archive-dl { margin: 0; padding: 0; }
  .archive-dl .dl-row { display: flex; }
  .archive-dl .dl-row dt {
    font-weight: 700;
    font-size: 0.88em;
    flex: 0 0 8em;
    margin: 0.5em 0;
  }
  .archive-dl .dl-row dd { margin: 0.5em 0; min-width: 0; flex: 1; }
  blockquote.body {
    margin: 0;
    padding: 0.4em 0.8em;
    background: var(--archive-bg-raised, #f5f5f5);
    border-left: 3px solid var(--archive-border, #dddddd);
    font-size: 0.92em;
    white-space: pre-wrap;
  }
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
  .archive-btn:disabled { opacity: 0.6; cursor: not-allowed; }
  .archive-btn-danger { color: var(--archive-link, #990000); border-color: var(--archive-link, #990000); }
  .archive-link { color: var(--archive-link, #990000); }
  .archive-msg { font-size: 0.92em; }
  .archive-ok { color: #2e7d32; }
  .archive-err { color: #990000; font-weight: 700; }
  .archive-error { color: #990000; font-weight: 700; font-size: 0.92em; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.9em; }

  /* ── Modern mode ──────────────────────────────────────────── */
  .triage-page { max-width: 900px; }
  .page-head h1 { margin-bottom: 0.3rem; }
  .subtitle { color: var(--color-text-muted, #888); font-size: 0.9rem; max-width: 90ch; }
  .controls { margin: 1rem 0; }
  .msg { margin: 0.5rem 0; font-size: 0.9rem; }
  .msg.ok { color: var(--color-success, #2a6); }
  .msg.err { color: var(--color-danger, #d33); }
  .comment-list { display: flex; flex-direction: column; gap: 0.75rem; }
  .comment-card { background: var(--color-surface, #fff); border: 1px solid var(--color-border, #ddd); border-radius: 12px; padding: 0.9rem 1rem; }
  .comment-card.dimmed { opacity: 0.55; }
  .comment-head { display: flex; align-items: center; gap: 0.6rem; flex-wrap: wrap; margin-bottom: 0.5rem; }
  .badge { display: inline-block; color: #fff; border-radius: 999px; padding: 0.1rem 0.6rem; font-size: 0.72rem; text-transform: uppercase; letter-spacing: 0.04em; }
  .conf { font-size: 0.78rem; color: var(--color-text-muted, #888); }
  .meta { font-size: 0.82rem; color: var(--color-text-muted, #888); margin-left: auto; }
  .body { margin: 0 0 0.5rem; padding: 0.6rem 0.8rem; background: var(--color-surface-2, #f4f4f4); border-left: 3px solid var(--color-border, #ccc); border-radius: 6px; font-size: 0.92rem; white-space: pre-wrap; }
  .reason { margin: 0 0 0.6rem; font-size: 0.85rem; color: var(--color-text-muted, #888); }
  .actions { display: flex; gap: 0.5rem; }
  .act { padding: 0.3rem 0.7rem; border-radius: 8px; border: 1px solid var(--color-border, #ccc); cursor: pointer; font-size: 0.82rem; }
  .act:disabled { opacity: 0.6; cursor: default; }
  .act.hide { background: color-mix(in srgb, var(--color-primary, #38c) 15%, transparent); color: var(--color-primary, #38c); }
  .act.delete { background: color-mix(in srgb, var(--color-danger, #d33) 12%, transparent); color: var(--color-danger, #d33); }
  .empty { padding: 2rem; text-align: center; color: var(--color-text-muted, #888); }
  .error-card { padding: 1rem; border: 1px solid var(--color-danger, #d33); border-radius: var(--radius-sm, 6px); }
</style>
