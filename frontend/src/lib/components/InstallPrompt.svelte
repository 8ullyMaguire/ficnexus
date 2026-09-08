<script lang="ts">
  // Shows an install prompt banner when the browser supports PWA installation.
  let deferredPrompt = $state<any>(null);
  let visible = $state(false);
  let dismissed = $state(false);

  $effect(() => {
    function onBeforeInstall(e: Event) {
      e.preventDefault();
      deferredPrompt = e;
      visible = true;
    }
    window.addEventListener('beforeinstallprompt', onBeforeInstall);
    return () => window.removeEventListener('beforeinstallprompt', onBeforeInstall);
  });

  async function install() {
    if (!deferredPrompt) return;
    deferredPrompt.prompt();
    const { outcome } = await deferredPrompt.userChoice;
    if (outcome === 'accepted') {
      visible = false;
    }
    deferredPrompt = null;
  }

  function dismiss() {
    visible = false;
    dismissed = true;
  }
</script>

{#if visible && !dismissed}
  <div class="install-banner" role="complementary" aria-label="Install FicNexus">
    <div class="install-content">
      <span class="install-icon">📱</span>
      <div class="install-text">
        <strong>Install FicNexus</strong>
        <span>Add to your home screen for quick access and offline reading.</span>
      </div>
    </div>
    <div class="install-actions">
      <button class="install-btn" onclick={install}>Install</button>
      <button class="dismiss-btn" onclick={dismiss} aria-label="Dismiss">&times;</button>
    </div>
  </div>
{/if}

<style>
  .install-banner {
    position: fixed;
    bottom: 0.75rem;
    left: 50%;
    transform: translateX(-50%);
    background: #161b22;
    border: 1px solid #30363d;
    padding: 0.75rem 1rem;
    border-radius: 10px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
    z-index: 9998;
    max-width: 400px;
    width: calc(100% - 2rem);
    animation: slideUp 0.3s ease-out;
  }
  .install-content {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }
  .install-icon { font-size: 1.5rem; }
  .install-text {
    display: flex;
    flex-direction: column;
    font-size: 0.85rem;
    color: #e1e4e8;
  }
  .install-text span { color: #8b949e; font-size: 0.8rem; }
  .install-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .install-btn {
    background: #238636;
    color: white;
    border: none;
    padding: 0.4rem 0.8rem;
    border-radius: 6px;
    font-weight: 600;
    cursor: pointer;
    font-size: 0.85rem;
    white-space: nowrap;
  }
  .install-btn:hover { background: #2ea043; }
  .dismiss-btn {
    background: none;
    border: none;
    color: #8b949e;
    font-size: 1.2rem;
    cursor: pointer;
    padding: 0 0.2rem;
  }
  .dismiss-btn:hover { color: #e1e4e8; }
  @keyframes slideUp {
    from { transform: translateX(-50%) translateY(1rem); opacity: 0; }
    to { transform: translateX(-50%) translateY(0); opacity: 1; }
  }
</style>
