import { describe, it, expect, vi, afterEach } from 'vitest';

// registerServiceWorker guards: SSR (no window), no SW support, insecure
// context, dev mode, and the production registration path.
async function loadRegister() {
  return await import('./register');
}

afterEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
});

describe('registerServiceWorker', () => {
  it('no-ops when window is undefined (SSR)', async () => {
    const { registerServiceWorker } = await loadRegister();
    expect(() => registerServiceWorker()).not.toThrow();
  });

  it('no-ops when serviceWorker is not supported', async () => {
    vi.stubGlobal('window', { isSecureContext: true, addEventListener: vi.fn() });
    vi.stubGlobal('navigator', {});
    const { registerServiceWorker } = await loadRegister();
    registerServiceWorker();
    expect(window.addEventListener).not.toHaveBeenCalled();
  });

  it('no-ops in dev builds', async () => {
    vi.stubGlobal('window', {
      isSecureContext: true,
      addEventListener: vi.fn(),
    });
    vi.stubGlobal('navigator', { serviceWorker: { register: vi.fn() } });
    // import.meta.env.DEV is true under vitest.
    const { registerServiceWorker } = await loadRegister();
    registerServiceWorker();
    expect(window.addEventListener).not.toHaveBeenCalled();
  });

  it('registers the SW on load in a secure production context', async () => {
    const registerMock = vi.fn().mockResolvedValue(undefined);
    const addEventListenerMock = vi.fn();
    vi.stubGlobal('window', {
      isSecureContext: true,
      addEventListener: addEventListenerMock,
    });
    vi.stubGlobal('navigator', { serviceWorker: { register: registerMock } });

    // Force the production branch: register.ts reads import.meta.env.DEV.
    vi.stubEnv('DEV', false);

    const { registerServiceWorker } = await loadRegister();
    registerServiceWorker();

    expect(addEventListenerMock).toHaveBeenCalledWith('load', expect.any(Function));
    // Fire the load listener → SW registration runs.
    const loadHandler = addEventListenerMock.mock.calls[0][1] as () => void;
    loadHandler();
    expect(registerMock).toHaveBeenCalledWith('/sw.js');
  });

  it('logs a console error when SW registration fails', async () => {
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
    const registerMock = vi.fn().mockRejectedValue(new Error('no sw'));
    vi.stubGlobal('window', {
      isSecureContext: true,
      addEventListener: vi.fn(),
    });
    vi.stubGlobal('navigator', { serviceWorker: { register: registerMock } });
    vi.stubEnv('DEV', false);

    const { registerServiceWorker } = await loadRegister();
    registerServiceWorker();
    const loadHandler = (window.addEventListener as ReturnType<typeof vi.fn>).mock.calls[0][1] as () => void;
    await loadHandler();

    expect(consoleError).toHaveBeenCalledWith(
      '[pwa] service worker registration failed:',
      expect.any(Error),
    );
    consoleError.mockRestore();
  });
});
