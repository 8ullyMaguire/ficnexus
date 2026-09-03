<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { getReactions, toggleReaction, ALLOWED_REACTIONS } from '$lib/api/social';
  import { getPref } from '$lib/prefs';
  const uiMode = $derived(getPref('uiMode'));

  let { targetType = 'work', targetId = 0 }: { targetType: 'work' | 'comment' | 'chapter'; targetId: number } = $props();

  let reactions = $state<Record<string, Array<{ user_id: number; username: string }>>>({});
  let myReactions = $state<string[]>([]);
  let loading = $state(false);

  async function load() {
    loading = true;
    try {
      const res = await getReactions(targetType, targetId);
      if (res.err === 0 && res.reactions) {
        reactions = res.reactions.reactions || {};
        myReactions = res.reactions.my_reactions || [];
      }
    } catch { /* offline / not signed in — bar simply renders empty */ }
    loading = false;
  }

  async function toggle(emoji: string) {
    if (!auth.user?.id) {
      // Redirect to login or show auth prompt
      return;
    }
    try {
      const res = await toggleReaction(targetType, targetId, emoji);
      if (res.err === 0 && res.reactions) {
        reactions = res.reactions.reactions || {};
        myReactions = res.reactions.my_reactions || [];
      }
    } catch { /* offline — keep current state */ }
  }

  // Build sorted list of emojis with counts
  let emojiList = $derived(ALLOWED_REACTIONS.map((emoji: string) => ({
    emoji,
    count: reactions[emoji]?.length || 0,
    active: myReactions.includes(emoji),
  })).filter((e: { emoji: string; count: number; active: boolean }) => e.count > 0 || e.active));

  onMount(load);
</script>

{#if !loading}
  <div class="reaction-bar" class:archive={uiMode === 'archive'}>
    {#each emojiList as { emoji, count, active }}
      <button
        class="reaction-btn"
        class:active={active}
        class:inactive={!active}
        on:click={() => toggle(emoji)}
        title={emoji}
      >
        <span class="emoji">{emoji}</span>
        <span class="count">{count}</span>
      </button>
    {/each}
    {#if emojiList.length === 0}
      <span class="no-reactions">Be the first to react!</span>
    {/if}
  </div>
{/if}

<style>
  .reaction-bar {
    display: flex;
    gap: 0.4rem;
    align-items: center;
    margin: 0.4rem 0;
    flex-wrap: wrap;
  }
  .reaction-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.2rem;
    background: transparent;
    border: 1px solid var(--color-border, #ddd);
    border-radius: var(--radius-sm, 4px);
    padding: 0.2rem 0.5rem;
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .reaction-btn:hover {
    background: var(--color-bg-hover, #f5f5f5);
  }
  .reaction-btn.active {
    background: var(--color-accent, #e0e0e0);
    border-color: var(--color-accent-border, #aaa);
  }
  .reaction-btn.inactive:hover .emoji {
    filter: saturate(1.5);
  }
  .count {
    font-size: 0.75rem;
    color: var(--color-text-muted, #888);
  }
  .no-reactions {
    font-size: 0.75rem;
    color: var(--color-text-muted, #888);
  }

  /* Archive mode: AO3-style flat chrome */
  .reaction-bar.archive .reaction-btn {
    border-radius: 0;
    border-color: var(--archive-border, #dddddd);
    background: var(--archive-bg, #ffffff);
    font-family: Georgia, 'Times New Roman', serif;
    padding: 0.15rem 0.45rem;
  }
  .reaction-bar.archive .reaction-btn:hover {
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .reaction-bar.archive .reaction-btn.active {
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
    color: var(--archive-bg, #ffffff);
  }
  .reaction-bar.archive .count,
  .reaction-bar.archive .no-reactions {
    color: var(--archive-muted, #666666);
    font-size: 0.8rem;
  }
</style>
