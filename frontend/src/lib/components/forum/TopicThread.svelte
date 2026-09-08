<script lang="ts">
  // Forum topic thread (F3): renders the OP + replies as sanitized markdown,
  // with reply composer, follow toggle, and edit/delete affordances for the
  // author (within the 15-minute edit window) and moderators (role ≥ 5).
  //
  // Shared by two routes:
  //   * /forum/board/{topicSlug}.{topicId}  — canonical (slug form)
  //   * /forum/{categorySlug}/{topicId}     — legacy numeric fallback
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { marked } from 'marked';
  import DOMPurify from 'dompurify';
  import {
    getForumTopic,
    markTopicRead,
    createPost,
    updatePost,
    getForumEditHistory,
    type ForumEditProposal,
    deletePost,
    updateTopic,
    deleteTopic,
    toggleFollow,
    getFollowState,
    getModerationStatus,
    moderatePost,
    getModerationUserGrants,
    type ForumUserGrant,
    reportForumPost,
    reportForumTopic,
    reactToPost,
    pinTopic,
    lockTopic,
    MODERATION_REASONS,
    type ForumTopicDetail,
    type ForumPost,
    type ForumModStatus,
  } from '$lib/api/forum';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';
    import PassageContextCard from './PassageContextCard.svelte';
  import ReactionPicker from './ReactionPicker.svelte';
  import PollBar from './PollBar.svelte';

  const uiMode = $derived(getPref('uiMode'));

  let {
    topicId,
    categorySlug = '',
    backHref = '',
  }: { topicId: number; categorySlug?: string; backHref?: string } = $props();

  let topic = $state<ForumTopicDetail | null>(null);
  let posts = $state<ForumPost[]>([]);
  let nextCursor = $state<number | null>(null);
  let loading = $state(true);
  let error = $state('');

  // F5: moderation + reporting
  let modStatus = $state<ForumModStatus | null>(null);
  let openModPostId = $state<number | null>(null);
  let moddingPostId = $state<number | null>(null);
  let modMessage = $state<{ postId: number; ok: boolean; text: string } | null>(null);
  let moddedPostIds = $state<Set<number>>(new Set());
  let reportingPostId = $state<number | null>(null);
  let reportReason = $state('');
  let reporting = $state(false);
  let reportMessage = $state<{ postId: number; ok: boolean; text: string } | null>(null);
  // Topic-level report (reuses reportReason/reporting state)
  let reportingTopic = $state(false);
  let reportTopicMessage = $state<{ ok: boolean; text: string } | null>(null);
  let expandedPostIds = $state<Set<number>>(new Set());

  // T037: mod drawer — per-user recent grants
  let modDrawerUserId = $state<number | null>(null);
  let modDrawerGrants = $state<ForumUserGrant[]>([]);
  let modDrawerLoading = $state(false);
  let modDrawerError = $state('');
  async function openModDrawer(userId: number) {
    if (modDrawerUserId === userId) { modDrawerUserId = null; return; }
    modDrawerUserId = userId;
    modDrawerGrants = [];
    modDrawerError = '';
    modDrawerLoading = true;
    try {
      const res = await getModerationUserGrants(userId);
      if (res.err === 0) modDrawerGrants = res.items ?? [];
      else modDrawerError = res.msg ?? 'Failed to load grants';
    } catch { modDrawerError = 'Failed to load grants'; }
    finally { modDrawerLoading = false; }
  }

  // Reply composer
  let replyBody = $state('');
  let postingReply = $state(false);
  let replyError = $state('');
  let replyOk = $state(false);
  let showPreview = $state(false);

  // Follow
  let following = $state(false);
  let followerCount = $state(0);
  let togglingFollow = $state(false);

  // Edit/delete
  let editingPostId = $state<number | null>(null);
  let editBody = $state('');
  let savingEdit = $state(false);
  let actionError = $state('');
  let deletingTopic = $state(false);
  let topicDeleted = $state(false);
  let editHistory = $state<ForumEditProposal[]>([]);
  let showEditHistory = $state(false);


  // Positive-only emoji set (mirrors backend ALLOWED_REACTIONS) — see
  // FORUM_REACTIONS in $lib/api/forum; rendered via the anchored picker.

  let reactingPostId = $state<number | null>(null);

  const isLocked = $derived(topic?.status === 'locked');
  const isCurator = $derived(auth.level >= 50);
  let togglingPin = $state(false);
  let togglingLock = $state(false);

  async function handleTogglePin() {
    if (!topic || !isCurator || togglingPin) return;
    togglingPin = true;
    try {
      const wantPinned = topic.status !== 'pinned';
      const res = await pinTopic(topicId, wantPinned);
      if (res.err === 0) topic.status = (res.status as typeof topic.status) ?? (wantPinned ? 'pinned' : 'open');
    } finally { togglingPin = false; }
  }
  async function handleToggleLock() {
    if (!topic || !isCurator || togglingLock) return;
    togglingLock = true;
    try {
      const wantLocked = topic.status !== 'locked';
      const res = await lockTopic(topicId, wantLocked);
      if (res.err === 0) topic.status = (res.status as typeof topic.status) ?? (wantLocked ? 'locked' : 'open');
    } finally { togglingLock = false; }
  }

  function renderMarkdown(source: string): string {
    const raw = marked.parse(source ?? '', { async: false }) as string;
    return DOMPurify.sanitize(raw, { USE_PROFILES: { html: true } });
  }

  function canEditPost(post: ForumPost): boolean {
    const level = auth.level;
    if (level >= 50) return true; // mods edit anytime
    if (auth.user?.id !== post.author_id) return false;
    return withinEditWindow(post.created_at); // authors edit ≤15 min
  }

  function canDeletePost(post: ForumPost): boolean {
    const level = auth.level;
    if (level >= 50) return true;
    return auth.user?.id === post.author_id;
  }

  function canEditTopic(): boolean {
    const level = auth.level;
    if (level >= 50) return true;
    if (!topic || auth.user?.id !== topic.author_id) return false;
    return withinEditWindow(topic.created_at); // authors edit ≤15 min
  }

  function canDeleteTopic(): boolean {
    const level = auth.level;
    if (level >= 50) return true;
    return !!topic && auth.user?.id === topic.author_id;
  }

  /** Forum posts are editable by their author only within a 15-minute window
   *  (mirrors the backend contract "author ≤15min or mod"). When the
   *  timestamp is missing we optimistically allow — the server still enforces
   *  the window. */
  const EDIT_WINDOW_MS = 15 * 60 * 1000;
  function withinEditWindow(iso: string | null | undefined): boolean {
    if (!iso) return true;
    return Date.now() - new Date(iso).getTime() < EDIT_WINDOW_MS;
  }

  async function handleReact(post: ForumPost, emoji: string) {
    if (!auth.isLoggedIn) {
      actionError = t('forum.loginToParticipate');
      return;
    }
    reactingPostId = post.id;
    try {
      const res = await reactToPost(post.id, emoji);
      if (res.err === 0) {
        post.reactions = res.reactions;
      }
    } catch {
      // non-fatal
    } finally {
      reactingPostId = null;
    }
  }

  function reactionCount(post: ForumPost, emoji: string): number {
    return post.reactions?.reactions[emoji]?.length ?? 0;
  }

  function reactionUsers(post: ForumPost, emoji: string): string {
    const users = post.reactions?.reactions[emoji];
    if (!users || users.length === 0) return '';
    return users.map(u => u.username).join(', ');
  }

  function myReactions(post: ForumPost): string[] {
    return post.reactions?.my_reactions ?? [];
  }

  // Only show reactions that exist (someone reacted) — the picker adds new ones.
  function visibleReactions(post: ForumPost): string[] {
    const counts = post.reactions?.reactions ?? {};
    const withCounts = Object.keys(counts).filter((e) => (counts[e]?.length ?? 0) > 0);
    for (const e of myReactions(post)) {
      if (!withCounts.includes(e)) withCounts.push(e);
    }
    return withCounts;
  }

  let openPickerPostId = $state<number | null>(null);

  function togglePicker(postId: number) {
    openPickerPostId = openPickerPostId === postId ? null : postId;
  }

  function followersLabel(count: number): string {
    return count === 1 ? t('forum.followers', { count }) : t('forum.followersPlural', { count });
  }

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await getForumTopic(topicId);
      if (res.err === 0) {
        topic = res;
        posts = res.items ?? [];
        nextCursor = res.next_cursor ?? null;
      } else {
        error = res.msg ?? t('forum.topicLoadError');
      }
    } catch {
      error = t('forum.topicLoadError');
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    await auth.init();
    await load();
    if (topic) {
      following = false;
      followerCount = 0;
      // Show the real follow state for logged-in users (GET follow state).
      if (auth.isLoggedIn && Number.isFinite(topicId)) {
        try {
          const fs = await getFollowState(topicId);
          if (fs.err === 0) {
            following = fs.following ?? false;
            followerCount = fs.follower_count ?? 0;
          }
        } catch {
          // keep defaults
        }
      }
      // F5: moderation allowance for logged-in curators (level ≥ 50). This
      // gates the per-post mod buttons; failure just hides them.
      await loadModStatus();
      // F4: mark the topic read up to its last post (debounced fire-and-forget —
      // never block render on this, and ignore failures silently).
      if (auth.isLoggedIn && Number.isFinite(topicId)) {
        const lastId = posts.length > 0 ? posts[posts.length - 1].id : undefined;
        setTimeout(() => {
          markTopicRead(topicId, lastId ?? null).catch(() => {});
        }, 250);
      }
    }
  });

  async function handleFollow() {
    if (!auth.isLoggedIn) {
      actionError = t('forum.loginToParticipate');
      return;
    }
    togglingFollow = true;
    actionError = '';
    try {
      const res = await toggleFollow(topicId);
      if (res.err === 0) {
        following = res.following ?? !following;
        followerCount = res.follower_count ?? followerCount;
      } else {
        actionError = res.msg ?? t('forum.followFailed');
      }
    } catch {
      actionError = t('forum.followFailed');
    } finally {
      togglingFollow = false;
    }
  }

  // ── F5: moderation + reporting ──────────────────────────────────────

  function canModerate(): boolean {
    return auth.isLoggedIn && auth.level >= 50 && (modStatus?.points_left ?? 0) > 0;
  }

  function postModdedByMe(post: ForumPost): boolean {
    return moddedPostIds.has(post.id);
  }

  function isCollapsed(post: ForumPost): boolean {
    return !post.deleted_at && post.score <= -2 && !expandedPostIds.has(post.id);
  }

  function toggleExpand(postId: number) {
    const next = new Set(expandedPostIds);
    if (next.has(postId)) next.delete(postId);
    else next.add(postId);
    expandedPostIds = next;
  }

  async function loadModStatus() {
    if (!auth.isLoggedIn) return;
    try {
      const res = await getModerationStatus();
      if (res.err === 0) modStatus = res;
    } catch {
      // Non-fatal: mod buttons simply stay hidden.
    }
  }

  function openModPicker(post: ForumPost) {
    openModPostId = post.id;
    modMessage = null;
  }

  function closeModPicker() {
    openModPostId = null;
  }

  async function applyModeration(post: ForumPost, reasonKey: string) {
    if (!canModerate() || postModdedByMe(post)) return;
    moddingPostId = post.id;
    modMessage = null;
    try {
      const res = await moderatePost(post.id, reasonKey);
      if (res.err === 0) {
        post.score = res.score_after;
        const marked = new Set(moddedPostIds);
        marked.add(post.id);
        moddedPostIds = marked;
        if (modStatus) {
          modStatus = { ...modStatus, points_left: Math.max(0, (modStatus.points_left ?? 1) - 1) };
        }
        openModPostId = null;
        modMessage = { postId: post.id, ok: true, text: t('forum.moderated', { score: res.score_after }) };
      } else if (res.err === 409) {
        // Already moderated by me — disable this post's mod button.
        const marked = new Set(moddedPostIds);
        marked.add(post.id);
        moddedPostIds = marked;
        openModPostId = null;
        modMessage = { postId: post.id, ok: true, text: t('forum.moderatedByMe') };
      } else {
        modMessage = { postId: post.id, ok: false, text: res.msg ?? t('forum.moderationFailed') };
      }
    } catch {
      modMessage = { postId: post.id, ok: false, text: t('forum.moderationFailed') };
    } finally {
      moddingPostId = null;
    }
  }

  function openReport(post: ForumPost) {
    reportingPostId = post.id;
    reportReason = '';
    reportMessage = null;
  }

  function closeReport() {
    reportingPostId = null;
    reportReason = '';
  }

  function openReportTopic() {
    reportingTopic = true;
    reportReason = '';
    reportTopicMessage = null;
  }

  function closeReportTopic() {
    reportingTopic = false;
    reportReason = '';
  }

  async function submitReportTopic() {
    const reason = reportReason.trim();
    if (!reason) return;
    reporting = true;
    reportTopicMessage = null;
    try {
      const res = await reportForumTopic(topicId, reason);
      if (res.err === 0) {
        reportingTopic = false;
        reportReason = '';
        reportTopicMessage = { ok: true, text: t('forum.reportSent') };
      } else {
        reportTopicMessage = { ok: false, text: res.msg ?? t('forum.reportFailed') };
      }
    } catch {
      reportTopicMessage = { ok: false, text: t('forum.reportFailed') };
    } finally {
      reporting = false;
    }
  }

  async function submitReport(post: ForumPost) {
    const reason = reportReason.trim();
    if (!reason) return;
    reporting = true;
    reportMessage = null;
    try {
      const res = await reportForumPost(post.id, reason);
      if (res.err === 0) {
        reportingPostId = null;
        reportReason = '';
        reportMessage = { postId: post.id, ok: true, text: t('forum.reportSent') };
      } else {
        reportMessage = { postId: post.id, ok: false, text: res.msg ?? t('forum.reportFailed') };
      }
    } catch {
      reportMessage = { postId: post.id, ok: false, text: t('forum.reportFailed') };
    } finally {
      reporting = false;
    }
  }

  async function handleReply() {
    if (!auth.isLoggedIn) {
      replyError = t('forum.loginToParticipate');
      return;
    }
    const body = replyBody.trim();
    if (!body) return;
    postingReply = true;
    replyError = '';
    replyOk = false;
    try {
      const res = await createPost(topicId, { body });
      if (res.err === 0) {
        replyBody = '';
        replyOk = true;
        await load();
      } else {
        replyError = res.msg ?? t('forum.actionFailed');
      }
    } catch {
      replyError = t('forum.actionFailed');
    } finally {
      postingReply = false;
    }
  }

  function startEditPost(post: ForumPost) {
    editingPostId = post.id;
    editBody = post.body;
    actionError = '';
  }

  function cancelEditPost() {
    editingPostId = null;
    editBody = '';
    actionError = '';
  }

  async function saveEditPost(post: ForumPost) {
    const body = editBody.trim();
    if (!body) return;
    savingEdit = true;
    actionError = '';
    try {
      const res = await updatePost(post.id, { body });
      if (res.err === 0) {
        editingPostId = null;
        editBody = '';
        await load();
      } else {
        actionError = res.msg ?? t('forum.actionFailed');
      }
    } catch {
      actionError = t('forum.actionFailed');
    } finally {
      savingEdit = false;
    }
  }

  async function handleDeletePost(post: ForumPost) {
    if (!confirm(t('forum.deleteConfirmPost'))) return;
    actionError = '';
    try {
      const res = await deletePost(post.id);
      if (res.err === 0) {
        await load();
      } else {
        actionError = res.msg ?? t('forum.actionFailed');
      }
    } catch {
      actionError = t('forum.actionFailed');
    }
  }

  let editingTopic = $state(false);
  let topicEditTitle = $state('');
  let topicEditBody = $state('');
  let savingTopic = $state(false);

  function startEditTopic() {
    if (!topic) return;
    editingTopic = true;
    topicEditTitle = topic.title;
    topicEditBody = topic.body;
    actionError = '';
  }

  function cancelEditTopic() {
    editingTopic = false;
    actionError = '';
  }

  async function saveEditTopic() {
    if (!topic) return;
    const title = topicEditTitle.trim();
    const body = topicEditBody.trim();
    if (!title || !body) return;
    savingTopic = true;
    actionError = '';
    try {
      const res = await updateTopic(topic.id, { title, body });
      if (res.err === 0) {
        editingTopic = false;
        await load();
      } else {
        actionError = res.msg ?? t('forum.actionFailed');
      }
    } catch {
      actionError = t('forum.actionFailed');
    } finally {
      savingTopic = false;
    }
  }

  async function handleDeleteTopic() {
    if (!topic) return;
    if (!confirm(t('forum.deleteConfirmTopic'))) return;
    deletingTopic = true;
    actionError = '';
    try {
      const res = await deleteTopic(topic.id);
      if (res.err === 0) {
        topicDeleted = true;
        try {
          await goto(`/forum/${categorySlug}`);
        } catch {
          /* unit tests have no router — deletion succeeded */
        }
      } else {
        actionError = res.msg ?? t('forum.actionFailed');
      }
    } catch {
      actionError = t('forum.actionFailed');
    } finally {
      deletingTopic = false;
    }
  }
</script>

<svelte:head>
  <title>{topic ? t('forum.threadTitle', { title: topic.title }) : 'Forum — FicNexus'}</title>
</svelte:head>

<div class="topic-page">
  <header class="page-head">
    <a class="back" href={backHref || `/forum/${categorySlug}`}>{t('forum.backToTopics')}</a>

    {#if loading}
      <p class="muted"><span class="spinner"></span> {t('forum.loading')}</p>
    {:else if error}
      <div class="error-card"><strong>⚠️ {error}</strong></div>
    {:else if topic}
      {#if topicDeleted}
        <p class="ok">{t('forum.topicDeleted')}</p>
      {:else}
        {#if uiMode === 'archive'}
          <div class="arc-topic-head">
            <h1 class="arc-title">{topic.title}</h1>
            <div class="arc-meta">
              {topic.author_username ?? '—'} · {topic.view_count} views · {topic.created_at.slice(0, 10)}
              {#if topic.status === 'pinned'} <span class="arc-chip">{t('forum.pinned')}</span>{/if}
              {#if topic.status === 'locked'} <span class="arc-chip">{t('forum.locked')}</span>{/if}
            </div>
            {#if canEditTopic() || canDeleteTopic()}
              <div class="arc-actions">
                {#if canEditTopic()}
                  <button class="arc-btn" type="button" onclick={startEditTopic}>{t('forum.edit')}</button>
                {/if}
                {#if canDeleteTopic()}
                  <button class="arc-btn arc-danger" type="button" onclick={handleDeleteTopic} disabled={deletingTopic}>{t('forum.delete')}</button>
                {/if}
              </div>
            {/if}
            {#if isCurator}
              <div class="arc-actions">
                <button class="arc-btn" type="button" onclick={handleTogglePin} disabled={togglingPin}>{topic?.status === 'pinned' ? t('forum.unpin') : t('forum.pin')}</button>
                <button class="arc-btn" type="button" onclick={handleToggleLock} disabled={togglingLock}>{topic?.status === 'locked' ? t('forum.unlock') : t('forum.lock')}</button>
              </div>
            {/if}
            {#if editingTopic}
              <div class="arc-edit-box">
                <label class="arc-label">
                  {t('forum.titleField')}
                  <input class="arc-input" type="text" bind:value={topicEditTitle} maxlength="120" />
                </label>
                <label class="arc-label">
                  {t('forum.topicBodyPlaceholder')}
                  <textarea class="arc-textarea" bind:value={topicEditBody} rows="5"></textarea>
                </label>
                <div class="row">
                  <button class="arc-btn" type="button" onclick={saveEditTopic} disabled={savingTopic || !topicEditTitle.trim() || !topicEditBody.trim()}>
                    {savingTopic ? t('forum.saving') : t('forum.save')}
                  </button>
                  <button class="arc-btn" type="button" onclick={cancelEditTopic}>{t('forum.cancel')}</button>
                </div>
              </div>
            {/if}
            {#if actionError}<div class="error-card"><strong>{actionError}</strong></div>{/if}
          </div>
        {:else}
          <div class="topic-head">
            <div class="title-row">
              <h1>{topic.title}</h1>
              {#if topic.status === 'pinned'}<span class="chip">{t('forum.pinned')}</span>{/if}
              {#if topic.status === 'locked'}<span class="chip">{t('forum.locked')}</span>{/if}
            </div>
            <span class="meta">
              {t('forum.author', { author: topic.author_username ?? '—' })} · {t('forum.views', { count: topic.view_count })} · {topic.created_at.slice(0, 10)}
            </span>
            {#if canEditTopic() || canDeleteTopic()}
              <div class="actions">
                {#if canEditTopic()}
                  <button class="btn-link" type="button" onclick={startEditTopic}>{t('forum.edit')}</button>
                {/if}
                {#if canDeleteTopic()}
                  <button class="btn-link danger" type="button" onclick={handleDeleteTopic} disabled={deletingTopic}>
                    {t('forum.delete')}
                  </button>
                {/if}
              </div>
            {/if}
            {#if isCurator}
              <div class="actions">
                <button class="btn-link" type="button" onclick={handleTogglePin} disabled={togglingPin}>{topic?.status === 'pinned' ? t('forum.unpin') : t('forum.pin')}</button>
                <button class="btn-link" type="button" onclick={handleToggleLock} disabled={togglingLock}>{topic?.status === 'locked' ? t('forum.unlock') : t('forum.lock')}</button>
              </div>
            {/if}
            {#if editingTopic}
              <div class="card edit-box">
                <label>
                  {t('forum.titleField')}
                  <input type="text" bind:value={topicEditTitle} maxlength="120" />
                </label>
                <label>
                  {t('forum.topicBodyPlaceholder')}
                  <textarea bind:value={topicEditBody} rows="5"></textarea>
                </label>
                <div class="row">
                  <button class="btn" type="button" onclick={saveEditTopic} disabled={savingTopic || !topicEditTitle.trim() || !topicEditBody.trim()}>
                    {savingTopic ? t('forum.saving') : t('forum.save')}
                  </button>
                  <button class="btn-link" type="button" onclick={cancelEditTopic}>{t('forum.cancel')}</button>
                </div>
              </div>
            {/if}
            {#if actionError}<div class="error-card"><strong>⚠️ {actionError}</strong></div>{/if}
          </div>
        {/if}

        <div class="follow-row">
          <button
            class={uiMode === 'archive' ? (following ? 'archive-btn arc-following' : 'archive-btn') : 'btn btn-follow'}
            type="button"
            onclick={handleFollow}
            disabled={togglingFollow}
          >
            {following ? t('forum.following') : t('forum.follow')}
          </button>
          <span class="muted">{followersLabel(followerCount)}</span>
          {#if auth.isLoggedIn}
            <button class="btn-link" type="button" onclick={openReportTopic}>{t('forum.reportTopic')}</button>
          {/if}
          <button class="btn-link" type="button" onclick={() => (showEditHistory = !showEditHistory)}>
            {showEditHistory ? 'Hide edit history' : 'Edit history'}
          </button>
        </div>
        {#if reportingTopic}
          <div class="report-box">
            <textarea
              bind:value={reportReason}
              rows="2"
              maxlength="500"
              placeholder={t('forum.reportTopicPlaceholder')}
              aria-label={t('forum.reportTopicPlaceholder')}
            ></textarea>
            <div class="row">
              <button class="btn" type="button" onclick={submitReportTopic} disabled={reporting || !reportReason.trim()}>
                {reporting ? t('forum.saving') : t('forum.reportSubmit')}
              </button>
              <button class="btn-link" type="button" onclick={closeReportTopic}>{t('forum.cancel')}</button>
            </div>
          </div>
        {/if}
        {#if reportTopicMessage}
          <p class={reportTopicMessage.ok ? 'ok' : 'error-card'}>{reportTopicMessage.text}</p>
        {/if}
        {#if showEditHistory}
          <section class="card edit-history" aria-label="Edit history">
            <h2>Edit history</h2>
            {#if editHistory.length === 0}<p class="muted">No edits recorded.</p>{/if}
            {#each editHistory as revision (revision.id)}
              <article><strong>{revision.author_username ?? 'Unknown'}</strong> · {revision.created_at.slice(0, 16)}
                <pre>{JSON.stringify(revision.snapshot, null, 2)}</pre>
                {#if revision.status !== 'approved'}<span class="muted">{revision.status}</span>{/if}
              </article>
            {/each}
          </section>
        {/if}
        {#if isCurator && modDrawerUserId !== null}
          <section class="card mod-drawer" aria-label="Moderation drawer">
            <div class="row"><h2>Mod drawer — user {modDrawerUserId}</h2><button class="btn-link" type="button" onclick={() => (modDrawerUserId = null)}>Close</button></div>
            {#if modDrawerLoading}<p class="muted">Loading grants…</p>
            {:else if modDrawerError}<p class="error-card">{modDrawerError}</p>
            {:else if modDrawerGrants.length === 0}<p class="muted">No recent moderation grants for this user.</p>
            {:else}
              <ul class="mod-drawer-list">
                {#each modDrawerGrants as g (g.id)}<li><span class="arc-chip">{g.reason}</span> delta {g.delta} score {g.score_after} · {g.created_at.slice(0,16)}</li>{/each}
              </ul>
            {/if}
          </section>
        {/if}

                {#if topic?.payload && typeof topic.payload === 'object' && (topic.payload as Record<string, unknown>).type === 'marginalia'}
          <PassageContextCard payload={topic.payload as Record<string, unknown>} ficTitle={topic.title} />
        {/if}

        {#if topic?.poll}
          <PollBar poll={topic.poll} />
        {/if}

        {#if uiMode === 'archive'}
          <!-- Archive: posts as bordered boxes -->
          <div class="arc-posts">
            {#each posts as post (post.id)}
              <div class="arc-post-box">
                <div class="arc-post-header">
                  <span class="arc-post-author">{post.author_username ?? '—'}</span>
                  {#if post.is_op}<span class="arc-chip">OP</span>{/if}
                  <span class="arc-score">{t('forum.moderated', { score: post.score ?? 0 })}</span>
                  <span class="arc-post-date">{post.created_at.slice(0, 10)}{#if post.edited_at} (edited {post.edited_at.slice(0, 10)}){/if}</span>
                </div>
                <div class="arc-post-body">
                  {#if post.deleted_at}
                    <p class="muted deleted-note">{t('forum.postDeleted')}</p>
                  {:else if editingPostId === post.id}
                    <textarea class="arc-textarea" bind:value={editBody} rows="5"></textarea>
                    <div class="arc-actions">
                      <button class="arc-btn" type="button" onclick={() => saveEditPost(post)} disabled={savingEdit || !editBody.trim()}>
                        {savingEdit ? t('forum.saving') : t('forum.save')}
                      </button>
                      <button class="arc-btn" type="button" onclick={cancelEditPost}>{t('forum.cancel')}</button>
                    </div>
                  {:else if isCollapsed(post)}
                    <button class="arc-btn" type="button" onclick={() => toggleExpand(post.id)}>
                      {t('forum.collapsed', { score: post.score ?? 0 })}
                    </button>
                  {:else}
                    {#if post.quote}
                      <blockquote class="arc-quote">
                        <div class="quote-author">{t('forum.quoteOf', { author: post.quote.author_username ?? '—' })}</div>
                        <div class="quote-preview">{post.quote.preview}</div>
                      </blockquote>
                    {/if}
                    <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                    <div class="post-body">{@html renderMarkdown(post.body)}</div>
                  {/if}
                </div>
                {#if !post.deleted_at}
                  <div class="reaction-bar">
                    {#each visibleReactions(post) as emoji (emoji)}
                      <button
                        class="reaction-btn"
                        class:active={myReactions(post).includes(emoji)}
                        type="button"
                        onclick={() => handleReact(post, emoji)}
                        disabled={reactingPostId === post.id}
                        title={reactionUsers(post, emoji) || emoji}
                      >
                        <span class="reaction-emoji">{emoji}</span>
                        {#if reactionCount(post, emoji) > 0}
                          <span class="reaction-count">{reactionCount(post, emoji)}</span>
                        {/if}
                      </button>
                    {/each}
                    {#if auth.isLoggedIn && !post.deleted_at}
                      <span class="reaction-anchor">
                        <button
                          class="reaction-btn reaction-add"
                          class:active={openPickerPostId === post.id}
                          type="button"
                          onclick={() => togglePicker(post.id)}
                          aria-expanded={openPickerPostId === post.id}
                        >{t('forum.react')}</button>
                        {#if openPickerPostId === post.id}
                          <ReactionPicker
                            myReactions={myReactions(post)}
                            onPick={(emoji: string) => handleReact(post, emoji)}
                            onClose={() => (openPickerPostId = null)}
                          />
                        {/if}
                      </span>
                    {/if}
                  </div>
                {/if}
                {#if !post.deleted_at && (canEditPost(post) || canDeletePost(post) || canModerate() || auth.isLoggedIn)}
                  <div class="arc-post-actions">
                    {#if canEditPost(post)}<button class="arc-btn" type="button" onclick={() => startEditPost(post)}>{t('forum.edit')}</button>{/if}
                    {#if canDeletePost(post)}<button class="arc-btn arc-danger" type="button" onclick={() => handleDeletePost(post)}>{t('forum.delete')}</button>{/if}
                    {#if canModerate() && !postModdedByMe(post)}
                      <button class="arc-btn" type="button" onclick={() => openModPicker(post)} disabled={moddingPostId !== null}>{t('forum.modAction')}</button>
                    {/if}
                    {#if auth.isLoggedIn}
                      <button class="arc-btn" type="button" onclick={() => openReport(post)}>{t('forum.report')}</button>
                    {/if}
                    {#if isCurator}<button class="arc-btn" type="button" onclick={() => openModDrawer(post.author_id)}>Mod drawer</button>{/if}
                  </div>
                  {#if openModPostId === post.id}
                    <div class="arc-mod-picker" aria-label="moderation reasons">
                      {#each MODERATION_REASONS as r (r.key)}
                        <button class="arc-btn" class:pos={r.delta > 0} class:neg={r.delta < 0} type="button" onclick={() => applyModeration(post, r.key)} disabled={moddingPostId === post.id}>
                          {r.delta > 0 ? '+' : ''}{r.delta} · {t(r.labelKey)}
                        </button>
                      {/each}
                      <button class="arc-btn" type="button" onclick={closeModPicker}>{t('forum.cancel')}</button>
                    </div>
                  {/if}
                  {#if modMessage && modMessage.postId === post.id}
                    <p class={modMessage.ok ? 'ok' : 'error-card'}>{modMessage.text}</p>
                  {/if}
                {/if}
                {#if reportingPostId === post.id}
                  <div class="arc-report-box">
                    <textarea class="arc-textarea" bind:value={reportReason} rows="2" maxlength="500" placeholder={t('forum.reportPlaceholder')} aria-label={t('forum.reportPlaceholder')}></textarea>
                    <div class="arc-actions">
                      <button class="arc-btn" type="button" onclick={() => submitReport(post)} disabled={reporting || !reportReason.trim()}>
                        {reporting ? t('forum.saving') : t('forum.reportSubmit')}
                      </button>
                      <button class="arc-btn" type="button" onclick={closeReport}>{t('forum.cancel')}</button>
                    </div>
                  </div>
                {/if}
                {#if reportMessage && reportMessage.postId === post.id}
                  <p class={reportMessage.ok ? 'ok' : 'error-card'}>{reportMessage.text}</p>
                {/if}
              </div>
            {/each}
          </div>

          <!-- Archive: reply form as fieldset -->
          {#if isLocked && !isCurator}
            <p class="muted locked-notice">{t('forum.topicLocked')}</p>
          {:else}
          <fieldset class="arc-reply-fieldset">
            <legend>Reply</legend>
            {#if replyError}<div class="error-card"><strong>{replyError}</strong></div>{/if}
            {#if replyOk}<p class="ok">{t('forum.replyPosted')}</p>{/if}
            <label class="arc-label">Comment
              <textarea class="arc-textarea" bind:value={replyBody} rows="4" placeholder={t('forum.replyPlaceholder')}></textarea>
            </label>
            {#if replyBody.trim()}
              <button class="arc-btn" type="button" onclick={() => (showPreview = !showPreview)}>{showPreview ? 'Hide preview' : 'Show preview'}</button>
              {#if showPreview}
                <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                <div class="arc-preview post-body">{@html renderMarkdown(replyBody)}</div>
              {/if}
            {/if}
            <p class="muted hint">{t('forum.mentionHint')}</p>
            <button class="arc-btn arc-submit" type="button" onclick={(e) => { e.preventDefault(); handleReply(); }} disabled={postingReply || !replyBody.trim()}>
              {postingReply ? t('forum.postingReply') : t('forum.postReply')}
            </button>
          </fieldset>
          {/if}
        {:else}
          <!-- Modern: card-based post list -->
          <ul class="post-list">
            {#each posts as post (post.id)}
              <li class="card post-card" class:op-post={post.is_op}>
                <div class="post-head">
                  <strong>{post.author_username ?? '—'}</strong>
                  {#if post.is_op}<span class="chip">{t('forum.op')}</span>{/if}
                  <span class="muted">
                    {t('forum.writtenBy', { when: post.created_at.slice(0, 10) })}
                    {#if post.edited_at}{t('forum.editedAt', { when: post.edited_at.slice(0, 10) })}{/if}
                  </span>
                  <span class="score-badge">{post.score ?? 0}</span>
                </div>

                {#if post.deleted_at}
                  <p class="muted deleted-note">⚠️ {t('forum.postDeleted')}</p>
                {:else if editingPostId === post.id}
                  <textarea bind:value={editBody} rows="5"></textarea>
                  <div class="row">
                    <button class="btn" type="button" onclick={() => saveEditPost(post)} disabled={savingEdit || !editBody.trim()}>
                      {savingEdit ? t('forum.saving') : t('forum.save')}
                    </button>
                    <button class="btn-link" type="button" onclick={cancelEditPost}>{t('forum.cancel')}</button>
                  </div>
                {:else if isCollapsed(post)}
                  <button class="btn collapsed-toggle" type="button" onclick={() => toggleExpand(post.id)}>
                    {t('forum.collapsed', { score: post.score ?? 0 })}
                  </button>
                {:else}
                  {#if post.quote}
                    <blockquote class="quote">
                      <div class="quote-author">{t('forum.quoteOf', { author: post.quote.author_username ?? '—' })}</div>
                      <div class="quote-preview">{post.quote.preview}</div>
                    </blockquote>
                  {/if}
                  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                  <div class="post-body">{@html renderMarkdown(post.body)}</div>
                {/if}

                {#if !post.deleted_at}
                  <div class="reaction-bar">
                    {#each visibleReactions(post) as emoji (emoji)}
                      <button
                        class="reaction-btn"
                        class:active={myReactions(post).includes(emoji)}
                        type="button"
                        onclick={() => handleReact(post, emoji)}
                        disabled={reactingPostId === post.id}
                        title={reactionUsers(post, emoji) || emoji}
                      >
                        <span class="reaction-emoji">{emoji}</span>
                        {#if reactionCount(post, emoji) > 0}
                          <span class="reaction-count">{reactionCount(post, emoji)}</span>
                        {/if}
                      </button>
                    {/each}
                    {#if auth.isLoggedIn && !post.deleted_at}
                      <span class="reaction-anchor">
                        <button
                          class="reaction-btn reaction-add"
                          class:active={openPickerPostId === post.id}
                          type="button"
                          onclick={() => togglePicker(post.id)}
                          aria-expanded={openPickerPostId === post.id}
                        >{t('forum.react')}</button>
                        {#if openPickerPostId === post.id}
                          <ReactionPicker
                            myReactions={myReactions(post)}
                            onPick={(emoji: string) => handleReact(post, emoji)}
                            onClose={() => (openPickerPostId = null)}
                          />
                        {/if}
                      </span>
                    {/if}
                  </div>
                {/if}

                {#if !post.deleted_at && (canEditPost(post) || canDeletePost(post) || canModerate() || auth.isLoggedIn)}
                  <div class="actions">
                    {#if canEditPost(post)}
                      <button class="btn-link" type="button" onclick={() => startEditPost(post)}>{t('forum.edit')}</button>
                    {/if}
                    {#if canDeletePost(post)}
                      <button class="btn-link danger" type="button" onclick={() => handleDeletePost(post)}>{t('forum.delete')}</button>
                    {/if}
                    {#if canModerate() && !postModdedByMe(post)}
                      <button class="btn-link" type="button" onclick={() => openModPicker(post)} disabled={moddingPostId !== null}>
                        {t('forum.modAction')}
                      </button>
                    {/if}
                    {#if auth.isLoggedIn}
                      <button class="btn-link" type="button" onclick={() => openReport(post)}>{t('forum.report')}</button>
                    {/if}
                    {#if isCurator}<button class="btn-link" type="button" onclick={() => openModDrawer(post.author_id)}>Mod drawer</button>{/if}
                  </div>

                  {#if openModPostId === post.id}
                    <div class="mod-picker" aria-label="moderation reasons">
                      <div class="mod-reasons">
                        {#each MODERATION_REASONS as r (r.key)}
                          <button
                            class="btn-link reason"
                            class:pos={r.delta > 0}
                            class:neg={r.delta < 0}
                            type="button"
                            onclick={() => applyModeration(post, r.key)}
                            disabled={moddingPostId === post.id}
                          >
                            {r.delta > 0 ? '+' : ''}{r.delta} · {t(r.labelKey)}
                          </button>
                        {/each}
                      </div>
                      <button class="btn-link muted" type="button" onclick={closeModPicker}>{t('forum.cancel')}</button>
                    </div>
                  {/if}

                  {#if modMessage && modMessage.postId === post.id}
                    <p class={modMessage.ok ? 'ok' : 'error-card'}>{modMessage.text}</p>
                  {/if}
                {/if}

                {#if reportingPostId === post.id}
                  <div class="report-box">
                    <textarea
                      bind:value={reportReason}
                      rows="2"
                      maxlength="500"
                      placeholder={t('forum.reportPlaceholder')}
                      aria-label={t('forum.reportPlaceholder')}
                    ></textarea>
                    <div class="row">
                      <button class="btn" type="button" onclick={() => submitReport(post)} disabled={reporting || !reportReason.trim()}>
                        {reporting ? t('forum.saving') : t('forum.reportSubmit')}
                      </button>
                      <button class="btn-link" type="button" onclick={closeReport}>{t('forum.cancel')}</button>
                    </div>
                  </div>
                {/if}

                {#if reportMessage && reportMessage.postId === post.id}
                  <p class={reportMessage.ok ? 'ok' : 'error-card'}>{reportMessage.text}</p>
                {/if}
              </li>
            {/each}
          </ul>

          {#if nextCursor !== null}
            <nav class="pagination">
              <button class="btn" type="button" onclick={() => load()}>{t('forum.nextPage')}</button>
            </nav>
          {/if}

          {#if isLocked && !isCurator}
            <p class="muted locked-notice">{t('forum.topicLocked')}</p>
          {:else}
          <form class="card reply-box" onsubmit={(e) => { e.preventDefault(); handleReply(); }}>
            <h2>{t('forum.reply')}</h2>
            {#if replyError}<div class="error-card"><strong>⚠️ {replyError}</strong></div>{/if}
            {#if replyOk}<p class="ok">{t('forum.replyPosted')}</p>{/if}
            <textarea bind:value={replyBody} rows="4" placeholder={t('forum.replyPlaceholder')}></textarea>
            {#if replyBody.trim()}
              <button class="btn btn-secondary" type="button" onclick={() => (showPreview = !showPreview)}>{showPreview ? 'Hide preview' : 'Show preview'}</button>
              {#if showPreview}
                <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                <div class="composer-preview post-body">{@html renderMarkdown(replyBody)}</div>
              {/if}
            {/if}
            <p class="muted hint">{t('forum.mentionHint')}</p>
            <button class="btn btn-primary" type="submit" disabled={postingReply || !replyBody.trim()}>
              {postingReply ? t('forum.postingReply') : t('forum.postReply')}
            </button>
          </form>
          {/if}
        {/if}
      {/if}
    {/if}
  </header>
</div>

<style>
  .topic-page { max-width: 820px; margin: 0 auto; padding: 1.5rem; }
  .page-head { display: flex; flex-direction: column; gap: 0.75rem; }
  .back { color: var(--color-link, #2b4bd7); text-decoration: none; font-size: 0.9rem; }
  .title-row { display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; }
  .topic-head h1 { margin: 0; font-size: 1.3rem; }
  .meta { color: var(--color-text-muted, #888); font-size: 0.85rem; }
  .actions { display: flex; gap: 0.75rem; margin-top: 0.35rem; }
  .btn-link { background: none; border: none; color: var(--color-link, #2b4bd7); cursor: pointer; font: inherit; padding: 0; }
  .btn-link.danger { color: #c0392b; }
  .btn-link:disabled { opacity: 0.5; cursor: default; }
  .follow-row { display: flex; align-items: center; gap: 0.75rem; margin: 0.75rem 0; }
  .btn-follow { background: var(--color-surface, #fff); color: var(--color-text, #222); border: 1px solid var(--color-border, #ddd); }
  .post-list { list-style: none; display: flex; flex-direction: column; gap: 0.75rem; padding: 0; }
  .post-card { padding: 1rem; display: flex; flex-direction: column; gap: 0.5rem; }
  .post-head { display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; font-size: 0.95rem; }
  .chip { background: var(--color-chip-bg, #eef); padding: 0.15rem 0.5rem; border-radius: 999px; font-size: 0.75rem; }
  .collapsed-toggle { color: var(--color-text-muted, #888); border: 1px dashed var(--color-border, #ddd); background: var(--color-bg-alt, #f6f6f8); font-size: 0.9rem; }
  .mod-picker, .report-box { display: flex; flex-direction: column; gap: 0.5rem; margin-top: 0.5rem; padding: 0.75rem; border: 1px solid var(--color-border, #ddd); border-radius: var(--radius-sm, 6px); background: var(--color-bg-alt, #f6f6f8); }
  .mod-reasons { display: flex; flex-wrap: wrap; gap: 0.35rem; }
  .reason { font-size: 0.85rem; padding: 0.25rem 0.5rem; border: 1px solid var(--color-border, #ddd); border-radius: 999px; background: var(--color-surface, #fff); }
  .reason.pos { color: #1e8e3e; }
  .reason.neg { color: #c0392b; }
  .report-box textarea { padding: 0.45rem 0.6rem; border-radius: var(--radius-sm, 6px); border: 1px solid var(--color-border, #ddd); font: inherit; }
  .quote { margin: 0.4rem 0; padding: 0.5rem 0.75rem; border-left: 3px solid var(--color-border, #ddd); background: var(--color-bg-alt, #f6f6f8); color: var(--color-text-muted, #888); font-size: 0.9rem; }
  .quote-author { font-weight: 600; margin-bottom: 0.2rem; }
  .quote-preview { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .post-body { line-height: 1.55; overflow-wrap: break-word; }
  .post-body :global(p) { margin: 0.4rem 0; }
  .post-body :global(pre) { overflow-x: auto; background: var(--color-bg-alt, #f6f6f8); padding: 0.6rem; border-radius: var(--radius-sm, 6px); }
  .post-body :global(img) { max-width: 100%; }
  .deleted-note { font-style: italic; }
  .reaction-bar { display: flex; flex-wrap: wrap; gap: 0.35rem; margin-top: 0.25rem; }
  .reaction-anchor { position: relative; display: inline-flex; }
  .reaction-add {
    color: var(--color-text-muted, #666);
    font-size: 0.8rem;
    padding-inline: 0.5rem;
  }
  .reaction-btn {
    display: inline-flex; align-items: center; gap: 0.2rem;
    padding: 0.2rem 0.5rem; border-radius: 999px;
    border: 1px solid var(--color-border, #ddd);
    background: var(--color-surface, #fff);
    cursor: pointer; font-size: 0.85rem; transition: background 0.15s;
  }
  .reaction-btn:hover { background: var(--color-bg-alt, #f0f0f4); }
  .reaction-btn.active { background: #e8f0fe; border-color: #4a8af4; }
  .reaction-btn:disabled { opacity: 0.6; cursor: default; }
  .reaction-emoji { font-size: 1rem; }
  .reaction-count { font-weight: 600; color: var(--color-text-muted, #666); }
  .edit-box, .reply-box { display: flex; flex-direction: column; gap: 0.6rem; padding: 1rem; margin-top: 0.75rem; }
  .edit-history { padding: 1rem; }
  .edit-history article { border-top: 1px solid var(--color-border, #ddd); padding: 0.6rem 0; }
  .edit-history pre { white-space: pre-wrap; overflow-wrap: anywhere; font: inherit; margin: 0.4rem 0 0; }
  .edit-box label, .reply-box label { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.9rem; font-weight: 600; }
  .edit-box input, .edit-box textarea, .reply-box textarea { padding: 0.45rem 0.6rem; border-radius: var(--radius-sm, 6px); border: 1px solid var(--color-border, #ddd); font: inherit; }
  .row { display: flex; align-items: center; gap: 0.75rem; }
  .hint { font-size: 0.8rem; }
  .pagination { display: flex; justify-content: center; margin-top: 1rem; }
  .ok { color: #1e8e3e; font-weight: 600; }
  .muted { color: var(--color-text-muted, #888); }

  /* ── Archive mode ──────────────────────────────────────────────── */
  .arc-topic-head { margin-bottom: 0.75rem; }
  .arc-title {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.3em;
    font-weight: 700;
    margin: 0 0 0.3em;
    color: var(--archive-text, #2a2a2a);
  }
  .arc-meta {
    font-size: 0.88em;
    color: var(--archive-muted, #666666);
    margin-bottom: 0.4em;
  }
  .arc-chip {
    display: inline-block;
    font-size: 0.75em;
    padding: 0.1em 0.4em;
    border: 1px solid var(--archive-border, #dddddd);
    color: var(--archive-muted, #666666);
    text-transform: uppercase;
    letter-spacing: 0.03em;
    vertical-align: middle;
  }
  .arc-actions { display: flex; gap: 0.5rem; margin: 0.4em 0; }
  .arc-btn {
    display: inline-block;
    padding: 0.2em 0.6em;
    font-size: 0.88em;
    font-family: inherit;
    font-weight: 600;
    line-height: 1.6;
    color: var(--archive-text, #2a2a2a);
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    cursor: pointer;
    text-decoration: none;
  }
  .arc-btn:hover { background: var(--archive-border, #dddddd); }
  .arc-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .arc-btn.arc-danger { color: #990000; }
  .arc-btn.arc-submit {
    background: var(--archive-link, #990000);
    color: var(--archive-bg, #ffffff);
    border-color: var(--archive-link, #990000);
  }
  .arc-btn.arc-submit:hover { background: var(--archive-link-visited, #660066); border-color: var(--archive-link-visited, #660066); }
  .arc-btn.pos { color: #1e8e3e; }
  .arc-btn.neg { color: #c0392b; }
  .arc-label {
    display: block;
    font-size: 0.85em;
    font-weight: 700;
    color: var(--archive-muted, #666666);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin-bottom: 0.3em;
  }
  .arc-input, .arc-textarea {
    padding: 0.4em 0.6em;
    font-size: 0.95em;
    font-family: inherit;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
    width: 100%;
    box-sizing: border-box;
  }
  .arc-textarea { resize: vertical; }
  .arc-edit-box { margin: 0.5em 0; }

  /* Archive post list */
  .arc-posts { display: flex; flex-direction: column; gap: 0.75em; margin: 0.75em 0; }
  .arc-post-box {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0;
  }
  .arc-post-header {
    display: flex;
    align-items: center;
    gap: 0.5em;
    padding: 0.4em 0.7em;
    background: var(--archive-bg-raised, #f5f5f5);
    border-bottom: 1px solid var(--archive-border, #dddddd);
    font-size: 0.92em;
  }
  .arc-post-author { font-weight: 700; color: var(--archive-text, #2a2a2a); }
.arc-score { font-size: 0.88em; color: var(--archive-muted, #666666); margin-left: 0.5em; }
  .arc-post-date { color: var(--archive-muted, #666666); font-size: 0.88em; margin-left: auto; }
  .arc-post-body { padding: 0.6em 0.7em; line-height: 1.55; overflow-wrap: break-word; }
  .arc-post-body :global(p) { margin: 0.4rem 0; }
  .arc-post-body :global(pre) { overflow-x: auto; background: var(--archive-bg-raised, #f5f5f5); padding: 0.6rem; }
  .arc-post-body :global(img) { max-width: 100%; }
  .arc-quote {
    margin: 0.4em 0;
    padding: 0.4em 0.6em;
    border-left: 3px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-muted, #666666);
    font-size: 0.9em;
  }
  .arc-post-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4em;
    padding: 0.3em 0.7em;
    border-top: 1px solid var(--archive-border, #dddddd);
  }
  .arc-mod-picker {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35em;
    padding: 0.5em 0.7em;
    border-top: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .arc-report-box {
    padding: 0.5em 0.7em;
    border-top: 1px solid var(--archive-border, #dddddd);
  }

  /* Archive reply fieldset */
  .arc-reply-fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.7em;
    margin: 0.75em 0;
  }
  .arc-reply-fieldset legend {
    font-family: Georgia, 'Times New Roman', serif;
    font-weight: 700;
    font-size: 0.95em;
    color: var(--archive-text, #2a2a2a);
    padding: 0 0.4em;
  }

  /* Archive follow button — both states */
  .follow-row .archive-btn {
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-link, #990000);
    background: var(--archive-bg, #ffffff);
    border-color: var(--archive-link, #990000);
    font-weight: 700;
  }
  .follow-row .archive-btn:hover:not(:disabled) {
    background: var(--archive-bg-raised, #f5f5f5);
    border-color: var(--archive-link-visited, #660066);
    color: var(--archive-link-visited, #660066);
  }
  .follow-row .archive-btn.arc-following {
    color: var(--archive-bg, #ffffff);
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }
  .follow-row .archive-btn.arc-following:hover:not(:disabled) {
    background: var(--archive-link-visited, #660066);
    border-color: var(--archive-link-visited, #660066);
    color: var(--archive-bg, #ffffff);
  }
  .composer-preview, .arc-preview { margin: 0.5rem 0; padding: 0.6rem 0.7rem; border: 1px solid var(--color-border, #ddd); border-radius: 6px; background: var(--color-bg-alt, #f6f6f8); }
  .arc-preview { background: var(--archive-bg-raised, #f5f5f5); border-color: var(--archive-border, #dddddd); }
</style>
