-- 072_activitypub: minimal federation tables + keys.
CREATE TABLE IF NOT EXISTS ap_keys (
  actor_type text NOT NULL,
  actor_id bigint NOT NULL,
  private_pem text NOT NULL,
  public_pem text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (actor_type, actor_id)
);
CREATE TABLE IF NOT EXISTS ap_remote_actors (
  actor_uri text PRIMARY KEY,
  last_seen_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS ap_inbox_log (
  activity_id text PRIMARY KEY,
  actor text NOT NULL DEFAULT '',
  payload jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS ap_outbox (
  id bigserial PRIMARY KEY,
  actor_type text NOT NULL,
  actor_id bigint NOT NULL,
  activity jsonb NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);
