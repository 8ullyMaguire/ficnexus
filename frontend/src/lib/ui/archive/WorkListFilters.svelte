<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    apply: {
      sort: string;
      complete: boolean | null;
      minWords: number | null;
      maxWords: number | null;
      includeTags: string;
      excludeTags: string;
    };
    clear: void;
  }>();

  let sort = 'updated';
  let complete: boolean | null = null;
  let minWords: number | null = null;
  let maxWords: number | null = null;
  let includeTags = '';
  let excludeTags = '';

  const SORT_OPTIONS = [
    { value: 'updated', label: 'Date Updated' },
    { value: 'created', label: 'Date Posted' },
    { value: 'words', label: 'Word Count' },
    { value: 'title', label: 'Title' },
  ];

  function apply() {
    dispatch('apply', { sort, complete, minWords, maxWords, includeTags, excludeTags });
  }

  function clear() {
    sort = 'updated';
    complete = null;
    minWords = null;
    maxWords = null;
    includeTags = '';
    excludeTags = '';
    dispatch('clear');
  }
</script>

<aside class="archive-sidebar">
  <fieldset class="archive-fieldset">
    <legend class="archive-legend">Filters</legend>
    <dl class="archive-dl">
      <div class="dl-row">
        <dt><label for="filt-sort">Sort by</label></dt>
        <dd>
          <select id="filt-sort" bind:value={sort} onchange={apply}>
            {#each SORT_OPTIONS as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </dd>
      </div>
      <div class="dl-row">
        <dt><label for="filt-complete">Complete</label></dt>
        <dd>
          <select id="filt-complete" bind:value={complete} onchange={apply}>
            <option value={null}>All</option>
            <option value={true}>Complete</option>
            <option value={false}>In-Progress</option>
          </select>
        </dd>
      </div>
      <div class="dl-row">
        <dt><label for="filt-minwords">Min Words</label></dt>
        <dd>
          <input
            id="filt-minwords"
            type="number"
            min="0"
            bind:value={minWords}
            onchange={apply}
            placeholder="e.g. 10000"
          />
        </dd>
      </div>
      <div class="dl-row">
        <dt><label for="filt-maxwords">Max Words</label></dt>
        <dd>
          <input
            id="filt-maxwords"
            type="number"
            min="0"
            bind:value={maxWords}
            onchange={apply}
            placeholder="e.g. 100000"
          />
        </dd>
      </div>
      <div class="dl-row">
        <dt><label for="filt-include">Include Tags</label></dt>
        <dd>
          <input
            id="filt-include"
            type="text"
            bind:value={includeTags}
            onchange={apply}
            placeholder="tag1, tag2"
          />
        </dd>
      </div>
      <div class="dl-row">
        <dt><label for="filt-exclude">Exclude Tags</label></dt>
        <dd>
          <input
            id="filt-exclude"
            type="text"
            bind:value={excludeTags}
            onchange={apply}
            placeholder="tag1, tag2"
          />
        </dd>
      </div>
    </dl>
    <p class="archive-actions">
      <button class="archive-btn archive-btn--small" onclick={clear}>Clear Filters</button>
    </p>
  </fieldset>
</aside>
