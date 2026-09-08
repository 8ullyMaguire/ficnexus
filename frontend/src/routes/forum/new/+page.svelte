<script lang="ts">
  // New-topic composer (F3): title + category + opening post, POSTs to
  // /api/forum/topics, then navigates to the created topic.
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { createTopic, getForumCategories, type ForumCategory } from '$lib/api/forum';
  import { saveDraft, deleteDraft, listDrafts } from '$lib/api/drafts';
  import { handleImageEvent } from '$lib/api/uploadImage';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';

  const uiMode = $derived(getPref('uiMode'));

  let categories = $state<ForumCategory[]>([]);
  let title = $state('');
  let categorySlug = $state('');
  let body = $state('');
  let loadingCategories = $state(true);
  let submitting = $state(false);
  let error = $state('');
  let ok = $state(false);
  let draftRestored = $state(false);
  let uploadStatus = $state('');
  let autosaveTimer: ReturnType<typeof setInterval> | null = null;

  function draftRef(): string {
    return 'new:' + (categorySlug || '_');
  }

  async function persistDraft() {
    if (!auth.isLoggedIn || (!title.trim() && !body.trim())) return;
    await saveDraft('forum_topic', draftRef(), { title, body });
  }

  function scheduleAutosave() {
    autosaveTimer = setInterval(() => { persistDraft(); }, 30_000);
  }

  onDestroy(() => { if (autosaveTimer) clearInterval(autosaveTimer); });

  onMount(async () => {
    const preset = $page?.url?.searchParams?.get('category');
    try {
      const res = await getForumCategories();
      if (res.err === 0) {
        categories = res.items ?? [];
        if (preset && categories.some((c) => c.slug === preset)) {
          categorySlug = preset;
        } else if (categories.length > 0) {
          categorySlug = categories[0].slug;
        }
      } else {
        error = res.msg ?? t('forum.categoryError');
      }
    } catch {
      error = t('forum.categoryError');
    } finally {
      loadingCategories = false;
    }
    // Restore draft if one exists
    if (auth.isLoggedIn) {
      try {
        const drafts = await listDrafts('forum_topic');
        if (drafts.err === 0 && drafts.items) {
          const draft = drafts.items.find(d => d.category_id === null && d.title);
          if (draft && (draft.title || draft.body)) {
            title = draft.title ?? '';
            body = draft.body;
            draftRestored = true;
          }
        }
      } catch { /* ignore restore errors */ }
    }
    scheduleAutosave();
  });

  async function submit() {
    if (!title.trim() || !categorySlug || !body.trim()) return;
    submitting = true;
    error = '';
    ok = false;
    try {
      const res = await createTopic({
        title: title.trim().slice(0, 120),
        category_slug: categorySlug,
        body: body.trim(),
      });
      if (res.err === 0 && res.id) {
        ok = true;
        // Draft no longer needed
        deleteDraft('forum_topic', draftRef()).catch(() => {});
        try {
          // Prefer the canonical slug URL when the server returned one.
          await goto(res.topic_slug ? `/forum/board/${res.topic_slug}.${res.id}` : `/forum/${categorySlug}/${res.id}`);
        } catch {
          /* unit tests have no router — creation succeeded */
        }
      } else {
        error = res.msg ?? t('forum.actionFailed');
      }
    } catch {
      error = t('forum.actionFailed');
    } finally {
      submitting = false;
    }
  }
</script>

<svelte:head><title>{t('forum.newTopicTitle')} — FicNexus</title></svelte:head>

<div class="new-topic">
  <header class="page-head">
    <a class="back" href={`/forum/${categorySlug || ''}`}>{t('forum.backToTopics')}</a>
    <h1>{t('forum.newTopicTitle')}</h1>
    {#if uiMode !== 'archive'}
      <p class="subtitle">{t('forum.newTopicSubtitle')}</p>
    {/if}
  </header>

  {#if uiMode === 'archive'}
    <!-- Archive: AO3 fieldset-style form -->
    <fieldset class="arc-new-topic-fieldset">
      <legend>New Topic</legend>
      <dl class="arc-dl">
        <dt class="arc-dt">Title</dt>
        <dd class="arc-dd">
          <input class="arc-input" type="text" bind:value={title} maxlength="120" placeholder={t('forum.topicTitlePlaceholder')} required />
        </dd>

        <dt class="arc-dt">Category</dt>
        <dd class="arc-dd">
          <select class="arc-select" bind:value={categorySlug} disabled={loadingCategories || categories.length === 0}>
            {#if loadingCategories}
              <option value="">{t('forum.loading')}</option>
            {:else}
              {#each categories as c (c.id)}
                <option value={c.slug}>{c.title}</option>
              {/each}
            {/if}
          </select>
        </dd>

        <dt class="arc-dt">Opening Post</dt>
        <dd class="arc-dd">
          <textarea class="arc-textarea" bind:value={body} rows="8" maxlength="20000" placeholder={t('forum.topicBodyPlaceholder')} onblur={() => persistDraft()}
            onpaste={(e) => handleImageEvent(e, e.currentTarget, (m) => uploadStatus = m)}
            ondrop={(e) => handleImageEvent(e, e.currentTarget, (m) => uploadStatus = m)}
          ></textarea>
        </dd>
      </dl>

      {#if uploadStatus}<p class="muted">{uploadStatus}</p>{/if}
      <p class="muted hint">{t('forum.mentionHint')}</p>

      {#if error}<div class="error-card"><strong>{error}</strong></div>{/if}
      {#if ok}<p class="ok">{t('forum.topicPosted')}</p>{/if}

      <ArchiveButton
        disabled={submitting || !title.trim() || !categorySlug || !body.trim()}
        onclick={(e) => { e.preventDefault(); submit(); }}
      >
        {submitting ? t('forum.posting') : t('forum.postTopic')}
      </ArchiveButton>
    </fieldset>
  {:else}
    <!-- Modern: card-based form -->
    <form onsubmit={(e) => { e.preventDefault(); submit(); }}>
      <label>
        {t('forum.titleField')}
        <input type="text" bind:value={title} maxlength="120" placeholder={t('forum.topicTitlePlaceholder')} required />
      </label>

      <label>
        {t('forum.category')}
        <select bind:value={categorySlug} disabled={loadingCategories || categories.length === 0}>
          {#if loadingCategories}
            <option value="">{t('forum.loading')}</option>
          {:else}
            {#each categories as c (c.id)}
              <option value={c.slug}>{c.title}</option>
            {/each}
          {/if}
        </select>
      </label>

      <label>
        {t('forum.topicBody')}
        <textarea bind:value={body} rows="8" maxlength="20000" placeholder={t('forum.topicBodyPlaceholder')} onblur={() => persistDraft()}
          onpaste={(e) => handleImageEvent(e, e.currentTarget, (m) => uploadStatus = m)}
          ondrop={(e) => handleImageEvent(e, e.currentTarget, (m) => uploadStatus = m)}
        ></textarea>
      </label>

      {#if uploadStatus}<p class="muted">{uploadStatus}</p>{/if}
      <p class="muted hint">{t('forum.mentionHint')}</p>

      {#if error}<div class="error-card"><strong>⚠️ {error}</strong></div>{/if}
      {#if ok}<p class="ok">{t('forum.topicPosted')}</p>{/if}

      <button class="btn btn-primary" type="submit" disabled={submitting || !title.trim() || !categorySlug || !body.trim()}>
        {submitting ? t('forum.posting') : t('forum.postTopic')}
      </button>
    </form>
  {/if}
</div>

<style>
  .new-topic { max-width: 620px; margin: 0 auto; padding: 1.5rem; }
  .page-head { margin-bottom: 1.25rem; display: flex; flex-direction: column; gap: 0.4rem; }
  .back { color: var(--color-link, #2b4bd7); text-decoration: none; font-size: 0.9rem; }
  .subtitle { color: var(--color-text-muted, #888); }
  form { display: flex; flex-direction: column; gap: 1rem; }
  label { display: flex; flex-direction: column; gap: 0.4rem; font-weight: 600; }
  input, select, textarea {
    padding: 0.6rem 0.75rem; border: 1px solid var(--color-border, #ddd); border-radius: 8px;
    font: inherit; font-weight: 400; width: 100%; box-sizing: border-box;
  }
  .hint { font-size: 0.8rem; margin: 0; }
  .btn-primary { align-self: flex-start; }
  .ok { color: #1e8e3e; font-weight: 600; }
  .muted { color: var(--color-text-muted, #888); }

  /* ── Archive new-topic form ────────────────────────────────────── */
  .arc-new-topic-fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.75em;
  }
  .arc-new-topic-fieldset legend {
    font-family: Georgia, 'Times New Roman', serif;
    font-weight: 700;
    font-size: 1em;
    color: var(--archive-text, #2a2a2a);
    padding: 0 0.4em;
  }
  .arc-dl {
    margin: 0.5em 0;
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.4em 0.7em;
    align-items: start;
  }
  .arc-dt {
    font-size: 0.85em;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--archive-muted, #666666);
    padding-top: 0.4em;
    white-space: nowrap;
  }
  .arc-dd {
    margin: 0;
    min-width: 0;
  }
  .arc-input, .arc-select, .arc-textarea {
    padding: 0.4em 0.6em;
    font-size: 0.95em;
    font-family: inherit;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
    width: 100%;
    box-sizing: border-box;
    border-radius: 0;
  }
  .arc-textarea { resize: vertical; }
</style>