-- qa/schema.sql — strict bug lifecycle schema (v2)
-- Applied by qa/mark.js and qa/run.js on startup (idempotent ALTERs for v1 DBs).

PRAGMA journal_mode=WAL;
PRAGMA busy_timeout=5000;

CREATE TABLE IF NOT EXISTS qa_runs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  started_at TEXT, finished_at TEXT, git_commit TEXT, git_branch TEXT,
  mode TEXT, open_bugs INTEGER, new_bugs INTEGER, fixed_bugs INTEGER, exit_code INTEGER
);

CREATE TABLE IF NOT EXISTS bugs (
  id TEXT PRIMARY KEY,
  fingerprint TEXT UNIQUE NOT NULL,
  title TEXT NOT NULL,
  severity TEXT NOT NULL DEFAULT 'unknown',
  status TEXT NOT NULL DEFAULT 'new',
  category TEXT NOT NULL DEFAULT 'unknown',
  route TEXT,
  method TEXT,
  status_code INTEGER,
  first_seen_run INTEGER,
  last_seen_run INTEGER,
  occurrences INTEGER NOT NULL DEFAULT 1,
  repro_command TEXT,
  repro_test_path TEXT,
  evidence_json TEXT,
  suspected_files_json TEXT,
  autofix_attempts INTEGER NOT NULL DEFAULT 0,
  cloud_attempts INTEGER NOT NULL DEFAULT 0,
  flaky_count INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  closed_at TEXT,
  resolution TEXT
);

CREATE TABLE IF NOT EXISTS bug_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  bug_id TEXT NOT NULL,
  run_id INTEGER,
  old_status TEXT, new_status TEXT, note TEXT, created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_bugs_status ON bugs(status);
CREATE INDEX IF NOT EXISTS idx_bugs_severity ON bugs(severity);
CREATE INDEX IF NOT EXISTS idx_bugs_fingerprint ON bugs(fingerprint);

-- v1→v2 idempotent migrations (fresh DBs have these already)
-- (bug id column was INTEGER AUTOINCREMENT in v1; v2 uses TEXT ids like BUG-0001)
