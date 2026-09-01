import { describe, it, expect, beforeEach } from 'vitest';
import { loadPrefs, savePrefs, getPref, setPref, applyUiParam, type UserPrefs } from './prefs';

beforeEach(() => {
  localStorage.clear();
});

describe('prefs', () => {
  it('returns defaults when nothing is stored', () => {
    expect(loadPrefs()).toEqual({
      defaultFormat: 'epub',
      hideRead: false,
      hideBookmarked: false,
      libraryOnly: false,
      readerTheme: 'light',
      uiMode: 'archive',
      archiveSkin: 'zerafina',
    });
  });

  it('round-trips a full blob', () => {
    const prefs: UserPrefs = {
      defaultFormat: 'mobi',
      hideRead: true,
      hideBookmarked: false,
      libraryOnly: true,
      readerTheme: 'dark',
      uiMode: 'archive',
      archiveSkin: 'ao3',
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
    expect(loadPrefs()).toEqual({
      defaultFormat: 'epub',
      hideRead: false,
      hideBookmarked: false,
      libraryOnly: false,
      readerTheme: 'light',
      uiMode: 'archive',
      archiveSkin: 'zerafina',
    });
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
      defaultFormat: 'epub',
      hideRead: false,
      hideBookmarked: false,
      libraryOnly: false,
      readerTheme: 'sepia',
      uiMode: 'archive',
      archiveSkin: 'zerafina',
    });
  });

  it('uiMode defaults to archive', () => {
    expect(getPref('uiMode')).toBe('archive');
  });

  it('uiMode can be set to modern', () => {
    setPref('uiMode', 'modern');
    expect(getPref('uiMode')).toBe('modern');
  });

  it('archiveSkin defaults to zerafina', () => {
    expect(getPref('archiveSkin')).toBe('zerafina');
  });

  it('archiveSkin can be set to ao3', () => {
    setPref('archiveSkin', 'ao3');
    expect(getPref('archiveSkin')).toBe('ao3');
  });

  it('uiMode survives load', () => {
    setPref('uiMode', 'modern');
    const loaded = loadPrefs();
    expect(loaded.uiMode).toBe('modern');
  });
});

describe('applyUiParam', () => {
  it('sets uiMode from ?ui=modern', () => {
    const url = new URL('http://localhost/?ui=modern');
    applyUiParam(url);
    expect(getPref('uiMode')).toBe('modern');
  });

  it('sets uiMode from ?ui=archive', () => {
    setPref('uiMode', 'modern');
    const url = new URL('http://localhost/?ui=archive');
    applyUiParam(url);
    expect(getPref('uiMode')).toBe('archive');
  });

  it('ignores invalid ui param', () => {
    setPref('uiMode', 'modern');
    const url = new URL('http://localhost/?ui=invalid');
    applyUiParam(url);
    expect(getPref('uiMode')).toBe('modern');
  });

  it('ignores missing ui param', () => {
    setPref('uiMode', 'modern');
    const url = new URL('http://localhost/');
    applyUiParam(url);
    expect(getPref('uiMode')).toBe('modern');
  });
});
