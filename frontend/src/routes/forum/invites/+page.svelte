<script lang="ts">
  // F7 invite management (level ≥ 50): generate single-use invite codes and
  // list existing invites with usage. The backend only allows this when
  // REGISTRATION_MODE=invite; the API enforces the level gate.
  import { onMount } from 'svelte';
  import { createInvite, listInvites, type ForumInvite } from '$lib/api/forum';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let loading = $state(true);
  let error = $state('');
  let items = $state<ForumInvite[]>([]);

  // Create form
  let note = $state('');
  let creating = $state(false);
  let createdCode = $state('');
  let createError = $state('');

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await listInvites();
      if (res.err === 0) {
        items = res.items ?? [];
      } else {
        error = res.msg ?? t('forum.invitesDenied');
      }
    } catch {
      error = t('forum.invitesDenied');
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    await auth.init();
    await load();
  });

  async function handleCreate() {
    createError = '';
    createdCode = '';
    creating = true;
    try {
      const res = await createInvite({ note: note.trim() || undefined });
      if (res.err === 0 && res.code) {
        createdCode = res.code;
        note = '';
        await load();
      } else {
        createError = res.msg ?? t('forum.invitesDenied');
      }
    } catch {
      createError = t('forum.invitesDenied');
    } finally {
      creating = false;
    }
  }

  function formatDate(iso: string | null): string {
    if (!iso) return t('forum.invitesNoExpiry');
    try {
      return new Date(iso).toLocaleDateString();
    } catch {
      return iso;
    }
  }
</script>

<svelte:head><title>{t('forum.invites')} — FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">{t('forum.invites')}</h1>
      <p class="archive-summary">{t('forum.invitesSubtitle')}</p>
      <p class="archive-backlink"><a class="archive-link" href="/forum">&larr; {t('forum.back')}</a></p>
    </header>

    <form onsubmit={(e) => { e.preventDefault(); handleCreate(); }}>
      <fieldset class="archive-fieldset">
        <legend>{t('forum.invitesGenerate')}</legend>
        <dl class="archive-dl">
          <dt><label for="invite-note">{t('forum.invitesNote')}</label></dt>
          <dd><input id="invite-note" class="archive-input" type="text" bind:value={note} placeholder={t('forum.invitesNotePlaceholder')} /></dd>
        </dl>
        {#if createError}<div class="archive-error"><strong>&#9888; {createError}</strong></div>{/if}
        <div class="archive-form-actions">
          <button class="archive-btn" type="submit" disabled={creating}>
            {creating ? t('forum.invitesGenerating') : t('forum.invitesGenerate')}
          </button>
        </div>
      </fieldset>
    </form>

    {#if createdCode}
      <blockquote class="archive-invite-created">
        <p>{t('forum.invitesCreated')}</p>
        <code>{createdCode}</code>
      </blockquote>
    {/if}

    {#if loading}
      <blockquote class="archive-empty"><p>{t('forum.loading')}</p></blockquote>
    {:else if error}
      <div class="archive-error"><strong>&#9888; {error}</strong></div>
    {:else if items.length === 0}
      <blockquote class="archive-empty"><p>{t('forum.invitesEmpty')}</p></blockquote>
    {:else}
      <div class="archive-table-wrap">
        <table class="archive-table">
          <thead>
            <tr>
              <th>Code</th>
              <th>Status</th>
              <th>Expires</th>
            </tr>
          </thead>
          <tbody>
            {#each items as inv (inv.id)}
              <tr>
                <td><code class="archive-code">{inv.code}</code></td>
                <td>
                  {#if inv.used_username}
                    {t('forum.invitesUsedBy', { username: inv.used_username })}
                  {:else}
                    {t('forum.invitesUnused')}
                  {/if}
                </td>
                <td class="archive-muted">{t('forum.invitesExpires', { date: formatDate(inv.expires_at) })}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
</main>
{:else}
<div class="invites-page">
  <header class="page-head">
    <a class="back" href="/forum">{t('forum.back')}</a>
    <h1>{t('forum.invites')}</h1>
    <p class="subtitle">{t('forum.invitesSubtitle')}</p>
  </header>

  <form class="card create-form" onsubmit={(e) => { e.preventDefault(); handleCreate(); }}>
    <label>
      {t('forum.invitesNote')}
      <input type="text" bind:value={note} placeholder={t('forum.invitesNotePlaceholder')} />
    </label>
    {#if createError}<p class="error-card"><strong>⚠️ {createError}</strong></p>{/if}
    <button class="btn btn-primary" type="submit" disabled={creating}>
      {creating ? t('forum.invitesGenerating') : t('forum.invitesGenerate')}
    </button>
  </form>

  {#if createdCode}
    <div class="card success-card created">
      <p>{t('forum.invitesCreated')}</p>
      <code class="invite-code">{createdCode}</code>
    </div>
  {/if}

  {#if loading}
    <p class="muted"><span class="spinner"></span> {t('forum.loading')}</p>
  {:else if error}
    <div class="error-card"><strong>⚠️ {error}</strong></div>
  {:else if items.length === 0}
    <p class="empty">{t('forum.invitesEmpty')}</p>
  {:else}
    <ul class="invite-list">
      {#each items as inv (inv.id)}
        <li class="card invite-item">
          <code class="invite-code">{inv.code}</code>
          <span class="meta">
            {#if inv.used_username}
              {t('forum.invitesUsedBy', { username: inv.used_username })}
            {:else}
              {t('forum.invitesUnused')}
            {/if}
            · {t('forum.invitesExpires', { date: formatDate(inv.expires_at) })}
          </span>
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

  .archive-code { font-family: ui-monospace, monospace; font-size: 0.92em; }
  .archive-invite-created {
    border-left: 3px solid var(--archive-border, #dddddd);
    padding: 0.6em 1em;
    background: var(--archive-bg-raised, #f5f5f5);
    margin: 0 0 1rem;
  }
  .archive-invite-created p { margin: 0 0 0.25em; font-size: 0.92em; color: var(--archive-text, #2a2a2a); }
  .invites-page { max-width: 820px; margin: 0 auto; padding: 1.5rem; }
  .page-head { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 1.25rem; }
  .back { color: var(--color-link, #2b4bd7); text-decoration: none; font-size: 0.9rem; }
  .subtitle { color: var(--color-text-muted, #888); }
  .create-form { display: flex; flex-direction: column; gap: 0.6rem; padding: 1rem; margin-bottom: 1rem; }
  .create-form label { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.9rem; }
  .create-form input { padding: 0.45rem 0.6rem; border-radius: var(--radius-sm, 6px); border: 1px solid var(--color-border, #ddd); font: inherit; }
  .created { padding: 1rem; margin-bottom: 1rem; }
  .created p { margin: 0 0 0.5rem; }
  .invite-code { font-family: ui-monospace, monospace; font-size: 0.95rem; background: var(--color-surface-2, #1c1c1c); padding: 0.25rem 0.5rem; border-radius: 6px; }
  .invite-list { list-style: none; display: flex; flex-direction: column; gap: 0.5rem; padding: 0; }
  .invite-item { padding: 0.75rem 1rem; display: flex; align-items: center; justify-content: space-between; gap: 0.75rem; flex-wrap: wrap; }
  .meta { color: var(--color-text-muted, #888); font-size: 0.85rem; }
  .empty { text-align: center; color: var(--color-text-muted, #888); padding: 3rem 0; }
  .error-card { color: var(--color-error, #c0392b); }
  .btn-primary { align-self: flex-start; }
</style>
