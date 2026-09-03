// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Page from './+page.svelte';

const mocks = vi.hoisted(() => ({
  adminFetch: vi.fn(),
}));

vi.mock('$lib/api/admin', () => ({
  adminFetch: mocks.adminFetch,
}));

describe('admin translations page', () => {
  beforeEach(() => {
    mocks.adminFetch.mockReset();
  });

  it('renders draft translations with approve/reject/edit actions', async () => {
    mocks.adminFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        err: 0,
        items: [
          { id: 1, work_id: 42, locale_code: 'es', title: 'Título', summary: 'Resumen', translated_by: 3, translated_at: '2026-08-11T00:00:00Z', status: 'draft', reviewed_by: null, reviewed_at: null },
        ],
        total: 1,
        status: 'draft',
      }),
    });
    render(Page);
    await screen.findByText(/Título/);
    expect(screen.getAllByText(/es/).length).toBeGreaterThan(0);
    expect(screen.getByText(/work #42/)).toBeTruthy();
    // Buttons are plain text (emoji strip, 2026-08-22).
    expect(screen.getAllByText('Approve').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Reject').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Edit').length).toBeGreaterThan(0);
  });

  it('shows empty state when no translations', async () => {
    mocks.adminFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, items: [], total: 0, status: 'draft' }),
    });
    render(Page);
    await screen.findByText(/No translations in this state/);
    expect(screen.getAllByText(/No translations in this state/).length).toBeGreaterThan(0);
  });
});
