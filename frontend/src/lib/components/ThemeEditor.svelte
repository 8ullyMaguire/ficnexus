<script lang="ts">
  import type { ThemeTokens } from '$lib/themes/presets.js';
  import { presets } from '$lib/themes/presets.js';
  import { applyTheme, saveTheme, loadTheme } from '$lib/themes/apply.js';
  import { saveThemeToServer } from '$lib/api/theme.js';

  let current = $state<ThemeTokens>(loadTheme());
  let saveError = $state('');
  let saveOk = $state('');

  const FONT_OPTIONS = [
    { label: 'System Default', value: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif" },
    { label: 'Inter', value: "'Inter', sans-serif" },
    { label: 'Georgia (Serif)', value: "Georgia, 'Times New Roman', serif" },
    { label: 'Atkinson Hyperlegible', value: "'Atkinson Hyperlegible', sans-serif" },
  ];

  const READER_FONT_OPTIONS = [
    { label: 'Georgia', value: 'Georgia' },
    { label: 'System Serif', value: "Charter, 'Bitstream Charter', Georgia, serif" },
    { label: 'Literata', value: "'Literata', Georgia, serif" },
    { label: 'Atkinson Hyperlegible', value: "'Atkinson Hyperlegible', sans-serif" },
  ];

  // ── Apply whenever current changes ──────────────────────────────
  $effect(() => {
    applyTheme(current);
  });

  function applyPreset(slug: string) {
    const preset = presets[slug];
    if (preset) {
      current = { ...preset };
      persist();
    }
  }

  async function persist() {
    saveError = '';
    saveOk = '';
    try {
      saveTheme(current);
      await saveThemeToServer(current);
      saveOk = 'Theme saved';
      setTimeout(() => (saveOk = ''), 2000);
    } catch {
      saveError = 'Could not save to server (local copy kept)';
      setTimeout(() => (saveError = ''), 4000);
    }
  }

  // ── Export / Import ──────────────────────────────────────────────
  function exportTheme() {
    const blob = new Blob([JSON.stringify(current, null, 2)], {
      type: 'application/json',
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `ficnexus-theme-${current.name.toLowerCase().replace(/\s+/g, '-')}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }

  let importInput = $state<HTMLInputElement>();

  function importTheme() {
    importInput?.click();
  }

  function handleImportFile(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      try {
        const parsed = JSON.parse(reader.result as string) as Partial<ThemeTokens>;
        // Merge with defaults so every field is present.
        current = {
          ...current,
          ...parsed,
          name: parsed.name ?? 'Custom',
        } as ThemeTokens;
        persist();
      } catch {
        saveError = 'Invalid theme file';
        setTimeout(() => (saveError = ''), 3000);
      }
    };
    reader.readAsText(file);
    // Reset so the same file can be re-imported.
    if (importInput) importInput.value = '';
  }

  function resetToDefault() {
    current = { ...presets['default-dark'] };
    persist();
  }
</script>

<div class="theme-editor">
  <!-- Status messages -->
  {#if saveOk}
    <div class="toast success">{saveOk}</div>
  {/if}
  {#if saveError}
    <div class="toast error">{saveError}</div>
  {/if}

  <!-- ── Preset grid ─────────────────────────────────────────── -->
  <section class="section">
    <h3>Presets</h3>
    <div class="preset-grid">
      {#each Object.entries(presets) as [slug, preset]}
        <button
          class="preset-card"
          class:active={current.name === preset.name}
          type="button"
          onclick={() => applyPreset(slug)}
        >
          <span
            class="preset-swatch"
            style="background: {preset.bg}; border-color: {preset.accent};"
          >
            <span class="swatch-accent" style="background: {preset.accent};"></span>
          </span>
          <span class="preset-label">{preset.name}</span>
        </button>
      {/each}
    </div>
  </section>

  <!-- ── Colour controls ─────────────────────────────────────── -->
  <section class="section">
    <h3>Colours</h3>
    <div class="field-grid">
      <label class="field">
        <span class="field-label">Accent</span>
        <div class="color-row">
          <input type="color" bind:value={current.accent} onchange={() => persist()} />
          <input type="text" class="hex-input" bind:value={current.accent} onchange={() => persist()} />
        </div>
      </label>
      <label class="field">
        <span class="field-label">Background</span>
        <div class="color-row">
          <input type="color" bind:value={current.bg} onchange={() => persist()} />
          <input type="text" class="hex-input" bind:value={current.bg} onchange={() => persist()} />
        </div>
      </label>
      <label class="field">
        <span class="field-label">Surface</span>
        <div class="color-row">
          <input type="color" bind:value={current.surface} onchange={() => persist()} />
          <input type="text" class="hex-input" bind:value={current.surface} onchange={() => persist()} />
        </div>
      </label>
      <label class="field">
        <span class="field-label">Text</span>
        <div class="color-row">
          <input type="color" bind:value={current.text} onchange={() => persist()} />
          <input type="text" class="hex-input" bind:value={current.text} onchange={() => persist()} />
        </div>
      </label>
      <label class="field">
        <span class="field-label">Muted</span>
        <div class="color-row">
          <input type="color" bind:value={current.muted} onchange={() => persist()} />
          <input type="text" class="hex-input" bind:value={current.muted} onchange={() => persist()} />
        </div>
      </label>
    </div>
  </section>

  <!-- ── Typography ──────────────────────────────────────────── -->
  <section class="section">
    <h3>Typography</h3>
    <div class="field-grid">
      <label class="field">
        <span class="field-label">UI Font</span>
        <select bind:value={current.font} onchange={() => persist()}>
          {#each FONT_OPTIONS as opt}
            <option value={opt.value}>{opt.label}</option>
          {/each}
        </select>
      </label>
      <label class="field">
        <span class="field-label">Corner Radius</span>
        <input type="text" bind:value={current.radius} placeholder="10px" onchange={() => persist()} />
      </label>
    </div>
  </section>

  <!-- ── Density ─────────────────────────────────────────────── -->
  <section class="section">
    <h3>Density</h3>
    <div class="density-row">
      {#each (['compact', 'comfortable', 'spacious'] as const) as d}
        <button
          class="density-btn"
          class:active={current.density === d}
          type="button"
          onclick={() => { current.density = d; persist(); }}
        >
          {d.charAt(0).toUpperCase() + d.slice(1)}
        </button>
      {/each}
    </div>
  </section>

  <!-- ── Reader settings ─────────────────────────────────────── -->
  <section class="section">
    <h3>Reader</h3>
    <div class="field-grid">
      <label class="field">
        <span class="field-label">Reader Font</span>
        <select bind:value={current.readerFont} onchange={() => persist()}>
          {#each READER_FONT_OPTIONS as opt}
            <option value={opt.value}>{opt.label}</option>
          {/each}
        </select>
      </label>
      <label class="field">
        <span class="field-label">Max Width</span>
        <input type="text" bind:value={current.readerWidth} placeholder="700px" onchange={() => persist()} />
      </label>
      <label class="field">
        <span class="field-label">Line Height</span>
        <input type="text" bind:value={current.readerLineHeight} placeholder="1.8" onchange={() => persist()} />
      </label>
    </div>
  </section>

  <!-- ── Live preview ────────────────────────────────────────── -->
  <section class="section">
    <h3>Preview</h3>
    <div
      class="preview"
      style="
        background: {current.bg};
        color: {current.text};
        border-color: {current.surface};
        border-radius: {current.radius};
        font-family: {current.font};
      "
    >
      <div class="preview-card" style="background: {current.surface}; border-radius: {current.radius};">
        <p class="preview-muted" style="color: {current.muted};">Muted text sample</p>
        <p>Regular body text in your chosen theme. The quick brown fox jumps over the lazy dog.</p>
        <a href="#!" style="color: {current.accent};">Link in accent colour</a>
      </div>
      <div
        class="preview-reader"
        style="
          font-family: {current.readerFont};
          max-width: {current.readerWidth};
          line-height: {current.readerLineHeight};
          margin-top: 0.75rem;
        "
      >
        <p style="color: {current.text};">
          Reader view preview: <em>Alice was beginning to get very tired of sitting by her sister on the bank, and of having nothing to do.</em>
        </p>
      </div>
    </div>
  </section>

  <!-- ── Import / Export / Reset ──────────────────────────────── -->
  <!-- ── Forum Theme ────────────────────────────────────────── -->
  <section class="section">
    <h3>Forum Theme</h3>
    <p class="muted" style="margin-bottom: 0.5rem; font-size: 0.85rem;">Customize forum post and category colors.</p>
    <div class="field-row">
      <label>
        Post Background
        <input type="color" value={current.forum?.postBg ?? "#161b22"}
          oninput={(e) => {
            if (!current.forum) current.forum = {};
            current.forum.postBg = e.currentTarget.value;
          }} />
      </label>
      <label>
        Post Border
        <input type="color" value={current.forum?.postBorder ?? "#21262d"}
          oninput={(e) => {
            if (!current.forum) current.forum = {};
            current.forum.postBorder = e.currentTarget.value;
          }} />
      </label>
    </div>
    <div class="field-row">
      <label>
        Mod Highlight
        <input type="color" value={current.forum?.modHighlight ?? "#1f6feb"}
          oninput={(e) => {
            if (!current.forum) current.forum = {};
            current.forum.modHighlight = e.currentTarget.value;
          }} />
      </label>
      <label>
        OP Highlight
        <input type="color" value={current.forum?.opHighlight ?? "#238636"}
          oninput={(e) => {
            if (!current.forum) current.forum = {};
            current.forum.opHighlight = e.currentTarget.value;
          }} />
      </label>
    </div>
  </section>
  <section class="section actions-row">
    <button class="btn btn-secondary" type="button" onclick={exportTheme}>
      ⬇ Export JSON
    </button>
    <button class="btn btn-secondary" type="button" onclick={importTheme}>
      ⬆ Import JSON
    </button>
    <input
      bind:this={importInput}
      type="file"
      accept=".json"
      class="sr-only"
      onchange={handleImportFile}
    />
    <button class="btn btn-secondary" type="button" onclick={resetToDefault}>
      ↺ Reset to Default
    </button>
  </section>
</div>

<style>
  .theme-editor {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  .section h3 {
    margin: 0 0 0.75rem;
    font-size: 1rem;
    font-weight: 700;
    color: var(--color-text);
  }

  /* ── Preset grid ─────────────────────────────────────────── */
  .preset-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
    gap: 0.6rem;
  }
  .preset-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.4rem;
    padding: 0.6rem;
    border: 2px solid transparent;
    border-radius: 8px;
    background: var(--color-surface-2);
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
  }
  .preset-card:hover {
    background: var(--color-border);
  }
  .preset-card.active {
    border-color: var(--color-primary);
    background: var(--color-surface);
  }
  .preset-swatch {
    width: 48px;
    height: 32px;
    border-radius: 6px;
    border: 1px solid var(--color-border);
    display: flex;
    align-items: flex-end;
    padding: 3px;
  }
  .swatch-accent {
    width: 100%;
    height: 6px;
    border-radius: 3px;
  }
  .preset-label {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text);
    text-align: center;
  }

  /* ── Fields ──────────────────────────────────────────────── */
  .field-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 0.75rem;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .field-label {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-muted);
  }
  .color-row {
    display: flex;
    gap: 0.4rem;
    align-items: center;
  }
  .color-row input[type='color'] {
    width: 2.2rem;
    height: 2.2rem;
    padding: 2px;
    border-radius: 6px;
    border: 1px solid var(--color-border);
    background: var(--color-surface-2);
    cursor: pointer;
  }
  .hex-input {
    flex: 1;
    min-width: 0;
    padding: 0.35rem 0.5rem;
    font-size: 0.85rem;
    font-family: var(--mono, ui-monospace, monospace);
  }
  select {
    padding: 0.4rem 0.6rem;
    border-radius: 6px;
    border: 1px solid var(--color-border);
    background: var(--color-surface-2);
    color: var(--color-text);
    font-size: 0.85rem;
  }

  /* ── Density buttons ─────────────────────────────────────── */
  .density-row {
    display: flex;
    gap: 0.5rem;
  }
  .density-btn {
    padding: 0.4rem 0.9rem;
    border-radius: 6px;
    border: 1px solid var(--color-border);
    background: var(--color-surface-2);
    color: var(--color-text);
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s, border-color 0.15s;
  }
  .density-btn:hover {
    background: var(--color-border);
  }
  .density-btn.active {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: #fff;
  }

  /* ── Preview ─────────────────────────────────────────────── */
  .preview {
    padding: 1rem;
    border: 1px solid var(--color-border);
  }
  .preview-card {
    padding: 0.8rem;
    border: 1px solid var(--color-border);
  }
  .preview-card p {
    margin: 0.3rem 0;
  }
  .preview-muted {
    font-size: 0.85rem;
  }
  .preview-reader {
    font-size: 0.95rem;
  }
  .preview-reader p {
    margin: 0;
  }

  /* ── Actions ─────────────────────────────────────────────── */
  .actions-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  .actions-row .btn {
    font-size: 0.85rem;
  }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  /* ── Toast ───────────────────────────────────────────────── */
  .toast {
    position: fixed;
    bottom: 1.5rem;
    right: 1.5rem;
    padding: 0.6rem 1rem;
    border-radius: 8px;
    font-size: 0.85rem;
    font-weight: 600;
    z-index: 100;
    animation: toast-in 0.2s ease-out;
  }
  .toast.success {
    background: var(--color-success, #22c55e);
    color: #fff;
  }
  .toast.error {
    background: var(--color-error, #ef4444);
    color: #fff;
  }
  @keyframes toast-in {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
