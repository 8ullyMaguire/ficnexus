import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';

// Logged-in CommentSection flows: posting a comment and replying.
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

// Seed a logged-in auth store before the component mounts.
import { auth } from '$lib/stores/auth.svelte';

beforeEach(() => {
  mockFetch.mockReset();
  // Default for any fetch not covered by mockResolvedValueOnce chains —
  // prevents unhandled rejections from undefined responses.
  mockFetch.mockResolvedValue({ ok: true, status: 200, json: async () => ({ err: 0, comments: [], total_top_level: 0, page: 1 }) });
  localStorage.clear();
  auth.initialized = true;
  auth.user = { id: 1, username: 'alice', role: 0, reputation: 0 };
});

async function loadSection() {
  return await import('$lib/components/CommentSection.svelte');
}

function okJson(body: unknown) {
  return { ok: true, status: 200, json: async () => body };
}

describe('CommentSection (logged in)', () => {
  it('renders the comment form with char count', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, comments: [], total_top_level: 0, page: 1 }));
    const { default: Section } = await loadSection();
    render(Section, { props: { workId: 7 } });
    await waitFor(() => {
      expect(screen.getByPlaceholderText(/write a comment/i)).toBeTruthy();
    });
    expect(screen.getByText('0/2000')).toBeTruthy();
  });

  it('posts a comment and reloads the list', async () => {
    mockFetch
      .mockResolvedValueOnce(okJson({ err: 0, comments: [], total_top_level: 0, page: 1 }))
      .mockResolvedValueOnce(
        okJson({ err: 0, comment_id: 42, created_at: 'c', username: 'alice' }),
      )
      .mockResolvedValueOnce(
        okJson({
          err: 0,
          comments: [{ id: 42, user_id: 1, username: 'alice', body: 'My new comment', parent_id: null, created_at: '2026-01-01T00:00:00Z' }],
          total_top_level: 1,
          page: 1,
        }),
      );

    const { default: Section } = await loadSection();
    render(Section, { props: { workId: 7 } });

    const textarea = await screen.findByPlaceholderText(/write a comment/i);
    await fireEvent.input(textarea, { target: { value: 'My new comment' } });
    await fireEvent.click(screen.getByRole('button', { name: /comment/i }));

    await waitFor(() => {
      expect(screen.getByText('My new comment')).toBeTruthy();
    });
    const postCall = mockFetch.mock.calls.find((c) => String(c[0]).includes('/api/works/7/comments') && c[1]?.method === 'POST');
    expect(postCall).toBeTruthy();
    expect(JSON.parse(postCall![1].body).body).toBe('My new comment');
  });

  it('starts a reply and cancels it', async () => {
    mockFetch.mockResolvedValue(
      okJson({
        err: 0,
        comments: [{ id: 1, user_id: 2, username: 'bob', body: 'A comment', parent_id: null, created_at: '2026-01-01T00:00:00Z' }],
        total_top_level: 1,
        page: 1,
      }),
    );
    const { default: Section } = await loadSection();
    render(Section, { props: { workId: 7 } });

    await waitFor(() => {
      expect(screen.getByText('A comment')).toBeTruthy();
    });
    await fireEvent.click(screen.getByRole('button', { name: /reply/i }));
    expect(screen.getByText(/replying to comment/i)).toBeTruthy();
    expect(screen.getByPlaceholderText(/write a reply/i)).toBeTruthy();
    await fireEvent.click(screen.getByRole('button', { name: /cancel/i }));
    expect(screen.queryByText(/replying to comment/i)).toBeNull();
  });
});
