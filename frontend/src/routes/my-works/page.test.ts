import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

vi.mock('$lib/stores/auth.svelte', () => ({
  auth: {
    init: vi.fn().mockResolvedValue(undefined),
    isLoggedIn: true,
    user: { id: 1, username: 'alice', role: 0, reputation: 0, level: 12, exp: 4800, email: null, locale: 'en' },
  },
}));

vi.mock('$lib/prefs', () => ({
  getPref: () => 'archive',
}));

vi.mock('$lib/i18n/index.svelte', () => ({ t: (s: string) => s }));
vi.mock('$lib/ui/archive/WorkBlurb.svelte', () => ({ default: {} }));
vi.mock('$lib/ui/archive/ArchiveButton.svelte', () => ({ default: {} }));

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

beforeEach(() => {
  mockFetch.mockReset();
});

describe('my-works page', () => {
  it('renders My Works heading', async () => {
    const { default: Page } = await import('./+page.svelte');
    render(Page);
    expect(screen.getByText('My Works')).toBeTruthy();
  });
});
