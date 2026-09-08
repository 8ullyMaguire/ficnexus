<script lang="ts">
  // Shows a toast when a new service worker version is available.
  let visible = $state(false);

  $effect(() => {
    function onUpdate() {
      visible = true;
    }
    window.addEventListener('sw-update-available', onUpdate);
    return () => window.removeEventListener('sw-update-available', onUpdate);
  });

  function reload() {
    window.location.reload();
  }

  function dismiss() {
    visible = false;
  }
</script>

{#if visible}
  <div class="update-toast" role="alert" aria-live="polite">
    <span class="update-text">New version available</span>
    <button class="update-btn" onclick={reload}>Reload</button>
    <button class="dismiss-btn" onclick={dismiss} aria-label="Dismiss">&times;</button>
  </div>
{/if}

<style>
  .update-toast {
    position: fixed;
    bottom: 4.5rem;
    left: 50%;
    transform: translateX(-50%);
    background: #1f6feb;
    color: white;
    padding: 0.6rem 1rem;
    border-radius: 8px;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    z-index: 9999;
    font-size: 0.9rem;
    animation: slideUp 0.3s ease-out;
  }
  .update-text { font-weight: 500; }
  .update-btn {
    background: white;
    color: #1f6feb;
    border: none;
    padding: 0.3rem 0.8rem;
    border-radius: 4px;
    font-weight: 600;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .update-btn:hover { background: #e6e6e6; }
  .dismiss-btn {
    background: none;
    border: none;
    color: white;
    font-size: 1.2rem;
    cursor: pointer;
    padding: 0 0.2rem;
    opacity: 0.7;
  }
  .dismiss-btn:hover { opacity: 1; }
  @keyframes slideUp {
    from { transform: translateX(-50%) translateY(1rem); opacity: 0; }
    to { transform: translateX(-50%) translateY(0); opacity: 1; }
  }
</style>
