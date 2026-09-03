import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';

// CommentSection: threaded comments with tree building, replies, collapse,
// and load-more. Auth-gated comment form when logged in.
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  // Default for any fetch not covered by mockResolvedValueOnce chains —
  // prevents "cannot read properties of undefined (reading 'status')"
  // unhandled rejections when the component issues an extra request.
  mockFetch.mockResolvedValue({ ok: true, status: 200, json: async () => ({ err: 0, comments: [], total_top_level: 0, page: 1 }) });
  localStorage.clear();
});

async function loadSection() {
  return await import('$lib/components/CommentSection.svelte');
}

function okJson(body: unknown) {
  return { ok: true, status: 200, json: async () => body };
}

const flatComments = [
  { id: 1, user_id: 1, username: 'alice', body: 'Great fic!', parent_id: null, created_at: '2026-01-01T10:00:00Z' },
  { id: 2, user_id: 2, username: 'bob', body: 'Agreed!', parent_id: 1, created_at: '2026-01-01T10:05:00Z' },
  { id: 3, user_id: 3, username: 'carol', body: 'Another top-level', parent_id: null, created_at: '2026-01-01T11:00:00Z' },
];

describe('CommentSection (logged out)', () => {
  it('renders the login hint instead of the form', async () => {
    const { default: Section } = await loadSection();
    render(Section, { props: { workId: 7 } });
    // The hint is split across an <a> + text node — match with a function.
    expect(
      screen.getAllByText((_, el) => el?.textContent?.includes('to comment') ?? false).length,
    ).toBeGreaterThan(0);
    expect(screen.queryByRole('button', { name: /comment/i })).toBeNull();
  });

  it('loads and renders the threaded tree (replies nested under parents)', async () => {
    mockFetch.mockResolvedValueOnce(
      okJson({ err: 0, comments: flatComments, total_top_level: 2, page: 1 }),
    );
    const { default: Section } = await loadSection();
    render(Section, { props: { workId: 7 } });

    await waitFor(() => {
      expect(screen.getByText('Great fic!')).toBeTruthy();
    });
    expect(screen.getByText('Agreed!')).toBeTruthy();
    expect(screen.getByText('Another top-level')).toBeTruthy();
    // Collapse button exists on the parent with children
    const collapseBtn = screen.getAllByRole('button').find((b) => b.textContent?.includes('[–]'));
    expect(collapseBtn).toBeTruthy();
  });

  it('collapses and expands a thread', async () => {
    mockFetch.mockResolvedValueOnce(
      okJson({ err: 0, comments: flatComments, total_top_level: 2, page: 1 }),
    );
    const { default: Section } = await loadSection();
    render(Section, { props: { workId: 7 } });

    await waitFor(() => {
      expect(screen.getByText('Agreed!')).toBeTruthy();
    });
    const collapseBtn = screen.getAllByRole('button').find((b) => b.textContent?.includes('[–]'))!;
    await fireEvent.click(collapseBtn);
    // Collapsed: reply hidden, button shows [+1]
    expect(screen.queryByText('Agreed!')).toBeNull();
    expect(screen.getByText('[+1]')).toBeTruthy();
    await fireEvent.click(screen.getByText('[+1]'));
    await waitFor(() => {
      expect(screen.getByText('Agreed!')).toBeTruthy();
    });
  });

  it('shows the empty state when there are no comments', async () => {
    mockFetch.mockResolvedValueOnce(okJson({ err: 0, comments: [], total_top_level: 0, page: 1 }));
    const { default: Section } = await loadSection();
    render(Section, { props: { workId: 7 } });
    await waitFor(() => {
      expect(screen.getByText(/no comments yet/i)).toBeTruthy();
    });
  });

  it('shows the error text when loading fails', async () => {
    mockFetch.mockResolvedValueOnce(okJson({ err: -1, msg: 'boom' }));
    const { default: Section } = await loadSection();
    render(Section, { props: { workId: 7 } });
    await waitFor(() => {
      expect(screen.getByText(/failed to load comments/i)).toBeTruthy();
    });
  });
});
