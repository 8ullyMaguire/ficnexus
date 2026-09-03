import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

vi.mock('$lib/stores/auth.svelte', () => ({
  auth: {
    init: vi.fn().mockResolvedValue(undefined),
    isLoggedIn: true,
    level: 1,
    exp: 0,
    user: { id: 1, username: 'applicant', role: 0, reputation: 0, level: 1, email: null, locale: 'en' },
  },
}));

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
});

describe('forum registration application page', () => {
  it('submits the application with a reason and shows the pending state', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith('/api/registration-applications') && init?.method === 'POST') {
        return Promise.resolve(jsonResponse({ err: 0, id: 5, status: 'pending' }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Submit application' })).toBeTruthy();
    });
    await fireEvent.input(screen.getByPlaceholderText(/who you are and why you want to join/), { target: { value: 'I love archives and want to contribute.' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Submit application' }));

    await waitFor(() => {
      expect(screen.getByText(/application was submitted/)).toBeTruthy();
    });
    const postCall = mockFetch.mock.calls.find((c) => String(c[0]).endsWith('/api/registration-applications') && c[1]?.method === 'POST');
    expect(postCall).toBeTruthy();
    expect(JSON.parse(String(postCall![1].body)).reason).toBe('I love archives and want to contribute.');
  });

  it('treats a 409 duplicate as already submitted', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith('/api/registration-applications') && init?.method === 'POST') {
        return Promise.resolve(jsonResponse({ err: 409, msg: 'already pending' }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Submit application' })).toBeTruthy();
    });
    await fireEvent.input(screen.getByPlaceholderText(/who you are and why you want to join/), { target: { value: 'again' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Submit application' }));

    await waitFor(() => {
      expect(screen.getByText(/application was submitted/)).toBeTruthy();
    });
  });

  it('shows an error when submission fails', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith('/api/registration-applications') && init?.method === 'POST') {
        return Promise.resolve(jsonResponse({ err: -1 }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Submit application' })).toBeTruthy();
    });
    await fireEvent.input(screen.getByPlaceholderText(/who you are and why you want to join/), { target: { value: 'please' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Submit application' }));

    await waitFor(() => {
      expect(screen.getByText(/Could not send the application/)).toBeTruthy();
    });
  });
});
