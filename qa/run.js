// fichub QA harness — deterministic bug discovery
// Zero LLM dependency: console/network/HTTP/OPDS/link checks produce structured findings.
// Run: node qa/run.js (entry point; see README.md)
import { chromium } from 'playwright-core';
import { createRequire } from 'module';
import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import os from 'os';
import crypto from 'crypto';

const require = createRequire(import.meta.url);
const sqlite3 = require('better-sqlite3');

const ROOT = path.resolve(import.meta.dirname, '..');
const BASE = process.env.QA_BASE || 'http://localhost:8000';
const DB_PATH = process.env.QA_DB || path.join(ROOT, 'qa', 'bugs.db');
const ART = path.join(ROOT, 'qa', 'artifacts');
const REPORT_DIR = path.join(ROOT, 'qa', 'reports');
function detectChromium() {
  if (process.env.QA_BROWSER) return process.env.QA_BROWSER;
  const base = path.join(os.homedir(), '.cache', 'ms-playwright');
  try {
    const dirs = fs.readdirSync(base).filter(d => d.startsWith('chromium-')).sort();
    if (dirs.length) return path.join(base, dirs[dirs.length - 1], 'chrome-linux64', 'chrome');
  } catch {}
  return path.join(base, 'chromium-1208', 'chrome-linux64', 'chrome');
}
const BROWSER = detectChromium();
const SHOT = process.env.QA_SHOT || (process.env.CI ? 'none' : 'on-failure');
const SKIP_BROWSER = process.env.QA_SKIP_BROWSER === '1';
const CRAWL_CACHE = path.join(ROOT, 'qa', 'crawl-cache');

fs.mkdirSync(ART, { recursive: true });
fs.mkdirSync(REPORT_DIR, { recursive: true });
fs.mkdirSync(CRAWL_CACHE, { recursive: true });

// ─── Queue: SQLite v2, fingerprints dedupe, events audit ───────────────────
const db = new sqlite3(DB_PATH);
db.exec(fs.readFileSync(path.join(import.meta.dirname, 'schema.sql'), 'utf8'));
try { db.exec("ALTER TABLE bugs ADD COLUMN triage TEXT"); } catch {}

// v2: TEXT ids — convert any legacy INTEGER ids to BUG-XXXX format
try {
  const legacy = db.prepare("SELECT id FROM bugs WHERE typeof(id) = 'integer'").all();
  for (const l of legacy) {
    db.prepare("UPDATE bugs SET id = 'BUG-' || printf('%04d', id) WHERE id = ?").run(l.id);
  }
} catch {}

const now = () => new Date().toISOString();
const run = db.prepare('INSERT INTO qa_runs (started_at, git_commit, git_branch) VALUES (?, ?, ?)');
const finishRun = db.prepare('UPDATE qa_runs SET finished_at=?, open_bugs=?, new_bugs=?, fixed_bugs=?, exit_code=? WHERE id=?');
let runId;
try {
  const commit = execSync('git -C ' + ROOT + ' rev-parse --short HEAD 2>/dev/null').toString().trim() || 'unknown';
  const branch = execSync('git -C ' + ROOT + ' rev-parse --abbrev-ref HEAD 2>/dev/null').toString().trim() || 'unknown';
  runId = run.run(now(), commit, branch).lastInsertRowid;
} catch { runId = run.run(now(), 'unknown', 'unknown').lastInsertRowid; }

const upsert = db.prepare(`INSERT INTO bugs (id, fingerprint, title, severity, status, category, route, method, status_code, evidence_json, first_seen_run, last_seen_run, occurrences, created_at, updated_at)
VALUES (@id, @fp, @title, @sev, 'new', @cat, @route, @method, @code, @evidence, @runId, @runId, 1, @now, @now)
ON CONFLICT(fingerprint) DO UPDATE SET
  occurrences = occurrences + 1,
  last_seen_run = excluded.last_seen_run,
  updated_at = excluded.updated_at,
  status = CASE WHEN bugs.status IN ('fixed','wontfix','duplicate') THEN 'regression' ELSE bugs.status END,
  evidence_json = excluded.evidence_json`);

let passed = 0, failed = 0;
const results = [];

function fingerprint(route, method, code, sig) {
  // v2: normalized, deterministic. IDs/timestamps/ports collapsed so the same
  // bug on different pages/params dedupes to one fingerprint.
  const { makeFingerprint } = require('./fingerprint.js');
  return makeFingerprint({ layer: 'app', route, method, errorClass: String(code), message: sig });
}

function report({ route = '', method = 'GET', code = null, title, severity = 'p2', category = 'functional', sig = '', evidence = {}, urls = [] }) {
  const fp = fingerprint(route, method, code, sig);
  const nowIso = now();
  const id = 'BUG-' + String(require('crypto').createHash('md5').update(fp).digest('hex').slice(0, 4)).toUpperCase();
  upsert.run({ id, fp, title, sev: severity, cat: category, route, method, code, evidence: JSON.stringify(evidence), runId, now: nowIso });
  failed++;
  results.push({ severity, category, route, method, code, title, fingerprint: fp, evidence, id });
}

function pass() { passed++; }

function checkHttp(label, res, opts = {}) {
  const code = res.status;
  const expected = opts.expected || 200;
  const route = opts.route || res.url;
  if (code === expected) { pass(); return res; }
  const sev = code >= 500 ? 'p0' : code >= 400 ? 'p1' : 'p2';
  report({ route, method: opts.method || 'GET', code, title: `${label}: HTTP ${code} (expected ${expected})`, severity: sev, category: 'api', sig: `HTTP ${code}`, evidence: { url: res.url, expected } });
  return res;
}

// ─── JSON helpers ───────────────────────────────────────────────────────────
async function getJson(url, { auth } = {}) {
  const headers = { accept: 'application/json' };
  if (auth) headers.authorization = 'Bearer ' + auth;
  const res = await fetch(BASE + url, { headers });
  const text = await res.text();
  let json = null; try { json = JSON.parse(text); } catch {}
  return { res, json, text };
}

// ─── API smoke suite (deterministic, no browser) ────────────────────────────
async function apiSmoke() {
  console.log('\n[1/4] API smoke suite');
  const health = await getJson('/api/health');
  checkHttp('health', health.res, { route: '/api/health' });

  const doc = await getJson('/api/');
  checkHttp('api docs', doc.res, { route: '/api/' });
  if (doc.json && doc.json.endpoints && Array.isArray(doc.json.endpoints)) {
    const eps = doc.json.endpoints;
    console.log(`  api docs advertise ${eps.length} endpoints`);
    // verify each advertised endpoint exists (HEAD probe)
    for (const e of eps.slice(0, 60)) {
      let url = typeof e === 'string' ? e : e.path || e.url;
      if (!url || !url.startsWith('/')) continue;
      url = url.replace(/\{[\w-]+\}/g, '1').replace(/:[a-zA-Z_]+/g, '1');
      try {
        const r = await fetch(BASE + url, { method: 'HEAD' });
        if (r.status === 404) {
          report({ route: url, method: 'HEAD', code: 404, title: `Advertised endpoint returns 404: ${url}`, severity: 'p1', category: 'api', sig: 'advertised-404' });
        } else pass();
      } catch (e) {
        report({ route: url, method: 'HEAD', code: null, title: `Advertised endpoint unreachable: ${url}`, severity: 'p2', category: 'api', sig: 'unreachable' });
      }
    }
  }

  // Public GET endpoints that should never 5xx
  const publicGets = [
    ['/', '/'], ['/api/search?q=harry', '/api/search'], ['/api/trending', '/api/trending'],
    ['/api/tags/search?q=romance', '/api/tags/search'], ['/api/tags/autocomplete?q=rom', '/api/tags/autocomplete'],
    ['/api/authors/search?q=rowling', '/api/authors/search'],
    ['/api/recommendations', '/api/recommendations'], ['/api/locales', '/api/locales'],
    ['/api/badges', '/api/badges'],
    ['/api/leaderboard/curators', '/api/leaderboard/curators'],
    ['/api/leaderboard/curators/weekly', '/api/leaderboard/curators/weekly'],
    ['/api/leaderboard/curators/monthly', '/api/leaderboard/curators/monthly'],
    ['/api/trending/tags', '/api/trending/tags'],
  ];
  for (const [url, route] of publicGets) {
    const { res } = await getJson(url);
    checkHttp(route, res, { route });
  }

  // Auth-required GET endpoints: expect 400/401 when unauthenticated; only 500 is a bug
  const authGets = [
    ['/api/feed?page=1', '/api/feed'], ['/api/quests', '/api/quests'],
    ['/api/notifications/unread-count', '/api/notifications/unread-count'],
    ['/api/shelves', '/api/shelves'], ['/api/bookmarks', '/api/bookmarks'],
    ['/api/reading/list', '/api/reading/list'],
  ];
  for (const [url, route] of authGets) {
    const { res } = await getJson(url);
    const code = res.status;
    if (code === 500) {
      report({ route, code, title: `Auth-required ${route} returns 500 (should 400/401 unauthenticated)`, severity: 'p0', category: 'api', sig: 'auth-500' });
    } else pass();
  }

  // OPDS catalog integrity
  console.log('  OPDS checks');
  const opds = ['/opds', '/opds/new', '/opds/popular', '/opds/tags', '/opds/authors', '/opds/search?q=harry', '/opds/shelves', '/opds/recommendations'];
  for (const u of opds) {
    const r = await fetch(BASE + u);
    checkHttp('opds ' + u, r, { route: u, expected: 200 });
    if (r.status === 200) {
      const ct = r.headers.get('content-type') || '';
      if (!ct.includes('xml') && !ct.includes('atom')) {
        report({ route: u, code: 200, title: `OPDS ${u} served wrong content-type: ${ct}`, severity: 'p1', category: 'opds', sig: 'content-type ' + ct, evidence: { ct } });
      } else {
        const body = await r.text();
        if (body.trim().startsWith('{')) {
          report({ route: u, code: 200, title: `OPDS ${u} returned JSON not XML`, severity: 'p1', category: 'opds', sig: 'json-not-xml' });
        } else if (body.includes('rel="self"')) pass(); else {
          report({ route: u, code: 200, title: `OPDS ${u} missing rel="self" link`, severity: 'p2', category: 'opds', sig: 'missing-self-link' });
        }
      }
    }
  }

  // Known-bad inputs should 400/404, not 500
  console.log('  negative-path checks');
  const negatives = [
    ['/api/tags', 'missing required params'],
    ['/api/search?q=', 'empty query'],
    ['/api/works/99999999', 'nonexistent work'],
    ['/api/authors/99999999', 'nonexistent author'],
    ['/api/reader/not-a-real-url-id', 'nonexistent reader'],
    ['/api/meta?url=', 'empty meta url'],
    ['/api/epub?url=', 'empty epub url'],
  ];
  for (const [u, label] of negatives) {
    const r = await fetch(BASE + u);
    const code = r.status;
    if (code === 500) report({ route: u, code, title: `Negative path 500: ${label}`, severity: 'p0', category: 'api', sig: 'negative-500' });
    else pass();
  }
  console.log(`  api smoke done: ${passed} ok, ${failed} findings`);
}

// ─── Link crawl (deterministic, from sitemap/root) ──────────────────────────
async function crawlLinks() {
  console.log('\n[2/4] Link crawl');
  const seen = new Set();
  const queue = ['/'];
  const failedLinks = new Set();
  const cacheFile = path.join(CRAWL_CACHE, 'crawl.json');
  let cached = {};
  try { cached = JSON.parse(fs.readFileSync(cacheFile, 'utf8')); } catch {}
  const fresh = {};

  const fetchPage = async (url) => {
    if (cached[url] && Date.now() - cached[url].t < 3600e3) return cached[url].html;
    const r = await fetch(BASE + url, { headers: { accept: 'text/html' } });
    const html = await r.text();
    fresh[url] = { t: Date.now(), code: r.status, html };
    if (r.status >= 400) return null;
    return html;
  };

  while (queue.length) {
    const url = queue.shift();
    if (seen.has(url)) continue;
    seen.add(url);
    const html = await fetchPage(url);
    if (!html) continue;
    const links = [...html.matchAll(/href="(\/[^"#?]*)/g)].map(m => m[1]).filter(u => u.length > 1 && u.length < 120);
    for (const link of links) {
      const clean = link.split('?')[0];
      if (!seen.has(clean) && queue.length < 220) queue.push(clean);
    }
    if (seen.size > 200) break;
  }
  fs.writeFileSync(cacheFile, JSON.stringify(fresh));

  // now verify every discovered link resolves
  for (const url of seen) {
    try {
      const r = await fetch(BASE + url, { method: 'HEAD' });
      if (r.status >= 400) {
        const key = `GET ${url}`;
        if (!failedLinks.has(key)) {
          failedLinks.add(key);
          report({ route: url, method: 'HEAD', code: r.status, title: `Broken link: ${url} → HTTP ${r.status}`, severity: r.status >= 500 ? 'p0' : 'p1', category: 'links', sig: `link-${r.status}` });
        }
      } else pass();
    } catch (e) {
      report({ route: url, method: 'HEAD', code: null, title: `Link fetch error: ${url}`, severity: 'p2', category: 'links', sig: 'fetch-error' });
    }
  }
  console.log(`  crawl visited ${seen.size} URLs, ${failedLinks.size} failed`);
}

// ─── OPDS + content-type deep check ─────────────────────────────────────────
async function opdsDeep() {
  console.log('\n[3/4] OPDS deep check');
  const feeds = ['/opds', '/opds/new', '/opds/popular', '/opds/tags', '/opds/authors', '/opds/shelves'];
  for (const u of feeds) {
    const r = await fetch(BASE + u);
    const ct = r.headers.get('content-type') || '';
    const body = await r.text();
    if (!ct.includes('xml') && !ct.includes('atom')) {
      report({ route: u, code: r.status, title: `OPDS content-type: ${ct}`, severity: 'p1', category: 'opds', sig: 'ct ' + ct });
      continue;
    }
    if (!body.includes('<feed')) {
      report({ route: u, code: r.status, title: `OPDS ${u} not a valid Atom feed`, severity: 'p1', category: 'opds', sig: 'no-feed' });
    }
    // collect links to validate
    const hrefs = [...body.matchAll(/href="([^"]+)"/g)].map(m => m[1]).filter(h => h.startsWith('/'));
    for (const href of hrefs.slice(0, 10)) {
      const rr = await fetch(BASE + href, { method: 'HEAD' });
      if (rr.status >= 400) report({ route: href, method: 'HEAD', code: rr.status, title: `OPDS link broken: ${href}`, severity: 'p1', category: 'opds', sig: 'opds-link-' + rr.status });
      else pass();
    }
  }
}

// ─── Browser E2E: console/network errors on core journeys ───────────────────
async function browserJourneys() {
  console.log('\n[4/4] Browser journeys');
  if (SHOT === 'none') { console.log('  screenshots disabled'); }
  const browser = await chromium.launch({ executablePath: BROWSER, headless: true });
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });

  const journeys = [
    { name: 'home', url: '/', click: null },
    { name: 'search', url: '/search', click: null },
    { name: 'trending', url: '/trending', click: null },
    { name: 'leaderboard', url: '/leaderboard', click: null },
    { name: 'badges', url: '/badges', click: null },
    { name: 'stats', url: '/stats', click: null },
    // auth-required pages: no heading expected when logged out
    { name: 'feed', url: '/feed', click: null, auth: true },
    { name: 'quests', url: '/quests', click: null, auth: true },
    { name: 'shelves', url: '/shelves', click: null, auth: true },
    { name: 'notifications', url: '/notifications', click: null, auth: true },
    { name: 'bookmarks', url: '/bookmarks', click: null, auth: true },
  ];

  for (const j of journeys) {
    const page = await context.newPage();
    const consoleErrors = [];
    const pageErrors = [];
    const badRequests = [];
    const failedAssets = [];

    page.on('console', msg => {
      if (msg.type() !== 'error') return;
      // Auth pages produce expected 401 console noise when unauthenticated.
      if (j.auth && /401|Unauthorized/.test(msg.text())) return;
      consoleErrors.push(msg.text());
    });
    page.on('pageerror', err => pageErrors.push(err.message));
    page.on('requestfailed', req => {
      const u = req.url();
      // Self-navigation abort on auth-redirect pages is a SPA redirect artifact, not a broken asset.
      const selfUrl = BASE + j.url;
      if (u === selfUrl && j.auth) return;
      failedAssets.push(u + ' :: ' + (req.failure()?.errorText || ''));
    });
    page.on('response', res => {
      const u = res.url();
      if (res.status() >= 500 || (res.status() >= 400 && u.startsWith(BASE))) {
        // Auth pages redirect to login; 401/403 from API is expected when unauthenticated.
        if (j.auth && (res.status() === 401 || res.status() === 403)) return;
        if (u.startsWith(BASE) && !u.includes('/api/health')) badRequests.push(`${res.status()} ${u}`);
      }
    });

    try {
      // Auth pages redirect to login; give them a shorter timeout and don't demand networkidle.
      const gotoOpts = j.auth
        ? { waitUntil: 'domcontentloaded', timeout: 10000 }
        : { waitUntil: 'networkidle', timeout: 20000 };
      await page.goto(BASE + j.url, gotoOpts);
      await page.waitForTimeout(800);
      const title = await page.title().catch(() => '');
      const heading = await page.locator('h1, h2').first().textContent().catch(() => '');
      if (!heading && j.url !== '/' && !j.auth) {
        report({ route: j.url, code: 200, title: `Page ${j.url} renders no visible heading`, severity: 'p2', category: 'ui', sig: 'no-heading', urls: [j.url] });
      }
      if (j.click) {
        const el = page.locator(j.click).first();
        if (await el.count()) await el.click().catch(() => {});
        await page.waitForTimeout(600);
      }
    } catch (e) {
      const msg = e.message.split('\n')[0];
      // Auth pages redirect to login; navigation "failure" is the SPA redirect artifact.
      if (!(j.auth && /ERR_ABORTED|frame was detached|Timeout/.test(msg))) {
        report({ route: j.url, code: null, title: `Journey ${j.name} navigation failed: ${msg}`, severity: 'p1', category: 'browser', sig: 'nav-fail' });
      }
    }

    if (consoleErrors.length) {
      report({ route: j.url, code: 200, title: `Console errors on ${j.name} (${consoleErrors.length})`, severity: 'p1', category: 'console', sig: consoleErrors.join('; ').slice(0, 120), evidence: { consoleErrors }, urls: [j.url] });
    }
    if (pageErrors.length) {
      report({ route: j.url, code: 200, title: `Uncaught page errors on ${j.name}`, severity: 'p1', category: 'console', sig: pageErrors[0].slice(0, 120), evidence: { pageErrors }, urls: [j.url] });
    }
    if (badRequests.length) {
      report({ route: j.url, code: 500, title: `Bad network responses on ${j.name}`, severity: 'p0', category: 'network', sig: badRequests[0].slice(0, 120), evidence: { badRequests }, urls: [j.url] });
    }
    if (failedAssets.length) {
      report({ route: j.url, code: null, title: `Failed asset loads on ${j.name}`, severity: 'p2', category: 'assets', sig: failedAssets[0].slice(0, 120), evidence: { failedAssets }, urls: [j.url] });
    }
    if (!consoleErrors.length && !pageErrors.length && !badRequests.length && !failedAssets.length) pass();

    if (SHOT === 'always' || (SHOT === 'on-failure' && (consoleErrors.length || pageErrors.length || badRequests.length))) {
      const f = path.join(ART, `shot-${j.name}-${Date.now()}.png`);
      await page.screenshot({ path: f }).catch(() => {});
    }
    await page.close();
  }

  await browser.close();
  console.log(`  browser journeys done: ${passed} ok, ${failed} findings`);
}

// ─── Backend log scan (journalctl) — turns root causes into queue evidence ──
function backendLogScan() {
  console.log('\n[4/5] Backend log scan');
  try {
    // Only look at the last 15 minutes: stale errors from pre-deploy processes
    // are not current bugs (endpoints may already be fixed).
    const out = execSync('journalctl -u fichub --since "15 min ago" --no-pager 2>/dev/null | grep -iE "ERROR|panic|failed" | tail -100', { encoding: 'utf8', timeout: 20000 });
    if (!out.trim()) { console.log('  no backend errors in last 15 min'); pass(); return; }
    const lines = out.split('\n').filter(Boolean);
    // group by error signature
    const groups = {};
    for (const l of lines) {
      const m = l.match(/ERROR fichub::error: (.+)/);
      if (!m) continue;
      const sig = m[1].replace(/\s+/g, ' ').slice(0, 160);
      groups[sig] = (groups[sig] || 0) + 1;
    }
    for (const [sig, n] of Object.entries(groups)) {
      const sev = /panic|VersionMismatch|must be owner|does not exist/.test(sig) ? 'p0' : 'p1';
      report({ route: '(backend)', code: 500, title: `Backend error ×${n} (last 15min): ${sig.slice(0, 100)}`, severity: sev, category: 'backend', sig: sig.slice(0, 120), evidence: { count: n, sample: sig } });
    }
    console.log(`  scanned ${lines.length} log lines, ${Object.keys(groups).length} error signatures`);
  } catch (e) {
    console.log('  journalctl unavailable, skipping');
  }
}

// ─── Main ───────────────────────────────────────────────────────────────────
async function main() {
  const t0 = Date.now();
  await apiSmoke();
  await crawlLinks();
  await opdsDeep();
  backendLogScan();
  if (!SKIP_BROWSER) await browserJourneys();
  else console.log('\n[4/4] Browser journeys SKIPPED (--skip-browser)');

  // Summary (v2: statuses other than 'open'; fixed → regression auto-escalation)
  const openRows = db.prepare("SELECT severity, COUNT(*) n FROM bugs WHERE status IN ('new','triaged','needs-repro','reproducible','autofix-proposed','autofix-verified','ready-for-cloud','cloud-in-progress','fixed-pending-verify','flaky','regression') GROUP BY severity ORDER BY CASE severity WHEN 'p0' THEN 1 WHEN 'p1' THEN 2 WHEN 'p2' THEN 3 ELSE 4 END").all();
  const newBugs = db.prepare('SELECT COUNT(*) n FROM bugs WHERE first_seen_run = ?').get(runId).n;
  const fixedBugs = db.prepare("SELECT COUNT(*) n FROM bugs WHERE status='fixed'").get().n;
  finishRun.run(now(), openRows.reduce((a, r) => a + r.n, 0), newBugs, fixedBugs, failed ? 1 : 0, runId);
  console.log(`  exit_code: ${failed ? 1 : 0}`);

  console.log('\n══════════════════════════════════════════');
  console.log(`QA run ${runId} complete in ${((Date.now() - t0) / 1000).toFixed(1)}s`);
  console.log(`  checks passed: ${passed}, findings: ${failed}`);
  console.log('  open bugs by severity:');
  for (const r of openRows) console.log(`    ${r.severity}: ${r.n}`);
  console.log(`  queue: ${DB_PATH}`);
  console.log('══════════════════════════════════════════');
  process.exit(failed ? 1 : 0);
}

main().catch(e => { console.error('harness crashed:', e); process.exit(1); });
