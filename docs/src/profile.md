# Your Profile & Leaderboard

Track your activity and climb the ranks.

## Your Profile

Click your username to see your profile. It shows:

- **Username** — your display name
- **Level** — your site-wide level (0–100), shown as a ⭐ chip next to your
  username in the top bar
- **Experience (exp)** — points that drive your level; earn exp by posting
  and contributing (moderator and admin actions award exp too)
- **Role** — your community role (Reader, Curator, Senior Curator, Admin)
- **Badges** — achievements you've earned
- **Member Since** — when you joined

## Leveling & Experience

Levels and experience (exp) replaced the old reputation system as the
site-wide progression. Key facts:

- **Levels run 0–100.** Your level is the gate for community powers:
  moderating posts, fast-hiding content, forum moderation/metamoderation,
  and admin actions all require a minimum level.
- **Exp is earned by contributing**: posting helpful content, proposing
  merges that get accepted, voting on proposals, and getting your comments
  liked all award exp. Moderator/admin actions record an idempotent
  `exp_events` audit trail so the same action never double-awards.
- **Role still exists** as a legacy label (Reader / Curator / Senior
  Curator / Admin), but level-based gates are the active mechanism.

## Roles

| Role | What it means |
|------|--------------|
| **Reader** | Everyone starts here! Browse, download, and enjoy. |
| **Curator** | Trusted members who help moderate tags and content. |
| **Senior Curator** | Experienced moderators with extra powers. |
| **Admin** | The team running FicHub. |

## The Leaderboard

Click **Browse → Leaderboard** in the menu to see the top contributors.

Everyone starts at 0 exp. As you contribute to the community, your score
goes up. Can you reach the top?

---

*Next: [Cool Features](./features.md)*

