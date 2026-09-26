-- Administrator as a distinct capability, not a rung on the trust ladder.
--
-- 013_trust_levels.sql defines trust_level as CHECK (trust_level BETWEEN 0 AND 6),
-- and JWT issuance mints the trust_level claim from that column
-- (src/routes/auth.rs). No account can therefore log in with a token carrying
-- trust_level >= 10, so admin-tier routes gated on a trust threshold were
-- unreachable in production while their tests passed - the tests mint tokens by
-- hand. Full analysis: docs/specs/admin-tier-unreachable.md.
--
-- The owner chose option C: administrator is a flag, and trust_level stays the
-- 0-6 community ladder exactly as 013 defines it.
-- See docs/specs/admin-flag.md.
--
-- users.role is deliberately untouched. It is a separate legacy trust ladder
-- holding the same 5 and 10 rungs, written only by subsystems.rs on account
-- activation and read only as profile data for display. Deprecating it is
-- separate work.
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_admin BOOLEAN NOT NULL DEFAULT false;

-- The admin user list reads this, and administrators are a tiny fraction of the
-- table, so a partial index is smaller than a full one.
CREATE INDEX IF NOT EXISTS users_is_admin_idx ON users (id) WHERE is_admin;

COMMENT ON COLUMN users.is_admin IS
  'Administrator. Distinct from trust_level (0-6 community ladder) and from the legacy users.role.';
