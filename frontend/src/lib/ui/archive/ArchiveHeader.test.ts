import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

// ArchiveHeader: unauthenticated users see both Login and Register links
// (AO3 parity). The Register dropdown form reuses the shared auth store and
// the site's registration_mode to decide whether to show the invite-code
// field or an application-mode hint.
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

import { auth } from '$lib/stores/auth.svelte';

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));

function okJson(body: unknown) {
  return { ok: true, json: async () => body };
}

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  auth.initialized = false;
  auth.user = null;
});

async function loadHeader() {
  return await import('$lib/ui/archive/ArchiveHeader.svelte');
}

describe('ArchiveHeader auth links', () => {
  it('shows a Register link alongside Log In when logged out (open mode)', async () => {
    mockFetch.mockResolvedValue(
      okJson({ err: 0, site: { name: 'FicNexus', description: '', registration_mode: 'open', version: 'x' } }),
    );
    const { default: Header } = await loadHeader();
    render(Header);

    await waitFor(() => {
      expect(screen.getAllByRole('button', { name: /^Log In$/i })).toBeTruthy();
      expect(screen.getAllByRole('button', { name: /^Register$/i }).length).toBeGreaterThan(0);
    });
  });

  it('shows the invite-code field in the Register dropdown when site is invite-only', async () => {
    mockFetch.mockResolvedValue(
      okJson({ err: 0, site: { name: 'FicNexus', description: '', registration_mode: 'invite', version: 'x' } }),
    );
    const { default: Header } = await loadHeader();
    render(Header);

    // Both the dropdown trigger and the (hidden) submit button are named
    // "Register", so grab the first match — the visible trigger.
    const regLink = (await screen.findAllByRole('button', { name: /^Register$/ }))[0];
    await fireEvent.click(regLink);

    await waitFor(() => {
      expect(screen.getByText(/Invitation only/i)).toBeTruthy();
      expect(screen.getByLabelText(/Invite code/)).toBeTruthy();
    });
  });

  it('shows an application-mode hint and no form when site is application-only', async () => {
    mockFetch.mockResolvedValue(
      okJson({ err: 0, site: { name: 'FicNexus', description: '', registration_mode: 'application', version: 'x' } }),
    );
    const { default: Header } = await loadHeader();
    render(Header);

    const regLink = await screen.findByRole('button', { name: /^Register$/ });
    await fireEvent.click(regLink);

    await waitFor(() => {
      expect(screen.getByText(/apply for an account/i)).toBeTruthy();
    });
    expect(screen.queryByLabelText('Username')).toBeNull();
  });

  it('registers a new user through the shared auth store on submit', async () => {
    mockFetch
      .mockResolvedValueOnce(
        okJson({ err: 0, site: { name: 'FicNexus', description: '', registration_mode: 'open', version: 'x' } }),
      )
      .mockResolvedValueOnce(
        okJson({ err: 0, token: 't-reg', user: { id: 7, username: 'newhead', role: 0, reputation: 0 } }),
      );

    const { default: Header } = await loadHeader();
    render(Header);

    // Both the dropdown trigger and the (hidden) submit button are named
    // "Register"; grab the first match — the visible trigger.
    const regLink = (await screen.findAllByRole('button', { name: /^Register$/ }))[0];
    await fireEvent.click(regLink);
    const regMenu = document.querySelector('#register')!;
    await fireEvent.input(regMenu.querySelector('#archive-reg-user')!, { target: { value: 'newhead' } });
    await fireEvent.input(regMenu.querySelector('#archive-reg-pass')!, { target: { value: 'sekret' } });
    await fireEvent.input(regMenu.querySelector('#archive-reg-email')!, { target: { value: 'new@head.test' } });

    // The dropdown trigger and the form submit button are both named "Register";
    // pick the submit button (inside the register menu form).
    const registerBtn = regMenu.querySelector('button[type="submit"]') as HTMLButtonElement;
    await fireEvent.click(registerBtn);

    await waitFor(() => {
      const regCall = mockFetch.mock.calls.find((c) => String(c[0]).includes('/auth/register'));
      expect(regCall).toBeTruthy();
      const body = JSON.parse(regCall![1].body);
      expect(body.username).toBe('newhead');
      expect(body.invite_code).toBeUndefined();
    });
    await waitFor(() => {
      expect(auth.user?.username).toBe('newhead');
    });
  });
});
