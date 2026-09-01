import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

// Mock the auth store so tests can flip login state without network calls.
let _user: any = null;
vi.mock('$lib/stores/auth.svelte', () => ({
  auth: {
    get user() { return _user; },
    set user(v: any) { _user = v; },
    get isLoggedIn() { return _user !== null; },
    get username() { return _user?.username ?? null; },
    loading: false,
    initialized: true,
    init: vi.fn().mockResolvedValue(undefined),
  },
}));

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  _user = null;
});

async function loadReviewsSection() {
  return await import('./ReviewsSection.svelte');
}

function reviewListPayload(reviews: unknown[]) {
  return jsonResponse({ err: 0, work_id: 7, total: reviews.length, reviews });
}

describe('ReviewsSection review form', () => {
  it('posts the review payload including the form_opened_at honeypot field', async () => {
    _user = { id: 1, username: 'alice', role: 0, reputation: 0 };

    let reviewsFetched = false;
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/auth/me')) {
        return Promise.resolve(jsonResponse({ err: 0, user: { id: 1, username: 'alice', role: 0, reputation: 0 } }));
      }
      if (url.endsWith('/works/7/reviews') && !reviewsFetched) {
        reviewsFetched = true;
        return Promise.resolve(reviewListPayload([]));
      }
      if (url.endsWith('/reviews') || url.endsWith('/works/7/reviews')) {
        return Promise.resolve(reviewListPayload([
          { id: 42, work_id: 7, username: 'alice', rating: 5, title: 'Loved it', body: 'Great pacing', constructive: true, created_at: '2024-01-01T00:00:00Z', updated_at: '2024-01-01T00:00:00Z' },
        ]));
      }
      return Promise.reject(new Error(`unexpected fetch: ${url}`));
    });

    const { default: ReviewsSection } = await loadReviewsSection();
    render(ReviewsSection, { props: { workId: 7 } });

    // Wait for the initial list load to settle
    await waitFor(() => expect(screen.getByText(/no reviews yet/i)).toBeTruthy());

    // Pick a star rating
    await fireEvent.click(screen.getByRole('button', { name: 'Rate 5 stars' }));
    await fireEvent.input(screen.getByPlaceholderText(/review title/i), { target: { value: 'Loved it' } });
    await fireEvent.input(screen.getByPlaceholderText(/write a short review/i), { target: { value: 'Great pacing and characters.' } });

    await fireEvent.click(screen.getByRole('button', { name: 'Post Review' }));

    await waitFor(() => {
      const call = mockFetch.mock.calls.find((c) => String(c[0]).endsWith('/api/reviews'));
      expect(call).toBeTruthy();
      const body = JSON.parse(call![1].body);
      expect(body.work_id).toBe(7);
      expect(body.rating).toBe(5);
      expect(body.title).toBe('Loved it');
      expect(body.body).toBe('Great pacing and characters.');
      // Honeypot/timing field MUST be sent — the backend silently drops
      // submissions without it.
      expect(typeof body.form_opened_at).toBe('number');
      expect(body.form_opened_at).toBeGreaterThan(0);
    });

    // After posting, the list re-fetches and shows the new review
    await waitFor(() => expect(screen.getByText('Great pacing')).toBeTruthy());
  });

  it('requires a star rating before posting', async () => {
    _user = { id: 1, username: 'alice', role: 0, reputation: 0 };
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/auth/me')) {
        return Promise.resolve(jsonResponse({ err: 0, user: { id: 1, username: 'alice', role: 0, reputation: 0 } }));
      }
      if (url.endsWith('/works/7/reviews')) {
        return Promise.resolve(reviewListPayload([]));
      }
      return Promise.reject(new Error(`unexpected fetch: ${url}`));
    });

    const { default: ReviewsSection } = await loadReviewsSection();
    render(ReviewsSection, { props: { workId: 7 } });

    await waitFor(() => expect(screen.getByText(/no reviews yet/i)).toBeTruthy());

    await fireEvent.input(screen.getByPlaceholderText(/write a short review/i), { target: { value: 'A review without stars' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Post Review' }));

    await waitFor(() => expect(screen.getByText(/please pick a star rating/i)).toBeTruthy());
    // No POST should have happened
    expect(mockFetch.mock.calls.some((c) => String(c[0]).endsWith('/api/reviews'))).toBe(false);
  });
});

describe('ReviewsSection review list', () => {
  it('renders reviews with username, stars, title, body and date', async () => {
    mockFetch.mockResolvedValue(reviewListPayload([
      { id: 1, work_id: 7, username: 'bob', rating: 4, title: 'Strong start', body: 'Loved the first arc.', constructive: true, created_at: '2024-03-01T00:00:00Z', updated_at: '2024-03-01T00:00:00Z' },
      { id: 2, work_id: 7, username: 'carol', rating: 5, title: null, body: 'A masterpiece.', constructive: true, created_at: '2024-02-01T00:00:00Z', updated_at: '2024-02-01T00:00:00Z' },
    ]));

    const { default: ReviewsSection } = await loadReviewsSection();
    render(ReviewsSection, { props: { workId: 7 } });

    await waitFor(() => expect(screen.getByText('bob')).toBeTruthy());
    expect(screen.getByText('Strong start')).toBeTruthy();
    expect(screen.getByText('Loved the first arc.')).toBeTruthy();
    expect(screen.getByText('carol')).toBeTruthy();
    expect(screen.getByText('A masterpiece.')).toBeTruthy();
    // bob's 4-star review renders exactly 4 filled stars (readonly display)
    const bobCard = screen.getByText('bob').closest('.review-card');
    expect(bobCard).toBeTruthy();
    const bobStars = Array.from(bobCard!.querySelectorAll('button')).filter(
      (s) => s.getAttribute('aria-pressed') === 'true',
    );
    expect(bobStars).toHaveLength(4);
    // carol's 5-star review renders 5 filled stars
    const carolCard = screen.getByText('carol').closest('.review-card');
    expect(carolCard).toBeTruthy();
    const carolStars = Array.from(carolCard!.querySelectorAll('button')).filter(
      (s) => s.getAttribute('aria-pressed') === 'true',
    );
    expect(carolStars).toHaveLength(5);
    expect(screen.getByText(/Reviews \(2\)/)).toBeTruthy();
  });

  it('shows the delete button for the reviewer\'s own review and deletes it', async () => {
    _user = { id: 1, username: 'bob', role: 0, reputation: 0 };

    let reviews = [
      { id: 1, work_id: 7, username: 'bob', rating: 4, title: 'Strong start', body: 'Loved the first arc.', constructive: true, created_at: '2024-03-01T00:00:00Z', updated_at: '2024-03-01T00:00:00Z' },
    ];
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/auth/me')) {
        return Promise.resolve(jsonResponse({ err: 0, user: { id: 1, username: 'bob', role: 0, reputation: 0 } }));
      }
      if (url.includes('/works/7/reviews')) {
        return Promise.resolve(reviewListPayload(reviews));
      }
      if (url.includes('/reviews/1') && String(input).startsWith('/api/reviews/')) {
        reviews = [];
        return Promise.resolve(jsonResponse({ err: 0, msg: 'Review deleted' }));
      }
      return Promise.reject(new Error(`unexpected fetch: ${url}`));
    });

    const { default: ReviewsSection } = await loadReviewsSection();
    render(ReviewsSection, { props: { workId: 7 } });

    await waitFor(() => expect(screen.getByText('Strong start')).toBeTruthy());

    const deleteBtn = screen.getByRole('button', { name: 'Delete' });
    expect(deleteBtn).toBeTruthy();
    await fireEvent.click(deleteBtn);

    // After delete, the list re-fetches and the review is gone
    await waitFor(() => expect(screen.getByText(/no reviews yet/i)).toBeTruthy());
    const deleteCall = mockFetch.mock.calls.find((c) => String(c[0]).includes('/reviews/1') && c[1]?.method === 'DELETE');
    expect(deleteCall).toBeTruthy();
  });

  it('does not show a delete button for someone else\'s review', async () => {
    _user = { id: 2, username: 'carol', role: 0, reputation: 0 };
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/auth/me')) {
        return Promise.resolve(jsonResponse({ err: 0, user: { id: 2, username: 'carol', role: 0, reputation: 0 } }));
      }
      if (url.endsWith('/works/7/reviews')) {
        return Promise.resolve(reviewListPayload([
          { id: 1, work_id: 7, username: 'bob', rating: 4, title: 'Strong start', body: 'Loved the first arc.', constructive: true, created_at: '2024-03-01T00:00:00Z', updated_at: '2024-03-01T00:00:00Z' },
        ]));
      }
      return Promise.reject(new Error(`unexpected fetch: ${url}`));
    });

    const { default: ReviewsSection } = await loadReviewsSection();
    render(ReviewsSection, { props: { workId: 7 } });

    await waitFor(() => expect(screen.getByText('Strong start')).toBeTruthy());
    expect(screen.queryByRole('button', { name: 'Delete' })).toBeNull();
  });
});
