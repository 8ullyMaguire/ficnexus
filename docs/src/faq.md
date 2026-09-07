# FAQ

## General

**Is FicHub free?**
Yes! 100% free, no ads, no hidden fees.

**Do I need an account?**
No! You can download stories without an account. You need an account for
bookmarks, ratings, comments, and posting in the forum.

**Is FicHub safe?**
Yes. It doesn't collect personal info, doesn't track you, and doesn't show ads. The code is open-source so anyone can verify.

**Does FicHub host stories?**
No! FicHub downloads stories from their original sites and converts them for you. The stories always stay on their original sites.

## Downloading

**What formats can I download?**
EPUB (best for e-readers), HTML (best for browsers), PDF (best for printing),
MOBI (older Kindles), AZW3 (newer Kindles), TXT, Markdown, KEPUB (Kobo), and
DOCX. See [How to Download Fics](./downloading.md) for the full table.

**Can I download multi-chapter stories?**
Yes! FicHub grabs all chapters automatically.

**Why did I get "Unknown Title"?**
Sometimes a site is busy or blocking requests. Try again in a minute. If it persists, the story might be restricted.

**Can I download restricted/private stories?**
No, FicHub can only download public stories.

## Account & Profile

**How do I create an account?**
Click "Register" in the top right, choose a username and password.

**How do I earn exp (experience)?**
Post helpful content, suggest great stories (other users upvote them), be
helpful in comments, and contribute to the community. Moderator/admin
actions award exp too (with an idempotent audit trail). Your level (0–100)
rises with exp and gates moderation powers.

**What do roles mean?**
- **Reader** — basic access
- **Curator** — can help moderate tags
- **Senior Curator** — experienced moderator
- **Admin** — runs the site

Roles still exist as labels, but site-wide **levels + exp** (see [Your
Profile & Leaderboard](./profile.md)) are the active gates.

## Forum

**What is the forum?**
A community discussion area (categories, topics, replies) with follows,
notifications, full-text search, read-state tracking, moderation points,
and metamoderation. It lives at `/forum` — see [Community
Forum](./forum.md) for the full guide.

**How do I find a fic I remember the title of?**
Type `Title by Author` (optionally `on Site`) into the Download tab, use
the command palette (`Ctrl+K`), or follow the suggestions on a 404 work
page. See [Find a fic by name](./searching.md#find-a-fic-by-name).

## Progression & Customization

**How do I earn XP?**
Read, download, bookmark, rate, review, comment, post in the forum, and
contribute to the community. Each action awards a specific XP amount. Your
level (0–100) rises with XP; your rank (Reader → Curator → Admin) unlocks
access to features.

**What is the Ability Tree?**
The Ability Tree (`/features`) shows every gateable feature in FicHub —
widgets, recipe builder, theme editor, custom views, admin tools — and
which ones you've unlocked. Activate or deactivate features individually.

**How do recipes work?**
Recipes (`/settings/recipes`) let you compose custom recommendation blends
by adjusting strategy weights (cooccur, tag_graph, embeddings, etc.),
setting filters (min words, exclude warnings), and adding boosts. Activate
a recipe to override your personal rec engine. Publish recipes to the
gallery for others to install.

**Can I change the site's look?**
Yes! Theme Design Tokens (`/settings/theme`) let you customize accent
colour, background, surface, text, font family, corner radius, density,
and reader settings. Five presets ship out of the box: Default Dark,
Default Light, High Contrast, Sepia, and Dyslexia-Friendly. Import/export
themes as JSON.

**What is a topic slug?**
Every topic gets a stable, human-readable URL at creation, e.g.
`/forum/board/what-s-your-opinion-on-the-site.42`. The slug is generated
from the title and never changes, so links keep working even if the topic
is later renamed. The old `/forum/{category}/{id}` URLs still work too.

**How do moderation points work?**
Trusted members earn moderation points; moderating from the queue spends
one. Moderator actions are audited (metamoderation) so the community can
vote on whether each was fair.

## Search

**Why is FicHub search better than AO3?**
FicHub supports full boolean queries (`AND`/`OR`/`NOT`), quoted phrases,
fielded search (`title:`, `author:`, `fandom:`), exclusions (`-angst`),
faceted navigation, primary tag filtering, comment/kudos counts,
no-warnings filter, and more. See [Finding Great Stories](./searching.md).

**Can I search across multiple sites?**
Yes! Use the Source filter to pick a specific site, or leave it on "All Sites" to search everywhere.

## Troubleshooting

**Download failed!**
- Check the URL is correct
- The site might be temporarily down
- The story might be restricted

**Can't log in?**
- Make sure your username and password are correct
- If you forgot your password, contact an admin

**Something looks broken?**
Report bugs on our [Forgejo](https://opencommit.eu/MagicZhang/fichub) or chat with us on [Discord](https://discord.gg/AmE93m23dW).

---

*Back to [Introduction](./intro.md)*
