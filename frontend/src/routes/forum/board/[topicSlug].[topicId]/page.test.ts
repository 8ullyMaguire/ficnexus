import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

import { auth } from '$lib/stores/auth.svelte';

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

beforeEach(() => {
  auth.initialized = false;
  auth.user = null;
  mockFetch.mockReset();
  localStorage.clear();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
});

function mePayload(role = 0) {
  return {
    err: 0,
    user: { id: 1, username: 'tester', role, reputation: 0, email: null, locale: 'en' },
  };
}

function topicPayload(overrides: Record<string, unknown> = {}) {
  return {
    err: 0,
    id: 5,
    title: 'The great debate',
    topic_slug: 'the-great-debate-5',
    author_id: 1,
    author_username: 'tester',
    category_slug: 'general',
    category_title: 'General',
    status: 'open',
    body: '**Hello** world',
    payload: null,
    view_count: 42,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    items: [
      {
        id: 51,
        author_id: 1,
        author_username: 'tester',
        body: '**Hello** world',
        quote_of: null,
        quote: null,
        edited_at: null,
        deleted_at: null,
        created_at: new Date().toISOString(),
        score: 0,
        is_op: true,
      },
    ],
    next_cursor: null,
    limit: 25,
    view_count_before: 41,
    ...overrides,
  };
}

function routerWith(input: RequestInfo | URL) {
  const url = String(input);
  if (url.endsWith('/api/auth/me')) {
    return Promise.resolve(jsonResponse(mePayload()));
  }
  if (url.includes('/api/forum/topics/by-slug/the-great-debate-5')) {
    return Promise.resolve(jsonResponse(topicPayload()));
  }
  if (url.includes('/api/forum/topics/5')) {
    return Promise.resolve(jsonResponse(topicPayload()));
  }
  if (url.endsWith('/api/forum/topics/5/read') || url.endsWith('/api/forum/topics/5/follow')) {
    return Promise.resolve(jsonResponse({ err: 0, following: false, follower_count: 0 }));
  }
  return Promise.reject(new Error(`unexpected: ${url}`));
}

async function renderBoard(topicSlug = 'the-great-debate-5', topicId = '5') {
  mockFetch.mockImplementation((input: RequestInfo | URL) => routerWith(input));
  const { default: Page } = await import('./+page.svelte');
  return render(Page, { props: { data: { topicSlug, topicId } } });
}

describe('forum board page (slug URL)', () => {
  it('resolves the topic by slug and renders the thread', async () => {
    await renderBoard();
    await waitFor(() => {
      expect(screen.getByText('The great debate')).toBeTruthy();
    });
    // The by-slug API was called (fresh slug link).
    const slugCall = mockFetch.mock.calls.find((c) =>
      String(c[0]).includes('/api/forum/topics/by-slug/the-great-debate-5'),
    );
    expect(slugCall).toBeTruthy();
    // The thread renders the OP markdown.
    expect(document.querySelector('.post-body strong')).toBeTruthy();
  });

  it('falls back to the numeric id when the slug is stale', async () => {
    // Server has no such slug → 400 "topic not found" → fall back to numeric.
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/topics/by-slug/stale-slug-999')) {
        return Promise.resolve(jsonResponse({ err: 400, msg: 'topic not found' }));
      }
      if (url.includes('/api/forum/topics/5')) return Promise.resolve(jsonResponse(topicPayload()));
      if (url.endsWith('/api/forum/topics/5/read') || url.endsWith('/api/forum/topics/5/follow')) {
        return Promise.resolve(jsonResponse({ err: 0, following: false, follower_count: 0 }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });
    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: { topicSlug: 'stale-slug-999', topicId: '5' } } });
    await waitFor(() => {
      expect(screen.getByText('The great debate')).toBeTruthy();
    });
    // Both calls happened: slug miss + numeric id hit.
    const slugCall = mockFetch.mock.calls.find((c) =>
      String(c[0]).includes('/api/forum/topics/by-slug/stale-slug-999'),
    );
    const idCall = mockFetch.mock.calls.find((c) => String(c[0]).includes('/api/forum/topics/5'));
    expect(slugCall).toBeTruthy();
    expect(idCall).toBeTruthy();
  });

  it('shows an error when both slug and id resolve to nothing', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/topics/by-slug/')) {
        return Promise.resolve(jsonResponse({ err: 400, msg: 'topic not found' }));
      }
      if (url.includes('/api/forum/topics/999')) {
        return Promise.resolve(jsonResponse({ err: 400, msg: 'topic not found' }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });
    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: { topicSlug: 'ghost-slug', topicId: '999' } } });
    await waitFor(() => {
      expect(screen.getByText(/topic not found/)).toBeTruthy();
    });
  });
});

describe('forum board page — emoji reactions', () => {
  function reactionsPayload() {
    return {
      err: 0,
      reactions: {
        '👍': [
          { user_id: 2, username: 'alice' },
          { user_id: 3, username: 'bob' },
        ],
        '🔥': [{ user_id: 4, username: 'cara' }],
      },
      my_reactions: ['👍'],
    };
  }

  it('shows existing reactions as chips and opens the anchored picker', async () => {
    auth.user = { id: 1, username: 'tester', role: 0, reputation: 0, email: null, locale: 'en' };
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/topics/by-slug/the-great-debate-5')) {
        return Promise.resolve(jsonResponse(topicPayload({
          items: [topicPayload().items[0], ...(topicPayload().items ?? [])].map((p) => ({
            ...p,
            reactions: reactionsPayload(),
          })),
        })));
      }
      if (url.includes('/api/forum/topics/5')) {
        return Promise.resolve(jsonResponse(topicPayload()));
      }
      if (url.endsWith('/api/forum/topics/5/read') || url.endsWith('/api/forum/topics/5/follow')) {
        return Promise.resolve(jsonResponse({ err: 0, following: false, follower_count: 0 }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: { topicSlug: 'the-great-debate-5', topicId: '5' } } });

    // Existing reaction chips render with counts.
    await waitFor(() => {
      expect(document.querySelector('.reaction-bar .reaction-btn')).toBeTruthy();
    });

    // Open the picker.
    const reactBtn = screen.getAllByText('React')[0];
    (reactBtn as HTMLElement).click();

    // Picker is anchored and shows search + category sections.
    await waitFor(() => {
      expect(document.querySelector('.picker')).toBeTruthy();
    });
    const picker = document.querySelector('.picker')!;
    expect(picker.querySelector('input[type="search"]')).toBeTruthy();
    expect(picker.textContent).toContain('Frequently used');
    expect(picker.textContent).toContain('Faces');

    // Search filters the sections.
    const input = picker.querySelector('input[type="search"]') as HTMLInputElement;
    input.value = 'applause';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    await waitFor(() => {
      // 🔥 (fire) disappears from the Gestures section when searching "applause".
      const rows = Array.from(picker.querySelectorAll('.picker-row'));
      expect(rows.some((r) => r.textContent?.includes('👏'))).toBe(true);
    });
  });

  it('posts a toggle to /reactions when picking an emoji', async () => {
    auth.user = { id: 1, username: 'tester', role: 0, reputation: 0, email: null, locale: 'en' };
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      const opts = (input as RequestInit) ?? {};
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/topics/by-slug/') || url.includes('/api/forum/topics/5')) {
        return Promise.resolve(jsonResponse(topicPayload()));
      }
      if (url.endsWith('/api/forum/topics/5/read') || url.endsWith('/api/forum/topics/5/follow')) {
        return Promise.resolve(jsonResponse({ err: 0, following: false, follower_count: 0 }));
      }
      if (url.endsWith('/api/forum/posts/51/reactions') && opts.method === 'POST') {
        return Promise.resolve(jsonResponse({ err: 0, added: true, reactions: { reactions: { '❤️': [{ user_id: 1, username: 'tester' }] }, my_reactions: ['❤️'] } }));
      }
      return Promise.reject(new Error(`unexpected: ${url} ${opts.method}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: { topicSlug: 'the-great-debate-5', topicId: '5' } } });

    await waitFor(() => {
      expect(document.querySelector('.reaction-bar')).toBeTruthy();
    });
    (screen.getAllByText('React')[0] as HTMLElement).click();
    await waitFor(() => {
      expect(document.querySelector('.picker')).toBeTruthy();
    });
    const heart = Array.from(document.querySelectorAll('.picker .picker-emoji'))
      .find((b) => b.textContent === '❤️') as HTMLButtonElement;
    expect(heart).toBeTruthy();
    heart.click();
    await waitFor(() => {
      const call = mockFetch.mock.calls.find(
        (c) => String(c[0]).endsWith('/api/forum/posts/51/reactions') && (c[1] as RequestInit)?.method === 'POST',
      );
      expect(call).toBeTruthy();
      expect(JSON.parse(String((call![1] as RequestInit).body))).toEqual({ emoji: '❤️' });
    });
  });
});
