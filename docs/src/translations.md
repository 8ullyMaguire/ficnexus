# Translations & the Curator Queue

FicNexus translates everything you see — fic bodies, requests, forum
posts, comments, reviews, even the interface itself — into your language.
Translation runs on three tiers, and the community is part of the loop.

## How it works

When you browse in a language other than the text's original, FicNexus
shows, in order of preference:

1. **Community translation** — reviewed and approved by curators. No
   marker needed; it is simply the best text we have.
2. **Machine translation** — produced once by the local model and
   *cached forever*. Look for the tiny **machine translated** tag next to
   a block. The model never runs twice for the same text and language, so
   you pay no extra wait.
3. **The original** — with a **Translate this page** button to kick off
   tier 2.

Click any translated block to see the original side by side. Your choice
is remembered for the session.

## Reading language

Your *interface* language (menus, labels) comes from your account
preference. The *reading* language — what gets translated to what — is set
independently in Settings, defaulting to your browser language. English
readers are unaffected: the archive's source language is English, so
nothing is fetched for them.

## Improve a translation (and earn reputation)

Machine translation is a first draft, not the last word:

- Click **improve** on any machine-translated block. The editor opens
  with the machine text *already filled in* — you edit, you don't
  retranslate.
- Submit, and your version enters the curator queue. Two curators
  (configurable) voting approve makes it the new approved translation —
  shown to everyone reading that language, from then on, without any
  model call.
- Approved contributions earn **reputation points** (see the
  [leaderboard](/leaderboard)): 15 for an approval, 25 when your version
  replaces one that was already approved (curators *can* improve the
  improved — each change restarts the consensus vote, and the history of
  every version is kept).
- Seen something bad from the machine? **Report it** — that dismisses the
  machine draft immediately, asks for a fresh translation, and earns you a
  point for the first report on a block.

## The curator queue — one page for every consensus decision

`Curator → Queue` in the navigation (curators and above) lists every
pending proposal in the system: translations, machine drafts awaiting
review, content fixes, metadata changes, non-author forum edits, work
deletion requests, doc edits. Each row shows who proposed, the votes so
far, and the quorum needed:

- **Approve / Dismiss** votes resolve when quorum is reached.
- **Fast-decide** ends a pending proposal immediately, with an audit
  note — for the obvious spam and the obvious gems.
- Every decision is written to the public [modlog](/modlog).

Competing proposals for the same block (three users, three better
versions) are grouped, and the first to build consensus wins; the others
are marked superseded — no penalty, just a lost round.

## Instance settings (self-hosted)

Admins tune the whole pipeline per instance: the translation master
switch, the model, chunk sizes, hourly budgets (global and per user),
whether machine output goes through review or auto-approves
(`TRANSLATE_AUTO_APPROVE_MACHINE=true` — "LLM translates, nobody votes"
mode for tiny instances), per-kind quorums, and the reputation amounts.
The full list lives in the repo's `.env.example` under `TRANSLATE_*` and
`PROPOSAL_*`.

## Search & privacy note

Search always runs over the original text — translated versions are a
reading layer, not a re-index. Machine translations are generated locally
by the instance's Ollama model; nothing is sent to third-party services.
