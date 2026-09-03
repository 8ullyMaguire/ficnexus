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

export type { LocaleCode } from './dictionaries';

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

  /**
   * Curator-reviewed UI string overrides keyed by locale, loaded from the
   * backend (`GET /api/translations/{locale}` → legacy `translations`
   * table). These win over the static TS dictionaries so the translation
   * pipeline (proposals → approval) can correct UI strings without a
   * frontend redeploy. English is the source of truth and never overridden.
   */
  overrides = $state<Partial<Record<LocaleCode, Record<string, string>>>>({});

  /** In-flight override fetches, keyed by locale (dedupe concurrent loads). */
  #overrideLoads = new Map<LocaleCode, Promise<void>>();

  get dictionary(): Dictionary {
    return dictionaries[this.locale] ?? dictionaries[DEFAULT_LOCALE];
  }

  /**
   * Look up a translation key in the current locale with fallback to
   * English and then the raw key. Supports `{var}` interpolation.
   * DB overrides (curator-approved UI strings) win over static entries.
   *
   *   t('search.results', { count: 42 })  →  "42 results"
   */
  t(key: string, vars?: Record<string, string | number>): string {
    let value: string | undefined =
      this.overrides[this.locale]?.[key] ?? dictionaries[this.locale][key as TranslationKey];
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
   * localStorage. Does NOT call the backend for locale persistence — the
   * LocaleSelector owns the PUT /api/auth/locale call so failures don't
   * block instant switching. Kicks off a best-effort load of DB UI-string
   * overrides for the new locale (fire-and-forget).
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

  /**
   * Fetch curator UI-string overrides for a locale from
   * `GET /api/translations/{locale}` and merge them into the override
   * map (reactively — existing renders update). Best-effort: on failure
   * the static dictionaries keep serving. Deduplicated per locale.
   */
  loadOverrides(locale: LocaleCode): Promise<void> {
    // English is the source of truth — never overridden from the DB.
    if (locale === DEFAULT_LOCALE) return Promise.resolve();
    const existing = this.#overrideLoads.get(locale);
    if (existing) return existing;
    const load = (async () => {
      try {
        const res = await fetch(`/api/translations/${encodeURIComponent(locale)}`);
        if (!res.ok) return;
        const json = (await res.json()) as { err: number; translations?: Record<string, Record<string, string>> };
        if (json.err !== 0 || !json.translations) return;
        // Namespace groups are flattened into dot-keys ('nav.discover').
        const flat: Record<string, string> = {};
        for (const [ns, entries] of Object.entries(json.translations)) {
          for (const [k, v] of Object.entries(entries)) flat[`${ns}.${k}`] = v;
        }
        this.overrides = { ...this.overrides, [locale]: { ...this.overrides[locale], ...flat } };
      } catch {
        /* offline — static dictionaries keep working */
      } finally {
        this.#overrideLoads.delete(locale);
      }
    })();
    this.#overrideLoads.set(locale, load);
    return load;
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
    // Best-effort pull of curator UI-string overrides for non-English
    // locales (fire-and-forget; static dictionaries serve until/if it lands).
    if (this.locale !== DEFAULT_LOCALE) void this.loadOverrides(this.locale);
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
