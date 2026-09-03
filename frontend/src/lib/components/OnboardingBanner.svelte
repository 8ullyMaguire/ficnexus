<script lang="ts">
  import { onMount } from 'svelte';

  const STORAGE_KEY = 'fichub_onboarding_dismissed';
  let visible = $state(false);

  onMount(() => {
    if (!localStorage.getItem(STORAGE_KEY)) {
      visible = true;
    }
  });

  function dismiss() {
    visible = false;
    localStorage.setItem(STORAGE_KEY, '1');
  }
</script>

{#if visible}
  <div class="onboarding-banner card">
    <div class="onboarding-content">
      <span class="onboarding-icon">👋</span>
      <div>
        <p class="onboarding-text">
          <strong>Welcome to FicNexus!</strong> Browse and download fanfiction, join forum discussions, and more.
        </p>
      </div>
    </div>
    <button class="dismiss-btn" onclick={dismiss} aria-label="Dismiss welcome banner">
      ✕
    </button>
  </div>
{/if}

<style>
  .onboarding-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    margin-bottom: 1rem;
    padding: 0.9rem 1rem;
    border-color: var(--color-primary);
    background: linear-gradient(135deg, var(--color-surface) 0%, var(--color-surface-2) 100%);
  }
  .onboarding-content {
    display: flex;
    align-items: flex-start;
    gap: 0.6rem;
    flex: 1;
  }
  .onboarding-icon {
    font-size: 1.5rem;
    flex-shrink: 0;
    margin-top: 0.1rem;
  }
  .onboarding-text {
    margin: 0;
    font-size: 0.9rem;
    line-height: 1.5;
  }
  .onboarding-text a {
    color: var(--color-primary-hover);
    font-weight: 600;
  }
  .dismiss-btn {
    flex-shrink: 0;
    background: none;
    border: 1px solid var(--color-border);
    color: var(--color-muted);
    border-radius: 999px;
    width: 1.6rem;
    height: 1.6rem;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    cursor: pointer;
    transition: color 0.15s, border-color 0.15s;
  }
  .dismiss-btn:hover {
    color: var(--color-text);
    border-color: var(--color-text);
  }
</style>
