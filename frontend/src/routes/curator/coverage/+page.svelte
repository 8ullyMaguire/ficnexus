<script lang="ts">
  // Curator coverage board — translation coverage matrix per locale
  // ("62% translated to es"). Aggregated from the translation pipeline
  // (approved + machine rows in translation_strings), AO3 archive styling
  // to match the rest of the curator pages.
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';
  import { t } from '$lib/i18n/index.svelte';
  import { coverageMatrix, type LocaleCoverage } from '$lib/api/translate';

  let coverage = $state<Record<string, LocaleCoverage>>({});
  let totalTargets = $state(0);
  let loading = $state(true);
  let error = $state('');

  let locales = $derived(Object.keys(coverage).sort());

  function pct(n: number | undefined): number {
    if (!totalTargets) return 0;
    return Math.round(((n ?? 0) / totalTargets) * 100);
  }

  onMount(async () => {
    await auth.init();
    // Curator gate: legacy role>=5 → site level >= 50.
    if (!auth.isLoggedIn || auth.level < 50) { goto('/'); return; }
    try {
      const res = await coverageMatrix();
      coverage = res.coverage;
      totalTargets = res.totalTargets;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load coverage';
    } finally {
      loading = false;
    }
  });
</script>

<svelte:head><title>{t('curator.coverageTitle')} — FicNexus</title></svelte:head>

<div class="coverage-page">
  <h1 class="page-title">{t('curator.coverageTitle')}</h1>
  <p class="page-sub">
    {totalTargets === 0
      ? t('curator.coverageEmpty')
      : t('curator.coverageSub', { total: totalTargets })}
  </p>

  {#if loading}
    <p class="muted">{t('common.loading')}</p>
  {:else if error}
    <p class="err">{error}</p>
  {:else if locales.length === 0}
    <p class="muted">{t('curator.coverageEmpty')}</p>
  {:else}
    <ul class="cov-list">
      {#each locales as loc (loc)}
        {@const c = coverage[loc] ?? {}}
        {@const approved = pct(c.approved)}
        {@const machine = pct(c.machine)}
        <li class="cov-row">
          <span class="locale">{loc}</span>
          <div class="bar" role="progressbar" aria-valuenow={approved + machine} aria-valuemin={0} aria-valuemax={100}
               aria-label={`${loc}: ${approved}% approved, ${machine}% machine`}>
            <div class="fill approved" style={`width:${approved}%`}></div>
            <div class="fill machine" style={`width:${machine}%`}></div>
          </div>
          <span class="pct">{approved + machine}%</span>
          <span class="detail">{c.approved ?? 0} approved · {c.machine ?? 0} machine</span>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .coverage-page { max-width: 720px; margin: 0 auto; padding: 1rem; }
  .page-title { font-size: 1.4rem; margin-bottom: 0.25rem; }
  .page-sub { color: var(--muted, #666); margin-bottom: 1rem; }
  .cov-list { list-style: none; padding: 0; display: flex; flex-direction: column; gap: 0.6rem; }
  .cov-row { display: flex; align-items: center; gap: 0.75rem; }
  .locale { width: 3.5rem; font-weight: 600; }
  .bar { flex: 1; height: 0.9rem; background: var(--bar-bg, #e5e3dc); border-radius: 0.25rem; display: flex; overflow: hidden; }
  .fill { height: 100%; }
  .fill.approved { background: var(--ok, #4c8a4c); }
  .fill.machine { background: var(--warn, #c9a83c); }
  .pct { width: 3rem; text-align: right; font-variant-numeric: tabular-nums; }
  .detail { color: var(--muted, #666); font-size: 0.85rem; width: 11rem; }
  .muted { color: var(--muted, #666); }
  .err { color: var(--err, #b33); }
</style>
