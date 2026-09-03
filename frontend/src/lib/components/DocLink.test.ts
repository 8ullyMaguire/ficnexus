// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import DocLink from './DocLink.svelte';

const mocks = vi.hoisted(() => ({
  openDoc: vi.fn(),
}));

vi.mock('$lib/stores/doc-help.svelte', () => ({
  openDoc: mocks.openDoc,
}));

describe('DocLink', () => {
  beforeEach(() => {
    mocks.openDoc.mockReset();
  });

  it('renders a link to the docs page with the anchor', () => {
    render(DocLink, { props: { slug: 'searching#boolean-query-syntax' } });
    const a = screen.getByTitle('View docs');
    expect(a.getAttribute('href')).toBe('/docs/searching.html#boolean-query-syntax');
  });

  it('opens the help modal on plain click (SPA state preserved)', async () => {
    render(DocLink, { props: { slug: 'searching#boolean-query-syntax' } });
    await fireEvent.click(screen.getByTitle('View docs'));
    expect(mocks.openDoc).toHaveBeenCalledWith('searching#boolean-query-syntax');
  });

  it('does not open the modal on ctrl+click (full page wins)', async () => {
    render(DocLink, { props: { slug: 'searching#boolean-query-syntax' } });
    await fireEvent.click(screen.getByTitle('View docs'), { ctrlKey: true });
    expect(mocks.openDoc).not.toHaveBeenCalled();
  });

  it('supports a custom label + title', () => {
    render(DocLink, { props: { slug: 'faq', label: '📲', title: 'E-reader help', inline: true } });
    expect(screen.getByText('📲')).toBeTruthy();
    expect(screen.getByTitle('E-reader help')).toBeTruthy();
  });
});
