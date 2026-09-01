<!--
  Per-follow exclusions control: lets a user filter out specific works,
  series, or fandoms from a followed author's updates feed. Rendered once per
  author follow. Manages its own state: loads the follow's exclusions on
  mount, and adds/removes rows via the follow-exclusion API.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import type { Follow, FollowExclusion } from '$lib/api/social-types';
  import { listFollowExclusions, addFollowExclusion, deleteFollowExclusion } from '$lib/api/social';
  import { t } from '$lib/i18n/index.svelte';

  let { follow }: { follow: Follow } = $props();

  let exclusions = $state<FollowExclusion[]>([]);
  let loaded = $state(false);
  let loading = $state(false);
  let type = $state<'work' | 'series' | 'fandom'>('work');
  let workId = $state('');
  let seriesId = $state('');
  let fandomName = $state('');
  let msg = $state('');

  onMount(() => { void load(); });

  async function load() {
    if (loaded) return;
    loading = true; msg = '';
    try {
      const res = await listFollowExclusions(follow.id);
      exclusions = res.err === 0 ? res.exclusions : [];
      loaded = true;
    } catch { msg = t('follows.excludeError'); }
    finally { loading = false; }
  }

  async function add() {
    msg = '';
    const target: { work_id?: number; series_id?: number; fandom?: string } = {};
    if (type === 'work') {
      const id = Number(workId);
      if (!id) { workId = ''; return; }
      target.work_id = id;
    } else if (type === 'series') {
      const id = Number(seriesId);
      if (!id) { seriesId = ''; return; }
      target.series_id = id;
    } else {
      const name = fandomName.trim();
      if (!name) { fandomName = ''; return; }
      target.fandom = name;
    }
    try {
      await addFollowExclusion(follow.id, type, target);
      workId = ''; seriesId = ''; fandomName = '';
      const res = await listFollowExclusions(follow.id);
      exclusions = res.err === 0 ? res.exclusions : exclusions;
    } catch { msg = t('follows.excludeError'); }
  }

  async function remove(ex: FollowExclusion) {
    msg = '';
    try {
      await deleteFollowExclusion(follow.id, ex.id);
      exclusions = exclusions.filter((e) => e.id !== ex.id);
    } catch { msg = t('follows.excludeError'); }
  }

  function label(ex: FollowExclusion): string {
    if (ex.exclude_type === 'work') return `${t('follows.excludeWork')}: #${ex.work_id}`;
    if (ex.exclude_type === 'series') return `${t('follows.excludeSeries')}: #${ex.series_id}`;
    return `${t('follows.excludeFandom')}: ${ex.fandom}`;
  }
</script>

<div class="follow-exclusions">
  <h4 class="follow-exclusions-title">{t('follows.exclusions')}</h4>
  <p class="follow-exclusions-hint muted">{t('follows.exclusionsHint')}</p>

  {#if loading}
    <p class="muted">{t('follows.loading')}</p>
  {:else if exclusions.length === 0}
    <p class="muted">{t('follows.exclusionsEmpty')}</p>
  {:else}
    <ul class="follow-exclusions-list">
      {#each exclusions as ex (ex.id)}
        <li class="follow-exclusion-item">
          <span class="follow-exclusion-label">{label(ex)}</span>
          <button
            type="button"
            class="follow-exclusion-remove"
            onclick={() => void remove(ex)}
          >{t('follows.excludeRemove')}</button>
        </li>
      {/each}
    </ul>
  {/if}

  <div class="follow-exclusions-add">
    <select class="follow-exclusions-type" bind:value={type} aria-label={t('follows.exclusions')}>
      <option value="work">{t('follows.excludeWork')}</option>
      <option value="series">{t('follows.excludeSeries')}</option>
      <option value="fandom">{t('follows.excludeFandom')}</option>
    </select>

    {#if type === 'work'}
      <input
        class="follow-exclusions-input" type="number" min="1" bind:value={workId}
        placeholder={t('follows.excludeWorkId')} aria-label={t('follows.excludeWorkId')}
      />
    {:else if type === 'series'}
      <input
        class="follow-exclusions-input" type="number" min="1" bind:value={seriesId}
        placeholder={t('follows.excludeSeriesId')} aria-label={t('follows.excludeSeriesId')}
      />
    {:else}
      <input
        class="follow-exclusions-input" type="text" bind:value={fandomName}
        placeholder={t('follows.excludeFandomName')} aria-label={t('follows.excludeFandomName')}
      />
    {/if}

    <button type="button" class="follow-exclusion-add-btn" onclick={() => void add()}>
      {t('follows.excludeAdd')}
    </button>
  </div>

  {#if msg}
    <p class="follow-exclusion-msg error-text">{msg}</p>
  {/if}
</div>

<style>
  .follow-exclusions {
    margin-top: 0.5rem;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border-color, rgba(0, 0, 0, 0.12));
    border-radius: var(--radius-sm, 6px);
    background: var(--bg-raised, rgba(0, 0, 0, 0.03));
    font-size: 0.9em;
  }
  .follow-exclusions-title {
    margin: 0 0 0.15em;
    font-size: 0.95em;
    font-weight: 700;
  }
  .follow-exclusions-hint {
    margin: 0 0 0.5em;
    font-size: 0.85em;
    font-style: italic;
  }
  .follow-exclusions-list {
    list-style: none;
    margin: 0 0 0.5em;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .follow-exclusion-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.6em;
    padding: 0.15rem 0.25rem;
  }
  .follow-exclusion-label {
    flex: 1;
    overflow-wrap: anywhere;
  }
  .follow-exclusion-remove,
  .follow-exclusion-add-btn {
    padding: 0.2rem 0.55rem;
    font-size: 0.8em;
    border-radius: var(--radius-sm, 4px);
    cursor: pointer;
    font-family: inherit;
  }
  .follow-exclusion-remove {
    background: transparent;
    border: 1px solid var(--color-error, #cc0000);
    color: var(--color-error, #cc0000);
  }
  .follow-exclusion-remove:hover {
    background: var(--color-error, #cc0000);
    color: #ffffff;
  }
  .follow-exclusions-add {
    display: flex;
    gap: 0.4rem;
    flex-wrap: wrap;
    align-items: center;
  }
  .follow-exclusions-type,
  .follow-exclusions-input {
    font-family: inherit;
    font-size: 0.85em;
    padding: 0.25rem 0.4rem;
    border: 1px solid var(--border-color, rgba(0, 0, 0, 0.2));
    border-radius: var(--radius-sm, 4px);
    background: transparent;
  }
  .follow-exclusions-input.follow-exclusions-input {
    flex: 1;
    min-width: 120px;
  }
  .follow-exclusion-add-btn {
    background: var(--accent, #555);
    border: 1px solid var(--accent, #555);
    color: #ffffff;
  }
  .follow-exclusion-add-btn:hover {
    filter: brightness(1.1);
  }
  .follow-exclusion-msg {
    margin: 0.4em 0 0;
    font-size: 0.85em;
    font-weight: 600;
  }
</style>
