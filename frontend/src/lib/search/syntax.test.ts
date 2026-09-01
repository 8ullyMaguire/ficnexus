import { describe, it, expect } from 'vitest';
import { parseSearchQuery } from './syntax';

describe('parseSearchQuery', () => {
  it('parses bare words into q', () => {
    const f = parseSearchQuery('hello world');
    expect(f.q).toBe('hello world');
    expect(f.include_tags).toBe('');
  });

  it('parses title:', () => {
    const f = parseSearchQuery('title:The Best Story');
    expect(f.q).toBe('The Best Story');
  });

  it('parses author:', () => {
    const f = parseSearchQuery('author:J.K. Rowling');
    expect(f.q).toBe('J.K. Rowling');
  });

  it('parses creator: as alias for author:', () => {
    const f = parseSearchQuery('creator:SomeAuthor');
    expect(f.q).toBe('SomeAuthor');
  });

  it('parses fandom:', () => {
    const f = parseSearchQuery('fandom:Harry Potter');
    expect(f.include_tags).toBe('1:Harry Potter');
  });

  it('parses char:', () => {
    const f = parseSearchQuery('char:Harry Potter');
    expect(f.include_tags).toBe('2:Harry Potter');
  });

  it('parses character: as alias', () => {
    const f = parseSearchQuery('character:Draco Malfoy');
    expect(f.include_tags).toBe('2:Draco Malfoy');
  });

  it('parses rel:', () => {
    const f = parseSearchQuery('rel:Harry/Hermione');
    expect(f.include_tags).toBe('3:Harry/Hermione');
  });

  it('parses tag:', () => {
    const f = parseSearchQuery('tag:Fluff');
    expect(f.include_tags).toBe('4:Fluff');
  });

  it('parses freeform: as alias for tag:', () => {
    const f = parseSearchQuery('freeform:Angst');
    expect(f.include_tags).toBe('4:Angst');
  });

  it('parses exclusion with -', () => {
    const f = parseSearchQuery('-tag:Major Character Death');
    expect(f.exclude_tags).toBe('4:Major Character Death');
  });

  it('parses -fandom:', () => {
    const f = parseSearchQuery('-fandom:Harry Potter');
    expect(f.exclude_tags).toBe('1:Harry Potter');
  });

  it('parses words:10000-50000', () => {
    const f = parseSearchQuery('words:10000-50000');
    expect(f.min_words).toBe(10000);
    expect(f.max_words).toBe(50000);
  });

  it('parses words:>10000', () => {
    const f = parseSearchQuery('words:>10000');
    expect(f.min_words).toBe(10000);
    expect(f.max_words).toBeNull();
  });

  it('parses words:<5000', () => {
    const f = parseSearchQuery('words:<5000');
    expect(f.min_words).toBeNull();
    expect(f.max_words).toBe(5000);
  });

  it('parses chapters:5-20', () => {
    const f = parseSearchQuery('chapters:5-20');
    expect(f.min_chapters).toBe(5);
    expect(f.max_chapters).toBe(20);
  });

  it('parses complete:true', () => {
    const f = parseSearchQuery('complete:true');
    expect(f.complete).toBe(true);
  });

  it('parses complete:false', () => {
    const f = parseSearchQuery('complete:false');
    expect(f.complete).toBe(false);
  });

  it('parses complete:yes', () => {
    const f = parseSearchQuery('complete:yes');
    expect(f.complete).toBe(true);
  });

  it('parses site:ao3', () => {
    const f = parseSearchQuery('site:ao3');
    expect(f.source).toBe('archiveofourown.org');
  });

  it('parses site:ffn', () => {
    const f = parseSearchQuery('site:ffn');
    expect(f.source).toBe('fanfiction.net');
  });

  it('parses site:sv', () => {
    const f = parseSearchQuery('site:sv');
    expect(f.source).toBe('forums.sufficientvelocity.com');
  });

  it('parses sort:updated', () => {
    const f = parseSearchQuery('sort:updated');
    expect(f.sort).toBe('updated');
  });

  it('parses after:2024-01-01', () => {
    const f = parseSearchQuery('after:2024-01-01');
    expect(f.date_from).toBe('2024-01-01T00:00:00Z');
  });

  it('parses before:2024-12-31', () => {
    const f = parseSearchQuery('before:2024-12-31');
    expect(f.date_to).toBe('2024-12-31T23:59:59Z');
  });

  it('handles complex queries', () => {
    const f = parseSearchQuery(
      'fandom:Harry Potter tag:Fluff -tag:Angst words:10000-50000 complete:true sort:updated',
    );
    expect(f.include_tags).toBe('1:Harry Potter,4:Fluff');
    expect(f.exclude_tags).toBe('4:Angst');
    expect(f.min_words).toBe(10000);
    expect(f.max_words).toBe(50000);
    expect(f.complete).toBe(true);
    expect(f.sort).toBe('updated');
  });

  it('handles empty string', () => {
    const f = parseSearchQuery('');
    expect(f.q).toBe('');
    expect(f.include_tags).toBe('');
  });

  it('handles mixed bare words and tokens', () => {
    const f = parseSearchQuery('best story fandom:Harry Potter');
    expect(f.q).toBe('best story');
    expect(f.include_tags).toBe('1:Harry Potter');
  });
});
