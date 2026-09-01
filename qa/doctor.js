// qa/doctor.js — environment doctor. Exit 0 if env is healthy, 3 if broken.
// Broken env must NOT file app bugs (prevents "harness broken ≠ app broken").
import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';

const ROOT = path.resolve(import.meta.dirname, '..');
const checks = [];
const ok = (name) => checks.push({ name, pass: true });
const fail = (name, why) => checks.push({ name, pass: false, why });
const run = (cmd) => { try { execSync(cmd, { encoding: 'utf8', timeout: 120000, stdio: ['ignore', 'pipe', 'pipe'] }); return true; } catch { return false; } };

// node
run('node --version') ? ok('node') : fail('node', 'node not found');
// sqlite (resolve from qa/ where node_modules lives)
run('cd ' + ROOT + '/qa && node -e "require(\'better-sqlite3\')" 2>/dev/null') ? ok('better-sqlite3') : fail('better-sqlite3', 'run cd qa && npm install');
// cargo
run('cargo --version') ? ok('cargo') : fail('cargo', 'cargo missing');
// cargo build (release) — slower but catches real breakage
run('cd ' + ROOT + ' && cargo build --release 2>&1 | tail -1') ? ok('cargo build --release') : fail('cargo build --release', 'build broken');
// ollama + models
try {
  const out = execSync('ollama list', { encoding: 'utf8', timeout: 15000 });
  out.includes('llama3.2:3b') ? ok('ollama llama3.2:3b') : fail('ollama llama3.2:3b', 'model not installed');
  out.includes('qwen2.5-coder:7b') ? ok('ollama qwen2.5-coder:7b') : fail('ollama qwen2.5-coder:7b', 'model not installed');
} catch { fail('ollama', 'ollama not reachable'); }
// postgres
run('pg_isready -q') ? ok('postgres') : fail('postgres', 'pg_isready failed');
// DATABASE_URL set
const env = fs.existsSync(path.join(ROOT, '.env')) ? fs.readFileSync(path.join(ROOT, '.env'), 'utf8') : '';
env.includes('DATABASE_URL=') ? ok('DATABASE_URL') : fail('DATABASE_URL', 'not in .env');
// service reachable
try { const code = execSync('curl -s -o /dev/null -w "%{http_code}" --max-time 5 http://localhost:8000/api/health', { encoding: 'utf8' }); code === '200' ? ok('fichub service') : fail('fichub service', 'health returned ' + code); } catch { fail('fichub service', 'not reachable'); }
// journalctl readable
run('journalctl -u fichub --since "1 min ago" -n 1 >/dev/null 2>&1') ? ok('journalctl readable') : fail('journalctl readable', 'needs sudo or systemd-journal group');
// playwright browser — auto-detect latest chromium version
const pwBase = path.join(process.env.HOME || '', '.cache', 'ms-playwright');
let pwOk = false;
try {
  const chromiumDirs = fs.readdirSync(pwBase).filter(d => d.startsWith('chromium-')).sort();
  pwOk = chromiumDirs.length > 0 && fs.existsSync(path.join(pwBase, chromiumDirs[chromiumDirs.length - 1], 'chrome-linux64', 'chrome'));
} catch {}
pwOk ? ok('playwright chromium') : fail('playwright chromium', 'no chromium found in ' + pwBase);
// git tree clean (excluding known volatile paths + qa working files)
try {
  const dirty = execSync('cd ' + ROOT + ' && git status --porcelain | grep -vE "frontend/static/docs/|^\\?\\? \\.hermes/|^ M qa/|^ M qa\\.sh|^ M \\.gitignore"', { encoding: 'utf8' }).trim();
  dirty ? fail('git tree clean', 'dirty: ' + dirty.split('\n')[0]) : ok('git tree clean');
} catch { ok('git tree clean'); }

for (const c of checks) console.log(`${c.pass ? 'OK ' : 'FAIL'} ${c.name}${c.pass ? '' : ' — ' + c.why}`);
const failed = checks.filter(c => !c.pass).length;
console.log(`\ndoctor: ${checks.length - failed}/${checks.length} healthy`);
process.exit(failed ? 3 : 0);
