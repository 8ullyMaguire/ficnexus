<script lang="ts">
  import { onMount } from 'svelte';

  interface Props {
    newLevel: number;
    newRank: number;
    unlockedFeatures: string[];
    ondismiss: () => void;
  }

  let { newLevel, newRank, unlockedFeatures, ondismiss }: Props = $props();
  let visible = $state(true);

  onMount(() => {
    const timer = setTimeout(() => {
      visible = false;
      ondismiss();
    }, 10_000);
    return () => clearTimeout(timer);
  });

  function close() {
    visible = false;
    ondismiss();
  }
</script>

{#if visible}
  <div class="toast-overlay" role="alert">
    <div class="toast card">
      <div class="toast-header">
        <span class="toast-icon">🎉</span>
        <div>
          <h3 class="toast-title">Level Up!</h3>
          <p class="toast-subtitle">
            Rank {newRank} · Level {newLevel}
          </p>
        </div>
      </div>
      {#if unlockedFeatures.length > 0}
        <p class="toast-features muted">
          New features unlocked: {unlockedFeatures.join(', ')}
        </p>
      {/if}
      <div class="toast-actions">
        <a href="/features?tab=available" class="btn btn-sm" onclick={close}>
          View Available
        </a>
        <button class="btn btn-secondary btn-sm" onclick={close}>
          Dismiss
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .toast-overlay {
    position: fixed;
    bottom: 1.5rem;
    right: 1.5rem;
    z-index: 9999;
    max-width: 340px;
    width: calc(100vw - 3rem);
    animation: toast-in 0.35s ease;
  }
  .toast {
    padding: 1rem 1.1rem;
    border-color: var(--color-success);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4), 0 0 0 1px var(--color-success);
  }
  .toast-header {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    margin-bottom: 0.4rem;
  }
  .toast-icon {
    font-size: 1.5rem;
    flex-shrink: 0;
  }
  .toast-title {
    margin: 0;
    font-size: 1.05rem;
    color: var(--color-success);
  }
  .toast-subtitle {
    margin: 0.1rem 0 0;
    font-size: 0.85rem;
  }
  .toast-features {
    margin: 0.3rem 0 0.6rem;
    font-size: 0.8rem;
    line-height: 1.4;
  }
  .toast-actions {
    display: flex;
    gap: 0.4rem;
  }
  .btn-sm {
    padding: 0.3rem 0.65rem;
    font-size: 0.8rem;
  }
  @keyframes toast-in {
    from {
      opacity: 0;
      transform: translateY(1rem) scale(0.95);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
</style>
