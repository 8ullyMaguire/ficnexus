import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

// AuthBar flows against the REAL auth store (Svelte 5 $state → reactive
// header): failed login shows the error, successful login flips the header
// to the user badge, register works, and logout returns to the auth buttons.
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

import { auth } from '$lib/stores/auth.svelte';

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  auth.initialized = false;
  auth.user = null;
});

vi.mock('$app/navigation', () => ({
  goto: vi.fn(),
}));

async function loadAuthBar() {
  return await import('$lib/components/AuthBar.svelte');
}

function okJson(body: unknown) {
  return { ok: true, json: async () => body };
}

async function openModal(kind: 'login' | 'register') {
  const { default: AuthBar } = await loadAuthBar();
  render(AuthBar);
  const label = kind === 'login' ? 'Login' : 'Register';
  const btn = screen.getAllByRole('button', { name: label })[0];
  await fireEvent.click(btn);
  return AuthBar;
}

function submitDialog() {
  const dialog = screen.getByRole('dialog');
  // The modal has Cancel (.btn-secondary) + submit (.btn): pick the LAST
  // button in the actions row (the submit), not the first .btn match.
  const actions = dialog.querySelector('.modal-actions');
  const buttons = (actions ?? dialog).querySelectorAll('button');
  return buttons[buttons.length - 1] as HTMLButtonElement;
}

describe('AuthBar flows (real store)', () => {
  it('shows the error message on failed login', async () => {
    mockFetch.mockResolvedValue(okJson({ err: -1, msg: 'bad credentials' }));
    await openModal('login');
    const username = screen.getByLabelText('Username') as HTMLInputElement;
    const password = screen.getByLabelText('Password') as HTMLInputElement;
    await fireEvent.input(username, { target: { value: 'alice' } });
    await fireEvent.input(password, { target: { value: 'wrong' } });
    await fireEvent.click(submitDialog());

    await waitFor(() => {
      expect(screen.getByText('Invalid username or password')).toBeTruthy();
    });
    expect(screen.getByRole('dialog')).toBeTruthy();
  });

  it('submits a successful login and closes the modal', async () => {
    mockFetch.mockResolvedValue(
      okJson({ err: 0, token: 't1', user: { id: 1, username: 'alice', role: 0, reputation: 0 } }),
    );
    await openModal('login');
    const username = screen.getByLabelText('Username') as HTMLInputElement;
    const password = screen.getByLabelText('Password') as HTMLInputElement;
    await fireEvent.input(username, { target: { value: 'alice' } });
    await fireEvent.input(password, { target: { value: 'good' } });
    await fireEvent.click(submitDialog());

    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    expect(screen.getByText(/alice/)).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Logout' })).toBeTruthy();
    // The login POST carried the credentials.
    const loginCall = mockFetch.mock.calls.find((c) => String(c[0]).includes('/auth/login'));
    expect(loginCall).toBeTruthy();
    expect(JSON.parse(loginCall![1].body).username).toBe('alice');
  });

  it('shows the register error and registers a new user', async () => {
    mockFetch
      // onMount GET /api/site (open mode) — consumes the first Once.
      .mockResolvedValueOnce(okJson({ err: 0, site: { name: 'FicNexus', description: '', registration_mode: 'open', version: 'x' } }))
      .mockResolvedValueOnce(okJson({ err: -1, msg: 'taken' }))
      .mockResolvedValueOnce(
        okJson({ err: 0, token: 't2', user: { id: 2, username: 'newuser', role: 0, reputation: 0 } }),
      );
    await openModal('register');
    const username = screen.getByLabelText('Username') as HTMLInputElement;
    const password = screen.getByLabelText('Password') as HTMLInputElement;
    await fireEvent.input(username, { target: { value: 'taken' } });
    await fireEvent.input(password, { target: { value: 'pw' } });
    await fireEvent.click(submitDialog());
    await waitFor(() => {
      expect(screen.getByText(/Registration failed/)).toBeTruthy();
    });

    await fireEvent.input(username, { target: { value: 'newuser' } });
    await fireEvent.click(submitDialog());
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    expect(screen.getByText(/newuser/)).toBeTruthy();
  });

  it('logs out when the logout button is clicked', async () => {
    // Seed a logged-in session (token + cached user).
    localStorage.setItem('fichub_token', 't1');
    localStorage.setItem('fichub_cached_user', JSON.stringify({ id: 1, username: 'alice', role: 0, reputation: 0 }));
    auth.user = { id: 1, username: 'alice', role: 0, reputation: 0 };
    auth.initialized = true;

    const { default: AuthBar } = await loadAuthBar();
    render(AuthBar);

    expect(screen.getByText(/alice/)).toBeTruthy();
    await fireEvent.click(screen.getByRole('button', { name: 'Logout' }));
    expect(screen.getAllByRole('button', { name: 'Login' }).length).toBeGreaterThan(0);
    expect(localStorage.getItem('fichub_token')).toBeNull();
  });

  it('shows the invite-code field and mode hint when the site runs on invitations', async () => {
    // First fetch (onMount) is GET /api/site → invite mode.
    mockFetch.mockResolvedValueOnce(
      okJson({ err: 0, site: { name: 'FicNexus', description: '', registration_mode: 'invite', version: 'x' } }),
    );
    await openModal('register');
    await waitFor(() => {
      expect(screen.getByText(/Registration is by invitation only/)).toBeTruthy();
    });
    expect(screen.getByLabelText('Invite code')).toBeTruthy();

    // Register submits the invite code alongside credentials.
    mockFetch.mockResolvedValueOnce(
      okJson({ err: 0, token: 't3', user: { id: 4, username: 'invitee', role: 0, reputation: 0 } }),
    );
    const username = screen.getByLabelText('Username') as HTMLInputElement;
    const password = screen.getByLabelText('Password') as HTMLInputElement;
    const code = screen.getByLabelText('Invite code') as HTMLInputElement;
    await fireEvent.input(username, { target: { value: 'invitee' } });
    await fireEvent.input(password, { target: { value: 'pw' } });
    await fireEvent.input(code, { target: { value: 'ABC123xyz789' } });
    await fireEvent.click(submitDialog());
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    const regCall = mockFetch.mock.calls.find((c) => String(c[0]).includes('/auth/register'));
    expect(regCall).toBeTruthy();
    expect(JSON.parse(regCall![1].body).invite_code).toBe('ABC123xyz789');
  });
});
