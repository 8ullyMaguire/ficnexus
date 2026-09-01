<script lang="ts">
  // Small banner shown when the browser is offline (or the SW reports
  // that the network is unreachable). Uses Svelte 5 runes.
  let online = $state(typeof navigator === 'undefined' ? true : navigator.onLine);

  function update() {
    online = navigator.onLine;
  }

  $effect(() => {
    window.addEventListener('online', update);
    window.addEventListener('offline', update);
    return () => {
      window.removeEventListener('online', update);
      window.removeEventListener('offline', update);
    };
  });
</script>

{#if !online}
  <div class="offline-banner" role="status" aria-live="polite">
    <span class="dot" aria-hidden="true"></span>
    You're offline — showing cached content.
  </div>
{/if}

<style>
  .offline-banner {
    position: fixed;
    bottom: 0.75rem;
    left: 50%;
    transform: translateX(-50%);
    z-index: 1000;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: var(--color-surface-2, #1f2430);
    color: var(--color-text, #e8eaf0);
    border: 1px solid var(--color-border, #2a3040);
    border-radius: var(--radius-sm, 8px);
    padding: 0.5rem 1rem;
    font-size: 0.85rem;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.45);
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-warning, #f59e0b);
    flex-shrink: 0;
  }
</style>
