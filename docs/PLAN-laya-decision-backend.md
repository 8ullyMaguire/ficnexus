# PLAN (optional) — Laya decision model as a classification backend

Status: **optional, not adopted, not implemented.** This document records an opportunity and
enough detail to act on it later. No dependency was added, no config was changed, and no model was
downloaded. If Laya is never installed, this file is still accurate — it is a plan, not a claim.

Origin: [[Laya Decision Model — Optional Use Cases and Per-Repo Ledger]] (2026-09-29).
Predecessor reasoning: [[Kev-4B — Use Cases Across the Local Projects]].

---

## 0. The finding that changes the priority

A full enum sweep of all 74 repos under `~/code` (668 enums, script
`~/.hermes/profiles/sysadmin/cache/scratch/enum-sweep.sh`) found a fit that the earlier
brainstorm missed **inside ficnexus itself**.

`src/services/comment_triage.rs` already does this:

```rust
// line 167
match ollama.generate(&triage_prompt(text), "llama3.1:8b").await {
```

It loads an **8B text-generation model** to answer a question that is a `choice` over exactly five
labels — `fine`, `constructive`, `non-constructive`, `toxic`, `spam` (`TriageCategory`, line 20) —
and then parses a string of the form `toxic | reason | 0.9` back out of free text
(`parse_triage_response`, line 98), defensively degrading to `Fine`/`0.0` on anything malformed.

**This is a decision model simulated with a text model.** It pays for 8B of weights, generative
latency, and a brittle `splitn('|')` parse, to produce a number that is already shaped like a
typed answer with a confidence field. Laya is 678 MB, returns that shape natively, and emits
**zero output tokens**.

That is the strongest case in this entire evaluation, and it is not a new integration — it is a
replacement of the model behind a function that already exists.

---

## 1. The seam, and a correction about it

**Correction to an earlier draft of this plan.** I first described a `decide()` sibling on
`OllamaClient`. Reading `src/services/ollama.rs` properly shows that would be wrong:

* It hardcodes Ollama's native paths — `/api/embeddings` (line 59), `/api/generate` (line 91),
  `/api/generate` for JSON (line 152) — by string interpolation onto `self.base_url`.
* Its own module doc (lines 1-8) scopes it to *"Ollama runs on localhost:11434 … thin wrapper
  avoids adding heavy Rust ML dependencies"*.

The Laya Decision API is **`POST /v1/systemone` on a different server (Unsloth, port 8888)** with a
different auth scheme. Retargeting `OllamaClient` at it would either break `embed()` (used by the
Roadmap Consensus Engine) and `generate_json()`, or make a client named for one vendor speak to
another. So this is a **new sibling client**, not a new method:

```
// src/services/decision.rs  — new file
pub struct DecisionClient { base_url: String, api_key: Option<String>, http: reqwest::Client }
impl DecisionClient {
    pub async fn decide(&self, state: &str, questions: serde_json::Value)
        -> Result<DecisionAnswer, OllamaError>;
}
```

`DecisionAnswer` is a typed struct (`answer`, `probabilities`, `confidence`), which is what lets
step 5 delete the prose parser. Reuse `OllamaError` rather than inventing a second error type —
both are "a model was unreachable", and the callers already treat that as non-fatal.

`OllamaClient` stays exactly as it is. The two clients talk to two different local servers and
that is a real boundary, not an abstraction to be tidied away.

### The second seam that already exists

`src/heal/agent.rs` (254 lines) already abstracts "ask a model" with two backends and a
capability probe:

| Symbol | Purpose |
|---|---|
| `remote_configured(cfg)` | `agent_enabled && base_url non-empty && api key present` |
| `diagnose_remote(...)` | OpenAI-compatible `{base}/chat/completions` |
| `diagnose_local(...)` | Ollama `/api/chat` |
| `extract_metadata_remote/local(...)` | same seam, metadata-extraction prompt |

Routed at `src/routes/heal.rs:92-128` (`let agent_available = cfg.agent_enabled && agent::remote_configured(cfg)`).

**So a third backend is a recognised shape in this codebase, not an invention.** But note the
routing there is "remote or local", decided by whether a key is configured — a decision model is
neither, so it needs its own path rather than a third arm on that `if`. And **`diagnose_*` returns
`String` and is genuinely generative; do not convert those.** The decision-model shape fits
`comment_triage` only.

---

## 2. The migration, if it is ever justified

Ordered so that each step is independently valuable and independently revertible.

| # | Step | Files | Reverts by |
|---|---|---|---|
| 0 | **Verify Unsloth runs headless here.** GUI app on a headless box is an open question, and every step below is blocked on it | — | — |
| 1 | New `DecisionClient` in its own file: `decide()` → typed `DecisionAnswer { answer, probabilities, confidence }`. Reuse `OllamaError` | **new** `src/services/decision.rs` | delete the file |
| 2 | Feature-gate it: `DECISION_MODEL_BASE_URL` / `DECISION_MODEL` env, empty = off | `Config`, `decision.rs` | unset the env |
| 3 | Port `classify_comment` to `decide()`, **behind the flag**, old path preserved | `src/services/comment_triage.rs` | unset the env |
| 4 | Dual-run both, log disagreements + latency for N days | new | unset the env |
| 5 | Only if disagreements are noise: delete `triage_prompt` and `parse_triage_response` | `src/services/comment_triage.rs` | git revert |

**Step 5 is the actual prize, and it is worth stating plainly:** it removes a
prompt-formatting function, a hand-written response parser, and its defensive fallbacks
(~200 of the file's 279 lines are triage plumbing) — because a typed API cannot return
`"toxic | rea"`. That is a correctness improvement, not only a cost one, because today's parser
silently maps a malformed reply to `Fine`, which is the *least* actionable label.

### Invariant that must survive all of it

`CommentTriage::fallback()` returns `Fine` with `confidence: 0.0`, and the existing comment is
already documented as **best-effort — "triage never fails the comment post"**. Any decision
backend must keep that: an unreachable model yields `fallback()`, never an error that blocks a
post. Note this means `Fine`/`0.0` must stay distinguishable from a model's genuine `Fine` —
which the existing `confidence` field already provides, and which a decision model's
`probabilities` map improves on.

---

## 3. Thresholding — the thing most likely to ship a bug

Laya, Jev, and Kev each compute `confidence` differently; the Unsloth docs warn about this twice
and say explicitly to threshold on `probabilities`, not `confidence`.

Today's code takes the model's self-reported `0-1` number at face value
(`parse_triage_response` → `confidence`). A tuned Jev threshold transplanted onto Laya would be
meaningless, and **a wrong threshold here silently changes which comments reach the moderation
queue**.

Two requirements, both non-negotiable:
* threshold on `probabilities` for the chosen label, never on `confidence`;
* pin the threshold in a unit test with a fixture, so a model swap cannot move it unnoticed.

Note the class imbalance to expect: most comments are `Fine`, so an uncalibrated 0.5 cut will look
fine on aggregate accuracy while misfiling real toxicity. Measure per-class, not overall.

---

## 4. Sizing — why this finally fits the hardware

The Kev note closed the door with *"Don't try to serve it on this box"* (4B LoRA, hybrid DeltaNet
attention, no CUDA). Measured on this hardware this session:

| | Model | Weights | Resident @ 64K ctx | Throughput |
|---|---|---|---|---|
| gaming-pc | `qwen2.5-coder:3b` | 1.9 GB | — | 10.58 tok/s warm |
| gaming-pc | **via sshfs pool** | 1.9 GB | — | **0.54 tok/s** (20x slower) |
| thinkcentre | `qwen2.5-coder:3b` | 1.9 GB | — | 0.68 tok/s (memory thrashing) |
| **Laya** | **multilingual** | **678 MB** | **~1.5 GB est.** | **sub-second after 10–20 s load** |

Laya is a decision head, not a generative model: one forward pass, no KV-cache growth over a
conversation, and the vendor's own target is 4 GB RAM on CPU. The comparison that matters is not
"678 MB vs 1.9 GB" but **"678 MB and zero output tokens" vs "8B weights, a prompt, and ~20 output
tokens parsed back out of prose"** for the same five-way decision.

Unsloth Desktop is a GUI app; whether it runs headless on this box is **unverified** and is a
packaging question to settle before any of the above.

---

## 5. What this does *not* cover

Deliberately excluded, with reasons:

* **`QualityVerdict`** (`src/scrape/quality.rs:16` — `Accept` / `Reject(&str)` /
  `Suspicious(&str)`) is today a **string/heuristic check** for placeholders, empty titles, and
  lorem ipsum (lines 65-71), not a judgement about content. A model would be slower and less
  reliable than `title == "Untitled"`. It becomes interesting only if the heuristic is replaced
  with a semantic "is this junk?" judgement, which is a design change, not an optimisation.
* **`RavenClaws` `InjectionVerdict`** (`src/policy.rs:557` — `Clean` / `Suspicious(String)`) is a
  **security gate on LLM output**. A false negative is a prompt injection reaching the agent. The
  existing two-layer heuristic defence should not be handed to a 678 MB model, and certainly not
  before a measured false-negative rate exists. Listed so it is not rediscovered and adopted by
  someone else later.
* **`RavenClaws` `ToolCategory`** (8 variants) would only be a routing win if routing were a
  bottleneck; the existing deterministic router should win any head-to-head.
* **`Commons` `ConsentTier`** — advisory only, never the write path. A model guessing about a
  real person's consent is worse than no model.

---

## 6. Repo note (corrected)

* **Correction to an earlier note in this document:** I first recorded that this repo's
  `AGENTS.md` "contains an invisible U+200D and the read guard blocked it as a potential prompt
  injection." **That was wrong**, and the evidence does not support it. On inspection the file
  contains exactly two U+200D characters, at offsets 3 and 53597, and both sit inside the emoji
  ZWJ sequence `🐦‍⬛` in `# 🐦‍⬛ RavenClaws — AI Agent Instructions` and its footer. A
  zero-width joiner between two emoji code points is normal text, not a hidden instruction. The
  read guard's heuristic fired on the character class, not on any injected content, and the
  mechanical check is what settles it. No security concern; no action needed.
* Observed during the sweep, unrelated to this plan: `rust/ficnexus` already runs an **Ollama
  embedding** call in `src/services/auto_tagger.rs` (`AutoTaggerError::Embed(OllamaError)`,
  line 241-245). So the deployment already has a local-model dependency; adding a second one is
  operationally cheap.

---

## Related

* [[Laya Decision Model — Optional Use Cases and Per-Repo Ledger]]
* [[Kev-4B — Use Cases Across the Local Projects]]
* [[Local Ollama Coding Model — Multi-Machine Deployment]] — the measured speeds in §4
* `30-Resources/laya-decision-model-repo-ledger.csv` — per-repo audit trail
## Measured baseline — 2026-09-29

**Unsloth is already running.** No install is required. On thinkcentre,
`gravity-decision-provider.service` serves Unsloth Studio on `127.0.0.1:8888`; the API key is at
`~/.unsloth/systemone.key` (mode 600 — read it in-process, never interpolate it into a command
line or a log). Verified live: it returns the documented envelope exactly.

Benchmarked on 12 hand-labelled comments against the real `triage_prompt` vocabulary:

| | Top-1 | Latency |
|---|---|---|
| `laya-multilingual` | 4/12 (33%) | ~100 ms |
| current baseline `llama3.1:8b` | 0/12 | errors out — **not installed** |

Two findings that change the plan:

1. **33% is a prompt defect, not a model limit.** Laya is 4/5 on unambiguous cases and 0/7 on
   judgement calls, and its probabilities separate cleanly (mean p 0.805 correct / 0.176 wrong), so a
   0.5 threshold acts at 100% precision and abstains otherwise. A `noul` control proved why: asked
   whether a clearly-actionable comment gives specific feedback, it answered **0.003** while `choice`
   in the same request said `toxic` at 0.46. It is reading the *category labels*, not the comment.
   **So do not port `triage_prompt` as-is.** Decompose into independent booleans —
   `actionable?` / `hostile?` / `promotional?` — and derive `TriageCategory` from them. That
   decomposition is the first thing to build and measure.
2. **The existing baseline is broken.** `llama3.1:8b` is not installed on any of the three hosts.
   `classify_comment` logs at `debug` and returns `fallback()`, so every comment has been recorded as
   `Fine` with no visible symptom. Fix this regardless of the Laya decision: either install the model,
   or raise that log to `warn` and make the fallback visible in the UI.

**On `confidence`:** the docs say twice to threshold on `probabilities`, and this benchmark shows
why — a `choice` answer's `confidence` was 0.2187 while its top probability was 0.4565. Threshold the
probability of the *specific* class, never the aggregate confidence.

