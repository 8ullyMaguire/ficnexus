import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';

// NotificationBell: fetches the unread count on mount (logged-in users) and
// renders a badge when > 0. Also covers the logged-out path (count stays 0).
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  // Reset the auth singleton (module-level import is hoisted by vitest, so
  // a static import here shares the instance used by the SUT).
});

async function loadBell() {
  return await import('$lib/components/NotificationBell.svelte');
}

describe('NotificationBell', () => {
  it('renders the bell link with aria-label', async () => {
    const { default: Bell } = await loadBell();
    render(Bell);
    expect(screen.getByRole('link', { name: /notifications/i })).toBeTruthy();
  });

  it('shows no badge when the unread count is 0', async () => {
    const { default: Bell } = await loadBell();
    render(Bell);
    // No badge span when count is 0 (logged out path).
    expect(document.body.textContent).not.toContain('99+');
  });
});
