import { describe, it, expect } from 'vitest';
import { decideStrategy } from './strategies';

describe('decideStrategy', () => {
  it('returns network-first for the root navigation (never stale shell)', () => {
    expect(decideStrategy('/')).toBe('network-first');
    expect(decideStrategy('https://fichub.test/')).toBe('network-first');
  });

  it('returns precache for the app shell assets', () => {
    expect(decideStrategy('/_app/immutable/entry/start.abc123.js')).toBe('precache');
    expect(decideStrategy('/_app/immutable/nodes/0.def456.css')).toBe('precache');
    expect(decideStrategy('/manifest.webmanifest')).toBe('precache');
    expect(decideStrategy('/sw.js')).toBe('precache');
    expect(decideStrategy('/icon-192.png')).toBe('precache');
    expect(decideStrategy('/favicon.png')).toBe('precache');
  });

  it('returns network-first for public fic-content API endpoints', () => {
    expect(decideStrategy('/api/reader/12345')).toBe('network-first');
    expect(decideStrategy('/api/epub?q=https%3A%2F%2Farchiveofourown.org%2Fworks%2F1')).toBe('network-first');
    expect(decideStrategy('/api/meta?q=https%3A%2F%2Ffanfiction.net%2Fs%2F1')).toBe('network-first');
    expect(decideStrategy('/api/search/similar/42')).toBe('network-first');
    expect(decideStrategy('https://fichub.test/api/reader/42')).toBe('network-first');
  });

  it('never caches private/user-specific API endpoints', () => {
    expect(decideStrategy('/api/auth/me')).toBe('none');
    expect(decideStrategy('/api/auth/login')).toBe('none');
    expect(decideStrategy('/api/bookmarks')).toBe('none');
    expect(decideStrategy('/api/admin/bots')).toBe('none');
    expect(decideStrategy('/api/recommendations/votes?url_id=1')).toBe('none');
    expect(decideStrategy('/api/work/1/ratings')).toBe('none');
    expect(decideStrategy('/api/comments')).toBe('none');
  });

  it('returns cache-first for fic reading routes', () => {
    expect(decideStrategy('/read/12345')).toBe('cache-first');
    expect(decideStrategy('/works/abc-123')).toBe('cache-first');
    expect(decideStrategy('/read/archiveofourown.org%2Fworks%2F12345')).toBe('cache-first');
    expect(decideStrategy('/works/https%3A%2F%2Ffanfiction.net%2Fs%2F1')).toBe('cache-first');
  });

  it('returns none for everything else', () => {
    expect(decideStrategy('/login')).toBe('none');
    expect(decideStrategy('/search?q=harry')).toBe('none');
    expect(decideStrategy('/api/reader/123')).toBe('network-first'); // api wins over read-like path
    expect(decideStrategy('https://cdn.example.com/app.js')).toBe('none');
    expect(decideStrategy('not a url')).toBe('none');
  });
});
