<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    href = '',
    onclick = undefined,
    disabled = false,
    type = 'button',
    children,
  }: {
    href?: string;
    onclick?: (e: MouseEvent) => void;
    disabled?: boolean;
    type?: 'button' | 'submit' | 'reset';
    children: Snippet;
  } = $props();
</script>

{#if href && !disabled}
  <a class="archive-btn" href="{href}" onclick={onclick}>
    {@render children()}
  </a>
{:else}
  <button class="archive-btn" {type} {disabled} {onclick}>
    {@render children()}
  </button>
{/if}

<style>
  .archive-btn {
    display: inline-block;
    padding: 0.25em 0.8em;
    font-size: 0.9em;
    font-family: inherit;
    font-weight: 600;
    line-height: 1.6;
    color: #444;
    font-weight: 400;
    line-height: 1.286;
    background: #eee;
    background-image: linear-gradient(#fff 2%, #ddd 95%, #bbb 100%);
    border: 1px solid #bbb;
    border-bottom: 1px solid #aaa;
    border-radius: 0.25em;
    cursor: pointer;
    text-decoration: none;
    text-align: center;
    vertical-align: middle;
    transition: color 0.15s, border-color 0.15s, box-shadow 0.15s;
  }

  /* AO3 .actions hover: ink lifts to #900, top/left edges darken,
     pressed-in inset shadow. */
  .archive-btn:hover,
  .archive-btn:focus {
    color: #900;
    border-top: 1px solid #999;
    border-left: 1px solid #999;
    box-shadow: inset 2px 2px 2px #bbb;
    text-decoration: none;
  }

  .archive-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
