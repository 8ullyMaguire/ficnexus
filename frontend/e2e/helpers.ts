// Shared helpers for the FicHub browser E2E suite.
import { test as base, expect, type Page, type APIRequestContext } from '@playwright/test';

export const BACKEND_URL = process.env.E2E_BASE_URL || 'http://localhost:8000';

// Unique username per run so repeated local runs never collide on the
// `users_username_key` / `users_email_key` unique constraints.
export function uniqueUser(prefix = 'e2e'): { username: string; password: string; email: string } {
  const stamp = `${Date.now().toString(36)}${Math.floor(Math.random() * 1e6).toString(36)}`;
  return {
    username: `${prefix}_${stamp}`.slice(0, 30),
    password: 'e2epass123',
    email: `${prefix}_${stamp}@example.com`,
  };
}

// Register a fresh account against the REAL backend.
// The register endpoint has a honeypot: `form_opened_at` must be present
// and at least 500ms in the past, and the hidden `website` field empty.
// We also pass a unique email because `users.email` is UNIQUE and the
// frontend sends `email: undefined` → COALESCE → '' → second registration
// without an email collides.
export async function registerUser(
  request: APIRequestContext,
  user: { username: string; password: string; email: string },
): Promise<string> {
  const openedAt = Date.now() - 5_000;
  const res = await request.post(`${BACKEND_URL}/api/auth/register`, {
    data: { ...user, form_opened_at: String(openedAt) },
  });
  expect(res.ok(), `register should succeed (got ${res.status()})`).toBeTruthy();
  const body = (await res.json()) as { err: number; token?: string };
  expect(body.err).toBe(0);
  expect(body.token, 'register must return a usable token').toBeTruthy();
  return body.token as string;
}

export async function login(
  request: APIRequestContext,
  username: string,
  password: string,
): Promise<string> {
  const res = await request.post(`${BACKEND_URL}/api/auth/login`, {
    data: { username, password },
  });
  expect(res.ok()).toBeTruthy();
  const body = (await res.json()) as { err: number; token?: string };
  expect(body.err).toBe(0);
  expect(body.token).toBeTruthy();
  return body.token as string;
}

// Seed a work via the manual-upload endpoint (TXT upload, no network
// scraping needed). Returns the fic's url_id as shown on the fic page.
export async function seedFic(
  request: APIRequestContext,
  token: string,
  title: string,
  author: string,
  summary: string,
): Promise<string> {
  const content = `Chapter 1\n${summary}\nThe story unfolds in a world of wonder.\n\nChapter 2\nThe plot thickens as secrets are revealed.\n`;
  const res = await request.post(`${BACKEND_URL}/api/upload`, {
    headers: { Authorization: `Bearer ${token}` },
    multipart: {
      title,
      author,
      summary,
      tags: 'fantasy, adventure',
      status: 'ongoing',
      file: {
        name: 'story.txt',
        mimeType: 'text/plain',
        buffer: Buffer.from(content, 'utf8'),
      },
    },
  });
  expect(res.ok(), `upload should succeed (got ${res.status()}: ${await res.text().catch(() => '')})`).toBeTruthy();
  const body = (await res.json()) as { err: number; url_id?: string; msg?: string };
  expect(body.err).toBe(0);
  expect(body.url_id).toBeTruthy();
  return body.url_id as string;
}

// Create a fic request (prompt board) via the API.
export async function seedRequest(
  request: APIRequestContext,
  token: string,
  title: string,
  body_text: string,
): Promise<number> {
  const res = await request.post(`${BACKEND_URL}/api/requests`, {
    headers: { Authorization: `Bearer ${token}` },
    data: { title, body: body_text },
  });
  expect(res.ok()).toBeTruthy();
  const body = (await res.json()) as { err: number; id?: number; msg?: string };
  expect(body.err).toBe(0);
  expect(body.id, `request creation must return an id (msg: ${body.msg ?? 'none'})`).toBeTruthy();
  return body.id as number;
}

// Sign the browser session in: put the JWT in localStorage the same way
// the app's auth store does, then reload so /api/auth/me resolves.
export async function signInViaStorage(page: Page, token: string): Promise<void> {
  await page.addInitScript((t) => {
    localStorage.setItem('fichub_token', t);
  }, token);
}

// Navigate and wait for the app shell (layout nav) to appear.
export async function gotoApp(page: Page, path: string): Promise<void> {
  await page.goto(path);
  await page.waitForLoadState('domcontentloaded');
}

// Sign the browser session in with a JWT: load home, put the token in
// localStorage the same way the app's auth store does, then reload so
// /api/auth/me resolves before the target page mounts. addInitScript alone is
// unreliable for SPA auth-init timing on first navigation.
export async function signInBrowser(page: Page, token: string): Promise<void> {
  await page.goto('/');
  await page.waitForLoadState('domcontentloaded');
  await page.evaluate((t) => {
    localStorage.setItem('fichub_token', t);
  }, token);
  await page.reload();
  await page.waitForLoadState('domcontentloaded');
  await page.waitForTimeout(1200);
}

// --- Backend liveness ------------------------------------------------------
// The suite requires the real backend on :8000. When it is missing we skip
// the whole file with a clear message instead of failing 50 tests.
export function backendUpCheck(page: Page): void {
  // Implementation is in each spec via test.beforeAll using fetch.
}

export const authBtn = (page: Page) => page.getByRole('button', { name: 'Login' });
export const registerBtn = (page: Page) => page.getByRole('button', { name: 'Register' });
