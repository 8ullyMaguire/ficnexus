<!--
  TranslatedText — one block of user-visible text that may be machine- or
  community-translated into the reader's language (docs/plans/
  translation-everything.md §2.3).

  Renders `original` until a translation resolves; then shows the translated
  text with a tiny provenance badge:
    machine  → "machine translated" badge + Improve/Report affordances,
    approved → subtle dot, title 'community translation — click for original'.
  Clicking the badge (or keyboard Enter) toggles the original, persisted per
  key in sessionStorage for the session.

  html=true renders via {@html}. Trust note: source bodies are already
  author HTML rendered by the app; machine text is our own Ollama output on
  that same HTML; approved text passed curator consensus. Same trust level
  as the original — no new XSS surface beyond what the untranslated render
  already has.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { targetReadingLocale } from '$lib/stores/readingLocale.svelte';
  import { requestTranslations, type TranslateItem } from '$lib/api/translate';
  import { auth } from '$lib/stores/auth.svelte';

  let {
    type,
    id,
    field,
    original,
    html = false,
    inline = false,
    class: className = '',
  }: {
    type: string;
    id: string;
    field: string;
    original: string;
    html?: boolean;
    inline?: boolean;
    class?: string;
  } = $props();

  let locale = $derived(targetReadingLocale());
  let translation = $state<{ text: string; status: 'machine' | 'approved' } | null>(null);
  let showOriginal = $state(false);

  const key = $derived(`${type}:${id}:${field}`);
  const stashKey = $derived(`fichub_showorig:${key}`);
  const shown = $derived(
    !locale || !translation || showOriginal ? original : translation.text,
  );

  async function load() {
    if (!locale || !original) return;
    const item: TranslateItem = { type, id, field };
    const all = await requestTranslations([item], locale);
    const r = all[`${type}:${id}:${field}:${locale}`];
    if (r?.text && (r.status === 'machine' || r.status === 'approved')) {
      translation = { text: r.text, status: r.status };
    }
  }

  function toggle() {
    showOriginal = !showOriginal;
    try {
      if (showOriginal) sessionStorage.setItem(stashKey, '1');
      else sessionStorage.removeItem(stashKey);
    } catch { /* storage unavailable */ }
  }

  function goImprove() {
    try {
      sessionStorage.setItem(`fichub_tr_original:${key}`, original);
    } catch { /* noop */ }
    const q = new URLSearchParams({ target: key, locale: locale ?? 'es' });
    window.location.href = `/translate?${q.toString()}`;
  }

  onMount(() => {
    try {
      showOriginal = sessionStorage.getItem(stashKey) === '1';
    } catch { /* noop */ }
    void load();
  });
</script>

{#if html}
  <!-- see trust note above -->
  <span class="tl-root {className}">{@html shown}</span>
{:else}
  <span class="tl-root {className}">{shown}</span>
{/if}

{#if locale && translation && !showOriginal}
  {#if translation.status === 'machine'}
    <button
      class="tl-badge"
      type="button"
      title={t('translate.badgeTitle')}
      onclick={toggle}
    >{t('translate.machineBadge')}</button>
    {#if auth.isLoggedIn}
      <button
        class="tl-improve"
        type="button"
        title={t('translate.improveTitle')}
        onclick={goImprove}
      >{t('translate.improve')}</button>
    {/if}
  {:else}
    <!-- approved: community translation — subtle affordance only -->
    <button
      class="tl-approved"
      type="button"
      title={t('translate.communityShort')}
      onclick={toggle}
      aria-label={t('translate.showOriginal')}>·</button>
  {/if}
{:else if locale && showOriginal && translation}
  <button class="tl-badge" type="button" title={t('translate.hideOriginal')} onclick={toggle}>orig</button>
{/if}

<style>
  .tl-badge,
  .tl-approved,
  .tl-improve {
    background: transparent;
    border: none;
    box-shadow: none;
    padding: 0 0.15em;
    margin-left: 0.3em;
    font-size: 0.72em;
    font-style: italic;
    color: var(--archive-muted, #666);
    cursor: pointer;
    font-family: inherit;
    vertical-align: baseline;
    line-height: 1;
  }
  .tl-badge { border-bottom: 1px dotted #999; border-radius: 0; }
  .tl-badge:hover { color: #900; border-bottom-color: #900; background: transparent; }
  .tl-approved { color: #2e9e5b; border-bottom: none; }
  .tl-approved:hover { color: #900; }
  .tl-improve { border-bottom: 1px dotted #999; border-radius: 0; }
  .tl-improve:hover { color: #900; border-bottom-color: #900; background: transparent; }
</style>
