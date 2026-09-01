// Apply, save, and load theme tokens to/from CSS custom properties.

import type { ThemeTokens } from './presets.js';
import { defaultDark, defaultLight } from './presets.js';

const STORAGE_KEY = 'fichub-theme';

/** Map density keyword to a CSS scale multiplier. */
function densityScale(d: ThemeTokens['density']): string {
  return d === 'compact' ? '0.8' : d === 'spacious' ? '1.2' : '1';
}

/**
 * Push every theme token onto `document.documentElement` as a
 * `--fh-*` custom property.  Components read these via
 * `var(--fh-*)` so the whole UI updates instantly.
 */
export function applyTheme(tokens: ThemeTokens): void {
  if (typeof document === 'undefined') return; // SSR safety
  const root = document.documentElement;
  root.style.setProperty('--fh-accent', tokens.accent);
  root.style.setProperty('--fh-bg', tokens.bg);
  root.style.setProperty('--fh-surface', tokens.surface);
  root.style.setProperty('--fh-text', tokens.text);
  root.style.setProperty('--fh-muted', tokens.muted);
  root.style.setProperty('--fh-radius', tokens.radius);
  root.style.setProperty('--fh-font', tokens.font);
  root.style.setProperty('--fh-density', densityScale(tokens.density));
  root.style.setProperty('--fh-reader-font', tokens.readerFont);
  root.style.setProperty('--fh-reader-width', tokens.readerWidth);
  root.style.setProperty('--fh-reader-line-height', tokens.readerLineHeight);

  // Also override the existing `--color-*` vars so legacy styles update too.
  root.style.setProperty('--color-primary', tokens.accent);
  root.style.setProperty('--color-primary-hover', tokens.accent);
  root.style.setProperty('--color-bg', tokens.bg);
  root.style.setProperty('--color-surface', tokens.surface);
  root.style.setProperty('--color-text', tokens.text);
  root.style.setProperty('--color-muted', tokens.muted);
  root.style.setProperty('--radius', tokens.radius);
  root.style.setProperty('--font', tokens.font);
}

/**
 * Clear all `--fh-*` and overridden `--color-*` properties so the
 * stylesheet defaults take over again (e.g. when resetting to default).
 */
export function clearTheme(): void {
  if (typeof document === 'undefined') return;
  const root = document.documentElement;
  const vars = [
    '--fh-accent', '--fh-bg', '--fh-surface', '--fh-text', '--fh-muted',
    '--fh-radius', '--fh-font', '--fh-density',
    '--fh-reader-font', '--fh-reader-width', '--fh-reader-line-height',
    '--color-primary', '--color-primary-hover',
    '--color-bg', '--color-surface', '--color-text', '--color-muted',
    '--radius', '--font',
  ];
  vars.forEach((v) => root.style.removeProperty(v));
}

/**
 * Load the user's saved theme from localStorage.
 * Falls back to the dark or light preset depending on the OS preference.
 */
export function loadTheme(): ThemeTokens {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return JSON.parse(raw) as ThemeTokens;
  } catch { /* corrupt data — fall through */ }
  return prefersDark() ? defaultDark : defaultLight;
}

/** Persist a theme to localStorage. */
export function saveTheme(tokens: ThemeTokens): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(tokens));
}

/** Remove the saved theme from localStorage. */
export function removeSavedTheme(): void {
  localStorage.removeItem(STORAGE_KEY);
}

/** Detect OS dark-mode preference. */
export function prefersDark(): boolean {
  if (typeof window === 'undefined') return true;
  return window.matchMedia('(prefers-color-scheme: dark)').matches;
}
