<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { fetchThreadedComments, postThreadedComment, editComment, deleteComment } from '$lib/api/social';
  import ReactionBar from './ReactionBar.svelte';
  import type { ThreadedComment } from '$lib/api/social';
  import { getPref } from '$lib/prefs';
  const uiMode = $derived(getPref('uiMode'));

  let { workId, urlId }: { workId?: number; urlId?: string } = $props();
  const identifier = $derived(workId ?? urlId ?? '');

  let comments = $state<ThreadedComment[]>([]);
  let totalTopLevel = $state(0);
  let page = $state(1);
  let loading = $state(false);
  let error = $state('');
  let newCommentBody = $state('');
  let replyingTo = $state<number | null>(null);
  let submitting = $state(false);
  let collapsed = $state<Set<number>>(new Set());
  let editingComment: number | null = $state(null);
  let editBody: string = $state('');
  let deletingCommentId: number | null = $state(null);

  // 7-color HSL palette — matches the old Lemmy approach
  const LINE_COLORS = [
    'hsla(0, 35%, 50%, 1)',
    'hsla(50, 35%, 50%, 1)',
    'hsla(100, 35%, 50%, 1)',
    'hsla(150, 35%, 50%, 1)',
    'hsla(200, 35%, 50%, 1)',
    'hsla(250, 35%, 50%, 1)',
    'hsla(300, 35%, 50%, 1)',
  ];

  function lineColor(depth: number): string {
    if (depth === 0) return LINE_COLORS[0];
    return LINE_COLORS[(depth - 1) % LINE_COLORS.length];
  }

  onMount(() => {
    loadComments();
  });

  async function loadComments(p = 1) {
    loading = true;
    error = '';
    try {
      const data = await fetchThreadedComments(identifier, p);
      if (data.err !== 0) throw new Error('Failed to load comments');
      const incoming = Array.isArray(data.comments) ? data.comments : [];
      if (p === 1) {
        comments = incoming;
      } else {
        comments = [...comments, ...incoming];
      }
      totalTopLevel = data.total_top_level;
      page = data.page;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load comments';
    } finally {
      loading = false;
    }
  }

  async function submitComment() {
    if (!newCommentBody.trim() || submitting) return;
    submitting = true;
    try {
      const data = await postThreadedComment(identifier, newCommentBody.trim(), replyingTo ?? undefined);
      if (data.err !== 0) throw new Error('Failed to post comment');
      newCommentBody = '';
      replyingTo = null;
      await loadComments(1);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to post comment';
    } finally {
      submitting = false;
    }
  }

  function toggleCollapse(id: number) {
    const next = new Set(collapsed);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    collapsed = next;
  }

  function startReply(commentId: number) {
    replyingTo = commentId;
    newCommentBody = '';
  }

  function cancelReply() {
    replyingTo = null;
  }

  function startEdit(comment: ThreadedComment) {
    editingComment = comment.id;
    editBody = comment.body;
  }

  function cancelEdit() {
    editingComment = null;
    editBody = '';
  }

  async function saveEdit() {
    if (editingComment === null) return;
    const commentId = editingComment;
    try {
      const data = await editComment(commentId, editBody);
      if (data.err === 0) {
        editingComment = null;
        editBody = '';
        await loadComments(1);
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to edit comment';
    }
  }

  async function deleteCommentHandler(comment: ThreadedComment) {
    if (!confirm('Delete this comment?')) return;
    deletingCommentId = comment.id;
    try {
      const data = await deleteComment(comment.id);
      if (data.err === 0) {
        await loadComments(1);
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to delete comment';
    } finally {
      deletingCommentId = null;
    }
  }

  // Build tree from flat list
  let tree = $derived.by(() => {
    const map = new Map<number, ThreadedComment & { children: ThreadedComment[] }>();
    const roots: (ThreadedComment & { children: ThreadedComment[] })[] = [];
    const items = Array.isArray(comments) ? comments : [];

    for (const c of items) {
      map.set(c.id, { ...c, children: [] });
    }
    for (const c of items) {
      const node = map.get(c.id)!;
      if (c.parent_id && map.has(c.parent_id)) {
        map.get(c.parent_id)!.children.push(node);
      } else if (!c.parent_id) {
        roots.push(node);
      }
    }
    return roots;
  });

  function formatDate(iso: string): string {
    const d = new Date(iso);
    return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  }
</script>

<div class="comment-section" class:archive={uiMode === 'archive'}>
  <h3>Comments ({totalTopLevel})</h3>

  {#if auth.isLoggedIn}
    <div class="new-comment-form">
      {#if replyingTo !== null}
        <div class="replying-to">
          Replying to comment... <button class="btn-text" onclick={cancelReply}>Cancel</button>
        </div>
      {/if}
      <textarea
        bind:value={newCommentBody}
        placeholder={replyingTo !== null ? 'Write a reply...' : 'Write a comment...'}
        rows="3"
        maxlength="2000"
      ></textarea>
      <div class="form-actions">
        <span class="char-count">{newCommentBody.length}/2000</span>
        <button class="btn" onclick={submitComment} disabled={submitting || !newCommentBody.trim()}>
          {#if submitting}Posting...{:else}{replyingTo !== null ? 'Reply' : 'Comment'}{/if}
        </button>
      </div>
    </div>
  {:else}
    <p class="muted login-hint"><a href="/">Login</a> to comment.</p>
  {/if}

  {#if error}
    <p class="error-text">⚠️ {error}</p>
  {/if}

  {#if loading && comments.length === 0}
    <p class="muted">Loading comments...</p>
  {:else if tree.length === 0}
    <p class="muted">No comments yet. Be the first!</p>
  {:else}
    <div class="comment-list">
      {#each tree as comment (comment.id)}
        {@render commentNode(comment, 0)}
      {/each}
    </div>

    {#if comments.length < totalTopLevel}
      <button class="btn btn-secondary load-more" onclick={() => loadComments(page + 1)}>
        Load more comments
      </button>
    {/if}
  {/if}
</div>

{#snippet commentNode(comment: ThreadedComment & { children?: ThreadedComment[]; can_edit?: boolean; updated_at?: string | null }, depth: number)}
  <div
    class="comment-node"
    class:deleted={false}
    class:hidden={false}
    style="margin-left: {Math.min(depth * 24, 120)}px"
  >
    <!-- COMMENT'S OWN CONTENT: this box carries the colored thread line.
         The border-left is only on this inner div, never on the container that holds children,
         so the line only spans this comment's own height — the old Lemmy trick. -->
    <div class="comment-content" style="border-left: 2px {lineColor(depth)} solid">
      <div class="comment-header">
        {#if (comment.children?.length ?? 0) > 0}
          <button class="collapse-btn" onclick={() => toggleCollapse(comment.id)}>
            {collapsed.has(comment.id) ? `[+${comment.children?.length ?? 0}]` : '[–]'}
          </button>
        {:else}
          <span class="collapse-spacer"></span>
        {/if}
        <span class="comment-user">{comment.username}</span>
        <span class="comment-date">{formatDate(comment.created_at)}</span>
        {#if comment.updated_at}
          <span class="comment-edited muted">· edited {formatDate(comment.updated_at)}</span>
        {/if}
        {#if comment.can_edit}
          <span class="comment-badge muted" title="Editable (within 24h)">✎</span>
        {/if}
      </div>

      <div class="comment-body">
        {#if editingComment === comment.id}
          <textarea
            class="edit-textarea"
            bind:value={editBody}
            rows="3"
            maxlength="2000"
          ></textarea>
          <p class="muted">Edit within 24 hours of posting.</p>
          <div class="comment-actions-row">
            <button class="btn-text" onclick={saveEdit} disabled={submitting}>Save</button>
            <button class="btn-text" onclick={cancelEdit}>Cancel</button>
          </div>
        {:else}
          {comment.body}
        {/if}
      </div>
      <ReactionBar targetType="comment" targetId={comment.id} />
      <div class="comment-actions-row">
        {#if auth.isLoggedIn && comment.user_id === auth.user?.id && comment.can_edit}
          <button class="btn-text" onclick={() => startEdit(comment)}>Edit</button>
          <button class="btn-text" onclick={() => deleteCommentHandler(comment)} disabled={deletingCommentId === comment.id}>
            {#if deletingCommentId === comment.id}Deleting…{:else}Delete{/if}
          </button>
        {/if}
        {#if auth.isLoggedIn}
          <button class="btn-text reply-btn" onclick={() => startReply(comment.id)}>Reply</button>
        {/if}
      </div>
    </div>

    {#if !collapsed.has(comment.id) && (comment.children?.length ?? 0) > 0}
      <div class="replies">
        {#each comment.children ?? [] as reply (reply.id)}
          {@render commentNode(reply, depth + 1)}
        {/each}
      </div>
    {/if}
  </div>
{/snippet}

<style>
  .comment-section {
    margin-top: 2rem;
    border-top: 1px solid var(--color-border);
    padding-top: 1rem;
  }
  .comment-section h3 {
    margin-bottom: 1rem;
  }
  .new-comment-form {
    margin-bottom: 1.5rem;
  }
  .new-comment-form textarea {
    width: 100%;
    resize: vertical;
    min-height: 60px;
    font-family: inherit;
  }
  .form-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 0.5rem;
  }
  .char-count {
    font-size: 0.78rem;
    color: var(--color-muted);
  }
  .replying-to {
    font-size: 0.85rem;
    color: var(--color-muted);
    margin-bottom: 0.5rem;
  }
  .btn-text {
    background: none;
    border: none;
    color: var(--color-primary);
    cursor: pointer;
    font-size: 0.82rem;
    padding: 0;
  }
  .btn-text:hover {
    text-decoration: underline;
  }
  .comment-list {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }
  .comment-node {
    padding: 0.5rem 0;
  }
  .comment-node.deleted {
    opacity: 0.5;
  }
  .comment-node.hidden {
    opacity: 0.4;
  }

  /* ── Thread line (old Lemmy approach) ── */
  .comment-content {
    padding: 0 0 0.3rem 0.6rem;
    /* border-left: 2px solid <color> set inline */
  }

  .comment-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.82rem;
  }
  .collapse-btn {
    background: none;
    border: none;
    color: var(--color-muted);
    cursor: pointer;
    font-family: var(--mono);
    font-size: 0.78rem;
    padding: 0;
    min-width: 3rem;
    text-align: left;
  }
  .collapse-btn:hover {
    color: var(--color-text);
  }
  .collapse-spacer {
    min-width: 3rem;
  }
  .comment-user {
    font-weight: 600;
    color: var(--color-text);
  }
  .comment-date {
    color: var(--color-muted);
  }
  .comment-edited {
    color: var(--color-muted);
    font-size: 0.78rem;
  }
  .comment-badge {
    font-size: 0.72rem;
    color: var(--color-accent, var(--color-primary));
  }
  .badge {
    font-size: 0.7rem;
    padding: 0.1rem 0.4rem;
    border-radius: 4px;
    font-weight: 600;
  }
  .deleted-badge {
    background: var(--color-surface-2);
    color: var(--color-muted);
  }
  .hidden-badge {
    background: #fee2e2;
    color: #991b1b;
  }
  .comment-body {
    font-size: 0.92rem;
    margin: 0.3rem 0 0.2rem 0;
    line-height: 1.5;
    white-space: pre-wrap;
  }
  .comment-actions-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.3rem;
  }
  .edit-textarea {
    width: 100%;
    resize: vertical;
    min-height: 60px;
    font-family: inherit;
    font-size: 0.92rem;
    margin-bottom: 0.3rem;
  }
  .reply-btn {
    margin-left: 0;
  }

  /* Children container — NO colored border, just a plain sibling */
  .replies {
    margin-left: 0;
  }

  .load-more {
    margin-top: 1rem;
    width: 100%;
  }
  .login-hint {
    margin-bottom: 1rem;
    font-size: 0.88rem;
  }
  .error-text {
    margin-bottom: 0.5rem;
  }

  /* Archive mode: AO3-style flat comment chrome */
  .comment-section.archive {
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .comment-section.archive .new-comment-form textarea,
  .comment-section.archive .new-comment-form input {
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
    font-family: inherit;
  }
  .comment-section.archive :global(.archive-btn),
  .comment-section.archive button {
    border-radius: 0;
    border: 1px solid var(--archive-border, #dddddd);
    background: linear-gradient(#f5f5f5, #e8e8e8);
    color: var(--archive-text, #2a2a2a);
    font-family: inherit;
  }
  .comment-section.archive .comment-user {
    color: var(--archive-link, #990000);
    font-weight: bold;
  }
  .comment-section.archive .comment-date,
  .comment-section.archive .comment-badge {
    color: var(--archive-muted, #666666);
  }
</style>
