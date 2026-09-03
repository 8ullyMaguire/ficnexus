# Data Model: OTW Archive Parity

## Entities

### Work (existing, extended)
- **Fields**: title, summary, notes, language, complete (bool), word_count, revised_at, expected_tags (fandoms/relationships/characters/freeforms), rating, warnings, categories, stats (kudos/bookmarks/hits/comments), chapters, series_id, collection_items.
- **Relations**: has_many chapters, pseuds via creatorships, series, collections via collection_items, kudos, bookmarks, comments, subscriptions, skins.
- **Validation**: title required; rating/warnings/category required per OTW; at least one fandom.

### Pseud (new)
- **Fields**: user_id FK, name, description, is_default, created_at.
- **Relations**: belongs_to user, has_many creatorships, has_many works/series via creatorships.
- **Validation**: name unique per user, format like OTW (no spaces edge cases), one default per user.

### Creatorship (join, new)
- **Fields**: work_id or series_id, pseud_id, approved (bool), invited_at.
- **Relations**: between work/series and pseud.

### Series (existing or new)
- **Fields**: title, summary, notes, pseuds via creatorships, ordered works.
- **Relations**: has_many works (ordered), belongs_to pseuds.

### Collection (existing or extended)
- **Fields**: title, name (slug), summary, moderated, closed, challenge-associated, prefs.
- **Relations**: has_many collection_items, has_many works via items, has_many participants.

### CollectionItem (new/extended)
- **Fields**: collection_id, work_id or bookmark_id, user_id, status (approved/pending).
- **Validation**: moderated collections require approval.

### Bookmark (existing)
- **Fields**: user_id, work_id, pseud_id, tags, notes, private, rec, created_at.
- **Relations**: belongs_to work + pseud.

### Kudo (existing)
- **Fields**: work_id, user_id (nullable for guest), ip hash for guest dedup.
- **Validation**: one per user per work.

### Subscription (existing, possibly extended)
- **Fields**: user_id, subscribable_type (Work/Series/User/Pseud), subscribable_id.
- **Validation**: unique per user+target.

### Comment / Thread / Inbox (existing)
- **Fields**: work_id/chapter_id, parent_id, pseud_id/user_id, content, guest fields, inbox copy.
- **Relations**: nested set via parent_id, inbox per user.
- **Validation**: guest allowed where work allows; threaded depth unlimited but rendered nested.

### Tag / TagWrangling (existing/extended)
- **Fields**: name, type (Fandom/Relationship/Character/Freeform), canonical_id, synonyms, wrangled.
- **Relations**: has_many works via taggings.
- **Validation**: canonical self-link; synonyms point to canonical.

### Skin (new)
- **Fields**: title, css, is_work_skin, pseud_id/user_id, public, cached.
- **Relations**: belongs_to user/pseud, has_many work_skin assignments.
- **Validation**: CSS sanitized (allowlist), preview before apply.

## State transitions

- **Work**: draft → posted → hidden (moderated) → revised (new chapter).
- **Bookmark**: private ↔ public, rec flag toggle.
- **CollectionItem**: pending → approved/rejected (moderated) or auto-approved.
- **Creatorship**: invited → approved.

## Notes

- Migrations additive; existing works get default pseud `username` creatorship on migrate.
- OTW reference models: `work.rb`, `pseud.rb`, `creatorship.rb`, `series.rb`, `collection.rb`, `collection_item.rb`, `bookmark.rb`, `kudo.rb`, `comment.rb`, `tag.rb`, `skin.rb`, `work_skin.rb`.
