<script lang="ts">
  import { createTopic } from '$lib/api/forum';
  import type { MarginaliaPayload } from '$lib/api/marginalia';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let {
    open = false,
    payload,
    passageText,
    ficTitle,
    chapterIndex,
    categorySlug = 'general',
    onClose,
    onCreated,
  }: {
    open: boolean;
    payload: MarginaliaPayload;
    passageText: string;
    ficTitle: string;
    chapterIndex: number;
    categorySlug?: string;
    onClose: () => void;
    onCreated: (topicId: number, topicSlug?: string | null) => void;
  } = $props();

  let title = $derived(
    `Discussion: "${passageText.slice(0, 40).replace(/\s+/g, ' ').trim()}${passageText.length > 40 ? '…' : ''}" in ${ficTitle} - Ch ${chapterIndex + 1}`
  );
  let body = $derived(
    `> ${passageText.replace(/\n/g, '\n> ')}\n> — *Chapter ${chapterIndex + 1}, ${ficTitle}*\n\n`
  );
  let extraBody = $state('');
  let submitting = $state(false);
  let error = $state('');

  async function submit() {
    submitting = true;
    error = '';
    try {
      const res = await createTopic({
        title: title.slice(0, 200),
        category_slug: categorySlug,
        body: body + extraBody,
        payload: payload as unknown as Record<string, unknown>,
      });
      if (res.err === 0 && res.id) {
        onCreated(res.id, res.topic_slug ?? null);
        onClose();
      } else {
        error = res.msg ?? 'Failed to create topic.';
      }
    } catch {
      error = 'Network error. Please try again.';
    } finally {
      submitting = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="overlay" onclick={onClose} onkeydown={handleKeydown} role="presentation">
    <div class="modal" class:archive={uiMode === 'archive'} onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" aria-label="Discuss this passage">
      <header class="modal-head">
        <h2>💬 Discuss this passage</h2>
        <button class="close" onclick={onClose} aria-label="Close">×</button>
      </header>

      <div class="modal-body">
        <label class="field">
          <span class="label">Title</span>
          <input class="input" value={title} disabled />
        </label>

        <blockquote class="passage-preview">{passageText}</blockquote>
        <p class="meta">Chapter {chapterIndex + 1}, {ficTitle}</p>

        <label class="field">
          <span class="label">Your thoughts</span>
          <textarea class="textarea" bind:value={extraBody} rows="4" placeholder="Add your comment…"></textarea>
        </label>

        {#if error}<p class="error">{error}</p>{/if}
      </div>

      <footer class="modal-foot">
        <button class="btn" onclick={onClose} disabled={submitting}>Cancel</button>
        <button class="btn primary" onclick={submit} disabled={submitting}>
          {submitting ? 'Posting…' : 'Start discussion'}
        </button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed; inset: 0; background: rgba(0,0,0,0.45);
    display: flex; align-items: center; justify-content: center;
    z-index: 1000; padding: 1rem;
  }
  .modal {
    background: var(--color-surface, #fff); color: var(--color-text, #222);
    border: 1px solid var(--color-border, #ddd); border-radius: 8px;
    max-width: 560px; width: 100%; max-height: 90vh; overflow: auto;
  }
  .modal.archive { font-family: Georgia, 'Times New Roman', serif; border-radius: 0; }
  .modal-head { display: flex; align-items: center; justify-content: space-between; padding: 0.8rem 1rem; border-bottom: 1px solid var(--color-border, #ddd); }
  .modal-head h2 { margin: 0; font-size: 1.05rem; }
  .close { background: none; border: none; font-size: 1.5rem; cursor: pointer; line-height: 1; }
  .modal-body { padding: 1rem; display: flex; flex-direction: column; gap: 0.75rem; }
  .field { display: flex; flex-direction: column; gap: 0.25rem; }
  .label { font-size: 0.85rem; font-weight: 600; }
  .input, .textarea { padding: 0.45rem 0.6rem; border: 1px solid var(--color-border, #ddd); border-radius: 6px; font: inherit; width: 100%; box-sizing: border-box; }
  .input:disabled { background: var(--color-bg-alt, #f6f6f8); }
  .passage-preview { margin: 0; padding: 0.5rem 0.75rem; border-left: 3px solid var(--color-border, #ddd); background: var(--color-bg-alt, #f6f6f8); font-style: italic; white-space: pre-wrap; word-break: break-word; }
  .archive .passage-preview { background: var(--archive-bg-raised, #f5f5f5); border-left-color: var(--archive-border, #ddd); }
  .meta { margin: 0; font-size: 0.85rem; color: var(--color-text-muted, #888); }
  .error { color: #c0392b; font-size: 0.9rem; }
  .modal-foot { display: flex; justify-content: flex-end; gap: 0.5rem; padding: 0.8rem 1rem; border-top: 1px solid var(--color-border, #ddd); }
  .btn { padding: 0.4rem 0.9rem; border: 1px solid var(--color-border, #ddd); border-radius: 6px; background: var(--color-surface, #fff); cursor: pointer; font: inherit; }
  .archive .btn { border-radius: 0; }
  .btn.primary { background: var(--color-primary, #2b4bd7); color: #fff; border-color: var(--color-primary, #2b4bd7); }
  .archive .btn.primary { background: var(--archive-link, #990000); border-color: var(--archive-link, #990000); }
  .btn:disabled { opacity: 0.6; cursor: default; }
</style>
