import { defineConfig } from '@playwright/test';

// FicHub browser E2E suite — drives the REAL backend over HTTP.
//
// The backend (Rust/Axum + Postgres + Redis) must be running on :8000
// BEFORE this suite starts. Start it from the repo root with:
//
//   set -a; . .env-e2e/runtime.env; set +a
//   target/release/fichub        # or: cargo run
//
// (or point E2E_BASE_URL at another instance). The suite skips cleanly —
// with a loud message — when the backend is unreachable, so a forgotten
// server never produces 50 confusing failures. Every test asserts real
// rendered UI state (headings, buttons, list items, empty states), not
// just "page loaded with no console errors".
//
// Runs:   cd frontend && npx playwright test
// One:    cd frontend && npx playwright test e2e/auth.spec.ts

const BASE_URL = process.env.E2E_BASE_URL || 'http://localhost:8000';

export default defineConfig({
  testDir: './e2e',
  timeout: 60_000,
  expect: { timeout: 15_000 },
  fullyParallel: false,
  workers: 1, // one backend, one DB — keep mutations serial
  retries: process.env.CI ? 1 : 0,
  reporter: [['list']],
  use: {
    baseURL: BASE_URL,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: {
        browserName: 'chromium',
        // Prefer the already-installed bundled chromium (same build qa/ uses).
        // Falls back to Playwright's default resolution if the path is gone.
        launchOptions: {
          executablePath:
            process.env.E2E_CHROMIUM ||
            (process.env.HOME
              ? `${process.env.HOME}/.cache/ms-playwright/chromium-1234/chrome-linux64/chrome`
              : undefined),
        },
      },
    },
  ],
});
