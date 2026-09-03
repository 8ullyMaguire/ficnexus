<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { goto } from '$app/navigation';
  import { updateReadingStatus, getChapterTranslations, listLocales, recordReadHistory, giveKudos, removeKudos, getKudos, addBookmark, removeBookmark, listBookmarks, addComment, fetchThreadedComments } from '$lib/api/social';
  import OfflineIndicator from '$lib/components/OfflineIndicator.svelte';
  import MarginaliaComposerModal from '$lib/components/MarginaliaComposerModal.svelte';
  import ReactionBar from '$lib/components/ReactionBar.svelte';
  import { getMarginalia, getShowMarginalia, type MarginaliaEntry, type MarginaliaPayload } from '$lib/api/marginalia';
  import { getPref } from '$lib/prefs';
  import { loadReaderData, saveReaderState, restoreReaderState, type ReaderWork } from './reader-lib';

  // Interface style: 'archive' (AO3-style) or 'modern'.
  const uiMode = $derived(getPref('uiMode'));

  let { data } = $props();
  // `$page.params.urlId` is undefined in jsdom unit tests (no router), so
  // fall back to the +page.ts load payload.
  const urlId = $derived(
    decodeURIComponent(
      ($page?.params?.urlId ?? (data as { urlId?: string } | undefined)?.urlId ?? '')
    )
  );

  // ── Core load state ──────────────────────────────────────────────────────
  let loading = $state(true);
  let error = $state('');
  let work = $state<ReaderWork | null>(null);
  // Browser connectivity for the offline banner (mirrors OfflineIndicator).
  let navigatorOnLine = $state(typeof navigator !== 'undefined' ? navigator.onLine : true);
  let chapterList = $state<{ title: string; url_id: string }[]>([]);
  let chapterIndex = $state(0);
  let chapterTitle = $state('');
  let htmlContent = $state('');
  let words = $state(0);
  let totalWords = $state(0);
  let isLastChapter = $state(false);

  // ── Reader preferences (persisted in localStorage) ───────────────────────
  let fontSize = $state(18);
  let lineHeight = $state(1.6);
  let maxWidth = $state(720);
  let theme = $state<'light' | 'sepia' | 'dark'>('light');
  let indentParagraphs = $state(true);

  // ── Progress ─────────────────────────────────────────────────────────────
  let progress = $state(0);
  let markSaved = $state(false);

  // ── Next Up panel (end-of-fic) ───────────────────────────────────────────
  let showNextUp = $state(false);
  let nextUpCandidates = $state<{ url_id: string; title: string; author: string; reason: string }[]>([]);
  // Set when the fic rendered from the localStorage HTML cache (offline).
  let fromCache = $state(false);

  // ── Reader action bar (AO3 reader parity) ────────────────────────────────
  let hasKudos = $state(false);
  let kudosCount = $state(0);
  let isBookmarked = $state(false);
  let isMarkedForLater = $state(false);
  let commentCount = $state(0);
  let kudosLoading = $state(false);
  let bookmarkLoading = $state(false);

  // Translation state (kept from v1 reader)
  let translatedContent = $state<string | null>(null);
  let showTranslated = $state(false);
  let translationLocale = $state('en');
  let translationsAvailable = $state(false);
  // Locale picker: list of supported locales for the reader's translation
  // dropdown (fetched from /api/locales via listLocales()).
  let locales = $state<import('$lib/api/social-types').Locale[]>([]);
  let translationPickerOpen = $state(false);
  let translationLoadingLocale = $state(false);

  // Scroll progress bar width
  let scrollProgress = $state(0);

  // ── Marginalia ─────────────────────────────────────────────────────────
  let marginaliaMap = $state<Map<string, MarginaliaEntry>>(new Map());
  let showMarginalia = $state(true);
  let marginaliaLoading = $state(false);
  let tooltipVisible = $state(false);
  let tooltipPos = $state({ top: 0, left: 0 });
  let tooltipData = $state<{ passageHash: string; passageText: string; rect?: DOMRect } | null>(null);
  let composerOpen = $state(false);
  let composerPayload = $state<MarginaliaPayload | null>(null);
  let composerPassageText = $state('');
  let readerContentEl = $state<HTMLDivElement | null>(null);
  let highlightedHash = $state('');

  async function loadMarginalia(chapterIdx: number) {
    if (!showMarginalia) return;
    marginaliaLoading = true;
    try {
      const res = await getMarginalia(urlId, chapterIdx + 1);
      const map = new Map<string, MarginaliaEntry>();
      for (const m of res.marginalia ?? []) map.set(m.passage_hash, m);
      marginaliaMap = map;
    } catch { /* keep empty */ }
    finally { marginaliaLoading = false; }
  }

  function handleSelection() {
    const sel = window.getSelection();
    if (!sel || sel.isCollapsed || sel.rangeCount === 0) { tooltipVisible = false; return; }
    const text = sel.toString().trim();
    const words = text.split(/\s+/).filter(Boolean).length;
    if (!text || words >= 500 || words === 0) { tooltipVisible = false; return; }
    const range = sel.getRangeAt(0);
    const startEl = (range.startContainer instanceof Element ? range.startContainer : range.startContainer.parentElement)?.closest('[data-passage-hash]') as HTMLElement | null;
    const endEl = (range.endContainer instanceof Element ? range.endContainer : range.endContainer.parentElement)?.closest('[data-passage-hash]') as HTMLElement | null;
    if (!startEl || !endEl || startEl !== endEl) { tooltipVisible = false; return; }
    const passageHash = startEl.getAttribute('data-passage-hash') ?? '';
    if (!passageHash) { tooltipVisible = false; return; }
    const rect = range.getBoundingClientRect();
    tooltipPos = { top: rect.top + window.scrollY - 40, left: rect.left + rect.width / 2 };
    tooltipData = { passageHash, passageText: text };
    tooltipVisible = true;
  }

  function openComposerForSelection() {
    if (!tooltipData || !work) return;
    composerPassageText = tooltipData.passageText;
    composerPayload = {
      type: 'marginalia', work_id: work.work_id, url_id: urlId,
      chapter_index: chapterIndex, passage_hash: tooltipData.passageHash, passage_text: tooltipData.passageText,
    };
    tooltipVisible = false;
    composerOpen = true;
    try { window.getSelection()?.removeAllRanges(); } catch { /* ignore */ }
  }

  function handleMarginaliaCreated(topicId: number, topicSlug?: string | null) {
    // Optimistically update map
    if (composerPayload) {
      const existing = marginaliaMap.get(composerPayload.passage_hash);
      marginaliaMap.set(composerPayload.passage_hash, {
        passage_hash: composerPayload.passage_hash, topic_id: topicId, topic_slug: topicSlug ?? null, post_count: (existing?.post_count ?? 0) + 1,
      });
      marginaliaMap = new Map(marginaliaMap);
    }
  }

  function scrollToHash(hash: string) {
    if (!hash) return;
    requestAnimationFrame(() => {
      const el = document.querySelector(`[data-passage-hash="${CSS.escape(hash)}"]`);
      if (el) {
        highlightedHash = hash;
        el.scrollIntoView({ behavior: 'smooth', block: 'center' });
        el.classList.add('marginalia-highlight');
        setTimeout(() => el.classList.remove('marginalia-highlight'), 3000);
      }
    });
  }

  function syncToScroll() {
    const doc = document.documentElement;
    const total = doc.scrollHeight - window.innerHeight;
    const pct = total > 0 ? Math.min(100, Math.round((window.scrollY / total) * 100)) : 0;
    scrollProgress = pct;
    progress = pct;
    // Save reading position (debounced by caller)
    savePosition();
  }

  function savePosition() {
    if (!work) return;
    const state = restoreReaderState(urlId);
    const saved = {
      ...state,
      fontSize,
      lineHeight,
      maxWidth,
      theme,
      indentParagraphs,
      chapterIndex,
      scrollPos: window.scrollY,
      finished: markSaved,
      lastReadAt: Date.now(),
    };
    saveReaderState(urlId, saved);
    // Server sync (best effort, only when signed in)
    void syncServerProgress('reading');
  }

  let lastServerSync = 0;
  async function syncServerProgress(status: 'reading' | 'completed') {
    if (!work?.work_id) return;
    const now = Date.now();
    if (status !== 'completed' && now - lastServerSync < 30_000) return; // throttle
    lastServerSync = now;
    try {
      if (auth.isLoggedIn) {
        await updateReadingStatus(work.work_id, status, chapterIndex + 1);
      }
    } catch { /* offline / not signed in — localStorage is the source of truth for v1 */ }
  }

  // ── Preference toggles ───────────────────────────────────────────────────
  function changeSize(delta: number) {
    fontSize = Math.max(12, Math.min(32, fontSize + delta));
    savePosition();
  }
  function changeLineHeight(delta: number) {
    lineHeight = Math.max(1.2, Math.min(2.4, Math.round((lineHeight + delta) * 10) / 10));
    savePosition();
  }
  function changeWidth(delta: number) {
    maxWidth = Math.max(480, Math.min(1100, maxWidth + delta));
    savePosition();
  }
  function cycleTheme() {
    theme = theme === 'light' ? 'sepia' : theme === 'sepia' ? 'dark' : 'light';
    savePosition();
  }
  function toggleIndent() {
    indentParagraphs = !indentParagraphs;
    savePosition();
  }

  // ── Chapter navigation ───────────────────────────────────────────────────
  function goToChapter(index: number) {
    if (!work || index < 0 || index >= chapterList.length) return;
    chapterIndex = index;
    scrollTo(0, 0);
    const saved = restoreReaderState(urlId);
    saved.chapterIndex = index;
    saved.scrollPos = 0;
    saveReaderState(urlId, saved);
    void syncServerProgress('reading');
    // Re-render content for the new chapter
    htmlContent = '';
    renderChapter(index);
    if (showTranslated && work.work_id && translationLocale !== 'en') void loadTranslation(translationLocale);
  }

  function renderChapter(index: number) {
    if (!work) return;
    const ch = work.chapters[index];
    if (!ch) return;
    chapterTitle = ch.title;
    htmlContent = ch.content;
    words = ch.words;
    chapterIndex = index;
    isLastChapter = index >= work.chapters.length - 1;
    showNextUp = false;
    nextUpCandidates = [];
    // Fetch marginalia for new chapter
    void loadMarginalia(index);
    // Deep-link highlight if ?hash= present
    try {
      const h = new URLSearchParams(location.search).get('hash');
      if (h) scrollToHash(h);
    } catch { /* ignore */ }
  }

  function nextChapter() {
    if (isLastChapter) {
      showNextUp = true;
      buildNextUp();
      return;
    }
    goToChapter(chapterIndex + 1);
  }

  function prevChapter() {
    if (chapterIndex === 0) {
      // Already at the first chapter: jump to the fic page.
      // In jsdom unit tests there is no SvelteKit client router, and calling
      // goto() leaks an unhandled rejection from the real router internals —
      // guard by feature-detecting the router before navigating.
      if (typeof window !== 'undefined' && typeof (window as unknown as { __sveltekit_router?: unknown }).__sveltekit_router === 'undefined') {
        return;
      }
      goto(`/works/${urlId}`);
      return;
    }
    goToChapter(chapterIndex - 1);
  }

  function handleKeydown(e: KeyboardEvent) {
    // Don't hijack typing in inputs / selects
    const tag = (e.target as HTMLElement)?.tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || (e.target as HTMLElement)?.isContentEditable) return;
    switch (e.key) {
      case 'ArrowRight': e.preventDefault(); nextChapter(); break;
      case 'ArrowLeft': e.preventDefault(); prevChapter(); break;
      case 'f': case 'F': e.preventDefault(); changeSize(2); break;
      // With a passage selected, D is a quick, keyboard-accessible path to
      // start its Marginalia discussion without reaching for the tooltip.
      case 'd': case 'D':
        if (tooltipVisible) { e.preventDefault(); openComposerForSelection(); }
        break;
    }
  }

  // ── Next Up panel (end-of-fic) ───────────────────────────────────────────
  async function buildNextUp() {
    if (!work) return;
    const candidates: { url_id: string; title: string; author: string; reason: string }[] = [];

    // 1. Next-in-series heuristic: same author, title looks like a sequel
    //    (contains a numeral / "II" / "2" / ordinal). Prefer the one that
    //    sorts right after the current title.
    const sequel = await findSequel(work);
    if (sequel) {
      candidates.push({ ...sequel, reason: 'Next in series' });
    }

    // 2. Top community suggestion (via /api/recommendations/votes)
    try {
      const res = await fetch(`/api/recommendations/votes?url_id=${encodeURIComponent(urlId)}`);
      const json = await res.json();
      if (json.err === 0 && json.suggestions?.length > 0) {
        const top = json.suggestions[0];
        const info = await fetchFicInfo(top.suggested_url_id);
        if (info) {
          candidates.push({
            url_id: top.suggested_url_id,
            title: info.title,
            author: info.author,
            reason: 'Top community suggestion',
          });
        }
      }
    } catch { /* skip gracefully */ }

    // 3. Readers also bookmarked (co-occurrence) — best effort
    try {
      const res = await fetch(`/api/reader/${encodeURIComponent(urlId)}/related`);
      const json = await res.json();
      if (json.err === 0 && Array.isArray(json.related)) {
        for (const rel of json.related.slice(0, 2)) {
          candidates.push({
            url_id: rel.url_id,
            title: rel.title,
            author: rel.author,
            reason: 'Readers also bookmarked',
          });
        }
      }
    } catch { /* skip gracefully */ }

    // 4. Sequential Markov (what readers typically read next) — gated on
    //    logged-in, deduped, serves as fallback/complement when sequel/related
    //    are empty. Queries the `sequential` strategy's rec_transitions.
    if (auth.isLoggedIn) {
      try {
        const res = await fetch(`/api/reader/${encodeURIComponent(urlId)}/sequential`);
        const json = await res.json();
        if (json.err === 0 && Array.isArray(json.sequential)) {
          for (const sq of json.sequential.slice(0, 3)) {
            candidates.push({
              url_id: sq.url_id,
              title: sq.title,
              author: sq.author,
              reason: sq.reason ?? 'What to read next',
            });
          }
        }
      } catch { /* skip gracefully */ }
    }

    // Dedupe by url_id
    const seen = new Set<string>();
    nextUpCandidates = candidates.filter((c) => {
      if (seen.has(c.url_id)) return false;
      seen.add(c.url_id);
      return true;
    });
  }

  async function findSequel(
    w: ReaderWork
  ): Promise<{ url_id: string; title: string; author: string } | null> {
    try {
      const res = await fetch(`/api/reader/${encodeURIComponent(urlId)}/sequel`);
      const json = await res.json();
      if (json.err === 0 && json.sequel) {
        return {
          url_id: json.sequel.url_id,
          title: json.sequel.title,
          author: json.sequel.author,
        };
      }
    } catch { /* skip */ }
    return null;
  }

  async function fetchFicInfo(url_id: string): Promise<{ title: string; author: string } | null> {
    try {
      const res = await fetch(`/api/reader/${encodeURIComponent(url_id)}/meta`);
      const json = await res.json();
      if (json.err === 0 && json.meta) {
        return { title: json.meta.title, author: json.meta.author };
      }
    } catch { /* skip */ }
    return null;
  }

  // ── Mark as read ─────────────────────────────────────────────────────────
  async function markCompleted() {
    if (!work?.work_id || markSaved) return;
    markSaved = true;
    const state = restoreReaderState(urlId);
    state.finished = true;
    saveReaderState(urlId, state);
    try {
      if (auth.isLoggedIn) {
        await updateReadingStatus(work.work_id, 'completed', work.chapters.length);
      }
    } catch { /* offline — local flag is enough for v1 */ }
  }

  // ── Translations ──────────────────────────────────────────────────────
  function translationForCurrentChapter(chapters: { chapter_id?: number; content_html: string }[]): string | null {
    const chapter = chapters.find((ch) => ch.chapter_id === chapterIndex + 1) ?? chapters[chapterIndex];
    return chapter?.content_html ?? null;
  }

  /** Load the saved locale's current chapter translation, if one exists. */
  async function loadTranslation(locale: string) {
    if (!work?.work_id) return;
    translationLoadingLocale = true;
    try {
      const existing = await getChapterTranslations(work.work_id, locale);
      if (existing.err === 0 && existing.chapters?.length > 0) {
        translatedContent = translationForCurrentChapter(existing.chapters);
        translationsAvailable = true;
        // Start on the current chapter when a translation exists for it.
        showTranslated = true;
      } else {
        translationsAvailable = false;
        translatedContent = null;
        showTranslated = false;
      }
      translationLocale = locale;
      try { localStorage.setItem('fichub_locale', locale); } catch { /* ignore */ }
    } catch {
      translationsAvailable = false;
      translatedContent = null;
      showTranslated = false;
    } finally {
      translationLoadingLocale = false;
    }
  }

  function toggleTranslationPicker() {
    translationPickerOpen = !translationPickerOpen;
  }

  async function handleLocaleChange(locale: string) {
    if (locale === translationLocale && translationsAvailable) return;
    translationPickerOpen = false;
    await loadTranslation(locale);
  }

  // ── Mount ────────────────────────────────────────────────────────────────
  onMount(async () => {
    await auth.init();
    // The reader works offline for previously opened fics; only redirect when
    // there is truly no session.
    if (!auth.isLoggedIn) { goto('/'); return; }

    // Restore prefs + position from localStorage
    const saved = restoreReaderState(urlId);
    fontSize = saved.fontSize ?? 18;
    lineHeight = saved.lineHeight ?? 1.6;
    maxWidth = saved.maxWidth ?? 720;
    theme = saved.theme ?? 'light';
    indentParagraphs = saved.indentParagraphs ?? true;
    markSaved = saved.finished ?? false;

    try {
      const loaded = await loadReaderData(urlId);
      work = loaded.work;
      fromCache = loaded.fromCache;
      const w = work;
      chapterList = w.chapters.map((c: { title: string }) => ({ title: c.title, url_id: w.url_id }));
      totalWords = w.words;

      // Restore chapter position (clamped to valid range)
      const startIndex = Math.min(Math.max(saved.chapterIndex ?? 0, 0), w.chapters.length - 1);
      renderChapter(startIndex);

      // Restore scroll position after content renders
      requestAnimationFrame(() => {
        window.scrollTo(0, saved.scrollPos ?? 0);
      });
      // Record a reading history visit (AO3-style per-visit log)
      if (auth.isLoggedIn && work?.work_id) {
        void recordReadHistory(work.work_id, saved.chapterIndex != null ? saved.chapterIndex + 1 : undefined)
          .catch(() => { /* best-effort */ });
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load reader.';
    } finally {
      loading = false;
    }

    // ── Load kudos / bookmark / comment status (AO3 reader parity) ──────
    if (work?.work_id && auth.isLoggedIn) {
      try {
        const [kudosRes, bookmarksRes, commentsRes] = await Promise.all([
          getKudos(work.work_id),
          listBookmarks(),
          fetchThreadedComments(work.work_id, 1, 1),
        ]);

        if (!kudosRes.err) {
          hasKudos = kudosRes.my_kudos;
          kudosCount = kudosRes.kudos_count;
        }

        if (!bookmarksRes.err) {
          isBookmarked = bookmarksRes.bookmarks.some((b) => b.work_id === (work as ReaderWork).work_id);
        }

        if (!commentsRes.err) {
          commentCount = commentsRes.total_top_level;
        }
      } catch { /* best-effort */ }
    } else if (work?.work_id) {
      // Public: get kudos count only
      try {
        const k = await getKudos(work.work_id);
        if (!k.err) {
          kudosCount = k.kudos_count;
        }
      } catch { /* best-effort */ }
    }

    // Translations (best effort)
    translationLocale = localStorage.getItem('fichub_locale') || 'en';
    try {
      const localesRes = await listLocales();
      if (localesRes.err === 0) locales = localesRes.locales ?? [];
    } catch { /* locales list is optional */ }
    if (translationLocale !== 'en' && work?.work_id) {
      try {
        const existing = await getChapterTranslations(work.work_id, translationLocale);
        if (existing.err === 0 && existing.chapters?.length > 0) {
          translatedContent = translationForCurrentChapter(existing.chapters);
          translationsAvailable = true;
        }
      } catch { /* no translations available */ }
    }

    // Scroll progress + keyboard shortcuts
    window.addEventListener('scroll', syncToScroll, { passive: true });
    window.addEventListener('keydown', handleKeydown);
    // Marginalia selection tooltip
    document.addEventListener('mouseup', handleSelection);
    document.addEventListener('selectionchange', () => {
      // selectionchange fires often; defer
      setTimeout(handleSelection, 50);
    });
    // Load show_marginalia pref (defaults true)
    void getShowMarginalia().then((v) => { showMarginalia = v; if (v && work) void loadMarginalia(chapterIndex); }).catch(() => { /* best-effort */ });
    // Initial marginalia load after work is set
    if (work && showMarginalia) void loadMarginalia(chapterIndex);
    // Deep-link hash from URL
    try {
      const hash = new URLSearchParams(location.search).get('hash');
      if (hash) scrollToHash(hash);
    } catch { /* ignore */ }

    // Track browser connectivity for the in-page offline banner. `navigator.onLine`
    // reflects the *browser's* connection state (independent of the SW).
    const updateOnline = () => {
      navigatorOnLine = navigator.onLine;
    };
    window.addEventListener('online', updateOnline);
    window.addEventListener('offline', updateOnline);
    updateOnline();
  });

  // Content shows translation when toggled (kept from v1)
  const displayedContent = $derived(showTranslated && translatedContent ? translatedContent : htmlContent);

  // Decorate paragraphs with marginalia icons when map or content changes
  $effect(() => {
    // Track deps
    void displayedContent;
    void marginaliaMap;
    void showMarginalia;
    if (!showMarginalia || marginaliaMap.size === 0) return;
    // Wait for DOM
    queueMicrotask(() => {
      const container = readerContentEl;
      if (!container) return;
      // Remove old icons
      container.querySelectorAll('.marginalia-icon').forEach((el) => el.remove());
      container.querySelectorAll('[data-passage-hash]').forEach((p) => {
        const hash = p.getAttribute('data-passage-hash') ?? '';
        const entry = marginaliaMap.get(hash);
        // Desktop: margin icon (absolute), Mobile: inline
        if (entry) {
          const icon = document.createElement('a');
          icon.className = 'marginalia-icon';
          icon.textContent = `💬 ${entry.post_count}`;
          icon.href = `/forum/board/discussion-${entry.topic_id}.${entry.topic_id}`;
          if (entry.topic_slug) icon.href = `/forum/board/${entry.topic_slug}.${entry.topic_id}`;
          icon.title = `${entry.post_count} discussion${entry.post_count === 1 ? '' : 's'}`;
          icon.setAttribute('aria-label', `Discuss passage (${entry.post_count})`);
          (p as HTMLElement).style.position = 'relative';
          p.appendChild(icon);
        } else {
          // For paragraphs without entry, add hover trigger for quick discuss (desktop only)
          // No-op; selection tooltip covers new discussions
        }
        // Highlight deep-link
        if (hash && hash === highlightedHash) p.classList.add('marginalia-highlight');
      });
    });
  });

  // ── Reader action bar handlers (AO3 reader parity) ─────────────────────
  async function toggleKudos() {
    if (!work?.work_id || kudosLoading) return;
    kudosLoading = true;
    try {
      if (hasKudos) {
        const res = await removeKudos(work.work_id);
        if (!res.err) { hasKudos = false; kudosCount = Math.max(0, kudosCount - 1); }
      } else {
        const res = await giveKudos(work.work_id);
        if (!res.err) { hasKudos = true; kudosCount = kudosCount + 1; }
      }
    } catch { /* best-effort */ }
    finally { kudosLoading = false; }
  }

  async function toggleBookmark() {
    if (!work?.work_id || bookmarkLoading) return;
    bookmarkLoading = true;
    try {
      if (isBookmarked) {
        const res = await removeBookmark(work.work_id);
        if (!res.err) isBookmarked = false;
      } else {
        const res = await addBookmark(work.work_id);
        if (!res.err) isBookmarked = true;
      }
    } catch { /* best-effort */ }
    finally { bookmarkLoading = false; }
  }

  function markForLater() {
    void toggleBookmark(); // AO3: Mark for Later is a bookmark with status=2
  }

  function scrollToComments() {
    const el = document.getElementById('comments');
    if (el) {
      el.scrollIntoView({ behavior: 'smooth' });
      el.focus();
    }
  }
</script>

<svelte:head>
  {#if chapterTitle}
    <title>{chapterTitle} — FicNexus Reader</title>
  {:else if work?.title}
    <title>{work.title} — FicNexus Reader</title>
  {/if}
</svelte:head>

<!-- Scroll progress bar -->
<div class="scroll-progress" class:dark={theme === 'dark'} class:sepia={theme === 'sepia'}>
  <div class="scroll-progress-fill" style="width: {scrollProgress}%"></div>
</div>

<!-- Offline indicator: navigator.onLine + cache-render fallback (in-page copy so
     the reader owns its banner even when the layout is a bare route shell). -->
{#if !navigatorOnLine || fromCache}
  <div class="offline-banner" role="status" aria-live="polite">
    <span class="dot" aria-hidden="true"></span>
    {fromCache ? t('reader.offlineSaved') : t('reader.offlineCached')}
  </div>
{/if}

{#if uiMode === 'archive'}
<!-- ── Archive mode: AO3-style reading chapter ─────────────── -->
<main class="archive-main">
  <div class="reader-toolbar archive-reader-toolbar" class:dark={theme === 'dark'} class:sepia={theme === 'sepia'}>
    <div class="toolbar-left">
      <a href={`/works/${urlId}`} class="back-link" title={t('reader.backTitle')}>{t('reader.back')}</a>
      <span class="separator">|</span>
      <span class="reader-title">{work?.title || t('reader.reader')}</span>
    </div>
    <div class="toolbar-center">
      <button onclick={() => changeSize(-2)} title={t('reader.decreaseFont') + ' (f −)'} aria-label={t('reader.decreaseFont')}>A−</button>
      <span class="font-size-indicator">{fontSize}px</span>
      <button onclick={() => changeSize(2)} title={t('reader.increaseFont') + ' (f)'} aria-label={t('reader.increaseFont')}>A+</button>
      <button onclick={() => changeLineHeight(-0.1)} title={t('reader.decreaseLineHeight')} aria-label={t('reader.decreaseLineHeight')}>⇕−</button>
      <button onclick={() => changeLineHeight(0.1)} title={t('reader.increaseLineHeight')} aria-label={t('reader.increaseLineHeight')}>⇕+</button>
      <button onclick={() => changeWidth(-40)} title={t('reader.decreaseWidth')} aria-label={t('reader.decreaseWidth')}>⊞−</button>
      <button onclick={() => changeWidth(40)} title={t('reader.increaseWidth')} aria-label={t('reader.increaseWidth')}>⊞+</button>
      <button onclick={toggleIndent} title={t('reader.toggleIndent')} aria-label={t('reader.toggleIndent')} class:active={indentParagraphs}>
        ¶
      </button>
      <button onclick={cycleTheme} title={t('reader.cycleTheme')} aria-label={t('reader.cycleTheme')}>
        {theme === 'dark' ? '☀️' : theme === 'sepia' ? '🌙' : '🕯️'}
      </button>
      {#if translationsAvailable}
        <button
          onclick={() => showTranslated = !showTranslated}
          title={showTranslated ? t('reader.showOriginal') : t('reader.showTranslation', { locale: translationLocale })}
          aria-label={t('reader.toggleTranslation')}
          class:active={showTranslated}
        >
          🌐
        </button>
      {/if}
      {#if locales.length > 0 && work?.work_id}
        <span class="translation-picker">
          <button
            onclick={toggleTranslationPicker}
            aria-label={t('reader.chooseTranslationLang')}
            aria-expanded={translationPickerOpen}
            title={t('reader.chooseTranslationLang')}
            class:active={translationPickerOpen}
          >
            {translationLocale}
          </button>
          {#if translationPickerOpen}
            <span class="translation-menu" role="listbox" aria-label={t('reader.chooseTranslationLang')}>
              {#each locales as loc (loc.code)}
                <button
                  role="option"
                  aria-selected={loc.code === translationLocale}
                  class:selected={loc.code === translationLocale}
                  onclick={() => handleLocaleChange(loc.code)}
                  disabled={translationLoadingLocale}
                >
                  {loc.name}
                </button>
              {/each}
            </span>
          {/if}
        </span>
      {/if}
    </div>
    <div class="toolbar-right">
      <span class="progress">{progress}%</span>
    </div>
  </div>

  <div class="archive-content">
    <div
      class="reader-container archive-reader-container"
      class:dark={theme === 'dark'}
      class:sepia={theme === 'sepia'}
      class:indent={indentParagraphs}
      style="font-size: {fontSize}px; line-height: {lineHeight}; max-width: {maxWidth}px;"
    >
      {#if loading}
        <div class="loading-card">
          <p><span class="spinner"></span> {t('reader.loading')}</p>
        </div>
      {:else if error}
        <div class="error-card">
          <strong>⚠️ {error}</strong>
          <p class="muted">{t('reader.errorHint')}</p>
        </div>
      {:else if work}
        <div class="archive-reader-chapter">
          <header class="archive-reader-head">
            <h2 class="archive-reader-work"><a href={`/works/${urlId}`}>{work.title}</a></h2>
            <h1 class="archive-reader-chapter-title">{chapterTitle || work.title}</h1>
            <p class="byline">{t('reader.by')} {work.author}</p>
            <p class="meta">
              {t('reader.chapterOf', { current: chapterIndex + 1, total: work.chapters.length })} · {words.toLocaleString()} {t('reader.words')}
              {#if chapterList.length > 1}
                · <label class="chapter-label">{t('reader.chapter')}
                  <select bind:value={chapterIndex} onchange={() => goToChapter(chapterIndex)} aria-label={t('reader.jumpToChapter')}>
                    {#each chapterList as ch, i (i)}
                      <option value={i}>{i + 1}. {ch.title}</option>
                    {/each}
                  </select>
                </label>
              {/if}
            </p>

            <!-- Reader action bar (AO3 parity: Kudos, Bookmark, Mark for Later, Comments) -->
            <div class="reader-actions">
              <button
                class="reader-action kudos-btn"
                class:checked={hasKudos}
                onclick={toggleKudos}
                disabled={kudosLoading || !auth.isLoggedIn}
                aria-label={hasKudos ? t('reader.unKudos') : t('reader.giveKudos')}
                title={hasKudos ? t('reader.unKudos') : t('reader.giveKudos')}
              >
                <span class="action-icon">👍</span>
                <span class="action-count">{kudosCount}</span>
              </button>

              <button
                class="reader-action bookmark-btn"
                class:checked={isBookmarked}
                onclick={toggleBookmark}
                disabled={bookmarkLoading || !auth.isLoggedIn}
                aria-label={t('reader.bookmark')}
              >
                <span class="action-icon">🔖</span>
                <span class="action-text">{isBookmarked ? t('reader.bookmarked') : t('reader.bookmark')}</span>
              </button>

              <button
                class="reader-action mfl-btn"
                onclick={markForLater}
                disabled={!auth.isLoggedIn}
                aria-label={t('reader.markForLater')}
              >
                <span class="action-icon">📋</span>
                <span class="action-text">{t('reader.markForLater')}</span>
              </button>

              <a
                href={`/works/${urlId}#comments`}
                class="reader-action comments-link"
                aria-label={t('reader.comments')}
              >
                <span class="action-icon">💬</span>
                <span class="action-count">{commentCount}</span>
              </a>
            </div>
            <div class="chapter-discussion" aria-label="Chapter discussion">
              <ReactionBar targetType="chapter" targetId={chapterIndex + 1} />
              <a class="chapter-discussion-link" href={`/works/${urlId}#comments?chapter=${chapterIndex + 1}`}>
                💬 Discuss chapter {chapterIndex + 1}
              </a>
            </div>
          </header>

          <div class="chapter-nav chapter-nav-top">
            <button class="archive-btn nav-btn" onclick={prevChapter} disabled={chapterIndex === 0 && work.chapters.length <= 1}>
              {t('reader.previous')}
            </button>
            <button class="archive-btn nav-btn" onclick={nextChapter}>
              {#if isLastChapter}
                {t('reader.finish')}
              {:else}
                {t('reader.next')}
              {/if}
            </button>
          </div>

          <div class="reader-content" bind:this={readerContentEl}>
            {@html displayedContent}
          </div>

          {#if tooltipVisible && showMarginalia}
            <button class="marginalia-tooltip" title="Press D to discuss the selected passage" style="top: {tooltipPos.top}px; left: {tooltipPos.left}px; transform: translateX(-50%)" onclick={openComposerForSelection}>
              Discuss this passage
            </button>
          {/if}

          <div class="chapter-nav chapter-nav-bottom">
            <button class="archive-btn nav-btn" onclick={prevChapter} disabled={chapterIndex === 0 && work.chapters.length <= 1}>
              {t('reader.previous')}
            </button>
            <button class="archive-btn nav-btn" onclick={nextChapter}>
              {#if isLastChapter}
                {t('reader.finish')}
              {:else}
                {t('reader.next')}
              {/if}
            </button>
          </div>

          <div class="reader-footer">
            <button
              class="archive-btn mark-read-btn"
              class:saved={markSaved}
              onclick={markCompleted}
              disabled={markSaved || !work.work_id}
            >
              {#if markSaved}
                {t('reader.completed')}
              {:else}
                {t('reader.markCompleted')}
              {/if}
            </button>
          </div>

          {#if showNextUp}
            <section class="next-up" aria-label={t('reader.nextUp')}>
              <h2>{t('reader.nextUp')}</h2>
              <p class="muted">{@html t('reader.youFinished', { title: work.title })}</p>
              {#if nextUpCandidates.length === 0}
                <p class="muted">{t('reader.noRecommendations')}</p>
              {:else}
                <div class="next-up-cards">
                  {#each nextUpCandidates as cand (cand.url_id)}
                    <a class="next-up-card" href={`/read/${cand.url_id}`}>
                      <span class="next-up-reason">{cand.reason}</span>
                      <strong class="next-up-title">{cand.title}</strong>
                      <span class="next-up-author">{t('reader.by')} {cand.author}</span>
                    </a>
                  {/each}
                </div>
              {/if}
            </section>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</main>
{:else}
<div class="reader-toolbar" class:dark={theme === 'dark'} class:sepia={theme === 'sepia'}>
  <div class="toolbar-left">
    <a href="/works/{urlId}" class="back-link" title={t('reader.backTitle')}>{t('reader.back')}</a>
    <span class="separator">|</span>
    <span class="reader-title">{work?.title || t('reader.reader')}</span>
  </div>
  <div class="toolbar-center">
    <button onclick={() => changeSize(-2)} title={t('reader.decreaseFont') + ' (f −)'} aria-label={t('reader.decreaseFont')}>A−</button>
    <span class="font-size-indicator">{fontSize}px</span>
    <button onclick={() => changeSize(2)} title={t('reader.increaseFont') + ' (f)'} aria-label={t('reader.increaseFont')}>A+</button>
    <button onclick={() => changeLineHeight(-0.1)} title={t('reader.decreaseLineHeight')} aria-label={t('reader.decreaseLineHeight')}>⇕−</button>
    <button onclick={() => changeLineHeight(0.1)} title={t('reader.increaseLineHeight')} aria-label={t('reader.increaseLineHeight')}>⇕+</button>
    <button onclick={() => changeWidth(-40)} title={t('reader.decreaseWidth')} aria-label={t('reader.decreaseWidth')}>⊞−</button>
    <button onclick={() => changeWidth(40)} title={t('reader.increaseWidth')} aria-label={t('reader.increaseWidth')}>⊞+</button>
    <button onclick={toggleIndent} title={t('reader.toggleIndent')} aria-label={t('reader.toggleIndent')} class:active={indentParagraphs}>
      ¶
    </button>
    <button onclick={cycleTheme} title={t('reader.cycleTheme')} aria-label={t('reader.cycleTheme')}>
      {theme === 'dark' ? '☀️' : theme === 'sepia' ? '🌙' : '🕯️'}
    </button>
    {#if translationsAvailable}
      <button
        onclick={() => showTranslated = !showTranslated}
        title={showTranslated ? t('reader.showOriginal') : t('reader.showTranslation', { locale: translationLocale })}
        aria-label={t('reader.toggleTranslation')}
        class:active={showTranslated}
      >
        🌐
      </button>
    {/if}
    {#if locales.length > 0 && work?.work_id}
      <span class="translation-picker">
        <button
          onclick={toggleTranslationPicker}
          aria-label={t('reader.chooseTranslationLang')}
          aria-expanded={translationPickerOpen}
          title={t('reader.chooseTranslationLang')}
          class:active={translationPickerOpen}
        >
          {translationLocale}
        </button>
        {#if translationPickerOpen}
          <span class="translation-menu" role="listbox" aria-label={t('reader.chooseTranslationLang')}>
            {#each locales as loc (loc.code)}
              <button
                role="option"
                aria-selected={loc.code === translationLocale}
                class:selected={loc.code === translationLocale}
                onclick={() => handleLocaleChange(loc.code)}
                disabled={translationLoadingLocale}
              >
                {loc.name}
              </button>
            {/each}
          </span>
        {/if}
      </span>
    {/if}
  </div>
  <div class="toolbar-right">
    <span class="progress">{progress}%</span>
  </div>
</div>

<div
  class="reader-container"
  class:dark={theme === 'dark'}
  class:sepia={theme === 'sepia'}
  class:indent={indentParagraphs}
  style="font-size: {fontSize}px; line-height: {lineHeight}; max-width: {maxWidth}px;"
>
  {#if loading}
    <div class="loading-card">
      <p><span class="spinner"></span> {t('reader.loading')}</p>
    </div>
  {:else if error}
    <div class="error-card">
      <strong>⚠️ {error}</strong>
      <p class="muted">{t('reader.errorHint')}</p>
    </div>
  {:else if work}
    <div class="reader-header">
      <h1>{chapterTitle || work.title}</h1>
      <p class="byline">{t('reader.by')} {work.author}</p>
      <p class="meta">
        {t('reader.chapterOf', { current: chapterIndex + 1, total: work.chapters.length })} · {words.toLocaleString()} {t('reader.words')}
        {#if chapterList.length > 1}
          · <label class="chapter-label">{t('reader.chapter')}
            <select bind:value={chapterIndex} onchange={() => goToChapter(chapterIndex)} aria-label={t('reader.jumpToChapter')}>
              {#each chapterList as ch, i (i)}
                <option value={i}>{i + 1}. {ch.title}</option>
              {/each}
            </select>
          </label>
        {/if}
      </p>

      <!-- Reader action bar (AO3 parity: Kudos, Bookmark, Mark for Later, Comments) -->
      <div class="reader-actions">
        <button
          class="reader-action kudos-btn"
          class:checked={hasKudos}
          onclick={toggleKudos}
          disabled={kudosLoading || !auth.isLoggedIn}
          aria-label={hasKudos ? t('reader.unKudos') : t('reader.giveKudos')}
          title={hasKudos ? t('reader.unKudos') : t('reader.giveKudos')}
        >
          <span class="action-icon">👍</span>
          <span class="action-count">{kudosCount}</span>
        </button>

        <button
          class="reader-action bookmark-btn"
          class:checked={isBookmarked}
          onclick={toggleBookmark}
          disabled={bookmarkLoading || !auth.isLoggedIn}
          aria-label={t('reader.bookmark')}
        >
          <span class="action-icon">🔖</span>
          <span class="action-text">{isBookmarked ? t('reader.bookmarked') : t('reader.bookmark')}</span>
        </button>

        <button
          class="reader-action mfl-btn"
          onclick={markForLater}
          disabled={!auth.isLoggedIn}
          aria-label={t('reader.markForLater')}
        >
          <span class="action-icon">📋</span>
          <span class="action-text">{t('reader.markForLater')}</span>
        </button>

        <a
          href={`/works/${urlId}#comments`}
          class="reader-action comments-link"
          aria-label={t('reader.comments')}
        >
          <span class="action-icon">💬</span>
          <span class="action-count">{commentCount}</span>
        </a>
      </div>
      <div class="chapter-discussion" aria-label="Chapter discussion">
        <ReactionBar targetType="chapter" targetId={chapterIndex + 1} />
        <a class="chapter-discussion-link" href={`/works/${urlId}#comments?chapter=${chapterIndex + 1}`}>
          💬 Discuss chapter {chapterIndex + 1}
        </a>
      </div>
    </div>

    <div class="chapter-nav chapter-nav-top">
      <button class="nav-btn" onclick={prevChapter} disabled={chapterIndex === 0 && work.chapters.length <= 1}>
        {t('reader.previous')}
      </button>
      <button class="nav-btn" onclick={nextChapter}>
        {#if isLastChapter}
          {t('reader.finish')}
        {:else}
          {t('reader.next')}
        {/if}
      </button>
    </div>

    <div class="reader-content" bind:this={readerContentEl}>
      {@html displayedContent}
    </div>

    <div class="chapter-nav chapter-nav-bottom">
      <button class="nav-btn" onclick={prevChapter} disabled={chapterIndex === 0 && work.chapters.length <= 1}>
        {t('reader.previous')}
      </button>
      <button class="nav-btn" onclick={nextChapter}>
        {#if isLastChapter}
          {t('reader.finish')}
        {:else}
          {t('reader.next')}
        {/if}
      </button>
    </div>

    <div class="reader-footer">
      <button
        class="mark-read-btn"
        class:saved={markSaved}
        onclick={markCompleted}
        disabled={markSaved || !work.work_id}
      >
        {#if markSaved}
          {t('reader.completed')}
        {:else}
          {t('reader.markCompleted')}
        {/if}
      </button>
    </div>

    {#if showNextUp}
      <section class="next-up" aria-label={t('reader.nextUp')}>
        <h2>{t('reader.nextUp')}</h2>
        <p class="muted">{@html t('reader.youFinished', { title: work.title })}</p>
        {#if nextUpCandidates.length === 0}
          <p class="muted">{t('reader.noRecommendations')}</p>
        {:else}
          <div class="next-up-cards">
            {#each nextUpCandidates as cand (cand.url_id)}
              <a class="next-up-card" href="/read/{cand.url_id}">
                <span class="next-up-reason">{cand.reason}</span>
                <strong class="next-up-title">{cand.title}</strong>
                <span class="next-up-author">{t('reader.by')} {cand.author}</span>
              </a>
            {/each}
          </div>
        {/if}
      </section>
    {/if}
  {/if}
  </div>
{/if}
{#if composerPayload}
  <MarginaliaComposerModal
    open={composerOpen}
    payload={composerPayload}
    passageText={composerPassageText}
    ficTitle={work?.title ?? ''}
    chapterIndex={chapterIndex}
    onClose={() => composerOpen = false}
    onCreated={handleMarginaliaCreated}
  />
{/if}
<style>
  /* ── Archive mode ─────────────────────────────────────────── */
  .archive-main {
    max-width: var(--archive-max-width, 1100px);
    margin: 0 auto;
    padding: 0;
  }
  .archive-reader-toolbar {
    font-family: Georgia, 'Times New Roman', serif;
  }
  .archive-content {
    max-width: 900px;
    margin: 0 auto;
    padding: 1rem;
    width: 100%;
    box-sizing: border-box;
  }
  .archive-reader-container {
    background: #ffffff;
    color: #2a2a2a;
    border: 1px solid var(--archive-border, #dddddd);
    padding: 2rem 3rem;
    margin: 0 auto;
    font-family: Georgia, 'Times New Roman', serif;
  }
  .archive-reader-head {
    text-align: center;
    padding-bottom: 1em;
    border-bottom: 2px solid var(--archive-link, #990000);
    margin-bottom: 1.5em;
  }
  .archive-reader-work {
    margin: 0 0 0.4em;
    font-size: 1.05em;
    font-weight: normal;
    letter-spacing: 0.02em;
  }
  .archive-reader-work a {
    color: var(--archive-muted, #666666);
    text-decoration: none;
  }
  .archive-reader-work a:hover { text-decoration: underline; }
  .archive-reader-chapter-title {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.7em;
    font-weight: 700;
    margin: 0 0 0.3em;
    color: var(--archive-text, #2a2a2a);
  }
  .nav-btn.archive-btn,
  .mark-read-btn.archive-btn {
    display: inline-block;
    padding: 0.32rem 0.9rem;
    font-size: 0.88em;
    font-family: Georgia, 'Times New Roman', serif;
    font-weight: 600;
    color: var(--archive-text, #2a2a2a);
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    cursor: pointer;
    line-height: 1.5;
  }
  .nav-btn.archive-btn:hover:not(:disabled),
  .mark-read-btn.archive-btn:hover:not(:disabled) {
    background: var(--archive-border, #dddddd);
    border-color: var(--archive-muted, #666666);
  }
  .mark-read-btn.archive-btn.saved {
    color: #ffffff;
    background: #2e7d32;
    border-color: #2e7d32;
  }
  @media (max-width: 600px) {
    .archive-reader-container { padding: 1rem 1.25rem; }
  }

  /* ── Modern mode ── */
  .scroll-progress {
    position: fixed; top: 0; left: 0; right: 0; height: 3px; z-index: 200;
    background: transparent;
  }
  .offline-banner {
    position: fixed;
    bottom: 0.75rem;
    left: 50%;
    transform: translateX(-50%);
    z-index: 1000;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: var(--color-surface-2, #1f2430);
    color: var(--color-text, #e8eaf0);
    border: 1px solid var(--color-border, #2a3040);
    border-radius: var(--radius-sm, 8px);
    padding: 0.5rem 1rem;
    font-size: 0.85rem;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.45);
  }
  .offline-banner .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-warning, #f59e0b);
    flex-shrink: 0;
  }
  .scroll-progress-fill {
    height: 100%;
    background: var(--color-primary, #3b82f6);
    transition: width 0.08s linear;
  }
  .reader-toolbar {
    position: sticky; top: 0; z-index: 100;
    display: flex; justify-content: space-between; align-items: center;
    padding: 0.5rem 1rem;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    font-size: 0.88rem;
    gap: 0.5rem; flex-wrap: wrap;
  }
  .reader-toolbar.dark {
    background: #1a1a1a;
    border-color: #333;
    color: #ddd;
  }
  .reader-toolbar.sepia {
    background: #efe6d0;
    border-color: #d8c9a8;
    color: #4a3f2f;
  }
  .reader-toolbar button {
    padding: 0.3rem 0.6rem;
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
    cursor: pointer;
    font-size: 0.85rem;
    transition: background 0.15s;
  }
  .reader-toolbar button:hover { background: var(--color-primary); color: white; }
  .reader-toolbar button.active { background: var(--color-primary); color: white; border-color: var(--color-primary); }
  .reader-toolbar.dark button { background: #333; border-color: #444; color: #ddd; }
  .reader-toolbar.dark button:hover { background: #555; }
  .reader-toolbar.sepia button { background: #f7f0dd; border-color: #d8c9a8; color: #4a3f2f; }
  .reader-toolbar.sepia button:hover { background: #d8c9a8; color: #4a3f2f; }
  .toolbar-left { display: flex; align-items: center; gap: 0.4rem; min-width: 0; flex: 1; }
  .toolbar-center { display: flex; gap: 0.3rem; align-items: center; flex-wrap: wrap; }
  .toolbar-right { display: flex; align-items: center; justify-content: flex-end; flex: 1; }
  .translation-picker { position: relative; display: inline-flex; align-items: center; }
  .translation-picker .translation-menu {
    position: absolute; top: 100%; right: 0; z-index: 200;
    display: flex; flex-direction: column; gap: 0.15rem;
    background: var(--color-surface); border: 1px solid var(--color-border);
    border-radius: var(--radius-sm); padding: 0.35rem; min-width: 140px;
    max-height: 260px; overflow-y: auto;
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.25);
  }
  .translation-picker .translation-menu button {
    text-align: left; white-space: nowrap;
  }
  .translation-picker .translation-menu button.selected {
    background: var(--color-primary); color: white; border-color: var(--color-primary);
  }
  .reader-container {
    margin: 0 auto;
    padding: 1rem 1.5rem 4rem;
    color: var(--color-text);
    background: var(--color-surface);
  }
  .reader-container.dark { background: #111; color: #ccc; }
  .reader-container.sepia { background: #f5ecd7; color: #4a3f2f; }

  .reader-container :global(p) { margin: 0.8em 0; line-height: inherit; }
  .reader-container.indent :global(p) { text-indent: 1.5em; }
  .reader-container.indent :global(p:first-of-type) { text-indent: 0; }
  .reader-container :global(h2) { margin: 1.5em 0 0.5em; font-size: 1.3em; }
  .reader-container :global(h3) { margin: 1.2em 0 0.4em; font-size: 1.15em; }
  .reader-container :global(a) { color: var(--color-primary); }
  .reader-container :global(blockquote) {
    margin: 1em 0; padding: 0.5em 1em;
    border-left: 3px solid var(--color-border); color: var(--color-muted);
  }
  .reader-container :global(pre) {
    overflow-x: auto; padding: 0.8em;
    background: var(--color-surface-2); border-radius: var(--radius-sm); font-size: 0.9em;
  }
  .reader-container :global(img) { max-width: 100%; height: auto; }
  .reader-container :global(hr) { border: none; border-top: 1px solid var(--color-border); margin: 2em 0; }
  .reader-container.dark :global(img) { opacity: 0.85; }

  .reader-header { text-align: center; margin-bottom: 1.2rem; padding-bottom: 0.8rem; border-bottom: 1px solid var(--color-border); }
  .reader-header h1 { margin: 0 0 0.3rem; font-size: 1.6rem; }
  .reader-content { min-height: 60vh; }
  .reader-footer { text-align: center; margin-top: 3rem; padding-top: 2rem; border-top: 1px solid var(--color-border); }
  .mark-read-btn {
    padding: 0.5rem 1.5rem; border-radius: var(--radius-sm);
    background: var(--color-primary); color: white; border: none; cursor: pointer;
    font-size: 1rem; transition: opacity 0.15s;
  }
  .mark-read-btn:disabled { opacity: 0.6; cursor: default; }
  .mark-read-btn.saved { background: #2e7d32; }
  .byline { font-size: 1.1rem; color: var(--color-muted); }
  .meta { font-size: 0.88rem; color: var(--color-muted); margin-top: 0.4rem; }

  /* ── Reader action bar (AO3 parity) ── */
  .reader-actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 1rem;
    flex-wrap: wrap;
    justify-content: center;
  }
  .reader-action {
    display: inline-flex;
    align-items: center;
    gap: 0.3em;
    padding: 0.4rem 0.8rem;
    font-size: 0.85rem;
    font-weight: 600;
    border: 1px solid var(--color-border, #ddd);
    border-radius: var(--radius-sm, 6px);
    background: var(--color-surface-2, #f5f5f5);
    color: var(--color-text, #2a2a2a);
    cursor: pointer;
    text-decoration: none;
    transition: all 0.15s;
    white-space: nowrap;
  }
  .reader-action:hover {
    background: var(--color-primary, #990000);
    color: #fff;
    border-color: var(--color-primary, #990000);
  }
  .reader-action:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .reader-action:disabled:hover {
    background: var(--color-surface-2, #f5f5f5);
    color: var(--color-text, #2a2a2a);
    border-color: var(--color-border, #ddd);
  }
  .reader-action.checked {
    background: var(--color-primary, #990000);
    color: #fff;
    border-color: var(--color-primary, #990000);
  }
  .action-icon { font-size: 1.1em; }
  .action-count { font-size: 0.9em; }
  .chapter-label select {
    font-size: 0.85rem; padding: 0.15rem 0.3rem;
    border: 1px solid var(--color-border); border-radius: var(--radius-sm);
    background: var(--color-surface-2); color: inherit;
    max-width: 260px;
  }
  .progress { font-family: var(--mono); font-size: 0.82rem; color: var(--color-muted); }
  .back-link { text-decoration: none; color: var(--color-primary); }
  .separator { color: var(--color-border); }
  .reader-title { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 200px; }
  .font-size-indicator { min-width: 2.5rem; text-align: center; font-family: var(--mono); font-size: 0.82rem; }
  .loading-card, .error-card { text-align: center; padding: 3rem; }
  .error-card strong { color: var(--color-error); display: block; margin-bottom: 0.5rem; }
  .muted { color: var(--color-muted); font-size: 0.9rem; }
  .spinner {
    display: inline-block; width: 1rem; height: 1rem;
    border: 2px solid var(--color-border); border-top-color: var(--color-primary);
    border-radius: 50%; animation: spin 0.6s linear infinite;
    vertical-align: middle; margin-right: 0.4rem;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .chapter-nav {
    display: flex; justify-content: space-between; gap: 0.5rem;
    margin: 1rem 0;
  }
  .chapter-nav-top { margin-bottom: 1.5rem; }
  .chapter-nav-bottom { margin-top: 2.5rem; }
  .nav-btn {
    padding: 0.4rem 1rem; border-radius: var(--radius-sm);
    background: var(--color-surface-2); border: 1px solid var(--color-border);
    cursor: pointer; font-size: 0.9rem; color: inherit;
  }
  .nav-btn:hover:not(:disabled) { background: var(--color-primary); color: white; }
  .nav-btn:disabled { opacity: 0.4; cursor: default; }
  .reader-container.dark .nav-btn { background: #222; border-color: #333; }
  .reader-container.sepia .nav-btn { background: #f7f0dd; border-color: #d8c9a8; }

  .next-up {
    margin-top: 3rem; padding: 1.5rem;
    border: 1px solid var(--color-border); border-radius: var(--radius-md, 8px);
    background: var(--color-surface-2, #f5f5f5);
    text-align: center;
  }
  .reader-container.dark .next-up { background: #1c1c1c; border-color: #333; }
  .reader-container.sepia .next-up { background: #efe6d0; border-color: #d8c9a8; }
  .next-up h2 { margin: 0 0 0.4rem; font-size: 1.3rem; }
  .next-up-cards { display: flex; flex-direction: column; gap: 0.6rem; margin-top: 1rem; }
  .next-up-card {
    display: flex; flex-direction: column; gap: 0.15rem;
    padding: 0.8rem 1rem; text-align: left;
    background: var(--color-surface); border: 1px solid var(--color-border);
    border-radius: var(--radius-sm); text-decoration: none; color: inherit;
    transition: border-color 0.15s;
  }
  .next-up-card:hover { border-color: var(--color-primary); }
  .reader-container.dark .next-up-card { background: #111; }
  .reader-container.sepia .next-up-card { background: #faf3e2; }
  .next-up-reason {
    font-size: 0.72rem; text-transform: uppercase; letter-spacing: 0.04em;
    color: var(--color-primary);
  }
  .next-up-title { font-size: 1rem; }
  .next-up-author { font-size: 0.85rem; color: var(--color-muted); }

  @media (max-width: 600px) {
    .reader-toolbar { flex-direction: column; gap: 0.3rem; padding: 0.4rem 0.6rem; }
    .toolbar-left, .toolbar-right { flex: none; width: 100%; }
    .reader-title { max-width: 140px; }
    .reader-container { padding: 0.5rem 0.8rem 3rem; }
    .reader-header h1 { font-size: 1.3rem; }
  }
  /* ── Marginalia ─────────────────────────────────────────────── */
  .marginalia-tooltip {
    position: absolute; z-index: 50;
    padding: 0.35rem 0.7rem; font-size: 0.85rem; font-weight: 600;
    background: var(--color-text, #222); color: #fff; border: none; border-radius: 6px;
    cursor: help; box-shadow: 0 2px 8px rgba(0,0,0,0.2);
  }
  :global(.marginalia-icon) {
    position: absolute; right: -3.2rem; top: 0.1rem;
    font-size: 0.75rem; text-decoration: none; cursor: help;
    background: var(--color-bg-alt, #f6f6f8); border: 1px solid var(--color-border, #ddd);
    border-radius: 999px; padding: 0.1rem 0.4rem; line-height: 1.4;
  }
  :global(.marginalia-highlight) {
    background: #fff3b0 !important; outline: 2px solid #f0c040; border-radius: 3px;
    animation: marginalia-fade 3s ease forwards;
  }
  @keyframes marginalia-fade { 0% { background: #ffe680; } 100% { background: transparent; } }
  @media (max-width: 900px) {
    :global(.marginalia-icon) { position: static; display: inline-block; margin-left: 0.4rem; vertical-align: middle; }
  }
</style>
