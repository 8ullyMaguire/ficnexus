// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import Page from './+page.svelte';

const mocks = vi.hoisted(() => ({
  adminFetch: vi.fn(),
}));

vi.mock('$lib/api/admin', () => ({
  adminFetch: mocks.adminFetch,
}));

describe('admin metadata correction page', () => {
  beforeEach(() => {
    mocks.adminFetch.mockReset();
    // global.fetch for the work lookup
    vi.stubGlobal('fetch', vi.fn(async () => ({
      json: async () => ({ err: 0, work: { id: 42, title: 'Original Title', author: 'Author', status: 'ongoing', description: 'Original desc' } }),
    })));
  });

  it('renders the search UI and load button', () => {
    render(Page);
    expect(screen.getByPlaceholderText(/work id or search query/)).toBeTruthy();
    expect(screen.getByText(/🔍 Load work/)).toBeTruthy();
  });

  it('shows the form with current metadata after loading a work', async () => {
    render(Page);
    const input = screen.getByPlaceholderText(/work id or search query/) as HTMLInputElement;
    await fireEvent.input(input, { target: { value: '42' } });
    // jsdom does not perform implicit submission on submit-button clicks —
    // fire the form's submit event directly (the page handles `onsubmit`).
    const form = input.closest('form')!;
    await fireEvent.submit(form);
    // The form appears after the fetch resolves
    await screen.findByText(/Work #42/);
    expect(screen.getByDisplayValue('Original Title')).toBeTruthy();
    expect(screen.getByText(/💾 Save correction/)).toBeTruthy();
  });

  it('shows an error when the work is not found', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({
      json: async () => ({ err: 1, msg: 'not found' }),
    })));
    render(Page);
    const input = screen.getByPlaceholderText(/work id or search query/) as HTMLInputElement;
    await fireEvent.input(input, { target: { value: '999' } });
    const form = input.closest('form')!;
    await fireEvent.submit(form);
    await screen.findByText(/Work not found by that id or query/);
  });
});
