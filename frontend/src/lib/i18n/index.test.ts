// i18n module tests: t() fallbacks + interpolation, setLocale persistence,
// detectBrowserLocale mapping, and initI18n resolution order.

import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import {
  i18n,
  t,
  setLocale,
  initI18n,
  detectBrowserLocale,
  DEFAULT_LOCALE,
} from './index.svelte';
import { dictionaries, LOCALES } from './dictionaries';

beforeEach(() => {
  localStorage.clear();
  i18n.setLocale(DEFAULT_LOCALE);
  i18n.initialized = false;
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe('t()', () => {
  it('returns the English string for the default locale', () => {
    expect(t('nav.discover')).toBe('Discover');
    expect(t('home.recommended')).toBe('Recommended for you');
  });

  it('returns the active locale translation', () => {
    i18n.setLocale('es');
    expect(t('nav.discover')).toBe('Descubrir');
    i18n.setLocale('de');
    expect(t('nav.discover')).toBe('Entdecken');
  });

  it('falls back to English when the active locale lacks a key', () => {
    i18n.setLocale('es');
    const dict = dictionaries.es as Record<string, string>;
    const saved = dict['nav.discover'];
    delete dict['nav.discover'];
    try {
      expect(t('nav.discover')).toBe('Discover');
    } finally {
      dict['nav.discover'] = saved;
    }
  });

  it('falls back to the raw key when neither locale nor en has it', () => {
    i18n.setLocale('es');
    expect(t('nope.missing.key')).toBe('nope.missing.key');
  });

  it('interpolates {var} placeholders', () => {
    expect(t('search.results', { count: 42 })).toBe('42 results');
    expect(t('search.page', { page: 3 })).toBe('Page 3');
  });
});

describe('dictionary integrity', () => {
  it('every locale has exactly the same keys as en (no holes, no extras)', () => {
    const enKeys = Object.keys(dictionaries.en).sort();
    for (const code of LOCALES) {
      const keys = Object.keys(dictionaries[code]).sort();
      expect(keys, `key set mismatch for ${code}`).toEqual(enKeys);
    }
  });

  it('no dictionary contains an untranslated English leftover', () => {
    const samples: [string, string][] = [
      ['nav.discover', 'Discover'],
      ['home.recommended', 'Recommended for you'],
      ['search.title', 'Advanced Search'],
      ['roadmap.title', 'Roadmap Arena'],
      ['fic.readOnline', '📖 Read Online'],
    ];
    for (const code of LOCALES) {
      if (code === 'en') continue;
      for (const [key, enValue] of samples) {
        const value = dictionaries[code][key as keyof typeof dictionaries[typeof code]];
        expect(value, `${code}.${key}`).toBeTruthy();
        expect(value, `${code}.${key} is still English`).not.toBe(enValue);
      }
    }
  });
});

describe('setLocale', () => {
  it('switches the reactive locale and persists to localStorage', () => {
    expect(setLocale('fr')).toBe(true);
    expect(i18n.locale).toBe('fr');
    expect(localStorage.getItem('fichub_locale')).toBe('fr');
    expect(t('nav.login')).toBe('Connexion');
  });

  it('rejects unsupported locale codes and keeps the current locale', () => {
    i18n.setLocale('en');
    expect(setLocale('xx')).toBe(false);
    expect(i18n.locale).toBe('en');
    expect(localStorage.getItem('fichub_locale')).toBe('en');
  });

  it('syncs the <html lang> attribute', () => {
    document.documentElement.lang = 'en';
    setLocale('zh');
    expect(document.documentElement.lang).toBe('zh');
  });
});

describe('detectBrowserLocale', () => {
  it('maps exact supported tags', () => {
    expect(detectBrowserLocale(['en-US'])).toBe('en');
    expect(detectBrowserLocale(['de-DE'])).toBe('de');
    expect(detectBrowserLocale(['es-ES'])).toBe('es');
    expect(detectBrowserLocale(['fr-FR'])).toBe('fr');
    expect(detectBrowserLocale(['pt-BR'])).toBe('pt-BR');
    expect(detectBrowserLocale(['zh-CN'])).toBe('zh');
  });

  it('maps regional variants to the closest supported locale', () => {
    expect(detectBrowserLocale(['pt-PT'])).toBe('pt-BR');
    expect(detectBrowserLocale(['zh-Hans'])).toBe('zh');
    expect(detectBrowserLocale(['zh-TW'])).toBe('zh');
    expect(detectBrowserLocale(['en-GB'])).toBe('en');
  });

  it('walks the full language list before falling back to en', () => {
    expect(detectBrowserLocale(['ja-JP', 'de-DE'])).toBe('de');
    expect(detectBrowserLocale(['xx', 'pt-PT'])).toBe('pt-BR');
    expect(detectBrowserLocale(['ar'])).toBe('en');
    expect(detectBrowserLocale([])).toBe('en');
    expect(detectBrowserLocale(undefined)).toBe('en');
  });
});

describe('initI18n', () => {
  it('prefers localStorage over everything else', () => {
    localStorage.setItem('fichub_locale', 'zh');
    const code = initI18n({ userLocale: 'de', browserLanguages: ['fr-FR'] });
    expect(code).toBe('zh');
    expect(i18n.locale).toBe('zh');
  });

  it('falls back to the user locale when nothing is stored', () => {
    localStorage.removeItem('fichub_locale');
    const code = initI18n({ userLocale: 'pt-BR', browserLanguages: ['fr-FR'] });
    expect(code).toBe('pt-BR');
  });

  it('falls back to browser auto-detect when nothing else matches', () => {
    localStorage.removeItem('fichub_locale');
    const code = initI18n({ userLocale: null, browserLanguages: ['de-DE'] });
    expect(code).toBe('de');
  });

  it('defaults to en when everything is empty', () => {
    localStorage.removeItem('fichub_locale');
    const code = initI18n({ userLocale: null, browserLanguages: [] });
    expect(code).toBe('en');
  });
});

describe('DB UI-string overrides', () => {
  const originalFetch = globalThis.fetch;

  afterEach(() => {
    globalThis.fetch = originalFetch;
    i18n.overrides = {};
  });

  it('loadOverrides flattens namespaces and overrides t() for the locale', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        err: 0,
        translations: { nav: { discover: 'Entdecken!' }, search: { results: '{count} Treffer' } },
      }),
    }) as unknown as typeof fetch;

    await i18n.loadOverrides('de');
    i18n.setLocale('de');
    expect(t('nav.discover')).toBe('Entdecken!');
    expect(t('search.results', { count: 3 })).toBe('3 Treffer');
    // Unknown key still falls through to English / raw key.
    expect(t('no.such.key')).toBe('no.such.key');
  });

  it('non-English locales are unaffected when overrides fail', async () => {
    globalThis.fetch = vi.fn().mockRejectedValue(new Error('offline')) as unknown as typeof fetch;
    await i18n.loadOverrides('es');
    i18n.setLocale('es');
    // Static Spanish dictionary still serves.
    expect(t('nav.discover')).toBe(dictionaries.es['nav.discover']);
  });

  it('English never gets overrides applied', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, translations: { nav: { discover: 'OVERRIDE' } } }),
    }) as unknown as typeof fetch;
    await i18n.loadOverrides('en');
    i18n.setLocale('en');
    expect(t('nav.discover')).toBe(dictionaries.en['nav.discover']);
  });

  it('initI18n pulls overrides for the resolved non-English locale', async () => {
    const fetchSpy = vi.fn().mockRejectedValue(new Error('offline'));
    globalThis.fetch = fetchSpy as unknown as typeof fetch;
    localStorage.removeItem('fichub_locale');
    initI18n({ userLocale: 'es', browserLanguages: [] });
    // Wait a microtask for the fire-and-forget load.
    await vi.waitFor(() => expect(fetchSpy).toHaveBeenCalled());
    expect(String(fetchSpy.mock.calls[0][0])).toContain('/api/translations/es');
  });
});
