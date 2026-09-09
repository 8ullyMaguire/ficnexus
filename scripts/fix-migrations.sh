#!/bin/bash
# Mark migrations 90-93 as already applied to production DB
# These migrations created XP tables that still exist in production
# but the code no longer references them. This prevents sqlx from trying to re-run them.

DB_NAME="ficnexus"
DB_USER="postgres"

psql -U $DB_USER -d $DB_NAME -c "
INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time)
VALUES
  (90, 'forum_xp_sources', NOW(), true, decode('0000000000000000000000000000000000000000000000000000000000000000','hex'), 0),
  (91, 'achievement_definitions', NOW(), true, decode('0000000000000000000000000000000000000000000000000000000000000000','hex'), 0),
  (92, 'trust_level_migration', NOW(), true, decode('0000000000000000000000000000000000000000000000000000000000000000','hex'), 0),
  (93, 'critical_audit_fixes', NOW(), true, decode('0000000000000000000000000000000000000000000000000000000000000000','hex'), 0);
"
