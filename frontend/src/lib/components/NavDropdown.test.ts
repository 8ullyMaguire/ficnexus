import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';

// NavDropdown is a pure presentational dropdown: open/close on trigger,
// Escape to close, click-outside to close, and item-click close.
async function loadNavDropdown() {
  return await import('$lib/components/NavDropdown.svelte');
}

// A real Svelte 5 snippet (createRawSnippet) — a plain arrow function is not
// a Snippet and {@render} ignores it.
const dropdownItems = createRawSnippet(() => ({
  render: () =>
    `<a class="dd-item" href="/foo">Item A</a><button class="dd-item" type="button">Item B</button>`,
}));

describe('NavDropdown', () => {
  it('renders the trigger with label and icon', async () => {
    const { default: NavDropdown } = await loadNavDropdown();
    render(NavDropdown, {
      props: { label: 'Account', icon: '👤', children: dropdownItems },
    });
    expect(screen.getByRole('button', { name: /Account/ })).toBeTruthy();
    expect(screen.getByText('👤')).toBeTruthy();
    // Menu is closed by default
    expect(screen.queryByRole('menu')).toBeNull();
  });

  it('opens and closes on trigger clicks (aria-expanded toggles)', async () => {
    const { default: NavDropdown } = await loadNavDropdown();
    render(NavDropdown, {
      props: { label: 'Menu', children: dropdownItems },
    });
    const trigger = screen.getByRole('button', { name: /Menu/ });
    expect(trigger.getAttribute('aria-expanded')).toBe('false');

    await fireEvent.click(trigger);
    expect(trigger.getAttribute('aria-expanded')).toBe('true');
    expect(screen.getByRole('menu')).toBeTruthy();
    // Snippet content renders inside the menu (roles resolve after the menu
    // is mounted — query by text node).
    expect(document.body.textContent).toContain('Item A');

    await fireEvent.click(trigger);
    expect(trigger.getAttribute('aria-expanded')).toBe('false');
    expect(screen.queryByRole('menu')).toBeNull();
  });

  it('closes on Escape', async () => {
    const { default: NavDropdown } = await loadNavDropdown();
    render(NavDropdown, {
      props: { label: 'Esc', children: dropdownItems },
    });
    await fireEvent.click(screen.getByRole('button', { name: /Esc/ }));
    expect(screen.getByRole('menu')).toBeTruthy();

    await fireEvent.keyDown(document.querySelector('.nav-dropdown')!, { key: 'Escape' });
    expect(screen.queryByRole('menu')).toBeNull();
  });

  it('closes when clicking an interactive item inside the menu', async () => {
    const { default: NavDropdown } = await loadNavDropdown();
    render(NavDropdown, {
      props: { label: 'Click', children: dropdownItems },
    });
    await fireEvent.click(screen.getByRole('button', { name: /Click/ }));
    // Click a .dd-item inside the menu (query the DOM directly — the
    // snippet's <a>/<button> don't get accessible roles in the fragment).
    const item = document.querySelector('.dd-menu .dd-item');
    expect(item).toBeTruthy();
    await fireEvent.click(item as Element);
    expect(screen.queryByRole('menu')).toBeNull();
  });

  it('aligns right when align=right', async () => {
    const { default: NavDropdown } = await loadNavDropdown();
    render(NavDropdown, {
      props: { label: 'Right', align: 'right', children: dropdownItems },
    });
    await fireEvent.click(screen.getByRole('button', { name: /Right/ }));
    expect(screen.getByRole('menu').classList.contains('align-right')).toBe(true);
  });

  it('renders the caret rotated when open', async () => {
    const { default: NavDropdown } = await loadNavDropdown();
    render(NavDropdown, {
      props: { label: 'Caret', children: dropdownItems },
    });
    expect(screen.getByText('▾')).toBeTruthy();
    await fireEvent.click(screen.getByRole('button', { name: /Caret/ }));
    expect(screen.getByText('▴')).toBeTruthy();
  });
});
