// Default-preference persistence (quick-wins item 5).
//
// v1 is localStorage-only, keyed under a single JSON blob so the whole
// prefs object round-trips in one read/write. Each field is optional —
// missing keys fall back to the built-in default, so old blobs (or a
// partial user edit) degrade gracefully.

export type ArchiveSkin = 'zerafina' | 'ao3';

export interface UserPrefs {
  /** Last/desired download format: 'epub' | 'html' | 'txt' | 'md' | 'mobi' | 'pdf' | 'azw3' | 'docx' | 'fb2' | 'kepub'. */
  defaultFormat?: string;
  /** Search: hide works marked read (status = 'completed'). */
  hideRead?: boolean;
  /** Search: hide works the user bookmarked. */
  hideBookmarked?: boolean;
  /** Search: only show bookmarked works ("My library"). */
  libraryOnly?: boolean;
  /** Web reader theme: 'light' | 'sepia' | 'dark'. */
  readerTheme?: 'light' | 'sepia' | 'dark';
  /** Interface style: 'archive' (AO3-style) or 'modern' (default SvelteKit UI). */
  uiMode?: 'archive' | 'modern';
  /** Archive skin: 'zerafina' (default, rich styling) or 'ao3' (minimal AO3 default). */
  archiveSkin?: ArchiveSkin;
}

const STORAGE_KEY = 'fichub_prefs_v1';

const DEFAULTS: UserPrefs = {
  defaultFormat: 'epub',
  hideRead: false,
  hideBookmarked: false,
  libraryOnly: false,
  readerTheme: 'light',
  uiMode: 'archive',
  archiveSkin: 'zerafina',
};

/** Read the prefs blob; never throws (corrupt JSON → defaults). */
export function loadPrefs(): UserPrefs {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULTS };
    const parsed = JSON.parse(raw) as UserPrefs;
    return { ...DEFAULTS, ...parsed };
  } catch {
    return { ...DEFAULTS };
  }
}

/** Persist the whole blob (only the fields set are stored). */
export function savePrefs(prefs: UserPrefs): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs));
  } catch {
    // Storage full / private mode — prefs are a nicety, never fatal.
  }
}

/** Read one pref with its default. */
export function getPref<K extends keyof UserPrefs>(key: K): NonNullable<UserPrefs[K]> {
  const prefs = loadPrefs();
  const value = prefs[key];
  return (value === undefined ? DEFAULTS[key] : value) as NonNullable<UserPrefs[K]>;
}

/** Set one pref and persist. */
export function setPref<K extends keyof UserPrefs>(key: K, value: UserPrefs[K]): void {
  const prefs = loadPrefs();
  prefs[key] = value;
  savePrefs(prefs);
}

/**
 * Apply a `?ui=archive|modern` URL parameter to prefs.
 * Called by the root layout on every navigation.
 */
export function applyUiParam(url: URL): void {
  const ui = url.searchParams.get('ui');
  if (ui === 'archive' || ui === 'modern') {
    setPref('uiMode', ui);
  }
}

/** Set the archive skin and persist. */
export function setArchiveSkin(skin: ArchiveSkin): void {
  setPref('archiveSkin', skin);
}
