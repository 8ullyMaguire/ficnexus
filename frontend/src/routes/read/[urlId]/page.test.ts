import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(async () => {
  // Reset the auth singleton so each test re-runs auth.init() → getMe.
  // (init() short-circuits once `initialized` is true, which leaks the
  // logged-in user from the previous test.)
  const { auth } = await import('$lib/stores/auth.svelte');
  auth.initialized = false;
  auth.user = null;
  mockFetch.mockReset();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
  localStorage.clear();
  // jsdom defaults: window innerHeight is 768 — force scroll math to be sane.
  Object.defineProperty(window, 'scrollY', { value: 0, writable: true, configurable: true });
  // The reader's scroll-progress handler calls window.scrollTo on scroll;
  // jsdom doesn't implement it.
  window.scrollTo = vi.fn() as unknown as typeof window.scrollTo;
});

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

/** Default reader payload with 2 chapters, matching the /api/reader contract. */
function readerPayload(overrides: Record<string, unknown> = {}) {
  return {
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
    ...overrides,
  };
}

async function loadPage() {
  return await import('./+page.svelte');
}

/** Mount the reader with the standard fetch sequence. */
async function mountReader(extraFetches: ((url: string) => unknown)[] = []) {
  // auth.init() → getMe (the reader requires a session)
  mockFetch.mockResolvedValueOnce(
    jsonResponse({ err: 0, user: { id: 1, username: 'alice', role: 0, reputation: 0 } })
  );
  // loadReaderData(urlId)
  mockFetch.mockResolvedValueOnce(jsonResponse(readerPayload()));

  const { default: Reader } = await loadPage();
  render(Reader, { props: { data: { urlId: 'fic123' } } });

  await waitFor(() => {
    // Wait for the CHAPTER content (renders only after work is loaded) —
    // the title/header can render before `work` is assigned, which would
    // make savePosition() early-return (`if (!work) return`).
    expect(screen.getByText('Prologue')).toBeTruthy();
  });
}

describe('web reader page', () => {
  it('renders fic content with chapter navigation', async () => {
    await mountReader();

    expect(screen.getByText('Prologue')).toBeTruthy();
    expect(screen.getByText(/It began with a whisper/i)).toBeTruthy();
    // Chapter 1 of 2 + per-chapter word count
    expect(screen.getByText(/Chapter 1 of 2/)).toBeTruthy();
    // Prev is ENABLED at the first chapter of a multi-chapter fic (it
    // navigates back to the fic page); Next is enabled too (top + bottom nav
    // both render the same controls; assert on the first).
    const prev = screen.getAllByRole('button', { name: /← Previous/i })[0];
    expect((prev as HTMLButtonElement).disabled).toBe(false);
    expect(screen.getAllByRole('button', { name: /Next ▸/i })[0]).toBeTruthy();
  });

  it('navigates between chapters with the Next button and keyboard arrows', async () => {
    await mountReader();

    const next = screen.getAllByRole('button', { name: /Next ▸/i })[0];
    await fireEvent.click(next);
    expect(screen.getByText('Chapter Two')).toBeTruthy();
    expect(screen.getByText(/Chapter 2 of 2/)).toBeTruthy();
    expect(screen.getByText(/The storm rolled in/i)).toBeTruthy();

    // ArrowLeft goes back to chapter 1
    await fireEvent.keyDown(window, { key: 'ArrowLeft' });
    expect(screen.getByText('Prologue')).toBeTruthy();
    expect(screen.getByText(/Chapter 1 of 2/)).toBeTruthy();

    // ArrowRight moves forward again
    await fireEvent.keyDown(window, { key: 'ArrowRight' });
    expect(screen.getByText('Chapter Two')).toBeTruthy();
  });

  it('persists typography preferences to localStorage and restores them', async () => {
    await mountReader();

    // Bump font size via the toolbar button (aria-label is the accessible name)
    const plus = screen.getByRole('button', { name: /Increase font size/i });
    await fireEvent.click(plus);
    expect(screen.getByText('20px')).toBeTruthy();

    // Cycle theme light → sepia → dark
    const themeBtn = screen.getByRole('button', { name: /cycle theme/i });
    await fireEvent.click(themeBtn);
    await fireEvent.click(themeBtn);
    await fireEvent.click(themeBtn); // light → sepia → dark → light
    // toggle indent
    const indentBtn = screen.getByRole('button', { name: /toggle paragraph indent/i });
    await fireEvent.click(indentBtn);

    const saved = JSON.parse(localStorage.getItem('fichub:reader:state:fic123')!);
    expect(saved.fontSize).toBe(20);
    expect(saved.chapterIndex).toBe(0);
    expect(saved.finished).toBe(false);

    // Simulate a fresh visit: remount with prefs pre-seeded.
    mockFetch.mockResolvedValueOnce(
      jsonResponse({ err: 0, user: { id: 1, username: 'alice', role: 0, reputation: 0 } })
    );
    mockFetch.mockResolvedValueOnce(jsonResponse(readerPayload()));
    localStorage.setItem(
      'fichub:reader:state:fic123',
      JSON.stringify({ fontSize: 24, theme: 'sepia', indentParagraphs: false, chapterIndex: 0 })
    );
    const { default: Reader } = await loadPage();
    render(Reader, { props: { data: { urlId: 'fic123' } } });
    await waitFor(() => {
      expect(screen.getByText('Prologue')).toBeTruthy();
    });
    expect(screen.getByText('24px')).toBeTruthy();
    expect(screen.getByText('🕯️')).toBeTruthy(); // sepia icon
  });

  it('saves reading position on scroll (progress bar updates)', async () => {
    await mountReader();

    // Simulate scrolling to the bottom of a tall document
    Object.defineProperty(window, 'scrollY', { value: 500, writable: true, configurable: true });
    Object.defineProperty(document.documentElement, 'scrollHeight', {
      value: 1000,
      writable: true,
      configurable: true,
    });
    Object.defineProperty(window, 'innerHeight', { value: 500, writable: true, configurable: true });
    // Wait for onMount's async chain (work load + listener attach in the finally block).
    await new Promise((r) => setTimeout(r, 50));
    await fireEvent.scroll(window);

    const saved = JSON.parse(localStorage.getItem('fichub:reader:state:fic123')!);
    expect(saved.scrollPos).toBe(500);

    // Progress indicator reflects scroll (500/500 → 100%)
    expect(screen.getByText('100%')).toBeTruthy();
  });

  it('marks the fic completed and persists the finished flag', async () => {
    await mountReader();

    const markBtn = screen.getByRole('button', { name: /mark as completed/i });
    await fireEvent.click(markBtn);

    await waitFor(() => {
      expect(screen.getByText('Completed')).toBeTruthy();
    });
    const saved = JSON.parse(localStorage.getItem('fichub:reader:state:fic123')!);
    expect(saved.finished).toBe(true);
  });

  it('shows the Next Up panel on the last chapter and fetches candidates', async () => {
    // URL-routed mocks: the page fires /api/reading/status on chapter nav,
    // which would consume a positional Once-queue and misalign the rest.
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/auth/me')) {
        return Promise.resolve(jsonResponse({ err: 0, user: { id: 1, username: 'alice', role: 0, reputation: 0 } }));
      }
      if (url.includes('/api/reader/fic123') && !url.includes('sequel') && !url.includes('related') && !url.includes('meta') && !url.includes('sequential')) {
        return Promise.resolve(jsonResponse(readerPayload()));
      }
      if (url.includes('/sequel')) {
        return Promise.resolve(jsonResponse({ err: 0, sequel: { url_id: 'fic456', title: 'The Awakening II', author: 'Alice' } }));
      }
      if (url.includes('/votes')) {
        return Promise.resolve(jsonResponse({
          err: 0,
          url_id: 'fic123',
          suggestions: [{ id: 1, suggested_url_id: 'fic789', comment: 'Must read', net_votes: 5 }],
        }));
      }
      if (url.includes('/meta')) {
        return Promise.resolve(jsonResponse({ err: 0, meta: { title: 'Community Pick', author: 'Bob' } }));
      }
      if (url.includes('/related')) {
        return Promise.resolve(jsonResponse({ err: 0, related: [{ url_id: 'fic111', title: 'Also Bookmarked', author: 'Carol' }] }));
      }
      if (url.includes('/sequential')) {
        return Promise.resolve(jsonResponse({ err: 0, sequential: [{ url_id: 'fic222', title: 'Markov Pick', author: 'Dave', score: 9.5, reason: 'readers typically continue with this' }] }));
      }
      return Promise.resolve(jsonResponse({ err: 0 }));
    });

    const { default: Reader } = await loadPage();
    render(Reader, { props: { data: { urlId: 'fic123' } } });

    await waitFor(() => {
      expect(screen.getByText('Prologue')).toBeTruthy();
    });

    // Jump to the last chapter (2 of 2)
    await fireEvent.click(screen.getAllByRole('button', { name: /Next ▸/i })[0]);
    await waitFor(() => expect(screen.getByText('Chapter Two')).toBeTruthy());

    // "Finish ▸" on the last chapter opens the panel
    await fireEvent.click(screen.getAllByRole('button', { name: /Finish ▸/i })[0]);

    await waitFor(() => {
      expect(screen.getByText('Next Up')).toBeTruthy();
    });
    // Sequel first (buildNextUp's fetches resolve async — wait for them)
    await waitFor(() => {
      expect(screen.getByText('Next in series')).toBeTruthy();
    });
    expect(screen.getByText('The Awakening II')).toBeTruthy();
    // Community suggestion second
    expect(screen.getByText('Top community suggestion')).toBeTruthy();
    expect(screen.getByText('Community Pick')).toBeTruthy();
    // Readers-also-bookmarked third
    expect(screen.getByText('Readers also bookmarked')).toBeTruthy();
    expect(screen.getByText('Also Bookmarked')).toBeTruthy();
    // Sequential Markov fourth (logged-in path)
    expect(screen.getByText('readers typically continue with this')).toBeTruthy();
    expect(screen.getByText('Markov Pick')).toBeTruthy();
  });

  it('shows Next Up sequential suggestions when sequel/related empty', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/auth/me')) {
        return Promise.resolve(jsonResponse({ err: 0, user: { id: 1, username: 'alice', role: 0, reputation: 0 } }));
      }
      if (url.includes('/api/reader/fic123') && !url.includes('sequel') && !url.includes('related') && !url.includes('meta') && !url.includes('sequential')) {
        return Promise.resolve(jsonResponse(readerPayload()));
      }
      if (url.includes('/locales')) {
        return Promise.resolve(jsonResponse({ err: 0, locales: [] }));
      }
      // sequel + community + related return empty; sequential has a pick
      if (url.includes('/sequential')) {
        return Promise.resolve(jsonResponse({ err: 0, sequential: [{ url_id: 'seq1', title: 'Markov Read', author: 'Dan', score: 7.2, reason: 'readers typically continue with this' }] }));
      }
      return Promise.resolve(jsonResponse({ err: 0 }));
    });

    const { default: Reader } = await loadPage();
    render(Reader, { props: { data: { urlId: 'fic123' } } });

    await waitFor(() => {
      expect(screen.getByText('Prologue')).toBeTruthy();
    });

    await fireEvent.click(screen.getAllByRole('button', { name: /Next ▸/i })[0]);
    await fireEvent.click(screen.getAllByRole('button', { name: /Finish ▸/i })[0]);

    await waitFor(() => {
      expect(screen.getByText('Next Up')).toBeTruthy();
    });
    // With no sequel/community/related, the sequential Markov pick fills the panel.
    await waitFor(() => {
      expect(screen.getByText('Markov Read')).toBeTruthy();
    });
    expect(screen.getByText('readers typically continue with this')).toBeTruthy();
  });

  it('falls back to empty state when no candidates resolve (incl. sequential)', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/auth/me')) {
        return Promise.resolve(jsonResponse({ err: 0, user: { id: 1, username: 'alice', role: 0, reputation: 0 } }));
      }
      if (url.includes('/api/reader/fic123') && !url.includes('sequel') && !url.includes('related') && !url.includes('meta')) {
        return Promise.resolve(jsonResponse(readerPayload()));
      }
      if (url.includes('/locales')) {
        return Promise.resolve(jsonResponse({ err: 0, locales: [] }));
      }
      // All candidate fetches return empty
      return Promise.resolve(jsonResponse({ err: 0 }));
    });

    const { default: Reader } = await loadPage();
    render(Reader, { props: { data: { urlId: 'fic123' } } });

    await waitFor(() => {
      expect(screen.getByText('Prologue')).toBeTruthy();
    });

    await fireEvent.click(screen.getAllByRole('button', { name: /Next ▸/i })[0]);
    await fireEvent.click(screen.getAllByRole('button', { name: /Finish ▸/i })[0]);

    await waitFor(() => {
      expect(screen.getByText('Next Up')).toBeTruthy();
    });
    expect(screen.getByText(/No recommendations yet/i)).toBeTruthy();
  });

  it('lists available locales and switches the translation content', async () => {
    // The reader's onMount fetches auth + reader payload + kudos + bookmarks
    // + comments + /locales (in that order). Mock the ones we care about by
    // URL so positional order doesn't matter; kudos/bookmarks/comments come
    // back empty to skip errors.
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) {
        return Promise.resolve(jsonResponse({ err: 0, user: { id: 1, username: 'alice', role: 0, reputation: 0 } }));
      }
      if (url.includes('/api/reader/fic123') && !url.includes('/meta') && !url.includes('/sequel') && !url.includes('/related')) {
        return Promise.resolve(jsonResponse(readerPayload()));
      }
      if (url === '/api/locales') {
        return Promise.resolve(jsonResponse({
          err: 0,
          locales: [
            { code: 'en', name: 'English', is_rtl: false },
            { code: 'es', name: 'Español', is_rtl: false },
          ],
        }));
      }
      if (url.includes('/api/kudos') || url.includes('/api/bookmarks') || url.includes('/api/comments')) {
        return Promise.resolve(jsonResponse({ err: 0 }));
      }
      return Promise.resolve(jsonResponse({ err: 0 }));
    });

    const { default: Reader } = await loadPage();
    render(Reader, { props: { data: { urlId: 'fic123' } } });

    await waitFor(() => {
      expect(screen.getByText('Prologue')).toBeTruthy();
    });

    // The locale picker button appears in the toolbar (shows current locale).
    // The /api/locales fetch resolves after the reader content, so wait for it.
    const picker = await screen.findByRole('button', { name: /Choose translation language/ });
    expect(picker.textContent?.trim()).toBe('en');

    await fireEvent.click(picker);

    // Dropdown lists the locales; selecting Español fetches the translation.
    await waitFor(() => {
      expect(screen.getByRole('option', { name: 'Español' })).toBeTruthy();
    });

    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/chapter-translations/es')) {
        return Promise.resolve(jsonResponse({
          err: 0,
          work_id: 7,
          locale_code: 'es',
          chapters: [{ chapter_num: 1, title: 'Prólogo', content_html: '<p>Traducción.</p>' }],
        }));
      }
      return Promise.resolve(jsonResponse({ err: 0 }));
    });

    await fireEvent.click(screen.getByRole('option', { name: 'Español' }));

    await waitFor(() => {
      expect(screen.getByText('Traducción.')).toBeTruthy();
    });
    // The saved locale is persisted so future visits open on the translation.
    expect(localStorage.getItem('fichub_locale')).toBe('es');
  });
});
