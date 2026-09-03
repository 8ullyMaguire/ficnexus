// Syntax parser for the search bar.
// Converts AO3-like query syntax into SearchFilters.
//
// Syntax:
//   bare words         → q (full-text search)
//   title:...          → q (title only)
//   author:...         → q (author only)
//   creator:...        → alias for author:
//   fandom:...         → include_tags (type 1)
//   char:...           → include_tags (type 2)
//   character:...      → alias for char:
//   rel:...            → include_tags (type 3)
//   relationship:...   → alias for rel:
//   tag:...            → include_tags (type 4)
//   freeform:...       → alias for tag:
//   -fandom:...        → exclude_tags (type 1)
//   -tag:...           → exclude_tags (type 4)
//   words:MIN-MAX      → min_words, max_words
//   words:>N           → min_words
//   words:<N           → max_words
//   chapters:MIN-MAX   → min_chapters, max_chapters
//   complete:true      → complete=true
//   complete:false     → complete=false
//   site:ao3           → source=archiveofourown.org
//   site:ffn           → source=fanfiction.net
//   site:sv            → source=forums.sufficientvelocity.com
//   site:sb            → source=forums.spacebattles.com
//   sort:updated       → sort=updated
//   after:YYYY-MM-DD   → date_from
//   before:YYYY-MM-DD  → date_to
//
// Multi-word values: everything after the colon is the value until the
// next recognized key token. Quoted strings also work:
//   fandom:"Harry Potter"   → same as fandom:Harry Potter

import type { SearchFilters } from '$lib/api/search';
import { defaultFilters } from '$lib/api/search';

const SITE_MAP: Record<string, string> = {
  ao3: 'archiveofourown.org',
  ff: 'fanfiction.net',
  ffn: 'fanfiction.net',
  fp: 'fictionpress.com',
  sb: 'forums.spacebattles.com',
  sv: 'forums.sufficientvelocity.com',
};

/** Keys that start a key:value token. */
const KNOWN_KEYS = new Set([
  'title', 't',
  'author', 'creator', 'a',
  'fandom', 'f',
  'char', 'character', 'c',
  'rel', 'relationship', 'r',
  'tag', 'freeform',
  'words', 'w',
  'chapters', 'ch',
  'complete', 'comp',
  'site', 's',
  'sort',
  'after',
  'before',
]);

/** Check if a token (or stripped token) starts with a known key. */
function isKeyToken(s: string): boolean {
  let t = s;
  if (t.startsWith('-')) t = t.slice(1);
  const colonIdx = t.indexOf(':');
  if (colonIdx < 1) return false;
  const key = t.slice(0, colonIdx).toLowerCase();
  return KNOWN_KEYS.has(key);
}

/** Tokenize the raw query, handling greedy multi-word values after colons. */
function tokenize(raw: string): string[] {
  const tokens: string[] = [];
  let current = '';
  let inQuote = false;
  let quoteChar = '';

  for (const ch of raw) {
    if (inQuote) {
      if (ch === quoteChar) {
        inQuote = false;
      } else {
        current += ch;
      }
    } else if (ch === '"' || ch === "'") {
      inQuote = true;
      quoteChar = ch;
    } else if (ch === ' ' || ch === '\t') {
      if (current) {
        tokens.push(current);
        current = '';
      }
    } else {
      current += ch;
    }
  }
  if (current) tokens.push(current);

  return tokens;
}

/** Greedy tokenize: merge tokens after key:value until next key token. */
function greedyTokenize(raw: string): string[] {
  const basic = tokenize(raw);
  const result: string[] = [];
  let i = 0;

  while (i < basic.length) {
    const token = basic[i];

    if (isKeyToken(token)) {
      // Check if this is a single-token value or multi-word
      const colonIdx = token.indexOf(':');
      const afterColon = token.slice(colonIdx + 1);

      // If the value after colon is empty (e.g. "fandom:"), take the next token(s)
      if (!afterColon) {
        const parts: string[] = [token];
        i++;
        while (i < basic.length && !isKeyToken(basic[i])) {
          parts.push(basic[i]);
          i++;
        }
        result.push(parts.join(' '));
      } else {
        // Value is present — but is it complete?
        // If the key is a "range" key (words, chapters), don't merge
        const key = token.slice(0, colonIdx).toLowerCase();
        if (key === 'words' || key === 'w' || key === 'chapters' || key === 'ch' || key === 'sort' || key === 'complete' || key === 'comp' || key === 'after' || key === 'before' || key === 'site' || key === 's') {
          result.push(token);
          i++;
        } else {
          // For tag-like keys, merge subsequent non-key tokens
          const parts: string[] = [token];
          i++;
          while (i < basic.length && !isKeyToken(basic[i])) {
            parts.push(basic[i]);
            i++;
          }
          result.push(parts.join(' '));
        }
      }
    } else {
      result.push(token);
      i++;
    }
  }

  return result;
}

interface ParsedToken {
  key: string;
  value: string;
  exclude: boolean;
}

/** Parse a single token like "fandom:Harry Potter" or "-tag:Angst". */
function parseToken(raw: string): ParsedToken | null {
  let exclude = false;
  let s = raw;
  if (s.startsWith('-')) {
    exclude = true;
    s = s.slice(1);
  }

  const colonIdx = s.indexOf(':');
  if (colonIdx < 1) return null;

  const key = s.slice(0, colonIdx).toLowerCase();
  const value = s.slice(colonIdx + 1).trim();
  if (!value) return null;

  return { key, value, exclude };
}

function resolveSite(value: string): string {
  return SITE_MAP[value.toLowerCase()] ?? value;
}

/** Parse the search query string into SearchFilters. */
export function parseSearchQuery(raw: string): SearchFilters {
  const filters = defaultFilters();
  if (!raw.trim()) return filters;

  const tokens = greedyTokenize(raw);
  const bareWords: string[] = [];

  for (const token of tokens) {
    const parsed = parseToken(token);
    if (!parsed) {
      bareWords.push(token);
      continue;
    }

    const { key, value, exclude } = parsed;

    switch (key) {
      case 'title':
      case 't':
        bareWords.push(value);
        break;

      case 'author':
      case 'creator':
      case 'a':
        bareWords.push(value);
        break;

      case 'fandom':
      case 'f':
        if (exclude) {
          filters.exclude_tags = appendTag(filters.exclude_tags, 1, value);
        } else {
          filters.include_tags = appendTag(filters.include_tags, 1, value);
        }
        break;

      case 'char':
      case 'character':
      case 'c':
        if (exclude) {
          filters.exclude_tags = appendTag(filters.exclude_tags, 2, value);
        } else {
          filters.include_tags = appendTag(filters.include_tags, 2, value);
        }
        break;

      case 'rel':
      case 'relationship':
      case 'r':
        if (exclude) {
          filters.exclude_tags = appendTag(filters.exclude_tags, 3, value);
        } else {
          filters.include_tags = appendTag(filters.include_tags, 3, value);
        }
        break;

      case 'tag':
      case 'freeform':
        if (exclude) {
          filters.exclude_tags = appendTag(filters.exclude_tags, 4, value);
        } else {
          filters.include_tags = appendTag(filters.include_tags, 4, value);
        }
        break;

      case 'words':
      case 'w': {
        const range = parseRange(value);
        if (range.min !== null) filters.min_words = range.min;
        if (range.max !== null) filters.max_words = range.max;
        break;
      }

      case 'chapters':
      case 'ch': {
        const range = parseRange(value);
        if (range.min !== null) filters.min_chapters = Math.round(range.min);
        if (range.max !== null) filters.max_chapters = Math.round(range.max);
        break;
      }

      case 'complete':
      case 'comp':
        if (value === 'true' || value === 'yes' || value === '1') {
          filters.complete = true;
        } else if (value === 'false' || value === 'no' || value === '0') {
          filters.complete = false;
        }
        break;

      case 'site':
      case 's':
        filters.source = resolveSite(value);
        break;

      case 'sort':
        filters.sort = value.toLowerCase();
        break;

      case 'after':
        filters.date_from = normalizeDate(value, 'start');
        break;

      case 'before':
        filters.date_to = normalizeDate(value, 'end');
        break;

      default:
        bareWords.push(token);
        break;
    }
  }

  if (bareWords.length > 0) {
    filters.q = bareWords.join(' ');
  }

  return filters;
}

function appendTag(existing: string, typeId: number, name: string): string {
  const entry = `${typeId}:${name}`;
  return existing ? `${existing},${entry}` : entry;
}

function parseRange(value: string): { min: number | null; max: number | null } {
  if (value.startsWith('>')) {
    const n = Number(value.slice(1));
    return { min: isNaN(n) ? null : n, max: null };
  }
  if (value.startsWith('<')) {
    const n = Number(value.slice(1));
    return { min: null, max: isNaN(n) ? null : n };
  }

  const parts = value.split('-');
  if (parts.length === 2) {
    const min = Number(parts[0]);
    const max = Number(parts[1]);
    return {
      min: isNaN(min) ? null : min,
      max: isNaN(max) ? null : max,
    };
  }

  const n = Number(value);
  return { min: isNaN(n) ? null : n, max: null };
}

function normalizeDate(value: string, bound: 'start' | 'end'): string {
  if (/^\d{4}-\d{2}-\d{2}$/.test(value)) {
    return bound === 'start' ? `${value}T00:00:00Z` : `${value}T23:59:59Z`;
  }
  if (/^\d{4}-\d{2}$/.test(value)) {
    return bound === 'start' ? `${value}-01T00:00:00Z` : `${value}-28T23:59:59Z`;
  }
  if (/^\d{4}$/.test(value)) {
    return bound === 'start' ? `${value}-01-01T00:00:00Z` : `${value}-12-31T23:59:59Z`;
  }
  return value;
}
