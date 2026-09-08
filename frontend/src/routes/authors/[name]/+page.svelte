<script lang="ts">
  import { onMount } from 'svelte';
  import { getAuthorByName, updateAuthorProfile } from '$lib/api/series';
  import type { AuthorDetail, AuthorOrphan, AuthorWork } from '$lib/api/series';
  import { auth } from '$lib/stores/auth.svelte';
  import { formatWords, stripHtml } from '$lib/util';
  import { getPref } from '$lib/prefs';
  import WorkListFilters from '$lib/ui/archive/WorkListFilters.svelte';
  import BatchAuthorDownload from '$lib/components/BatchAuthorDownload.svelte';
  import { isSupportedAuthorPageUrl, siteNameForAuthorUrl } from '$lib/api/social';

  let { data } = $props();
  const authorName = $derived(decodeURIComponent(data.name));

  let author = $state<AuthorDetail | null>(null);
  let works = $state<AuthorWork[]>([]);
  let orphans = $state<AuthorOrphan[]>([]);
  let loading = $state(true);
  let error = $state('');
  let editingProfile = $state(false);
  let editBio = $state('');
  let editAvatar = $state('');
  let editBadge = $state('');
  let isOwner = $state(false);
  let authorId: number | null = $state(null);

  const uiMode = $derived.by(() => {
    try {
      return getPref('uiMode');
    } catch {
      return 'modern';
    }
  });
  const isArchive = $derived(uiMode === 'archive');
  const DEFAULT_AVATAR = 'https://archiveofourown.org/images/cons/avatar.gif';

  /** Author's supported source URL for batch download (AO3/XenForo). */
  const sourceUrl = $derived(
    author?.socials?.find(
      (s) => isSupportedAuthorPageUrl(s.url),
    )?.url ?? null,
  );
  const sourceSite = $derived(sourceUrl ? siteNameForAuthorUrl(sourceUrl) : '');

  onMount(async () => {
    await auth.init();
    await load();
  });

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await getAuthorByName(authorName);
      if (res.err !== 0) { error = 'Could not load author.'; return; }
      author = res.author;
      works = res.works ?? [];
      orphans = res.orphans ?? [];
      isOwner = auth.isLoggedIn && author?.name === auth.username;
      if (author?.id) authorId = author.id;
    } catch {
      error = 'Network error loading author.';
    } finally {
      loading = false;
    }
  }

  async function saveProfile() {
    if (!authorId) return;
    const updates: Record<string, string> = {};
    if (editAvatar.trim()) updates.avatar_url = editAvatar;
    if (editBio.trim()) updates.bio = editBio;
    if (editBadge.trim()) updates.badge_text = editBadge;
    try {
      const res = await updateAuthorProfile(authorId, updates);
      if (res.err === 0) { editingProfile = false; await load(); }
    } catch { error = 'Failed to save profile.'; }
  }

  function startEditingProfile() {
    editingProfile = true;
    editBio = author?.bio ?? '';
    editAvatar = author?.avatar_url ?? '';
    editBadge = author?.badge_text ?? '';
  }

  function workHref(w: AuthorWork | AuthorOrphan): string {
    if (w.url_id) return `/works/${encodeURIComponent(w.url_id)}`;
    return w.work_id ? `/work/${w.work_id}` : '#';
  }

  function renderBio(): string {
    if (!author?.bio) return '';
    return isArchive ? author.bio : stripHtml(author.bio);
  }

  // ── Filter state (for the sidebar) ────────────────────────────────
  type WorkEntry = AuthorWork | AuthorOrphan;

  interface FilterState {
    sort: string;
    complete: boolean | null;
    minWords: number | null;
    maxWords: number | null;
    includeTags: string;
    excludeTags: string;
  }

  let filters = $state<FilterState>({
    sort: 'updated',
    complete: null,
    minWords: null,
    maxWords: null,
    includeTags: '',
    excludeTags: '',
  });

  // All works combined for client-side filtering.
  let allWorks = $derived<WorkEntry[]>([...works, ...orphans]);

  function applyFilters(
    ws: WorkEntry[],
    f: FilterState,
  ): WorkEntry[] {
    let result = ws;
    if (f.complete === true) {
      result = result.filter((w) => w.status === 'complete');
    } else if (f.complete === false) {
      result = result.filter((w) => w.status !== 'complete');
    }
    if (f.minWords != null && f.minWords > 0) {
      result = result.filter((w) => w.words >= (f.minWords ?? 0));
    }
    if (f.maxWords != null && f.maxWords > 0) {
      result = result.filter((w) => w.words <= (f.maxWords ?? Infinity));
    }
    switch (f.sort) {
      case 'words':
        return [...result].sort((a, b) => b.words - a.words);
      case 'title':
        return [...result].sort((a, b) =>
          a.canonical_title.localeCompare(b.canonical_title),
        );
      default:
        return result; // 'updated'/'created' — already sorted from API
    }
  }

  let filteredWorks = $derived(applyFilters(allWorks, filters));

  function handleFilterApply(e: CustomEvent<FilterState>) {
    filters = e.detail;
  }

  function handleFilterClear() {
    filters = {
      sort: 'updated',
      complete: null,
      minWords: null,
      maxWords: null,
      includeTags: '',
      excludeTags: '',
    };
  }
</script>

{#if loading}
  <div class="card"><p class="muted"><span class="spinner"></span> Loading author…</p></div>
{:else if error}
  <div class="card error-card"><strong class="error-text">! {error}</strong></div>
{:else if author}
  <div class="author-page">
    {#if isArchive}
      <div class="archive-page">
        <WorkListFilters on:apply={handleFilterApply} on:clear={handleFilterClear} />
        <main class="archive-main">
        <fieldset class="archive-fieldset">
        <legend class="archive-legend">About {author.name}</legend>
        <div class="archive-author-header">
          <div class="archive-author-avatar">
            {#if author.avatar_url}
              <img class="avatar" src={author.avatar_url} alt={`${author.name} avatar`} loading="lazy" />
            {:else}
              <img class="avatar avatar-fallback" src={DEFAULT_AVATAR} alt={`${author.name} avatar`} loading="lazy" />
            {/if}
          </div>
          <div class="archive-author-meta">
            <h1 class="archive-author-name">✍️ {author.name}</h1>
            {#if author.badge_text}<p class="archive-badge-text">{author.badge_text}</p>{/if}
            {#if author.badges && author.badges.length > 0}
              <div class="archive-badges">
                {#each author.badges as b (b.badge_type)}
                  <span class="archive-badge-icon" title={b.name}>{b.icon}</span>
                {/each}
              </div>
            {/if}
            <dl class="archive-dl">
              <div class="dl-row"><dt>Works</dt><dd>{author.work_count}</dd></div>
              <div class="dl-row"><dt>Words</dt><dd>{formatWords(author.total_words)}</dd></div>
            </dl>
            {#if author.top_tags && author.top_tags.length > 0}
              <div class="archive-tags">
                {#each author.top_tags.slice(0, 12) as t (t.name)}
                  <a class="archive-tag" href={`/search?q=${encodeURIComponent(t.name)}`}>{t.name}</a>
                {/each}
              </div>
            {/if}
            {#if author.socials && author.socials.length > 0}
              <div class="archive-socials">
                {#each author.socials as s (s.id)}
                  <a class="archive-link" href={s.url} target="_blank" rel="noopener">
                    {s.platform}{s.label ? `: ${s.label}` : ''}
                  </a>
                {/each}
              </div>
            {/if}
          </div>
        </div>
        {#if isOwner && !editingProfile}
          <p class="archive-actions">
            <button class="archive-btn archive-btn--small" onclick={startEditingProfile}>Edit Profile</button>
          </p>
        {/if}
      </fieldset>

      {#if isOwner && editingProfile}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Edit Profile</legend>
          <dl class="archive-dl">
            <div class="dl-row">
              <dt><label for="edit-avatar">Avatar URL</label></dt>
              <dd><input id="edit-avatar" type="url" bind:value={editAvatar} placeholder="https://..." /></dd>
            </div>
            <div class="dl-row">
              <dt><label for="edit-bio">Bio</label></dt>
              <dd><textarea id="edit-bio" bind:value={editBio} rows="3" placeholder="Tell us about yourself..."></textarea></dd>
            </div>
            <div class="dl-row">
              <dt><label for="edit-badge">Badge Text</label></dt>
              <dd><input id="edit-badge" type="text" bind:value={editBadge} placeholder="e.g. 'Top Author'" /></dd>
            </div>
          </dl>
          <p class="archive-actions">
            <button class="archive-btn" onclick={saveProfile}>Save</button>
            <button class="archive-btn archive-btn--small" onclick={() => editingProfile = false}>Cancel</button>
          </p>
        </fieldset>
      {/if}

      {#if author.bio}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Bio</legend>
          <div class="archive-bio">{@html author.bio}</div>
        </fieldset>
      {/if}

      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Works by {author.name}</legend>
        {#if allWorks.length === 0}
          <p class="archive-muted">No works by this author yet.</p>
        {:else if filteredWorks.length === 0}
          <p class="archive-muted">No works match the current filters.</p>
        {:else}
          <ul class="archive-work-list">
            {#each filteredWorks as w (w.work_id ?? w.url_id)}
              <li>
                <a class="archive-work-link" href={workHref(w)}><strong>{w.canonical_title}</strong></a>
                <span class="archive-work-meta">{formatWords(w.words)} words · {w.chapters} ch · {w.status}</span>
              </li>
            {/each}
          </ul>
        {/if}
      </fieldset>
      <!-- Batch download (AO3/XenForo) -->
      {#if sourceUrl}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Batch Download</legend>
          <BatchAuthorDownload
            authorUrl={sourceUrl}
            authorName={author.name}
            siteName={sourceSite}
          />
        </fieldset>
      {/if}
        </main>
      </div>
    {:else}
      <div class="author-header card">
        <div class="avatar-wrapper">
          {#if author.avatar_url}
            <img class="avatar" src={author.avatar_url} alt={`${author.name} avatar`} loading="lazy" />
          {:else}
            <img class="avatar avatar-fallback" src={DEFAULT_AVATAR} alt={`${author.name} avatar`} loading="lazy" />
          {/if}
        </div>
        <div class="author-heading">
          <h1>✍️ {author.name}</h1>
          <div class="author-badges">
            {#if author.badge_text}<span class="badge-text">{author.badge_text}</span>{/if}
            {#if author.badges && author.badges.length > 0}
              <span class="badge-icons" title="Earned badges">
                {#each author.badges as b (b.badge_type)}<span class="badge-icon" title={b.name}>{b.icon}</span>{/each}
              </span>
            {/if}
          </div>
          <div class="stats-row">
            <span class="stat">{author.work_count} {author.work_count === 1 ? 'work' : 'works'}</span>
            <span class="stat">{formatWords(author.total_words)} words</span>
          </div>
          {#if author.top_tags.length > 0}
            <div class="tags">
              {#each author.top_tags.slice(0, 12) as t (t.name)}
                <a class="tag-chip" href={`/search?q=${encodeURIComponent(t.name)}`}>{t.name}</a>
              {/each}
            </div>
          {/if}
          {#if author.favorite_tags && author.favorite_tags.length > 0}
            <div class="fav-tags">
              <span class="muted">Favorite tags:</span>
              {#each author.favorite_tags as t (t.name)}
                <a class="tag-chip tag-chip--fav" href={`/search?q=${encodeURIComponent(t.name)}`}>{t.name}</a>
              {/each}
            </div>
          {/if}
          {#if author.socials && author.socials.length > 0}
            <div class="author-socials">
              <span class="muted">Find me on:</span>
              {#each author.socials as s (s.id)}
                <a class="social-link" href={s.url} target="_blank" rel="noopener">
                  {s.platform}{s.label ? `: ${s.label}` : ''}
                </a>
              {/each}
            </div>
          {/if}
        </div>
      </div>

      {#if isOwner && editingProfile}
        <div class="card profile-edit-form">
          <h3>Edit Profile</h3>
          <dl>
            <div class="dl-row"><dt><label for="edit-avatar-m">Avatar URL</label></dt><dd><input id="edit-avatar-m" type="url" bind:value={editAvatar} /></dd></div>
            <div class="dl-row"><dt><label for="edit-bio-m">Bio</label></dt><dd><textarea id="edit-bio-m" bind:value={editBio} rows="3"></textarea></dd></div>
            <div class="dl-row"><dt><label for="edit-badge-m">Badge Text</label></dt><dd><input id="edit-badge-m" type="text" bind:value={editBadge} /></dd></div>
          </dl>
          <p class="submit actions">
            <button class="btn" onclick={saveProfile}>Save</button>
            <button class="btn btn-secondary" onclick={() => editingProfile = false}>Cancel</button>
          </p>
        </div>
      {:else if isOwner}
        <button class="btn btn-secondary" onclick={startEditingProfile}>Edit Profile</button>
      {/if}

      {#if author.bio}
        <div class="card author-bio"><p>{renderBio()}</p></div>
      {/if}

      {#if works.length === 0 && orphans.length === 0}
        <div class="card"><p class="muted">No works by this author yet.</p></div>
      {:else}
        <div class="works-grid">
          {#each works as w (w.work_id)}
            <a class="card work-card" href={workHref(w)}>
              <strong class="work-title">{w.canonical_title}</strong>
              <p class="muted desc">{stripHtml(w.description).slice(0, 160)}{w.description.length > 160 ? '...' : ''}</p>
              <span class="muted work-meta">{formatWords(w.words)} words · {w.chapters} ch · {w.status}</span>
            </a>
          {/each}
          {#each orphans as o (o.url_id)}
            <a class="card work-card" href={workHref(o)}>
              <strong class="work-title">{o.canonical_title}</strong>
              <p class="muted desc">{formatWords(o.words)} words · {o.chapters} ch · {o.status}</p>
            </a>
          {/each}
        </div>
      {/if}
    {/if}
    <!-- Batch download (AO3/XenForo) -->
    {#if sourceUrl}
      <div class="card section">
        <h3>Batch Download</h3>
        <BatchAuthorDownload
          authorUrl={sourceUrl}
          authorName={author.name}
          siteName={sourceSite}
        />
      </div>
    {/if}

    <p class="muted back-link"><a href="/"> Back to home</a></p>
  </div>
{/if}

<style>
  .author-page { max-width: var(--max-width); margin: 0 auto; padding: 1rem; }
  .back-link { margin-top: 1.5rem; }

  /* Archive two-column layout (sidebar + main) */
  .archive-page {
    display: grid;
    grid-template-columns: 240px 1fr;
    gap: 1rem;
    max-width: var(--max-width);
    margin: 0 auto;
    padding: 1rem;
  }
  .archive-main { min-width: 0; }
  @media (max-width: 720px) {
    .archive-page {
      grid-template-columns: 1fr;
    }
  }
  /* Modern UI. */
  .author-header { display: flex; align-items: flex-start; gap: 1rem; }
  .avatar-wrapper { flex-shrink: 0; }
  .avatar { width: 80px; height: 80px; border-radius: 50%; object-fit: cover; border: 1px solid var(--color-border); }
  .avatar-fallback { opacity: 0.7; }
  .author-heading { flex: 1; }
  .author-heading h1 { margin: 0 0 0.5rem; font-size: 1.6rem; }
  .author-badges { margin: 0.25rem 0 0.5rem; }
  .badge-text { display: inline-block; font-weight: 600; color: var(--color-primary); margin-right: 0.5rem; }
  .badge-icons { display: inline-block; }
  .badge-icon { display: inline-block; font-size: 1.2rem; margin-right: 0.15rem; }
  .stats-row { display: flex; gap: 1.2rem; font-weight: 600; color: var(--color-muted); margin-bottom: 0.5rem; }
  .tags { display: flex; flex-wrap: wrap; gap: 0.35rem; }
  .tag-chip { font-size: 0.78rem; padding: 0.2rem 0.55rem; background: var(--color-surface-2); border: 1px solid var(--color-border); border-radius: var(--radius-sm); color: var(--color-primary); text-decoration: none; }
  .tag-chip--fav { color: var(--color-warning); }
  .fav-tags { margin-top: 0.5rem; display: flex; flex-wrap: wrap; gap: 0.35rem; align-items: center; }
  .fav-tags .muted { font-size: 0.85rem; }
  .author-socials { margin-top: 0.5rem; display: flex; flex-wrap: wrap; gap: 0.5rem; align-items: center; }
  .social-link { font-size: 0.85rem; color: var(--color-primary); text-decoration: underline; }
  .works-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 0.8rem; margin-top: 1rem; }
  .work-card { display: flex; flex-direction: column; gap: 0.3rem; text-decoration: none; color: inherit; }
  .work-title { font-size: 1.02rem; }
</style>
