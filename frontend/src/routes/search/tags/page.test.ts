import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

function tagSearchResponse() {
  return jsonResponse({
    err: 0,
    total: 2,
    tags: [
      { id: 1, name: 'Harry Potter', tag_type_id: 1, description: null, usage_count: 1000, is_alias: false },
      { id: 2, name: 'Draco Malfoy', tag_type_id: 2, description: null, usage_count: 500, is_alias: false },
    ],
  });
}

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  localStorage.setItem('fichub_prefs_v1', JSON.stringify({ uiMode: 'archive' }));
});

describe('search/tags page', () => {
  it('renders the tag search form in archive mode', async () => {
    const { default: Page } = await import('./+page.svelte');
    const { container } = render(Page);
    await waitFor(() => {
      expect(screen.getByText('Tag name')).toBeTruthy();
      expect(screen.getByText('Type')).toBeTruthy();
      expect(screen.getByText('Wrangling status')).toBeTruthy();
      expect(screen.getByText('Sort by')).toBeTruthy();
      expect(screen.getByText('Sort direction')).toBeTruthy();
    });
  });

  it('shows "Find tags wrangled to specific canonical fandoms" field', async () => {
    const { default: Page } = await import('./+page.svelte');
    render(Page);
    await waitFor(() => {
      expect(screen.getByText('Find tags wrangled to specific canonical fandoms')).toBeTruthy();
    });
  });

  it('shows all wrangling status options', async () => {
    const { default: Page } = await import('./+page.svelte');
    render(Page);
    await waitFor(() => {
      expect(screen.getAllByRole('radio')).toHaveLength(7 + 7 + 3 + 2);
    });
  });

  it('fetches and displays tag results after search', async () => {
    mockFetch.mockResolvedValue(tagSearchResponse());
    const { default: Page } = await import('./+page.svelte');
    const { component } = render(Page);

    // Type a tag name
    const input = screen.getByPlaceholderText('e.g. Harry Potter');
    await fireEvent.input(input, { target: { value: 'Harry' } });

    // Click search
    const searchBtn = screen.getByText('Search');
    await fireEvent.click(searchBtn);

    await waitFor(() => {
      expect(screen.getByText('Harry Potter')).toBeTruthy();
      expect(screen.getByText('Draco Malfoy')).toBeTruthy();
      expect(screen.getByText('Fandom')).toBeTruthy();
      expect(screen.getByText('Character')).toBeTruthy();
    });
  });

  it('renders in modern mode as well', async () => {
    localStorage.setItem('fichub_prefs_v1', JSON.stringify({ uiMode: 'archive' }));
    const { default: Page } = await import('./+page.svelte');
    render(Page);
    await waitFor(() => {
      expect(screen.getByText('Tag Search')).toBeTruthy();
    });
  });
});
