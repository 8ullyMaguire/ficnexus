import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { readFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';

// Docs hub: every link must point at a real built mdbook page served from
// frontend/static/docs (no dead external links).
const source = readFileSync('src/routes/docs/+page.svelte', 'utf8');
const hrefs = [...source.matchAll(/href: '([^']+)'/g)].map((m) => m[1]);

describe('/docs hub', () => {
  it('links only to in-app pages', () => {
    expect(hrefs.length).toBeGreaterThan(10);
    for (const h of hrefs) expect(h.startsWith('/docs/')).toBe(true);
  });

  it('every linked book page exists on disk', () => {
    const staticDir = join('static', 'docs');
    for (const h of hrefs) {
      const file = h.replace('/docs/', '');
      expect(existsSync(join(staticDir, file)), `missing built page: ${h}`).toBe(true);
    }
  });

  it('renders the section headings and quick links', async () => {
    const { default: Page } = await import('./+page.svelte');
    render(Page);
    expect(screen.getByText('Start Here')).toBeTruthy();
    expect(screen.getByText('User Guide')).toBeTruthy();
    expect(screen.getByText('Contributing')).toBeTruthy();
    expect(screen.getAllByRole('link', { name: 'Quick Start' }).length).toBeGreaterThan(0);
  });
});
