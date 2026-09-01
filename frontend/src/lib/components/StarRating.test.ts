import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

async function loadStarRating() {
  return await import('./StarRating.svelte');
}

describe('StarRating', () => {
  it('renders 5 stars, filled according to the value', async () => {
    const { default: StarRating } = await loadStarRating();
    render(StarRating, { props: { value: 3, readonly: true } });

    const stars = screen.getAllByRole('button');
    expect(stars).toHaveLength(5);
    // 3 filled stars (aria-pressed) + 2 empty
    const pressed = stars.filter((s) => s.getAttribute('aria-pressed') === 'true');
    expect(pressed).toHaveLength(3);
    // Filled stars render ★ (aria-label carries the star count)
    expect(screen.getByRole('button', { name: '3 stars' })).toBeTruthy();
    expect(screen.getByRole('button', { name: '5 stars' })).toBeTruthy();
  });

  it('renders 0 filled stars when no value is set', async () => {
    const { default: StarRating } = await loadStarRating();
    render(StarRating, { props: { value: 0, readonly: true } });
    const pressed = screen.getAllByRole('button').filter((s) => s.getAttribute('aria-pressed') === 'true');
    expect(pressed).toHaveLength(0);
  });

  it('clicking a star sets the rating and fires onrate', async () => {
    const { default: StarRating } = await loadStarRating();
    const onrate = vi.fn();
    const { component } = render(StarRating, { props: { value: 0, onrate } });

    await fireEvent.click(screen.getByRole('button', { name: 'Rate 4 stars' }));

    expect(onrate).toHaveBeenCalledWith(4);
    expect(component).toBeTruthy();
  });

  it('fills stars on hover preview', async () => {
    const { default: StarRating } = await loadStarRating();
    render(StarRating, { props: { value: 2 } });

    const stars = screen.getAllByRole('button');
    await fireEvent.mouseEnter(stars[4]); // 5th star
    // Hover preview is a visual (filled class) effect; the selected value
    // (aria-pressed) stays at 2 until a click commits.
    const filled = screen.getAllByRole('button').filter((s) => s.classList.contains('filled'));
    expect(filled).toHaveLength(5);
  });

  it('does not fire onrate when readonly', async () => {
    const { default: StarRating } = await loadStarRating();
    const onrate = vi.fn();
    render(StarRating, { props: { value: 3, readonly: true, onrate } });

    await fireEvent.click(screen.getByRole('button', { name: '5 stars' }));
    expect(onrate).not.toHaveBeenCalled();
  });
});
