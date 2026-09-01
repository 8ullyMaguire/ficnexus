// Reading locale store for translated content display (M1 translation display layer).
//
// - Reads/writes prefs key 'readingLocale' (registered in prefs.ts).
// - Init priority: auth.user.locale → navigator.language (split to base).
// - targetLocale(): string | null — returns null when === 'en' (source language)
//   so components can skip fetching entirely.
// - localeNames: map for the picker UI.

import { auth } from '$lib/stores/auth.svelte';
import { getPref, setPref, persistPref } from '$lib/prefs';
import { supportedLocales, detectBrowserLocale } from '$lib/i18n/index.svelte';
import type { LocaleCode } from '$lib/i18n/dictionaries';

const STORAGE_KEY = 'fichub_reading_locale';

/** Human-readable names for each supported locale (used in the picker). */
export const localeNames: Record<LocaleCode, string> = {
  en: 'English',
  de: 'Deutsch',
  es: 'Español',
  fr: 'Français',
  'pt-BR': 'Português (Brasil)',
  zh: '中文',
};

/**
 * Resolve the initial reading locale:
 * 1. Stored preference (localStorage)
 * 2. Authenticated user's locale (from backend)
 * 3. Browser language detection (falls back to 'en')
 */
function resolveInitialLocale(): LocaleCode {
  // Check localStorage first (explicit user choice)
  if (typeof localStorage !== 'undefined') {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored && supportedLocales.includes(stored)) return stored as LocaleCode;
  }

  // Fall back to auth user's locale
  if (auth.user?.locale && supportedLocales.includes(auth.user.locale)) {
    return auth.user.locale as LocaleCode;
  }

  // Fall back to browser language
  return detectBrowserLocale();
}

class ReadingLocaleStore {
  /** Current reading locale — reactive via $state. */
  locale = $state<LocaleCode>(resolveInitialLocale());

  /** Initialize from auth (call after auth.init() resolves). */
  initFromAuth(): void {
    if (auth.user?.locale && supportedLocales.includes(auth.user.locale)) {
      const userLocale = auth.user.locale as LocaleCode;
      // Only auto-switch if no explicit localStorage choice exists
      const stored = typeof localStorage !== 'undefined' ? localStorage.getItem(STORAGE_KEY) : null;
      if (!stored) {
        this.setLocale(userLocale);
      }
    }
  }

  /** Get the current locale (reactive). */
  get currentLocale(): LocaleCode {
    return this.locale;
  }

  /**
   * Set the reading locale and persist to localStorage + backend prefs.
   * Returns false if the locale is not supported.
   */
  async setLocale(code: string): Promise<boolean> {
    if (!supportedLocales.includes(code)) return false;
    const newLocale = code as LocaleCode;
    this.locale = newLocale;
    try {
      localStorage.setItem(STORAGE_KEY, newLocale);
    } catch {
      // Storage unavailable — in-memory switch still works
    }
    // Also persist to backend prefs (non-blocking)
    persistPref('readingLocale', newLocale);
    return true;
  }

  /**
   * The effective target locale for translations.
   * Returns null when the locale is English (source language) so
   * components can skip fetching translations entirely.
   */
  targetLocale(): string | null {
    return this.locale === 'en' ? null : this.locale;
  }

  /** Check if translations are needed (i.e., targetLocale is not null). */
  get needsTranslation(): boolean {
    return this.targetLocale() !== null;
  }
}

export const readingLocale = new ReadingLocaleStore();

/** Reactive helper for components: returns the current reading locale. */
export function currentReadingLocale(): LocaleCode {
  return readingLocale.currentLocale;
}

/** Reactive helper: returns the target locale for translations (null = English). */
export function targetReadingLocale(): string | null {
  return readingLocale.targetLocale();
}

/** Reactive helper: whether translations should be fetched. */
export function needsTranslation(): boolean {
  return readingLocale.needsTranslation;
}