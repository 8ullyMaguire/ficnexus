// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import MetadataEditModal from './MetadataEditModal.svelte';

const mocks = vi.hoisted(() => ({
  // F7: admin role (10) → effective level 100 so the level ≥ 100 gate renders.
  auth: { isLoggedIn: true, user: { role: 10, level: 100 }, level: 100 },
}));

vi.mock('$lib/stores/auth.svelte', () => ({ auth: mocks.auth }));

const PROPS = {
  workId: 42,
  urlId: 'ao3_12345',
  currentTitle: 'Old Title',
  currentAuthor: 'Author',
  currentStatus: 'ongoing',
  currentDescription: 'Old desc',
  onClose: () => {},
};

describe('MetadataEditModal', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    vi.stubGlobal('fetch', vi.fn());
  });

  it('renders the field tabs with current values', () => {
    render(MetadataEditModal, { props: PROPS });
    expect(screen.getByText('Title')).toBeTruthy();
    expect(screen.getByText('Author')).toBeTruthy();
    expect(screen.getByText('Status')).toBeTruthy();
    expect(screen.getByText('Description')).toBeTruthy();
    // Default active field is Title → its current value shows
    expect(screen.getByText(/Old Title/)).toBeTruthy();
  });

  it('posts a proposal when the curator submits', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      json: async () => ({ err: 0, proposal_id: 7 }),
    });
    vi.stubGlobal('fetch', fetchMock);
    render(MetadataEditModal, { props: PROPS });

    const input = screen.getByPlaceholderText('New title…');
    await fireEvent.input(input, { target: { value: 'New Correct Title' } });
    await fireEvent.click(screen.getByText('Send to vote queue'));

    expect(fetchMock).toHaveBeenCalledWith('/api/curator/metadata/propose', expect.anything());
    const [, init] = fetchMock.mock.calls[0];
    const body = JSON.parse(init.body);
    expect(body.work_id).toBe(42);
    expect(body.field).toBe('title');
    expect(body.old_value).toBe('Old Title');
    expect(body.new_value).toBe('New Correct Title');
    // Success message with proposal id
    expect(await screen.findByText(/pending curator vote \(proposal #7\)/)).toBeTruthy();
  });

  it('does not render for non-admins (curator level < 100)', () => {
    mocks.auth.user = { role: 5, level: 50 };
    mocks.auth.level = 50;
    const { container } = render(MetadataEditModal, { props: PROPS });
    expect(container.querySelector('.md-backdrop')).toBeNull();
    mocks.auth.user = { role: 10, level: 100 };
    mocks.auth.level = 100;
  });
});
