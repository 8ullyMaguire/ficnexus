import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
});

function categoriesPayload() {
  return {
    err: 0,
    items: [
      { id: 1, slug: 'general', title: 'General', description: '', position: 0, is_mod_only: false, created_at: 'c', topic_count: 0, last_activity_at: null },
      { id: 2, slug: 'fanfic-talk', title: 'Fanfic Talk', description: '', position: 1, is_mod_only: false, created_at: 'c', topic_count: 0, last_activity_at: null },
    ],
  };
}

describe('forum new-topic composer', () => {
  it('loads categories, selects the ?category= preset, and posts a topic', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.includes('/api/forum/categories')) {
        return Promise.resolve(jsonResponse(categoriesPayload()));
      }
      if (url === '/api/forum/topics' && init?.method === 'POST') {
        return Promise.resolve(jsonResponse({ err: 0, id: 77 }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('Start a new topic')).toBeTruthy();
    });
    // First category is selected by default.
    const select = screen.getByRole('combobox') as HTMLSelectElement;
    await waitFor(() => {
      expect(select.value).toBe('general');
    });

    const titleInput = screen.getByPlaceholderText(/what are you reading/i);
    await fireEvent.input(titleInput, { target: { value: 'Weekly check-in' } });
    const bodyTextarea = screen.getByPlaceholderText(/write your post/i);
    await fireEvent.input(bodyTextarea, { target: { value: 'What is everyone reading?' } });

    await fireEvent.click(screen.getByRole('button', { name: 'Post topic' }));
    await waitFor(() => {
      expect(screen.getByText('Topic posted')).toBeTruthy();
    });
    const postCall = mockFetch.mock.calls.find((c) => String(c[0]) === '/api/forum/topics');
    expect(postCall).toBeTruthy();
    expect(JSON.parse(String((postCall as any)[1].body))).toEqual({
      title: 'Weekly check-in',
      category_slug: 'general',
      body: 'What is everyone reading?',
    });
  });

  it('disables submit until title, category and body are filled', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      if (String(input).includes('/api/forum/categories')) {
        return Promise.resolve(jsonResponse(categoriesPayload()));
      }
      return Promise.reject(new Error(`unexpected: ${input}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      const btn = screen.getByRole('button', { name: 'Post topic' }) as HTMLButtonElement;
      expect(btn.disabled).toBe(true);
    });
    const titleInput = screen.getByPlaceholderText(/what are you reading/i);
    await fireEvent.input(titleInput, { target: { value: 'A title' } });
    const bodyTextarea = screen.getByPlaceholderText(/write your post/i);
    await fireEvent.input(bodyTextarea, { target: { value: 'A body' } });
    const btn = screen.getByRole('button', { name: 'Post topic' }) as HTMLButtonElement;
    expect(btn.disabled).toBe(false);
  });
});
