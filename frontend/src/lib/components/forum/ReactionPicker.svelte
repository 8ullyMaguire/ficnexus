<script lang="ts">
  // Anchored emoji reaction picker (docs/REACTION-EMOJI-DESIGN.md).
  // Search bar at top, frequently-used row, then category sections.
  // Stays anchored below its trigger until dismissed; positive-only set
  // mirrors the backend ALLOWED_REACTIONS. Archive-mode variant restyles
  // the panel to match the legacy theme.
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';
  import { FORUM_REACTIONS } from '$lib/api/forum';

  let {
    myReactions = [],
    onPick,
    onClose,
  }: {
    myReactions?: string[];
    onPick: (emoji: string) => void;
    onClose: () => void;
  } = $props();

  const uiMode = $derived(getPref('uiMode'));

  // Frequently used — first row shown above the categories.
  const COMMON = ['👍', '❤️', '🔥', '😂'];

  const CATEGORIES: { labelKey: string; emojis: string[] }[] = [
    { labelKey: 'forum.reactionFaces', emojis: ['😂', '🤔', '😢', '😮'] },
    { labelKey: 'forum.reactionGestures', emojis: ['👍', '👏', '❤️', '🔥'] },
  ];

  let query = $state('');

  function matches(emoji: string): boolean {
    if (!query.trim()) return true;
    const q = query.trim().toLowerCase();
    return emoji.includes(q) || emojiName(emoji).includes(q);
  }

  // Tiny keyword index so the search bar is actually useful.
  const NAMES: Record<string, string> = {
    '👍': 'like agree thumbs up',
    '❤️': 'love heart support',
    '😂': 'funny laugh lol haha',
    '🔥': 'fire hot exciting lit',
    '👏': 'clap applause well said bravo',
    '🤔': 'thinking hmm interesting',
    '😢': 'sad cry sympathetic tears',
    '😮': 'wow surprised shocked',
  };

  function emojiName(emoji: string): string {
    return NAMES[emoji] ?? '';
  }

  function pick(emoji: string) {
    onPick(emoji);
    onClose();
  }
</script>

<div
  class="picker"
  class:arc={uiMode === 'archive'}
  role="dialog"
  aria-label={t('forum.react')}
>
  <div class="picker-search">
    <input
      type="search"
      placeholder={t('forum.reactionSearch')}
      bind:value={query}
      aria-label={t('forum.reactionSearch')}
    />
    <button class="picker-close" type="button" onclick={onClose} aria-label="✕">✕</button>
  </div>

  {#if !query.trim()}
    <div class="picker-section">
      <div class="picker-label">{t('forum.reactionCommon')}</div>
      <div class="picker-row">
        {#each COMMON as emoji (emoji)}
          <button
            class="picker-emoji"
            class:mine={myReactions.includes(emoji)}
            type="button"
            onclick={() => pick(emoji)}
          >{emoji}</button>
        {/each}
      </div>
    </div>
  {/if}

  {#each CATEGORIES as cat (cat.labelKey)}
    {@const visible = cat.emojis.filter(matches)}
    {#if visible.length > 0}
      <div class="picker-section">
        <div class="picker-label">{t(cat.labelKey)}</div>
        <div class="picker-row">
          {#each visible as emoji (emoji)}
            <button
              class="picker-emoji"
              class:mine={myReactions.includes(emoji)}
              type="button"
              onclick={() => pick(emoji)}
            >{emoji}</button>
          {/each}
        </div>
      </div>
    {/if}
  {/each}

  {#if query.trim() && [...COMMON, ...CATEGORIES.flatMap((c) => c.emojis)].filter(matches).length === 0}
    <div class="picker-empty">{t('forum.reactionNone')}</div>
  {/if}
</div>

<style>
  .picker {
    position: absolute;
    top: calc(100% + 0.35rem);
    left: 0;
    z-index: 30;
    min-width: 15rem;
    background: var(--color-bg, #fff);
    border: 1px solid var(--color-border, #d8d8e2);
    border-radius: 8px;
    box-shadow: 0 6px 20px rgba(20, 20, 40, 0.16);
    padding: 0.5rem;
  }
  .picker-search {
    display: flex;
    gap: 0.25rem;
    margin-bottom: 0.4rem;
  }
  .picker-search input {
    flex: 1;
    font-size: 0.85rem;
    padding: 0.3rem 0.5rem;
    border: 1px solid var(--color-border, #d8d8e2);
    border-radius: 6px;
    background: var(--color-bg, #fff);
    color: inherit;
  }
  .picker-close {
    border: none;
    background: transparent;
    color: var(--color-text-muted, #666);
    cursor: pointer;
    padding: 0 0.35rem;
  }
  .picker-section + .picker-section {
    margin-top: 0.45rem;
  }
  .picker-label {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-text-muted, #666);
    margin-bottom: 0.2rem;
  }
  .picker-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.15rem;
  }
  .picker-emoji {
    font-size: 1.15rem;
    line-height: 1;
    padding: 0.28rem 0.34rem;
    border-radius: 6px;
    border: 1px solid transparent;
    background: transparent;
    cursor: pointer;
  }
  .picker-emoji:hover {
    background: var(--color-bg-alt, #f0f0f4);
  }
  .picker-emoji.mine {
    border-color: #4a8af4;
    background: #e8f0fe;
  }
  .picker-empty {
    font-size: 0.82rem;
    color: var(--color-text-muted, #666);
    padding: 0.3rem 0.15rem;
  }

  /* ── Archive mode variant ─────────────────────────────────────────── */
  .picker.arc {
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    box-shadow: 0 5px 16px rgba(60, 60, 60, 0.22);
    border-radius: 2px;
  }
  .picker.arc .picker-search input {
    background: var(--archive-bg, #ffffff);
    border-color: var(--archive-border, #dddddd);
    color: var(--archive-text, #2a2a2a);
    border-radius: 2px;
  }
  .picker.arc .picker-label,
  .picker.arc .picker-empty {
    color: var(--archive-muted, #666666);
  }
  .picker.arc .picker-emoji:hover {
    background: var(--archive-bg, #ffffff);
  }
  .picker.arc .picker-emoji.mine {
    border-color: var(--archive-muted, #666666);
    background: var(--archive-bg, #ffffff);
  }
</style>
