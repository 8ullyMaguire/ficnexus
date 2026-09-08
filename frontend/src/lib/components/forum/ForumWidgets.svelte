<script lang="ts">
  import RecentTopicsWidget from './RecentTopicsWidget.svelte';
  import PopularTopicsWidget from './PopularTopicsWidget.svelte';
  import ForumStatsWidget from './ForumStatsWidget.svelte';
</script>

<aside class="forum-widgets">
  <ForumStatsWidget />
  <RecentTopicsWidget />
  <PopularTopicsWidget />
</aside>

<style>
  .forum-widgets {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    min-width: 240px;
    max-width: 280px;
  }
  .forum-widgets :global(.widget) {
    background: #161b22;
    border: 1px solid #21262d;
    border-radius: 8px;
  }
</style>
