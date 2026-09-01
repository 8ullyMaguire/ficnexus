import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';

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
      {
        id: 52,
        author_id: 2,
        author_username: 'mod',
        body: 'A reply with **markdown**',
        quote_of: 51,
        quote: { author_username: 'tester', preview: 'Hello world' },
        edited_at: null,
        deleted_at: null,
        created_at: new Date().toISOString(),
        score: 0,
        is_op: false,
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
  if (url.includes('/api/forum/topics/5')) {
    return Promise.resolve(jsonResponse(topicPayload()));
  }
  if (url.includes('/api/forum/topics')) {
    return Promise.resolve(jsonResponse({ err: 0, items: [], next_cursor: null, category: 'general', limit: 25 }));
  }
  if (url.includes('/api/forum/categories')) {
    return Promise.resolve(jsonResponse({ err: 0, items: [{ id: 1, slug: 'general', title: 'General', description: '', position: 0, is_mod_only: false, created_at: 'c', topic_count: 0, last_activity_at: null }] }));
  }
  return Promise.reject(new Error(`unexpected: ${url}`));
}

async function renderTopic(role = 0, me: Record<string, unknown> = {}) {
  mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
    const url = String(input);
    if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse({ ...mePayload(role), user: { ...mePayload(role).user, ...me } }));
    if (url.endsWith('/api/forum/moderation/status')) {
      return Promise.resolve(jsonResponse({ err: 0, points_left: 3, expires_at: null, eligible: true }));
    }
    if (url.endsWith('/api/forum/topics/5/follow')) {
      return Promise.resolve(jsonResponse({ err: 0, following: true, follower_count: 3 }));
    }
    if (url.endsWith('/api/forum/topics/5/read') && init?.method === 'POST') {
      return Promise.resolve(jsonResponse({ err: 0, last_read_post_id: 52, updated_at: '2026-08-14T10:00:00Z' }));
    }
    if (url.endsWith('/api/forum/topics/5/posts') && init?.method === 'POST') {
      return Promise.resolve(jsonResponse({ err: 0, id: 99 }));
    }
    if (url.endsWith('/api/forum/posts/52') && init?.method === 'PATCH') {
      return Promise.resolve(jsonResponse({ err: 0, id: 52 }));
    }
    if (url.endsWith('/api/forum/posts/52') && init?.method === 'DELETE') {
      return Promise.resolve(jsonResponse({ err: 0, id: 52 }));
    }
    if (url.endsWith('/api/forum/posts/51') && init?.method === 'PATCH') {
      return Promise.resolve(jsonResponse({ err: 0, id: 51 }));
    }
    if (url.endsWith('/api/forum/posts/51') && init?.method === 'DELETE') {
      return Promise.resolve(jsonResponse({ err: 0, id: 51 }));
    }
    if (url.endsWith('/api/forum/posts/51/moderate') && init?.method === 'POST') {
      return Promise.resolve(jsonResponse({ err: 0, delta: -2, score_after: -2, hidden_until: null }));
    }
    if (url.endsWith('/api/reports') && init?.method === 'POST') {
      return Promise.resolve(jsonResponse({ err: 0, report_id: 33 }));
    }
    return routerWith(input);
  });
  const { default: Page } = await import('./+page.svelte');
  return render(Page, { props: { data: { categorySlug: 'general', topicId: '5' } } });
}

describe('forum topic detail page', () => {
  it('renders the topic with OP and replies, markdown and quote preview', async () => {
    await renderTopic();
    await waitFor(() => {
      expect(screen.getByText('The great debate')).toBeTruthy();
    });
    // OP body rendered as markdown (bold).
    expect(document.querySelector('.post-body strong')).toBeTruthy();
    // Reply body with markdown.
    expect(document.querySelectorAll('.post-body strong').length).toBeGreaterThanOrEqual(2);
    // Quote preview block.
    expect(screen.getByText('tester wrote:')).toBeTruthy();
    expect(screen.getByText('Hello world')).toBeTruthy();
    // Meta: author + views.
    expect(screen.getByText(/42 views/)).toBeTruthy();
    // Reply composer present.
    expect(screen.getByRole('button', { name: 'Post reply' })).toBeTruthy();
  });

  it('shows edit affordances to the author within the 15-minute window', async () => {
    await renderTopic();
    await waitFor(() => {
      expect(screen.getByText('The great debate')).toBeTruthy();
    });
    // OP belongs to the logged-in user (id 1) and was created just now.
    expect(screen.getAllByRole('button', { name: 'Edit' }).length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByRole('button', { name: 'Delete' }).length).toBeGreaterThanOrEqual(1);
  });

  it('hides edit affordances from a non-author when the window has expired', async () => {
    const old = new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString();
    const payload = topicPayload();
    payload.created_at = old;
    payload.updated_at = old;
    (payload.items as Array<Record<string, unknown>>)[0].created_at = old;
    (payload.items as Array<Record<string, unknown>>)[1].created_at = old;
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload(0)));
      if (url.includes('/api/forum/topics/5')) return Promise.resolve(jsonResponse(payload));
      return routerWith(input);
    });
    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: { categorySlug: 'general', topicId: '5' } } });
    await waitFor(() => {
      expect(screen.getByText('The great debate')).toBeTruthy();
    });
    // No Edit buttons: the OP belongs to the current user but is outside the
    // 15-minute edit window, and the reply belongs to another user (author
    // id 2). Delete stays visible for the OP because authors can delete their
    // own posts any time (topic delete + OP post delete = 2 buttons).
    expect(screen.queryByRole('button', { name: 'Edit' })).toBeNull();
    expect(screen.getAllByRole('button', { name: 'Delete' })).toHaveLength(2);
  });

  it('shows edit/delete affordances to a moderator regardless of window', async () => {
    await renderTopic(5);
    await waitFor(() => {
      expect(screen.getByText('The great debate')).toBeTruthy();
    });
    expect(screen.getAllByRole('button', { name: 'Edit' }).length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByRole('button', { name: 'Delete' }).length).toBeGreaterThanOrEqual(1);
  });

  it('posts a reply and reloads the topic', async () => {
    await renderTopic();
    await waitFor(() => {
      expect(screen.getByText('The great debate')).toBeTruthy();
    });
    const textarea = screen.getByPlaceholderText(/write a reply/i);
    await fireEvent.input(textarea, { target: { value: 'My new reply' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Post reply' }));
    await waitFor(() => {
      expect(screen.getByText('Reply posted')).toBeTruthy();
    });
    const postCall = mockFetch.mock.calls.find((c) => String(c[0]).endsWith('/api/forum/topics/5/posts'));
    expect(postCall).toBeTruthy();
    expect(JSON.parse(String((postCall as unknown[])[1] && (postCall as any)[1].body))).toEqual({ body: 'My new reply' });
  });

  it('marks the topic as read when a logged-in user views it', async () => {
    await renderTopic();
    await waitFor(() => {
      expect(screen.getByText('The great debate')).toBeTruthy();
    });
    // Fire-and-forget mark-read lands after the follow-state GET; wait for it.
    const readCall = await vi.waitFor(() => {
      const call = mockFetch.mock.calls.find(
        (c) => String(c[0]).endsWith('/api/forum/topics/5/read') && (c[1] as RequestInit)?.method === 'POST',
      );
      expect(call).toBeTruthy();
      return call;
    });
        // The component posts the explicit last read post id returned by the
    // topic detail (POST .../read {last_read_post_id:N} — see api/forum.ts).
    expect(JSON.parse(String((readCall as any)[1].body))).toEqual({ last_read_post_id: 52 });
  });

  it('does not mark read when the viewer is logged out', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse({ err: 0, user: null }));
      return routerWith(input);
    });
    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: { categorySlug: 'general', topicId: '5' } } });
    await waitFor(() => {
      expect(screen.getByText('The great debate')).toBeTruthy();
    });
    const readCall = mockFetch.mock.calls.find(
      (c) => String(c[0]).endsWith('/api/forum/topics/5/read') && (c[1] as RequestInit)?.method === 'POST',
    );
    expect(readCall).toBeUndefined();
  });

  it('toggles follow and updates the follower count', async () => {
    await renderTopic();
    await waitFor(() => {
      expect(screen.getByText('The great debate')).toBeTruthy();
    });
    await fireEvent.click(screen.getByRole('button', { name: 'Follow' }));
    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Following' })).toBeTruthy();
    });
    expect(screen.getByText('3 followers')).toBeTruthy();
  });

  it('edits a post inline and reloads', async () => {
    await renderTopic();
    await waitFor(() => {
      expect(screen.getByText('The great debate')).toBeTruthy();
    });
    // The OP is editable (author, within window). The topic-head Edit button
    // comes first in the DOM; the OP post's Edit is index 1.
    const editButtons = screen.getAllByRole('button', { name: 'Edit' });
    await fireEvent.click(editButtons[1]);
    const textarea = await screen.findByDisplayValue('**Hello** world');
    await fireEvent.input(textarea, { target: { value: '**Edited** body' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => {
      expect(screen.queryByRole('button', { name: 'Save' })).toBeNull();
    });
    const patchCall = mockFetch.mock.calls.find(
      (c) => String(c[0]).includes('/api/forum/posts/51') && (c[1] as RequestInit)?.method === 'PATCH',
    );
    expect(patchCall).toBeTruthy();
    expect(JSON.parse(String((patchCall as any)[1].body))).toEqual({ body: '**Edited** body' });
  });

  // ── F5: score badges, collapsed posts, moderation, reporting ─────────

  it('shows a score badge on each post', async () => {
    await renderTopic();
    await waitFor(() => {
      expect(screen.getByText('The great debate')).toBeTruthy();
    });
    // Both seeded posts have score 0 -> archive shows 'Score 0' per post.
    expect(screen.getAllByText(/Score 0/)).toHaveLength(2);
  });

  it('collapses posts with score <= -2 and expands them on click', async () => {
    const payload = topicPayload();
    (payload.items as Array<Record<string, unknown>>)[1].score = -3;
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload(0)));
      if (url.endsWith('/api/forum/moderation/status')) {
        return Promise.resolve(jsonResponse({ err: 0, points_left: 0, expires_at: null, eligible: false }));
      }
      if (url.includes('/api/forum/topics/5')) return Promise.resolve(jsonResponse(payload));
      return routerWith(input);
    });
    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: { categorySlug: 'general', topicId: '5' } } });
    await waitFor(() => {
      expect(screen.getByText('The great debate')).toBeTruthy();
    });
    // The -3 post is collapsed: its body is hidden behind an indicator.
    const collapsedButton = await screen.findByRole('button', { name: /Post collapsed — score -3/ });
    expect(document.querySelector('.arc-btn')).toBeTruthy();
    // Only the OP body renders while the reply is collapsed.
    expect(document.querySelectorAll('.post-body')).toHaveLength(1);
    // Click expands and reveals the reply body (client-side only).
    await fireEvent.click(collapsedButton);
    await waitFor(() => {
      expect(document.querySelectorAll('.post-body')).toHaveLength(2);
    });
  });

  it('keeps posts with score > -2 expanded', async () => {
    const payload = topicPayload();
    (payload.items as Array<Record<string, unknown>>)[1].score = -1;
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload(0)));
      if (url.endsWith('/api/forum/moderation/status')) {
        return Promise.resolve(jsonResponse({ err: 0, points_left: 0, expires_at: null, eligible: false }));
      }
      if (url.includes('/api/forum/topics/5')) return Promise.resolve(jsonResponse(payload));
      return routerWith(input);
    });
    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: { categorySlug: 'general', topicId: '5' } } });
    await waitFor(() => {
      expect(screen.getByText('The great debate')).toBeTruthy();
    });
    // Both posts render their bodies; no collapsed indicator anywhere.
    expect(document.querySelectorAll('.post-body')).toHaveLength(2);
  });

  it('shows the mod button only to curators with points and applies a reason', async () => {
    await renderTopic(5);
    await waitFor(() => {
      expect(screen.getByText('The great debate')).toBeTruthy();
    });
    // Mod button renders once per post (mod status fetched after auth).
    const modButtons = await screen.findAllByRole('button', { name: 'Moderate' });
    expect(modButtons.length).toBeGreaterThanOrEqual(1);
    expect(modButtons[0]).toBeTruthy();

    // Open the picker on the first post and pick "Troll".
    await fireEvent.click(modButtons[0]);
    const trollButtons = await screen.findAllByRole('button', { name: /-2 · Troll$/ });
    expect(trollButtons.length).toBeGreaterThan(0);
    await fireEvent.click(trollButtons[0]);

    await waitFor(() => {
      expect(screen.getAllByText(/Score -2/).length).toBeGreaterThan(0);
    });
    const modCall = mockFetch.mock.calls.find(
      (c) => String(c[0]).endsWith('/api/forum/posts/51/moderate') && (c[1] as RequestInit)?.method === 'POST',
    );
    expect(modCall).toBeTruthy();
    expect(JSON.parse(String((modCall as any)[1].body))).toEqual({ reason: 'troll' });
    // The post's score badge reflects the new score (header + mod message).
    expect(screen.getAllByText(/Score -2/).length).toBeGreaterThan(0);
    // The mod button disappears for that post (already moderated by me).
    expect(screen.getAllByRole('button', { name: 'Moderate' })).toHaveLength(1);
  });

  it('disables the mod button after the API reports 409 (already moderated)', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload(5)));
      if (url.endsWith('/api/forum/moderation/status')) {
        return Promise.resolve(jsonResponse({ err: 0, points_left: 3, expires_at: null, eligible: true }));
      }
      if (url.endsWith('/api/forum/posts/51/moderate') && init?.method === 'POST') {
        return Promise.resolve(jsonResponse({ err: 409, msg: 'already moderated' }));
      }
      if (url.includes('/api/forum/topics/5')) return Promise.resolve(jsonResponse(topicPayload()));
      return routerWith(input);
    });
    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: { categorySlug: 'general', topicId: '5' } } });
    await waitFor(() => {
      expect(screen.getByText('The great debate')).toBeTruthy();
    });

    const modButtons = await screen.findAllByRole('button', { name: 'Moderate' });
    await fireEvent.click(modButtons[0]);
    const trollButtons = await screen.findAllByRole('button', { name: /-2 · Troll$/ });
    await fireEvent.click(trollButtons[0]);

    await waitFor(() => {
      expect(screen.getByText('Already moderated by you')).toBeTruthy();
    });
    expect(screen.getAllByRole('button', { name: 'Moderate' })).toHaveLength(1);
  });

  it('hides the mod button when the user has no moderation points', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload(5)));
      if (url.endsWith('/api/forum/moderation/status')) {
        return Promise.resolve(jsonResponse({ err: 0, points_left: 0, expires_at: null, eligible: false }));
      }
      if (url.includes('/api/forum/topics/5')) return Promise.resolve(jsonResponse(topicPayload()));
      return routerWith(input);
    });
    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: { categorySlug: 'general', topicId: '5' } } });
    await waitFor(() => {
      expect(screen.getByText('The great debate')).toBeTruthy();
    });
    expect(screen.queryByRole('button', { name: /Moderate/ })).toBeNull();
  });

  it('reports a post via /api/reports with forum_post target', async () => {
    await renderTopic();
    await waitFor(() => {
      expect(screen.getByText('The great debate')).toBeTruthy();
    });

    const reportButtons = screen.getAllByRole('button', { name: /Report/ });
    expect(reportButtons.length).toBeGreaterThanOrEqual(1);
    await fireEvent.click(reportButtons[0]);

    const textarea = await screen.findByPlaceholderText('Why are you reporting this post?');
    await fireEvent.input(textarea, { target: { value: 'Spam link' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Send report' }));

    await waitFor(() => {
      expect(screen.getByText('Report sent')).toBeTruthy();
    });
    const reportCall = mockFetch.mock.calls.find((c) => String(c[0]).endsWith('/api/reports'));
    expect(reportCall).toBeTruthy();
    expect(JSON.parse(String((reportCall as any)[1].body))).toEqual({
      target_type: 'forum_post',
      target_id: 51,
      reason: 'Spam link',
    });
  });
});
