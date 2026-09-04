// qa/verify.js — targeted verification for one bug. Part of the fixed gate.
// Usage: node qa/verify.js BUG-0001
// Runs: repro command (if recorded), regression test (if recorded), route smoke.
// Prints PASS/FAIL per gate. Exit 0 if all gates pass.
import { createRequire } from 'module';
import path from 'path';
import { execSync } from 'child_process';
import fs from 'fs';
const require = createRequire(import.meta.url);
const sqlite3 = require('better-sqlite3');

const ROOT = path.resolve(import.meta.dirname, '..');
const DB = path.join(ROOT, 'qa', 'bugs.db');
const db = new sqlite3(DB);
const bugId = process.argv[2];
if (!bugId) { console.error('usage: node qa/verify.js BUG-0001'); process.exit(1); }
const bug = db.prepare('SELECT * FROM bugs WHERE id = ?').get(bugId);
if (!bug) { console.error('bug not found'); process.exit(1); }

let allPass = true;
const gate = (name, pass, detail = '') => {
  console.log(`${pass ? 'PASS' : 'FAIL'} ${name}${detail ? ' — ' + detail : ''}`);
  if (!pass) allPass = false;
};

// 1. repro command
if (bug.repro_command) {
  try { execSync(`cd ${ROOT} && ${bug.repro_command}`, { encoding: 'utf8', timeout: 60000 }); gate('repro command', true); }
  catch (e) { gate('repro command', false, String(e.message || e).slice(0, 80)); }
} else { console.log('SKIP repro command (none recorded)'); }

// 2. regression test (recorded path)
if (bug.repro_test_path) {
  const p = path.join(ROOT, bug.repro_test_path.split(':')[0]);
  if (!fs.existsSync(p)) { gate('regression test', false, 'missing ' + p); }
  else {
    try { execSync(`cd ${ROOT} && node ${p} 2>&1`, { encoding: 'utf8', timeout: 120000 }); gate('regression test', true); }
    catch (e) { gate('regression test', false, String(e.message || e).slice(0, 80)); }
  }
} else { console.log('SKIP regression test (none recorded)'); }

// 3. route smoke
if (bug.route && !bug.route.startsWith('(backend)')) {
  try {
    const route = bug.route.startsWith('http') ? bug.route : 'http://localhost:8000' + bug.route;
    const code = execSync(`curl -s -o /dev/null -w '%{http_code}' --max-time 10 '${route}'`, { encoding: 'utf8' });
    gate('route smoke', code === '200' || code === '400' || code === '401', 'HTTP ' + code);
  } catch { gate('route smoke', false, 'curl failed'); }
}

console.log(allPass ? '\nverify: ALL GATES PASS' : '\nverify: SOME GATES FAIL');
process.exit(allPass ? 0 : 1);
