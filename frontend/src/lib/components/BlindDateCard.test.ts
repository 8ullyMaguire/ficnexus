import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  sessionStorage.clear();
});

async function loadCard() {
  return await import('$lib/components/BlindDateCard.svelte');
}

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

describe('BlindDateCard', () => {
  it('does not render the title or fandom in the DOM before reveal', async () => {
    // The discovery payload deliberately omits title/author/source/fandom.
    mockFetch.mockResolvedValueOnce(
      jsonResponse({
        err: 0,
        fic: {
          url_id: 'fic1',
          description: 'A brave knight fights a dragon.',
          words: 5000,
          chapters: 5,
          status: 'complete',
          tropes: ['BlindTest Fluff', 'BlindTest Angst'],
          reveal: { nonce: 'n1', sig: 's1' },
        },
      }),
    );

    const { default: Card } = await loadCard();
    render(Card);

    // Draw a fic (the card does not auto-draw on mount).
    const drawBtn = screen.getByRole('button', { name: /draw a fic/i });
    await fireEvent.click(drawBtn);

    await waitFor(() => {
      expect(screen.getByText(/a brave knight fights a dragon/i)).toBeTruthy();
    });
    // The summary is visible…
    expect(screen.getByText(/5,000 words/i)).toBeTruthy();
    // …but the hidden fields are NOT in the DOM (they were never in the
    // network response either — the fixture has no title/fandom keys).
    expect(screen.queryByText('Secret Title')).toBeNull();
    expect(screen.queryByText('Secret Fandom')).toBeNull();
    // The mystery placeholder is visible until reveal.
    expect(screen.getByText('???')).toBeTruthy();
  });

  it('fetches and shows the title + fandom only after clicking Reveal', async () => {
    mockFetch
      // Draw
      .mockResolvedValueOnce(
        jsonResponse({
          err: 0,
          fic: {
            url_id: 'fic1',
            description: 'A brave knight fights a dragon.',
            words: 5000,
            chapters: 5,
            status: 'complete',
            tropes: ['BlindTest Fluff'],
            reveal: { nonce: 'n1', sig: 's1' },
          },
        }),
      )
      // Reveal — returns the hidden fields
      .mockResolvedValueOnce(
        jsonResponse({
          err: 0,
          fic: {
            url_id: 'fic1',
            title: 'Secret Title',
            author: 'Secret Author',
            source: 'https://example.com/fic1',
            fandom: 'Secret Fandom',
          },
        }),
      );

    const { default: Card } = await loadCard();
    render(Card);

    await fireEvent.click(screen.getByRole('button', { name: /draw a fic/i }));
    await waitFor(() => {
      expect(screen.getByText(/a brave knight fights a dragon/i)).toBeTruthy();
    });

    // Before clicking: no title/fandom anywhere.
    expect(screen.queryByText('Secret Title')).toBeNull();
    expect(screen.queryByText('Secret Fandom')).toBeNull();

    // Click Reveal → the card fetches the reveal endpoint and shows them.
    const revealBtn = screen.getByRole('button', { name: /reveal/i });
    await fireEvent.click(revealBtn);

    await waitFor(() => {
      expect(screen.getByText('Secret Title')).toBeTruthy();
      // The fandom renders as "📁 Secret Fandom" (emoji prefix) — match
      // with a regex that ignores the icon.
      expect(screen.getByText(/secret fandom/i)).toBeTruthy();
    });

    // The reveal request carried the signed handle.
    const revealCall = mockFetch.mock.calls.find((c) =>
      String(c[0]).includes('/api/blind-date/reveal'),
    );
    expect(revealCall).toBeTruthy();
    expect(String(revealCall![0])).toContain('url_id=fic1');
    expect(String(revealCall![0])).toContain('nonce=n1');
    expect(String(revealCall![0])).toContain('sig=s1');
  });

  it('passes seen url_ids as exclude so the server never repeats', async () => {
    mockFetch
      .mockResolvedValueOnce(
        jsonResponse({
          err: 0,
          fic: {
            url_id: 'fic1',
            description: 'First fic.',
            words: 1000,
            chapters: 1,
            status: 'complete',
            tropes: [],
            reveal: { nonce: 'n1', sig: 's1' },
          },
        }),
      )
      .mockResolvedValueOnce(
        jsonResponse({
          err: 0,
          fic: {
            url_id: 'fic2',
            description: 'Second fic.',
            words: 2000,
            chapters: 2,
            status: 'ongoing',
            tropes: [],
            reveal: { nonce: 'n2', sig: 's2' },
          },
        }),
      );

    const { default: Card } = await loadCard();
    render(Card);

    await fireEvent.click(screen.getByRole('button', { name: /draw a fic/i }));
    await waitFor(() => {
      expect(screen.getByText(/first fic/i)).toBeTruthy();
    });

    const anotherBtn = screen.getByRole('button', { name: /another/i });
    await fireEvent.click(anotherBtn);

    await waitFor(() => {
      expect(screen.getByText(/second fic/i)).toBeTruthy();
    });

    // The second draw requested exclude=fic1.
    const drawCalls = mockFetch.mock.calls.filter((c) =>
      String(c[0]).includes('/api/blind-date'),
    );
    expect(drawCalls.length).toBe(2);
    expect(String(drawCalls[0][0])).not.toContain('exclude=');
    expect(String(drawCalls[1][0])).toContain('exclude=fic1');
  });
});
