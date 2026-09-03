<script lang="ts">
  import { getPref } from '$lib/prefs';
  let { payload, ficTitle }: { payload: Record<string, unknown>; ficTitle?: string } = $props();
  const uiMode = $derived(getPref('uiMode'));
  const passageText = $derived((payload.passage_text as string) ?? '');
  const passageHash = $derived((payload.passage_hash as string) ?? '');
  const chapterIndex = $derived((payload.chapter_index as number) ?? 0);
  const urlId = $derived((payload.url_id as string) ?? '');
  const href = $derived(urlId ? `/read/${encodeURIComponent(urlId)}?chapter=${chapterIndex}&hash=${encodeURIComponent(passageHash)}` : '#');
</script>

<div class="passage-card" class:archive={uiMode === 'archive'}>
  <div class="card-head">💬 Passage Context</div>
  <blockquote class="passage">{passageText}</blockquote>
  <p class="meta">Chapter {chapterIndex + 1}{ficTitle ? `, ${ficTitle}` : ''}</p>
  {#if urlId}
    <a class="read-link" href={href}>Read in Context →</a>
  {/if}
</div>

<style>
  .passage-card { border: 1px solid var(--color-border, #ddd); border-radius: 8px; padding: 0.9rem 1rem; margin-bottom: 1rem; background: var(--color-bg-alt, #f6f6f8); }
  .passage-card.archive { border-radius: 0; background: var(--archive-bg-raised, #f5f5f5); border-color: var(--archive-border, #ddd); font-family: Georgia, 'Times New Roman', serif; }
  .card-head { font-weight: 700; font-size: 0.9rem; margin-bottom: 0.5rem; }
  .passage { margin: 0; padding: 0.5rem 0.75rem; border-left: 3px solid var(--color-border, #ddd); background: var(--color-surface, #fff); white-space: pre-wrap; word-break: break-word; font-style: italic; }
  .archive .passage { background: var(--archive-bg, #fff); border-left-color: var(--archive-link, #900); }
  .meta { font-size: 0.85rem; color: var(--color-text-muted, #888); margin: 0.4rem 0; }
  .read-link { font-size: 0.9rem; color: var(--color-link, #2b4bd7); text-decoration: none; cursor: help; }
  .read-link:hover { text-decoration: underline; }
  .archive .read-link { color: var(--archive-link, #990000); }
</style>
