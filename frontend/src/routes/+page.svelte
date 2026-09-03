<script lang="ts">
  import { getPref } from '$lib/prefs';
  import ArchiveHome from '$lib/ui/archive/ArchiveHome.svelte';

  // In archive mode the root page renders ArchiveHome inside the
  // ArchiveLayout shell.  In modern mode the layout itself renders
  // HomeDashboard instead of {children}, so this file stays inert.
  const uiMode = $derived(getPref('uiMode'));
</script>

<svelte:head>
  <title>FicHub — Read & download fanfiction from 100+ sites</title>
  <meta name="description" content="Paste any fanfic URL to read it here or download it as EPUB, MOBI, PDF and more. Bookmarks, follows, update notifications — no account required to start." />
</svelte:head>

{#if uiMode === 'archive'}
  <ArchiveHome />
{/if}
