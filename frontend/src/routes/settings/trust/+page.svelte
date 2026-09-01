<script lang="ts">
  /**
   * Trust level dashboard — the Discourse-style participation axis shown
   * separately from reputation. Explains what the member can already do and
   * exactly what is left to unlock the next level.
   */
  import { onMount } from 'svelte';
  import { getMyTrust, type TrustStatus } from '$lib/api/social';
  import TrustBadge from '$lib/components/trust-badge.svelte';

  let trust = $state<TrustStatus | null>(null);
  let error = $state('');
  let loading = $state(true);

  const METRIC_LABELS: Record<string, string> = {
    works_entered: 'Works entered',
    works_read: 'Works read',
    words_read: 'Words read',
    days_active_30: 'Active days (30d)',
    forum_posts: 'Forum posts',
    reviews: 'Reviews written',
  };

  onMount(async () => {
    try {
      trust = await getMyTrust();
    } catch {
      error = 'Could not load your trust status. Are you signed in?';
    } finally {
      loading = false;
    }
  });
</script>

<svelte:head>
  <title>Trust level — FicNexus</title>
</svelte:head>

<div class="mx-auto max-w-3xl px-4 py-8">
  <header class="mb-6">
    <h1 class="text-2xl font-bold">Trust level</h1>
    <p class="mt-1 text-sm opacity-70">
      Trust measures participation safety, not rank. It decides which write
      surfaces are open to you and how much your reports weigh — it never
      grants moderation power.
    </p>
  </header>

  {#if loading}
    <p class="opacity-60">Loading…</p>
  {:else if error}
    <p class="text-red-600">{error}</p>
  {:else if trust}
    <div class="rounded-lg border p-5">
      <div class="flex items-center gap-3">
        <TrustBadge level={trust.level} levelName={trust.level_name} compact={false} />
      </div>

      <dl class="mt-4 grid grid-cols-2 gap-x-6 gap-y-2 text-sm sm:grid-cols-3">
        {#each Object.entries(METRIC_LABELS) as [key, label]}
          <div class="flex justify-between gap-2 border-b py-1">
            <dt class="opacity-70">{label}</dt>
            <dd class="font-semibold">
              {trust.metrics[key as keyof TrustStatus['metrics']] ?? 0}
            </dd>
          </div>
        {/each}
      </dl>

      {#if trust.next_level?.target != null}
        <div class="mt-4 rounded bg-indigo-50 p-3 text-sm">
          <p class="font-semibold">Next: Trust Level {trust.next_level.target}</p>
          <p class="mt-1 opacity-80">{trust.next_level.hint}</p>
        </div>
      {:else}
        <p class="mt-4 text-sm opacity-70">You have reached the top trust level.</p>
      {/if}
    </div>

    <h2 class="mt-6 text-lg font-semibold">What you can do</h2>
    <ul class="mt-2 space-y-1 text-sm">
      <li>
        {trust.publish_allowed
          ? '✅ Publish skins, recipes, and themes to the marketplace.'
          : '🔒 Publishing to the marketplace unlocks at Trust Level 2.'}
      </li>
      <li>
        {trust.resolve_allowed
          ? '✅ Resolve community reports as a community moderator.'
          : '🔒 Report resolution unlocks at Trust Level 5 (staff-granted).'}
      </li>
    </ul>
  {/if}
</div>
