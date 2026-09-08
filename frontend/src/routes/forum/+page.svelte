<script lang="ts">
  import { onMount } from 'svelte';
  import { getForumCategories, createCategory, type ForumCategory } from '$lib/api/forum';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';
  import ForumWidgets from '$lib/components/forum/ForumWidgets.svelte';

  const uiMode = $derived(getPref('uiMode'));

  let items = $state<ForumCategory[]>([]);
  let loading = $state(true);
  let error = $state('');
  let isAdmin = $state(false);

  // New-category form (admin only)
  let showForm = $state(false);
  let slug = $state('');
  let title = $state('');
  let description = $state('');
  let creating = $state(false);
  let createError = $state('');

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await getForumCategories();
      if (res.err === 0) {
        items = res.items ?? [];
      } else {
        error = res.msg ?? t('forum.categoryError');
      }
    } catch {
      error = t('forum.categoryError');
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    await auth.init();
    isAdmin = auth.level >= 100;
    await load();
  });

  async function handleCreate() {
    createError = '';
    if (!slug.trim() || !title.trim()) {
      createError = t('forum.categoryError');
      return;
    }
    creating = true;
    try {
      const res = await createCategory({
        slug: slug.trim().toLowerCase(),
        title: title.trim(),
        description: description.trim(),
      });
      if (res.err === 0) {
        slug = '';
        title = '';
        description = '';
        showForm = false;
        await load();
      } else {
        createError = res.msg ?? t('forum.categoryError');
      }
    } catch {
      createError = t('forum.categoryError');
    } finally {
      creating = false;
    }
  }

  function topicsLabel(count: number): string {
    if (count === 1) return t('forum.replySingular', { count });
    return t('forum.repliesPlural', { count });
  }
</script>

<svelte:head><title>Forum — FicNexus</title></svelte:head>

<div class="forum-page">
  <div class="forum-layout">
  <header class="page-head">
    <h1>{uiMode === 'archive' ? 'Forum' : t('forum.title')}</h1>
    {#if uiMode !== 'archive'}
      <p class="subtitle">{t('forum.subtitle')}</p>
    {/if}
    {#if isAdmin}
      {#if uiMode === 'archive'}
        <ArchiveButton type="button" onclick={() => (showForm = !showForm)}>
          {showForm ? t('forum.close') : 'New Category'}
        </ArchiveButton>
      {:else}
        <button class="btn btn-primary" type="button" onclick={() => (showForm = !showForm)}>
          {showForm ? '✕' : t('forum.newCategory')}
        </button>
      {/if}
    {/if}
  </header>

  {#if showForm}
    <form class="card new-category" class:archive-form={uiMode === 'archive'} onsubmit={(e) => { e.preventDefault(); handleCreate(); }}>
      <h2>{t('forum.newCategoryTitle')}</h2>
      <label>
        {t('forum.slug')}
        <input type="text" bind:value={slug} placeholder={t('forum.slugPlaceholder')} />
      </label>
      <label>
        {t('forum.titleField')}
        <input type="text" bind:value={title} placeholder={t('forum.titlePlaceholder')} />
      </label>
      <label>
        {t('forum.descriptionField')}
        <textarea bind:value={description} placeholder={t('forum.descriptionPlaceholder')}></textarea>
      </label>
      {#if createError}<div class="error-card"><strong>{createError}</strong></div>{/if}
      {#if uiMode === 'archive'}
        <ArchiveButton type="submit" disabled={creating}>{creating ? t('forum.creating') : t('forum.create')}</ArchiveButton>
      {:else}
        <button class="btn btn-primary" type="submit" disabled={creating}>
          {creating ? t('forum.creating') : t('forum.create')}
        </button>
      {/if}
    </form>
  {/if}

  {#if loading}
    <p class="muted"><span class="spinner"></span> {t('forum.loading')}</p>
  {:else if error}
    <div class="error-card"><strong>{error}</strong></div>
  {:else if items.length === 0}
    <p class="empty">{t('forum.empty')}</p>
    <p class="empty-hint">{t('forum.emptyHint')}</p>
  {:else if uiMode === 'archive'}
    <!-- Archive: bordered category table -->
    <table class="archive-cat-table">
      <thead>
        <tr>
          <th class="col-name">Category</th>
          <th class="col-stat">Topics</th>
          <th class="col-stat">Last Post</th>
        </tr>
      </thead>
      <tbody>
        {#each items as c (c.id)}
          <tr>
            <td class="col-name">
              <a class="arc-link" href={`/forum/${c.slug}`}>{c.title}</a>
              {#if c.description}<span class="arc-desc">{c.description}</span>{/if}
            </td>
            <td class="col-stat">{c.topic_count}</td>
            <td class="col-stat">{c.last_activity_at ? c.last_activity_at.slice(0, 10) : '—'}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {:else}
    <!-- Modern: card-based category list -->
    <ul class="category-list">
      {#each items as c (c.id)}
        <li class="card category-card">
          <a class="title" href={`/forum/${c.slug}`}>{c.title}</a>
          {#if c.is_mod_only}<span class="chip">{t('forum.pinned')}</span>{/if}
          {#if c.description}<p class="desc">{c.description}</p>{/if}
          <span class="meta">
            {topicsLabel(c.topic_count)} · {t('forum.lastActivity')}: {c.last_activity_at ? c.last_activity_at.slice(0, 10) : t('forum.noActivity')}
          </span>
        </li>
      {/each}
    </ul>
  {/if}
</div>
  <ForumWidgets />
</div>

<style>
  .forum-page { max-width: 1100px; margin: 0 auto; padding: 1.5rem; }
  .forum-layout { display: flex; gap: 1.5rem; flex: 1; }
  .forum-layout > :first-child { flex: 1; min-width: 0; }
  .page-head { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 1.25rem; }
  .subtitle { color: var(--color-text-muted, #888); }
  .btn-primary { align-self: flex-start; }
  .category-list { list-style: none; display: flex; flex-direction: column; gap: 0.75rem; padding: 0; }
  .category-card { padding: 1rem; display: flex; flex-direction: column; gap: 0.4rem; }
  .title { font-size: 1.1rem; font-weight: 600; color: var(--color-link, #2b4bd7); text-decoration: none; }
  .desc { color: var(--color-text-muted, #888); font-size: 0.92rem; margin: 0; }
  .meta { color: var(--color-text-muted, #888); font-size: 0.85rem; }
  .chip { align-self: flex-start; background: var(--color-chip-bg, #eef); padding: 0.15rem 0.5rem; border-radius: 999px; font-size: 0.75rem; }
  .new-category { display: flex; flex-direction: column; gap: 0.6rem; padding: 1rem; margin-bottom: 1rem; }
  .new-category label { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.9rem; }
  .new-category input, .new-category textarea { padding: 0.45rem 0.6rem; border-radius: var(--radius-sm, 6px); border: 1px solid var(--color-border, #ddd); font: inherit; }

  /* ── Archive-mode new-category form (AO3-style bordered fieldset) ── */
  .archive-form.new-category {
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg, #ffffff);
    border-radius: 0;
    max-width: 46em;
  }
  .archive-form label {
    color: var(--archive-muted, #666666);
    font-size: 0.82em;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }
  .archive-form input, .archive-form textarea {
    margin-top: 0.15em;
  }
  .archive-form .btn-primary {
    width: auto;
  }
  .empty { text-align: center; color: var(--color-text-muted, #888); padding: 3rem 0 0; }
  .empty-hint { text-align: center; color: var(--color-text-muted, #888); }

  /* ── Archive table ─────────────────────────────────────────────── */
  .archive-cat-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.92em;
    border: 1px solid var(--archive-border, #dddddd);
  }
  .archive-cat-table thead th {
    text-align: left;
    padding: 0.45em 0.7em;
    border-bottom: 2px solid var(--archive-border, #dddddd);
    font-size: 0.82em;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--archive-muted, #666666);
  }
  .archive-cat-table tbody tr {
    border-bottom: 1px solid var(--archive-border, #dddddd);
  }
  .archive-cat-table tbody tr:last-child {
    border-bottom: none;
  }
  .archive-cat-table td {
    padding: 0.5em 0.7em;
    vertical-align: top;
  }
  .col-stat {
    text-align: right;
    white-space: nowrap;
  }
  .col-name { width: 100%; }
  .arc-link {
    font-weight: 700;
    color: var(--archive-link, #990000);
    text-decoration: none;
  }
  .arc-link:hover { color: #999; }
  .arc-desc {
    display: block;
    font-size: 0.85em;
    color: var(--archive-muted, #666666);
    margin-top: 0.15em;
  }
</style>
