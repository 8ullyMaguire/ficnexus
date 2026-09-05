<script lang="ts">
 import { onMount } from 'svelte';
 import { fetchExport, convertFormat } from '$lib/api/client';
 import { fetchAlsoBookmarked } from '$lib/api/recommendations';
 import type { ExportResponse } from '$lib/api/types';
 import { addBookmark, listBookmarks, removeBookmark, rateWork, getRatings, giveKudos, removeKudos, getKudos, downloadAuthorWorks, downloadSeries, getChapterTranslations, saveChapterTranslations, follow, unfollow, checkWorkFollow, markFollowSeen, refreshFic } from '$lib/api/social';
 import { sendToKindle } from '$lib/api/kindle';
 import { auth } from '$lib/stores/auth.svelte';
import { isDeviceBookmarked, toggleDeviceBookmark } from '$lib/stores/deviceLibrary.svelte';
 import { t } from '$lib/i18n/index.svelte';
 import { getPref, setPref } from '$lib/prefs';
 import CommentSection from '$lib/components/CommentSection.svelte';
import FicSuggestionsPanel from '$lib/components/FicSuggestionsPanel.svelte';
 import StarRating from '$lib/components/StarRating.svelte';
 import ReviewsSection from '$lib/components/ReviewsSection.svelte';
 import ReactionBar from '$lib/components/ReactionBar.svelte';
 import MetadataEditModal from '$lib/components/MetadataEditModal.svelte';
 import ArchiveWork from '$lib/ui/archive/ArchiveWork.svelte';
 import ArchiveBreadcrumbs from '$lib/ui/archive/ArchiveBreadcrumbs.svelte';
import TranslatePageButton from '$lib/components/TranslatePageButton.svelte';
 import { formatWords, detectSite, stripHtml, relativeTime } from '$lib/util';

 let { data } = $props();
 const urlId = $derived(decodeURIComponent(data.urlId));

 // ── SEO: per-fic <head> metadata ────────────────────────────────────────
 // Crawler-visible title/description/OG tags. Filled once `fic` loads.
 const metaDescription = $derived.by(() => {
  if (!fic?.meta) return 'Download and read fanfiction from 100+ sites.';
  const raw = stripHtml(fic.meta.description || '');
  const trimmed = raw.replace(/\\s+/g, ' ').trim();
  if (!trimmed) return `Read ${fic.meta.title} by ${fic.meta.author} on FicHub.`;
  return trimmed.length > 155 ? trimmed.slice(0, 152).trimEnd() + '…' : trimmed;
 });
 // ── Slug URL support (Task 13) ────────────────────────────────
 // The [urlId] param absorbs both the legacy bare url_id and the new
 // canonical `/works/{slug}.{work_id}` form. Parse the trailing `.id`
 // here so the rest of the page keeps using the url_id-based API.
 const parsedTail = $derived.by(() => {
  const raw = urlId ?? '';
  const dot = raw.lastIndexOf('.');
  const numericId =
    dot > 0 && /^\d+$/.test(raw.slice(dot + 1)) ? Number(raw.slice(dot + 1)) : null;
  return { raw, numericId };
 });

 /** Tiny client-side slug helper (mirrors server work_url_slug). */
 function deriveSlug(title: string | null | undefined): string {
  if (!title) return 'work';
  const cleaned = title
   .split('')
   .filter((c: string) => /[A-Za-z0-9 \-']/.test(c))
   .join('')
   .replace(/'/g, '')
   .split(/\s+/)
   .filter(Boolean)
   .join('-')
   .toLowerCase()
   .replace(/^-+|-+$/g, '')
   .slice(0, 60);
  return cleaned || 'work';
 }

 const slugFromId = $derived.by(() => deriveSlug(fic?.meta?.title));

 const canonicalUrl = $derived.by(() => {
  if (typeof window === 'undefined') return '';
  if (workId && slugFromId) {
    return `${window.location.origin}/works/${slugFromId}.${workId}`;
  }
  // Fallback to the legacy form when we don't have workId yet.
  return `${window.location.origin}/works/${encodeURIComponent(urlId)}`;
 });

 let fic = $state<ExportResponse | null>(null);
 let loading = $state(true);
 let showMetaEdit = $state(false);
 let error = $state('');

 // Derived work_id from export response
 const workId = $derived(fic?.meta?.work_id ?? null);

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

 // Comment state

 // Send-to-Kindle state
 let kindleSending = $state(false);
 let kindleSent = $state(false);
 let kindleError = $state('');

 // Translation state
 let showTranslationModal = $state(false);
 let translationLoading = $state(false);
 let translationSaving = $state(false);
 let translationLocale = $state('en');
 let translatedChapters = $state<{ chapter_num: number; title: string; content_html: string }[]>([]);
 let activeTranslationChapter = $state(0);

 // Follow state (fic + author)
 let isFollowingWork = $state(false);
 let workFollowId = $state<number | null>(null);
 let isFollowingAuthor = $state(false);
 let authorFollowId = $state<number | null>(null);
 let followSaving = $state(false);

 // Refresh state
 let refreshing = $state(false);
 let refreshMessage = $state('');
 let refreshError = $state('');

 // Similar fics state
 let similarItems = $state<{ work_id: number; title: string; similarity?: string; co_bookmarkers?: number }[]>([]);
 let similarLoading = $state(false);
 let similarLoaded = $state(false);

 // Readers-also-bookmarked state (co-occurrence rec anchors)
 let alsoBookmarkedItems = $state<import('$lib/api/recommendations').AlsoBookmarkedItem[]>([]);
 let alsoBookmarkedLoading = $state(true);
 let alsoBookmarkedLoaded = $state(false);

 // ── Bulk download state (author ZIP + series ZIP) ─────────────────────
 // The AO3 scraper records the author's AO3 profile URL in `author_url`
 // (e.g. https://archiveofourown.org/users/{name}) and a series page URL
 // (when the work is part of an AO3 series) in `extra_meta`.
 let batchDownloading = $state(false);
 let batchDownloadError = $state('');
 let seriesDownloading = $state(false);
 let seriesDownloadError = $state('');

 // Is this an AO3 fic? (show batch download button)
 const isAo3 = $derived(fic?.meta?.source ? detectSite(fic.meta.source) === 'AO3' : false);
 const authorPageUrl = $derived(fic?.meta?.author_url ?? '');

 // ── Series ZIP download ───────────────────────────────────────────────
 const seriesAo3Url = $derived((() => {
 const raw = fic?.meta?.extra_meta;
 if (typeof raw === 'string' && raw.includes('archiveofourown.org/series/')) return raw;
 if (raw && typeof raw === 'object') {
 const o = raw as Record<string, unknown>;
 for (const v of Object.values(o)) {
 if (typeof v === 'string' && v.includes('archiveofourown.org/series/')) return v;
 }
 }
 return '';
 })());

 // When author_url is only a bare pseud (no host), build the full AO3
 // author page so the download endpoint's URL validation passes. The AO3
 // scraper's author_url is typically https://archiveofourown.org/users/{name}
 // already — this handles edge cases where only the pseud is stored.
 function ao3AuthorUrl(): string {
 if (!authorPageUrl) return '';
 if (authorPageUrl.includes('archiveofourown.org')) return authorPageUrl;
 return `https://archiveofourown.org/users/${authorPageUrl.replace(/^\/+/, '')}`;
 }

 async function handleSeriesDownload() {
 if (!seriesAo3Url || seriesDownloading) return;
 seriesDownloading = true;
 seriesDownloadError = '';
 try {
 await downloadSeries(seriesAo3Url);
 } catch (e) {
 seriesDownloadError = e instanceof Error ? e.message : 'Download failed';
 } finally {
 seriesDownloading = false;
 }
 }

 async function handleBatchDownload() {
 const url = ao3AuthorUrl();
 if (!url || batchDownloading) return;
 batchDownloading = true;
 batchDownloadError = '';
 try {
 await downloadAuthorWorks(url);
 } catch (e) {
 batchDownloadError = e instanceof Error ? e.message : 'Download failed';
 } finally {
 batchDownloading = false;
 }
 }

 // Download formats
 // Calibre-backed formats (mobi/pdf/azw3) may not have a URL yet — they
 // convert on demand when clicked (lazy). Everything else is a direct link.
 let downloads = $derived.by(() => {
 if (!fic) return [];
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
 add('EPUB', 'epub', fic.epub_url);
 add('MOBI', 'mobi', fic.mobi_url, true);
 add('AZW3', 'azw3', fic.azw3_url, true);
 add('PDF', 'pdf', fic.pdf_url, true);
 add('DOCX', 'docx', fic.docx_url, true);
 add('KEPUB', 'kepub', fic.kepub_url, true);
 add('FB2', 'fb2', fic.fb2_url, true);
 add('HTML', 'html', fic.html_url, true);
 add('TXT', 'txt', fic.txt_url, true);
 add('MD', 'md', fic.md_url, true);
 return out;
 });

 // ── Preferred download format ──────────────────────────────────────────
 // Persisted in localStorage by lib/prefs.ts. The "Preferred format"
 // dropdown lets the user pick it explicitly; clicking any download button
 // also sets it. The preferred button is highlighted and sorted first so a
 // one-click download is always available.
 let defaultFormat = $state(getPref('defaultFormat'));

 // Formats present in the current result (dropdown options).
 let availableFormats = $derived.by(() =>
 downloads.map((d) => ({ label: d.label, type: d.type })),
 );

 // The stored pref may name a format this fic doesn't have (e.g. MOBI on a
 // source site that only renders EPUB/HTML). Fall back to the first
 // available format for highlighting/sorting without clobbering the stored
 // preference.
 let effectiveFormat = $derived.by(() => {
 if (availableFormats.some((f) => f.type === defaultFormat)) return defaultFormat;
 return availableFormats[0]?.type ?? 'epub';
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
 const res = await convertFormat(urlId, d.type);
 if (res.err !== 0) {
 convertError[d.type] = res.msg || 'Conversion failed.';
 return;
 }
 if (res.url) {
 // Trigger the download, then update the fic so the button becomes a
 // normal link.
 const a = document.createElement('a');
 a.href = res.url;
 a.download = '';
 a.rel = 'noopener';
 document.body.appendChild(a);
 a.click();
 a.remove();
 if (fic) {
 fic = { ...fic, [`${d.type}_url`]: res.url };
 }
 }
 } catch {
 convertError[d.type] = 'Conversion failed.';
 } finally {
 converting[d.type] = false;
 }
 }

 onMount(async () => {
 await auth.init();
 await loadFic();
 await Promise.allSettled([loadSimilar(), loadAlsoBookmarked()]);
 });

 async function loadFic() {
 loading = true;
 error = '';
 try {
 const res = await fetchExport(urlId);
 if (res.err !== 0) {
 error = res.msg || 'Could not load fic metadata.';
 return;
 }
 fic = res;
 // Load secondary data in parallel.
 await Promise.allSettled([loadBookmarkStatus(), loadRatings(), loadKudos(), loadFollowStatus()]);
 } catch {
 error = 'Network error loading fic.';
 } finally {
 loading = false;
 }
 }

 async function loadFollowStatus() {
 if (!auth.isLoggedIn || !workId) return;
 try {
 const res = await checkWorkFollow(workId);
 if (res.err === 0) {
 isFollowingWork = res.is_following;
 workFollowId = (res as { follow_id?: number | null }).follow_id ?? null;
 }
 } catch { /* ignore */ }
 }

 async function toggleFollowWork() {
 if (!auth.isLoggedIn || followSaving || !workId) return;
 followSaving = true;
 try {
 if (isFollowingWork) {
 if (workFollowId) await unfollow(workFollowId);
 isFollowingWork = false;
 workFollowId = null;
 } else {
 const res = await follow('work', workId);
 if (res.err === 0) {
 isFollowingWork = true;
 workFollowId = (res as { follow_id?: number | null }).follow_id ?? null;
 if (workFollowId) { markFollowSeen(workFollowId).catch(() => {}); }
 }
 }
 } catch { /* ignore */ }
 followSaving = false;
 }

 async function toggleFollowAuthor() {
 if (!auth.isLoggedIn || followSaving) return;
 const authorName = fic?.meta?.author;
 if (!authorName) return;
 followSaving = true;
 try {
 if (isFollowingAuthor) {
 if (authorFollowId) await unfollow(authorFollowId);
 isFollowingAuthor = false;
 authorFollowId = null;
 } else {
 const res = await follow('author', undefined, authorName);
 if (res.err === 0) {
 isFollowingAuthor = true;
 authorFollowId = (res as { follow_id?: number | null }).follow_id ?? null;
 }
 }
 } catch { /* ignore */ }
 followSaving = false;
 }

 async function handleRefresh() {
 if (!auth.isLoggedIn || refreshing) return;
 refreshing = true;
 refreshMessage = '';
 refreshError = '';
 try {
 const res = await refreshFic(urlId);
 if (res.err === 0) {
 if (res.status === 'no_change') {
 refreshMessage = 'Already up to date — no new chapters found.';
 } else {
 refreshMessage = 'Fic updated — new version cached' + (res.notified ? `, ${res.notified} follower(s) notified` : '');
 }
 } else {
 refreshError = (res as { msg?: string }).msg || 'Refresh failed.';
 }
 } catch (e) {
 refreshError = e instanceof Error ? e.message : 'Refresh failed.';
 } finally {
 refreshing = false;
 }
 }

   async function loadBookmarkStatus() {
  if (!workId) return;
  if (auth.isLoggedIn) {
    try {
      const res = await listBookmarks();
      if (res.err === 0) {
        isBookmarked = res.bookmarks.some((b) => b.work_id === workId);
      }
    } catch { /* ignore */ }
  } else {
    isBookmarked = isDeviceBookmarked(workId);
  }
  }

  async function toggleBookmark() {
  if (savingBookmark || !workId) return;
  savingBookmark = true;
  try {
    if (auth.isLoggedIn) {
      if (isBookmarked) {
        await removeBookmark(workId);
        isBookmarked = false;
      } else {
        await addBookmark(workId, bookmarkNotes || undefined, bookmarkPrivate);
        isBookmarked = true;
        bookmarkNotes = '';
      }
    } else {
      await toggleDeviceBookmark(workId);
      isBookmarked = isDeviceBookmarked(workId);
    }
  } catch { /* ignore */ }
  savingBookmark = false;
  }

 async function loadRatings() {
 if (!workId) return;
 try {
 const res = await getRatings(workId);
 if (res.err === 0) {
 avgRating = res.avg_rating ?? 0;
 ratingCount = res.rating_count ?? 0;
 reviewCount = res.review_count ?? 0;
 }
 } catch { /* ignore */ }
 // The public aggregate has no per-user field; the widget reflects the
 // caller's own stars from the client-side record (set on rate).
 const saved = localStorage.getItem(`fichub_my_rating_${workId}`);
 if (saved) myRating = parseInt(saved, 10) || 0;
 }

 async function handleRate(rating: number) {
 if (!auth.isLoggedIn || ratingLoading || !workId) return;
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

 async function loadKudos() {
 if (!workId) return;
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
 if (!auth.isLoggedIn || kudosLoading || !workId) return;
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

 async function handleSendToKindle() {
 if (!auth.isLoggedIn || kindleSending) return;
 kindleSending = true;
 kindleError = '';
 kindleSent = false;
 try {
 const res = await sendToKindle({ url_id: urlId });
 if (res.err !== 0) {
 kindleError = res.msg || 'Could not send to Kindle.';
 } else {
 kindleSent = true;
 }
 } catch (e) {
 kindleError = e instanceof Error ? e.message : 'Send to Kindle failed';
 } finally {
 kindleSending = false;
 }
 }

 async function openTranslationModal() {
 if (!workId) return;
 translationLocale = localStorage.getItem('fichub_locale') || 'en';
 translationLoading = true;
 showTranslationModal = true;
 activeTranslationChapter = 0;
 // Pre-fill chapter list from the work's chapter count
 const chapterCount = fic?.meta?.chapters || 1;
 const chapters = [];
 for (let i = 1; i<= chapterCount; i++) {
 chapters.push({ chapter_num: i, title: `Chapter ${i}`, content_html: '' });
 }
 translatedChapters = chapters;
 // Try to load existing translations
 try {
 const existing = await getChapterTranslations(workId, translationLocale);
 if (existing.err === 0 && existing.chapters?.length > 0) {
 translatedChapters = existing.chapters;
 }
 } catch { /* no existing translations */ }
 finally { translationLoading = false; }
 }

 function closeTranslationModal() {
 showTranslationModal = false;
 }

 async function handleSaveTranslations() {
 if (!workId || translationSaving) return;
 translationSaving = true;
 try {
 const res = await saveChapterTranslations(workId, translationLocale, translatedChapters);
 if (res.err === 0) {
 closeTranslationModal();
 }
 } catch { /* silent */ }
 finally { translationSaving = false; }
 }

 function updateTranslationContent(chapterNum: number, content: string) {
 const idx = translatedChapters.findIndex(c => c.chapter_num === chapterNum);
 if (idx >= 0) {
 translatedChapters[idx] = { ...translatedChapters[idx], content_html: content };
 }
 }

 async function loadSimilar() {
 if (!workId) return;
 similarLoading = true;
 try {
 const res = await fetch(`/api/search/similar/${workId}`, { credentials: 'include' });
 if (res.ok) {
 const data = await res.json();
 similarItems = data.items || [];
 }
 } catch { /* ignore */ }
 finally { similarLoading = false; similarLoaded = true; }
 }

 async function loadAlsoBookmarked() {
 alsoBookmarkedLoading = true;
 try {
 const res = await fetchAlsoBookmarked(urlId);
 if (res.err === 0) {
 alsoBookmarkedItems = res.items ?? [];
 } else {
 alsoBookmarkedItems = [];
 }
 } catch { /* ignore — leave empty */ }
 finally { alsoBookmarkedLoading = false; alsoBookmarkedLoaded = true; }
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

<svelte:head>
  <title>{fic?.meta ? `${fic.meta.title} — ${fic.meta.author} | FicHub` : 'FicHub'}</title>
  <meta name="description" content={metaDescription} />
  {#if canonicalUrl}
  <link rel="canonical" href={canonicalUrl} />
  <meta property="og:title" content={fic?.meta ? `${fic.meta.title} — ${fic.meta.author}` : 'FicHub'} />
  <meta property="og:description" content={metaDescription} />
  <meta property="og:type" content="article" />
  <meta property="og:url" content={canonicalUrl} />
  <meta name="twitter:card" content="summary" />
  <meta name="twitter:title" content={fic?.meta ? `${fic.meta.title} — ${fic.meta.author}` : 'FicHub'} />
  <meta name="twitter:description" content={metaDescription} />
  {/if}
</svelte:head>

{#if loading}
<div class="card">
<p class="muted"><span class="spinner"></span> {t('fic.loading')}</p>
</div>
 {:else if error}
<div class="card error-card">
<strong class="error-text"> {error}</strong>
<p class="muted">
 {t('fic.errorHint')}
</p>
</div>
 {:else if fic?.meta}
 {@const m = fic.meta}

 {#if fic}
<ArchiveBreadcrumbs {fic} />
<ArchiveWork
 fic={fic}
 isBookmarked={isBookmarked}
 onToggleBookmark={toggleBookmark}
 savingBookmark={savingBookmark}
 />

<!-- Reader actions (AO3 work-actions parity) -->
<h3 class="landmark heading">Actions</h3>
<div class="work-actions">
 {#if auth.isLoggedIn}
<button
 class="action-btn action-bookmark"
 class:bookmarked={isBookmarked}
 onclick={toggleBookmark}
 disabled={savingBookmark}
 >
 {#if savingBookmark}
<span class="spinner"></span>
 {:else if isBookmarked}
 {t('fic.saved')}
 {:else}
 {t('fic.save')}
 {/if}
</button>

<div class="rating-inline">
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

<button
 class="action-btn action-kudos"
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
<span class="rating-count">{t('work.kudosCount', { count: kudosCount })}</span>
 {#if guestCount > 0}
<span class="archive-muted-inline">{t('work.kudosGuests', { count: guestCount })}</span>
 {/if}
 {/if}

<div class="reactions-inline">
<ReactionBar targetType="work" targetId={workId ?? 0} />
</div>

<button
 class="action-btn action-translate"
 onclick={openTranslationModal}
 aria-label={t('fic.translateTitle', { title: m.title })}
 >
 {t('fic.translate')}
</button>
 {:else}
<a class="action-btn" href="/">{t('fic.loginToRate')}</a>
 {/if}
</div>

<!-- Machine-translation queue for unread visible text (M7) -->
<TranslatePageButton />

<!-- Comments (threaded + reactions) -->
<div id="comments" class="work-comments-section">
<CommentSection workId={fic.meta?.work_id} urlId={urlId} />
</div>

<!-- Reviews (in-depth written reviews + 5-star) -->
{#if fic.meta?.work_id}
<div class="work-reviews-section">
<ReviewsSection workId={fic.meta.work_id} />
</div>
{/if}

<!-- Per-fic similar fic suggestions (community-driven) -->
{#if urlId}
  <FicSuggestionsPanel urlId={urlId} />
{/if}

<!-- Readers also bookmarked (co-occurrence rec anchors) -->
{#if urlId}
  <div class="also-bookmarked-section">
    <div class="section-head">
      <h3>{t('fic.readersAlsoBookmarked')}</h3>
      <a class="btn btn-small" href={fic?.meta?.work_id ? `/requests/new?seed=${fic.meta.work_id}` : '/requests/new'}>
        {t('fic.requestSimilar')}
      </a>
    </div>
    {#if alsoBookmarkedLoading}
      <p class="muted"><span class="spinner"></span> Loading…</p>
    {:else if alsoBookmarkedItems.length > 0}
      <div class="similar-grid">
        {#each alsoBookmarkedItems as item}
          <a href={item.work_id ? `/work/${item.work_id}` : `/works/${encodeURIComponent(item.url_id)}`} class="similar-card">
            <strong>{item.title}</strong>
            <span class="similar-score">{t('fic.readerSaved', { count: item.cooccur_count })}</span>
          </a>
        {/each}
      </div>
    {:else if alsoBookmarkedLoaded}
      <p class="no-similar">{t('fic.notEnoughData')}</p>
    {/if}
  </div>
{/if}

<!-- Similar fics section -->
{#if workId}
  <div class="similar-section">
    <h3>{t('fic.moreLikeThis')}</h3>
    {#if similarLoading}
      <p class="muted"><span class="spinner"></span> {t('fic.loadingSimilar')}</p>
    {:else if similarItems.length > 0}
      <div class="similar-grid">
        {#each similarItems as item}
          <a href={`/work/${item.work_id}`} class="similar-card">
            <strong>{item.title}</strong>
            <span class="similar-score">
              {#if item.similarity}
                {t('fic.match', { percent: (parseFloat(item.similarity) * 100).toFixed(0) })}
              {:else if item.co_bookmarkers}
                {t('fic.coBookmarkers', { count: item.co_bookmarkers })}
              {/if}
            </span>
          </a>
        {/each}
      </div>
    {:else if similarLoaded}
      <p class="no-similar">{t('fic.noSimilar')}</p>
    {/if}
  </div>
{/if}



<!-- Chapter translations editor -->
{#if showTranslationModal && workId}
<div class="modal-backdrop" onclick={closeTranslationModal} role="presentation">
<div class="modal translation-modal" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()} role="dialog" aria-modal="true" tabindex="-1">
<div class="modal-header">
<h3>{t('fic.translateTitle', { title: fic?.meta?.title || t('fic.work') })}</h3>
<button class="modal-close" onclick={closeTranslationModal} aria-label="Close">{t('fic.cancel')}</button>
</div>

 {#if translationLoading}
<div class="modal-loading"><span class="spinner"></span> {t('fic.loadingTranslations')}</div>
 {:else}
<!-- Chapter tabs -->
<div class="chapter-tabs">
 {#each translatedChapters as ch}
<button
 class="chapter-tab"
 class:active={ch.chapter_num === activeTranslationChapter}
 onclick={() => activeTranslationChapter = ch.chapter_num}
 >
 {t('fic.chapterShort', { num: ch.chapter_num })}
</button>
 {/each}
</div>

<!-- Translation editor -->
<div class="translation-editor">
 {#each translatedChapters as ch}
 {#if ch.chapter_num === activeTranslationChapter}
<label class="chapter-label" for="translation-textarea">{ch.title}</label>
<textarea
 id="translation-textarea"
 class="translation-textarea"
 bind:value={ch.content_html}
 oninput={(e) => updateTranslationContent(ch.chapter_num, (e.target as HTMLTextAreaElement).value)}
 placeholder={t('fic.typeTranslation')}
 ></textarea>
 {/if}
 {/each}
</div>

<div class="modal-actions">
<button class="btn btn-secondary" onclick={closeTranslationModal}>{t('fic.cancel')}</button>
<button class="btn" onclick={handleSaveTranslations} disabled={translationSaving}>
 {#if translationSaving}
<span class="spinner"></span> {t('fic.saving')}
 {:else}
 {t('fic.saveTranslation')}
 {/if}
</button>
</div>
 {/if}
</div>
</div>
{/if}
 {/if}
  {/if}  <!-- close loading/error/meta chain -->

<style>
 /* ── Archive-mode reader actions (AO3 work-actions parity) ── */
 .landmark.heading {
 font-size: 0.95em;
 font-weight: 700;
 color: var(--color-muted, #666);
 margin: 1em 0 0.3em;
 }
 .work-actions {
 display: flex;
 flex-wrap: wrap;
 align-items: center;
 gap: 0.5rem;
 margin: 0.4em 0 1em;
 }
 .action-btn {
 display: inline-block;
 padding: 0.3em 0.9em;
 font-size: 0.9rem;
 font-weight: 600;
 line-height: 1.5;
 color: var(--color-text);
 background: var(--color-surface-2);
 border: 1px solid var(--color-border);
 border-radius: var(--radius-sm);
 cursor: pointer;
 text-decoration: none;
 }
 .action-btn:hover:not(:disabled) {
 background: var(--color-border);
 }
 .action-btn:disabled {
 opacity: 0.5;
 cursor: not-allowed;
 }
 .action-bookmark.bookmarked,
 .action-kudos.kudoed {
 color: white;
 background: var(--color-primary);
 border-color: var(--color-primary);
 }
 .rating-inline {
 display: inline-flex;
 align-items: center;
 gap: 0.4rem;
 }
 .rating-count {
 font-weight: 700;
 font-size: 0.88rem;
 color: var(--color-muted, #666);
 }
 .archive-muted-inline {
 font-size: 0.82rem;
 color: var(--color-muted, #666);
 }
 .reactions-inline {
 display: inline-flex;
 align-items: center;
 }
 .work-comments-section,
 .work-reviews-section {
 margin-top: 1.2rem;
 }

 /* Translation modal shared styles (modern + archive branches) */
 .modal-header {
 display: flex;
 justify-content: space-between;
 align-items: center;
 margin-bottom: 0.8rem;
 }
 .fic-page {
 max-width: var(--max-width);
 margin: 0 auto;
 padding: 1rem;
 }
 .error-card {
 border-color: var(--color-error);
 }
 .fic-header h1 {
 margin: 0 0 0.3rem;
 font-size: 1.6rem;
 }
 .author-link {
 color: var(--color-primary);
 }
 .meta-line {
 margin: 0.4rem 0;
 font-size: 0.95rem;
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
 align-items: center;
 gap: 0.5rem;
 }
 .pref-row {
 display: flex;
 align-items: center;
 gap: 0.4rem;
 width: 100%;
 }
 .pref-label {
 font-size: 0.82rem;
 color: var(--color-muted);
 }
 .pref-select {
 padding: 0.25rem 0.45rem;
 font-size: 0.85rem;
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
 .kindle-btn {
 text-decoration: none;
 }
 .kindle-error {
 font-size: 0.82rem;
 max-width: 260px;
 }
 .read-online-btn {
 text-decoration: none;
 font-size: 0.9rem;
 }
 .batch-download-group {
 display: flex;
 flex-direction: column;
 gap: 0.3rem;
 }
 .batch-error {
 font-size: 0.82rem;
 max-width: 260px;
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
 .rating-count {
 font-family: var(--mono);
 font-weight: 600;
 font-size: 0.9rem;
 min-width: 1.5rem;
 text-align: center;
 color: var(--color-muted);
 }
 .follow-group {
 display: flex;
 align-items: center;
 gap: 0.5rem;
 flex-wrap: wrap;
 }
 .follow-btn.following {
 background: var(--color-primary);
 color: white;
 border-color: var(--color-primary);
 }
 .refresh-msg {
 font-size: 0.82rem;
 width: 100%;
 margin: 0.2rem 0 0;
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
 .desc-empty {
 color: var(--color-text-muted, #999);
 font-style: italic;
 font-family: Georgia, 'Times New Roman', serif;
 }
 .notes-card {
 margin-top: 0.8rem;
 }
 .note-text {
 margin: 0.2rem 0;
 font-size: 0.85rem;
 }
 .comments-section {
 margin-top: 1.5rem;
 }

 /* Similar fics section */
 .similar-section {
 margin-top: 1.5rem;
 }
 .also-bookmarked-section {
 margin-top: 1.5rem;
 }
 .also-bookmarked-section h3 {
 font-size: 1.05rem;
 margin-bottom: 0.6rem;
 }
 .similar-section h3 {
 margin-bottom: 0.8rem;
 }
 .similar-grid {
 display: grid;
 grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
 gap: 0.8rem;
 }
 .similar-card {
 display: flex;
 flex-direction: column;
 padding: 0.8rem;
 background: var(--color-surface);
 border: 1px solid var(--color-border);
 border-radius: var(--radius-md);
 text-decoration: none;
 color: var(--color-text);
 transition: border-color 0.15s;
 }
 .similar-card:hover {
 border-color: var(--color-primary);
 }
 .similar-score {
 font-size: 0.82rem;
 color: var(--color-muted);
 margin-top: 0.3rem;
 }
 .no-similar {
 color: var(--color-muted);
 font-size: 0.88rem;
 }

 /* Translation Modal */
 :global(.translation-modal) {
 max-width: 720px;
 width: 90vw;
 max-height: 80vh;
 display: flex;
 flex-direction: column;
 }
 .modal-close {
 background: none;
 border: none;
 color: var(--color-muted);
 font-size: 1.2rem;
 cursor: pointer;
 padding: 0.2rem 0.4rem;
 border-radius: var(--radius-sm);
 }
 .modal-close:hover {
 background: var(--color-surface-2);
 color: var(--color-text);
 }
 .modal-loading {
 text-align: center;
 padding: 2rem;
 color: var(--color-muted);
 }
 .chapter-tabs {
 display: flex;
 flex-wrap: wrap;
 gap: 0.3rem;
 margin-bottom: 0.8rem;
 border-bottom: 1px solid var(--color-border);
 padding-bottom: 0.5rem;
 }
 .chapter-tab {
 padding: 0.3rem 0.6rem;
 border-radius: var(--radius-sm);
 border: 1px solid var(--color-border);
 background: var(--color-surface-2);
 color: var(--color-muted);
 cursor: pointer;
 font-size: 0.82rem;
 transition: all 0.15s;
 }
 .chapter-tab.active {
 background: var(--color-primary);
 color: white;
 border-color: var(--color-primary);
 }
 .chapter-tab:hover:not(.active) {
 color: var(--color-text);
 }
 .translation-editor {
 flex: 1;
 min-height: 0;
 overflow-y: auto;
 }
 .chapter-label {
 display: block;
 font-weight: 600;
 font-size: 0.88rem;
 margin-bottom: 0.4rem;
 color: var(--color-text);
 }
 .translation-textarea {
 width: 100%;
 min-height: 300px;
 padding: 0.8rem;
 font-size: 0.9rem;
 line-height: 1.6;
 border-radius: var(--radius-sm);
 border: 1px solid var(--color-border);
 background: var(--color-surface-2);
 color: var(--color-text);
 resize: vertical;
 font-family: sans-serif;
 }
 .modal-actions {
 display: flex;
 justify-content: flex-end;
 gap: 0.6rem;
 margin-top: 0.8rem;
 padding-top: 0.8rem;
 border-top: 1px solid var(--color-border);
 }

 @media (max-width: 600px) {
 .actions-row {
 flex-direction: column;
 align-items: flex-start;
 }
 }
</style>
