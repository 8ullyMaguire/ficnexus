<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { listReviews, postReview, deleteReview } from '$lib/api/social';
  import type { Review } from '$lib/api/social-types';
  import { relativeTime } from '$lib/util';
  import StarRating from '$lib/components/StarRating.svelte';

  let { workId }: { workId: number } = $props();

  let reviews = $state<Review[]>([]);
  let loading = $state(false);
  let error = $state('');

  // Review form state
  let formRating = $state<number>(0);
  let formTitle = $state('');
  let formBody = $state('');
  let submitting = $state(false);
  let formError = $state('');

  // Honeypot + timing trap: the backend silently drops comment/review
  // submissions that do not carry a `form_opened_at` timestamp set by the
  // frontend when the form mounts (bots script the API directly and skip it).
  let formOpenedAt = $state(0);
  let website = $state(''); // hidden CSS-invisible honeypot field

  onMount(() => {
    formOpenedAt = Date.now();
    loadReviews();
  });

  async function loadReviews() {
    loading = true;
    error = '';
    try {
      const res = await listReviews(workId);
      if (res.err !== 0) throw new Error('Failed to load reviews');
      reviews = res.reviews ?? [];
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load reviews';
    } finally {
      loading = false;
    }
  }

  async function submitReview() {
    if (!auth.isLoggedIn || submitting) return;
    if (formRating < 1) {
      formError = 'Please pick a star rating (1–5).';
      return;
    }
    if (!formBody.trim()) {
      formError = 'Please write a few words in the review body.';
      return;
    }
    submitting = true;
    formError = '';
    try {
      const res = await postReview(
        workId,
        formRating as 1 | 2 | 3 | 4 | 5,
        formTitle.trim() || undefined,
        formBody.trim(),
        { form_opened_at: formOpenedAt },
      );
      if (res.err !== 0) throw new Error('Failed to post review');
      formRating = 0;
      formTitle = '';
      formBody = '';
      await loadReviews();
    } catch (e) {
      formError = e instanceof Error ? e.message : 'Failed to post review';
    } finally {
      submitting = false;
    }
  }

  async function handleDelete(id: number) {
    if (!auth.isLoggedIn) return;
    try {
      const res = await deleteReview(id);
      if (res.err !== 0) throw new Error('Failed to delete review');
      await loadReviews();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to delete review';
    }
  }

  function formatDate(iso: string): string {
    const d = new Date(iso);
    if (isNaN(d.getTime())) return '';
    return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  }
</script>

<div class="reviews-section">
  <h3>Reviews ({reviews.length})</h3>

  {#if auth.isLoggedIn}
    <div class="review-form card">
      <div class="review-form-row">
        <span class="review-form-label">Your rating</span>
        <StarRating bind:value={formRating} onrate={(n) => (formRating = n)} />
        {#if formRating > 0}
          <span class="review-form-stars">{formRating}/5</span>
        {/if}
      </div>
      <input
        type="text"
        class="review-title-input"
        placeholder="Review title (optional)"
        maxlength="200"
        bind:value={formTitle}
      />
      <textarea
        class="review-body-input"
        placeholder="Write a short review… (what did you love? pacing, characters, prose)"
        rows="4"
        maxlength="8000"
        bind:value={formBody}
      ></textarea>
      <!-- Honeypot: CSS-hidden, humans never see it, bots fill it -->
      <input
        type="text"
        name="website"
        class="honeypot"
        tabindex="-1"
        autocomplete="off"
        aria-hidden="true"
        bind:value={website}
      />
      {#if formError}
        <p class="error-text">{formError}</p>
      {/if}
      <div class="form-actions">
        <span class="char-count">{formBody.length}/8000</span>
        <button
          class="btn"
          onclick={submitReview}
          disabled={submitting || !formBody.trim() || formRating < 1}
        >
          {#if submitting}Posting…{:else}Post Review{/if}
        </button>
      </div>
    </div>
  {:else}
    <p class="muted login-hint"><a href="/">Login</a> to write a review.</p>
  {/if}

  {#if error}
    <p class="error-text">⚠️ {error}</p>
  {/if}

  {#if loading && reviews.length === 0}
    <p class="muted">Loading reviews…</p>
  {:else if reviews.length === 0}
    <p class="muted">No reviews yet. Be the first to write one!</p>
  {:else}
    <div class="review-list">
      {#each reviews as r (r.id)}
        <div class="review-card">
          <div class="review-header">
            <span class="review-user">{r.username}</span>
            <StarRating value={r.rating} readonly />
            <span class="review-date">{formatDate(r.created_at)}</span>
            {#if r.updated_at && r.updated_at !== r.created_at}
              <span class="review-edited muted">(edited)</span>
            {/if}
            {#if auth.isLoggedIn && auth.username === r.username}
              <button class="btn-text delete-review-btn" onclick={() => handleDelete(r.id)}>Delete</button>
            {/if}
          </div>
          {#if r.title}
            <div class="review-title">{r.title}</div>
          {/if}
          <p class="review-body">{r.body}</p>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .reviews-section {
    margin-top: 1.5rem;
    border-top: 1px solid var(--color-border);
    padding-top: 1rem;
  }
  .reviews-section h3 {
    margin-bottom: 1rem;
  }
  .review-form {
    margin-bottom: 1.5rem;
  }
  .review-form-row {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    margin-bottom: 0.6rem;
  }
  .review-form-label {
    font-size: 0.85rem;
    font-weight: 600;
  }
  .review-form-stars {
    font-size: 0.85rem;
    color: var(--color-muted);
  }
  .review-title-input {
    width: 100%;
    margin-bottom: 0.5rem;
  }
  .review-body-input {
    width: 100%;
    resize: vertical;
    min-height: 80px;
    font-family: inherit;
  }
  .honeypot {
    position: absolute;
    left: -9999px;
    width: 1px;
    height: 1px;
    opacity: 0;
    pointer-events: none;
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
  .login-hint {
    margin-bottom: 1rem;
    font-size: 0.88rem;
  }
  .review-list {
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
  }
  .review-card {
    padding: 0.8rem;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-surface);
  }
  .review-header {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    font-size: 0.85rem;
    flex-wrap: wrap;
  }
  .review-user {
    font-weight: 600;
  }
  .review-date {
    color: var(--color-muted);
  }
  .review-edited {
    font-size: 0.75rem;
  }
  .delete-review-btn {
    margin-left: auto;
  }
  .review-title {
    font-weight: 600;
    margin-top: 0.4rem;
    font-size: 0.95rem;
  }
  .review-body {
    font-size: 0.9rem;
    line-height: 1.5;
    margin: 0.3rem 0 0 0;
    white-space: pre-wrap;
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
</style>
