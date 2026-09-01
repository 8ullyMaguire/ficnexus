<script lang="ts">
  import { onMount } from 'svelte';
  import { adminFetch } from '$lib/api/admin';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  interface Feature {
    id: number;
    slug: string;
    name: string;
    description: string;
    gate_type: string;
    gate_value: number;
    is_default: boolean;
    is_revocable: boolean;
    [key: string]: unknown;
  }

  interface FeatureStat {
    slug: string;
    enabled_count: number;
  }

  let features = $state<Feature[]>([]);
  let stats = $state<FeatureStat[]>([]);
  let loading = $state(true);
  let saving = $state(false);
  let error = $state('');
  let msg = $state('');
  let dirtySlugs = $state<Set<string>>(new Set());

  onMount(async () => {
    await loadFeatures();
    await loadStats();
  });

  async function loadFeatures() {
    loading = true;
    error = '';
    try {
      const data = await adminFetch('/api/admin/features');
      const json = await data.json();
      // admin_features returns [{feature: {...}, unlocked, enabled}]
      features = json.map((item: any) => ({
        ...item.feature,
      }));
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function loadStats() {
    try {
      const data = await adminFetch('/api/admin/features/stats');
      stats = await data.json();
    } catch {
      // non-critical, silently ignore
    }
  }

  function markDirty(slug: string) {
    dirtySlugs = new Set([...dirtySlugs, slug]);
  }

  async function saveFeature(slug: string) {
    saving = true;
    error = '';
    msg = '';
    try {
      const f = features.find((feat) => feat.slug === slug);
      if (!f) return;
      const res = await adminFetch(`/api/admin/features/${slug}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: f.name,
          description: f.description,
          gate_type: f.gate_type,
          gate_value: f.gate_value,
          is_default: f.is_default,
          is_revocable: f.is_revocable,
        }),
      });
      const body = await res.json();
      if (res.ok) {
        msg = `✓ Saved ${slug}`;
        dirtySlugs = new Set([...dirtySlugs].filter((s) => s !== slug));
      } else {
        error = body.msg ?? `Failed to save ${slug}`;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  async function saveAll() {
    saving = true;
    error = '';
    msg = '';
    try {
      for (const slug of dirtySlugs) {
        const f = features.find((feat) => feat.slug === slug);
        if (!f) continue;
        const res = await adminFetch(`/api/admin/features/${slug}`, {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            name: f.name,
            description: f.description,
            gate_type: f.gate_type,
            gate_value: f.gate_value,
            is_default: f.is_default,
            is_revocable: f.is_revocable,
          }),
        });
        if (!res.ok) {
          const body = await res.json();
          throw new Error(body.msg ?? `Failed to save ${slug}`);
        }
      }
      msg = `✓ Saved ${dirtySlugs.size} feature(s)`;
      dirtySlugs = new Set();
      await loadStats();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  function getEnabledCount(slug: string): number {
    return stats.find((s) => s.slug === slug)?.enabled_count ?? 0;
  }
</script>

<svelte:head><title>Feature Flags — FicNexus Admin</title></svelte:head>

{#if uiMode === 'archive'}
  <!-- ── Archive mode ─────────────────────────────────────────── -->
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Feature Flags</h1>
      </header>

      <p class="archive-links">
        <a class="archive-link" href="/admin/features/analytics">📊 Analytics</a>
        {#if dirtySlugs.size > 0}
          {' · '}
          <button class="archive-btn" type="button" onclick={saveAll} disabled={saving}>
            {saving ? 'Saving…' : `💾 Save ${dirtySlugs.size} change(s)`}
          </button>
        {/if}
      </p>

      {#if error}
        <p class="archive-error" role="alert">{error}</p>
      {/if}
      {#if msg}
        <p class="archive-msg">{msg}</p>
      {/if}

      {#if loading}
        <p class="archive-note">Loading features…</p>
      {:else if features.length === 0}
        <p class="archive-note">No features defined.</p>
      {:else}
        <table class="archive-table">
          <thead>
            <tr>
              <th>Slug</th>
              <th>Name</th>
              <th>Gate Type</th>
              <th>Gate Value</th>
              <th>Default</th>
              <th>Revocable</th>
              <th>Description</th>
              <th>Users</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {#each features as f (f.slug)}
              <tr class:dirty={dirtySlugs.has(f.slug)}>
                <td class="archive-mono"><code>{f.slug}</code></td>
                <td>
                  <input
                    type="text"
                    bind:value={f.name}
                    oninput={() => markDirty(f.slug)}
                    class="field-input"
                  />
                </td>
                <td>
                  <select
                    bind:value={f.gate_type}
                    onchange={() => markDirty(f.slug)}
                    class="field-select"
                  >
                    <option value="rank">rank</option>
                    <option value="trust">trust</option>
                    <option value="none">none</option>
                    <option value="feature">feature</option>
                  </select>
                </td>
                <td>
                  <input
                    type="number"
                    bind:value={f.gate_value}
                    oninput={() => markDirty(f.slug)}
                    class="field-number"
                    min="0"
                  />
                </td>
                <td class="center">
                  <input
                    type="checkbox"
                    bind:checked={f.is_default}
                    onchange={() => markDirty(f.slug)}
                  />
                </td>
                <td class="center">
                  <input
                    type="checkbox"
                    bind:checked={f.is_revocable}
                    onchange={() => markDirty(f.slug)}
                  />
                </td>
                <td>
                  <textarea
                    bind:value={f.description}
                    oninput={() => markDirty(f.slug)}
                    rows="2"
                    class="field-textarea"
                  ></textarea>
                </td>
                <td class="center stat-cell">{getEnabledCount(f.slug)}</td>
                <td>
                  {#if dirtySlugs.has(f.slug)}
                    <button
                      class="archive-btn"
                      type="button"
                      onclick={() => saveFeature(f.slug)}
                      disabled={saving}
                    >Save</button>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
        <p class="archive-note footnote">Changes are highlighted. Click "Save" per row or "Save N change(s)" at the top.</p>
      {/if}
    </div>
  </main>
{:else}
<div class="admin-page">
  <div class="page-header">
    <h1>⚙️ Feature Flags</h1>
    <div class="header-actions">
      <a class="btn btn-secondary btn-sm" href="/admin/features/analytics">📊 Analytics</a>
      {#if dirtySlugs.size > 0}
        <button class="btn btn-primary btn-sm" onclick={saveAll} disabled={saving}>
          {saving ? 'Saving…' : `💾 Save ${dirtySlugs.size} change(s)`}
        </button>
      {/if}
    </div>
  </div>

  {#if error}
    <div class="error-card">⚠️ {error}</div>
  {/if}
  {#if msg}
    <p class="ok">{msg}</p>
  {/if}

  {#if loading}
    <p class="muted">Loading features…</p>
  {:else}
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Slug</th>
            <th>Name</th>
            <th>Gate Type</th>
            <th>Gate Value</th>
            <th>Default</th>
            <th>Revocable</th>
            <th>Description</th>
            <th>Users</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each features as f (f.slug)}
            <tr class:dirty={dirtySlugs.has(f.slug)}>
              <td class="slug-cell"><code>{f.slug}</code></td>
              <td>
                <input
                  type="text"
                  bind:value={f.name}
                  oninput={() => markDirty(f.slug)}
                  class="field-input"
                />
              </td>
              <td>
                <select
                  bind:value={f.gate_type}
                  onchange={() => markDirty(f.slug)}
                  class="field-select"
                >
                  <option value="rank">rank</option>
                  <option value="trust">trust</option>
                  <option value="none">none</option>
                  <option value="feature">feature</option>
                </select>
              </td>
              <td>
                <input
                  type="number"
                  bind:value={f.gate_value}
                  oninput={() => markDirty(f.slug)}
                  class="field-number"
                  min="0"
                />
              </td>
              <td class="center">
                <input
                  type="checkbox"
                  bind:checked={f.is_default}
                  onchange={() => markDirty(f.slug)}
                />
              </td>
              <td class="center">
                <input
                  type="checkbox"
                  bind:checked={f.is_revocable}
                  onchange={() => markDirty(f.slug)}
                />
              </td>
              <td>
                <textarea
                  bind:value={f.description}
                  oninput={() => markDirty(f.slug)}
                  rows="2"
                  class="field-textarea"
                ></textarea>
              </td>
              <td class="center stat-cell">{getEnabledCount(f.slug)}</td>
              <td>
                {#if dirtySlugs.has(f.slug)}
                  <button
                    class="btn btn-primary btn-xs"
                    onclick={() => saveFeature(f.slug)}
                    disabled={saving}
                  >Save</button>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    <p class="muted footnote">Changes are highlighted. Click "Save" per row or "Save N change(s)" at the top.</p>
  {/if}
</div>
{/if}

<style>
  /* ── Archive mode ─────────────────────────────────────────── */
  .archive-main {
    max-width: var(--archive-max-width, 1100px);
    margin: 0 auto;
    padding: 0;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-content { max-width: 1040px; margin: 0 auto; padding: 1rem; }
  .archive-header {
    padding: 0.8rem 1rem 0.6rem;
    border-bottom: 2px solid var(--archive-link, #990000);
    background: var(--archive-bg, #ffffff);
  }
  .archive-page-title { font-size: 1.6em; font-weight: 700; margin: 0; }
  .archive-links { font-size: 0.92em; margin: 0.9em 0 0; }
  .archive-table { width: 100%; border-collapse: collapse; font-size: 0.88em; }
  .archive-table th,
  .archive-table td {
    text-align: left;
    padding: 0.45em 0.5em;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    vertical-align: top;
  }
  .archive-table th {
    font-weight: 700;
    font-size: 0.9em;
    border-bottom: 2px solid var(--archive-border, #dddddd);
    white-space: nowrap;
  }
  .archive-table tr.dirty { background: #fffbeb; }
  .archive-table td.center { text-align: center; }
  .archive-table .stat-cell { font-weight: 700; }
  .archive-mono { font-family: 'Courier New', Courier, monospace; white-space: nowrap; }
  .archive-table code {
    font-family: inherit;
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0 0.3em;
    font-size: 0.9em;
  }
  .field-input, .field-select, .field-number, .field-textarea {
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.95em;
    background: var(--archive-bg, #ffffff);
    box-sizing: border-box;
  }
  .field-input { width: 100%; padding: 0.25em 0.4em; }
  .field-textarea { width: 100%; resize: vertical; padding: 0.25em 0.4em; }
  .field-number { width: 60px; padding: 0.25em 0.4em; }
  .archive-btn {
    display: inline-block;
    padding: 0.25em 0.9em;
    font-size: 0.85em;
    font-weight: 700;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-btn-bg, #f5f5f5);
    color: var(--archive-text, #2a2a2a);
    cursor: pointer;
    font-family: inherit;
  }
  .archive-btn:hover:not(:disabled) { background: #eeeeee; }
  .archive-btn:disabled { opacity: 0.6; cursor: not-allowed; }
  .archive-error { color: #990000; font-weight: 700; font-size: 0.92em; }
  .archive-msg { color: #2e7d32; font-size: 0.92em; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.85em; }
  p.archive-note { margin: 0.75em 0; }
  .archive-note.footnote { margin-top: 0.9em; }

  /* ── Modern mode ──────────────────────────────────────────── */
  .admin-page { max-width: 1100px; margin: 0 auto; padding: 1.5rem; }
  .page-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  .page-header h1 { margin: 0; }
  .header-actions { display: flex; gap: 0.5rem; align-items: center; }
  .table-wrap { overflow-x: auto; }
  table { width: 100%; border-collapse: collapse; font-size: 0.88rem; }
  th { text-align: left; padding: 0.5rem 0.6rem; border-bottom: 2px solid var(--color-border, #ccc); white-space: nowrap; }
  td { padding: 0.4rem 0.6rem; border-bottom: 1px solid var(--color-border, #eee); vertical-align: top; }
  tr.dirty { background: #fffbeb; }
  .slug-cell { white-space: nowrap; }
  .slug-cell code { font-size: 0.82rem; background: var(--color-surface-2, #f3f4f6); padding: 0.15rem 0.35rem; border-radius: 4px; }
  .center { text-align: center; }
  .stat-cell { font-weight: 600; font-size: 0.9rem; }
  .field-input { width: 100%; padding: 0.3rem; border: 1px solid var(--color-border, #ddd); border-radius: 4px; box-sizing: border-box; }
  .field-select { padding: 0.3rem; border: 1px solid var(--color-border, #ddd); border-radius: 4px; }
  .field-number { width: 60px; padding: 0.3rem; border: 1px solid var(--color-border, #ddd); border-radius: 4px; }
  .field-textarea { width: 100%; padding: 0.3rem; border: 1px solid var(--color-border, #ddd); border-radius: 4px; box-sizing: border-box; font-size: 0.82rem; resize: vertical; }
  .muted { color: var(--color-text-muted, #888); }
  .footnote { font-size: 0.82rem; margin-top: 0.75rem; }
  .error-card { background: #fee2e2; color: #991b1b; padding: 0.75rem; border-radius: 8px; margin-bottom: 1rem; }
  .ok { color: var(--color-ok, #2e9e5b); margin-bottom: 0.5rem; }
  .btn { padding: 0.4rem 0.8rem; border-radius: 6px; border: 1px solid var(--color-border, #ccc); cursor: pointer; background: var(--color-surface, #fff); text-decoration: none; display: inline-block; }
  .btn-primary { background: var(--color-accent, #6366f1); color: #fff; border-color: var(--color-accent, #6366f1); }
  .btn-secondary { background: var(--color-surface-2, #f3f4f6); }
  .btn-sm { font-size: 0.82rem; padding: 0.3rem 0.6rem; }
  .btn-xs { font-size: 0.75rem; padding: 0.2rem 0.5rem; }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
