// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import HelpModal from './HelpModal.svelte';

const mocks = vi.hoisted(() => ({
  docHelpState: { active: null as unknown, loading: false, error: '' },
  closeDoc: vi.fn(),
  loadDocSection: vi.fn(),
}));

vi.mock('$lib/stores/doc-help.svelte', () => ({
  docHelpState: mocks.docHelpState,
  closeDoc: mocks.closeDoc,
  loadDocSection: mocks.loadDocSection,
}));

function stubState(active: unknown) {
  mocks.docHelpState.active = active;
}

describe('HelpModal', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    vi.stubGlobal('fetch', vi.fn());
  });

  it('renders nothing when no doc is active', () => {
    stubState(null);
    render(HelpModal);
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('renders the section content when a doc is active', () => {
    stubState({ title: 'Search Syntax', page: 'searching.html', anchor: 'boolean-query-syntax', html: '<p>Boolean operators…</p>' });
    render(HelpModal);
    expect(screen.getByRole('dialog')).toBeTruthy();
    expect(screen.getByText('Boolean operators…')).toBeTruthy();
  });

  it('asks the docs and shows results, opening the chosen section', async () => {
    stubState({ title: 'Search Syntax', page: 'searching.html', anchor: 'boolean-query-syntax', html: '<p>x</p>' });
    const fetchMock = vi.fn().mockResolvedValue({
      json: async () => ({
        err: 0,
        results: [
          { slug: 'searching#try-it-now', page: 'searching.html', anchor: 'try-it-now', title: 'Try it now', snippet: 'dark harry', score: 0.8 },
        ],
      }),
    });
    vi.stubGlobal('fetch', fetchMock);
    mocks.loadDocSection.mockResolvedValue(null);
    render(HelpModal);

    const input = screen.getByLabelText('Ask the docs');
    await fireEvent.input(input, { target: { value: 'dark harry' } });
    await fireEvent.keyDown(input, { key: 'Enter' });

    // wait for the result button
    expect(fetchMock).toHaveBeenCalledWith(expect.stringContaining('/api/docs/ask?q=dark%20harry'), expect.anything());
    const result = await screen.findByText(/Try it now/);
    expect(result).toBeTruthy();
    await fireEvent.click(result);
    expect(mocks.loadDocSection).toHaveBeenCalledWith('searching#try-it-now');
  });
});
