// qa/gen-workpackets.js — per-bug work packets for the cloud fixer (compact context).
// Usage: node qa/gen-workpackets.js [--severity p0,p1] [--max 10]
import { createRequire } from 'module';
import path from 'path';
import fs from 'fs';
const require = createRequire(import.meta.url);
const sqlite3 = require('better-sqlite3');

const ROOT = path.resolve(import.meta.dirname, '..');
const DB = path.join(ROOT, 'qa', 'bugs.db');
const OUT = path.join(ROOT, 'qa', 'reports', 'workpackets');
fs.mkdirSync(OUT, { recursive: true });
const db = new sqlite3(DB);

const sevIdx = process.argv.indexOf('--severity');
const sevs = sevIdx >= 0 ? process.argv[sevIdx + 1].split(',') : ['p0', 'p1'];
const maxIdx = process.argv.indexOf('--max');
const max = maxIdx >= 0 ? parseInt(process.argv[maxIdx + 1]) : 10;

const bugs = db.prepare(`
  SELECT * FROM bugs WHERE status NOT IN ('fixed','wontfix','duplicate','closed-false-positive')
  ORDER BY CASE severity WHEN 'p0' THEN 1 WHEN 'p1' THEN 2 ELSE 3 END, occurrences DESC
`).all().filter(b => sevs.includes(b.severity)).slice(0, max);

for (const b of bugs) {
  const ev = (() => { try { return JSON.parse(b.evidence_json || b.evidence || '{}'); } catch { return {}; } })();
  const files = (() => { try { return JSON.parse(b.suspected_files_json || '[]'); } catch { return []; } })();
  const md = `# ${b.id}

- Severity: ${b.severity}
- Category: ${b.category}
- Status: ${b.status}
- Fingerprint: \`${b.fingerprint}\`
- First seen: ${b.first_seen_run || '?'} · Last seen: ${b.last_seen_run || '?'} · Occurrences: ${b.occurrences}

## Repro

\`\`\`
${b.repro_command || 'curl -sS -o /dev/null -w "%{http_code}" http://localhost:8000' + (b.route && !b.route.startsWith('(backend)') ? b.route : '/api/health') + '  # expected 200'}
\`\`\`

Expected: 200 (or appropriate status)
Actual: ${b.status_code || 'unknown'}

## Evidence

\`\`\`json
${JSON.stringify(ev, null, 2).slice(0, 500)}
\`\`\`

## Suspected files

${files.length ? files.map(f => '- ' + f).join('\n') : '- unknown — grep evidence'}

## Regression test required

Add a test that hits ${b.route} and expects the correct behavior (fingerprint must disappear).

## Acceptance

- Fingerprint absent from ./qa.sh output
- Regression test fails before fix, passes after
- No new journalctl errors
- Status updated via: node qa/mark.js ${b.id} fixed-pending-verify --test <path>
`;
  fs.writeFileSync(path.join(OUT, b.id + '.md'), md);
}
console.log(`wrote ${bugs.length} work packets to ${OUT}`);
