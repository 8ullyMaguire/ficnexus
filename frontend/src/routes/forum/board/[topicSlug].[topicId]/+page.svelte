<script lang="ts">
  // /forum/board/{topicSlug}.{topicId} — canonical topic URL. Resolves the
  // topic via the by-slug API (fresh links), then falls back to the numeric
  // id when the slug is stale (topic title edited elsewhere, legacy link
  // carrying an old slug, or slug unknown to the server yet).
  import TopicThread from '$lib/components/forum/TopicThread.svelte';
import TranslatePageButton from '$lib/components/TranslatePageButton.svelte';
  import { getForumTopicBySlug, getForumTopic, markTopicRead, getForumTopic as getTopicDetail } from '$lib/api/forum';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';
  import { auth } from '$lib/stores/auth.svelte';

  const uiMode = $derived(getPref('uiMode'));

  let { data } = $props();
  let topicSlug = $derived(data.topicSlug ?? '');
  let topicId = $derived(Number(data.topicId ?? ''));
  let resolvedTopicId = $state<number | null>(null);
  let error = $state('');
  let resolved = $state(false);
  let lastKey = $state('');
  let backCategorySlug = $state('');

  async function resolve() {
    // Prefer the slug; the numeric id is the authoritative fallback.
    if (topicSlug) {
      try {
        const res = await getForumTopicBySlug(topicSlug);
        if (res.err === 0) {
          resolvedTopicId = res.id;
          backCategorySlug = res.category_slug ?? '';
          resolved = true;
          // F4: debounced fire-and-forget mark-read from the board route as well
          // (TopicThread also does this; double-call is idempotent).
          if (auth.isLoggedIn) {
            const lastId = (res as unknown as { items?: { id: number }[] }).items?.at(-1)?.id ?? null;
            setTimeout(() => markTopicRead(res.id, lastId).catch(() => {}), 300);
          }
          return;
        }
        error = res.msg ?? '';
      } catch {
        // Fall through to numeric id below.
      }
    }
    if (Number.isFinite(topicId) && topicId > 0) {
      try {
        const res = await getForumTopic(topicId);
        if (res.err === 0) {
          resolvedTopicId = res.id;
          backCategorySlug = res.category_slug ?? '';
          resolved = true;
          if (auth.isLoggedIn) {
            const lastId = (res as unknown as { items?: { id: number }[] }).items?.at(-1)?.id ?? null;
            setTimeout(() => markTopicRead(res.id, lastId).catch(() => {}), 300);
          }
          return;
        }
        error = res.msg ?? '';
      } catch {
        error = t('forum.topicLoadError');
      }
    }
    if (!error) error = t('forum.topicLoadError');
  }

  $effect(() => {
    const key = `${topicSlug}|${topicId}`;
    if (key === lastKey && (resolved || error)) return;
    lastKey = key;
    resolved = false;
    resolvedTopicId = null;
    error = '';
    void resolve();
  });
</script>

{#if uiMode === 'archive'}
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">{t('forum.title')}</h1>
    </header>
    {#if resolved && resolvedTopicId !== null}
      <TopicThread topicId={resolvedTopicId} backHref={backCategorySlug ? `/forum/${backCategorySlug}` : '/forum'} />
    {:else if error}
      <div class="archive-error"><strong>&#9888; {error}</strong></div>
      <p><a class="archive-link" href="/forum">&larr; {t('forum.back')}</a></p>
    {:else}
      <blockquote class="archive-empty"><p>{t('forum.loading')}</p></blockquote>
    {/if}
    <TranslatePageButton />
  </div>
</main>
{:else}
{#if resolved && resolvedTopicId !== null}
  <TopicThread topicId={resolvedTopicId} backHref={backCategorySlug ? `/forum/${backCategorySlug}` : '/forum'} />
{:else if error}
  <div class="board-error">
    <div class="error-card"><strong>{error}</strong></div>
    <a class="back" href="/forum">{t('forum.back')}</a>
  </div>
{:else}
  <p class="muted board-loading"><span class="spinner"></span> {t('forum.loading')}</p>
{/if}
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
  .archive-link { color: var(--archive-link, #990000); }
  .board-error { max-width: 820px; margin: 0 auto; padding: 1.5rem; display: flex; flex-direction: column; gap: 1rem; }
  .board-loading { max-width: 820px; margin: 0 auto; padding: 1.5rem; }
  .back { color: var(--color-link, #2b4bd7); text-decoration: none; font-size: 0.9rem; }
</style>
