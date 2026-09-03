import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
});

// Mock auth store
vi.mock('$lib/stores/auth.svelte', () => {
  let _user: any = null;
  return {
    auth: {
      get user() { return _user; },
      set user(v: any) { _user = v; },
      get isLoggedIn() { return _user !== null; },
      get username() { return _user?.username ?? null; },
      loading: false,
      initialized: true,
      init: vi.fn().mockResolvedValue(undefined),
      handleLogin: vi.fn().mockImplementation(async (u: string, p: string) => {
        _user = { id: 1, username: u, role: 0, reputation: 0 };
        return true;
      }),
      handleRegister: vi.fn().mockImplementation(async (u: string, p: string) => {
        _user = { id: 2, username: u, role: 0, reputation: 0 };
        return true;
      }),
      handleLogout: vi.fn().mockImplementation(() => {
        _user = null;
      }),
    },
  };
});

vi.mock('$app/navigation', () => ({
  goto: vi.fn(),
}));

async function loadAuthBar() {
  return await import('$lib/components/AuthBar.svelte');
}

describe('AuthBar', () => {
  it('shows login and register buttons when not logged in', async () => {
    const { default: AuthBar } = await loadAuthBar();
    render(AuthBar);

    expect(screen.getByText('Login')).toBeTruthy();
    expect(screen.getByText('Register')).toBeTruthy();
  });

  it('opens login modal', async () => {
    const { default: AuthBar } = await loadAuthBar();
    render(AuthBar);

    await fireEvent.click(screen.getByText('Login'));

    await waitFor(() => {
      expect(screen.getByRole('dialog')).toBeTruthy();
    });
    expect(screen.getByLabelText('Username')).toBeTruthy();
    expect(screen.getByLabelText('Password')).toBeTruthy();
  });

  it('opens register modal', async () => {
    const { default: AuthBar } = await loadAuthBar();
    render(AuthBar);

    await fireEvent.click(screen.getByText('Register'));

    await waitFor(() => {
      expect(screen.getByRole('dialog')).toBeTruthy();
    });
    expect(screen.getByText('Register', { selector: 'h3' })).toBeTruthy();
    expect(screen.getByLabelText('Email (optional)')).toBeTruthy();
  });

  it('closes modal on cancel', async () => {
    const { default: AuthBar } = await loadAuthBar();
    render(AuthBar);

    await fireEvent.click(screen.getByText('Login'));
    await waitFor(() => {
      expect(screen.getByRole('dialog')).toBeTruthy();
    });

    await fireEvent.click(screen.getByText('Cancel'));

    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
  });
});
