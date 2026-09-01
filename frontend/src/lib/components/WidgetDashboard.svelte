<script lang="ts">
  import { onMount } from 'svelte';
  import {
    fetchLayout,
    saveLayout,
    fetchWidgets,
    fetchFeatures,
    type LayoutItem,
    type WidgetInfo,
    type Feature,
  } from '$lib/api/layouts';
  import {
    getWidgetComponent,
    getAvailableWidgets,
    defaultLayout,
  } from '$lib/components/WidgetRegistry';
  import { auth } from '$lib/stores/auth.svelte';

  let { page = 'home', editable = false }: { page?: string; editable?: boolean } = $props();

  // ── State ──────────────────────────────────────────────────────────────
  let layout = $state<LayoutItem[]>([]);
  let allWidgets = $state<WidgetInfo[]>([]);
  let features = $state<Feature[]>([]);
  let loading = $state(true);
  let saving = $state(false);
  let showPicker = $state(false);
  let dirty = $state(false);

  // Widgets visible after feature/rank filtering
  const availableWidgets = $derived(
    getAvailableWidgets(allWidgets, features, auth.level),
  );

  // Slugs already on the dashboard
  const placedSlugs = $derived(new Set(layout.map((l) => l.widget)));

  // Widgets available to add (not yet placed)
  const addableWidgets = $derived(
    availableWidgets.filter((w) => !placedSlugs.has(w.slug)),
  );

  // ── Mount ──────────────────────────────────────────────────────────────
  onMount(async () => {
    const [layoutRes, widgetsRes, featuresRes] = await Promise.allSettled([
      fetchLayout(page),
      fetchWidgets(),
      fetchFeatures(),
    ]);

    if (layoutRes.status === 'fulfilled') {
      layout = layoutRes.value;
    }
    if (widgetsRes.status === 'fulfilled') {
      allWidgets = widgetsRes.value;
    }
    if (featuresRes.status === 'fulfilled') {
      features = featuresRes.value;
    }

    // If no saved layout exists, create a default from available widgets
    if (layout.length === 0 && availableWidgets.length > 0) {
      layout = defaultLayout(availableWidgets);
      dirty = true;
    }

    loading = false;
  });

  // ── Helpers ────────────────────────────────────────────────────────────

  function widgetMeta(slug: string): WidgetInfo | undefined {
    return allWidgets.find((w) => w.slug === slug);
  }

  function addWidget(slug: string) {
    const nextPos = layout.length > 0
      ? Math.max(...layout.map((l) => l.pos)) + 1
      : 0;
    layout = [...layout, { widget: slug, size: 'half', pos: nextPos }];
    dirty = true;
    showPicker = false;
  }

  function removeWidget(slug: string) {
    layout = layout.filter((l) => l.widget !== slug).map((l, i) => ({ ...l, pos: i }));
    dirty = true;
  }

  function cycleSize(slug: string) {
    const sizes: Array<'full' | 'half' | 'quarter'> = ['full', 'half', 'quarter'];
    layout = layout.map((l) => {
      if (l.widget !== slug) return l;
      const idx = sizes.indexOf(l.size);
      const next = sizes[(idx + 1) % sizes.length];
      return { ...l, size: next };
    });
    dirty = true;
  }

  async function handleSave() {
    saving = true;
    try {
      await saveLayout(page, layout);
      dirty = false;
    } catch (e) {
      console.error('Failed to save layout', e);
    } finally {
      saving = false;
    }
  }

  // Sort layout by pos for rendering
  const sortedLayout = $derived([...layout].sort((a, b) => a.pos - b.pos));

  // ── Grid CSS class helper ──────────────────────────────────────────────
  function gridClass(size: string): string {
    switch (size) {
      case 'full': return 'widget-full';
      case 'half': return 'widget-half';
      case 'quarter': return 'widget-quarter';
      default: return 'widget-half';
    }
  }
</script>

{#if loading}
  <div class="widget-dashboard skeleton">
    <span class="spinner"></span> Loading dashboard…
  </div>
{:else}
  <div class="widget-dashboard">
    {#if editable}
      <div class="dashboard-toolbar">
        {#if dirty}
          <button class="btn btn-save" type="button" onclick={handleSave} disabled={saving}>
            {#if saving}<span class="spinner"></span> Saving…{:else}💾 Save Layout{/if}
          </button>
        {/if}
        <button
          class="btn btn-add"
          type="button"
          onclick={() => (showPicker = !showPicker)}
        >
          {showPicker ? '✕ Close' : '＋ Add Widget'}
        </button>
      </div>

      {#if showPicker && addableWidgets.length > 0}
        <div class="widget-picker card">
          <h3 class="picker-title">Available Widgets</h3>
          <div class="picker-grid">
            {#each addableWidgets as w (w.slug)}
              <button class="picker-item" type="button" onclick={() => addWidget(w.slug)}>
                <span class="picker-icon">{w.icon}</span>
                <span class="picker-name">{w.name}</span>
                <span class="picker-desc muted">{w.description}</span>
              </button>
            {/each}
          </div>
        </div>
      {:else if showPicker}
        <div class="widget-picker card">
          <p class="muted">All available widgets are already on your dashboard.</p>
        </div>
      {/if}
    {/if}

    {#if sortedLayout.length === 0}
      <div class="card empty-widget">
        <p class="muted">No widgets configured yet.</p>
        {#if editable}
          <p class="muted">Click <strong>＋ Add Widget</strong> to get started.</p>
        {/if}
      </div>
    {:else}
      <div class="widget-grid">
        {#each sortedLayout as item (item.widget)}
          {@const meta = widgetMeta(item.widget)}
          {@const Comp = getWidgetComponent(item.widget)}
          <div class="widget-cell {gridClass(item.size)}">
            <div class="widget-card card">
              {#if editable}
                <div class="widget-controls">
                  <button
                    class="ctrl-btn"
                    type="button"
                    title="Drag handle (visual only)"
                    aria-label="Drag handle for {item.widget}"
                  >⋮⋮</button>
                  <button
                    class="ctrl-btn"
                    type="button"
                    title="Cycle size: {item.size}"
                    onclick={() => cycleSize(item.widget)}
                    aria-label="Change size for {item.widget}"
                  >{@html item.size === 'full' ? '⬛' : item.size === 'half' ? '◧' : '◻'}</button>
                  <button
                    class="ctrl-btn ctrl-remove"
                    type="button"
                    title="Remove widget"
                    onclick={() => removeWidget(item.widget)}
                    aria-label="Remove {item.widget}"
                  >✕</button>
                </div>
              {/if}

              {#if Comp}
                <Comp widget={meta ?? { slug: item.widget, name: item.widget, description: '', min_rank: 0, icon: '📦' }} />
              {:else if meta}
                <div class="widget-placeholder">
                  <span class="wp-icon">{meta.icon}</span>
                  <span class="wp-name">{meta.name}</span>
                  <span class="wp-desc muted">{meta.description}</span>
                </div>
              {:else}
                <div class="widget-placeholder">
                  <span class="wp-icon">📦</span>
                  <span class="wp-name">{item.widget}</span>
                  <span class="wp-desc muted">Widget not found in registry</span>
                </div>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  /* ── Dashboard shell ─────────────────────────────────────────────── */
  .widget-dashboard {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  /* ── Toolbar ─────────────────────────────────────────────────────── */
  .dashboard-toolbar {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .btn-save,
  .btn-add {
    padding: 0.45rem 0.9rem;
    font-size: 0.85rem;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface-2);
    color: var(--color-text);
    transition: background 0.15s, border-color 0.15s;
  }
  .btn-save:hover { background: var(--color-primary); border-color: var(--color-primary); color: #fff; }
  .btn-add:hover { border-color: var(--color-primary); color: var(--color-primary); }
  .btn-save:disabled { opacity: 0.5; cursor: not-allowed; }

  /* ── Widget picker panel ─────────────────────────────────────────── */
  .widget-picker { padding: 1rem; }
  .picker-title { margin: 0 0 0.6rem; font-size: 0.95rem; }
  .picker-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 0.5rem;
  }
  .picker-item {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.15rem;
    padding: 0.7rem 0.8rem;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
    color: var(--color-text);
    text-align: left;
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
  }
  .picker-item:hover { border-color: var(--color-primary); background: var(--color-surface); }
  .picker-icon { font-size: 1.3rem; }
  .picker-name { font-weight: 600; font-size: 0.88rem; }
  .picker-desc { font-size: 0.78rem; line-height: 1.3; }

  /* ── Widget grid ─────────────────────────────────────────────────── */
  .widget-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1rem;
  }
  .widget-full  { grid-column: 1 / -1; }
  .widget-half  { grid-column: span 2; }
  .widget-quarter { grid-column: span 1; }

  @media (max-width: 768px) {
    .widget-grid { grid-template-columns: 1fr; }
    .widget-full,
    .widget-half,
    .widget-quarter { grid-column: 1 / -1; }
  }

  /* ── Individual widget card ──────────────────────────────────────── */
  .widget-card {
    position: relative;
    min-height: 4rem;
    padding: 1rem;
  }

  /* ── Edit controls ───────────────────────────────────────────────── */
  .widget-controls {
    position: absolute;
    top: 0.4rem;
    right: 0.4rem;
    display: flex;
    gap: 0.25rem;
  }
  .ctrl-btn {
    width: 1.6rem;
    height: 1.6rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
    color: var(--color-muted);
    font-size: 0.75rem;
    line-height: 1;
    cursor: pointer;
    transition: color 0.15s, border-color 0.15s;
  }
  .ctrl-btn:hover { color: var(--color-text); border-color: var(--color-primary); }
  .ctrl-remove:hover { color: var(--color-error); border-color: var(--color-error); }

  /* ── Placeholder for unregistered widgets ────────────────────────── */
  .widget-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.3rem;
    padding: 1rem;
    text-align: center;
    min-height: 3rem;
  }
  .wp-icon { font-size: 1.8rem; }
  .wp-name { font-weight: 600; font-size: 0.92rem; }
  .wp-desc { font-size: 0.8rem; }

  .empty-widget { padding: 2rem; text-align: center; }
  .empty-widget p { margin: 0.3rem 0; }

  .skeleton {
    padding: 2rem;
    text-align: center;
  }
</style>
