import { describe, it, expect, beforeEach } from 'vitest';
import { loadPrefs, savePrefs, getPref, setPref, applyUiParam, type UserPrefs } from './prefs';

beforeEach(() => {
  localStorage.clear();
});

const DEFAULT_BLOB = {
      defaultFormat: 'epub',
      hideRead: false,
      hideBookmarked: false,
      libraryOnly: false,
      readerTheme: 'light',
      uiMode: 'archive',
      archiveSkin: 'zerafina',
      timezone: '',
      profileVisibility: {
        profile: 'public',
        works: 'public',
        reading_history: 'private',
      },
      readingLocale: '',
    } as const;

describe('prefs', () => {
  it('returns defaults when nothing is stored', () => {
    expect(loadPrefs()).toEqual(DEFAULT_BLOB);
  });

  it('round-trips a full blob', () => {
    const prefs: UserPrefs = {
      defaultFormat: 'mobi',
      hideRead: true,
      hideBookmarked: false,
      libraryOnly: true,
      readerTheme: 'dark',
      uiMode: 'archive', // archive-only: modern was removed
      archiveSkin: 'ao3',
      timezone: 'Europe/Madrid',
      profileVisibility: {
        profile: 'private',
        works: 'followers',
        reading_history: 'public',
      },
      readingLocale: 'es',
    };
    savePrefs(prefs);
    expect(loadPrefs()).toEqual(prefs);
  });

  it('merges partial blobs over defaults', () => {
    savePrefs({ hideRead: true });
    const loaded = loadPrefs();
    expect(loaded.hideRead).toBe(true);
    expect(loaded.defaultFormat).toBe('epub'); // default preserved
  });

  it('falls back to defaults on corrupt JSON', () => {
    localStorage.setItem('fichub_prefs_v1', '{not json');
    expect(loadPrefs()).toEqual(DEFAULT_BLOB);
  });

  it('getPref returns the stored value or the default', () => {
    expect(getPref('defaultFormat')).toBe('epub');
    setPref('defaultFormat', 'pdf');
    expect(getPref('defaultFormat')).toBe('pdf');
    expect(getPref('hideRead')).toBe(false);
  });

  it('setPref persists only the changed blob', () => {
    setPref('readerTheme', 'sepia');
    expect(JSON.parse(localStorage.getItem('fichub_prefs_v1')!)).toEqual({
      ...DEFAULT_BLOB,
      readerTheme: 'sepia',
    });
  });

  it('uiMode defaults to archive', () => {
    expect(getPref('uiMode')).toBe('archive');
  });

  it('uiMode always resolves to archive (modern removed)', () => {
    setPref('uiMode', 'modern'); // legacy write is ignored
    expect(getPref('uiMode')).toBe('archive');
    localStorage.setItem('fichub_prefs_v1', JSON.stringify({ ...DEFAULT_BLOB, uiMode: 'modern' }));
    expect(getPref('uiMode')).toBe('archive'); // stored modern value coerced
  });

  it('archiveSkin defaults to zerafina', () => {
    expect(getPref('archiveSkin')).toBe('zerafina');
  });

  it('archiveSkin can be set to ao3', () => {
    setPref('archiveSkin', 'ao3');
    expect(getPref('archiveSkin')).toBe('ao3');
  });
});

describe('applyUiParam', () => {
  it('ignores ?ui=modern (archive-only)', () => {
    const url = new URL('http://localhost/?ui=modern');
    applyUiParam(url);
    expect(getPref('uiMode')).toBe('archive');
  });

  it('keeps archive for ?ui=archive', () => {
    const url = new URL('http://localhost/?ui=archive');
    applyUiParam(url);
    expect(getPref('uiMode')).toBe('archive');
  });

  it('ignores invalid ui param', () => {
    const url = new URL('http://localhost/?ui=invalid');
    applyUiParam(url);
    expect(getPref('uiMode')).toBe('archive');
  });

  it('ignores missing ui param', () => {
    const url = new URL('http://localhost/');
    applyUiParam(url);
    expect(getPref('uiMode')).toBe('archive');
  });
});
