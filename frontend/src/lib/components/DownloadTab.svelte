<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchExport, convertFormat, ApiError } from '$lib/api/client';
  import type { ExportResponse } from '$lib/api/types';
  import { formatWords, detectSite, stripHtml } from '$lib/util';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref, setPref } from '$lib/prefs';
  import {
    addBookmark,
    removeBookmark,
    listBookmarks,
    rateWork,
    getRatings,
    addComment,
    listComments,
  } from '$lib/api/social';
  import type { Comment } from '$lib/api/social-types';
  import StarRating from '$lib/components/StarRating.svelte';

  let url = $state('');
  let loading = $state(false);
  let error = $state('');
  let result = $state<ExportResponse | null>(null);

  // Interface style: 'archive' (AO3-style) or 'modern'.
  const uiMode = $derived(getPref('uiMode'));

  onMount(() => {
    // Home dashboard hands off a URL via sessionStorage before navigating
    // to /download. Start the export automatically once mounted.
    const pre = sessionStorage.getItem('fichub_dl_url');
    if (pre) {
      url = pre;
      sessionStorage.removeItem('fichub_dl_url');
      handleDownload();
    }
  });

  // ── Bookmark state ──
  let isBookmarked = $state(false);
  let bookmarkLoading = $state(false);

  // ── Rating state (5-star scale, feedback rework) ──
  let avgRating = $state(0);
  let ratingCount = $state(0);
  let reviewCount = $state(0);
  let myRating = $state<number>(0);
  let ratingLoading = $state(false);

  // ── Comment state ──
  let comments = $state<Comment[]>([]);
  let newComment = $state('');
  let commentsLoading = $state(false);
  let commentSubmitting = $state(false);
  let commentsError = $state('');

  const urlId = $derived(result?.url_id ?? null);
  // Social APIs use the numeric work_id (not the string url_id).
  const workId = $derived(result?.meta?.work_id ?? null);

  // Load bookmarks to check if this fic is already bookmarked
  async function checkBookmark() {
    if (!auth.isLoggedIn) return;
    try {
      const res = await listBookmarks();
      if (res.err === 0 && res.bookmarks) {
        isBookmarked = res.bookmarks.some((b) => b.work_id === workId);
      }
    } catch {
      // silently ignore — bookmark status is non-critical
    }
  }

  async function toggleBookmark() {
    if (!workId || bookmarkLoading) return;
    bookmarkLoading = true;
    try {
      if (isBookmarked) {
        await removeBookmark(workId);
        isBookmarked = false;
      } else {
        await addBookmark(workId);
        isBookmarked = true;
      }
    } catch {
      // ignore — keep current state
    } finally {
      bookmarkLoading = false;
    }
  }

  // Load ratings for the current fic
  async function loadRatings() {
    if (!workId) return;
    try {
      const res = await getRatings(workId);
      if (res.err === 0) {
        avgRating = res.avg_rating ?? 0;
        ratingCount = res.rating_count ?? 0;
        reviewCount = res.review_count ?? 0;
      }
    } catch {
      // ignore
    }
    const saved = localStorage.getItem(`fichub_my_rating_${workId}`);
    if (saved) myRating = parseInt(saved, 10) || 0;
  }

  async function handleRate(rating: number) {
    if (!workId || ratingLoading) return;
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
    } catch {
      // ignore
    } finally {
      ratingLoading = false;
    }
  }

  // Load comments for the current fic
  async function loadComments() {
    if (!workId) return;
    commentsLoading = true;
    commentsError = '';
    try {
      const res = await listComments(workId);
      if (res.err === 0 && res.comments) {
        comments = res.comments;
      }
    } catch (e) {
      commentsError = 'Could not load comments.';
    } finally {
      commentsLoading = false;
    }
  }

  async function submitComment() {
    if (!workId || !newComment.trim() || commentSubmitting) return;
    commentSubmitting = true;
    commentsError = '';
    try {
      const res = await addComment(workId, newComment.trim());
      if (res.err === 0) {
        newComment = '';
        await loadComments();
      } else {
        commentsError = 'Failed to post comment.';
      }
    } catch {
      commentsError = 'Failed to post comment.';
    } finally {
      commentSubmitting = false;
    }
  }

  async function handleDownload() {
    if (!url.trim()) {
      error = 'Please paste a fanfiction URL first.';
      return;
    }
    loading = true;
    error = '';
    result = null;
    try {
      const res = await fetchExport(url.trim());
      if (res.err !== 0) {
        error = res.msg || 'Something went wrong with that URL.';
        return;
      }
      result = res;
    } catch (e) {
      if (e instanceof ApiError) {
        error = `Server returned ${e.status}. Is the backend running?`;
      } else {
        error = e instanceof Error ? e.message : 'Network error.';
      }
    } finally {
      loading = false;
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') handleDownload();
  }

  // When a new result arrives and user is logged in, fetch bookmark status, ratings, and comments
  $effect(() => {
    const id = urlId;
    if (id && auth.isLoggedIn) {
      checkBookmark();
      loadRatings();
      loadComments();
    }
  });

  // Build list of available download formats from response.
  // Calibre-backed formats (mobi/pdf/azw3) AND native formats whose export
  // rows are missing (e.g. legacy exports) convert on demand via
  // GET /api/epub/convert when clicked (lazy ⚡). Everything else is a direct
  // link.
  let downloads = $derived.by(() => {
    if (!result) return [];
    const out: {
      label: string;
      type: string;
      href?: string | null;
      lazy?: boolean;
    }[] = [];
    const add = (label: string, type: string, href?: string | null, lazy = false) => {
      if (href) out.push({ label, type, href });
      else if (lazy) out.push({ label, type, href: null, lazy: true });
    };
    add('EPUB', 'epub', result.epub_url);
    add('MOBI', 'mobi', result.mobi_url, true);
    add('AZW3', 'azw3', result.azw3_url, true);
    add('PDF', 'pdf', result.pdf_url, true);
    add('DOCX', 'docx', result.docx_url, true);
    add('KEPUB', 'kepub', result.kepub_url, true);
    add('FB2', 'fb2', result.fb2_url, true);
    add('HTML', 'html', result.html_url, true);
    add('TXT', 'txt', result.txt_url, true);
    add('MD', 'md', result.md_url, true);
    return out;
  });

  // ── Default format pref ────────────────────────────────────────────────
  // Persisted in localStorage by lib/prefs.ts. The "Preferred format"
  // dropdown lets the user pick it explicitly; clicking any download button
  // also sets it. The preferred button is highlighted and sorted first so a
  // one-click download is always available.
  let defaultFormat = $state(getPref('defaultFormat'));

  // Formats present in the current result (dropdown options).
  let availableFormats = $derived.by(() =>
    downloads.map((d) => ({ label: d.label, type: d.type })),
  );

  // Types that have a ready download right now. Lazy formats without a
  // rendered URL don't count as "available" for fallback purposes.
  let directTypes = $derived.by(() => downloads.filter((d) => d.href).map((d) => d.type));

  // The stored pref may name a format the current result doesn't have
  // (e.g. MOBI on a source site that only renders EPUB/HTML). Fall back to
  // the first available format for highlighting/sorting without clobbering
  // the stored preference.
  let effectiveFormat = $derived.by(() => {
    if (defaultFormat && directTypes.includes(defaultFormat)) return defaultFormat;
    return directTypes[0] ?? 'epub';
  });

  // Preferred format first, then the rest.
  let sortedDownloads = $derived.by(() => {
    const list = [...downloads];
    const idx = list.findIndex((d) => d.type === effectiveFormat);
    if (idx > 0) {
      const [pref] = list.splice(idx, 1);
      list.unshift(pref);
    }
    return list;
  });

  function selectFormat(type: string) {
    setPref('defaultFormat', type);
    defaultFormat = type;
  }

  // ── Lazy (on-demand) format conversion ───────────────────────────────
  // MOBI/PDF/AZW3 convert via Calibre only when clicked. Tracks in-flight
  // conversion per format so the button shows a spinner and is disabled
  // until the download URL is ready.
  let converting = $state<Record<string, boolean>>({});
  let convertError = $state<Record<string, string>>({});

  async function handleFormatClick(d: { label: string; type: string; href?: string | null; lazy?: boolean }) {
    // If a URL is ready, just download (normal anchor behavior).
    if (d.href) return;
    if (!d.lazy || converting[d.type]) return;
    converting[d.type] = true;
    convertError[d.type] = '';
    try {
      const res = await convertFormat(url, d.type);
      if (res.err !== 0) {
        convertError[d.type] = res.msg || 'Conversion failed.';
        return;
      }
      if (res.url) {
        // Trigger the download, then update the result so the button becomes
        // a normal link.
        const a = document.createElement('a');
        a.href = res.url;
        a.download = '';
        a.rel = 'noopener';
        document.body.appendChild(a);
        a.click();
        a.remove();
        if (result) {
          result = { ...result, [`${d.type}_url`]: res.url };
        }
      }
    } catch {
      convertError[d.type] = 'Conversion failed.';
    } finally {
      converting[d.type] = false;
    }
  }
</script>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<div class="dl-archive">
  <div class="dl-arc-search">
    <input
      class="arc-input"
      type="url"
      placeholder={t('dl.pasteUrl')}
      bind:value={url}
      onkeydown={onKeydown}
      disabled={loading}
      aria-label="Fanfiction URL"
    />
    <button class="archive-btn" type="button" onclick={handleDownload} disabled={loading}>
      {#if loading}
        {t('dl.working')}
      {:else}
        {t('dl.download')}
      {/if}
    </button>
  </div>

  {#if error}
    <p class="dl-arc-error">
      {error}
      <span class="dl-arc-sites">{t('dl.supportedSites')}</span>
    </p>
  {/if}

  {#if result && result.meta}
    {@const m = result.meta}
    <dl class="archive-dl dl-arc-dl">
      <div class="dl-row">
        <dt>Title</dt>
        <dd><strong>{m.title}</strong></dd>
      </div>
      <div class="dl-row">
        <dt>Author</dt>
        <dd>by {m.author} · {detectSite(m.source)}</dd>
      </div>
      <div class="dl-row">
        <dt>Stats</dt>
        <dd>
          {formatWords(m.words)} {t('home.words')} · {m.chapters} {t('home.chapters')} · {t('dl.status')} {m.status}
        </dd>
      </div>
      {#if m.description}
        <div class="dl-row">
          <dt>Summary</dt>
          <dd class="dl-arc-desc">{stripHtml(m.description).slice(0, 300)}</dd>
        </div>
      {/if}
      {#if downloads.length > 0}
        <div class="dl-row">
          <dt>
            <label for="dl-arc-pref-format">{t('dl.chooseFormat')}</label>
          </dt>
          <dd>
            <select
              id="dl-arc-pref-format"
              class="arc-select"
              value={defaultFormat}
              onchange={(e) => selectFormat((e.target as HTMLSelectElement).value)}
              aria-label={t('dl.chooseFormat')}
            >
              {#each availableFormats as f}
                <option value={f.type}>{f.label}</option>
              {/each}
            </select>
          </dd>
        </div>
        <div class="dl-row dl-formats-row">
          <dt>Formats</dt>
          <dd>
            <span class="dl-arc-buttons">
              {#each sortedDownloads as d}
                {#if d.href}
                  <a
                    class="archive-btn"
                    class:preferred={d.type === effectiveFormat}
                    href={d.href}
                    download
                    onclick={() => selectFormat(d.type)}
                  >{d.label}</a>
                {:else if d.lazy}
                  <button
                    class="archive-btn"
                    class:preferred={d.type === effectiveFormat}
                    type="button"
                    onclick={() => handleFormatClick(d)}
                    disabled={converting[d.type]}
                    aria-label={`Convert and download ${d.label}`}
                  >
                    {#if converting[d.type]}
                      {d.label}…
                    {:else}
                      {d.label}
                    {/if}
                  </button>
                {/if}
              {/each}
            </span>
            {#each sortedDownloads.filter((d) => !d.href && convertError[d.type]) as d (d.type)}
              <span class="dl-arc-convert-error">{convertError[d.type]}</span>
            {/each}
            <span class="dl-arc-hint">{t('dl.prefFormat')} <strong>{effectiveFormat}</strong></span>
          </dd>
        </div>
      {:else}
        <div class="dl-row">
          <dt>Formats</dt>
          <dd>{result.notes?.[0] ?? t('dl.notAvailable')}</dd>
        </div>
      {/if}
      {#if result.notes && result.notes.length > 0 && downloads.length > 0}
        <div class="dl-row">
          <dt>Notes</dt>
          <dd class="dl-arc-desc">{result.notes[0]}</dd>
        </div>
      {/if}
    </dl>

    {#if auth.isLoggedIn}
      <div class="dl-arc-actions">
        <button
          class="archive-btn"
          class:active={isBookmarked}
          type="button"
          onclick={toggleBookmark}
          disabled={bookmarkLoading}
          title={isBookmarked ? t('dl.removeBookmark') : t('dl.addBookmark')}
        >
          {isBookmarked ? t('dl.bookmarked') : t('dl.bookmark')}
        </button>

        <span class="dl-arc-rating">
          <StarRating
            bind:value={myRating}
            onrate={handleRate}
          />
          {#if ratingCount > 0 || reviewCount > 0}
            <span class="dl-arc-rating-count">
              {avgRating > 0 ? `${avgRating.toFixed(1)}★` : ''}
              ({ratingCount + reviewCount})
            </span>
          {/if}
        </span>
      </div>

      <!-- Comments section -->
      <div class="dl-arc-comments">
        <h3>{t('dl.commentsCount', { count: comments.length })}</h3>

        {#if commentsError}
          <p class="dl-arc-error">{commentsError}</p>
        {/if}

        {#if commentsLoading}
          <p class="dl-arc-muted">{t('dl.loadingComments')}</p>
        {:else if comments.length === 0}
          <p class="dl-arc-muted">{t('dl.noComments')}</p>
        {:else}
          <dl class="archive-dl dl-arc-comment-dl">
            {#each comments as c}
              <div class="dl-row">
                <dt>{c.user?.username ?? 'Anonymous'}</dt>
                <dd>
                  <span class="dl-arc-comment-date">{new Date(c.created_at).toLocaleDateString()}</span>
                  {c.body}
                </dd>
              </div>
            {/each}
          </dl>
        {/if}

        <div class="dl-arc-comment-form">
          <textarea
            class="arc-input"
            placeholder={t('dl.writeComment')}
            bind:value={newComment}
            rows="3"
            disabled={commentSubmitting}
          ></textarea>
          <button
            class="archive-btn"
            type="button"
            onclick={submitComment}
            disabled={commentSubmitting || !newComment.trim()}
          >
            {commentSubmitting ? t('dl.posting') : t('dl.postComment')}
          </button>
        </div>
      </div>
    {/if}
  {/if}

  <p class="dl-arc-muted dl-arc-tip">
    {t('dl.tip')}
    <!-- svelte-ignore a11y_invalid_attribute -->
    <a
      class="dl-arc-bookmarklet"
      draggable="true"
      href="javascript:location.href=location.origin+'/api/epub?q='+encodeURIComponent(location.href)"
      >FicNexus ↗</a
    >
  </p>
</div>
{:else}
<div class="download-tab">
  <div class="search-row">
    <input
      type="url"
      placeholder={t('dl.pasteUrl')}
      bind:value={url}
      onkeydown={onKeydown}
      disabled={loading}
      aria-label="Fanfiction URL"
    />
    <button class="btn" onclick={handleDownload} disabled={loading}>
      {#if loading}
        <span class="spinner"></span> {t('dl.working')}
      {:else}
        {t('dl.download')}
      {/if}
    </button>
  </div>

  {#if error}
    <div class="card error-card">
      <strong class="error-text">⚠️ {error}</strong>
      <p class="muted">
        {t('dl.supportedSites')}
      </p>
    </div>
  {/if}

  {#if result && result.meta}
    {@const m = result.meta}
    <div class="card result-card">
      <h2>{m.title}</h2>
      <p class="muted">by {m.author} · <span class="tag">{detectSite(m.source)}</span></p>
      <p class="meta-line">
        {formatWords(m.words)} {t('home.words')} · {m.chapters} {t('home.chapters')} · {t('dl.status')} {m.status}
      </p>
      {#if m.description}
        <p class="desc">{stripHtml(m.description).slice(0, 300)}</p>
      {/if}

      {#if downloads.length > 0}
        <div class="pref-row">
          <label class="pref-label" for="dl-pref-format">{t('dl.chooseFormat')}</label>
          <select
            id="dl-pref-format"
            class="pref-select"
            value={defaultFormat}
            onchange={(e) => selectFormat((e.target as HTMLSelectElement).value)}
            aria-label={t('dl.chooseFormat')}
          >
            {#each availableFormats as f}
              <option value={f.type}>{f.label}</option>
            {/each}
          </select>
        </div>
        <div class="downloads">
          {#each sortedDownloads as d}
            {#if d.href}
              <a
                class="btn btn-secondary dl-btn"
                class:preferred={d.type === effectiveFormat}
                href={d.href}
                download
                onclick={() => selectFormat(d.type)}
              >{d.label}</a>
            {:else if d.lazy}
              <button
                class="btn btn-secondary dl-btn"
                class:preferred={d.type === effectiveFormat}
                onclick={() => handleFormatClick(d)}
                disabled={converting[d.type]}
                aria-label={`Convert and download ${d.label}`}
              >
                {#if converting[d.type]}
                  <span class="spinner"></span> {d.label}
                {:else}
                  {d.label} ⚡
                {/if}
              </button>
              {#if convertError[d.type]}
                <span class="muted error-text convert-error">{convertError[d.type]}</span>
              {/if}
            {/if}
          {/each}
        </div>
        <p class="muted format-hint">{t('dl.prefFormat')} <strong>{effectiveFormat}</strong></p>
      {:else}
        <p class="muted">
          {result.notes?.[0] ?? t('dl.notAvailable')}
        </p>
      {/if}

      {#if result.notes && result.notes.length > 0 && downloads.length > 0}
        <p class="note muted">{result.notes[0]}</p>
      {/if}

      <!-- ── Auth-gated actions: Bookmark / Rate ── -->
      {#if auth.isLoggedIn}
        <div class="fic-actions">
          <button
            class="action-btn"
            class:active={isBookmarked}
            onclick={toggleBookmark}
            disabled={bookmarkLoading}
            title={isBookmarked ? t('dl.removeBookmark') : t('dl.addBookmark')}
          >
            <span class="icon">{isBookmarked ? '♥' : '♡'}</span>
            <span>{isBookmarked ? t('dl.bookmarked') : t('dl.bookmark')}</span>
          </button>

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
        </div>

        <!-- ── Comments section ── -->
        <div class="comments-section">
          <h3>{t('dl.commentsCount', { count: comments.length })}</h3>

          {#if commentsError}
            <p class="muted error-text">{commentsError}</p>
          {/if}

          {#if commentsLoading}
            <p class="muted">{t('dl.loadingComments')}</p>
          {:else if comments.length === 0}
            <p class="muted">{t('dl.noComments')}</p>
          {:else}
            <ul class="comment-list">
              {#each comments as c}
                <li class="comment-item">
                  <div class="comment-header">
                    <strong>{c.user?.username ?? 'Anonymous'}</strong>
                    <span class="muted comment-date">{new Date(c.created_at).toLocaleDateString()}</span>
                  </div>
                  <p class="comment-body">{c.body}</p>
                </li>
              {/each}
            </ul>
          {/if}

          <div class="comment-form">
            <textarea
              placeholder={t('dl.writeComment')}
              bind:value={newComment}
              rows="3"
              disabled={commentSubmitting}
            ></textarea>
            <button
              class="btn btn-secondary"
              onclick={submitComment}
              disabled={commentSubmitting || !newComment.trim()}
            >
              {commentSubmitting ? t('dl.posting') : t('dl.postComment')}
            </button>
          </div>
        </div>
      {/if}
    </div>
  {/if}

  <p class="muted hint">
    {t('dl.tip')}
    <!-- svelte-ignore a11y_invalid_attribute -->
    <a
      class="bookmarklet"
      draggable="true"
      href="javascript:location.href=location.origin+'/api/epub?q='+encodeURIComponent(location.href)"
      >FicNexus ↗</a
    >
  </p>
</div>
{/if}

<style>
  /* ── Archive mode ─────────────────────────────────────────── */
  .dl-archive {
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .dl-arc-search {
    display: flex;
    gap: 0.6rem;
    margin-bottom: 1rem;
  }
  .arc-input {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 0.95em;
    padding: 0.35rem 0.6rem;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
  }
  .dl-arc-search .arc-input { flex: 1; }
  .dl-arc-comment-form .arc-input {
    width: 100%;
    resize: vertical;
    margin-bottom: 0.5rem;
  }
  .archive-btn {
    display: inline-block;
    padding: 0.3rem 0.85rem;
    font-size: 0.88em;
    font-family: inherit;
    font-weight: 600;
    color: var(--archive-text, #2a2a2a);
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    cursor: pointer;
    line-height: 1.5;
    text-decoration: none;
  }
  .archive-btn:hover:not(:disabled) {
    background: var(--archive-border, #dddddd);
    border-color: var(--archive-muted, #666666);
    text-decoration: none;
  }
  .archive-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .archive-btn.preferred,
  .dl-arc-actions .archive-btn.active {
    color: var(--archive-bg, #ffffff);
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }
  .arc-select {
    font-family: inherit;
    font-size: 0.9em;
    padding: 0.25rem 0.4rem;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
  }
  .dl-arc-error {
    color: var(--color-error, #cc0000);
    font-weight: 600;
    margin-bottom: 0.8rem;
  }
  .dl-arc-sites {
    display: block;
    font-weight: 400;
    color: var(--archive-muted, #666666);
  }
  .archive-dl.dl-arc-dl,
  .archive-dl.dl-arc-comment-dl {
    margin: 0;
  }
  .dl-arc-dl .dl-row,
  .dl-arc-comment-dl .dl-row {
    display: grid;
    grid-template-columns: 140px 1fr;
    gap: 0.3em 1em;
    padding: 0.45em 0;
    border-bottom: 1px solid var(--archive-border, #eeeeee);
  }
  .dl-arc-dl .dl-row:last-child,
  .dl-arc-comment-dl .dl-row:last-child { border-bottom: none; }
  .dl-arc-dl dt,
  .dl-arc-comment-dl dt {
    font-weight: 700;
    font-size: 0.9em;
    color: var(--archive-muted, #666666);
  }
  .dl-arc-comment-dl dt { font-size: 0.95em; }
  .dl-arc-dl dd,
  .dl-arc-comment-dl dd {
    margin: 0;
    font-size: 0.92em;
  }
  .dl-formats-row dd {
    display: flex;
    flex-direction: column;
    gap: 0.35em;
  }
  .dl-arc-buttons {
    display: flex;
    gap: 0.4rem;
    flex-wrap: wrap;
  }
  .dl-arc-desc {
    font-style: italic;
  }
  .dl-arc-hint,
  .dl-arc-muted {
    color: var(--archive-muted, #666666);
  }
  .dl-arc-hint { font-size: 0.88em; }
  .dl-arc-convert-error {
    color: var(--color-error, #cc0000);
    font-size: 0.85em;
  }
  .dl-arc-actions {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    flex-wrap: wrap;
    margin-top: 1em;
    padding-top: 0.8em;
    border-top: 1px solid var(--archive-border, #dddddd);
  }
  .dl-arc-rating {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }
  .dl-arc-rating-count {
    font-size: 0.85em;
    color: var(--archive-muted, #666666);
  }
  .dl-arc-comments {
    margin-top: 1em;
  }
  .dl-arc-comments h3 {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.05em;
    margin: 0 0 0.4em;
  }
  .dl-arc-comment-date {
    display: block;
    font-size: 0.82em;
    color: var(--archive-muted, #666666);
    font-style: italic;
  }
  .dl-arc-comment-form {
    margin-top: 0.8em;
  }
  .dl-arc-tip {
    margin-top: 1.2em;
    font-size: 0.88em;
  }
  .dl-arc-bookmarklet {
    color: var(--archive-link, #990000);
    text-decoration: none;
    font-weight: 600;
  }
  .dl-arc-bookmarklet:hover { text-decoration: underline; }

  @media (max-width: 600px) {
    .dl-arc-dl .dl-row,
    .dl-arc-comment-dl .dl-row { grid-template-columns: 1fr; }
    .dl-arc-search { flex-direction: column; }
  }

  /* ── Modern mode (unchanged) ── */
  .search-row {
    display: flex;
    gap: 0.6rem;
    margin-bottom: 1rem;
  }
  .search-row input {
    flex: 1;
  }
  .error-card {
    border-color: var(--color-error);
  }
  .result-card h2 {
    margin: 0 0 0.3rem;
    font-size: 1.4rem;
  }
  .meta-line {
    color: var(--color-text);
    margin: 0.4rem 0;
  }
  .desc {
    color: var(--color-muted);
    font-size: 0.92rem;
  }
  .downloads {
    display: flex;
    flex-wrap: wrap;
    gap: 0.6rem;
    margin-top: 1rem;
  }
  .pref-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 1rem;
  }
  .pref-label {
    font-size: 0.85rem;
    color: var(--color-muted);
  }
  .pref-select {
    padding: 0.3rem 0.5rem;
    font-size: 0.88rem;
    background: var(--color-surface-2);
    color: var(--color-text);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
  }
  .dl-btn {
    text-decoration: none;
  }
  .convert-error {
    display: block;
    width: 100%;
    font-size: 0.8rem;
  }
  .dl-btn.preferred {
    border-color: var(--color-primary);
    color: var(--color-primary);
    font-weight: 700;
  }
  .format-hint {
    margin-top: 0.5rem;
    font-size: 0.8rem;
  }
  .note {
    margin-top: 0.8rem;
    font-size: 0.85rem;
  }
  .hint {
    margin-top: 2rem;
    font-size: 0.85rem;
  }
  .bookmarklet {
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 0.2rem 0.5rem;
    cursor: grab;
    font-family: var(--mono);
  }

  /* ── Fic actions: bookmark + rating ── */
  .fic-actions {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-top: 1rem;
    padding-top: 0.8rem;
    border-top: 1px solid var(--color-border);
  }
  .action-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 0.35rem 0.7rem;
    color: var(--color-text);
    cursor: pointer;
    font-size: 0.88rem;
    transition: border-color 0.15s, background 0.15s;
  }
  .action-btn:hover {
    border-color: var(--color-primary);
  }
  .action-btn.active {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: #fff;
  }
  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .action-btn .icon {
    font-size: 1rem;
    line-height: 1;
  }
  .rating-group {
    display: flex;
    gap: 0.4rem;
  }

  /* ── Comments section ── */
  .comments-section {
    margin-top: 1rem;
    padding-top: 0.8rem;
    border-top: 1px solid var(--color-border);
  }
  .comments-section h3 {
    margin: 0 0 0.6rem;
    font-size: 1.05rem;
  }
  .comment-list {
    list-style: none;
    padding: 0;
    margin: 0 0 1rem;
  }
  .comment-item {
    padding: 0.6rem 0;
    border-bottom: 1px solid var(--color-border);
  }
  .comment-item:last-child {
    border-bottom: none;
  }
  .comment-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.25rem;
  }
  .comment-date {
    font-size: 0.78rem;
  }
  .comment-body {
    margin: 0;
    font-size: 0.9rem;
    color: var(--color-text);
    white-space: pre-wrap;
  }
  .comment-form {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .comment-form textarea {
    width: 100%;
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 0.5rem;
    color: var(--color-text);
    font-family: inherit;
    font-size: 0.9rem;
    resize: vertical;
  }
  .comment-form textarea:focus {
    outline: none;
    border-color: var(--color-primary);
  }
  .comment-form .btn {
    align-self: flex-end;
  }

  @media (max-width: 600px) {
    .search-row {
      flex-direction: column;
    }
  }
</style>
