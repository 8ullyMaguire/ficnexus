import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';

// /fic/[urlId] is a permanent redirect shim to /works/[urlId] (+page.ts).
// The component itself only renders a "Redirecting" notice.

describe('fic/[urlId] redirect shim', () => {
  it('renders the redirect notice', async () => {
    const { default: Page } = await import('./+page.svelte');
    render(Page);
    expect(screen.getByText(/Redirecting/i)).toBeTruthy();
  });
});
