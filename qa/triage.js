// fichub QA triage worker — local LLM (ollama) enriches bug queue.
// Deterministic harness finds facts; this adds human-readable summaries + suspected files.
// Usage: node qa/triage.js [--dry-run]
import { createRequire } from 'module';
import path from 'path';
import { execSync } from 'child_process';
import fs from 'fs';

const require = createRequire(import.meta.url);
const sqlite3 = require('better-sqlite3');

const ROOT = path.resolve(import.meta.dirname, '..');
const DB_PATH = process.env.QA_DB || path.join(ROOT, 'qa', 'bugs.db');
const OLLAMA = process.env.OLLAMA_URL || 'http://127.0.0.1:11434';
const MODEL = process.env.QA_MODEL || 'llama3.2:3b';
const db = new sqlite3(DB_PATH);
try { db.exec("ALTER TABLE bugs ADD COLUMN triage TEXT"); } catch {} // idempotent

// collect evidence for open bugs
const bugs = db.prepare("SELECT id, title, severity, category, route, evidence_json, fingerprint FROM bugs WHERE status IN ('new','triaged','needs-repro','reproducible','autofix-proposed','autofix-verified','ready-for-cloud','cloud-in-progress','fixed-pending-verify','flaky','regression') ORDER BY CASE severity WHEN 'p0' THEN 1 WHEN 'p1' THEN 2 WHEN 'p2' THEN 3 ELSE 4 END").all();

const summarize = async (bug) => {
  const ev = (() => { try { return JSON.parse(bug.evidence_json || '{}'); } catch { return {}; } })();
  const prompt = `You are a bug triage assistant for a Rust/Axum + SvelteKit fanfiction website (fichub).
Given a deterministic test finding, produce a SHORT bug report. Output JSON only with these fields:
- title (concise, <80 chars)
- summary (2-3 sentences, what breaks and why it matters)
- suspected_files (array of file paths, relative to repo root, may be empty)
- likely_cause (1 sentence, may be "unknown")
- severity: p0|p1|p2|p3
- next_repro_step (1 concrete action to confirm the bug)

Do not invent facts. If unsure, use "unknown".

Finding:
severity: ${bug.severity}
category: ${bug.category}
route: ${bug.route}
title: ${bug.title}
evidence: ${JSON.stringify(ev).slice(0, 600)}`;

  const res = await fetch(OLLAMA + '/api/generate', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ model: MODEL, prompt, stream: false, options: { temperature: 0.1, num_predict: 300 } }),
  });
  const j = await res.json();
  const text = j.response || '';
  try {
    const m = text.match(/\{[\s\S]*\}/);
    return JSON.parse(m ? m[0] : text);
  } catch {
    return { title: bug.title, summary: text.slice(0, 300), suspected_files: [], likely_cause: 'unknown', severity: bug.severity, next_repro_step: 'unknown' };
  }
};

const dry = process.argv.includes('--dry-run');
const out = [];
for (const bug of bugs) {
  process.stdout.write(`  triaging #${bug.id} [${bug.severity}] ${bug.title.slice(0, 50)}... `);
  try {
    const s = await summarize(bug);
    out.push({ id: bug.id, ...s });
    process.stdout.write('ok\n');
    if (!dry) {
      db.prepare('UPDATE bugs SET triage = ? WHERE id = ?').run(JSON.stringify(s), bug.id);
    }
  } catch (e) {
    process.stdout.write(`FAILED: ${e.message.slice(0, 60)}\n`);
  }
}
console.log(`\ntriage complete: ${out.length}/${bugs.length} enriched`);
if (dry) console.log(JSON.stringify(out, null, 2));
