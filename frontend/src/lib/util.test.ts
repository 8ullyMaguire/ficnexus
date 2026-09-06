import { describe, it, expect } from 'vitest';
import { formatWords, detectSite, stripHtml, relativeTime, cacheUrl, isFicUrl } from './util';

describe('formatWords', () => {
  it('adds thousands separators', () => {
    expect(formatWords(1234567)).toBe('1,234,567');
    expect(formatWords(50000)).toBe('50,000');
    expect(formatWords(0)).toBe('0');
  });
});

describe('detectSite', () => {
  it('detects AO3', () => {
    expect(detectSite('https://archiveofourown.org/works/1')).toBe('AO3');
  });
  it('detects FanFiction.net', () => {
    expect(detectSite('https://www.fanfiction.net/s/1/1/Title')).toBe('FanFiction.net');
  });
  it('detects SpaceBattles', () => {
    expect(detectSite('https://forums.spacebattles.com/threads/x.1')).toBe('SpaceBattles');
  });
  it('detects SufficientVelocity', () => {
    expect(detectSite('https://forums.sufficientvelocity.com/threads/x.1')).toBe('SufficientVelocity');
  });
  it('detects QuestionableQuesting', () => {
    expect(detectSite('https://forum.questionablequesting.com/threads/x.1')).toBe('QuestionableQuesting');
  });
  it('detects RoyalRoad', () => {
    expect(detectSite('https://www.royalroad.com/fiction/12345')).toBe('RoyalRoad');
  });
  it('returns Unknown for unrecognized', () => {
    expect(detectSite('https://example.com/story')).toBe('Unknown');
  });
});

describe('stripHtml', () => {
  it('removes tags and decodes entities', () => {
    expect(stripHtml('<p>Hello &amp; welcome</p>')).toBe('Hello & welcome');
    expect(stripHtml('<br>line1<br>line2')).toBe('line1 line2');
    expect(stripHtml('')).toBe('');
  });
});

describe('relativeTime', () => {
  it('returns recent for now', () => {
    expect(relativeTime(new Date().toISOString())).toContain('minute');
  });
  it('returns empty for empty input', () => {
    expect(relativeTime('')).toBe('');
  });
});

describe('cacheUrl', () => {
  it('builds correct cache path', () => {
    expect(cacheUrl('epub', 'abc', 'def')).toBe('/cache/epub/abc?h=def');
  });
});

describe('isFicUrl', () => {
  it('returns true for https URLs', () => {
    expect(isFicUrl('https://archiveofourown.org/works/1')).toBe(true);
    expect(isFicUrl('https://www.fanfiction.net/s/1/1/Title')).toBe(true);
  });
  it('returns true for http URLs', () => {
    expect(isFicUrl('http://example.com/fic')).toBe(true);
  });
  it('returns false for plain text (title by author)', () => {
    expect(isFicUrl("Governor's Gambit by Freefaller")).toBe(false);
    expect(isFicUrl('Just a title')).toBe(false);
  });
  it('returns false for empty string', () => {
    expect(isFicUrl('')).toBe(false);
  });
  it('returns false for strings with no protocol', () => {
    expect(isFicUrl('www.archiveofourown.org/works/1')).toBe(false);
    expect(isFicUrl('archiveofourown.org/works/1')).toBe(false);
  });
});
