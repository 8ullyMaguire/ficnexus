<script lang="ts">
  import { onMount } from 'svelte';
  import { getChapterTranslations, saveChapterTranslations, getWork } from '$lib/api/social';
  import { getPref } from '$lib/prefs';
  import { auth } from '$lib/stores/auth.svelte';
  import type { ChapterTranslation } from '$lib/api/social-types';

  const uiMode = $derived(getPref('uiMode'));
  let { data } = $props();
  const workId = $derived(Number(data.workId));
  let locale = $state('es');
  let title = $state('');
  let chapters = $state<ChapterTranslation[]>([]);
  let selected = $state(0);
  let loading = $state(true);
  let saving = $state(false);
  let error = $state('');
  let message = $state('');
  let pending = $state(false);
  let submittedAt = $state('');
  let translatedBy = $state<number | undefined>();

  const current = $derived(chapters[selected]);

  async function load() {
    loading = true; error = ''; message = '';
    try {
      const [work, result] = await Promise.all([getWork(workId), getChapterTranslations(workId, locale)]);
      if (work.err === 0) title = work.work.canonical_title;
      if (result.err !== 0) throw new Error('Could not load translations');
      chapters = result.chapters ?? [];
      translatedBy = result.translated_by;
      submittedAt = result.created_at ?? '';
      pending = false;
      selected = Math.min(selected, Math.max(0, chapters.length - 1));
    } catch (e) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  }

  async function submit() {
    if (!auth.isLoggedIn || !chapters.length || saving) return;
    saving = true; error = ''; message = '';
    try {
      const result = await saveChapterTranslations(workId, locale, chapters);
      if (result.err !== 0) throw new Error('Translation submission failed');
      pending = true;
      submittedAt = new Date().toISOString();
      message = 'Submitted for curator review. The approved version will appear in the reader.';
    } catch (e) { error = e instanceof Error ? e.message : String(e); }
    finally { saving = false; }
  }

  function update(field: 'title' | 'content_html', value: string) {
    chapters = chapters.map((chapter, index) => index === selected ? { ...chapter, [field]: value } : chapter);
  }
  onMount(async () => { await auth.init(); await load(); });
</script>

<svelte:head><title>Translate {title || 'work'} — FicNexus</title></svelte:head>

<main class:archive-main={uiMode === 'archive'} class="translation-editor">
  <a href={`/work/${workId}`}>← Back to work</a>
  <h1>Chapter translation editor</h1>
  {#if title}<p class="muted">{title}</p>{/if}
  <label>Language <input bind:value={locale} maxlength="16" aria-label="Translation language" /></label>
  <button class="btn" type="button" onclick={load} disabled={loading}>Load language</button>

  {#if error}<p class="error" role="alert">{error}</p>{/if}
  {#if message}<p class="success" role="status">{message}</p>{/if}
  {#if loading}<p>Loading approved chapters…</p>
  {:else if chapters.length === 0}<p class="muted">No approved translation exists yet. A chapter source is required before editing.</p>
  {:else}
    <div class="status" class:pending>
      {#if pending}Pending curator review{:else if submittedAt}Last version loaded {submittedAt.slice(0, 10)}{:else}Draft{/if}
      {#if translatedBy} · translator #{translatedBy}{/if}
    </div>
    <nav class="chapters" aria-label="Chapters">
      {#each chapters as chapter, index}
        <button type="button" class:active={selected === index} onclick={() => selected = index}>Chapter {chapter.chapter_num}</button>
      {/each}
    </nav>
    {#if current}
      <label for="chapter-title">Chapter title</label>
      <input id="chapter-title" value={current.title} oninput={(event) => update('title', event.currentTarget.value)} />
      <label for="chapter-content">Translation (HTML)</label>
      <textarea id="chapter-content" rows="18" value={current.content_html} oninput={(event) => update('content_html', event.currentTarget.value)}></textarea>
      <button class="btn primary" type="button" onclick={submit} disabled={saving || !auth.isLoggedIn}>{saving ? 'Submitting…' : 'Submit all chapters for review'}</button>
      {#if !auth.isLoggedIn}<p class="muted">Sign in to submit a translation.</p>{/if}
    {/if}
  {/if}
</main>

<style>
  .translation-editor { max-width: 900px; margin: 2rem auto; padding: 1rem; }
  .translation-editor > * { margin-top: .75rem; }
  label { display: block; font-weight: 600; }
  input, textarea { display: block; width: 100%; box-sizing: border-box; padding: .55rem; margin-top: .3rem; font: inherit; }
  textarea { font-family: ui-monospace, monospace; }
  .chapters { display: flex; flex-wrap: wrap; gap: .35rem; border-bottom: 1px solid #ccc; padding-bottom: .5rem; }
  .chapters button { padding: .45rem .7rem; border: 1px solid #bbb; background: transparent; cursor: pointer; }
  .chapters button.active { font-weight: 700; border-color: var(--link-color, #900); }
  .btn { padding: .55rem .9rem; cursor: pointer; }
  .primary { background: var(--link-color, #900); color: white; }
  .status { padding: .65rem; background: #f1f1f1; }
  .status.pending { background: #fff3cd; }
  .error { color: #a00; } .success { color: #176b35; }
  .archive-main { font-family: Georgia, serif; }
</style>
