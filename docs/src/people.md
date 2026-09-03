# People Search — Finding Authors & Users

FicHub's **People Search** (`/people`) is an AO3-style browse + search page for
finding users — authors, bookmarkers, reviewers, and other community members.

## Finding People

- **Search** — type a name (or partial name) into the search box and press Enter.
  The page shows matching users as PeopleBlurb-style cards with avatar, name,
  and bio snippet.
- **Browse** — click any letter (A–Z) to jump to users whose names start with
  that letter. Users with no display name appear under `#`.
- **Filter by fandom** — the search form includes a "Fandom" field (type-ahead).
  Select a fandom to narrow the people list to those who write for or bookmark
  works in that fandom.

## Results

Each result shows:
- **Avatar** (if the user has set one)
- **Name** (display name, linked to the user's profile)
- **Bio snippet** (first 200 characters of the user's bio)

Click any name to go to that user's profile (`/profile/[id]`, which shows their
shelves, reading history, ratings, and reviews — visibility depends on their
privacy settings).

## How it works

The page calls `GET /api/forum/users/search?q={query}&fandom={slug}&page={n}`
and renders the returned users in archive-style `dl` markup. The API is a
thin wrapper over the `users` table (filtered by name LIKE and optional
fandom join).

---

*Next: [Cool Features](./features.md)*
