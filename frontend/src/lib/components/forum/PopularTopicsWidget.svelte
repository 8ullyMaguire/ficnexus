<script lang="ts">
  import { fetchPopularTopics, type WidgetTopic } from '$lib/api/forumWidgets';

  let topics = $state<WidgetTopic[]>([]);
  let loading = $state(true);

  $effect(() => {
    fetchPopularTopics()
      .then((data) => (topics = data.topics))
      .catch(() => {})
      .finally(() => (loading = false));
  });
</script>

<div class="widget">
  <h3 class="widget-title">Popular Topics</h3>
  {#if loading}
    <p class="widget-loading">Loading...</p>
  {:else if topics.length === 0}
    <p class="widget-empty">No topics yet</p>
  {:else}
    <ul class="widget-list">
      {#each topics as topic (topic.id)}
        <li class="widget-item">
          <a href="/forum/topics/{topic.id}" class="widget-link">{topic.title}</a>
          <span class="widget-meta">
            {topic.view_count ?? 0} views · {topic.score ?? 0} score
          </span>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .widget { padding: 0.75rem; }
  .widget-title { font-size: 0.9rem; font-weight: 600; margin-bottom: 0.5rem; color: #e1e4e8; }
  .widget-list { list-style: none; padding: 0; margin: 0; }
  .widget-item { padding: 0.4rem 0; border-bottom: 1px solid #21262d; }
  .widget-item:last-child { border-bottom: none; }
  .widget-link { color: #58a6ff; text-decoration: none; font-size: 0.85rem; }
  .widget-link:hover { text-decoration: underline; }
  .widget-meta { display: block; font-size: 0.75rem; color: #8b949e; margin-top: 0.1rem; }
  .widget-loading, .widget-empty { color: #8b949e; font-size: 0.85rem; }
</style>
