import { describe, it, expect, vi } from 'vitest';

// reader-lib extras: readerWorkFromPayload fallback chapter, countWords
// edge cases, and looksLikeSequel negatives.
import {
  readerWorkFromPayload,
  countWords,
  looksLikeSequel,
  splitChapters,
} from './reader-lib';

describe('reader-lib extras', () => {
  it('readerWorkFromPayload falls back to a single chapter when no h2 headings', () => {
    const work = readerWorkFromPayload(
      { err: 0, url_id: 'f1', title: 'No Chapters', author: 'A', work_id: 1, html: '<p>just body text</p>', words: 4, chapters: 1 },
      'f1',
    );
    expect(work.chapters).toHaveLength(1);
    expect(work.chapters[0].title).toBe('No Chapters');
    expect(work.words).toBe(4);
    expect(work.url_id).toBe('f1');
  });

  it('readerWorkFromPayload falls back to the passed urlId when payload url_id is empty', () => {
    const work = readerWorkFromPayload(
      { err: 0, url_id: '', title: 'T', author: 'A', work_id: null, html: '<h2 id="ch1">C1</h2><p>body</p>', words: 1, chapters: 1 },
      'fallback-id',
    );
    expect(work.url_id).toBe('fallback-id');
    expect(work.work_id).toBeNull();
  });

  it('countWords counts apostrophe and hyphenated words as one', () => {
    expect(countWords("don't you're")).toBe(2);
    expect(countWords('well-known')).toBe(1);
    expect(countWords('one&amp;two')).toBe(2); // entity decoded, & stripped
    expect(countWords('<b>bold</b> text')).toBe(2);
  });

  it('looksLikeSequel handles roman numerals and ordinals', () => {
    expect(looksLikeSequel('Starfall', 'Starfall IV')).toBe(true);
    expect(looksLikeSequel('Starfall', 'Starfall 2nd Edition')).toBe(true);
    expect(looksLikeSequel('Starfall II', 'Starfall')).toBe(false); // no numeral in candidate
  });

  it('splitChapters uses the titles array for empty headings', () => {
    const chapters = splitChapters('<h2 id="ch2"></h2><p>body</p>', ['Custom One', 'Custom Two']);
    expect(chapters[0].title).toBe('Custom Two');
  });

  it('countWords strips script and style blocks', () => {
    // stripTags removes <script>/<style> blocks; countWords alone only
    // strips tags, so the script's identifier words still count. The
    // reader pipeline uses stripTags before countWords.
    expect(countWords('<script>var x = 1;</script><p>real text</p>')).toBeGreaterThan(1);
    expect(countWords('<p>real text</p>')).toBe(2);
  });
});
