#!/usr/bin/env node
// qa/mark.js — status transitions CLI. The ONLY way agents/humans change bug status.
// Usage: node qa/mark.js <BUG-ID> <new-status> [--reason "..."] [--test path]
import { createRequire } from 'module';
import path from 'path';
import { execSync } from 'child_process';
import fs from 'fs';
const require = createRequire(import.meta.url);
const sqlite3 = require('better-sqlite3');

const ROOT = path.resolve(import.meta.dirname, '..');
const DB = path.join(ROOT, 'qa', 'bugs.db');
const db = new sqlite3(DB);
db.exec(fs.readFileSync(path.join(ROOT, 'qa', 'schema.sql'), 'utf8'));

const VALID = ['new', 'triaged', 'needs-repro', 'reproducible', 'flaky', 'autofix-proposed', 'autofix-verified', 'autofix-rejected', 'ready-for-cloud', 'cloud-in-progress', 'fixed-pending-verify', 'fixed', 'needs-human', 'duplicate', 'wontfix', 'regression', 'env-failure'];

const [bugId, newStatus] = process.argv.slice(2);
const reasonIdx = process.argv.indexOf('--reason');
const reason = reasonIdx >= 0 ? process.argv[reasonIdx + 1] : '';
const testIdx = process.argv.indexOf('--test');
const testPath = testIdx >= 0 ? process.argv[testIdx + 1] : '';

if (!bugId || !newStatus) { console.error('usage: node qa/mark.js <BUG-ID> <status> [--reason ..] [--test path]'); process.exit(1); }
if (!VALID.includes(newStatus)) { console.error('invalid status. valid: ' + VALID.join(', ')); process.exit(1); }

const bug = db.prepare('SELECT * FROM bugs WHERE id = ?').get(bugId);
if (!bug) { console.error('bug not found: ' + bugId); process.exit(1); }

// guards
if (newStatus === 'fixed-pending-verify' || newStatus === 'fixed') {
  if (bug.repro_test_path && !fs.existsSync(path.join(ROOT, bug.repro_test_path))) {
    console.error('refusing: repro_test_path missing: ' + bug.repro_test_path); process.exit(1);
  }
}
if (testPath) {
  if (!fs.existsSync(path.join(ROOT, testPath))) { console.error('test path missing: ' + testPath); process.exit(1); }
  db.prepare('UPDATE bugs SET repro_test_path = ? WHERE id = ?').run(testPath, bugId);
}
if (newStatus === 'fixed') {
  // require a repro test on record
  const hasTest = (bug.repro_test_path || testPath);
  if (!hasTest) { console.error('refusing: cannot mark fixed without repro_test_path (use --test)'); process.exit(1); }
}

db.prepare('UPDATE bugs SET status = ?, updated_at = ?, resolution = CASE WHEN ? IN ("fixed","wontfix","duplicate") THEN ? ELSE resolution END, closed_at = CASE WHEN ? IN ("fixed","wontfix","duplicate") THEN datetime("now") ELSE closed_at END WHERE id = ?')
  .run(newStatus, new Date().toISOString(), newStatus, reason || newStatus, newStatus, bugId);
db.prepare('INSERT INTO bug_events (bug_id, old_status, new_status, note, created_at) VALUES (?,?,?,?,?)')
  .run(bugId, bug.status, newStatus, reason, new Date().toISOString());
console.log(`${bugId}: ${bug.status} → ${newStatus}${reason ? ' (' + reason + ')' : ''}`);
