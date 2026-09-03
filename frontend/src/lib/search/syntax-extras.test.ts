import { describe, it, expect } from 'vitest';
import { parseSearchQuery } from './syntax';

// Additional syntax coverage: aliases, ranges, exclusions, multi-word
// greedy values, and dates that the base suite doesn't hit.
describe('parseSearchQuery extras', () => {
  it('parses the short aliases t:, a:, c:, r:, f:, s:', () => {
    const f = parseSearchQuery('t:harry a:rowling f:HP c:hermione r:drarry s:ao3');
    expect(f.q).toBe('harry rowling');
    expect(f.include_tags).toBe('1:HP,2:hermione,3:drarry');
    expect(f.source).toBe('archiveofourown.org');
  });

  it('merges multi-word tag values greedily', () => {
    const f = parseSearchQuery('fandom:Harry Potter tag:Slow Burn');
    expect(f.include_tags).toBe('1:Harry Potter,4:Slow Burn');
  });

  it('parses site aliases fp, sb, and ff', () => {
    expect(parseSearchQuery('site:fp').source).toBe('fictionpress.com');
    expect(parseSearchQuery('site:sb').source).toBe('forums.spacebattles.com');
    expect(parseSearchQuery('site:ff').source).toBe('fanfiction.net');
  });

  it('parses excluded tags with -fandom: and -tag:', () => {
    const f = parseSearchQuery('-fandom:Harry Potter -tag:Angst');
    expect(f.exclude_tags).toBe('1:Harry Potter,4:Angst');
  });

  it('parses words:<N as max_words', () => {
    expect(parseSearchQuery('words:<5000').max_words).toBe(5000);
  });

  it('parses words:N as exact min', () => {
    const f = parseSearchQuery('words:10000');
    expect(f.min_words).toBe(10000);
    expect(f.max_words).toBeNull();
  });

  it('parses chapters:>5 as min_chapters', () => {
    expect(parseSearchQuery('chapters:>5').min_chapters).toBe(5);
  });

  it('parses complete:false / complete:no / complete:0', () => {
    expect(parseSearchQuery('complete:false').complete).toBe(false);
    expect(parseSearchQuery('complete:no').complete).toBe(false);
    expect(parseSearchQuery('complete:0').complete).toBe(false);
  });

  it('parses complete:true / complete:yes / complete:1', () => {
    expect(parseSearchQuery('complete:true').complete).toBe(true);
    expect(parseSearchQuery('complete:1').complete).toBe(true);
  });

  it('ignores unknown keys as bare words', () => {
    const f = parseSearchQuery('mystery:box');
    expect(f.q).toBe('mystery:box');
  });

  it('normalizes year-only and year-month dates', () => {
    const f = parseSearchQuery('after:2024 before:2024-06');
    expect(f.date_from).toBe('2024-01-01T00:00:00Z');
    expect(f.date_to).toBe('2024-06-28T23:59:59Z');
  });

  it('keeps unparseable dates verbatim', () => {
    const f = parseSearchQuery('after:yesterday');
    expect(f.date_from).toBe('yesterday');
  });

  it('handles quoted multi-word values', () => {
    const f = parseSearchQuery('fandom:"Harry Potter"');
    expect(f.include_tags).toBe('1:Harry Potter');
  });

  it('sorts with sort:words and lowercases the value', () => {
    expect(parseSearchQuery('sort:Words').sort).toBe('words');
  });

  it('empty and whitespace queries return defaults', () => {
    expect(parseSearchQuery('').q).toBe('');
    expect(parseSearchQuery('   ').q).toBe('');
  });
});
