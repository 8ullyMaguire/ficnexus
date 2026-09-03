import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';

// MarginaliaComposerModal: renders the pre-filled title + blockquote body and
// POSTs a marginalia topic to /api/forum/topics on submit.
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

vi.mock('$lib/prefs', () => ({
  getPref: vi.fn(() => 'modern'),
}));

vi.mock('$lib/api/social', () => ({
  authHeaders: () => ({}),
}));

function okJson(body: unknown) {
  return { ok: true, status: 200, json: async () => body };
}

const baseProps = {
  passageText: 'The sky above the port was the color of television, tuned to a dead channel.',
  ficTitle: 'Neuromancer',
  chapterIndex: 2,
  categorySlug: 'general',
};

async function mountModal(overrides: { onClose?: () => void; onCreated?: (topicId: number, topicSlug?: string | null) => void } = {}) {
  const { default: Modal } = await import('./MarginaliaComposerModal.svelte');
  const props = { ...baseProps, ...overrides };
  return render(Modal, {
    open: true,
    payload: {
      type: 'marginalia',
      work_id: 42,
      url_id: 'neuro-abc',
      chapter_index: props.chapterIndex,
      passage_hash: 'deadbeef1234',
      passage_text: props.passageText,
    },
    onClose: overrides.onClose ?? (() => {}),
    onCreated: overrides.onCreated ?? (() => {}),
    ...props,
  });
}

beforeEach(() => {
  mockFetch.mockReset();
});

describe('MarginaliaComposerModal', () => {
  it('renders with pre-filled title in the canonical format', async () => {
    await mountModal();
    // Title format: Discussion: "<40-char excerpt>…" in {title} - Ch {n}
    const input = screen.getByDisplayValue(
      /Discussion: "The sky above the port was the color of…" in Neuromancer - Ch 3/,
    ) as HTMLInputElement;
    expect(input.disabled).toBe(true);
    expect(screen.getByText(baseProps.passageText)).toBeTruthy();
  });

  it('builds a blockquote body and POSTs payload.type=marginalia on submit', async () => {
    const onCreated = vi.fn();
    const onClose = vi.fn();
    mockFetch.mockResolvedValue(okJson({ err: 0, id: 777, topic_slug: 'discussion-777' }));
    await mountModal({ onCreated, onClose });

    const textarea = screen.getByPlaceholderText('Add your comment…') as HTMLTextAreaElement;
    await fireEvent.input(textarea, { target: { value: 'Great line.' } });

    await fireEvent.click(screen.getByRole('button', { name: 'Start discussion' }));

    await waitFor(() => expect(onCreated).toHaveBeenCalled());
    expect(onCreated).toHaveBeenCalledWith(777, 'discussion-777');
    expect(onClose).toHaveBeenCalled();

    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/forum/topics');
    expect(init.method).toBe('POST');
    const sent = JSON.parse(init.body);
    expect(sent.title.startsWith('Discussion: "')).toBe(true);
    expect(sent.category_slug).toBe('general');
    // Blockquote body: quoted passage lines then user comment
    expect(sent.body).toContain('> The sky above the port');
    expect(sent.body).toContain('> — *Chapter 3, Neuromancer*');
    expect(sent.body.endsWith('\n\nGreat line.')).toBe(true);
    // Payload forwarded verbatim with type=marginalia
    expect(sent.payload.type).toBe('marginalia');
    expect(sent.payload.work_id).toBe(42);
    expect(sent.payload.passage_hash).toBe('deadbeef1234');
    expect(sent.payload.url_id).toBe('neuro-abc');
  });

  it('shows an error message when the API rejects', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 1, msg: 'Level 5 required for marginalia' }));
    await mountModal();
    await fireEvent.click(screen.getByRole('button', { name: 'Start discussion' }));
    await waitFor(() => expect(screen.getByText(/Level 5 required/)).toBeTruthy());
  });
});
