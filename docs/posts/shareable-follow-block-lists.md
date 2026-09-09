# Shareable Follow/Block Lists — Analysis

> **Feature:** Let users create named, shareable lists of who they follow (curated author lists) or who they block (moderation lists), which others can subscribe to or import.

---

## How It Would Work

- User creates a list: "My Favorite Dramione Authors" or "Spam Accounts to Avoid"
- List has visibility: **public** (discoverable), **unlisted** (link-only), or **private**
- Other users can **subscribe** (auto-updates when list changes) or **import** (one-time copy)
- Lists appear on user profiles under a "Lists" tab
- API: `GET /api/users/{id}/lists`, `POST /api/lists`, `POST /api/lists/{id}/subscribe`

---

## Pros

| Benefit | Description |
|---------|-------------|
| **Content discovery** | New users can instantly follow 50 great authors via a trusted curator's list instead of searching manually |
| **Community curation** | Power users become "tastemakers" — their lists gain followers, creating a meta-layer of recommendation |
| **Reduced duplication** | No need for 100 users to manually find and follow the same 20 authors; subscribe once |
| **Onboarding accelerator** | Fresh accounts can subscribe to "Starter Pack" lists to populate their feed immediately |
| **Collaborative moderation** | Community-maintained block lists for known spammers/trolls reduce moderator workload |
| **Network effects** | Popular curators attract followers, which incentivizes quality curation |
| **Portability** | Export/import lists as JSON — useful for account migration or backup |
| **Social proof** | "42 users subscribe to this list" signals trustworthiness |

---

## Cons

| Risk | Description |
|------|-------------|
| **Privacy leakage** | Users may not want others to know *exactly* who they follow (reveals interests, fandoms, even identity) |
| **Harassment weaponization** | Public block lists can be used for targeted harassment — "block list of everyone who disagrees with X" |
| **Stalking vector** | Aggregated follow lists make it easy to map someone's interests, associations, or real identity |
| **Spam/gaming** | Fake "recommendation" lists promoting low-quality works or sockpuppet accounts |
| **Staleness** | Lists go out of date — authors abandon fics, spammers create new accounts |
| **Moderation burden** | Who reviews public lists? A list titled "Good Authors" that contains only one person's alts is abuse |
| **Mob dynamics** | Coordinated "block campaigns" where a group adds innocent users to their block lists en masse |
| **API leakage** | Even "private" lists might leak through API endpoints, search indexing, or data exports |
| **Scope creep** | Feature starts as follow lists, expands to block lists, then to "list of lists" — complexity grows fast |

---

## Verdict

### Follow Lists: **YES — with privacy controls**

- Default to **unlisted** (not discoverable, but shareable via link)
- Public opt-in only (user must explicitly set list to public)
- Show subscriber count, not subscriber identities
- No auto-follow — subscribing to a list shows recommendations, user picks which to follow
- Rate-limit list creation (max 10 lists per user) to prevent spam

### Block Lists: **NO — or private-only**

- Public block lists are a harassment vector with no redeeming social value
- If implemented at all: **private import only** (user uploads a list of user IDs to block, no one sees the list)
- Better alternative: **crowd-weighted auto-moderation** — if 100+ users block the same account, flag for moderator review (no public list needed)

### Recommendation Lists: **YES — separate feature**

- Instead of sharing raw follow lists, let users create **"Recommendation Lists"** — curated picks with descriptions
- These are explicitly public-facing and moderated
- Decoupled from actual follow graph — no privacy leak
- Example: "Best Completed Fics Under 20k" with 10 works and a paragraph each

---

## Implementation Priority: **Medium**

- Follow lists: low effort, high value, but privacy design must be right
- Block lists: skip entirely (use auto-moderation signals instead)
- Recommendation lists: medium effort, high value, no privacy concerns
