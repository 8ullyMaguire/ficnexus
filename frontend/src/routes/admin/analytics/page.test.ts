// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Page from './+page.svelte';

function mockAdminFetch(body: unknown) {
  globalThis.fetch = vi.fn().mockResolvedValue({
    ok: true,
    status: 200,
    json: async () => body,
  });
}

beforeEach(() => {
  vi.resetModules();
  vi.restoreAllMocks();
});

describe('admin analytics page', () => {
  it('renders engagement cards with active vs view-only users', async () => {
    mockAdminFetch({
      unique_visitors: {
        daily: [{ date: '2026-08-10', visitors: 42 }],
        weekly: [{ week_start: '2026-08-10', visitors: 120 }],
        monthly: [{ month: '2026-08', visitors: 300 }],
      },
      engagement: {
        active_users: { '1d': 5, '7d': 20, '30d': 60 },
        view_only_users: { '1d': 10, '7d': 40, '30d': 100 },
        action_events: { '7d': 55, '30d': 180 },
        total_events: { '7d': 500, '30d': 2000 },
      },
      recent_events: [
        { at: '2026-08-11T10:00:00Z', client_id: 'abc12345', path: '/api/epub?q=x', type: 'action', user_agent: 'test' },
      ],
    });

    render(Page);
    // Wait for the async load
    await new Promise((r) => setTimeout(r, 50));

    expect(screen.getByText(/Usage Analytics/)).toBeTruthy();
    expect(screen.getByText(/Active users \(30d\)/)).toBeTruthy();
    expect(screen.getAllByText('60').length).toBeGreaterThan(0);
    expect(screen.getByText(/View-only \(30d\)/)).toBeTruthy();
    expect(screen.getAllByText('100').length).toBeGreaterThan(0);
    // Recent timeline shows the action event
    expect(screen.getAllByText('action').length).toBeGreaterThan(0);
  });

  it('shows empty state when no events yet', async () => {
    mockAdminFetch({
      unique_visitors: { daily: [], weekly: [], monthly: [] },
      engagement: {
        active_users: { '1d': 0, '7d': 0, '30d': 0 },
        view_only_users: { '1d': 0, '7d': 0, '30d': 0 },
        action_events: { '7d': 0, '30d': 0 },
        total_events: { '7d': 0, '30d': 0 },
      },
      recent_events: [],
    });

    render(Page);
    await new Promise((r) => setTimeout(r, 50));

    expect(screen.getByText(/No usage events yet/)).toBeTruthy();
    expect(screen.getByText(/No events yet/)).toBeTruthy();
  });

  it('shows error on fetch failure', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({ ok: false, status: 500 });
    render(Page);
    await new Promise((r) => setTimeout(r, 50));
    expect(screen.getByText(/Failed to load analytics/)).toBeTruthy();
  });
});
