<script lang="ts">
  // F7 registration application: when the site runs REGISTRATION_MODE=application,
  // logged-in (pending) users apply to join with a reason. One pending
  // application per user — the backend 409s duplicates.
  import { onMount } from 'svelte';
  import { applyForRegistration } from '$lib/api/forum';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let reason = $state('');
  let submitting = $state(false);
  let submitted = $state(false);
  let error = $state('');

  onMount(async () => {
    await auth.init();
  });

  async function handleSubmit() {
    error = '';
    if (!reason.trim()) {
      error = t('forum.applyError');
      return;
    }
    submitting = true;
    try {
      const res = await applyForRegistration(reason.trim());
      if (res.err === 0) {
        submitted = true;
        reason = '';
      } else if (res.err === 409) {
        // Duplicate pending application — treat as already submitted.
        submitted = true;
      } else {
        error = res.msg ?? t('forum.applyError');
      }
    } catch {
      error = t('forum.applyError');
    } finally {
      submitting = false;
    }
  }
</script>

<svelte:head><title>{t('forum.apply')} — FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">{t('forum.apply')}</h1>
      <p class="archive-summary">{t('forum.applySubtitle')}</p>
      <p class="archive-backlink"><a class="archive-link" href="/forum">&larr; {t('forum.back')}</a></p>
    </header>

    {#if !auth.isLoggedIn}
      <blockquote class="archive-empty"><p>{t('forum.blocksLogin')}</p></blockquote>
    {:else if submitted}
      <blockquote class="archive-empty"><p>{t('forum.applyPending')}</p></blockquote>
    {:else}
      <form class="archive-apply-form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
        <fieldset class="archive-fieldset">
          <legend>{t('forum.apply')}</legend>
          <dl class="archive-dl">
            <dt><label for="apply-reason">{t('forum.applyReason')}</label></dt>
            <dd><textarea id="apply-reason" class="archive-textarea" bind:value={reason} rows="6" maxlength="2000" placeholder={t('forum.applyReasonPlaceholder')}></textarea></dd>
          </dl>
          {#if error}<div class="archive-error"><strong>&#9888; {error}</strong></div>{/if}
          <div class="archive-form-actions">
            <button class="archive-btn" type="submit" disabled={submitting}>
              {submitting ? t('forum.applySubmitting') : t('forum.applySubmit')}
            </button>
          </div>
        </fieldset>
      </form>
    {/if}
  </div>
</main>
{:else}
<div class="apply-page">
  <header class="page-head">
    <a class="back" href="/forum">{t('forum.back')}</a>
    <h1>{t('forum.apply')}</h1>
    <p class="subtitle">{t('forum.applySubtitle')}</p>
  </header>

  {#if !auth.isLoggedIn}
    <div class="card empty"><p class="muted">{t('forum.blocksLogin')}</p></div>
  {:else if submitted}
    <div class="card success-card">
      <p>{t('forum.applyPending')}</p>
    </div>
  {:else}
    <form class="card apply-form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
      <label>
        {t('forum.applyReason')}
        <textarea
          bind:value={reason}
          rows="6"
          maxlength="2000"
          placeholder={t('forum.applyReasonPlaceholder')}
        ></textarea>
      </label>
      {#if error}<p class="error-card"><strong>⚠️ {error}</strong></p>{/if}
      <button class="btn btn-primary" type="submit" disabled={submitting}>
        {submitting ? t('forum.applySubmitting') : t('forum.applySubmit')}
      </button>
    </form>
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
  .apply-page { max-width: 820px; margin: 0 auto; padding: 1.5rem; }
  .page-head { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 1.25rem; }
  .back { color: var(--color-link, #2b4bd7); text-decoration: none; font-size: 0.9rem; }
  .subtitle { color: var(--color-text-muted, #888); }
  .apply-form { display: flex; flex-direction: column; gap: 0.6rem; padding: 1rem; }
  .apply-form label { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.9rem; }
  .apply-form textarea { padding: 0.45rem 0.6rem; border-radius: var(--radius-sm, 6px); border: 1px solid var(--color-border, #ddd); font: inherit; min-height: 8rem; resize: vertical; }
  .btn-primary { align-self: flex-start; }
  .empty { text-align: center; padding: 2rem; }
  .success-card { padding: 1rem; }
  .error-card { color: var(--color-error, #c0392b); }
</style>
