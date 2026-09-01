// Reactive i18n store for FicNexus's SvelteKit frontend (Svelte 5 runes).
//
// - `locale` is a $state singleton that components can read reactively.
// - `t(key, vars)` looks up the current locale's dictionary, falls back to
//   English, then to the raw key, and interpolates `{var}` placeholders.
// - `setLocale(code)` switches the UI language immediately and persists the
//   choice to localStorage ('fichub_locale', the same key the LocaleSelector
//   and the backend PUT /api/auth/locale flow already use).
// - `detectBrowserLocale()` maps navigator.languages to the closest
//   supported locale ('en-US'→'en', 'pt-PT'→'pt-BR', 'zh-CN'→'zh', ...).
// - `initI18n()` resolves the initial locale: localStorage → auth.user.locale
//   → browser language, and keeps the <html lang> attribute in sync.

import { dictionaries, LOCALES, type Dictionary, type LocaleCode, type TranslationKey } from './dictionaries';

export const DEFAULT_LOCALE: LocaleCode = 'en';
export const STORAGE_KEY = 'fichub_locale';

export const supportedLocales: readonly string[] = LOCALES;

export function isSupportedLocale(code: string | null | undefined): code is LocaleCode {
  return !!code && (LOCALES as readonly string[]).includes(code);
}

class I18nStore {
  /** Currently active locale — components read this reactively. */
  locale = $state<LocaleCode>(DEFAULT_LOCALE);

  /** Optional: set after auth.init() resolves so user.locale can win over
   *  the browser auto-detect when no explicit localStorage choice exists. */
  initialized = $state(false);

  get dictionary(): Dictionary {
    return dictionaries[this.locale] ?? dictionaries[DEFAULT_LOCALE];
  }

  /**
   * Look up a translation key in the current locale with fallback to
   * English and then the raw key. Supports `{var}` interpolation.
   *
   *   t('search.results', { count: 42 })  →  "42 results"
   */
  t(key: string, vars?: Record<string, string | number>): string {
    const dict = this.dictionary;
    let value: string | undefined = dict[key as TranslationKey];
    if (value === undefined) value = dictionaries.en[key as TranslationKey];
    if (value === undefined) value = key;
    if (vars && Object.keys(vars).length > 0) {
      for (const [k, v] of Object.entries(vars)) {
        value = value.replaceAll(`{${k}}`, String(v));
      }
    }
    return value;
  }

  /**
   * Switch the UI language immediately and persist the choice to
   * localStorage. Does NOT call the backend — the LocaleSelector owns the
   * PUT /api/auth/locale call so failures don't block instant switching.
   */
  setLocale(code: string): boolean {
    if (!isSupportedLocale(code)) return false;
    this.locale = code;
    try {
      localStorage.setItem(STORAGE_KEY, code);
    } catch {
      /* storage unavailable — in-memory switch still works */
    }
    this.syncHtmlLang();
    return true;
  }

  /** Map a browser language tag (or list) to the closest supported locale. */
  detectBrowserLocale(languages: readonly string[] | undefined = typeof navigator !== 'undefined' ? navigator.languages : undefined): LocaleCode {
    if (!languages || languages.length === 0) return DEFAULT_LOCALE;
    for (const raw of languages) {
      const tag = String(raw).toLowerCase();
      const base = tag.split('-')[0];
      // Exact supported match (case-insensitive): 'en' / 'es' / 'pt-br' / 'zh'
      for (const supported of LOCALES) {
        if (supported.toLowerCase() === tag || supported.toLowerCase() === base) {
          return supported;
        }
      }
      // Regional fallbacks: 'pt-PT' → 'pt-BR', 'zh-Hans' → 'zh', 'en-GB' → 'en'
      if (base === 'pt') return 'pt-BR';
      if (base === 'zh') return 'zh';
      if (base === 'en') return 'en';
      if (base === 'de') return 'de';
      if (base === 'es') return 'es';
      if (base === 'fr') return 'fr';
    }
    return DEFAULT_LOCALE;
  }

  /** Keep <html lang="..."> in sync with the active locale. */
  syncHtmlLang(): void {
    if (typeof document !== 'undefined') {
      document.documentElement.lang = this.locale;
    }
  }

  /**
   * Resolve the initial locale: localStorage → auth.user.locale →
   * detectBrowserLocale(). Safe to call more than once (idempotent;
   * an explicit user choice always wins).
   */
  initI18n(options?: { userLocale?: string | null; browserLanguages?: readonly string[] }): LocaleCode {
    const stored = typeof localStorage !== 'undefined' ? localStorage.getItem(STORAGE_KEY) : null;
    if (isSupportedLocale(stored)) {
      this.setLocale(stored);
    } else if (isSupportedLocale(options?.userLocale)) {
      this.setLocale(options!.userLocale!);
    } else {
      this.setLocale(this.detectBrowserLocale(options?.browserLanguages));
    }
    this.initialized = true;
    return this.locale;
  }
}

export const i18n = new I18nStore();

/** Reactive translation helper for components:
 *  `{t('nav.discover')}` — re-renders when the locale changes. */
export function t(key: string, vars?: Record<string, string | number>): string {
  return i18n.t(key, vars);
}

/** `$derived`-friendly current locale (reactive). */
export function currentLocale(): LocaleCode {
  return i18n.locale;
}

/** Map browser language tags to the closest supported locale. */
export function detectBrowserLocale(
  languages: readonly string[] | undefined = typeof navigator !== 'undefined' ? navigator.languages : undefined,
): LocaleCode {
  return i18n.detectBrowserLocale(languages);
}

export function setLocale(code: string): boolean {
  return i18n.setLocale(code);
}

export function initI18n(options?: { userLocale?: string | null; browserLanguages?: readonly string[] }): LocaleCode {
  return i18n.initI18n(options);
}
