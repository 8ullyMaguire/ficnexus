// Auth flows: register / login / logout via the real /api/auth/* endpoints,
// plus the auth guard behavior on protected routes (follows redirects
// logged-out users home; bookmarks renders a logged-out empty state).
import { test, expect, request as pwRequest } from '@playwright/test';
import { BACKEND_URL, uniqueUser, registerUser, login, signInViaStorage, gotoApp } from './helpers';

let backendUp = false;
let backendReason = '';

test.beforeAll(async () => {
  const req = await pwRequest.newContext({ baseURL: BACKEND_URL });
  try {
    const res = await req.get('/api/health?skip_redis=true');
    backendUp = res.ok();
    if (!backendUp) backendReason = `health returned ${res.status()}`;
  } catch (e) {
    backendReason = `backend unreachable: ${e instanceof Error ? e.message : e}`;
  } finally {
    await req.dispose();
  }
});

// Skip every test in this file (loudly) when the real backend is missing —
// a forgotten server must never produce 50 confusing failures.
test.beforeEach(async ({}, testInfo) => {
  test.skip(!backendUp, `Skipping ${testInfo.title} — backend not up on ${BACKEND_URL} (${backendReason})`);
});

test('register flow: fills form, submits, and lands logged-in with a JWT', async ({ page }) => {
  // The API client sends form_opened_at (Date.now()-5000) and website:''
  // which pass the backend honeypot checks. Verify the full round-trip:
  // modal opens -> fill -> submit -> modal closes -> JWT stored -> username shown.
  const user = uniqueUser('reg');
  await gotoApp(page, '/');

  await page.getByRole('button', { name: 'Register' }).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Register' })).toBeVisible();

  await page.waitForTimeout(800);
  await page.getByPlaceholder('Username').fill(user.username);
  await page.getByPlaceholder('Password').fill(user.password);
  await page.getByPlaceholder('email@example.com').fill(user.email);
  await page.getByRole('button', { name: 'Register' }).last().click();

  // Modal closes on success.
  await expect(page.getByRole('dialog')).toBeHidden();
  // JWT is persisted in localStorage.
  await expect
    .poll(async () => page.evaluate(() => localStorage.getItem('fichub_token')), {
      message: 'register must store a JWT in localStorage',
      timeout: 10_000,
    })
    .toBeTruthy();
  // Username appears in the nav (logged-in state).
  await expect(page.getByText(user.username)).toBeVisible({ timeout: 10_000 });
});

test('register without email does not collide with existing empty-email users', async ({ page }) => {
  // Regression: the users_email_key index was a full btree on email.
  // COALESCE(email,'') in register_user meant the first user (admin with
  // empty email) blocked ALL email-less registrations. Fixed to a partial
  // index: CREATE UNIQUE INDEX ... WHERE email != ''.
  const user = uniqueUser('noemail');
  await gotoApp(page, '/');

  await page.getByRole('button', { name: 'Register' }).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await page.waitForTimeout(800);
  await page.getByPlaceholder('Username').fill(user.username);
  await page.getByPlaceholder('Password').fill(user.password);
  // Leave email empty — the field exists but we don't fill it.
  await page.getByRole('button', { name: 'Register' }).last().click();

  await expect(page.getByRole('dialog')).toBeHidden();
  await expect
    .poll(async () => page.evaluate(() => localStorage.getItem('fichub_token')), {
      message: 'email-less register must still store a JWT',
      timeout: 10_000,
    })
    .toBeTruthy();
});

test('login with wrong password shows an error and stays logged out', async ({ page, request }) => {
  const user = uniqueUser('bad');
  await registerUser(request, user);
  await gotoApp(page, '/');

  await page.getByRole('button', { name: 'Login' }).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await page.waitForTimeout(800);
  await page.getByPlaceholder('Username').fill(user.username);
  await page.getByPlaceholder('Password').fill('wrong-password');
  await page.getByRole('button', { name: 'Login' }).last().click();

  await expect(page.getByText('Invalid username or password')).toBeVisible();
  await expect(page.getByText(`👤 ${user.username}`)).not.toBeVisible();
  const token = await page.evaluate(() => localStorage.getItem('fichub_token'));
  expect(token).toBeNull();
});

test('login succeeds and logout returns to anonymous state', async ({ page, request }) => {
  const user = uniqueUser('ok');
  await registerUser(request, user);
  await gotoApp(page, '/');

  await page.getByRole('button', { name: 'Login' }).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await page.waitForTimeout(800);
  await page.getByPlaceholder('Username').fill(user.username);
  await page.getByPlaceholder('Password').fill(user.password);
  await page.getByRole('button', { name: 'Login' }).last().click();

  // The logged-in nav replaces AuthBar with an Account dropdown labelled
  // with the username (NavDropdown label) — no 👤 prefix in the label text.
  await expect(page.getByRole('button', { name: user.username })).toBeVisible();

  // Logout is inside the user dropdown menu — open it first.
  await page.getByRole('button', { name: user.username }).click();
  await page.getByRole('button', { name: '🚪 Logout' }).click();
  // Logout returns the nav to Login/Register buttons.
  await expect(page.getByRole('button', { name: 'Login' })).toBeVisible();
  await expect(page.getByRole('button', { name: user.username })).not.toBeVisible();
});

test('auth guard: /follows redirects anonymous users to the home page', async ({ page }) => {
  await gotoApp(page, '/follows');
  // follows page calls goto('/') when not logged in.
  await page.waitForURL('**/');
  await expect(page.getByRole('button', { name: 'Login' })).toBeVisible();
});

test('bookmarks page shows a logged-out empty state (no crash, no data leak)', async ({ page }) => {
  await gotoApp(page, '/bookmarks');
  await expect(page.getByRole('heading', { name: /Bookmarks/ })).toBeVisible();
  // Anonymous: page does not fetch user bookmarks.
  await page.waitForTimeout(800);
  const body = await page.locator('body').innerText();
  expect(body).not.toContain('You have no bookmarks yet'); // logged-in empty text
});

test('existing token in localStorage signs the session in (getMe path)', async ({ page, request }) => {
  const user = uniqueUser('tok');
  const token = await registerUser(request, user);
  await signInViaStorage(page, token);
  await gotoApp(page, '/');
  await expect(page.getByRole('button', { name: user.username })).toBeVisible({ timeout: 20_000 });
});
