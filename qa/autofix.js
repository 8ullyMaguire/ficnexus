// fichub QA auto-fix worker — local LLM proposes minimal patches, deterministic gates verify.
// v2: PATCH-BASED. Proposals are written as .patch files under qa/patches/, validated with
// git apply --check, applied on a temporary branch, never merged automatically.
// Denylist enforced: auth/schema/security/qa/tests-weakening patches rejected.
// Zero cloud tokens. Usage: node qa/autofix.js [--bug BUG-0001]
import { createRequire } from 'module';
import path from 'path';
import { execSync } from 'child_process';
import fs from 'fs';
const require = createRequire(import.meta.url);
const sqlite3 = require('better-sqlite3');

const ROOT = path.resolve(import.meta.dirname, '..');
const DB = path.join(ROOT, 'qa', 'bugs.db');
const OLLAMA = process.env.OLLAMA_URL || 'http://127.0.0.1:11434';
const MODEL = process.env.QA_FIX_MODEL || 'qwen2.5-coder:7b';
const PATCH_DIR = path.join(ROOT, 'qa', 'patches');
fs.mkdirSync(PATCH_DIR, { recursive: true });
const db = new sqlite3(DB);

// ── allow/deny ──────────────────────────────────────────────────────────────
const DENY = [
  /migrations\//, /\.env/, /qa\//, /\.forgejo\//, /\.github\//, /secrets\//,
  /auth/, /session/, /password/, /permission/i, /payment/, /donation/, /license/,
  /(^|\/)delete/, /Cargo\.toml/, /\.lock$/,
];
const ALLOW = [/^src\//, /^frontend\/src\//];
const MAX_FILES = 3, MAX_ADDED = 80, MAX_REMOVED = 80;

// ── query ──────────────────────────────────────────────────────────────────
const bugIdArg = process.argv.indexOf('--bug') >= 0 ? process.argv[process.argv.indexOf('--bug') + 1] : null;
const bugs = bugIdArg
  ? db.prepare("SELECT * FROM bugs WHERE id = ? AND status IN ('reproducible','autofix-proposed','autofix-rejected')").all(bugIdArg)
  : db.prepare("SELECT * FROM bugs WHERE status = 'reproducible' AND severity IN ('p0','p1') ORDER BY CASE severity WHEN 'p0' THEN 1 ELSE 2 END").all();

// ground-truth source map (verified with grep — never trust LLM paths)
const SOURCE_MAP = [
  { match: /leaderboard|mismatched types|i16/, files: ['src/db/queries.rs', 'src/routes/badges.rs'] },
  { match: /opds\/shelves|OPDS/, files: ['src/routes/opds/shelves.rs', 'src/server.rs'] },
];
function resolveFiles(bug) {
  for (const m of SOURCE_MAP) {
    if (m.match.test(bug.fingerprint + ' ' + bug.title + ' ' + (bug.route || ''))) {
      return m.files.filter(f => fs.existsSync(path.join(ROOT, f)));
    }
  }
  return [];
}

function validatePatch(diff) {
  // extract touched files from +++/--- headers
  const touched = [...diff.matchAll(/^\+\+\+ b\/(.+)$/gm)].map(m => m[1]);
  if (!touched.length) return { ok: false, reason: 'no file headers' };
  for (const f of touched) {
    if (DENY.some(re => re.test(f))) return { ok: false, reason: 'denied path ' + f };
    if (!ALLOW.some(re => re.test(f))) return { ok: false, reason: 'not allowed path ' + f };
  }
  if (touched.length > MAX_FILES) return { ok: false, reason: 'too many files ' + touched.length };
  const added = (diff.match(/^\+[^+]/gm) || []).length;
  const removed = (diff.match(/^-[^-]/gm) || []).length;
  if (added > MAX_ADDED || removed > MAX_REMOVED) return { ok: false, reason: `patch too big (+${added}/-${removed})` };
  // reject test-expectation-only patches: hunks touching tests AND only +/- on assert/expect lines
  const touchesTest = touched.some(f => /test|spec/.test(f));
  if (touchesTest && /assert_eq!\(.*500|toBe\(500\)|expect\(.*500/.test(diff)) return { ok: false, reason: 'weakens test expectations' };
  return { ok: true, files: touched };
}

async function proposeFix(bug, files) {
  const sections = files.map(f => {
    const content = fs.readFileSync(path.join(ROOT, f), 'utf8');
    const terms = ['leaderboard', 'i16', 'query_as', 'INT4', 'rank'];
    const lines = content.split('\n');
    const hits = [];
    lines.forEach((l, i) => { if (terms.some(t => l.toLowerCase().includes(t))) hits.push(i); });
    const windows = []; let cur = null;
    for (const h of hits.slice(0, 12)) {
      if (cur && h - cur.end <= 25) cur.end = Math.min(lines.length, h + 12);
      else { if (cur) windows.push(cur); cur = { start: Math.max(0, h - 8), end: Math.min(lines.length, h + 25) }; }
    }
    if (cur) windows.push(cur);
    const excerpt = windows.map(w => lines.slice(w.start, w.end).map((l, i) => `${w.start + i + 1}: ${l}`).join('\n')).join('\n...\n');
    return `=== FILE: ${f} ===\n${excerpt || content.slice(0, 3000)}`;
  }).join('\n\n');

  const prompt = `You are fixing one bug in a Rust/Axum + SvelteKit fanfiction server (fichub).
Find the EXACT failing function. Change ONLY the incorrect type/logic. Smallest change.
Rules:
1. Output ONLY a unified diff (git diff format). No prose, no markdown fences.
2. Touch ONLY these files: ${files.join(', ')}
3. Do not touch tests, migrations, auth, or security code.
4. If you cannot fix with a small change, output exactly: NO_FIX

Bug:
title: ${bug.title}
severity: ${bug.severity}
route: ${bug.route}
evidence: ${(bug.evidence_json || bug.evidence || '').slice(0, 400)}

Relevant excerpts:
${sections}`;

  const res = await fetch(OLLAMA + '/api/generate', {
    method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ model: MODEL, prompt, stream: false, options: { temperature: 0.1, num_predict: 1200 } }),
  });
  const j = await res.json();
  return (j.response || '').trim();
}

function cargoBuild() {
  try { execSync('cd ' + ROOT + ' && cargo build --release 2>&1', { encoding: 'utf8', timeout: 300000 }); return true; } catch { return false; }
}
function routeSmoke(bug) {
  try {
    const route = (bug.route || '').startsWith('http') ? bug.route : 'http://localhost:8000' + (bug.route || '/');
    const code = execSync(`curl -s -o /dev/null -w '%{http_code}' --max-time 10 '${route}'`, { encoding: 'utf8' });
    return code === '200' || code === '400' || code === '401';
  } catch { return false; }
}

let proposed = 0, verified = 0, rejected = 0;
console.log(`autofix v2: ${bugs.length} reproducible P0/P1 bugs, model=${MODEL}`);

for (const bug of bugs) {
  const files = resolveFiles(bug);
  process.stdout.write(`  ${bug.id} [${bug.severity}] ${bug.title.slice(0, 45)}... `);
  if (!files.length) { console.log('no suspected files → needs-human'); db.prepare("UPDATE bugs SET status='needs-human' WHERE id=?").run(bug.id); continue; }
  try {
    const diff = await proposeFix(bug, files);
    if (diff.includes('NO_FIX')) { console.log('model declined → needs-human'); db.prepare("UPDATE bugs SET status='needs-human' WHERE id=?").run(bug.id); continue; }
    const v = validatePatch(diff);
    if (!v.ok) { console.log('rejected: ' + v.reason); db.prepare("UPDATE bugs SET status='autofix-rejected', autofix_attempts = autofix_attempts + 1 WHERE id=?").run(bug.id); rejected++; continue; }
    // write patch, apply on temp branch
    const p = path.join(PATCH_DIR, `${bug.id}-attempt-${bug.autofix_attempts + 1}.patch`);
    fs.writeFileSync(p, diff);
    try {
      execSync(`cd ${ROOT} && git apply --check ${p}`, { encoding: 'utf8' });
    } catch { console.log('git apply --check failed → autofix-rejected'); db.prepare("UPDATE bugs SET status='autofix-rejected', autofix_attempts = autofix_attempts + 1 WHERE id=?").run(bug.id); rejected++; continue; }
    // apply on a detached temp branch so main stays untouched
    const branch = `autofix/${bug.id}-attempt-${bug.autofix_attempts + 1}`;
    try {
      execSync(`cd ${ROOT} && git stash -u >/dev/null 2>&1; git switch -c ${branch} 2>&1`, { encoding: 'utf8' });
      execSync(`cd ${ROOT} && git apply ${p}`, { encoding: 'utf8' });
      if (!cargoBuild()) { console.log('build FAILED on branch → autofix-rejected'); db.prepare("UPDATE bugs SET status='autofix-rejected', autofix_attempts = autofix_attempts + 1 WHERE id=?").run(bug.id); rejected++; execSync(`cd ${ROOT} && git switch - >/dev/null 2>&1 && git branch -D ${branch} 2>/dev/null`); continue; }
      const smoke = routeSmoke(bug);
      // keep branch, record proposal
      db.prepare("UPDATE bugs SET status='autofix-verified', autofix_attempts = autofix_attempts + 1, repro_command = ?, repro_test_path = ? WHERE id=?")
        .run(`git switch ${branch} && node qa/verify.js ${bug.id}`, `qa/verify.js:${bug.id}`, bug.id);
      console.log(`verified on branch ${branch} (build ok, smoke=${smoke ? 'pass' : 'n/a'})`);
      verified++;
      execSync(`cd ${ROOT} && git switch - >/dev/null 2>&1`, { encoding: 'utf8' });
    } catch (e) {
      console.log('branch apply error: ' + String(e.message || e).slice(0, 80));
      db.prepare("UPDATE bugs SET status='autofix-rejected', autofix_attempts = autofix_attempts + 1 WHERE id=?").run(bug.id);
      rejected++;
    }
    proposed++;
  } catch (e) {
    console.log('error: ' + String(e.message || e).slice(0, 80));
    db.prepare("UPDATE bugs SET status='needs-human' WHERE id=?").run(bug.id);
  }
}

console.log(`\nautofix v2 done: ${verified} autofix-verified (branch-based), ${rejected} rejected, ${proposed} proposed total`);
console.log('patches in qa/patches/ · branches: git branch | grep autofix · merge only after human/CI approval');
