# Getting Recommendations

Not sure what to read next? FicHub has two ways to help.

## How Recommendations Work

FicHub looks at what other people who read the same stories as you also
enjoyed. It's like asking a friend "what should I read next?" — but based on
thousands of readers.

## 1. From the Recommendations Tab

1. Click the **★ Recommendations** tab.
2. Paste a story URL you've enjoyed.
3. Click **Get Recommendations**.
4. See a list of similar stories!

## 2. From a Story Page

When you view a story on FicHub, you'll see recommendations at the bottom of
the page.

## Personal recommendations (home page)

Sign in and bookmark (or download) a few fics. The home page then shows a
**"Recommended for you"** row with a "Because you bookmarked: …" label — the
more you bookmark, the better it gets.

## Suggesting Stories

Have a story you think others would love? Suggest it on the story page:

1. Open any story page and scroll to the **Similar fics** section.
2. Use the autocomplete to pick an in-archive story you think is similar, or
   paste the URL of a story from another site (FicHub will scrape and add it).
3. Add a short comment about why it fits (optional).
4. Click **Suggest**.

Other users can then upvote or downvote your suggestion right on that story page.

## Voting on Suggestions

- **Upvote** — "Yes, this is a great recommendation!"
- **Downvote** — "No, this doesn't fit well"

The best suggestions rise to the top of the per-fic list.

## Similar Tags, Authors, Collections & People

Recommendations aren't only for fics — `GET /api/v0/recommendations/entities`
suggests *entities* too:

| `kind=` | Finds | Based on |
|---|---|---|
| `tag` / `fandom` | Similar tags & fandoms | Shared works (Jaccard over fic tags) |
| `author` | Similar authors | The author co-occurrence graph (+ shared-tag overlap) |
| `collection` | Similar reading lists | Shared works between public lists |
| `user` | Readers like you | Shared positively-signalled works |

Every result comes with a human-readable reason ("readers of Author X also
read this author"). No extra data collection — these reuse the graphs the
recommendation engine already materializes.

---

*Next: [Your Profile & Leaderboard](./profile.md)*

