import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

// jsdom gives window.localStorage but ESM bare `localStorage.xxx` is not
// automatically globalThis.localStorage — stub it as a true global binding.
const ls = typeof window !== 'undefined' ? (window as any).localStorage : undefined;
if (ls) {
  vi.stubGlobal('localStorage', ls);
} else {
  const store = new Map<string, string>();
  const fake = {
    getItem: (k: string) => store.get(k) ?? null,
    setItem: (k: string, v: string) => { store.set(k, String(v)); },
    removeItem: (k: string) => store.delete(k),
    clear: () => store.clear(),
    get length() { return store.size; },
    key: (i: number) => Array.from(store.keys())[i] ?? null,
  } as unknown as Storage;
  vi.stubGlobal('localStorage', fake);
}
