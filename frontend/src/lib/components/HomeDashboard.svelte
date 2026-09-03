<script lang="ts">
  import { onMount } from 'svelte';
  import { getTrending } from '$lib/api/social';
  import { fetchPersonalRecs, isPersonalizedRecsEnabled } from '$lib/api/recommendations';
  import type { PersonalRecsResponse } from '$lib/api/recommendations';
  import type { RecResult } from '$lib/api/types';
  import type { TrendingItem } from '$lib/api/social-types';
  import { formatWords, stripHtml } from '$lib/util';
  import { goto } from '$app/navigation';
  import { t } from '$lib/i18n/index.svelte';
  import DocLink from '$lib/components/DocLink.svelte';
  import { auth } from '$lib/stores/auth.svelte';

  // ── Compact download input ────────────────────────────────────────────
  let dlUrl = $state('');
  let dlLoading = $state(false);
  let dlError = $state('');

  async function handleDownload() {
    if (!dlUrl.trim()) return;
    dlLoading = true;
    dlError = '';
    try {
      // Navigate to the Download route page with the URL prefilled via
      // sessionStorage; DownloadTab reads it on mount and starts the export.
      sessionStorage.setItem('fichub_dl_url', dlUrl.trim());
      goto('/download');
    } catch {
      dlError = t('home.couldNotStart');
    } finally {
      dlLoading = false;
    }
  }

  function onDlKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') handleDownload();
  }

  // ── Sections (independent, allSettled) ────────────────────────────────
  let recs = $state<RecResult[]>([]);
  let basedOn = $state<{ title: string; url_id: string }[]>([]);
  let recsEnough = $state(false);
  // True when the pluggable engine reports curator-shaped recs
  // (response.curator_alpha > 0 / strategies present with curator influence).
  let curatorInfluenced = $state(false);
  let trending = $state<TrendingItem[]>([]);

  let leftLoading = $state(true);
  let rightLoading = $state(true);
  let leftError = $state('');
  let rightError = $state('');

  onMount(async () => {
    // Pref-aware: default-on for logged-in users; opt-out via
    // recs.personalized="false" pref; anonymous users get fallback recs.
    const prefsEnabled = await isPersonalizedRecsEnabled();
    const doPersonal = prefsEnabled && auth.isLoggedIn;
    const recsPromise = doPersonal
      ? fetchPersonalRecs()
      : Promise.reject(new Error('opted-out-or-anon'));
    const [recsRes, trendRes] = await Promise.allSettled([
      recsPromise,
      getTrending(7, 10),
    ]);

    // Left: personal recs (or fallback)
    if (recsRes.status === 'fulfilled') {
      const r = recsRes.value as PersonalRecsResponse & { curator_alpha?: number; strategies?: unknown[] };
      recsEnough = r.enough_data && r.recs.length > 0;
      recs = r.recs ?? [];
      basedOn = r.based_on ?? [];
      // Curator-shaped: the pluggable engine marks the blend when the
      // curator prior is active (curator_alpha present and < 1 means the
      // user's own taste is still in the mix; alpha === 1 is pure curator).
      curatorInfluenced = r.curator_alpha != null && r.curator_alpha > 0;
      if (!recsEnough) leftError = 'not_enough'; // signal the fallback chain
    } else if (recsRes.reason instanceof Error && recsRes.reason.message === 'opted-out-or-anon') {
      // Anonymous or opted-out: graceful skip, not an error — let the
      // fallback chain render trending (if available) or the hint card.
      leftError = 'not_enough';
    } else {
      leftError = 'error';
    }
    leftLoading = false;

    // Right: trending
    if (trendRes.status === 'fulfilled') {
      trending = (trendRes.value as { err: number; trending: TrendingItem[] }).trending ?? [];
    } else {
      rightError = 'error';
    }
    rightLoading = false;
  });

  // Left-section fallback chain: personal → trending → hint card
  const showPersonal = $derived(recsEnough && recs.length > 0 && !leftError);
  const showTrendingLeft = $derived(!showPersonal && trending.length > 0 && !leftError);
</script>

<div class="home-dashboard">
  <!-- Compact download input: the core action stays on the first screen -->
  <div class="dl-card card">
    <form onsubmit={(e) => { e.preventDefault(); handleDownload(); }}>
      <label class="muted dl-label" for="home-dl-url">{t('home.dlLabel')}</label>
      <div class="dl-row">
        <input
          id="home-dl-url"
          type="url"
          placeholder={t('home.dlPlaceholder')}
          bind:value={dlUrl}
          onkeydown={onDlKeydown}
          disabled={dlLoading}
          aria-label={t('home.dlAria')}
        />
        <button class="btn" type="submit" disabled={dlLoading}>
          {#if dlLoading}<span class="spinner"></span> {t('home.starting')}{:else}{t('home.download')}{/if}
        </button>
      </div>
      {#if dlError}<p class="muted dl-error">{dlError}</p>{/if}
    </form>
  </div>

  <div class="home-grid">
    <!-- Left: personalized recommendations -->
    <section class="home-section" aria-label={t('home.recommended')}>
      <h2 class="section-title">
        {#if curatorInfluenced}{t('home.curatorsPick')}{:else}{t('home.recommended')}{/if}
      </h2>

      {#if leftLoading}
        <div class="card skeleton"><span class="spinner"></span> {t('home.loading')}</div>
      {:else if showPersonal}
        <p class="muted based-on">
          {#if basedOn.length > 0}
            {t('home.basedOn')} {basedOn.slice(0, 3).map((b) => b.title).join(', ')}
          {/if}
        </p>
        <div class="rec-list">
          {#each recs.slice(0, 5) as r (r.url_id)}
            <div class="card rec-item">
              <h3><a href={r.download_urls?.epub ?? '#'}>{r.title}</a></h3>
              <p class="muted">{t('home.by')} {r.author} · <span class="tag">{r.site_domain}</span></p>
              <p class="meta-line">{formatWords(r.words)} {t('home.words')} · {r.chapters} {t('home.chapters')} · {r.status}</p>
              {#if r.summary}<p class="desc">{stripHtml(r.summary).slice(0, 160)}</p>{/if}
            </div>
          {/each}
        </div>
      {:else if showTrendingLeft}
        <p class="muted based-on">{t('home.popularNow')}</p>
        <div class="rec-list">
          {#each trending.slice(0, 5) as tItem (tItem.url_id)}
            <div class="card rec-item">
              <h3><a href={`/works/${tItem.url_id}`}>{tItem.title}</a></h3>
              <p class="muted">{t('home.by')} {tItem.author}</p>
              <p class="meta-line">{formatWords(tItem.words)} {t('home.words')} · {tItem.chapters} {t('home.chapters')} · {tItem.status}</p>
            </div>
          {/each}
        </div>
      {:else if leftError === 'error'}
        <div class="card empty"><p class="muted">{t('home.couldNotLoadRecs')}</p></div>
      {:else}
        <div class="card empty">
          <p class="muted">{t('home.bookmarkHint')} <DocLink slug="recommendations#getting-recommendations-" label="?" title="How personal recommendations work" inline /></p>
          <a class="btn btn-secondary sm" href="/search">{t('home.findStories')}</a>
        </div>
      {/if}
    </section>

    <!-- Right: trending -->
    <section class="home-section" aria-label={t('home.trendingWeek')}>
      <h2 class="section-title">{t('home.trendingWeek')}</h2>
      {#if rightLoading}
        <div class="card skeleton"><span class="spinner"></span> {t('home.loading')}</div>
      {:else if rightError}
        <div class="card empty"><p class="muted">{t('home.couldNotLoadTrending')}</p></div>
      {:else if trending.length > 0}
        <div class="rec-list">
          {#each trending.slice(0, 5) as tItem (tItem.url_id)}
            <div class="card rec-item">
              <h3><a href={`/works/${tItem.url_id}`}>{tItem.title}</a></h3>
              <p class="muted">{t('home.by')} {tItem.author} · {tItem.downloads} {t('home.downloads')}</p>
              <p class="meta-line">{formatWords(tItem.words)} {t('home.words')} · {tItem.chapters} {t('home.chapters')} · {tItem.status}</p>
            </div>
          {/each}
        </div>
      {:else}
        <div class="card empty"><p class="muted">{t('home.nothingTrending')}</p></div>
      {/if}
    </section>
  </div>
</div>

<style>
  .home-dashboard { display: flex; flex-direction: column; gap: 1.2rem; }
  .dl-card { padding: 1rem; }
  .dl-label { display: block; margin-bottom: 0.4rem; font-size: 0.85rem; }
  .dl-row { display: flex; gap: 0.5rem; }
  .dl-row input { flex: 1; padding: 0.6rem 0.8rem; border-radius: var(--radius-sm); border: 1px solid var(--color-border); background: var(--color-surface); color: var(--color-text); }
  .dl-error { margin: 0.4rem 0 0; font-size: 0.85rem; color: var(--color-error); }
  .home-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1.2rem; }
  @media (max-width: 768px) { .home-grid { grid-template-columns: 1fr; } }
  .section-title { margin: 0 0 0.4rem; font-size: 1.05rem; }
  .based-on { font-size: 0.85rem; margin: 0 0 0.5rem; }
  .rec-list { display: flex; flex-direction: column; gap: 0.6rem; }
  .rec-item { padding: 0.8rem 1rem; }
  .rec-item h3 { margin: 0 0 0.2rem; font-size: 0.98rem; }
  .rec-item h3 a { color: var(--color-text); text-decoration: none; }
  .rec-item h3 a:hover { text-decoration: underline; }
  .meta-line { margin: 0.2rem 0 0; font-size: 0.82rem; }
  .desc { margin: 0.4rem 0 0; font-size: 0.85rem; }
  .empty { padding: 1.2rem; text-align: center; }
  .empty p { margin: 0 0 0.6rem; }
  .skeleton { padding: 1rem; }
</style>
