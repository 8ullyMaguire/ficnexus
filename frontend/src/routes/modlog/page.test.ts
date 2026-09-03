// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Page from './+page.svelte';

// Hoisted state for the auth mock (vi.mock is hoisted above the test body).
const authState = vi.hoisted(() => ({ isLoggedIn: true }));

function mockAuthAndFetch(entries: unknown[]) {
  authState.isLoggedIn = true;
  vi.mock('$lib/stores/auth.svelte', () => ({
    auth: {
      isLoggedIn: authState.isLoggedIn,
      init: vi.fn(async () => {}),
    },
  }));
  globalThis.fetch = vi.fn().mockResolvedValue({
    ok: true,
    status: 200,
    json: async () => ({ entries }),
  });
  globalThis.localStorage = {
    getItem: () => 'token',
    setItem: vi.fn(),
    removeItem: vi.fn(),
    clear: vi.fn(),
    key: vi.fn(),
    length: 0,
  } as Storage;
}

beforeEach(() => {
  vi.resetModules();
  vi.restoreAllMocks();
});

describe('modlog page', () => {
  it('renders moderation entries with actor + action', async () => {
    mockAuthAndFetch([
      {
        id: 1,
        actor_id: 1,
        actor_username: 'admin1',
        action: 'ban_user',
        target_type: 'user',
        target_id: '42',
        details: {},
        created_at: '2026-08-11T10:00:00Z',
      },
      {
        id: 2,
        actor_id: 2,
        actor_username: 'curator2',
        action: 'merge_tags',
        target_type: 'tag',
        target_id: '7',
        details: { target_tag_id: 8 },
        created_at: '2026-08-11T11:00:00Z',
      },
    ]);

    render(Page);
    await new Promise((r) => setTimeout(r, 100));

    expect(screen.getByText(/Moderation Log/)).toBeTruthy();
    expect(screen.getByText('admin1')).toBeTruthy();
    expect(screen.getAllByText('Banned user').length).toBeGreaterThan(0);
    expect(screen.getByText('curator2')).toBeTruthy();
    expect(screen.getAllByText('Merged tags').length).toBeGreaterThan(0);
  });

  it('shows empty state', async () => {
    mockAuthAndFetch([]);
    render(Page);
    await new Promise((r) => setTimeout(r, 100));
    expect(screen.getByText(/No moderation actions recorded/)).toBeTruthy();
  });
});
