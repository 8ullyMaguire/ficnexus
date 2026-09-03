<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    label,
    icon = '',
    triggerClass = '',
    align = 'left',
    children,
  }: {
    label: string;
    icon?: string;
    triggerClass?: string;
    align?: 'left' | 'right';
    children: Snippet;
  } = $props();

  let open = $state(false);
  let root = $state<HTMLDivElement | null>(null);

  function toggle() {
    open = !open;
  }

  function close() {
    open = false;
  }

  function onRootKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }

  function onWindowPointerDown(e: PointerEvent) {
    if (root && !root.contains(e.target as Node)) close();
  }

  // While open, close when clicking anywhere outside the dropdown
  $effect(() => {
    if (!open) return;
    window.addEventListener('pointerdown', onWindowPointerDown);
    return () => window.removeEventListener('pointerdown', onWindowPointerDown);
  });

  // Close when an interactive item inside the menu is activated
  function onMenuClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (target.closest('a, button')) close();
  }
</script>

<div class="nav-dropdown" class:open bind:this={root} onkeydown={onRootKeydown} role="none">
  <button
    class="dd-trigger {triggerClass}"
    type="button"
    aria-haspopup="menu"
    aria-expanded={open}
    onclick={toggle}
  >
    {#if icon}<span class="dd-icon" aria-hidden="true">{icon}</span>{/if}
    <span>{label}</span>
    <span class="dd-caret" aria-hidden="true">{open ? '▴' : '▾'}</span>
  </button>
  {#if open}
    <div class="dd-menu" class:align-right={align === 'right'} role="menu" tabindex="-1" onclick={onMenuClick} onkeydown={onRootKeydown}>
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .nav-dropdown {
    position: relative;
  }
  .dd-trigger {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    background: transparent;
    border: 1px solid transparent;
    color: var(--color-muted);
    padding: 0.45rem 0.7rem;
    border-radius: var(--radius-sm);
    font-weight: 600;
    font-size: 0.92rem;
    transition: all 0.15s;
  }
  .dd-trigger:hover,
  .nav-dropdown.open .dd-trigger {
    color: var(--color-text);
    background: var(--color-surface-2);
  }
  .dd-icon {
    font-size: 0.95rem;
  }
  .dd-caret {
    font-size: 0.7rem;
    opacity: 0.75;
  }
  .dd-menu {
    position: absolute;
    top: calc(100% + 0.4rem);
    left: 0;
    min-width: 210px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    box-shadow: var(--shadow);
    padding: 0.35rem;
    z-index: 60;
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }
  .dd-menu.align-right {
    left: auto;
    right: 0;
  }
  .dd-item {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    width: 100%;
    padding: 0.45rem 0.7rem;
    border: none;
    border-radius: var(--radius-sm);
    background: none;
    color: var(--color-muted);
    font-size: 0.88rem;
    font-weight: 600;
    text-align: left;
    text-decoration: none;
    cursor: pointer;
    transition: all 0.15s;
  }
  .dd-item:hover {
    color: var(--color-text);
    background: var(--color-surface-2);
    text-decoration: none;
  }
  .dd-divider {
    height: 1px;
    background: var(--color-border);
    margin: 0.3rem 0.2rem;
  }
  .dd-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.6rem;
    padding: 0.45rem 0.7rem;
    color: var(--color-muted);
    font-size: 0.88rem;
    font-weight: 600;
  }
  .dd-inline {
    padding: 0.3rem 0.7rem;
  }
  .dd-error {
    color: var(--color-danger, #e5484d);
    font-size: 0.8rem;
    justify-content: flex-start;
  }
  .dd-item:disabled {
    opacity: 0.6;
    cursor: default;
  }
</style>
