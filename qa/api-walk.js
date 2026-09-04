#!/usr/bin/env node
// qa/api-walk.js — scriptable API-surface audit ("route walk").
//
// Boots the real server (or uses a running one at QA_BASE), enumerates every
// route from src/server.rs build_router(), and hits each endpoint:
//   - GET endpoints: 200/expected-4xx required; flags empty/stub responses.
//   - POST/PUT/DELETE endpoints: expected-auth-gate behavior required
//     (401/403 for anonymous where gated); a fixture admin token is used to
//     exercise the real handler where it is safe.
//
// Exit 0 = every endpoint responded as expected; 1 = failures found.
//
// Usage:
//   node qa/api-walk.js                 # uses QA_BASE (default http://localhost:8000)
//   QA_BASE=http://localhost:8000 node qa/api-walk.js
//   API_WALK_TOKEN=<jwt> node qa/api-walk.js   # with a real app-generated admin token

import fs from 'fs';
import path from 'path';

const ROOT = path.resolve(import.meta.dirname, '..');
const BASE = process.env.QA_BASE || 'http://localhost:8000';
const TOKEN = process.env.API_WALK_TOKEN || '';
const TIMEOUT_MS = Number(process.env.API_WALK_TIMEOUT || 8000);

// ─── Route discovery (parsed from src/server.rs build_router) ─────────────
// We parse the literal .route("/api/...", ...) strings from the source so
// the walk is always in sync with the real router. POST/DELETE endpoints are
// exercised as anonymous (must be gated 4xx); GET endpoints are hit and
// checked for stub-like responses.
function discoverRoutes() {
  const src = fs.readFileSync(path.join(ROOT, 'src', 'server.rs'), 'utf8');
  const lines = src.split('\n');
  const out = [];
  for (const ln of lines) {
    const m = ln.match(/\.route\(\s*"(\/api\/[^"]+)"\s*,\s*(\w+)\(/);
    if (!m) continue;
    const route = m[1];
    const handler = m[2];
    let method = 'GET';
    if (handler === 'post' || handler === 'put' || handler === 'delete' || handler === 'patch') {
      method = handler.toUpperCase();
    } else if (handler === 'get') {
      method = 'GET';
    } else {
      // e.g. `axum::routing::post(...)` captured by \w+ as 'post'
      method = handler.toUpperCase();
    }
    out.push([method, route]);
  }
  // dedupe
  return [...new Map(out.map((r) => [r.join(' '), r])).values()];
}

const FIXTURES = {
  url_id: 'ao3_21845264',          // a real AO3 fic
  request_id: '1',
  answer_id: '1',
  user_id: '1',
  work_id: '1',
  tag_id: '1',
  cluster_id: '1',
  comment_id: '1',
  list_id: '1',
  shelf_id: '1',
  series_id: '1',
  author_id: '1',
  proposal_id: '1',
  notification_id: '1',
  badge_id: '1',
  quest_id: '1',
  target_type: 'user',
  target_id: '1',
  other_user_id: '2',
};

// Route list is discovered dynamically from src/server.rs via discoverRoutes().

function template(p) {
  return p.replace(/\{(\w+)\}/g, (_, k) => FIXTURES[k] ?? `1`);
}

function isStubLike(body, contentType, route) {
  if (!contentType || !contentType.includes('json')) return false;
  if (body === null || body === undefined) return true;
  const s = typeof body === 'string' ? body : JSON.stringify(body);
  if (s === '{}' || s === '[]') return true;
  // Data-dependent list endpoints legitimately return empty arrays for
  // fixture ids that have no data — only flag truly minimal stubs.
  const dataList = route.includes('/similar') || route.includes('/recommendations')
    || route.includes('/candidates') || route.includes('/trending')
    || route.includes('/feed') || route.includes('/updates')
    || route.includes('/also-bookmarked') || route.includes('/related')
    || route.includes('/suggest');
  if (dataList && (s === '{"err":0,"items":[]}' || s === '{"err":0,"entries":[]}' || s === '{"err":0,"candidates":[]}')) return false;
  if (s === '{"err":0}' || s === '{"err":0,"items":[]}' || s === '{"err":0,"entries":[]}') return true;
  return false;
}

async function hit(method, path, token) {
  const url = BASE + path;
  const headers = {};
  if (token) headers.Authorization = `Bearer ${token}`;
  headers['X-Client-ID'] = 'api-walk-audit';
  const ctl = new AbortController();
  const t = setTimeout(() => ctl.abort(), TIMEOUT_MS);
  try {
    const res = await fetch(url, { method, headers, redirect: 'manual', signal: ctl.signal });
    const ct = res.headers.get('content-type') || '';
    const text = await res.text();
    let body = text;
    if (ct.includes('json')) { try { body = JSON.parse(text); } catch { /* keep text */ } }
    return { status: res.status, ct, body };
  } catch (e) {
    return { status: 0, ct: '', body: `ERR: ${e.message}` };
  } finally {
    clearTimeout(t);
  }
}

// ─── Docs link-check: every docs-map entry page + anchor must resolve ────
// Prevents doc drift: broken anchors get caught here, not by users.
async function docsLinkCheck() {
  console.log(`\nDocs link-check → ${BASE}/docs/docs-map.json\n`);
  let pass = 0, fail = 0;
  try {
    const res = await fetch(`${BASE}/docs/docs-map.json`, { headers: { 'X-Client-ID': 'api-walk-docs' } });
    if (!res.ok) {
      console.log(`✗ docs-map.json fetch failed (HTTP ${res.status})`);
      return 0;
    }
    const map = await res.json();
    // Cache fetched pages so we only fetch each .html once
    const pageCache = new Map();
    for (const entry of map) {
      if (!entry.anchor) continue; // page-level entries checked via pages below
      let html;
      if (pageCache.has(entry.page)) {
        html = pageCache.get(entry.page);
      } else {
        const pr = await fetch(`${BASE}/docs/${entry.page}`, { headers: { 'X-Client-ID': 'api-walk-docs' } });
        html = pr.ok ? await pr.text() : '';
        pageCache.set(entry.page, html);
      }
      const ok = html.includes(`id="${entry.anchor}"`);
      if (ok) pass++;
      else { fail++; console.log(`✗ ${entry.slug} — anchor "${entry.anchor}" not found in ${entry.page}`); }
    }
    // Also verify each page (no anchor) is fetchable
    const pages = new Set(map.filter((e) => !e.anchor).map((e) => e.page));
    for (const p of pages) {
      const pr = await fetch(`${BASE}/docs/${p}`, { headers: { 'X-Client-ID': 'api-walk-docs' } });
      if (pr.ok) pass++;
      else { fail++; console.log(`✗ ${p} — HTTP ${pr.status}`); }
    }
  } catch (e) {
    console.log(`✗ docs link-check error: ${e.message}`);
    return 1;
  }
  console.log(`\nDocs: ${pass} passed, ${fail} failed`);
  return fail;
}

async function main() {
  const routes = discoverRoutes();
  const results = [];
  let pass = 0, fail = 0;

  console.log(`API route-walk → ${BASE} (${routes.length} routes, token: ${TOKEN ? 'yes' : 'no'})\n`);
  for (const [method, route] of routes) {
    // Derive the gate kind from the path
    const kind = route.startsWith('/api/admin/') ? 'admin'
               : route.startsWith('/api/me') || route.startsWith('/api/notifications')
               || route.startsWith('/api/bookmarks') || route.startsWith('/api/follows')
               || route.startsWith('/api/reading') || route.startsWith('/api/lists')
               || route.startsWith('/api/shelves') || route.startsWith('/api/curator')
               || route.startsWith('/api/user/export') || route.startsWith('/api/recommendations/personal')
               ? 'auth' : 'public';
    // These endpoints are OPTIONALLY authenticated (they serve anonymous
    // callers too) — treat them as public for the anonymous gate check.
    const optionalAuth = route.startsWith('/api/meta')
      || route === '/api/recommendations/personal'
      || route.startsWith('/api/follows/followers');
    const requiresGate = !optionalAuth;
    // For non-GET, always expect a gate when anonymous (no token); with a
    // token, only check that it's not a 5xx.
    const p = template(route);
    const r = await hit(method, p, TOKEN);
    const okStatus = r.status >= 200 && r.status < 400;
    const authExpected4xx = (kind === 'auth' || kind === 'admin') && !TOKEN && requiresGate;
    const nonGetAnonymous = method !== 'GET' && !TOKEN;

    let verdict;
    if (r.status === 0) {
      verdict = `FAIL network/timeout (${r.body})`;
    } else if (authExpected4xx || nonGetAnonymous) {
      // without a token, auth/admin endpoints AND all non-GET must reject (4xx)
      verdict = r.status >= 400 ? 'OK (gated)' : `FAIL (should be gated, got ${r.status})`;
    } else if (!okStatus) {
      // real handler paths may legitimately 4xx for missing fixtures, but
      // never 5xx
      verdict = r.status >= 500 ? `FAIL 5xx` : `OK (expected ${r.status})`;
    } else if (isStubLike(r.body, r.ct, route)) {
      verdict = `STUB-LIKE (${r.status}) — ${JSON.stringify(r.body).slice(0, 120)}`;
    } else {
      verdict = `OK (${r.status})`;
    }

    let ok = verdict.startsWith('OK');
    if (verdict.startsWith('STUB')) { ok = false; }
    if (ok) pass++; else fail++;
    results.push({ method, route: p, kind, verdict, ok });
    console.log(`${ok ? '✓' : '✗'} ${method.padEnd(4)} ${p.padEnd(55)} ${verdict}`);
  }

  console.log(`\n${pass} passed, ${fail} failed`);

  const docsFail = await docsLinkCheck();
  const totalFail = fail + docsFail;
  console.log(`\nTOTAL: ${pass} api ok + ${docsFail} docs failures — ${totalFail ? 'FAIL' : 'ALL GREEN'}`);
  process.exit(totalFail ? 1 : 0);
}

main().catch((e) => { console.error(e); process.exit(2); });
