// fichub QA report — dump the bug queue from SQLite.
// Usage: node qa/report.js [--json]
import { createRequire } from 'module';
import path from 'path';
const require = createRequire(import.meta.url);
const sqlite3 = require('better-sqlite3');

const ROOT = path.resolve(import.meta.dirname, '..');
const DB_PATH = process.env.QA_DB || path.join(ROOT, 'qa', 'bugs.db');
const db = new sqlite3(DB_PATH);
const json = process.argv.includes('--json');

const rows = db.prepare(`
  SELECT id, fingerprint, title, severity, category, route, method, status_code, occurrences, evidence_json, first_seen_run, last_seen_run, status
  FROM bugs WHERE status NOT IN ('wontfix','duplicate','closed-false-positive')
  ORDER BY CASE severity WHEN 'p0' THEN 1 WHEN 'p1' THEN 2 WHEN 'p2' THEN 3 ELSE 4 END, id
`).all();

if (process.argv.includes('--metrics')) {
  const m = {
    open_by_severity: db.prepare("SELECT severity, COUNT(*) n FROM bugs WHERE status IN ('new','triaged','needs-repro','reproducible','autofix-proposed','autofix-verified','ready-for-cloud','cloud-in-progress','fixed-pending-verify','flaky','regression') GROUP BY severity").all(),
    open_by_category: db.prepare("SELECT category, COUNT(*) n FROM bugs WHERE status IN ('new','triaged','needs-repro','reproducible','autofix-proposed','autofix-verified','ready-for-cloud','cloud-in-progress','fixed-pending-verify','flaky','regression') GROUP BY category").all(),
    fixed_total: db.prepare("SELECT COUNT(*) n FROM bugs WHERE status='fixed'").get().n,
    flaky: db.prepare("SELECT COUNT(*) n FROM bugs WHERE status='flaky'").get().n,
    regression: db.prepare("SELECT COUNT(*) n FROM bugs WHERE status='regression'").get().n,
    autofix_attempts: db.prepare('SELECT COALESCE(SUM(autofix_attempts),0) n FROM bugs').get().n,
    autofix_verified: db.prepare("SELECT COUNT(*) n FROM bugs WHERE status='autofix-verified'").get().n,
    recent_runs: db.prepare('SELECT id, started_at, open_bugs, new_bugs, fixed_bugs, exit_code FROM qa_runs ORDER BY id DESC LIMIT 10').all(),
  };
  console.log(JSON.stringify(m, null, 2));
  process.exit(0);
}

if (json) {
  console.log(JSON.stringify(rows, null, 2));
} else {
  for (const r of rows) {
    console.log(`#${r.id} [${r.severity}/${r.category}] x${r.occurrences} ${r.route} (${r.status_code ?? '-'}) [${r.status}]`);
    console.log(`   ${r.title}`);
    console.log(`   fp: ${r.fingerprint}`);
    if (r.evidence_json && r.evidence_json !== '{}') {
      try { const ev = JSON.parse(r.evidence_json); for (const [k, v] of Object.entries(ev)) {
        if (Array.isArray(v) && v.length > 3) console.log(`   ${k}: [${v.length} items] ${v.slice(0, 3).join(' | ')}...`);
        else if (Array.isArray(v)) console.log(`   ${k}: ${v.join(' | ')}`);
        else if (typeof v === 'object') console.log(`   ${k}: ${JSON.stringify(v).slice(0, 150)}`);
        else console.log(`   ${k}: ${String(v).slice(0, 150)}`);
      }} catch {}
    }
    console.log('');
  }
  console.log(`total: ${rows.length} open`);
}
