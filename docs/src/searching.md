# Finding Great Stories

FicHub's search is way more powerful than AO3's. You can combine boolean operators,
quoted phrases, field-specific searches, and exclusions — then refine with facets and
filters. Here's everything you can do!

## Basic Search

Click the **Search** tab in the menu. Type what you're looking for and press Enter.

You can search for:
- Story titles
- Authors
- Descriptions
- Tags

## Boolean Query Syntax

Type a query in the search box to combine terms with operators. Everything is
**case-insensitive**, and multiple words with no operator are combined with **AND**
automatically.

### AND / OR / NOT

| Operator | Example | What it does |
|----------|---------|-------------|
| `AND` | `harry AND slytherin` | Both terms must appear |
| `OR` | `fluff OR humor` | At least one term must appear |
| `NOT` | `angst NOT fluff` | First term appears, second must not |
| `-` (minus) | `angst -fluff` | Excludes the term after the minus |
| Implicit AND | `harry potter` | Same as `harry AND potter` |

You can mix them freely: `(fluff OR humor) AND angst -sad`

### Quoted Phrases

Put quotes around a phrase to match it as a whole:

```
"slow burn"
"enemies to lovers" AND angst
```

### Parentheses

Group terms to control precedence, just like math:

```
(fluff OR humor) AND angst
"slow burn" OR (hurt AND comfort)
```

### Fielded Search

Narrow a term to a specific field:

| Field | Example | What it does |
|-------|---------|-------------|
| `title:` | `title:harry` | Matches the story title |
| `author:` | `author:rowling` | Matches the author name |
| `fandom:` | `fandom:harry potter` | Matches a fandom tag |
| `character:` | `character:hermione` | Matches a character tag |
| `relationship:` | `relationship:draco/hermione` | Matches a relationship tag |

Fielded searches work alongside boolean operators:

```
title:harry AND author:rowling
fandom:"harry potter" -character:draco
```

### Search Examples

**"Find me complete Harry Potter fics over 100k words with no warnings"**
→ Type `fandom:"harry potter"` in the search box, click "Complete Only" and "100k+"
chips, click "No Warnings"

**"Find Hurt/Comfort OR Angst fics"**
→ Type `"hurt/comfort" OR angst` in the search box

**"Find popular fics with lots of comments"**
→ Click "Popular (50+ kudos)" and "Has Comments (10+)" chips

## Quick Filter Chips

Below the search bar, you'll see quick filter buttons. Click one to instantly filter results:

| Chip | What it does |
|------|-------------|
| **No Warnings** | Hides stories with content warnings |
| **Complete Only** | Only finished stories |
| **100k+ Words** | Epic-length stories |
| **Short (<5k)** | Quick reads |
| **Multi-chapter (20+)** | Long ongoing stories |
| **Has Comments (10+)** | Stories people are talking about |
| **Popular (50+ kudos)** | Fan favorites |

You can click multiple chips to combine them!

## Facets & Filter Chips

After you search, a **facet sidebar** shows the most common Fandoms, Characters,
Relationships, Warnings, Categories, Freeforms, and Statuses among your results.
Click a facet value to narrow your search to it.

Your active filters appear as **removable chips** above the results — click the **×**
on any chip to remove just that filter, or **✕ Clear all** to reset everything.

## Suggested Filters

After a search, a **Suggested** row shows popular tags (and, when you're signed in,
tags matching your bookmarks — marked **for you**). Click a suggestion to add it
as a filter. Suggestions you already filtered by are hidden, and the row disappears
if the suggestion service is unavailable.

## Find a fic by name

Already know the story and just want its archive page? Three ways in:

- **Download tab, dual-mode box** — type `Title by Author` (optionally `on
  Site`, e.g. `on AO3`) instead of a URL. FicHub parses the title/author/site
  and shows matching works already in the archive. If nothing matches and you
  pasted a URL instead, the normal download flow takes over.
- **Command palette (`Ctrl+K`)** — type the title; a "🔍 Find fic" row jumps
  to the archive match.
- **Work pages that 404** — a dead work link offers "did you mean …"
  suggestions from fuzzy title matching, plus a one-click "Request this fic"
  button that prefills a Fic Request.

New Fic Requests also auto-suggest: as you type the title/body on
`/requests/new`, a dismissible box lists up to 5 works already in the
archive so duplicates get caught before posting.

## Advanced Filters

Click **▸ Show Advanced Filters** to see all the options:

### Tags
- **Must Have Tags (AND)** — story must have ALL these tags (format: `1:Fandom, 2:Character`)
- **Any Of These Tags (OR)** — story must have at least ONE of these tags
- **Exclude Tags** — hide stories with these tags
- **Main-Character Attribute** — `main_char_attr=Character|Attribute` finds
  stories where the **main character** (the first-listed character) has a
  specific attribute tag. Example: `Harry Potter|Dark Harry Potter` finds fics
  **starring** Harry with the Dark tag — not fics where Harry is a side
  character and the Dark tag belongs to someone else.
- **Primary Tag** — find stories where a tag is the MAIN tag
- **Tag IDs** — search by numeric tag IDs

Tag type IDs: `1`=Fandom, `2`=Character, `3`=Relationship, `4`=Freeform, `5`=Warning,
`6`=Category

### Numbers
- **Min/Max Words** — filter by story length
- **Min/Max Chapters** — filter by chapter count
- **Min Comments** — only stories with lots of discussion
- **Min Kudos** — only highly-rated stories

### Other
- **Source Site** — pick a specific website
- **Completion Status** — complete, in progress, or all
- **Sort By** — relevance, date, word count, or kudos
- **Date Range** — stories updated in a specific time period
- **Relationship Characters** — type character names (comma-separated) to find
  relationships that include them, e.g. `Draco Malfoy, Hermione Granger`

### Try it now

- [Run a boolean search: `(fluff OR humor) AND angst`](/search?q=%28fluff%20OR%20humor%29%20AND%20angst)
- [Find dark-Harry fics: main-character attribute](/search?q=harry%20potter&main_char_attr=Harry%20Potter%7CDark%20Harry%20Potter)
- [Ask the Archive in plain English](/ask)

---

*Next: [Saving Your Favorites](./bookmarks.md)*
