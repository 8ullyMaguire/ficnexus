import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import {
  splitChapters,
  countWords,
  stripTags,
  assembleNextUp,
  looksLikeSequel,
  restoreReaderState,
  saveReaderState,
  stateKey,
  htmlCacheKey,
  saveReaderHtml,
  tryCachedReaderHtml,
  readerWorkFromPayload,
  loadReaderData,
} from './reader-lib';

describe('reader-lib', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  describe('splitChapters', () => {
    const html = [
      '<!DOCTYPE html><html><body>',
      '<h1>My Fic</h1><p>summary</p>',
      '<h2 id="ch1">Chapter One</h2><p>first chapter text</p>',
      '<h2 id="ch2">Chapter Two</h2><p>second chapter text</p>',
      '<h2 id="ch3">Chapter Three</h2><p>third chapter text</p>',
      '</body></html>',
    ].join('');

    it('splits a full-html bundle into per-chapter fragments', () => {
      const chapters = splitChapters(html);
      expect(chapters).toHaveLength(3);
      expect(chapters[0].title).toBe('Chapter One');
      expect(chapters[1].title).toBe('Chapter Two');
      expect(chapters[2].title).toBe('Chapter Three');
      expect(chapters[0].content).toContain('first chapter text');
      expect(chapters[1].content).toContain('second chapter text');
      // Chapter bodies are isolated: chapter 1 must NOT contain chapter 2's text
      expect(chapters[0].content).not.toContain('second chapter text');
      expect(chapters[2].content).toContain('third chapter text');
    });

    it('counts words per chapter from the body only', () => {
      const chapters = splitChapters(html);
      expect(chapters[0].words).toBe(3); // "first chapter text"
      expect(chapters[1].words).toBe(3);
    });

    it('returns [] when no chapter headings are found', () => {
      expect(splitChapters('<p>no chapters here</p>')).toEqual([]);
    });

    it('falls back to numbered titles when a heading is empty', () => {
      const bare = '<h2 id="ch1"></h2><p>body</p>';
      const chapters = splitChapters(bare);
      expect(chapters[0].title).toBe('Chapter 1');
    });
  });

  describe('countWords / stripTags', () => {
    it('strips tags and counts words', () => {
      expect(countWords('<p>Hello <strong>world</strong>!</p>')).toBe(2);
      expect(countWords('')).toBe(0);
      expect(countWords('one &amp; two')).toBe(2); // entity decoded; '&' is not a word
    });
  });

  describe('looksLikeSequel', () => {
    it('detects numeral / roman-numeral sequels', () => {
      expect(looksLikeSequel('The Awakening', 'The Awakening 2')).toBe(true);
      expect(looksLikeSequel('The Awakening', 'The Awakening II')).toBe(true);
      expect(looksLikeSequel('The Awakening', 'The Awakening: Part 3')).toBe(true);
      expect(looksLikeSequel('The Awakening', 'The Awakening 2nd Edition')).toBe(true);
    });
    it('rejects non-sequels and identical titles', () => {
      expect(looksLikeSequel('The Awakening', 'The Awakening')).toBe(false);
      expect(looksLikeSequel('The Awakening', 'The Return')).toBe(false);
      expect(looksLikeSequel('The Awakening', 'The Awakening, II: The Sequel')).toBe(true); // numeral still detected
    });
  });

  describe('assembleNextUp', () => {
    const sequel = { url_id: 'a1', title: 'Awakening 2', author: 'Alice' };
    const community = { url_id: 'b2', title: 'Other Fic', author: 'Bob' };
    const also = [
      { url_id: 'c3', title: 'Third Fic', author: 'Carol' },
      { url_id: 'd4', title: 'Fourth Fic', author: 'Dave' },
    ];

    it('orders sequel → community → also-bookmarked and caps at limit', () => {
      const out = assembleNextUp(sequel, community, also, 3);
      expect(out.map((c) => c.reason)).toEqual([
        'Next in series',
        'Top community suggestion',
        'Readers also bookmarked',
      ]);
      expect(out).toHaveLength(3);
    });

    it('dedupes by url_id', () => {
      const out = assembleNextUp(sequel, { ...sequel }, also, 5);
      expect(out.filter((c) => c.url_id === 'a1')).toHaveLength(1);
    });

    it('handles missing sources gracefully', () => {
      expect(assembleNextUp(null, null, [], 3)).toEqual([]);
      const out = assembleNextUp(sequel, null, [], 3);
      expect(out).toHaveLength(1);
      expect(out[0].reason).toBe('Next in series');
    });
  });

  describe('state persistence', () => {
    it('round-trips a reader state through localStorage', () => {
      saveReaderState('fic123', { fontSize: 22, theme: 'sepia', chapterIndex: 2, finished: true });
      const s = restoreReaderState('fic123');
      expect(s.fontSize).toBe(22);
      expect(s.theme).toBe('sepia');
      expect(s.chapterIndex).toBe(2);
      expect(s.finished).toBe(true);
    });

    it('returns {} for missing or corrupt state', () => {
      expect(restoreReaderState('nope')).toEqual({});
      localStorage.setItem(stateKey('bad'), '{not json');
      expect(restoreReaderState('bad')).toEqual({});
    });

    it('scopes state per url_id', () => {
      saveReaderState('fic1', { fontSize: 20 });
      saveReaderState('fic2', { fontSize: 24 });
      expect(restoreReaderState('fic1').fontSize).toBe(20);
      expect(restoreReaderState('fic2').fontSize).toBe(24);
    });
  });

  describe('offline HTML cache fallback', () => {
    const payload = {
      err: 0,
      url_id: 'fic123',
      title: 'The Awakening',
      author: 'Alice',
      work_id: 7,
      words: 1200,
      chapters: 2,
      html: [
        '<h1>The Awakening</h1>',
        '<h2 id="ch1">Prologue</h2><p>It began with a whisper.</p>',
        '<h2 id="ch2">Chapter Two</h2><p>The storm rolled in.</p>',
      ].join(''),
    };

    it('saves and restores the reader payload per url_id', () => {
      expect(tryCachedReaderHtml('fic123')).toBeNull();
      saveReaderHtml('fic123', payload as never);
      const cached = tryCachedReaderHtml('fic123');
      expect(cached?.title).toBe('The Awakening');
      expect(cached?.html).toContain('It began with a whisper.');
      expect(localStorage.getItem(htmlCacheKey('fic123'))).toBeTruthy();
      // scoped: other fics don't see this fic's cache
      expect(tryCachedReaderHtml('fic456')).toBeNull();
    });

    it('returns null for missing or corrupt cache entries', () => {
      expect(tryCachedReaderHtml('nope')).toBeNull();
      localStorage.setItem(htmlCacheKey('bad'), '{not json');
      expect(tryCachedReaderHtml('bad')).toBeNull();
      localStorage.setItem(htmlCacheKey('empty'), JSON.stringify({ err: 0 }));
      expect(tryCachedReaderHtml('empty')).toBeNull(); // no html string
    });

    it('falls back to the cached HTML when the fetch rejects (offline)', async () => {
      vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('network down')));
      // Seed the cache the way a successful prior visit would have.
      saveReaderHtml('fic123', payload as never);

      const { work, fromCache } = await loadReaderData('fic123');
      expect(fromCache).toBe(true);
      expect(work.title).toBe('The Awakening');
      expect(work.author).toBe('Alice');
      expect(work.chapters).toHaveLength(2);
      expect(work.chapters[0].title).toBe('Prologue');
      expect(work.chapters[0].content).toContain('It began with a whisper.');
      expect(work.chapters[1].content).toContain('The storm rolled in.');
    });

    it('caches the fresh payload on a successful fetch', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({ ok: true, json: async () => payload })
      );
      const { work, fromCache } = await loadReaderData('fic123');
      expect(fromCache).toBe(false);
      expect(work.chapters).toHaveLength(2);
      expect(tryCachedReaderHtml('fic123')?.title).toBe('The Awakening');
    });

    it('still rethrows when there is no cached HTML to fall back to', async () => {
      vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('network down')));
      await expect(loadReaderData('never-opened')).rejects.toThrow('network down');
    });

    it('falls back for non-OK responses (e.g. 503 from the origin)', async () => {
      saveReaderHtml('fic123', payload as never);
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({ ok: false, status: 503, json: async () => ({}) })
      );
      const { work, fromCache } = await loadReaderData('fic123');
      expect(fromCache).toBe(true);
      expect(work.title).toBe('The Awakening');
    });
  });
});
