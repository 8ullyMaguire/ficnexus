import { describe, it, expect } from 'vitest';
import {
  parseRange,
  buildSearchQuery,
  formStateToUrl,
  urlToFormState,
  mergeChipIntoForm,
  defaultFormState,
  SUPPORTED,
  type FormState,
  type Chip,
  type SupportedMatrix,
} from './searchForm';

describe('parseRange', () => {
  it('parses plain number as min', () => {
    expect(parseRange('1000')).toEqual({ min: 1000 });
  });

  it('parses > prefix as min', () => {
    expect(parseRange('>1000')).toEqual({ min: 1000 });
  });

  it('parses < prefix as max', () => {
    expect(parseRange('<5000')).toEqual({ max: 5000 });
  });

  it('parses range as min+max', () => {
    expect(parseRange('1000-5000')).toEqual({ min: 1000, max: 5000 });
  });

  it('parses ≥ prefix as min', () => {
    expect(parseRange('≥1000')).toEqual({ min: 1000 });
  });

  it('parses ≤ prefix as max', () => {
    expect(parseRange('≤5000')).toEqual({ max: 5000 });
  });

  it('returns empty for empty string', () => {
    expect(parseRange('')).toEqual({});
  });

  it('returns empty for garbage', () => {
    expect(parseRange('garbage')).toEqual({});
  });

  it('returns empty for whitespace', () => {
    expect(parseRange('   ')).toEqual({});
  });

  it('handles em-dash in range', () => {
    expect(parseRange('1000—5000')).toEqual({ min: 1000, max: 5000 });
  });

  it('handles en-dash in range', () => {
    expect(parseRange('1000–5000')).toEqual({ min: 1000, max: 5000 });
  });

  it('handles spaces around range', () => {
    expect(parseRange(' 1000 - 5000 ')).toEqual({ min: 1000, max: 5000 });
  });

  it('handles space after operator', () => {
    expect(parseRange('> 1000')).toEqual({ min: 1000 });
    expect(parseRange('< 5000')).toEqual({ max: 5000 });
  });
});

describe('buildSearchQuery', () => {
  it('produces params for supported fields only', () => {
    const state: FormState = {
      ...defaultFormState(),
      q: 'harry potter',
      sort: 'words',
      min_words: '1000',
    };
    const { params, notes } = buildSearchQuery(state, SUPPORTED);
    expect(params.get('q')).toBe('harry potter');
    expect(params.get('sort')).toBe('words');
    expect(params.get('min_words')).toBe('1000');
    expect(notes).toEqual([]);
  });

  it('drops inert fields and adds notes', () => {
    const state: FormState = {
      ...defaultFormState(),
      language: 'English',
      max_kudos: '500',
    };
    const { params, notes } = buildSearchQuery(state, SUPPORTED);
    // language is unsupported → not in params
    expect(params.get('language')).toBeNull();
    // max_kudos unsupported
    expect(params.get('max_kudos')).toBeNull();
    // min_bookmarks unsupported
    expect(notes.some((n) => n.includes('max_kudos'))).toBe(true);
  });

  it('parses word count range across min/max fields', () => {
    const state: FormState = {
      ...defaultFormState(),
      min_words: '1000',
      max_words: '5000',
    };
    const { params } = buildSearchQuery(state, SUPPORTED);
    expect(params.get('min_words')).toBe('1000');
    expect(params.get('max_words')).toBe('5000');
  });

  it('parses > prefix in word count', () => {
    const state: FormState = {
      ...defaultFormState(),
      min_words: '>1000',
    };
    const { params } = buildSearchQuery(state, SUPPORTED);
    expect(params.get('min_words')).toBe('1000');
  });

  it('parses < prefix in word count', () => {
    const state: FormState = {
      ...defaultFormState(),
      max_words: '<5000',
    };
    const { params } = buildSearchQuery(state, SUPPORTED);
    expect(params.get('max_words')).toBe('5000');
  });

  it('combines q with typed tag fields', () => {
    const state: FormState = {
      ...defaultFormState(),
      q: 'magic',
      fandoms: 'Harry Potter',
      characters: 'Hermione',
    };
    const { params } = buildSearchQuery(state, SUPPORTED);
    const q = params.get('q')!;
    expect(q).toContain('magic');
    expect(q).toContain('1:Harry Potter');
    expect(q).toContain('2:Hermione');
  });

  it('sets complete=true for complete status', () => {
    const state: FormState = { ...defaultFormState(), complete: 'true' };
    const { params } = buildSearchQuery(state, SUPPORTED);
    expect(params.get('complete')).toBe('true');
  });

  it('sets max_chapters=1 for single chapter checkbox', () => {
    const state: FormState = { ...defaultFormState(), single_chapter: true };
    const { params } = buildSearchQuery(state, SUPPORTED);
    expect(params.get('max_chapters')).toBe('1');
  });

  it('sets date_from', () => {
    const state: FormState = { ...defaultFormState(), date_from: '2024-01-15' };
    const { params } = buildSearchQuery(state, SUPPORTED);
    expect(params.get('date_from')).toBe('2024-01-15');
  });

  it('includes min_kudos and notes on unsupported max_kudos', () => {
    const state: FormState = {
      ...defaultFormState(),
      min_kudos: '50',
      max_kudos: '500',
    };
    const { params, notes } = buildSearchQuery(state, SUPPORTED);
    expect(params.get('min_kudos')).toBe('50');
    expect(params.get('max_kudos')).toBeNull();
    expect(notes.some((n) => n.includes('max_kudos'))).toBe(true);
  });

  it('includes min_comments and notes on unsupported max_comments', () => {
    const state: FormState = {
      ...defaultFormState(),
      min_comments: '5',
      max_comments: '100',
    };
    const { params, notes } = buildSearchQuery(state, SUPPORTED);
    expect(params.get('min_comments')).toBe('5');
    expect(params.get('max_comments')).toBeNull();
    expect(notes.some((n) => n.includes('max_comments'))).toBe(true);
  });

  it('notes on unsupported min/max bookmarks', () => {
    const state: FormState = {
      ...defaultFormState(),
      min_bookmarks: '10',
      max_bookmarks: '200',
    };
    const { params, notes } = buildSearchQuery(state, SUPPORTED);
    expect(params.get('min_bookmarks')).toBeNull();
    expect(params.get('max_bookmarks')).toBeNull();
    expect(notes.some((n) => n.includes('min_bookmarks'))).toBe(true);
    expect(notes.some((n) => n.includes('max_bookmarks'))).toBe(true);
  });

  it('sets no_warnings=true', () => {
    const state: FormState = { ...defaultFormState(), no_warnings: true };
    const { params } = buildSearchQuery(state, SUPPORTED);
    expect(params.get('no_warnings')).toBe('true');
  });

  it('sets include_tags', () => {
    const state: FormState = { ...defaultFormState(), include_tags: '1:Magic' };
    const { params } = buildSearchQuery(state, SUPPORTED);
    expect(params.get('include_tags')).toBe('1:Magic');
  });

  it('sets exclude_tags', () => {
    const state: FormState = { ...defaultFormState(), exclude_tags: '5:Violence' };
    const { params } = buildSearchQuery(state, SUPPORTED);
    expect(params.get('exclude_tags')).toBe('5:Violence');
  });

  it('sets tag_ids', () => {
    const state: FormState = { ...defaultFormState(), tag_ids: '123,456' };
    const { params } = buildSearchQuery(state, SUPPORTED);
    expect(params.get('tag_ids')).toBe('123,456');
  });

  it('respects a custom supported matrix', () => {
    const matrix: SupportedMatrix = { ...SUPPORTED, q: false };
    const state: FormState = { ...defaultFormState(), q: 'test' };
    const { params, notes } = buildSearchQuery(state, matrix);
    expect(params.get('q')).toBeNull();
    expect(notes).toContain('q is not supported by this archive');
  });

  it('adds warnings as type-5 q terms', () => {
    const state: FormState = {
      ...defaultFormState(),
      warnings: ['Major Death', 'Non-Con'],
    };
    const { params } = buildSearchQuery(state, SUPPORTED);
    const q = params.get('q')!;
    expect(q).toContain('5:Major Death');
    expect(q).toContain('5:Non-Con');
  });

  it('adds categories as type-6 q terms', () => {
    const state: FormState = {
      ...defaultFormState(),
      categories: ['F/F', 'M/M'],
    };
    const { params } = buildSearchQuery(state, SUPPORTED);
    const q = params.get('q')!;
    expect(q).toContain('6:F/F');
    expect(q).toContain('6:M/M');
  });

  it('ignores empty fields', () => {
    const state = defaultFormState();
    const { params, notes } = buildSearchQuery(state, SUPPORTED);
    expect(params.toString()).toBe('');
    expect(notes).toEqual([]);
  });
});

describe('formStateToUrl / urlToFormState round-trip', () => {
  it('round-trips a populated form state', () => {
    const state: FormState = {
      ...defaultFormState(),
      q: 'space opera',
      sort: 'words',
      dir: 'asc',
      min_words: '50000',
      max_words: '100000',
      complete: 'true',
      single_chapter: true,
      date_from: '2023-06-01',
      min_kudos: '100',
      min_comments: '10',
      tag_ids: '42',
      include_tags: '1:Star Wars',
      exclude_tags: '5:Violence',
      no_warnings: true,
      rating: 'mature',
      fandoms: 'Foundation',
      characters: 'Hari Seldon',
      relationships: 'Hari/Gaia',
      additional_tags: 'Angst',
      warnings: ['Major Death'],
      categories: ['Gen', 'Multi'],
    };
    const params = formStateToUrl(state);
    const roundTripped = urlToFormState(params);

    expect(roundTripped.q).toBe('space opera');
    expect(roundTripped.sort).toBe('words');
    expect(roundTripped.dir).toBe('asc');
    expect(roundTripped.min_words).toBe('50000');
    expect(roundTripped.max_words).toBe('100000');
    expect(roundTripped.complete).toBe('true');
    expect(roundTripped.single_chapter).toBe(true);
    expect(roundTripped.date_from).toBe('2023-06-01');
    expect(roundTripped.min_kudos).toBe('100');
    expect(roundTripped.min_comments).toBe('10');
    expect(roundTripped.tag_ids).toBe('42');
    expect(roundTripped.include_tags).toBe('1:Star Wars');
    expect(roundTripped.exclude_tags).toBe('5:Violence');
    expect(roundTripped.no_warnings).toBe(true);
    expect(roundTripped.rating).toBe('mature');
    expect(roundTripped.fandoms).toBe('Foundation');
    expect(roundTripped.characters).toBe('Hari Seldon');
    expect(roundTripped.relationships).toBe('Hari/Gaia');
    expect(roundTripped.additional_tags).toBe('Angst');
    expect(roundTripped.warnings).toEqual(['Major Death']);
    expect(roundTripped.categories).toEqual(['Gen', 'Multi']);
  });

  it('round-trips empty/default state', () => {
    const state = defaultFormState();
    const params = formStateToUrl(state);
    expect(params.toString()).toBe('');
    const roundTripped = urlToFormState(params);
    expect(roundTripped).toEqual(state);
  });

  it('handles string input', () => {
    const params = formStateToUrl({ ...defaultFormState(), q: 'hello' });
    const roundTripped = urlToFormState(params.toString());
    expect(roundTripped.q).toBe('hello');
  });

  it('handles multi-value array round-trip', () => {
    const state: FormState = {
      ...defaultFormState(),
      warnings: ['Major Death', 'Non-Con', 'Underage'],
      categories: ['F/F', 'M/M'],
    };
    const params = formStateToUrl(state);
    const roundTripped = urlToFormState(params);
    expect(roundTripped.warnings).toEqual(['Major Death', 'Non-Con', 'Underage']);
    expect(roundTripped.categories).toEqual(['F/F', 'M/M']);
  });
});

describe('mergeChipIntoForm', () => {
  it('adds a fandom chip (type 1) to fandoms field', () => {
    const state = defaultFormState();
    const chip: Chip = { tag_type_id: 1, name: 'Harry Potter' };
    const next = mergeChipIntoForm(state, chip);
    expect(next.fandoms).toBe('Harry Potter');
    // original not mutated
    expect(state.fandoms).toBe('');
  });

  it('appends to existing fandoms', () => {
    const state: FormState = { ...defaultFormState(), fandoms: 'Foundation' };
    const chip: Chip = { tag_type_id: 1, name: 'Dune' };
    const next = mergeChipIntoForm(state, chip);
    expect(next.fandoms).toBe('Foundation, Dune');
  });

  it('adds a character chip (type 2)', () => {
    const state = defaultFormState();
    const chip: Chip = { tag_type_id: 2, name: 'Hermione Granger' };
    const next = mergeChipIntoForm(state, chip);
    expect(next.characters).toBe('Hermione Granger');
  });

  it('adds a relationship chip (type 3)', () => {
    const state = defaultFormState();
    const chip: Chip = { tag_type_id: 3, name: 'Harry/Draco' };
    const next = mergeChipIntoForm(state, chip);
    expect(next.relationships).toBe('Harry/Draco');
  });

  it('adds an additional tag chip (type 4)', () => {
    const state = defaultFormState();
    const chip: Chip = { tag_type_id: 4, name: 'Time Travel' };
    const next = mergeChipIntoForm(state, chip);
    expect(next.additional_tags).toBe('Time Travel');
  });

  it('adds a warning chip (type 5) to warnings array', () => {
    const state = defaultFormState();
    const chip: Chip = { tag_type_id: 5, name: 'Major Death' };
    const next = mergeChipIntoForm(state, chip);
    expect(next.warnings).toEqual(['Major Death']);
  });

  it('does not duplicate a warning chip', () => {
    const state: FormState = { ...defaultFormState(), warnings: ['Major Death'] };
    const chip: Chip = { tag_type_id: 5, name: 'Major Death' };
    const next = mergeChipIntoForm(state, chip);
    expect(next.warnings).toEqual(['Major Death']);
  });

  it('adds a category chip (type 6) to categories array', () => {
    const state = defaultFormState();
    const chip: Chip = { tag_type_id: 6, name: 'F/M' };
    const next = mergeChipIntoForm(state, chip);
    expect(next.categories).toEqual(['F/M']);
  });

  it('handles unknown tag type by falling back to include_tags', () => {
    const state = defaultFormState();
    const chip: Chip = { tag_type_id: 99, name: 'Unknown Tag' };
    const next = mergeChipIntoForm(state, chip);
    expect(next.include_tags).toBe('Unknown Tag');
  });

  it('does not mutate the original state', () => {
    const state: FormState = {
      ...defaultFormState(),
      fandoms: 'Foundation',
      warnings: ['Major Death'],
      categories: ['Gen'],
    };
    const chip: Chip = { tag_type_id: 1, name: 'Dune' };
    mergeChipIntoForm(state, chip);
    expect(state.fandoms).toBe('Foundation');
    expect(state.warnings).toEqual(['Major Death']);
    expect(state.categories).toEqual(['Gen']);
  });
});
