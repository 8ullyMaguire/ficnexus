<script lang="ts">
  import { onMount } from 'svelte';
  import { getRecentTopics } from '$lib/api/forum';
  import type { ForumTopic } from '$lib/api/forum';
  let items: ForumTopic[] = [];
  let err = '';
  let loading = true;
  onMount(async () => {
    try { const r: any = await getRecentTopics(25); if (r.err) err = r.msg ?? 'failed'; else items = r.items ?? []; } catch (e: any) { err = e?.message ?? String(e); } finally { loading = false; }
  });
</script>
<svelte:head><title>Recent — Forum — FicNexus</title></svelte:head>
<div class="archive-wrap">
  <h1>Recent</h1>
  {#if loading}<p>Loading...</p>
  {:else if err}<p class="err">{err}</p>
  {:else if !items.length}<p class="empty">No topics.</p>
  {:else}
    <ul class="topic-list">
      {#each items as t}
        <li><a href={`/forum/board/${encodeURIComponent(t.topic_slug ?? `topic-${t.id}`)}.${t.id}`}>{t.title}</a> <span class="meta">{t.reply_count} replies</span></li>
      {/each}
    </ul>
  {/if}
</div>
<style>
  .archive-wrap { max-width: 860px; margin: 1.2rem auto; padding: 0 1rem; }
  .topic-list { list-style: none; padding: 0; }
  .topic-list li { padding: 0.55rem 0; border-bottom: 1px solid #e6e6e6; }
  .topic-list a { font-weight: 600; text-decoration: none; }
  .meta { color: #666; font-size: 0.9em; margin-left: 0.5rem; }
  .err { color: #900; }
</style>
