<script lang="ts">
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';
  import {
    getAuthor,
    updateAuthor,
    addSocial,
    removeSocial,
    proposeMerge,
    type AuthorProfile,
    type SocialLink,
  } from '$lib/api/authors';
  import { auth } from '$lib/stores/auth.svelte';
  import BatchAuthorDownload from '$lib/components/BatchAuthorDownload.svelte';
  import { isSupportedAuthorPageUrl, siteNameForAuthorUrl } from '$lib/api/social';

  const uiMode = $derived(getPref('uiMode'));

  let { data } = $props();
  const authorId = $derived(Number(data.id));

  let profile = $state<AuthorProfile | null>(null);
  let loading = $state(true);
  let error = $state('');

  const isCurator = $derived(auth.level >= 50);

  /** Author's supported source URL for batch download (AO3/XenForo). */
  const sourceUrl = $derived(
    profile?.socials?.find(
      (s) => s.platform === 'ao3' && isSupportedAuthorPageUrl(s.url),
    )?.url
    ?? profile?.socials?.find(
      (s) => s.platform !== 'ao3' && isSupportedAuthorPageUrl(s.url),
    )?.url
    ?? null,
  );
  const sourceSite = $derived(sourceUrl ? siteNameForAuthorUrl(sourceUrl) : '');

  // Bio editing
  let bioDraft = $state('');
  let savingBio = $state(false);
  let bioMsg = $state('');

  // Avatar editing
  let avatarDraft = $state('');
  let savingAvatar = $state(false);
  let avatarMsg = $state('');

  // Social link form
  let socialPlatform = $state('');
  let socialUrl = $state('');
  let socialLabel = $state('');
  let addingSocial = $state(false);
  let socialMsg = $state('');

  // Merge proposal form (curator)
  let mergeSourceAuthor = $state('');
  let mergeSourceUrl = $state('');
  let mergeAutoApprove = $state(false);
  let proposingMerge = $state(false);
  let mergeMsg = $state('');

  const SOCIAL_PLATFORMS = ['ao3', 'ffn', 'xitter', 'bluesky', 'tumblr', 'discord', 'website', 'other'];

  onMount(async () => {
    await auth.init();
    await load();
  });

  async function load() {
    loading = true;
    error = '';
    try {
      profile = await getAuthor(authorId);
      bioDraft = profile.bio ?? '';
      avatarDraft = profile.avatar_url ?? '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Could not load author profile.';
    } finally {
      loading = false;
    }
  }

  async function saveBio() {
    if (!profile || savingBio) return;
    savingBio = true;
    bioMsg = '';
    try {
      await updateAuthor(profile.id, { bio: bioDraft });
      profile.bio = bioDraft;
      bioMsg = '✅ Bio saved';
    } catch (e) {
      bioMsg = `⚠️ ${e instanceof Error ? e.message : 'Save failed'}`;
    } finally {
      savingBio = false;
    }
  }

  async function saveAvatar() {
    if (!profile || savingAvatar) return;
    savingAvatar = true;
    avatarMsg = '';
    try {
      await updateAuthor(profile.id, { avatar_url: avatarDraft });
      profile.avatar_url = avatarDraft;
      avatarMsg = '✅ Avatar saved';
    } catch (e) {
      avatarMsg = `⚠️ ${e instanceof Error ? e.message : 'Save failed'}`;
    } finally {
      savingAvatar = false;
    }
  }

  async function handleAddSocial() {
    if (!profile || addingSocial) return;
    const platform = socialPlatform.trim() || 'other';
    const url = socialUrl.trim();
    if (!url) {
      socialMsg = '⚠️ URL is required';
      return;
    }
    addingSocial = true;
    socialMsg = '';
    try {
      await addSocial(profile.id, {
        platform,
        url,
        ...(socialLabel.trim() ? { label: socialLabel.trim() } : {}),
      });
      socialMsg = '✅ Social link added';
      socialUrl = '';
      socialLabel = '';
      await load();
    } catch (e) {
      socialMsg = `⚠️ ${e instanceof Error ? e.message : 'Add failed'}`;
    } finally {
      addingSocial = false;
    }
  }

  async function handleRemoveSocial(social: SocialLink) {
    if (!profile) return;
    try {
      await removeSocial(profile.id, social.id);
      profile.socials = profile.socials.filter((s) => s.id !== social.id);
    } catch (e) {
      socialMsg = `⚠️ ${e instanceof Error ? e.message : 'Remove failed'}`;
    }
  }

  async function handleProposeMerge() {
    if (!profile || proposingMerge) return;
    if (!mergeSourceAuthor.trim() || !mergeSourceUrl.trim()) {
      mergeMsg = '⚠️ Source author and URL are required';
      return;
    }
    proposingMerge = true;
    mergeMsg = '';
    try {
      const res = await proposeMerge(
        mergeSourceAuthor.trim(),
        mergeSourceUrl.trim(),
        profile.id,
        { auto_approve: mergeAutoApprove },
      );
      mergeMsg = `✅ ${res.msg}`;
      mergeSourceAuthor = '';
      mergeSourceUrl = '';
      await load();
    } catch (e) {
      mergeMsg = `⚠️ ${e instanceof Error ? e.message : 'Proposal failed'}`;
    } finally {
      proposingMerge = false;
    }
  }

  function socialHref(s: SocialLink): string {
    const url = s.url;
    if (url.startsWith('http://') || url.startsWith('https://')) return url;
    return `https://${url}`;
  }

  function platformLabel(p: string): string {
    const labels: Record<string, string> = {
      ao3: 'Archive of Our Own',
      ffn: 'FanFiction.net',
      xitter: 'X / Twitter',
      bluesky: 'Bluesky',
      tumblr: 'Tumblr',
      discord: 'Discord',
      website: 'Website',
      other: 'Link',
    };
    return labels[p] ?? p;
  }
</script>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      {#if loading}
        <p class="archive-muted"><span class="spinner"></span> Loading author profile…</p>
      {:else if error}
        <h1 class="archive-page-title">Author</h1>
        <p class="archive-error">⚠️ {error}</p>
      {:else if profile}
        <div class="archive-title-row">
          {#if profile.avatar_url}
            <img class="avatar" src={profile.avatar_url} alt={`${profile.canonical_name} avatar`} />
          {/if}
          <h1 class="archive-page-title">{profile.canonical_name}</h1>
        </div>
        <p class="archive-muted">
          {profile.socials.length} social {profile.socials.length === 1 ? 'link' : 'links'} ·{' '}
          {profile.linked_authors.length} linked {profile.linked_authors.length === 1 ? 'account' : 'accounts'}
        </p>
      {/if}
    </header>

    {#if !loading && !error && profile}
      <!-- Bio -->
      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Bio</legend>
        {#if profile.bio}
          <p class="bio-text">{profile.bio}</p>
        {:else}
          <p class="archive-muted">No bio yet.</p>
        {/if}
        {#if isCurator}
          <div class="archive-form">
            <textarea class="archive-input" bind:value={bioDraft} rows="4" placeholder="Write a bio for this author…"></textarea>
            <button class="archive-btn archive-btn-primary" onclick={saveBio} disabled={savingBio}>
              {#if savingBio}<span class="spinner"></span>{:else}Save bio{/if}
            </button>
            {#if bioMsg}<p class="form-msg">{bioMsg}</p>{/if}
          </div>
        {/if}
      </fieldset>

      <!-- Avatar (curator) -->
      {#if isCurator}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Avatar</legend>
          <dl class="archive-dl">
            <div class="dl-row">
              <dt><label for="author-avatar-url">URL</label></dt>
              <dd class="archive-actions-inline">
                <input id="author-avatar-url" class="archive-input" type="url" bind:value={avatarDraft} placeholder="https://…" />
                <button class="archive-btn archive-btn-primary" onclick={saveAvatar} disabled={savingAvatar}>
                  {#if savingAvatar}<span class="spinner"></span>{:else}Save avatar{/if}
                </button>
                {#if avatarMsg}<span class="form-msg">{avatarMsg}</span>{/if}
              </dd>
            </div>
          </dl>
        </fieldset>
      {/if}

      <!-- Social links -->
      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Social Links</legend>
        {#if profile.socials.length === 0}
          <p class="archive-muted">No social links yet.</p>
        {:else}
          <ul class="archive-pref-list">
            {#each profile.socials as s (s.id)}
              <li class="archive-pref-item">
                <a href={socialHref(s)} target="_blank" rel="noopener noreferrer" class="archive-link">
                  {platformLabel(s.platform)}{#if s.label} — {s.label}{/if}
                </a>
                {#if isCurator}
                  <button class="archive-btn" onclick={() => handleRemoveSocial(s)} aria-label={`Remove ${s.platform} link`}>✕</button>
                {/if}
              </li>
            {/each}
          </ul>
        {/if}

        {#if isCurator}
          <form class="archive-form" onsubmit={(e) => { e.preventDefault(); handleAddSocial(); }}>
            <dl class="archive-dl">
              <div class="dl-row">
                <dt><label for="social-platform">Platform</label></dt>
                <dd>
                  <select id="social-platform" class="archive-input" bind:value={socialPlatform}>
                    {#each SOCIAL_PLATFORMS as p}
                      <option value={p}>{platformLabel(p)}</option>
                    {/each}
                  </select>
                </dd>
              </div>
              <div class="dl-row">
                <dt><label for="social-url">URL</label></dt>
                <dd><input id="social-url" class="archive-input" type="url" bind:value={socialUrl} placeholder="https://…" /></dd>
              </div>
              <div class="dl-row">
                <dt><label for="social-label">Label</label></dt>
                <dd><input id="social-label" class="archive-input" type="text" bind:value={socialLabel} placeholder="Label (optional)" /></dd>
              </div>
            </dl>
            <button class="archive-btn archive-btn-primary" type="submit" disabled={addingSocial}>
              {#if addingSocial}<span class="spinner"></span>{:else}Add{/if}
            </button>
            {#if socialMsg}<p class="form-msg">{socialMsg}</p>{/if}
          </form>
        {/if}
      </fieldset>

      <!-- Linked accounts -->
      {#if profile.linked_authors.length > 0}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Linked Accounts</legend>
          <ul class="archive-pref-list">
            {#each profile.linked_authors as l (l.id)}
              <li class="archive-pref-item">
                <span>
                  <strong>{l.source_author}</strong>
                  {' '}
                  <a href={l.source_url} target="_blank" rel="noopener noreferrer" class="archive-muted linked-url">{l.source_url}</a>
                </span>
              </li>
            {/each}
          </ul>
        </fieldset>
      {/if}

      <!-- Batch download (AO3/XenForo) -->
      {#if sourceUrl}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Batch Download</legend>
          <BatchAuthorDownload
            authorUrl={sourceUrl}
            authorName={profile.canonical_name}
            siteName={sourceSite}
          />
        </fieldset>
      {/if}

      <!-- Merge proposal (curator) -->
      {#if isCurator}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Link Another Account to This Author</legend>
          <blockquote class="archive-summary">
            Propose merging a pseudonym / account (e.g. from AO3) into this profile.
          </blockquote>
          <form class="archive-form" onsubmit={(e) => { e.preventDefault(); handleProposeMerge(); }}>
            <dl class="archive-dl">
              <div class="dl-row">
                <dt><label for="merge-author">Source author</label></dt>
                <dd><input id="merge-author" class="archive-input" type="text" bind:value={mergeSourceAuthor} placeholder="Source author name" /></dd>
              </div>
              <div class="dl-row">
                <dt><label for="merge-url">Source URL</label></dt>
                <dd><input id="merge-url" class="archive-input" type="url" bind:value={mergeSourceUrl} placeholder="https://archiveofourown.org/users/…" /></dd>
              </div>
              <div class="dl-row">
                <dt></dt>
                <dd>
                  <label class="archive-checkbox-label">
                    <input type="checkbox" bind:checked={mergeAutoApprove} />
                    Auto-approve (admins only)
                  </label>
                </dd>
              </div>
            </dl>
            <button class="archive-btn archive-btn-primary" type="submit" disabled={proposingMerge}>
              {#if proposingMerge}<span class="spinner"></span>{:else}Propose merge{/if}
            </button>
            {#if mergeMsg}<p class="form-msg">{mergeMsg}</p>{/if}
          </form>
        </fieldset>
      {/if}

      <p class="archive-muted back-link"><a class="archive-link" href="/authors">← Back to author search</a></p>
    {/if}
  </div>
</main>
{:else}
<!-- Modern mode (unchanged) -->
{#if loading}
  <div class="card"><p class="muted"><span class="spinner"></span> Loading author profile…</p></div>
{:else if error}
  <div class="card error-card"><strong class="error-text">⚠️ {error}</strong></div>
{:else if profile}
  <div class="author-page">
    <!-- Header -->
    <div class="author-header card">
      {#if profile.avatar_url}
        <img class="avatar" src={profile.avatar_url} alt={`${profile.canonical_name} avatar`} />
      {/if}
      <div class="author-heading">
        <h1>{profile.canonical_name}</h1>
        <p class="muted">
          {profile.socials.length} social {profile.socials.length === 1 ? 'link' : 'links'} ·{' '}
          {profile.linked_authors.length} linked {profile.linked_authors.length === 1 ? 'account' : 'accounts'}
        </p>
      </div>
    </div>

    <!-- Bio -->
    <div class="card section">
      <h3>Bio</h3>
      {#if profile.bio}
        <p class="bio-text">{profile.bio}</p>
      {:else}
        <p class="muted">No bio yet.</p>
      {/if}
      {#if isCurator}
        <div class="edit-row">
          <textarea bind:value={bioDraft} rows="4" placeholder="Write a bio for this author…"></textarea>
          <button class="btn btn-secondary" onclick={saveBio} disabled={savingBio}>
            {#if savingBio}<span class="spinner"></span>{:else}💾 Save bio{/if}
          </button>
          {#if bioMsg}<p class="form-msg">{bioMsg}</p>{/if}
        </div>
      {/if}
    </div>

    <!-- Avatar (curator) -->
    {#if isCurator}
      <div class="card section">
        <h3>Avatar</h3>
        <div class="edit-row">
          <input type="url" bind:value={avatarDraft} placeholder="https://…" />
          <button class="btn btn-secondary" onclick={saveAvatar} disabled={savingAvatar}>
            {#if savingAvatar}<span class="spinner"></span>{:else}💾 Save avatar{/if}
          </button>
          {#if avatarMsg}<p class="form-msg">{avatarMsg}</p>{/if}
        </div>
      </div>
    {/if}

    <!-- Social links -->
    <div class="card section">
      <h3>Social Links</h3>
      {#if profile.socials.length === 0}
        <p class="muted">No social links yet.</p>
      {:else}
        <ul class="social-list">
          {#each profile.socials as s (s.id)}
            <li class="social-item">
              <a href={socialHref(s)} target="_blank" rel="noopener noreferrer">
                {platformLabel(s.platform)}{#if s.label} — {s.label}{/if}
              </a>
              {#if isCurator}
                <button class="btn btn-small" onclick={() => handleRemoveSocial(s)} aria-label={`Remove ${s.platform} link`}>✕</button>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}

      {#if isCurator}
        <div class="edit-row social-form">
          <select bind:value={socialPlatform}>
            {#each SOCIAL_PLATFORMS as p}
              <option value={p}>{platformLabel(p)}</option>
            {/each}
          </select>
          <input type="url" bind:value={socialUrl} placeholder="https://…" />
          <input type="text" bind:value={socialLabel} placeholder="Label (optional)" />
          <button class="btn btn-secondary" onclick={handleAddSocial} disabled={addingSocial}>
            {#if addingSocial}<span class="spinner"></span>{:else}➕ Add{/if}
          </button>
        </div>
        {#if socialMsg}<p class="form-msg">{socialMsg}</p>{/if}
      {/if}
    </div>

    <!-- Linked accounts -->
    {#if profile.linked_authors.length > 0}
      <div class="card section">
        <h3>Linked Accounts</h3>
        <ul class="link-list">
          {#each profile.linked_authors as l (l.id)}
            <li>
              <strong>{l.source_author}</strong>
              <a href={l.source_url} target="_blank" rel="noopener noreferrer" class="muted">{l.source_url}</a>
            </li>
          {/each}
        </ul>
      </div>
    {/if}

    <!-- Batch download (AO3/XenForo) -->
    {#if sourceUrl}
      <div class="card section">
        <h3>Batch Download</h3>
        <BatchAuthorDownload
          authorUrl={sourceUrl}
          authorName={profile.canonical_name}
          siteName={sourceSite}
        />
      </div>
    {/if}

    <!-- Merge proposal (curator) -->
    {#if isCurator}
      <div class="card section">
        <h3>Link Another Account to This Author</h3>
        <p class="muted">Propose merging a pseudonym / account (e.g. from AO3) into this profile.</p>
        <div class="edit-row merge-form">
          <input type="text" bind:value={mergeSourceAuthor} placeholder="Source author name" />
          <input type="url" bind:value={mergeSourceUrl} placeholder="https://archiveofourown.org/users/…" />
          <label class="inline-label">
            <input type="checkbox" bind:checked={mergeAutoApprove} />
            Auto-approve (admins only)
          </label>
          <button class="btn btn-secondary" onclick={handleProposeMerge} disabled={proposingMerge}>
            {#if proposingMerge}<span class="spinner"></span>{:else}🔗 Propose merge{/if}
          </button>
        </div>
        {#if mergeMsg}<p class="form-msg">{mergeMsg}</p>{/if}
      </div>
    {/if}

    <p class="muted back-link"><a href="/authors">← Back to author search</a></p>
  </div>
{/if}
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
  .archive-title-row {
    display: flex;
    align-items: center;
    gap: 0.8rem;
  }
  .archive-page-title {
    font-size: 1.6em;
    font-weight: 700;
    margin: 0;
  }
  .avatar {
    width: 64px;
    height: 64px;
    object-fit: cover;
    border: 1px solid var(--archive-border, #dddddd);
    padding: 2px;
    background: var(--archive-bg, #ffffff);
  }
  .archive-muted {
    color: var(--archive-muted, #666666);
    font-style: italic;
  }
  .linked-url {
    text-decoration: none;
  }
  .archive-error {
    color: var(--archive-link, #990000);
    font-weight: 700;
    font-size: 0.9rem;
  }
  .archive-link {
    color: var(--archive-link, #990000);
    text-decoration: none;
    font-weight: 600;
  }
  .archive-link:hover {
    text-decoration: underline;
  }
  .archive-fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.6rem 0.9rem 0.75rem;
    margin: 0 0 1rem;
    background: var(--archive-bg, #ffffff);
  }
  .archive-fieldset legend,
  .archive-legend {
    font-weight: 700;
    font-size: 0.9em;
    color: var(--archive-link, #990000);
    padding: 0 0.4rem;
  }
  .bio-text { white-space: pre-wrap; }
  .archive-summary {
    margin: 0 0 0.5rem;
    padding: 0.3rem 0 0.3rem 0.6rem;
    border-left: 3px solid var(--archive-border, #dddddd);
    color: var(--archive-muted, #666666);
    font-style: italic;
    background: transparent;
  }
  .archive-dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.4rem 1rem;
    margin: 0.4rem 0 0.6rem;
  }
  .archive-dl dt {
    font-weight: 600;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-dl dd { margin: 0; }
  .archive-actions-inline {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .archive-input {
    padding: 0.3rem 0.5rem;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.9rem;
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
  }
  dl > .archive-dl dd .archive-input,
  dd > .archive-input { width: 100%; box-sizing: border-box; max-width: 420px; }
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
    font-size: 0.88rem;
    cursor: pointer;
  }
  .archive-checkbox-label input { width: auto; padding: 0; }
  .archive-form { display: flex; flex-direction: column; gap: 0.5rem; align-items: flex-start; }
  .archive-form textarea.archive-input { width: 100%; box-sizing: border-box; resize: vertical; }
  .archive-pref-list {
    list-style: none;
    margin: 0.3rem 0 0;
    padding: 0;
    border-top: 1px solid var(--archive-border, #eeeeee);
  }
  .archive-pref-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.45rem 0.2rem;
    border-bottom: 1px solid var(--archive-border, #eeeeee);
    font-size: 0.92rem;
  }
  .archive-pref-item:last-child { border-bottom: none; }
  .form-msg {
    font-size: 0.85rem;
    margin: 0.2rem 0 0;
  }
  .back-link { margin-top: 1.2rem; }

  /* ── Modern mode (scoped — no bleed) ── */
  .author-page {
    max-width: var(--max-width);
    margin: 0 auto;
    padding: 1rem;
  }
  .author-page .author-header {
    display: flex;
    align-items: center;
    gap: 1rem;
  }
  .author-page .avatar {
    width: 80px;
    height: 80px;
    border-radius: 50%;
    object-fit: cover;
  }
  .author-page .author-heading h1 {
    margin: 0 0 0.2rem;
    font-size: 1.5rem;
  }
  .author-page .section {
    margin-top: 1rem;
  }
  .author-page .section h3 {
    margin-top: 0;
    font-size: 1.05rem;
  }
  .author-page .bio-text {
    white-space: pre-wrap;
  }
  .author-page .edit-row {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin-top: 0.6rem;
  }
  .author-page .edit-row textarea {
    flex: 1 1 100%;
    min-width: 0;
  }
  .author-page .edit-row input,
  .author-page .edit-row select {
    flex: 1;
    min-width: 140px;
  }
  .author-page .social-list,
  .author-page .link-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
  .author-page .social-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.45rem 0.6rem;
    background: var(--color-surface-2);
    border-radius: var(--radius-sm);
  }
  .author-page .social-form select {
    flex: 0 0 160px;
  }
  .author-page .inline-label {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.85rem;
    color: var(--color-muted);
  }
  .author-page .form-msg {
    font-size: 0.85rem;
    margin: 0.4rem 0 0;
  }
  .author-page .btn-small {
    padding: 0.2rem 0.5rem;
    font-size: 0.8rem;
  }
  .author-page .back-link {
    margin-top: 1.5rem;
  }
</style>
