import { describe, it, expect, vi } from 'vitest';
import { formatWords, formatDate, relativeTime, stripHtml, cacheUrl } from './util';

describe('util extras', () => {
  it('formatDate returns ISO date slice', () => {
    expect(formatDate('2024-01-15T10:30:00Z')).toBe('2024-01-15');
  });

  it('formatDate returns empty for invalid/empty input', () => {
    expect(formatDate('')).toBe('');
    expect(formatDate('not-a-date')).toBe('');
  });

  it('relativeTime returns empty for invalid input', () => {
    expect(relativeTime('garbage')).toBe('');
  });

  it('relativeTime renders minutes/hours/days with pluralization', () => {
    const now = Date.now();
    expect(relativeTime(new Date(now - 2 * 60 * 1000).toISOString())).toBe('2 minutes ago');
    expect(relativeTime(new Date(now - 60 * 60 * 1000).toISOString())).toBe('1 hour ago');
    expect(relativeTime(new Date(now - 3 * 24 * 60 * 60 * 1000).toISOString())).toBe('3 days ago');
    expect(relativeTime(new Date(now - 30 * 1000).toISOString())).toBe('less than a minute ago');
  });

  it('stripHtml collapses whitespace and decodes entities', () => {
    expect(stripHtml('<p>Line 1</p><div>Line 2</div>')).toBe('Line 1 Line 2');
    expect(stripHtml('&lt;b&gt;bold&lt;/b&gt; &amp; more')).toBe('<b>bold</b> & more');
    expect(stripHtml('  lots   of   space  ')).toBe('lots of space');
  });

  it('cacheUrl builds the cache path with hash', () => {
    expect(cacheUrl('pdf', 'url-id', 'h1')).toBe('/cache/pdf/url-id?h=h1');
  });

  it('formatWords uses en-US grouping', () => {
    expect(formatWords(1234567)).toBe('1,234,567');
  });
});
