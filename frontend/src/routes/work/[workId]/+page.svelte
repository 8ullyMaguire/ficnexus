<script lang="ts">
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';
  import { getWork, addBookmark, removeBookmark, listBookmarks, rateWork, getRatings, giveKudos, removeKudos, getKudos, follow, unfollow, checkWorkFollow } from '$lib/api/social';
  import type { Work, WorkSource } from '$lib/api/social-types';
  import { auth } from '$lib/stores/auth.svelte';
  import ReactionBar from '$lib/components/ReactionBar.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import CommentSection from '$lib/components/CommentSection.svelte';
  import StarRating from '$lib/components/StarRating.svelte';
  import ReviewsSection from '$lib/components/ReviewsSection.svelte';
  import { formatWords, detectSite, stripHtml } from '$lib/util';

  const uiMode = $derived(getPref('uiMode'));

  let { data } = $props();
  const workId = $derived(Number(data.workId));

  let work = $state<Work | null>(null);
  let loading = $state(true);
  let error = $state('');

  // Selected source for downloading
  let selectedSource = $state<WorkSource | null>(null);

  // Bookmark state
  let isBookmarked = $state(false);
  let bookmarkNotes = $state('');
  let bookmarkPrivate = $state(false);
  let savingBookmark = $state(false);

  // Rating state (5-star scale, feedback rework)
  let avgRating = $state(0);
  let ratingCount = $state(0);
  let reviewCount = $state(0);
  let myRating = $state(0);
  let ratingLoading = $state(false);

  // Kudos state (AO3-style anonymous-appreciable likes)
  let kudosCount = $state(0);
  let guestCount = $state(0);
  let myKudos = $state(false);
  let kudosLoading = $state(false);

  // Work-level follow (subscription) state
  let isFollowingWork = $state(false);
  let workFollowId: number | null = $state(null);
  let followLoading = $state(false);

  // Download formats for selected source
  let downloads = $derived.by(() => {
    if (!selectedSource) return [];
    // The source URL is used for export - formats are fetched via /api/epub?q=<url>
    return [
      { label: 'EPUB', type: 'epub' },
      { label: 'HTML', type: 'html' },
      { label: 'TXT', type: 'txt' },
      { label: 'MD', type: 'md' },
      { label: 'MOBI', type: 'mobi' },
      { label: 'PDF', type: 'pdf' },
      { label: 'AZW3', type: 'azw3' },
    ];
  });

  onMount(async () => {
    await auth.init();
    await loadWork();
  });

  async function loadWork() {
    loading = true;
    error = '';
    try {
      const res = await getWork(workId);
      if (res.err !== 0) {
        error = 'Could not load work.';
        return;
      }
      work = res.work;
      // Select first source by default
      if (work.sources && work.sources.length > 0) {
        selectedSource = work.sources[0];
      }
      // Load secondary data in parallel
      await Promise.allSettled([loadBookmarkStatus(), loadRatings(), loadKudos(), loadFollowStatus()]);
    } catch {
      error = 'Network error loading work.';
    } finally {
      loading = false;
    }
  }

  async function loadBookmarkStatus() {
    if (!auth.isLoggedIn) return;
    try {
      const res = await listBookmarks();
      if (res.err === 0) {
        isBookmarked = res.bookmarks.some((b) => b.work_id === workId);
      }
    } catch { /* ignore */ }
  }

  async function toggleBookmark() {
    if (!auth.isLoggedIn || savingBookmark) return;
    savingBookmark = true;
    try {
      if (isBookmarked) {
        await removeBookmark(workId);
        isBookmarked = false;
      } else {
        await addBookmark(workId, bookmarkNotes || undefined, bookmarkPrivate);
        isBookmarked = true;
        bookmarkNotes = '';
      }
    } catch { /* ignore */ }
    savingBookmark = false;
  }

  async function loadRatings() {
    try {
      const res = await getRatings(workId);
      if (res.err === 0) {
        avgRating = res.avg_rating ?? 0;
        ratingCount = res.rating_count ?? 0;
        reviewCount = res.review_count ?? 0;
      }
    } catch { /* ignore */ }
    const saved = localStorage.getItem(`fichub_my_rating_${workId}`);
    if (saved) myRating = parseInt(saved, 10) || 0;
  }

  async function loadKudos() {
    try {
      const res = await getKudos(workId);
      if (res.err === 0) {
        kudosCount = res.kudos_count ?? 0;
        guestCount = res.guest_count ?? 0;
        myKudos = res.my_kudos === true;
      }
    } catch { /* ignore */ }
  }

  async function handleKudos() {
    if (!auth.isLoggedIn || kudosLoading) return;
    kudosLoading = true;
    try {
      const res = myKudos ? await removeKudos(workId) : await giveKudos(workId);
      if (res.err === 0) {
        kudosCount = res.kudos_count ?? 0;
        guestCount = res.guest_count ?? 0;
        myKudos = res.my_kudos === true;
      }
    } catch { /* ignore */ }
    kudosLoading = false;
  }

  async function loadFollowStatus() {
    if (!auth.isLoggedIn) return;
    try {
      const res = await checkWorkFollow(workId);
      if (res.err === 0) {
        isFollowingWork = res.is_following;
        workFollowId = res.follow_id ?? null;
      }
    } catch { /* ignore */ }
  }

  async function toggleFollow() {
    if (!auth.isLoggedIn || followLoading) return;
    followLoading = true;
    try {
      if (isFollowingWork && workFollowId !== null) {
        await unfollow(workFollowId);
        isFollowingWork = false;
        workFollowId = null;
      } else {
        const res = await follow('work', workId);
        if (res.err === 0) {
          await loadFollowStatus();
        }
      }
    } catch { /* ignore */ }
    followLoading = false;
  }

  async function handleRate(rating: number) {
    if (!auth.isLoggedIn || ratingLoading) return;
    ratingLoading = true;
    try {
      const res = await rateWork(workId, rating as 1 | 2 | 3 | 4 | 5);
      if (res.err === 0) {
        avgRating = res.avg_rating ?? 0;
        ratingCount = res.rating_count ?? 0;
        reviewCount = res.review_count ?? 0;
        myRating = rating;
        try { localStorage.setItem(`fichub_my_rating_${workId}`, String(rating)); } catch { /* ignore */ }
      }
    } catch { /* ignore */ }
    ratingLoading = false;
  }

  function getDownloadUrl(source: WorkSource, format: string): string {
    return `/api/epub?q=${encodeURIComponent(source.url)}&format=${format}`;
  }

  /** Split a comma-joined author string into individual author names. */
  function splitAuthors(authorStr: string): string[] {
    if (!authorStr) return ['Anonymous'];
    return authorStr
      .split(/, | and /)
      .map(a => a.trim())
      .filter(a => a.length > 0);
  }

  /** Generate author links for display. */
  function authorLinks(authorStr: string): Array<{ name: string; href: string }> {
    const authors = splitAuthors(authorStr);
    return authors.map(name => ({
      name,
      href: `/authors/${encodeURIComponent(name)}`
    }));
  }
</script>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      {#if loading}
        <p class="archive-muted"><span class="spinner"></span> {t('work.loading')}</p>
      {:else if error}
        <h1 class="archive-page-title">Work</h1>
        <p class="archive-error">⚠️ {error}</p>
        <p class="archive-muted">{t('work.errorHint')}</p>
      {:else if work}
        <h1 class="archive-page-title">{work.canonical_title}</h1>
        <p class="archive-byline">
          by {#each authorLinks(work.canonical_author) as link, i}
            {#if i > 0}, {/if}
            <a href={link.href} class="archive-link">{link.name}</a>
          {/each}
          {#if work.sources && work.sources.length > 1}
            <span class="archive-muted">· Available on {work.sources.length} sources</span>
          {/if}
        </p>
      {/if}
    </header>

    {#if !loading && !error && work}
      <!-- Source selector -->
      {#if work.sources && work.sources.length > 0}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">{t('work.selectSource')}</legend>
          <dl class="archive-dl">
            {#each work.sources as source}
              <div class="dl-row">
                <dt>
                  <label class="archive-radio">
                    <input type="radio" name="source" checked={selectedSource?.id === source.id} onchange={() => selectedSource = source} />
                    {detectSite(source.source)}
                  </label>
                </dt>
                <dd>{formatWords(source.words)} · {source.chapters} ch</dd>
              </div>
            {/each}
          </dl>
        </fieldset>
      {/if}

      <!-- Download formats -->
      {#if selectedSource}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Download</legend>
          <div class="archive-actions">
            {#each downloads as d}
              <a class="archive-btn" href={getDownloadUrl(selectedSource, d.type)} download>{d.label}</a>
            {/each}
          </div>
        </fieldset>
      {/if}

      <!-- Actions row -->
      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Actions</legend>
        {#if auth.isLoggedIn}
          <dl class="archive-dl">
            <div class="dl-row">
              <dt>Bookmark</dt>
              <dd class="archive-actions-inline">
                <button
                  class="archive-btn archive-btn-primary"
                  onclick={toggleBookmark}
                  disabled={savingBookmark}
                >
                  {#if savingBookmark}
                    <span class="spinner"></span>
                  {:else if isBookmarked}
                    {t('work.saved')}
                  {:else}
                    {t('work.save')}
                  {/if}
                </button>
                {#if !isBookmarked}
                  <input
                    type="text"
                    class="archive-input"
                    placeholder="Add a note…"
                    bind:value={bookmarkNotes}
                  />
                  <label class="archive-checkbox-label">
                    <input type="checkbox" bind:checked={bookmarkPrivate} />
                    Private
                  </label>
                {/if}
              </dd>
            </div>
            <div class="dl-row">
              <dt>Rating</dt>
              <dd class="archive-actions-inline">
                <StarRating
                  bind:value={myRating}
                  onrate={handleRate}
                />
                {#if ratingCount > 0 || reviewCount > 0}
                  <span class="rating-count">
                    {avgRating > 0 ? `${avgRating.toFixed(1)}★` : ''}
                    ({ratingCount + reviewCount})
                  </span>
                {/if}
              </dd>
            </div>
            <div class="dl-row">
              <dt>Kudos</dt>
              <dd class="archive-actions-inline">
                <button
                  class="archive-btn"
                  onclick={handleKudos}
                  disabled={kudosLoading}
                  aria-label={myKudos ? t('work.kudosRemove') : t('work.kudos')}
                >
                  {#if kudosLoading}
                    <span class="spinner"></span>
                  {:else if myKudos}
                    {t('work.kudosGiven')}
                  {:else}
                    {t('work.kudos')}
                  {/if}
                </button>
                {#if kudosCount > 0 || guestCount > 0}
                  <span class="rating-count">{t('work.kudosCount', { count: kudosCount })}</span>
                  {#if guestCount > 0}
                    <span class="archive-muted">{t('work.kudosGuests', { count: guestCount })}</span>
                  {/if}
                {/if}
              </dd>
            </div>
            <div class="dl-row">
              <dt>Reactions</dt>
              <dd><ReactionBar targetType="work" targetId={workId} /></dd>
            </div>
            <div class="dl-row">
              <dt>Updates</dt>
              <dd class="archive-actions-inline">
                {#if auth.isLoggedIn}
                  <button
                    class="archive-btn"
                    onclick={toggleFollow}
                    disabled={followLoading}
                    aria-label={isFollowingWork ? 'Unsubscribe' : 'Subscribe to updates'}
                  >
                    {#if followLoading}
                      <span class="spinner"></span>
                    {:else if isFollowingWork}
                      Subscribed
                    {:else}
                      Subscribe
                    {/if}
                  </button>
                {/if}
              </dd>
            </div>
          </dl>
        {:else}
          <p><a class="archive-link" href="/">{t('work.loginToRate')}</a></p>
        {/if}
      </fieldset>

      <!-- Description -->
      {#if work.description}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">{t('work.description')}</legend>
          <p class="archive-desc">{stripHtml(work.description)}</p>
        </fieldset>
      {/if}

      <!-- Stats -->
      <dl class="archive-dl archive-stats-dl">
        <div class="dl-row">
          <dt>Stats</dt>
          <dd class="archive-stats-links">
            <a class="archive-link" href={`/work/${workId}/stats`}>
              {work.total_bookmarks} {t('work.bookmarks')}
            </a>
            <a class="archive-link" href={`/work/${workId}/stats`}>
              {avgRating > 0 ? `${avgRating.toFixed(1)}★` : '—'} {t('work.rating')}
            </a>
            <a class="archive-link" href={`/work/${workId}/stats`}>
              {work.total_comments} {t('work.comments')}
            </a>
            {#if kudosCount > 0 || guestCount > 0}
              <a class="archive-link" href={`/work/${workId}/stats`}>
                {kudosCount + guestCount} {t('work.kudos')}
              </a>
            {/if}
          </dd>
        </div>
      </dl>

      <!-- Suggestions & Request Similar (AO3-style) -->
      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Suggestions</legend>
        <div class="archive-actions-inline">
          <a class="archive-btn" href={`/recommendations?work_id=${workId}`}>Find Similar Works</a>
          <a class="archive-btn" href={`/ask?ref=work_${workId}`}>Request a Recommendation</a>
        </div>
        <p class="archive-muted" style="margin: 0.5rem 0 0; font-size: 0.85em;">
          Looking for something like this? Browse works with similar tags or ask the community for personalized suggestions.
        </p>
      </fieldset>

      <!-- Comments -->
      <CommentSection workId={workId} />

      <!-- Reviews (feedback rework: in-depth written reviews + 5-star) -->
      <ReviewsSection workId={workId} />
    {/if}
  </div>
</main>
{:else}
<!-- Modern mode (unchanged) -->
{#if loading}
  <div class="card">
    <p class="muted"><span class="spinner"></span> {t('work.loading')}</p>
  </div>
{:else if error}
  <div class="card error-card">
    <strong class="error-text">⚠️ {error}</strong>
    <p class="muted">
      {t('work.errorHint')}
    </p>
  </div>
{:else if work}
  <!-- Header -->
  <div class="work-header">
    <h1>{work.canonical_title}</h1>
    <p class="muted">
      by {#each authorLinks(work.canonical_author) as link, i}
        {#if i > 0}, {/if}
        <a href={link.href} class="author-link">{link.name}</a>
      {/each}
    </p>
    {#if work.sources && work.sources.length > 1}
      <p class="source-count">Available on {work.sources.length} sources</p>
    {/if}
  </div>

  <!-- Source selector -->
  {#if work.sources && work.sources.length > 0}
    <div class="card source-card">
      <h3>{t('work.selectSource')}</h3>
      <div class="source-tabs">
        {#each work.sources as source}
          <button
            class="source-tab"
            class:active={selectedSource?.id === source.id}
            onclick={() => selectedSource = source}
          >
            <span class="source-badge">{detectSite(source.source)}</span>
            <span class="source-meta">{formatWords(source.words)} · {source.chapters} ch</span>
          </button>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Translation workspace -->
  <div class="card translation-card">
    <h3>Community translations</h3>
    <p class="muted">Submit chapter-by-chapter translations for curator review.</p>
    <a class="btn btn-secondary" href={`/work/${workId}/translate`}>Open translation editor</a>
  </div>

  <!-- Actions row -->
  <div class="actions-row">
    {#if selectedSource}
      <div class="download-group">
        {#each downloads as d}
          <a
            class="btn btn-secondary dl-btn"
            href={getDownloadUrl(selectedSource, d.type)}
            download
          >{d.label}</a>
        {/each}
      </div>
    {/if}

    {#if auth.isLoggedIn}
      <div class="bookmark-group">
        <button
          class="btn bookmark-btn"
          class:bookmarked={isBookmarked}
          onclick={toggleBookmark}
          disabled={savingBookmark}
        >
          {#if savingBookmark}
            <span class="spinner"></span>
          {:else if isBookmarked}
            {t('work.saved')}
          {:else}
            {t('work.save')}
          {/if}
        </button>
        {#if !isBookmarked}
          <input
            type="text"
            class="bookmark-notes"
            placeholder="Add a note…"
            bind:value={bookmarkNotes}
          />
          <label class="private-label">
            <input type="checkbox" bind:checked={bookmarkPrivate} />
            Private
          </label>
        {/if}
      </div>

      <div class="rating-group">
        <StarRating
          bind:value={myRating}
          onrate={handleRate}
        />
        {#if ratingCount > 0 || reviewCount > 0}
          <span class="rating-count">
            {avgRating > 0 ? `${avgRating.toFixed(1)}★` : ''}
            ({ratingCount + reviewCount})
          </span>
        {/if}
      </div>

      <div class="kudos-group">
        <button
          class="btn kudos-btn"
          class:kudoed={myKudos}
          onclick={handleKudos}
          disabled={kudosLoading}
          aria-label={myKudos ? t('work.kudosRemove') : t('work.kudos')}
        >
          {#if kudosLoading}
            <span class="spinner"></span>
          {:else if myKudos}
            {t('work.kudosGiven')}
          {:else}
            {t('work.kudos')}
          {/if}
        </button>
        {#if kudosCount > 0 || guestCount > 0}
          <span class="kudos-count">{t('work.kudosCount', { count: kudosCount })}</span>
          {#if guestCount > 0}
            <span class="kudos-guests">{t('work.kudosGuests', { count: guestCount })}</span>
          {/if}
        {/if}
      </div>  <!-- kudos-group end -->
      <ReactionBar targetType="work" targetId={workId} />

      {#if auth.isLoggedIn}
        <button
          class="btn follow-btn"
          class:following={isFollowingWork}
          onclick={toggleFollow}
          disabled={followLoading}
          aria-label={isFollowingWork ? 'Unsubscribe' : 'Subscribe to updates'}
        >
          {#if followLoading}
            <span class="spinner"></span>
          {:else if isFollowingWork}
            Subscribed
          {:else}
            Subscribe
          {/if}
        </button>
      {/if}
    {:else}
      <a class="btn btn-secondary" href="/">{t('work.loginToRate')}</a>
    {/if}
  </div>

  <!-- Description -->
  {#if work.description}
    <div class="card desc-card">
      <h3>{t('work.description')}</h3>
      <p class="desc">{stripHtml(work.description)}</p>
    </div>
  {/if}

  <!-- Stats -->
  <div class="card stats-card">
    <a href={`/work/${workId}/stats`} class="stat stat-link">
      <span class="stat-value">{work.total_bookmarks}</span>
      <span class="stat-label">{t('work.bookmarks')}</span>
    </a>
    <a href={`/work/${workId}/stats`} class="stat stat-link">
      <span class="stat-value">{avgRating > 0 ? `${avgRating.toFixed(1)}★` : '—'}</span>
      <span class="stat-label">{t('work.rating')}</span>
    </a>
    <a href={`/work/${workId}/stats`} class="stat stat-link">
      <span class="stat-value">{work.total_comments}</span>
      <span class="stat-label">{t('work.comments')}</span>
    </a>
    {#if kudosCount > 0 || guestCount > 0}
      <a href={`/work/${workId}/stats`} class="stat stat-link">
        <span class="stat-value">{kudosCount + guestCount}</span>
        <span class="stat-label">{t('work.kudos')}</span>
      </a>
    {/if}
  </div>

  <!-- Comments -->
  <CommentSection workId={workId} />

  <!-- Reviews (feedback rework: in-depth written reviews + 5-star) -->
  <ReviewsSection workId={workId} />
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
  .archive-page-title {
    font-size: 1.6em;
    font-weight: 700;
    margin: 0;
  }
  .archive-byline { margin: 0.3rem 0 0; }
  .archive-muted {
    color: var(--archive-muted, #666666);
    font-style: italic;
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
  .archive-dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.4rem 1rem;
    margin: 0.4rem 0 0;
  }
  .archive-dl dt {
    font-weight: 600;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-dl dd { margin: 0; }
  .archive-radio {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    cursor: pointer;
  }
  .archive-radio input { margin: 0; }
  .archive-actions,
  .archive-actions-inline {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .archive-btn {
    padding: 0.28rem 0.7rem;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-text, #2a2a2a);
    font-family: inherit;
    font-size: 0.85rem;
    text-decoration: none;
    cursor: pointer;
    display: inline-block;
  }
  .archive-btn:hover { background: var(--archive-border, #dddddd); }
  .archive-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .archive-btn-primary {
    color: var(--archive-bg, #ffffff);
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }
  .archive-btn-primary:hover { background: var(--archive-link-visited, #660066); border-color: var(--archive-link-visited, #660066); }
  .archive-input {
    padding: 0.3rem 0.5rem;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.88rem;
    width: 160px;
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
  }
  .archive-checkbox-label {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.85rem;
    cursor: pointer;
  }
  .archive-checkbox-label input { width: auto; padding: 0; }
  .archive-desc {
    white-space: pre-wrap;
    line-height: 1.6;
    margin: 0.2rem 0 0;
  }
  .archive-stats-dl dt { min-width: 3.2rem; }
  .archive-stats-links {
    display: flex;
    gap: 1.2rem;
    flex-wrap: wrap;
  }
  .rating-count {
    font-weight: 700;
    font-size: 0.9rem;
    color: var(--archive-muted, #666666);
  }

  /* ── Modern mode (unchanged) ── */
  .work-header h1 {
    margin: 0 0 0.3rem;
    font-size: 1.6rem;
  }
  .author-link {
    color: var(--color-primary);
  }
  .source-count {
    margin: 0.3rem 0;
    font-size: 0.9rem;
    color: var(--color-muted);
  }
  .source-card h3 {
    margin-top: 0;
    margin-bottom: 0.8rem;
    font-size: 1rem;
  }
  .source-tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  .source-tab {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.2rem;
    padding: 0.6rem 1rem;
    background: var(--color-surface-2);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: border-color 0.15s;
  }
  .source-tab:hover {
    border-color: var(--color-primary);
  }
  .source-tab.active {
    border-color: var(--color-primary);
    background: var(--color-primary-alpha);
  }
  .source-badge {
    font-weight: 600;
    font-size: 0.85rem;
  }
  .source-meta {
    font-size: 0.75rem;
    color: var(--color-muted);
  }
  .actions-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 1rem;
    margin: 1.2rem 0;
  }
  .download-group {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  .dl-btn {
    text-decoration: none;
  }
  .bookmark-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .bookmark-btn {
    background: var(--color-surface-2);
    color: var(--color-text);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 0.5rem 0.9rem;
    font-weight: 600;
    font-size: 0.9rem;
  }
  .bookmark-btn.bookmarked {
    background: var(--color-primary);
    color: white;
    border-color: var(--color-primary);
  }
  .bookmark-notes {
    width: 160px;
    padding: 0.4rem 0.6rem;
    font-size: 0.85rem;
  }
  .private-label {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.82rem;
    color: var(--color-muted);
    cursor: pointer;
  }
  .private-label input {
    padding: 0;
    width: auto;
  }
  .rating-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .kudos-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .kudos-btn {
    background: var(--color-surface-2);
    color: var(--color-text);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 0.5rem 0.9rem;
    font-weight: 600;
    font-size: 0.9rem;
  }
  .kudos-btn.kudoed {
    background: var(--color-primary);
    color: white;
    border-color: var(--color-primary);
  }
  .kudos-count {
    font-family: var(--mono);
    font-weight: 600;
    font-size: 0.9rem;
    color: var(--color-muted);
  }
  .kudos-guests {
    font-size: 0.8rem;
    color: var(--color-muted);
  }
  .follow-btn {
    background: var(--color-surface-2);
    color: var(--color-text);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 0.5rem 0.9rem;
    font-weight: 600;
    font-size: 0.9rem;
  }
  .follow-btn.following {
    background: var(--color-primary);
    color: white;
    border-color: var(--color-primary);
  }
  .stat-link {
    text-decoration: none;
    color: inherit;
  }
  .stat-link:hover {
    text-decoration: underline;
  }
  .desc-card h3 {
    margin-top: 0;
    margin-bottom: 0.5rem;
    font-size: 1rem;
  }
  .desc {
    color: var(--color-muted);
    font-size: 0.9rem;
    line-height: 1.6;
    white-space: pre-wrap;
  }
  .stats-card {
    display: flex;
    gap: 2rem;
    justify-content: center;
    margin-top: 1rem;
  }
  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
  }
  .stat-value {
    font-size: 1.4rem;
    font-weight: 700;
    color: var(--color-text);
  }
  .stat-label {
    font-size: 0.8rem;
    color: var(--color-muted);
  }
  .error-card {
    border-color: var(--color-error);
  }
  @media (max-width: 600px) {
    .actions-row {
      flex-direction: column;
      align-items: flex-start;
    }
    .stats-card {
      gap: 1rem;
    }
  }
</style>
