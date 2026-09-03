<script lang="ts">
  // F7 user blocks: list blocked users, unblock, and block by user id.
  // Self-blocking → 400 (backend); block/unblock are idempotent.
  import { onMount } from 'svelte';
  import { listBlocks, blockUser, unblockUser, type BlockedUser } from '$lib/api/forum';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let loading = $state(true);
  let error = $state('');
  let items = $state<BlockedUser[]>([]);

  // Block-by-input form
  let userIdInput = $state('');
  let blocking = $state(false);
  let blockMsg = $state('');
  let blockError = $state('');

  let unblockingId = $state<number | null>(null);

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await listBlocks();
      if (res.err === 0) {
        items = res.items ?? [];
      } else {
        error = res.msg ?? t('forum.blocksLoadError');
      }
    } catch {
      error = t('forum.blocksLoadError');
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    await auth.init();
    await load();
  });

  async function handleBlock() {
    blockMsg = '';
    blockError = '';
    // Svelte 5 binds <input type="number"> as a number; coerce safely.
    const raw = String(userIdInput ?? '').trim();
    const id = Number(raw);
    if (!raw || !Number.isInteger(id) || id <= 0) {
      blockError = t('forum.blocksError');
      return;
    }
    blocking = true;
    try {
      const res = await blockUser(id);
      if (res.err === 0) {
        userIdInput = '';
        blockMsg = t('forum.blocksBlocked', { username: `#${id}` });
        await load();
      } else if (res.err === 400) {
        blockError = t('forum.blocksSelf');
      } else if (res.err === 404) {
        blockError = t('forum.blocksNotFound');
      } else {
        blockError = res.msg ?? t('forum.blocksError');
      }
    } catch {
      blockError = t('forum.blocksError');
    } finally {
      blocking = false;
    }
  }

  async function handleUnblock(userId: number) {
    unblockingId = userId;
    try {
      const res = await unblockUser(userId);
      if (res.err === 0) {
        items = items.filter((b) => b.user_id !== userId);
      }
    } catch {
      // Leave the row; a reload will reflect the real state.
    } finally {
      unblockingId = null;
    }
  }

  function formatDate(iso: string): string {
    try {
      return new Date(iso).toLocaleDateString();
    } catch {
      return iso;
    }
  }
</script>

<svelte:head><title>{t('forum.blocks')} — FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">{t('forum.blocks')}</h1>
      <p class="archive-summary">{t('forum.blocksSubtitle')}</p>
      <p>
        <a class="archive-link" href="/forum/apply">{t('forum.blocksToApply')}</a>
        &middot;
        <a class="archive-link" href="/forum/invites">{t('forum.blocksToInvites')}</a>
      </p>
      <p class="archive-backlink"><a class="archive-link" href="/forum">&larr; {t('forum.back')}</a></p>
    </header>

    {#if !auth.isLoggedIn}
      <blockquote class="archive-empty"><p>{t('forum.blocksLogin')}</p></blockquote>
    {:else}
      <form onsubmit={(e) => { e.preventDefault(); handleBlock(); }}>
        <fieldset class="archive-fieldset">
          <legend>{t('forum.blocksBlockByInput')}</legend>
          <dl class="archive-dl">
            <dt><label for="block-user-id">{t('forum.blocksUserIdPlaceholder')}</label></dt>
            <dd><input id="block-user-id" class="archive-input" type="number" min="1" bind:value={userIdInput} placeholder={t('forum.blocksUserIdPlaceholder')} /></dd>
          </dl>
          {#if blockMsg}<p class="archive-ok">{blockMsg}</p>{/if}
          {#if blockError}<div class="archive-error"><strong>&#9888; {blockError}</strong></div>{/if}
          <div class="archive-form-actions">
            <button class="archive-btn" type="submit" disabled={blocking}>
              {blocking ? '…' : t('forum.blocksBlock')}
            </button>
          </div>
        </fieldset>
      </form>

      {#if loading}
        <blockquote class="archive-empty"><p>{t('forum.loading')}</p></blockquote>
      {:else if error}
        <div class="archive-error"><strong>&#9888; {error}</strong></div>
      {:else if items.length === 0}
        <blockquote class="archive-empty"><p>{t('forum.blocksEmpty')}</p></blockquote>
      {:else}
        <div class="archive-table-wrap">
          <table class="archive-table">
            <thead>
              <tr>
                <th>Username</th>
                <th>Blocked On</th>
                <th>&nbsp;</th>
              </tr>
            </thead>
            <tbody>
              {#each items as b (b.user_id)}
                <tr>
                  <td><strong>{b.username}</strong> <span class="archive-muted">#{b.user_id}</span></td>
                  <td class="archive-muted">{(b.created_at ?? '').slice(0, 10)}</td>
                  <td>
                    <button class="archive-btn" type="button" onclick={() => handleUnblock(b.user_id)} disabled={unblockingId === b.user_id}>
                      {unblockingId === b.user_id ? '…' : t('forum.blocksUnblock')}
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    {/if}
  </div>
</main>
{:else}
<div class="blocks-page">
  <header class="page-head">
    <a class="back" href="/forum">{t('forum.back')}</a>
    <h1>{t('forum.blocks')}</h1>
    <p class="subtitle">{t('forum.blocksSubtitle')}</p>
    <p class="entry">
      <a class="link" href="/forum/apply">{t('forum.blocksToApply')}</a>
      <span class="muted">·</span>
      <a class="link" href="/forum/invites">{t('forum.blocksToInvites')}</a>
    </p>
  </header>

  {#if !auth.isLoggedIn}
    <div class="card empty"><p class="muted">{t('forum.blocksLogin')}</p></div>
  {:else}
    <form class="card block-form" onsubmit={(e) => { e.preventDefault(); handleBlock(); }}>
      <label>
        {t('forum.blocksBlockByInput')}
        <input type="number" min="1" bind:value={userIdInput} placeholder={t('forum.blocksUserIdPlaceholder')} />
      </label>
      {#if blockMsg}<p class="success-text">{blockMsg}</p>{/if}
      {#if blockError}<p class="error-card"><strong>⚠️ {blockError}</strong></p>{/if}
      <button class="btn btn-primary" type="submit" disabled={blocking}>
        {blocking ? '…' : t('forum.blocksBlock')}
      </button>
    </form>

    {#if loading}
      <p class="muted"><span class="spinner"></span> {t('forum.loading')}</p>
    {:else if error}
      <div class="error-card"><strong>⚠️ {error}</strong></div>
    {:else if items.length === 0}
      <p class="empty">{t('forum.blocksEmpty')}</p>
    {:else}
      <ul class="block-list">
        {#each items as b (b.user_id)}
          <li class="card block-item">
            <span class="username"><strong>{b.username}</strong> <span class="muted">#{b.user_id}</span></span>
            <span class="muted date">{(b.created_at ?? '').slice(0, 10)}</span>
            <button class="btn btn-secondary sm" type="button" onclick={() => handleUnblock(b.user_id)} disabled={unblockingId === b.user_id}>
              {unblockingId === b.user_id ? '…' : t('forum.blocksUnblock')}
            </button>
          </li>
        {/each}
      </ul>
    {/if}
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
  .archive-dl { margin: 0; padding: 0; }
  .archive-dl dt {
    font-weight: 700;
    font-size: 0.9em;
    margin: 0.85em 0 0.2em;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-dl dt:first-child { margin-top: 0; }
  .archive-dl dd { margin: 0; }
  .archive-input, .archive-select, .archive-textarea {
    width: 100%;
    box-sizing: border-box;
    padding: 0.5em 0.6em;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.95em;
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
  }
  .archive-textarea { resize: vertical; }
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
  .archive-form-actions { margin-top: 0.75em; display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap; }
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
  .archive-ok { color: #2e7d32; font-size: 0.92em; margin: 0.4em 0; }
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

  .archive-table-wrap { overflow-x: auto; margin: 0 0 1rem; }
  .archive-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.92em;
    border: 1px solid var(--archive-border, #dddddd);
  }
  .archive-table th, .archive-table td {
    text-align: left;
    padding: 0.45em 0.7em;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    vertical-align: top;
  }
  .archive-table th {
    color: var(--archive-muted, #666666);
    font-weight: 700;
    font-size: 0.82em;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .archive-table tbody tr:last-child td { border-bottom: none; }

  .archive-header > p:not(.archive-summary):not(.archive-backlink) { margin: 0.3em 0 0; font-size: 0.95em; }
  .blocks-page { max-width: 820px; margin: 0 auto; padding: 1.5rem; }
  .page-head { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 1.25rem; }
  .back { color: var(--color-link, #2b4bd7); text-decoration: none; font-size: 0.9rem; }
  .subtitle { color: var(--color-text-muted, #888); }
  .entry { font-size: 0.9rem; margin: 0; }
  .link { color: var(--color-link, #2b4bd7); text-decoration: none; font-weight: 600; }
  .link:hover { color: #999; }
  .block-form { display: flex; flex-direction: column; gap: 0.6rem; padding: 1rem; margin-bottom: 1rem; }
  .block-form label { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.9rem; }
  .block-form input { padding: 0.45rem 0.6rem; border-radius: var(--radius-sm, 6px); border: 1px solid var(--color-border, #ddd); font: inherit; max-width: 14rem; }
  .btn-primary { align-self: flex-start; }
  .btn.sm { padding: 0.35rem 0.7rem; font-size: 0.82rem; }
  .success-text { color: var(--color-success, #1e8e3e); margin: 0; font-size: 0.9rem; }
  .error-card { color: var(--color-error, #c0392b); }
  .block-list { list-style: none; display: flex; flex-direction: column; gap: 0.5rem; padding: 0; }
  .block-item { padding: 0.75rem 1rem; display: flex; align-items: center; gap: 0.75rem; flex-wrap: wrap; }
  .username { flex: 1; }
  .date { font-size: 0.8rem; }
  .empty { text-align: center; color: var(--color-text-muted, #888); padding: 3rem 0; }
</style>
