import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { i18n, STORAGE_KEY } from '$lib/i18n/index.svelte';

// LocaleSelector: loads locales on mount, remembers the stored locale,
// PUTs the chosen locale back to /api/auth/locale — AND switches the UI
// language immediately via the reactive i18n store.
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  i18n.setLocale('en');
  // Reset the auth singleton so auth.init() re-fetches /api/auth/me.
  // (Dynamic import keeps the singleton module instance shared with the SUT.)
  void import('$lib/stores/auth.svelte').then(({ auth }) => {
    auth.initialized = false;
    auth.user = null;
  });
});

async function loadSelector() {
  return await import('$lib/components/LocaleSelector.svelte');
}

describe('LocaleSelector', () => {
  it('loads locales and renders them as options', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, user: null }) }) // auth.getMe
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, locales: [{ code: 'en', name: 'English' }, { code: 'es', name: 'Español' }] }) });

    const { default: Selector } = await loadSelector();
    render(Selector);

    await waitFor(() => {
      expect(screen.getByRole('option', { name: 'English' })).toBeTruthy();
    });
    expect(screen.getByRole('option', { name: 'Español' })).toBeTruthy();
  });

  it('preselects the stored locale from localStorage', async () => {
    localStorage.setItem(STORAGE_KEY, 'es');
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, user: null }) })
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, locales: [{ code: 'en', name: 'English' }, { code: 'es', name: 'Español' }] }) });

    const { default: Selector } = await loadSelector();
    render(Selector);

    await waitFor(() => {
      const select = screen.getByRole('combobox') as HTMLSelectElement;
      expect(select.value).toBe('es');
    });
  });

  it('PUTs the new locale and persists it to localStorage', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, user: null }) })
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, locales: [{ code: 'en', name: 'English' }, { code: 'fr', name: 'Français' }] }) })
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0 }) });

    const { default: Selector } = await loadSelector();
    render(Selector);

    await waitFor(() => {
      expect(screen.getByRole('option', { name: 'Français' })).toBeTruthy();
    });

    await fireEvent.change(screen.getByRole('combobox'), { target: { value: 'fr' } });

    await waitFor(() => {
      const putCall = mockFetch.mock.calls.find((c) => String(c[0]).includes('/api/auth/locale'));
      expect(putCall).toBeTruthy();
      expect(putCall![1].method).toBe('PUT');
      expect(JSON.parse(String(putCall![1].body)).locale).toBe('fr');
      expect(localStorage.getItem(STORAGE_KEY)).toBe('fr');
    });
  });

  it('switches the UI language immediately via the i18n store', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, user: null }) })
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, locales: [{ code: 'en', name: 'English' }, { code: 'es', name: 'Español' }] }) })
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0 }) });

    const { default: Selector } = await loadSelector();
    render(Selector);

    await waitFor(() => {
      expect(screen.getByRole('option', { name: 'Español' })).toBeTruthy();
    });

    expect(i18n.locale).toBe('en');
    await fireEvent.change(screen.getByRole('combobox'), { target: { value: 'es' } });

    await waitFor(() => {
      expect(i18n.locale).toBe('es');
    });
    expect(localStorage.getItem(STORAGE_KEY)).toBe('es');
  });
});
