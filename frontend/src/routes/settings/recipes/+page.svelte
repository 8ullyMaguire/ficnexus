<script lang="ts">
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';
  import { auth } from '$lib/stores/auth.svelte';
  import {
    listRecipes,
    createRecipe,
    updateRecipe,
    deleteRecipe,
    activateRecipe,
    getActiveRecipe,
    browseGallery,
    installRecipe,
    publishRecipe,
    type Recipe,
    type RecipeResponse,
  } from '$lib/api/recipes';

  const uiMode = $derived(getPref('uiMode'));

  // ── State ──────────────────────────────────────────────────────────────
  let loading = $state(true);
  let error = $state('');
  let saved = $state('');

  // Tabs: 'mine' | 'gallery'
  let tab = $state<'mine' | 'gallery'>('mine');

  // User's recipes
  let recipes = $state<Recipe[]>([]);
  let activeRecipeId = $state<number | null>(null);

  // Gallery
  let galleryRecipes = $state<Recipe[]>([]);
  let galleryLoading = $state(false);

  // ── Create form ────────────────────────────────────────────────────────
  let showCreateForm = $state(false);
  let newName = $state('');
  let newDescription = $state('');
  let newBlendCooccur = $state(0.4);
  let newBlendTagGraph = $state(0.3);
  let newBlendEmbeddings = $state(0.3);
  let newMinWords = $state(10000);
  let newExcludeWarnings = $state<string[]>([]);
  let newCuratorPrior = $state(0.1);
  let creating = $state(false);

  // Editing
  let editingId = $state<number | null>(null);
  let editName = $state('');
  let editDescription = $state('');
  let editBlendCooccur = $state(0.4);
  let editBlendTagGraph = $state(0.3);
  let editBlendEmbeddings = $state(0.3);
  let editMinWords = $state(10000);
  let editExcludeWarnings = $state<string[]>([]);
  let editCuratorPrior = $state(0.1);
  let savingEdit = $state(false);

  const WARNING_TAGS = [
    'Major Character Death',
    'Graphic Violence',
    'Underage',
    'Non-Con',
    'Dub-Con',
  ];

  // ── Helpers ────────────────────────────────────────────────────────────
  function blendTotal(cooccur: number, tagGraph: number, embeddings: number): number {
    return cooccur + tagGraph + embeddings;
  }

  function normalizeBlend(cooccur: number, tagGraph: number, embeddings: number) {
    const total = cooccur + tagGraph + embeddings;
    if (total === 0) return { cooccur: 0.33, tag_graph: 0.33, embeddings: 0.34 };
    return {
      cooccur: cooccur / total,
      tag_graph: tagGraph / total,
      embeddings: embeddings / total,
    };
  }

  function formatDate(iso: string): string {
    try {
      return new Date(iso).toLocaleDateString();
    } catch {
      return iso;
    }
  }

  // ── Load data ──────────────────────────────────────────────────────────
  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn) {
      loading = false;
      return;
    }
    await loadRecipes();
    await loadActiveRecipe();
  });

  async function loadRecipes() {
    loading = true;
    error = '';
    try {
      const res = await listRecipes();
      if (res.err === 0 && res.recipes) {
        recipes = res.recipes;
      }
    } catch {
      error = 'Failed to load recipes.';
    } finally {
      loading = false;
    }
  }

  async function loadActiveRecipe() {
    try {
      const res = await getActiveRecipe();
      if (res.err === 0 && res.recipe) {
        activeRecipeId = res.recipe.id;
      } else {
        activeRecipeId = null;
      }
    } catch {
      activeRecipeId = null;
    }
  }

  async function loadGallery() {
    galleryLoading = true;
    try {
      const res = await browseGallery(30, 0);
      if (res.err === 0 && res.recipes) {
        galleryRecipes = res.recipes;
      }
    } catch {
      error = 'Failed to load gallery.';
    } finally {
      galleryLoading = false;
    }
  }

  // ── Create recipe ──────────────────────────────────────────────────────
  async function handleCreate() {
    if (!newName.trim()) {
      error = 'Recipe name is required.';
      return;
    }
    const blendTotal_ = blendTotal(newBlendCooccur, newBlendTagGraph, newBlendEmbeddings);
    if (blendTotal_ === 0) {
      error = 'Strategy weights must sum to > 0.';
      return;
    }
    creating = true;
    error = '';
    saved = '';
    try {
      const blend = normalizeBlend(newBlendCooccur, newBlendTagGraph, newBlendEmbeddings);
      const filters: Record<string, unknown> = {};
      if (newMinWords > 0) filters.min_words = newMinWords;
      if (newExcludeWarnings.length > 0) filters.exclude_warnings = newExcludeWarnings;

      const res = await createRecipe({
        name: newName.trim(),
        description: newDescription.trim() || undefined,
        blend,
        filters,
        curator_prior: newCuratorPrior,
      });
      if (res.err === 0) {
        saved = 'Recipe created!';
        showCreateForm = false;
        newName = '';
        newDescription = '';
        newBlendCooccur = 0.4;
        newBlendTagGraph = 0.3;
        newBlendEmbeddings = 0.3;
        newMinWords = 10000;
        newExcludeWarnings = [];
        newCuratorPrior = 0.1;
        await loadRecipes();
      } else {
        error = res.message || 'Failed to create recipe.';
      }
    } catch {
      error = 'Failed to create recipe.';
    } finally {
      creating = false;
    }
  }

  // ── Activate recipe ────────────────────────────────────────────────────
  async function handleActivate(id: number) {
    error = '';
    try {
      const res = await activateRecipe(id);
      if (res.err === 0) {
        activeRecipeId = id;
        saved = 'Recipe activated!';
        await loadRecipes();
      } else {
        error = res.message || 'Failed to activate recipe.';
      }
    } catch {
      error = 'Failed to activate recipe.';
    }
  }

  // ── Delete recipe ──────────────────────────────────────────────────────
  async function handleDelete(id: number) {
    if (!confirm('Delete this recipe?')) return;
    error = '';
    try {
      const res = await deleteRecipe(id);
      if (res.err === 0) {
        saved = 'Recipe deleted.';
        if (activeRecipeId === id) activeRecipeId = null;
        await loadRecipes();
      } else {
        error = res.message || 'Failed to delete recipe.';
      }
    } catch {
      error = 'Failed to delete recipe.';
    }
  }

  // ── Publish/unpublish ──────────────────────────────────────────────────
  async function handlePublish(id: number, currentPublic: boolean) {
    error = '';
    try {
      const res = await publishRecipe(id, !currentPublic);
      if (res.err === 0) {
        saved = currentPublic ? 'Recipe unpublished.' : 'Recipe published to gallery!';
        await loadRecipes();
      } else {
        error = res.message || 'Failed to update publish status.';
      }
    } catch {
      error = 'Failed to update publish status.';
    }
  }

  // ── Install from gallery ───────────────────────────────────────────────
  async function handleInstall(id: number) {
    error = '';
    try {
      const res = await installRecipe(id);
      if (res.err === 0) {
        saved = 'Recipe installed to your collection!';
        await loadRecipes();
      } else {
        error = res.message || 'Failed to install recipe.';
      }
    } catch {
      error = 'Failed to install recipe.';
    }
  }

  // ── Edit recipe ────────────────────────────────────────────────────────
  function startEdit(recipe: Recipe) {
    editingId = recipe.id;
    editName = recipe.name;
    editDescription = recipe.description || '';
    // Extract blend values
    const b = recipe.blend as Record<string, number>;
    editBlendCooccur = b.cooccur ?? 0.4;
    editBlendTagGraph = b.tag_graph ?? 0.3;
    editBlendEmbeddings = b.embeddings ?? 0.3;
    // Extract filters
    const f = recipe.filters as Record<string, unknown>;
    editMinWords = (typeof f.min_words === 'number' ? f.min_words : 10000) as number;
    editExcludeWarnings = Array.isArray(f.exclude_warnings)
      ? (f.exclude_warnings as string[])
      : [];
    editCuratorPrior = recipe.curator_prior;
    saved = '';
    error = '';
  }

  function cancelEdit() {
    editingId = null;
  }

  async function handleUpdate() {
    if (editingId === null) return;
    if (!editName.trim()) {
      error = 'Recipe name is required.';
      return;
    }
    savingEdit = true;
    error = '';
    try {
      const blend = normalizeBlend(editBlendCooccur, editBlendTagGraph, editBlendEmbeddings);
      const filters: Record<string, unknown> = {};
      if (editMinWords > 0) filters.min_words = editMinWords;
      if (editExcludeWarnings.length > 0) filters.exclude_warnings = editExcludeWarnings;

      const res = await updateRecipe(editingId, {
        name: editName.trim(),
        description: editDescription.trim() || null,
        blend,
        filters,
        curator_prior: editCuratorPrior,
      });
      if (res.err === 0) {
        saved = 'Recipe updated!';
        editingId = null;
        await loadRecipes();
      } else {
        error = res.message || 'Failed to update recipe.';
      }
    } catch {
      error = 'Failed to update recipe.';
    } finally {
      savingEdit = false;
    }
  }
</script>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">Recipe Builder</h1>
      <p class="archive-muted">Customise how recommendations are blended for you.</p>
    </header>

    {#if !auth.isLoggedIn}
      <blockquote class="archive-summary">Login to create and manage recipes.</blockquote>
    {:else}
      {#if error}
        <p class="archive-error">⚠️ {error}</p>
      {/if}
      {#if saved}
        <p class="archive-success">✅ {saved}</p>
      {/if}

      <!-- ── Tab bar ──────────────────────────────────────────────── -->
      <ul class="archive-tabs" role="tablist">
        <li class:current={tab === 'mine'} role="presentation">
          <button role="tab" aria-selected={tab === 'mine'} type="button" onclick={() => { tab = 'mine'; }}>
            My Recipes
          </button>
        </li>
        <li class:current={tab === 'gallery'} role="presentation">
          <button role="tab" aria-selected={tab === 'gallery'} type="button" onclick={() => { tab = 'gallery'; loadGallery(); }}>
            Gallery
          </button>
        </li>
      </ul>

      {#if tab === 'mine'}
        <!-- ── My Recipes tab ──────────────────────────────────────── -->
        {#if loading}
          <p class="archive-muted"><span class="spinner"></span> Loading…</p>
        {:else}
          <fieldset class="archive-fieldset">
            <legend class="archive-legend">My Recipes</legend>
            <!-- Create button -->
            <div class="archive-actions">
              <button class="archive-btn archive-btn-primary" onclick={() => { showCreateForm = !showCreateForm; }}>
                {showCreateForm ? 'Cancel' : '+ New Recipe'}
              </button>
            </div>

            <!-- Create form -->
            {#if showCreateForm}
              <form class="archive-form" onsubmit={(e) => { e.preventDefault(); handleCreate(); }}>
                <dl class="archive-dl">
                  <div class="dl-row">
                    <dt><label for="recipe-name">Name</label></dt>
                    <dd><input id="recipe-name" class="archive-input" type="text" bind:value={newName} placeholder="e.g. Dark &amp; Angsty" /></dd>
                  </div>
                  <div class="dl-row">
                    <dt><label for="recipe-desc">Description</label></dt>
                    <dd><textarea id="recipe-desc" class="archive-input" bind:value={newDescription} placeholder="What this recipe is good for…" rows="2"></textarea></dd>
                  </div>
                </dl>

                <fieldset class="archive-fieldset nested">
                  <legend class="archive-legend">Strategy Blend</legend>
                  <p class="archive-muted small">Weights normalise to sum to 1.0</p>
                  <dl class="archive-dl">
                    <div class="dl-row">
                      <dt><label for="blend-cooccur">Co-occurrence</label></dt>
                      <dd class="archive-actions-inline">
                        <input id="blend-cooccur" type="range" min="0" max="1" step="0.05" bind:value={newBlendCooccur} />
                        <span class="val">{newBlendCooccur.toFixed(2)}</span>
                      </dd>
                    </div>
                    <div class="dl-row">
                      <dt><label for="blend-taggraph">Tag Graph</label></dt>
                      <dd class="archive-actions-inline">
                        <input id="blend-taggraph" type="range" min="0" max="1" step="0.05" bind:value={newBlendTagGraph} />
                        <span class="val">{newBlendTagGraph.toFixed(2)}</span>
                      </dd>
                    </div>
                    <div class="dl-row">
                      <dt><label for="blend-embeddings">Embeddings</label></dt>
                      <dd class="archive-actions-inline">
                        <input id="blend-embeddings" type="range" min="0" max="1" step="0.05" bind:value={newBlendEmbeddings} />
                        <span class="val">{newBlendEmbeddings.toFixed(2)}</span>
                      </dd>
                    </div>
                  </dl>
                  <p class="archive-muted small">
                    Total: {blendTotal(newBlendCooccur, newBlendTagGraph, newBlendEmbeddings).toFixed(2)}
                  </p>
                </fieldset>

                <fieldset class="archive-fieldset nested">
                  <legend class="archive-legend">Filters</legend>
                  <dl class="archive-dl">
                    <div class="dl-row">
                      <dt><label for="recipe-minwords">Min. word count</label></dt>
                      <dd><input id="recipe-minwords" class="archive-input" type="number" min="0" step="1000" bind:value={newMinWords} /></dd>
                    </div>
                  </dl>
                  <p class="archive-muted small">Exclude warnings:</p>
                  <div class="warning-tags">
                    {#each WARNING_TAGS as tag}
                      <label class="archive-checkbox-label">
                        <input
                          type="checkbox"
                          checked={newExcludeWarnings.includes(tag)}
                          onchange={() => {
                            if (newExcludeWarnings.includes(tag)) {
                              newExcludeWarnings = newExcludeWarnings.filter(t => t !== tag);
                            } else {
                              newExcludeWarnings = [...newExcludeWarnings, tag];
                            }
                          }}
                        />
                        {tag}
                      </label>
                    {/each}
                  </div>
                </fieldset>

                <dl class="archive-dl">
                  <div class="dl-row">
                    <dt><label for="recipe-curator">Curator prior ({newCuratorPrior.toFixed(2)})</label></dt>
                    <dd><input id="recipe-curator" type="range" min="0" max="1" step="0.05" bind:value={newCuratorPrior} /></dd>
                  </div>
                </dl>

                <button class="archive-btn archive-btn-primary" type="submit" disabled={creating}>
                  {#if creating}<span class="spinner"></span>{:else}Create Recipe{/if}
                </button>
              </form>
            {/if}

            <!-- Recipe list -->
            {#if recipes.length === 0}
              <p class="archive-muted">No recipes yet. Create one to customise your recommendations!</p>
            {:else}
              <ul class="archive-pref-list">
                {#each recipes as recipe (recipe.id)}
                  {#if editingId === recipe.id}
                    <!-- Edit mode -->
                    <li class="archive-pref-item editing">
                      <form class="archive-form edit-form" onsubmit={(e) => { e.preventDefault(); handleUpdate(); }}>
                        <dl class="archive-dl">
                          <div class="dl-row">
                            <dt><label for="edit-name">Name</label></dt>
                            <dd><input id="edit-name" class="archive-input" type="text" bind:value={editName} /></dd>
                          </div>
                          <div class="dl-row">
                            <dt><label for="edit-desc">Description</label></dt>
                            <dd><textarea id="edit-desc" class="archive-input" bind:value={editDescription} rows="2"></textarea></dd>
                          </div>
                          <div class="dl-row">
                            <dt><label for="edit-cooccur">Co-occurrence</label></dt>
                            <dd class="archive-actions-inline">
                              <input id="edit-cooccur" type="range" min="0" max="1" step="0.05" bind:value={editBlendCooccur} />
                              <span class="val">{editBlendCooccur.toFixed(2)}</span>
                            </dd>
                          </div>
                          <div class="dl-row">
                            <dt><label for="edit-taggraph">Tag Graph</label></dt>
                            <dd class="archive-actions-inline">
                              <input id="edit-taggraph" type="range" min="0" max="1" step="0.05" bind:value={editBlendTagGraph} />
                              <span class="val">{editBlendTagGraph.toFixed(2)}</span>
                            </dd>
                          </div>
                          <div class="dl-row">
                            <dt><label for="edit-embeddings">Embeddings</label></dt>
                            <dd class="archive-actions-inline">
                              <input id="edit-embeddings" type="range" min="0" max="1" step="0.05" bind:value={editBlendEmbeddings} />
                              <span class="val">{editBlendEmbeddings.toFixed(2)}</span>
                            </dd>
                          </div>
                          <div class="dl-row">
                            <dt><label for="edit-minwords">Min. word count</label></dt>
                            <dd><input id="edit-minwords" class="archive-input" type="number" min="0" step="1000" bind:value={editMinWords} /></dd>
                          </div>
                          <div class="dl-row">
                            <dt><label for="edit-curator">Curator prior ({editCuratorPrior.toFixed(2)})</label></dt>
                            <dd><input id="edit-curator" type="range" min="0" max="1" step="0.05" bind:value={editCuratorPrior} /></dd>
                          </div>
                        </dl>
                        <div class="archive-actions-inline">
                          <button class="archive-btn archive-btn-primary" type="submit" disabled={savingEdit}>
                            {#if savingEdit}<span class="spinner"></span>{:else}Save{/if}
                          </button>
                          <button class="archive-btn" type="button" onclick={cancelEdit}>Cancel</button>
                        </div>
                      </form>
                    </li>
                  {:else}
                    <!-- View mode -->
                    <li class="archive-pref-item" class:active={activeRecipeId === recipe.id}>
                      <div class="recipe-main">
                        <span class="archive-pref-label">
                          {recipe.name}
                          {#if activeRecipeId === recipe.id}
                            <span class="badge badge-active">Active</span>
                          {/if}
                          {#if recipe.is_public}
                            <span class="badge badge-public">Public</span>
                          {/if}
                        </span>
                        {#if recipe.description}
                          <span class="archive-muted"> — {recipe.description}</span>
                        {/if}
                        <span class="archive-muted small">Updated {formatDate(recipe.updated_at)}</span>
                        <span class="recipe-meta">
                          Co-occur: {((recipe.blend as Record<string, number>).cooccur ?? 0).toFixed(2)}
                          · Tag Graph: {((recipe.blend as Record<string, number>).tag_graph ?? 0).toFixed(2)}
                          · Embeddings: {((recipe.blend as Record<string, number>).embeddings ?? 0).toFixed(2)}
                          · Curator: {recipe.curator_prior.toFixed(2)}
                          {#if recipe.is_public}
                            · Installs: {recipe.installs}
                          {/if}
                        </span>
                      </div>
                      <div class="archive-actions-inline recipe-btns">
                        {#if activeRecipeId !== recipe.id}
                          <button class="archive-btn archive-btn-primary" onclick={() => handleActivate(recipe.id)}>
                            Activate
                          </button>
                        {/if}
                        <button class="archive-btn" onclick={() => startEdit(recipe)}>
                          Edit
                        </button>
                        <button class="archive-btn" onclick={() => handlePublish(recipe.id, recipe.is_public)}>
                          {recipe.is_public ? 'Unpublish' : 'Publish'}
                        </button>
                        <button class="archive-btn" onclick={() => handleDelete(recipe.id)}>
                          Delete
                        </button>
                      </div>
                    </li>
                  {/if}
                {/each}
              </ul>
            {/if}
          </fieldset>
        {/if}

      {:else}
        <!-- ── Gallery tab ──────────────────────────────────────────── -->
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Gallery</legend>
          {#if galleryLoading}
            <p class="archive-muted"><span class="spinner"></span> Loading gallery…</p>
          {:else if galleryRecipes.length === 0}
            <p class="archive-muted">No public recipes yet. Publish one to share with the community!</p>
          {:else}
            <ul class="archive-pref-list">
              {#each galleryRecipes as recipe (recipe.id)}
                <li class="archive-pref-item">
                  <div class="recipe-main">
                    <span class="archive-pref-label">{recipe.name}</span>
                    <span class="archive-muted small">by user #{recipe.user_id} · {recipe.installs} installs</span>
                    {#if recipe.description}
                      <span class="archive-muted"> — {recipe.description}</span>
                    {/if}
                    <span class="recipe-meta">
                      Co-occur: {((recipe.blend as Record<string, number>).cooccur ?? 0).toFixed(2)}
                      · Tag Graph: {((recipe.blend as Record<string, number>).tag_graph ?? 0).toFixed(2)}
                      · Embeddings: {((recipe.blend as Record<string, number>).embeddings ?? 0).toFixed(2)}
                    </span>
                  </div>
                  <button class="archive-btn archive-btn-primary" onclick={() => handleInstall(recipe.id)}>
                    Install
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </fieldset>
      {/if}
    {/if}
  </div>
</main>
{:else}
<!-- Modern mode (unchanged) -->
<div class="settings-page">
  <h1>Recipe Builder</h1>
  <p class="muted subtitle">Customise how recommendations are blended for you.</p>

  {#if !auth.isLoggedIn}
    <div class="card empty">
      <p class="muted">Login to create and manage recipes.</p>
    </div>
  {:else}
    {#if error}
      <div class="card error-card"><p class="error-text">⚠️ {error}</p></div>
    {/if}
    {#if saved}
      <div class="card success-card"><p>✅ {saved}</p></div>
    {/if}

    <!-- ── Tab bar ──────────────────────────────────────────────────── -->
    <div class="tab-bar">
      <button
        class="tab"
        class:active={tab === 'mine'}
        onclick={() => { tab = 'mine'; }}
      >
        My Recipes
      </button>
      <button
        class="tab"
        class:active={tab === 'gallery'}
        onclick={() => { tab = 'gallery'; loadGallery(); }}
      >
        Gallery
      </button>
    </div>

    {#if tab === 'mine'}
      <!-- ── My Recipes tab ──────────────────────────────────────────── -->
      {#if loading}
        <div class="card"><p class="muted"><span class="spinner"></span> Loading…</p></div>
      {:else}
        <!-- Create button -->
        <button
          class="btn btn-primary"
          onclick={() => { showCreateForm = !showCreateForm; }}
        >
          {showCreateForm ? 'Cancel' : '+ New Recipe'}
        </button>

        <!-- Create form -->
        {#if showCreateForm}
          <div class="card form-card">
            <h2>New Recipe</h2>
            <label>
              Name
              <input type="text" bind:value={newName} placeholder="e.g. Dark &amp; Angsty" />
            </label>
            <label>
              Description
              <textarea bind:value={newDescription} placeholder="What this recipe is good for…" rows="2"></textarea>
            </label>

            <fieldset>
              <legend>Strategy Blend</legend>
              <p class="muted small">Weights normalise to sum to 1.0</p>
              <div class="slider-group">
                <label>
                  Co-occurrence
                  <input type="range" min="0" max="1" step="0.05" bind:value={newBlendCooccur} />
                  <span class="val">{newBlendCooccur.toFixed(2)}</span>
                </label>
                <label>
                  Tag Graph
                  <input type="range" min="0" max="1" step="0.05" bind:value={newBlendTagGraph} />
                  <span class="val">{newBlendTagGraph.toFixed(2)}</span>
                </label>
                <label>
                  Embeddings
                  <input type="range" min="0" max="1" step="0.05" bind:value={newBlendEmbeddings} />
                  <span class="val">{newBlendEmbeddings.toFixed(2)}</span>
                </label>
                <p class="muted small">
                  Total: {blendTotal(newBlendCooccur, newBlendTagGraph, newBlendEmbeddings).toFixed(2)}
                </p>
              </div>
            </fieldset>

            <fieldset>
              <legend>Filters</legend>
              <label>
                Min. word count
                <input type="number" min="0" step="1000" bind:value={newMinWords} />
              </label>
              <label class="checkbox-label">
                Exclude warnings:
              </label>
              <div class="warning-tags">
                {#each WARNING_TAGS as tag}
                  <label class="chip-toggle">
                    <input
                      type="checkbox"
                      checked={newExcludeWarnings.includes(tag)}
                      onchange={() => {
                        if (newExcludeWarnings.includes(tag)) {
                          newExcludeWarnings = newExcludeWarnings.filter(t => t !== tag);
                        } else {
                          newExcludeWarnings = [...newExcludeWarnings, tag];
                        }
                      }}
                    />
                    {tag}
                  </label>
                {/each}
              </div>
            </fieldset>

            <label>
              Curator prior ({newCuratorPrior.toFixed(2)})
              <input type="range" min="0" max="1" step="0.05" bind:value={newCuratorPrior} />
            </label>

            <button class="btn btn-primary" onclick={handleCreate} disabled={creating}>
              {#if creating}<span class="spinner"></span>{:else}Create Recipe{/if}
            </button>
          </div>
        {/if}

        <!-- Recipe list -->
        {#if recipes.length === 0}
          <div class="card empty">
            <p class="muted">No recipes yet. Create one to customise your recommendations!</p>
          </div>
        {:else}
          <div class="recipe-list">
            {#each recipes as recipe (recipe.id)}
              {#if editingId === recipe.id}
                <!-- Edit mode -->
                <div class="card recipe-card editing">
                  <h3>Edit Recipe</h3>
                  <label>
                    Name
                    <input type="text" bind:value={editName} />
                  </label>
                  <label>
                    Description
                    <textarea bind:value={editDescription} rows="2"></textarea>
                  </label>
                  <fieldset>
                    <legend>Strategy Blend</legend>
                    <div class="slider-group">
                      <label>
                        Co-occurrence
                        <input type="range" min="0" max="1" step="0.05" bind:value={editBlendCooccur} />
                        <span class="val">{editBlendCooccur.toFixed(2)}</span>
                      </label>
                      <label>
                        Tag Graph
                        <input type="range" min="0" max="1" step="0.05" bind:value={editBlendTagGraph} />
                        <span class="val">{editBlendTagGraph.toFixed(2)}</span>
                      </label>
                      <label>
                        Embeddings
                        <input type="range" min="0" max="1" step="0.05" bind:value={editBlendEmbeddings} />
                        <span class="val">{editBlendEmbeddings.toFixed(2)}</span>
                      </label>
                    </div>
                  </fieldset>
                  <label>
                    Min. word count
                    <input type="number" min="0" step="1000" bind:value={editMinWords} />
                  </label>
                  <label>
                    Curator prior ({editCuratorPrior.toFixed(2)})
                    <input type="range" min="0" max="1" step="0.05" bind:value={editCuratorPrior} />
                  </label>
                  <div class="btn-row">
                    <button class="btn btn-primary sm" onclick={handleUpdate} disabled={savingEdit}>
                      {#if savingEdit}<span class="spinner"></span>{:else}Save{/if}
                    </button>
                    <button class="btn btn-secondary sm" onclick={cancelEdit}>Cancel</button>
                  </div>
                </div>
              {:else}
                <!-- View mode -->
                <div class="card recipe-card" class:active={activeRecipeId === recipe.id}>
                  <div class="recipe-head">
                    <h3>
                      {recipe.name}
                      {#if activeRecipeId === recipe.id}
                        <span class="active-badge">Active</span>
                      {/if}
                      {#if recipe.is_public}
                        <span class="public-badge">Public</span>
                      {/if}
                    </h3>
                    <span class="muted small">Updated {formatDate(recipe.updated_at)}</span>
                  </div>
                  {#if recipe.description}
                    <p class="muted">{recipe.description}</p>
                  {/if}
                  <div class="recipe-meta">
                    <span>Co-occur: {((recipe.blend as Record<string, number>).cooccur ?? 0).toFixed(2)}</span>
                    <span>Tag Graph: {((recipe.blend as Record<string, number>).tag_graph ?? 0).toFixed(2)}</span>
                    <span>Embeddings: {((recipe.blend as Record<string, number>).embeddings ?? 0).toFixed(2)}</span>
                    <span>Curator: {recipe.curator_prior.toFixed(2)}</span>
                    {#if recipe.is_public}
                      <span>Installs: {recipe.installs}</span>
                    {/if}
                  </div>
                  <div class="btn-row">
                    {#if activeRecipeId !== recipe.id}
                      <button class="btn btn-primary sm" onclick={() => handleActivate(recipe.id)}>
                        Activate
                      </button>
                    {/if}
                    <button class="btn btn-secondary sm" onclick={() => startEdit(recipe)}>
                      Edit
                    </button>
                    <button
                      class="btn btn-secondary sm"
                      onclick={() => handlePublish(recipe.id, recipe.is_public)}
                    >
                      {recipe.is_public ? 'Unpublish' : 'Publish'}
                    </button>
                    <button class="btn btn-danger sm" onclick={() => handleDelete(recipe.id)}>
                      Delete
                    </button>
                  </div>
                </div>
              {/if}
            {/each}
          </div>
        {/if}
      {/if}

    {:else}
      <!-- ── Gallery tab ─────────────────────────────────────────────── -->
      {#if galleryLoading}
        <div class="card"><p class="muted"><span class="spinner"></span> Loading gallery…</p></div>
      {:else if galleryRecipes.length === 0}
        <div class="card empty">
          <p class="muted">No public recipes yet. Publish one to share with the community!</p>
        </div>
      {:else}
        <div class="recipe-list">
          {#each galleryRecipes as recipe (recipe.id)}
            <div class="card recipe-card">
              <div class="recipe-head">
                <h3>{recipe.name}</h3>
                <span class="muted small">by user #{recipe.user_id} · {recipe.installs} installs</span>
              </div>
              {#if recipe.description}
                <p class="muted">{recipe.description}</p>
              {/if}
              <div class="recipe-meta">
                <span>Co-occur: {((recipe.blend as Record<string, number>).cooccur ?? 0).toFixed(2)}</span>
                <span>Tag Graph: {((recipe.blend as Record<string, number>).tag_graph ?? 0).toFixed(2)}</span>
                <span>Embeddings: {((recipe.blend as Record<string, number>).embeddings ?? 0).toFixed(2)}</span>
              </div>
              <button class="btn btn-primary sm" onclick={() => handleInstall(recipe.id)}>
                Install
              </button>
            </div>
          {/each}
        </div>
      {/if}
    {/if}
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
  .archive-content {
    max-width: var(--archive-max-width, 900px);
    margin: 0 auto;
    padding: 1rem;
  }
  .archive-header {
    padding: 0 0 0.6rem;
    border-bottom: 2px solid var(--archive-link, #990000);
    margin-bottom: 1rem;
  }
  .archive-page-title {
    font-size: 1.6em;
    font-weight: 700;
    margin: 0;
  }
  .archive-muted {
    color: var(--archive-muted, #666666);
    font-style: italic;
  }
  .archive-error {
    color: var(--archive-link, #990000);
    font-weight: 700;
    font-size: 0.9rem;
  }
  .archive-success {
    color: #326342;
    font-weight: 700;
    font-size: 0.9rem;
  }
  .archive-summary {
    margin: 1rem 0;
    padding: 0.4rem 1rem;
    border-left: 3px solid var(--archive-border, #dddddd);
    color: var(--archive-muted, #666666);
    font-style: italic;
    background: var(--archive-bg-raised, #f5f5f5);
  }
  /* Tabs (trending pattern) */
  .archive-tabs {
    display: flex;
    list-style: none;
    margin: 0 0 1.2rem;
    padding: 0;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    gap: 0;
  }
  .archive-tabs li {
    margin: 0;
    margin-bottom: -1px;
  }
  .archive-tabs li button {
    display: block;
    padding: 0.45rem 1rem;
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 0.9em;
    font-weight: 600;
    color: var(--archive-muted, #666666);
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    cursor: pointer;
    line-height: 1.4;
  }
  .archive-tabs li button:hover {
    color: var(--archive-link, #990000);
    background: var(--archive-bg, #ffffff);
  }
  .archive-tabs li.current button {
    color: var(--archive-link, #990000);
    background: var(--archive-bg, #ffffff);
    border-bottom-color: var(--archive-bg, #ffffff);
  }
  .archive-fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.6rem 0.9rem 0.75rem;
    margin: 0 0 1rem;
    background: var(--archive-bg, #ffffff);
  }
  .archive-fieldset.nested { margin-bottom: 0.6rem; }
  .archive-fieldset legend,
  .archive-legend {
    font-weight: 700;
    font-size: 0.9em;
    color: var(--archive-link, #990000);
    padding: 0 0.4rem;
  }
  .archive-actions { margin: 0.3rem 0 0.6rem; }
  .archive-form { display: flex; flex-direction: column; gap: 0.6rem; align-items: flex-start; }
  .archive-form .archive-dl { margin: 0; }
  .edit-form { width: 100%; }
  .archive-dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.4rem 1rem;
    margin: 0.4rem 0;
  }
  .archive-dl dt {
    font-weight: 600;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-dl dd { margin: 0; }
  .archive-input {
    padding: 0.3rem 0.5rem;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.9rem;
    width: 100%;
    box-sizing: border-box;
    max-width: 420px;
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
  }
  .archive-actions-inline {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  input[type='range'] { width: 180px; }
  .val {
    font-family: monospace;
    font-size: 0.85rem;
    color: var(--archive-link, #990000);
  }
  .archive-btn {
    padding: 0.28rem 0.7rem;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-text, #2a2a2a);
    font-family: inherit;
    font-size: 0.85rem;
    cursor: pointer;
  }
  .archive-btn:hover { background: var(--archive-border, #dddddd); }
  .archive-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .archive-btn-primary {
    color: var(--archive-bg, #ffffff);
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }
  .archive-btn-primary:hover { background: var(--archive-link-visited, #660066); border-color: var(--archive-link-visited, #660066); }
  .archive-checkbox-label {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.85rem;
    cursor: pointer;
  }
  .archive-checkbox-label input { width: auto; padding: 0; }
  .warning-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem 0.9rem;
  }
  .archive-pref-list {
    list-style: none;
    margin: 0.3rem 0 0;
    padding: 0;
    border-top: 1px solid var(--archive-border, #dddddd);
  }
  .archive-pref-item {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.55rem 0.3rem;
    border-bottom: 1px solid var(--archive-border, #eeeeee);
    font-size: 0.92rem;
  }
  .archive-pref-item:nth-child(odd) { background: var(--archive-bg-raised, #f5f5f5); }
  .archive-pref-item:last-child { border-bottom: none; }
  .archive-pref-item.editing {
    background: var(--archive-bg, #ffffff);
    border-left: 3px solid var(--archive-link, #990000);
  }
  .archive-pref-item.active {
    border-left: 3px solid var(--archive-link, #990000);
  }
  .recipe-main {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
  }
  .archive-pref-label { font-weight: 700; }
  .badge {
    display: inline-block;
    font-size: 0.72rem;
    padding: 0.05rem 0.4rem;
    font-weight: 700;
    border: 1px solid var(--archive-border, #dddddd);
    vertical-align: middle;
  }
  .badge-active {
    color: var(--archive-bg, #ffffff);
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }
  .badge-public {
    color: #326342;
    border-color: #326342;
  }
  .recipe-meta {
    font-size: 0.82rem;
    color: var(--archive-muted, #666666);
  }
  .recipe-btns { flex-shrink: 0; }
  .small { font-size: 0.82rem; }

  /* ── Modern mode (scoped — no bleed) ── */
  .settings-page {
    max-width: var(--max-width);
    margin: 0 auto;
    padding: 1rem;
  }
  .settings-page h1 { margin-top: 0; margin-bottom: 0.25rem; }
  .settings-page h2 { margin: 0 0 0.5rem; font-size: 1.2rem; }
  .settings-page h3 { margin: 0 0 0.25rem; font-size: 1.1rem; }
  .settings-page .subtitle { margin-top: 0; margin-bottom: 1rem; }
  .settings-page .small { font-size: 0.82rem; }

  /* Tabs */
  .settings-page .tab-bar {
    display: flex;
    gap: 0;
    margin-bottom: 1rem;
    border-bottom: 1px solid var(--color-border, #333);
  }
  .settings-page .tab {
    background: none;
    border: none;
    padding: 0.5rem 1rem;
    cursor: pointer;
    font-size: 0.95rem;
    color: var(--color-muted, #999);
    border-bottom: 2px solid transparent;
    transition: color 0.2s, border-color 0.2s;
  }
  .settings-page .tab:hover { color: var(--color-text); }
  .settings-page .tab.active {
    color: var(--color-primary, #7c3aed);
    border-bottom-color: var(--color-primary, #7c3aed);
  }

  /* Cards */
  .settings-page .card {
    border: 1px solid var(--color-border, #333);
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 0.75rem;
    background: var(--color-surface-2, #1c1c1c);
  }
  .settings-page .empty { text-align: center; padding: 2rem; }
  .settings-page .error-card { border-color: var(--color-error); }
  .settings-page .success-card { border-color: var(--color-success, #2e7d32); }
  .settings-page .error-text { color: var(--color-error); margin: 0; }

  /* Form */
  .settings-page .form-card {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  .settings-page .form-card label, .settings-page .recipe-card label, .settings-page fieldset label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.85rem;
  }
  .settings-page .form-card input[type="text"],
  .settings-page .form-card textarea,
  .settings-page .form-card input[type="number"],
  .settings-page .recipe-card input[type="text"],
  .settings-page .recipe-card textarea,
  .settings-page .recipe-card input[type="number"] {
    padding: 0.4rem 0.6rem;
    border-radius: 6px;
    border: 1px solid var(--color-border, #444);
    background: var(--color-surface, #141414);
    color: var(--color-text);
  }
  .settings-page input[type="range"] {
    width: 100%;
  }
  .settings-page fieldset {
    border: 1px solid var(--color-border, #333);
    border-radius: 8px;
    padding: 0.75rem;
    margin: 0;
  }
  .settings-page legend {
    font-weight: 600;
    font-size: 0.9rem;
    padding: 0 0.4rem;
  }
  .settings-page .slider-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .settings-page .val {
    font-family: monospace;
    font-size: 0.85rem;
    color: var(--color-primary, #7c3aed);
  }
  .settings-page .warning-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }
  .settings-page .chip-toggle {
    display: inline-flex !important;
    flex-direction: row !important;
    align-items: center;
    gap: 0.25rem;
    padding: 0.2rem 0.6rem;
    border: 1px solid var(--color-border, #444);
    border-radius: 999px;
    font-size: 0.8rem;
    cursor: pointer;
    transition: background 0.2s;
  }
  .settings-page .chip-toggle:has(input:checked) {
    background: var(--color-primary, #7c3aed);
    color: white;
    border-color: var(--color-primary, #7c3aed);
  }
  .settings-page .chip-toggle input { margin: 0; }

  /* Recipe list */
  .settings-page .recipe-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-top: 0.75rem;
  }
  .settings-page .recipe-card { transition: border-color 0.2s; }
  .settings-page .recipe-card.active {
    border-color: var(--color-primary, #7c3aed);
    box-shadow: 0 0 0 1px var(--color-primary, #7c3aed);
  }
  .settings-page .recipe-card.editing {
    border-color: var(--color-primary, #7c3aed);
    background: var(--color-surface-3, #222);
  }
  .settings-page .recipe-head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  .settings-page .recipe-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    font-size: 0.82rem;
    color: var(--color-muted, #999);
    margin: 0.4rem 0;
  }
  .settings-page .active-badge, .settings-page .public-badge {
    font-size: 0.72rem;
    padding: 0.1rem 0.5rem;
    border-radius: 999px;
    font-weight: 600;
    vertical-align: middle;
  }
  .settings-page .active-badge {
    background: var(--color-primary, #7c3aed);
    color: white;
  }
  .settings-page .public-badge {
    background: var(--color-success, #2e7d32);
    color: white;
  }

  /* Buttons */
  .settings-page .btn {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    font-weight: 600;
    cursor: pointer;
    border: 1px solid transparent;
    transition: opacity 0.2s;
  }
  .settings-page .btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .settings-page .btn-primary {
    background: var(--color-primary, #7c3aed);
    color: white;
  }
  .settings-page .btn-secondary {
    background: var(--color-surface-3, #333);
    color: var(--color-text);
    border-color: var(--color-border, #444);
  }
  .settings-page .btn-danger {
    background: var(--color-error, #d32f2f);
    color: white;
  }
  .settings-page .btn.sm { padding: 0.3rem 0.7rem; font-size: 0.82rem; }
  .settings-page .btn-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }
  .settings-page .spinner {
    display: inline-block;
    width: 0.8rem;
    height: 0.8rem;
    border: 2px solid var(--color-border, #555);
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    vertical-align: middle;
  }
  .archive-main .spinner {
    display: inline-block;
    width: 0.8rem;
    height: 0.8rem;
    border: 2px solid var(--archive-muted, #666);
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    vertical-align: middle;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
