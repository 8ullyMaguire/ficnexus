<script lang="ts">
  import { getPref } from '$lib/prefs';
  import ArchiveHome from '$lib/ui/archive/ArchiveHome.svelte';

  // In archive mode the root page renders ArchiveHome inside the
  // ArchiveLayout shell.  In modern mode the layout itself renders
  // HomeDashboard instead of {children}, so this file stays inert.
  const uiMode = $derived(getPref('uiMode'));
</script>

{#if uiMode === 'archive'}
  <ArchiveHome />
{/if}
