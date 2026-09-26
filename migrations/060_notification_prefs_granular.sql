-- 060: granular per-work notification preference toggles.
--
-- The NotificationPreference struct, get_notification_preferences, the GET
-- handler, the PUT handler, update_notification_preferences and the frontend
-- settings page all read and write six toggles that no migration ever created:
--
--     comments_on_work, replies_to_comments, kudos_on_work,
--     bookmarks_on_work, follows, mentions
--
-- So GET /api/notifications/preferences returned
--
--     {"err":-1,"msg":"database error"}
--
-- for every authenticated user, PUT and the delivery paths failed the same
-- way, and the settings page rendered six toggles that could not be saved.
--
-- The model documented these as "migration 060". This is migration 060. It was
-- never written; the sequence jumps 059 -> 064.
--
-- The six columns are NOT vestigial. The settings page binds a checkbox to
-- each one with a translated label (frontend/src/routes/settings/+page.svelte,
-- t('settings.notifKudosOnWork') and five siblings) and saves on change, so
-- this is shipped, user-facing behaviour -- not dead weight to be trimmed. They
-- are also read as a set by NotificationPreference, so they cannot be dropped
-- without changing the model, the three queries, two handlers and the UI
-- together.
--
-- Defaults are TRUE, matching every other toggle in this table, so an existing
-- user gains working toggles without opting in and without a behaviour change:
-- the delivery paths gate on comment_reply and work_update (both of which
-- already existed), so nothing starts or stops sending as a result of this
-- migration. A user who turns a toggle off gets a row where the column is
-- FALSE, which is exactly what the UPDATE has been trying to write.

ALTER TABLE public.notification_preferences
    ADD COLUMN IF NOT EXISTS comments_on_work   boolean DEFAULT true  NOT NULL,
    ADD COLUMN IF NOT EXISTS replies_to_comments boolean DEFAULT true  NOT NULL,
    ADD COLUMN IF NOT EXISTS kudos_on_work      boolean DEFAULT true  NOT NULL,
    ADD COLUMN IF NOT EXISTS bookmarks_on_work  boolean DEFAULT true  NOT NULL,
    ADD COLUMN IF NOT EXISTS follows            boolean DEFAULT true  NOT NULL,
    ADD COLUMN IF NOT EXISTS mentions           boolean DEFAULT true  NOT NULL;

COMMENT ON COLUMN public.notification_preferences.comments_on_work IS
  'Notify when someone comments on a work I authored.';
COMMENT ON COLUMN public.notification_preferences.replies_to_comments IS
  'Notify when someone replies to a comment I wrote.';
COMMENT ON COLUMN public.notification_preferences.kudos_on_work IS
  'Notify when someone kudos (likes) a work I authored.';
COMMENT ON COLUMN public.notification_preferences.bookmarks_on_work IS
  'Notify when someone bookmarks a work I authored.';
COMMENT ON COLUMN public.notification_preferences.follows IS
  'Notify when someone follows me.';
COMMENT ON COLUMN public.notification_preferences.mentions IS
  'Notify when I am mentioned.';
