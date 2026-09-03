import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

vi.mock('$lib/stores/auth.svelte', () => ({
  auth: {
    init: vi.fn().mockResolvedValue(undefined),
    isLoggedIn: true,
    username: 'tester',
    user: { id: 1, username: 'tester', role: 0, level: 1 },
    level: 1,
    initialized: true,
  },
}));

vi.mock('$lib/api/social', async (importOriginal) => {
  const actual = (await importOriginal()) as Record<string, unknown>;
  return {
    ...actual,
    authHeaders: () => ({ Authorization: 'Bearer test-token' }),
  };
});

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  localStorage.setItem('fichub_prefs_v1', JSON.stringify({ uiMode: 'archive' }));
});

describe('upload page', () => {
  it('renders heading Import Work', async () => {
    const { default: Page } = await import('./+page.svelte');
    render(Page);
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /Import Work/i })).toBeTruthy();
    });
  });

  it('renders required form fields when logged in', async () => {
    const { default: Page } = await import('./+page.svelte');
    render(Page);
    await waitFor(() => {
      expect(screen.getByLabelText(/Title/i)).toBeTruthy();
    });
    expect(screen.getByLabelText(/Author/i)).toBeTruthy();
    expect(screen.getByText(/Accepted:/)).toBeTruthy();
    // status select
    expect(screen.getByLabelText(/Status/i)).toBeTruthy();
    // submit button
    expect(screen.getByRole('button', { name: /Import Work/i })).toBeTruthy();
  });

  it('shows file input with correct accept types', async () => {
    const { default: Page } = await import('./+page.svelte');
    render(Page);
    await waitFor(() => {
      const input = document.querySelector('input[type="file"]') as HTMLInputElement | null;
      expect(input).toBeTruthy();
      expect(input!.accept).toContain('.txt');
      expect(input!.accept).toContain('.epub');
      expect(input!.accept).toContain('.html');
      expect(input!.accept).toContain('.md');
    });
  });
});
