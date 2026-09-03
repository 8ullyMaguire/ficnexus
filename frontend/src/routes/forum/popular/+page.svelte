<script lang="ts">
  import { onMount } from 'svelte';
  import { getPopularTopics } from '$lib/api/forum';
  import type { ForumTopic } from '$lib/api/forum';
  let items: ForumTopic[] = [];
  let err = '';
  let loading = true;
  let sort: string = 'views';
  async function load(s: string) {
    sort = s; loading = true; err = '';
    try { const r: any = await getPopularTopics(25, null, s); if (r.err) err = r.msg ?? 'failed'; else items = r.items ?? []; } catch (e: any) { err = e?.message ?? String(e); } finally { loading = false; }
  }
  onMount(() => load('views'));
</script>
<svelte:head><title>Popular — Forum — FicNexus</title></svelte:head>
<div class="archive-wrap">
  <h1>Popular</h1>
  <div class="sort-row">
    <button class="sort-btn" class:active={sort==='views'} on:click={() => load('views')}>Views</button>
    <button class="sort-btn" class:active={sort==='posts'} on:click={() => load('posts')}>Replies</button>
    <button class="sort-btn" class:active={sort==='votes'} on:click={() => load('votes')}>Votes</button>
  </div>
  {#if loading}<p>Loading...</p>
  {:else if err}<p class="err">{err}</p>
  {:else if !items.length}<p class="empty">No topics.</p>
  {:else}
    <ul class="topic-list">
      {#each items as t}
        <li><a href={`/forum/board/${encodeURIComponent(t.topic_slug ?? `topic-${t.id}`)}.${t.id}`}>{t.title}</a> <span class="meta">{t.reply_count} replies · {t.view_count} views</span></li>
      {/each}
    </ul>
  {/if}
</div>
<style>
  .archive-wrap { max-width: 860px; margin: 1.2rem auto; padding: 0 1rem; }
  .sort-row { display: flex; gap: 0.4rem; margin: 0.6rem 0; }
  .sort-btn { padding: 0.25rem 0.7rem; border: 1px solid #ddd; background: #fff; border-radius: 999px; font-size: 0.85rem; cursor: pointer; }
  .sort-btn.active { background: #222; color: #fff; border-color: #222; }
  .topic-list { list-style: none; padding: 0; }
  .topic-list li { padding: 0.55rem 0; border-bottom: 1px solid #e6e6e6; }
  .topic-list a { font-weight: 600; text-decoration: none; }
  .meta { color: #666; font-size: 0.9em; margin-left: 0.5rem; }
  .err { color: #900; }
</style>
