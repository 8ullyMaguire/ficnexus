# Tag Search — Finding Tags

FicHub's **Tag Search** (`/search/tags`) mirrors AO3's tag search form. You can
find tags by name, fandom, type, wrangling status, and sort order — useful for
discovering canonical tags to use in your own searches.

---

## How to access

Click **Search > Tags** in the archive header. The page opens an AO3-style form
with `fieldset.dl` formatting in archive mode.

---

## Form fields

| Field | Widget | API parameter | Options |
|---|---|---|---|
| Tag name | text input | `q` | Free text; matches partial names |
| Fandom | text input + autocomplete | `fandom` | Start typing to find canonical fandoms |
| Type | radio group | `tag_type` | Fandom, Character, Relationship, Freeform, Warning, Category, Any |
| Wrangling status | select dropdown | `canonical` | Canonical, Non-canonical, Synonymous, Canonical or Synonymous, Unwrangleable, Any |
| Sort by | select | `sort` | Name (alphabetical), Usage (most-tagged first), Created (newest first) |
| Sort direction | select | `dir` | Descending, Ascending |

---

## Tag type IDs

FicHub uses the same tag type IDs as AO3:

| ID | Type |
|---|---|
| 1 | Fandom |
| 2 | Character (warning in AO3, but FicHub uses for character) |
| 3 | Relationship |
| 4 | Freeform / Additional |
| 5 | Warning (category in AO3) |
| 6 | Category |

---

## Results

The results table shows:

- Tag name (clickable — goes to the work search filtered by that tag)
- Type (Fandom, Character, etc.)
- Wrangling status (Canonical / Non-canonical / Synonymous / etc.)
- Canonicity (green check for canonical tags)
- Item count (how many works use this tag)
- Pagination at the bottom (20 tags per page)

---

## API

The backend endpoint is `GET /api/tags/search` with parameters:

- `name` — tag name search
- `fandom` — canonical fandom to scope to
- `tag_type` — numeric type ID (1–6) or omit for all
- `wrangling_status` — canonical / non-canonical / synonymous / etc.
- `sort_by` — name / usage / created
- `sort_direction` — desc / asc

The frontend client (`src/lib/api/tags.ts`) provides two functions:

- `searchTags(params)` — basic search
- `searchTagsAdvanced(params)` — full parameter support including `q`, `canonical`, `fandom`, `page`, `limit`, `sort`

---

## Example

Find all canonical character tags related to "Harry Potter":

1. Type `Harry` in the Tag name field
2. Select **Character** in the Type radio group
3. Select **Canonical** in the Wrangling status dropdown
4. Select **Usage** for Sort by
5. Click Search

Results show the most-used canonical Harry Potter character tags, each linking
to a work search filtered by that tag.

---

*Next: [Back to Finding Great Stories](./searching.md)*
