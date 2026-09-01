<!--
  ProposalDiff — side-by-side diff for curator proposals (M7).
  Shows original text vs proposed text, highlighting changed words.
  Only renders when payload has an `original` field alongside `text`.
-->
<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';

  let {
    original,
    proposed,
    field,
  }: {
    original: string;
    proposed: string;
    field?: string;
  } = $props();

  // Word-level diff: split on word boundaries, mark changed words.
  // Simple algorithm: tokenize, compare positions, mark mismatches.
  function wordDiff(a: string, b: string): Array<{ text: string; changed: boolean }> {
    if (a === b) return [{ text: a, changed: false }];
    const ta = a.split(/(\s+)/).filter(s => s.trim());
    const tb = b.split(/(\s+)/).filter(s => s.trim());
    const result: Array<{ text: string; changed: boolean }> = [];
    const maxLen = Math.max(ta.length, tb.length);
    for (let i = 0; i < maxLen; i++) {
      const wa = ta[i] ?? '';
      const wb = tb[i] ?? '';
      if (wa === wb) {
        result.push({ text: wa, changed: false });
      } else {
        if (wa) result.push({ text: wa, changed: true });
        if (wb) result.push({ text: wb, changed: true });
      }
    }
    return result.length > 0 ? result : [{ text: a, changed: false }];
  }

  // Char-level inline diff for a single cell (lightweight, no deps)
  function charDiffClass(orig: string, newVal: string, idx: number): boolean {
    // Mark chars that differ between original and proposed at same position
    return orig[idx] !== newVal[idx];
  }

  let expanded = $state(false);
</script>

{#if original && proposed && original !== proposed}
  <div class="diff-wrap">
    {#if !expanded}
      <button class="diff-preview-btn" onclick={() => expanded = true}>
        {t('curator.viewDiff')}
      </button>
    {/if}
    {#if expanded}
      <div class="diff-hint muted">{t('curator.diffHint')}</div>
      <div class="diff-grid">
        <div class="diff-col">
          <div class="diff-header">{t('curator.diffOriginal')}</div>
          <div class="diff-cell diff-cell-orig">
            {#each original.split('\n') as line, i}
              <div class="diff-line">
                {#each wordDiff(line, proposed.split('\n')[i] ?? '') as w}
                  <span class={w.changed ? 'diff-word changed' : 'diff-word'}>
                    {w.text}
                  </span>
                {/each}
              </div>
            {/each}
          </div>
        </div>
        <div class="diff-col">
          <div class="diff-header">{t('curator.diffProposed')}</div>
          <div class="diff-cell diff-cell-new">
            {#each proposed.split('\n') as line, i}
              <div class="diff-line">
                {#each wordDiff(original.split('\n')[i] ?? '', line) as w}
                  <span class={w.changed ? 'diff-word changed' : 'diff-word'}>
                    {w.text}
                  </span>
                {/each}
              </div>
            {/each}
          </div>
        </div>
      </div>
      <button class="diff-preview-btn" onclick={() => expanded = false}>
        {t('curator.hideDiff')}
      </button>
    {/if}
  </div>
{:else if original && proposed}
  <div class="diff-noop muted">{t('curator.diffNoChange')}</div>
{/if}

<style>
  .diff-wrap { margin: .5em 0; }
  .diff-preview-btn {
    font-size: .8em; color: #666; border: 1px solid #bbb; border-radius: .25em;
    padding: .15em .5em; cursor: pointer; background: #eee; font: inherit;
  }
  .diff-preview-btn:hover { color: #900; border-color: #999; background: #ddd; }
  .diff-hint { font-size: .8em; color: #666; margin: .3em 0; }
  .diff-grid { display: flex; gap: .8rem; flex-wrap: wrap; }
  .diff-col { flex: 1; min-width: 200px; }
  .diff-header {
    font-size: .8em; font-weight: bold; color: #666;
    padding: .2em .4em; border-bottom: 1px solid #ddd;
  }
  .diff-cell {
    font-family: 'Courier New', monospace; font-size: .85em;
    line-height: 1.5; white-space: pre-wrap; word-break: break-word;
    border: 1px solid #ddd; border-radius: .25em; padding: .5em;
    background: #fafafa; max-height: 200px; overflow-y: auto;
  }
  .diff-cell-orig { background: #fdf6f6; }
  .diff-cell-new { background: #f0faf0; }
  .diff-line { display: block; }
  .diff-word { display: inline; }
  .diff-word.changed { color: #c0392b; background: #ffd6d6; }
  .diff-noop { font-size: .8em; color: #666; margin: .3em 0; }
</style>
