import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';

// OfflineIndicator: shows the offline banner when the browser goes offline
// and hides it when the connection returns.
async function loadIndicator() {
  return await import('$lib/components/OfflineIndicator.svelte');
}

// jsdom does not implement navigator.onLine as a settable property in a
// useful way — stub it via Object.defineProperty on the navigator instance.
function setOnline(value: boolean) {
  Object.defineProperty(navigator, 'onLine', {
    configurable: true,
    get: () => value,
  });
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('OfflineIndicator', () => {
  it('does not render the banner when online', async () => {
    setOnline(true);
    const { default: Indicator } = await loadIndicator();
    render(Indicator);
    expect(screen.queryByText(/you're offline/i)).toBeNull();
  });

  it('shows the banner when the offline event fires', async () => {
    setOnline(true);
    const { default: Indicator } = await loadIndicator();
    render(Indicator);

    setOnline(false);
    window.dispatchEvent(new Event('offline'));

    await waitFor(() => {
      expect(screen.getByText(/you're offline/i)).toBeTruthy();
    });
  });

  it('hides the banner when the online event fires', async () => {
    setOnline(false);
    const { default: Indicator } = await loadIndicator();
    render(Indicator);
    await waitFor(() => {
      expect(screen.getByText(/you're offline/i)).toBeTruthy();
    });

    setOnline(true);
    window.dispatchEvent(new Event('online'));

    await waitFor(() => {
      expect(screen.queryByText(/you're offline/i)).toBeNull();
    });
  });
});
