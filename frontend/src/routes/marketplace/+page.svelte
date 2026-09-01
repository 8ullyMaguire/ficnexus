<script lang="ts">
  /**
   * Extension marketplace — browse themes, skins, recipes, layouts, and views.
   * Anything published here can be installed in one click and remixed as a
   * starting point for your own version.
   */
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';

  type ExtensionRow = {
    id: number;
    kind: string;
    slug: string;
    name: string;
    description: string;
    author_id: number;
    version: number;
    installs: number;
    rating: number;
    is_verified: boolean;
    is_public: boolean;
  };

  const KINDS = [
    { value: '', label: 'All' },
    { value: 'theme', label: 'Themes' },
    { value: 'skin', label: 'Skins' },
    { value: 'recipe', label: 'Recipes' },
    { value: 'layout', label: 'Layouts' },
    { value: 'view', label: 'Views' },
    { value: 'rec_strategy', label: 'Recommendation Strategies' },
    { value: 'search', label: 'Search Plugins' },
  ];

  let items: ExtensionRow[] = $state([]);
  let kind = $state('');
  let q = $state('');
  let loading = $state(false);
  let error = $state('');
  let toast = $state('');
  let isAdmin = $state(auth.level >= 100);

  async function load() {
    loading = true;
    error = '';
    try {
      const params = new URLSearchParams();
      if (kind) params.set('kind', kind);
      if (q.trim()) params.set('q', q.trim());
      params.set('limit', '48');
      // Admins see the full manage list (public + private) to verify/clean up.
      const base = isAdmin ? '/api/extensions/admin' : '/api/extensions';
      const res = await fetch(`${base}?${params.toString()}`);
      const data = await res.json();
      items = data.items ?? [];
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load gallery';
    } finally {
      loading = false;
    }
  }

  async function toggleVerify(id: number) {
    try {
      const res = await fetch(`/api/extensions/admin/${id}/verify`, { method: 'POST' });
      const data = await res.json();
      if (data.err === 0) {
        const row = items.find((i) => i.id === id);
        if (row) row.is_verified = data.is_verified;
        toast = data.is_verified ? 'Marked verified' : 'Unmarked';
        setTimeout(() => (toast = ''), 2000);
      }
    } catch {
      /* ignore */
    }
  }

  async function install(id: number) {
    if (!auth.isLoggedIn) return;
    try {
      const res = await fetch(`/api/extensions/${id}/install`, { method: 'POST' });
      if (res.ok) {
        toast = 'Installed';
        const row = items.find((i) => i.id === id);
        if (row) row.installs += 1;
      } else {
        toast = 'Install failed';
      }
    } finally {
      setTimeout(() => (toast = ''), 2500);
    }
  }

  onMount(load);
</script>

<svelte:head>
  <title>Marketplace — FicNexus</title>
</svelte:head>

<div class="mx-auto max-w-5xl px-4 py-8">
  <header class="mb-6">
    <h1 class="text-2xl font-bold">Marketplace</h1>
    <p class="mt-1 text-sm opacity-70">
      Themes, skins, recipes, and layouts shared by the community. Install to
      apply, remix to make it yours.
    </p>
  </header>

  {#if toast}
    <div class="mb-4 rounded bg-emerald-100 px-3 py-2 text-sm text-emerald-800" role="status">
      {toast}
    </div>
  {/if}

  <form
    class="mb-6 flex flex-wrap items-center gap-3"
    onsubmit={(e) => {
      e.preventDefault();
      load();
    }}
  >
    <label class="sr-only" for="kind">Category</label>
    <select
      id="kind"
      bind:value={kind}
      class="rounded border px-3 py-1.5 text-sm"
      onchange={load}
    >
      {#each KINDS as k}
        <option value={k.value}>{k.label}</option>
      {/each}
    </select>

    <label class="sr-only" for="q">Search</label>
    <input
      id="q"
      type="search"
      bind:value={q}
      placeholder="Search extensions…"
      class="min-w-40 flex-1 rounded border px-3 py-1.5 text-sm"
    />
    <button
      type="submit"
      class="rounded bg-indigo-600 px-4 py-1.5 text-sm font-medium text-white hover:bg-indigo-700"
    >
      Search
    </button>
  </form>

  {#if loading}
    <p class="opacity-60">Loading…</p>
  {:else if error}
    <p class="text-red-600">{error}</p>
  {:else if items.length === 0}
    <p class="opacity-60">Nothing published in this category yet.</p>
  {:else}
    <ul class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
      {#each items as item (item.id)}
        <li class="rounded-lg border p-4">
          <div class="flex items-start justify-between gap-2">
            <h2 class="font-semibold">{item.name}</h2>
            <div class="flex shrink-0 items-center gap-1.5">
              {#if item.is_verified}
                <span
                  class="rounded bg-emerald-100 px-1.5 py-0.5 text-xs font-semibold text-emerald-800"
                  title="Reviewed by a curator">verified</span
                >
              {/if}
              {#if isAdmin}
                <button
                  type="button"
                  class="rounded border px-1.5 py-0.5 text-xs font-semibold hover:bg-amber-100"
                  title={item.is_verified ? 'Unmark verified' : 'Mark verified'}
                  onclick={() => toggleVerify(item.id)}
                >
                  {item.is_verified ? 'Unverify' : 'Verify'}
                </button>
              {/if}
            </div>
          </div>
          <p class="mt-1 text-xs uppercase tracking-wide opacity-60">
            {item.kind} · v{item.version}
            {#if !item.is_public && isAdmin}
              <span class="ml-1 rounded bg-gray-200 px-1 py-0.5 text-[10px] not-italic normal-case">private draft</span>
            {/if}
          </p>
          {#if item.description}
            <p class="mt-2 text-sm opacity-80">{item.description}</p>
          {/if}
          <div class="mt-3 flex items-center justify-between text-xs opacity-70">
            <span>
              ★ {item.rating > 0 ? item.rating.toFixed(1) : '—'} · {item.installs}
              install{item.installs === 1 ? '' : 's'}
            </span>
            {#if auth.isLoggedIn}
              <button
                type="button"
                class="rounded bg-indigo-600 px-3 py-1 font-medium text-white hover:bg-indigo-700"
                onclick={() => install(item.id)}
              >
                Install
              </button>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>
