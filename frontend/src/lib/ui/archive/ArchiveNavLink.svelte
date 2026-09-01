<script lang="ts">
  import type { Snippet } from 'svelte';
  import { page } from '$app/stores';

  let {
    href,
    label = '',
    children,
  }: {
    href: string;
    label?: string;
    children?: Snippet;
  } = $props();

  let active = $derived($page.url.pathname === href || $page.url.pathname.startsWith(href + '/'));
</script>

<a class="archive-nav-link" class:active href="{href}">
  {#if children}
    {@render children()}
  {:else}
    {label}
  {/if}
</a>

<style>
  .archive-nav-link {
    display: inline-block;
    padding: 0.3em 0.7em;
    font-size: 0.9em;
    font-weight: 600;
    color: var(--archive-link, #990000);
    text-decoration: none;
    border-bottom: 2px solid transparent;
    transition: color 0.15s, border-color 0.15s;
    white-space: nowrap;
  }

  .archive-nav-link:hover {
    background: #ddd;
    color: #111;
    text-decoration: none;
  }

  .archive-nav-link.active {
    color: var(--archive-text, #2a2a2a);
    border-bottom-color: var(--archive-accent-line, #990000);
    text-decoration: none;
  }
</style>
