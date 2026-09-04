# Part 6 — The API: Meta, Search, and Reader

> **Part 6 of 13** — Part 5 took us from URL to file: the export pipeline,
> the pure-Rust EPUB builder, and the hash-addressed cache. This part zooms
> out from files back to JSON. We cover the three public API surfaces that
> make FicHub feel alive: the metadata endpoint in `src/routes/meta.rs`
> (Chapter 23), the boolean search engine in `src/search/parser.rs` with its
> AND/OR/NOT, phrases, and fielded terms (Chapter 24), the "Dark Harry"
> main-character-attribute filter that shows how a scraping-time scoring
> decision becomes a query-time superpower (Chapter 25), the web reader
> that renders our cached HTML bundle in the browser (Chapter 26), and
> finally the SvelteKit shell that ties it all together — `+layout.svelte`,
> `+page.svelte`, and the API client in `client.ts` (Chapter 27). By the
> end of Part 6, you'll be able to trace a single search keystroke from
> the nav bar all the way down to a PostgreSQL `to_tsquery`, and back up
> to a rendered chapter.

---

## Chapter 23 — Metadata Lookup: routes/meta.rs

Every fic in FicHub starts the same way: someone pastes a URL. In Part 5
you saw what happens when that URL wants *files* — the export pipeline,
the semaphores, the seven formats. But there's a lighter question you can
ask the server: *"just tell me about this fic, don't build me anything."*
That's the metadata endpoint, and it's the perfect place to start Part 6
because it's the export pipeline's shy cousin: same family, same JSON
shape, but it never generates a single file.

The route lives in `src/routes/meta.rs` (about 200 lines — small enough
to hold in your head all at once) and is registered in `src/server.rs`
right next to its big sibling:

```rust
.route("/api/meta", get(routes::meta::meta_handler))
```

One query parameter, one job. Let's look at the contract first.

```rust
/// Query parameters for meta requests
#[derive(Debug, Deserialize)]
pub struct MetaQuery {
    pub q: Option<String>,
}

/// Metadata-only handler: GET /api/meta?q=<url_or_hash>
/// Returns same response as epub but without generating/downloading files
pub async fn meta_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MetaQuery>,
) -> Result<Json<Value>, AppError> {
    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query", "q": ""})));
    }
```

Notice what's *not* here: no `ExportQuery` with `automated` and `format`
flags, no proof-of-work check, no rate limiting. The meta endpoint is
deliberately unguarded — it's a cheap, read-only lookup. A reader wanting
"show me what this URL is about" before committing to a download
shouldn't have to pass the same gauntlet as someone grabbing an EPUB.
That asymmetry is a design decision worth copying: gate the expensive,
abusable operations; keep the cheap read paths friendly.

The handler then makes a binary choice that defines the whole file:
is `q` a URL, or is it a hash?

```rust
    // If q looks like a hash (not a URL), look up from fic_info directly
    if !query.starts_with("http") {
        return handle_hash_lookup_meta(&state, query).await;
    }

    // Find scraper (prefer native scraper, fall back to FanFicFare)
    let scraper = state.scraper_registry.find_specific_or_fff(query)
        .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;

    // Lookup metadata only (no chapters fetch)
    let mut meta = scraper.lookup(&state.http_client, query).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;
```

### 23.1 The two lookup paths: live scrape vs. database

That `starts_with("http")` check is the entire routing logic, and it's
worth pausing on because it encodes two very different worlds.

If `q` is a URL, we're in **live mode**: find a scraper that can handle
this site (preferring the native scraper, falling back to FanFicFare —
exactly the registry dance we built in Part 4), then call `lookup()`.
Remember from Part 4 that `lookup` is a *pure read*: it fetches the
story page and extracts `FicMetadata` without pulling a single chapter
body. That's why this endpoint can be fast even for fics that have never
been scraped before. The comment says it explicitly: *"Lookup metadata
only (no chapters fetch)"*.

If `q` is *not* a URL, we assume it's a url_id — the 12-hex-character
sha256 fingerprint we met in Part 4 — and skip the network entirely:

```rust
/// Handle hash lookup for meta endpoint
async fn handle_hash_lookup_meta(
    state: &Arc<AppState>,
    hash: &str,
) -> Result<Json<Value>, AppError> {
    let fic = queries::get_fic_info(&state.db, hash).await?
        .ok_or_else(|| AppError::NotFound(format!("fic not found: {}", hash)))?;
```

This is the **database mode**: one indexed `SELECT` on `fic_info`, and if
the row is missing we answer with a 404-style `AppError::NotFound`. No
scraper, no HTTP call to the source site, no `lookup`. This is the path
the frontend hits constantly — every link that carries a `url_id` around
the app (and they all do, remember the hash-is-the-contract lesson from
Part 5) can resolve to metadata instantly.

🧪 **Try It Yourself — hit both paths back to back.** Start FicHub with a
local database, export any fic once so it lands in `fic_info`, then run:

```bash
curl -s "http://localhost:8080/api/meta?q=<paste-a-fic-url>" | jq '.info'
curl -s "http://localhost:8080/api/meta?q=<the-returned-url_id>" | jq '.info'
```

The first request scrapes live (watch the server logs for the outgoing
fetch); the second is a pure database read. Both return the same
`"Title by Author - N words, M chapters"` summary string.

### 23.2 Author-link enrichment: the best-effort upgrade

Between the lookup and the response, the handler runs a small but
fascinating block: author-link enrichment. If the scraper came back
without an author profile URL (the comment notes FanFicFare often omits
it for newer sites), FicHub tries to recover it — first with its own
generic extraction, then with an LLM as a last resort:

```rust
    // Author-link enrichment: if the scraper didn't return an author
    // profile URL (FanFicFare often omits it for newer sites), try our own
    // generic extraction, then an LLM fallback. Best-effort — never blocks.
    if meta.author_url.is_empty() && !meta.author.trim().is_empty() {
        match state.http_client.get(query).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(page_text) = resp.text().await {
                    let llm = OllamaLlmAdapter {
                        ollama: &state.ollama,
                        chat_model: &state.config.ollama_chat_model,
                    };
                    if let Some(author_url) = crate::scrape::author_link::ensure_author_url(
                        Some(&llm),
                        &meta.author_url,
                        &meta.author,
                        &page_text,
                    )
                    .await
                    {
                        meta.author_url = author_url;
                    }
                }
            }
            _ => {}
        }
    }
```

Three things to learn here. First, the **adapter pattern**: the meta
handler needs an `LlmClient` trait (defined in `scrape/author_link.rs`),
but FicHub's actual Ollama client lives in `services/ollama.rs` with its
own shape. The tiny `OllamaLlmAdapter` struct implements the trait by
delegating — and even captures the *chat* model name, because the
client's own `model` field is the *embedding* model. That's a genuinely
subtle bug-avoidance detail: two different models, two different jobs,
and the adapter is where the distinction gets pinned down.

Second, the **best-effort philosophy**. Every failure path here is a
quiet no-op: if the fetch fails, if the page isn't text, if the LLM call
errors, if the extraction returns nothing — `meta.author_url` simply
stays empty and the response proceeds. A broken author link must never
break the metadata lookup. This is the opposite of the export pipeline's
fail-loud style, and that contrast is the lesson: *decide what's on the
critical path, and let everything else degrade gracefully.*

Third, look at where the adapter is constructed: it borrows `state.ollama`
and `state.config.ollama_chat_model`. We met `AppState` back in Part 2 —
this is exactly why it's an `Arc`-wrapped bundle of everything the
handlers might need: the db pool, the HTTP client, the scraper registry,
and now the LLM client.

⚠️ **Watch Out — enrichment only runs on the URL path.** The hash-lookup
path (`handle_hash_lookup_meta`) reads whatever is already stored and
never enriches — it can't, there's no page to fetch. So the same fic can
return `author_url: ""` when looked up by hash and a real URL when looked
up by URL, if the enrichment ran during the scrape. Your frontend must
treat `author_url` as best-effort data, exactly like the server does.

### 23.3 Auto-merge, blacklists, and the slug

With metadata in hand, the handler does three quick things before
building the response. First, the auto-merge step that links this scrape
to FicHub's `works` table:

```rust
    // Auto-merge: find or create work for this source
    let auto_merge_result = crate::works::find_or_create_work(&state.db, &meta).await?;
    let work_id = auto_merge_result.work_id;

    // Check blacklists
    let fic_blacklist = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;
```

`find_or_create_work` is FicHub's deduplication layer: given the scraped
metadata, it finds the canonical `work` row this fic belongs to (or
creates one). We'll dig into the full `works` model in a later part —
for now, the takeaway is that every meta response carries a `work_id`
that unifies all the URLs pointing at the same story across sites. And
the blacklist check is the greylist gate we'll revisit in Part 11 —
for now, notice it feeds the `notes` field:

```rust
    let slug = {
        let sanitized: String = meta.title
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        let re = regex_lite::Regex::new(r"_+").unwrap();
        let slug = re.replace_all(&sanitized, "_").to_string();
        format!("{}-{}", slug.trim_matches('_'), meta.url_id)
    };

    let notes: Vec<String> = if fic_blacklist.iter().any(|b| b.reason == 6) {
        vec!["This fic is greylisted.".to_string()]
    } else {
        vec![]
    };
```

The slug is a tiny pure function with two steps: (1) replace every
non-alphanumeric character with `_` — so *"Harry Potter and the
Chamber of Secrets"* becomes `Harry_Potter_and_the_Chamber_of_Secrets` —
then (2) collapse runs of `_` into one and append the url_id. The
url_id suffix is the crucial part: titles collide, url_ids don't. The
slug is the human-readable half of a unique key, and the hash is the
machine half. We'll see this exact slug appear again in Chapter 26 —
it's the URL segment for the fic's own page.

💡 **Key Concept — one envelope, many endpoints.** Compare this response
to the export response from Part 5: same `err`/`q`/`fixits`/`info`/`url_id`/
`work_id`/`slug`/`meta`/`hashes`/`urls`/`epub_url`/.../`notes` skeleton.
The meta endpoint returns the *same shape* with `hashes: {}` and all
download URLs null. That shared envelope is a contract: the frontend has
one response type for "tell me about this fic", and the download links
either exist or don't. When they do exist (export path), the client just
reads `epub_url`; when they don't (meta path), it knows to offer an
export button. One parser, two endpoints, zero special cases.

### 23.4 The response: JSON, not files

Finally, the payoff — the full envelope with the `meta` block:

```rust
    Ok(Json(json!({
        "err": 0,
        "q": query,
        "fixits": [],
        "info": format!("{} by {} - {} words, {} chapters", meta.title, meta.author, meta.words, meta.chapters),
        "url_id": meta.url_id,
        "work_id": work_id,
        "slug": slug,
        "meta": {
            "id": meta.url_id,
            "work_id": work_id,
            "title": meta.title,
            "author": meta.author,
            "chapters": meta.chapters,
            "words": meta.words,
            "description": meta.desc,
            "status": meta.status,
            "source": meta.source,
            "created": chrono::DateTime::from_timestamp_millis(meta.published)
                .map(|d| d.to_rfc3339()).unwrap_or_default(),
            "updated": chrono::DateTime::from_timestamp_millis(meta.updated)
                .map(|d| d.to_rfc3339()).unwrap_or_default(),
            "extra_meta": meta.extra_meta,
            "raw_extended_meta": meta.raw_extended_meta,
            "author_url": meta.author_url,
            "author_local_id": meta.author_local_id,
            "source_id": meta.source_id,
            "author_id": meta.author_id,
        },
        "hashes": {},
        "urls": {},
        "epub_url": null,
        "html_url": null,
        "mobi_url": null,
        "pdf_url": null,
        "notes": notes,
    })))
}
```

Read the `meta` block as a teaching list of *what FicHub knows about a
fic*: identity (`id`, `source_id`, `author_id`), presentation (`title`,
`author`, `description`), shape (`chapters`, `words`, `status`), and
provenance (`source`, timestamps, `author_url`). Two details deserve a
closer look.

The timestamps go through `chrono::DateTime::from_timestamp_millis(...)`
then `.to_rfc3339()`. The scrapers hand us `published`/`updated` as
milliseconds-since-epoch (that's the `FicMetadata` contract from Part 4),
but a JSON API should speak ISO 8601 — unambiguous, human-readable, and
what `DateTime::parse_from_rfc3339` on the frontend expects. The
`unwrap_or_default()` means an epoch-zero timestamp degrades to `"1970-01-01T00:00:00+00:00"` rather than failing the whole request —
again, best-effort.

The other detail is `"fixits": []` and the nulled-out URLs. The fixits
array is the curator-fixes hook (we'll meet it properly in Part 11's
curator content system), and the null URLs are the contract with the
export pipeline: `epub_url`, `html_url`, `mobi_url`, `pdf_url` all
present-but-null, screaming "nothing downloaded yet."

🧪 **Try It Yourself — compare meta vs. epub responses for the same
fic.** Run the export endpoint first (which populates `hashes` and
`urls`), then hit meta again with the same `q`. Watch `epub_url` stay
null in meta even though the file exists on disk. That's the *"without
generating/downloading files"* contract — meta is a snapshot of
metadata, not of the cache. If you want to know whether a download
exists, the export endpoint is the one that tells you (and it'll answer
from cache instantly, as Part 5 taught us).

And that's the whole metadata endpoint: one struct, two lookup paths,
one best-effort enrichment, one envelope. It's the smallest public API
in FicHub, and precisely because it's small, it's a perfect template for
every read-only endpoint you'll ever write: cheap by default, network
only when it must, and shaped identically to its expensive sibling.

⚠️ **Watch Out — `unwrap_or("")` hides a missing parameter.** The
`q` extraction uses `unwrap_or("")` and then checks `is_empty()`, which
means *both* a missing `?q` and an explicitly empty `?q=` produce the
same `{"err": -1, "msg": "no query"}` response. For this endpoint that's
fine — but notice the pattern: `Option<String>` plus a sentinel empty
string means "missing" and "empty" are indistinguishable downstream. If
you ever need to tell them apart (say, a default-value search), switch
to matching on the `Option` directly.

Next up in Chapter 24, we leave the single-fic view behind and dive into
the biggest, most intricate pure function in FicHub: the boolean query
parser that turns `"enemies to lovers" AND (fluff OR humor) -angst` into
a PostgreSQL tsquery.

## Chapter 24 — The Search Engine: search/parser.rs

Every search box in FicHub — the one in the nav bar, the one on the
search page, the one inside Ask the Archive (Part 10) — funnels into a
single, gloriously self-contained file: `src/search/parser.rs`, 737
lines including tests. No HTTP, no database, no I/O of any kind. It is a
pure function from *query string* to *boolean expression tree* to
*PostgreSQL tsquery string*, and it implements the whole user-facing
syntax documented in `docs/src/searching.md`:

- Simple words: `coffee`
- Quoted phrases: `"enemies to lovers"`
- AND: `coffee AND angst`
- OR: `coffee OR tea`
- NOT: `NOT major character death` or `-major character death`
- Exclusion prefix: `-angst`
- Fielded search: `title:harry`, `author:jk`, `fandom:...`, `character:...`,
  `relationship:...`
- Parenthesized grouping: `(fluff OR humor) AND -angst`
- Mixed: `"enemies to lovers" AND (fluff OR humor) -angst`

Before we read any code, let me tell you why this file deserves a whole
chapter. A query language is a *security surface*: the user's raw string
eventually lands inside SQL. If the parser naively interpolates input
into a `to_tsquery('english', ...)` call, a malicious query like
`a'); DROP TABLE fic_info; --` becomes an injection vector. FicHub
defuses this the right way: parse the string into a typed tree, then
*regenerate* a tsquery from the tree, never from the raw text. The user
never reaches SQL — their input reaches an AST, and the AST is the only
thing that gets quoted into the query. That's the architectural lesson
of this chapter, and it's worth more than the syntax itself.

### 24.1 The model: QueryTerm and BooleanExpression

Everything hangs off two enums. First, the leaf:

```rust
/// A parsed query leaf term.
#[derive(Debug, Clone, PartialEq)]
pub enum QueryTerm {
    /// A simple word (will be stemmed for tsquery)
    Word(String),
    /// A quoted phrase: "coffee shop" → distance-based tsquery
    Phrase(String),
    /// An excluded term: -angst or -"major character death"
    Excluded(String),
    /// A fielded search: title:something → applies as a separate WHERE clause
    Fielded {
        field: String,
        value: String,
    },
}
```

Four kinds of leaf, and each one maps to a different *destination* when
the query runs: `Word` and `Phrase` go into the full-text `tsvector`
match; `Excluded` becomes a negation (and later, as we'll see in Chapter
25's builder, a tag-based NOT EXISTS); `Fielded` is pulled out of the
text search entirely and applied as a separate WHERE clause (ILIKE for
`title:`/`author:`, tag filters for `fandom:`/`character:`/`relationship:`).
Keeping the four kinds distinct in the *model* is what makes those
different destinations possible — a `String` soup could never tell them
apart.

Then the tree:

```rust
/// A boolean expression tree.
#[derive(Debug, Clone)]
pub enum BooleanExpression {
    /// All sub-expressions must match (AND — implicit between terms)
    And(Vec<BooleanExpression>),
    /// At least one sub-expression must match (OR)
    Or(Vec<BooleanExpression>),
    /// The sub-expression must NOT match (NOT)
    Not(Box<BooleanExpression>),
    /// A leaf term
    Term(QueryTerm),
}
```

This is the classic recursive AST shape. `And` and `Or` hold a *vec* of
sub-expressions (so `a AND b AND c` is one node, not a chain — we'll see
why flattening matters in 24.3). `Not` holds exactly one sub-expression,
boxed so the enum stays a fixed size. `Term` is the base case. Because
the enum is recursive, any function over it is naturally recursive too —
and the `Display` impl is the perfect first example:

```rust
impl fmt::Display for BooleanExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BooleanExpression::And(terms) => {
                let parts: Vec<String> = terms.iter().map(|t| t.to_string()).collect();
                write!(f, "({})", parts.join(" AND "))
            }
            BooleanExpression::Or(terms) => {
                let parts: Vec<String> = terms.iter().map(|t| t.to_string()).collect();
                write!(f, "({})", parts.join(" OR "))
            }
            BooleanExpression::Not(inner) => write!(f, "NOT ({})", inner),
            BooleanExpression::Term(t) => write!(f, "{}", t),
        }
    }
}
```

💡 **Key Concept — the AST is the single source of truth.** Everything in
the search pipeline is a transformation *of* this tree: `Display` renders
it back to text, `expr_to_tsquery` compiles it to SQL syntax, the
`extract_*` functions walk it for fielded/excluded terms. Because the
tree is the one canonical representation, every downstream consumer sees
exactly the same structure — the tokenizer and parser produce it once,
and nobody ever re-parses the raw string. That's the difference between
a query *language* and a pile of string surgery.

### 24.2 The tokenizer: characters in, tokens out

Parsing starts with tokenizing — splitting the raw string into a flat
list of meaningful atoms:

```rust
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Word(String),
    Phrase(String), // content inside quotes, including the quotes
    ParenOpen,
    ParenClose,
    OpAnd,
    OpOr,
    OpNot,
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '(' => {
                tokens.push(Token::ParenOpen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::ParenClose);
                chars.next();
            }
            // A '-' immediately followed by a quote or word is an exclusion
            // prefix on the NEXT token (e.g. `-angst`, `-"major character death"`).
            // Emit it as its own Word token so the parser can glue it.
            '-' => {
                chars.next();
                tokens.push(Token::Word("-".to_string()));
            }
            '"' => {
                chars.next(); // consume opening "
                let mut content = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '"' {
                        chars.next(); // consume closing "
                        break;
                    }
                    content.push(ch);
                    chars.next();
                }
                tokens.push(Token::Phrase(content));
            }
            ' ' | '\t' | '\n' | '\r' => {
                chars.next(); // skip whitespace
            }
            _ => {
                // Read a word (up to whitespace, parens, or quote)
                let mut word = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' || ch == '(' || ch == ')' || ch == '"' {
                        break;
                    }
                    word.push(ch);
                    chars.next();
                }
                // Check for operators (case-insensitive)
                let upper = word.to_uppercase();
                match upper.as_str() {
                    "AND" => tokens.push(Token::OpAnd),
                    "OR" => tokens.push(Token::OpOr),
                    "NOT" => tokens.push(Token::OpNot),
                    _ => tokens.push(Token::Word(word)),
                }
            }
        }
    }

    tokens
}
```

The design detail that prevents a whole class of bugs hides in the
`'-'` arm. A minus sign is *not* swallowed into the next word: it's
emitted as its own `Word("-")` token, and the parser glues it to the
next primary later. Why bother? Because `-angst` and `-"major character
death"` would otherwise need *two* different tokenizer behaviors — one
where `-` prefixes a word, one where it prefixes a phrase. By making the
minus a standalone token, the tokenizer stays dumb and uniform, and the
*parser* handles both cases in one place. This is the same principle as
the earlier design rule: emit a lone `-` as its own token, then glue in
the parser.

Also notice the `'"'` arm: everything between quotes is collected
verbatim into a `Phrase`, including spaces. That's what makes `"enemies
to lovers"` one token instead of three — and it's also why the fielded
case `title:"harry potter"` needs special handling later: the word
reader stops at `"`, so `title:` becomes a `Word`, and the quoted part
arrives as the *next* `Phrase` token.

⚠️ **Watch Out — the word reader stops at quotes but not at colons.**
`title:harry` tokenizes as a single `Word("title:harry")` — the colon
isn't a delimiter. The fielded-term detection therefore happens in the
parser (24.4), not the tokenizer, and it must check the *known field
names* before deciding something is a field. That's why `1:Harry`
(tested in the file) stays a plain word while `title:harry` becomes a
fielded term — and why `matches!(field.as_str(), "title" | "author" |
"fandom" | "character" | "relationship" | "source")` gates the whole
thing.

### 24.3 The parser: recursive descent with precedence

With tokens in hand, the parser builds the tree. FicHub uses a
recursive-descent design with three levels, one per precedence tier:

```rust
/// Precedence (highest to lowest):
/// 1. NOT (prefix unary)
/// 2. AND (binary)
/// 3. OR (binary)
pub fn parse_query(input: &str) -> BooleanExpression {
    let tokens = tokenize(input);
    if tokens.is_empty() {
        return BooleanExpression::And(vec![]);
    }
    let mut pos = 0;
    parse_or_expr(&tokens, &mut pos)
}
```

`parse_query` is the only public entry point: tokenize, handle the empty
query (an empty `And` — a subtle choice we'll justify in 24.5), and
delegate to the lowest-precedence function. Each `parse_*_expr` function
handles one level and delegates *down* for its operands:

```rust
/// Parse OR expressions (lowest precedence)
fn parse_or_expr(tokens: &[Token], pos: &mut usize) -> BooleanExpression {
    let mut left = parse_and_expr(tokens, pos);

    while *pos < tokens.len() {
        match &tokens[*pos] {
            Token::OpOr => {
                *pos += 1;
                let right = parse_and_expr(tokens, pos);
                // Flatten OR chains: Or([Or([a, b]), c]) → Or([a, b, c])
                left = match (left, right) {
                    (BooleanExpression::Or(mut terms), BooleanExpression::Or(more)) => {
                        terms.extend(more);
                        BooleanExpression::Or(terms)
                    }
                    (BooleanExpression::Or(mut terms), r) => {
                        terms.push(r);
                        BooleanExpression::Or(terms)
                    }
                    (l, BooleanExpression::Or(mut terms)) => {
                        let mut all = vec![l];
                        all.append(&mut terms);
                        BooleanExpression::Or(all)
                    }
                    (l, r) => BooleanExpression::Or(vec![l, r]),
                };
            }
            _ => break,
        }
    }

    left
}
```

The OR loop is a textbook left-associative fold with one inspired
addition: the **flattening**. `a OR b OR c` naively parses as
`Or([Or([a, b]), c])` — a right-nested tree. The match arms detect both
operands being (or containing) `Or` nodes and *merge their vecs*, so the
result is always a flat `Or([a, b, c])`. Why does flatness matter? The
tsquery emitter (24.5) joins an `Or`'s children with ` | ` — a nested
tree emits `((a | b) | c)`, which PostgreSQL accepts, but flat trees are
simpler to reason about, simpler to test (`fluff:* | humor:* | angst:*`
appears verbatim in the tests), and cheaper to walk. Same story for
`And`:

```rust
/// Parse AND expressions (medium precedence)
fn parse_and_expr(tokens: &[Token], pos: &mut usize) -> BooleanExpression {
    let mut terms = Vec::new();
    terms.push(parse_not_expr(tokens, pos));

    while *pos < tokens.len() {
        match &tokens[*pos] {
            Token::OpAnd => {
                *pos += 1;
                terms.push(parse_not_expr(tokens, pos));
            }
            Token::OpOr | Token::ParenClose => break,
            // Implicit AND between adjacent terms
            Token::Word(_) | Token::Phrase(_) | Token::OpNot | Token::ParenOpen => {
                terms.push(parse_not_expr(tokens, pos));
            }
            _ => break,
        }
    }

    if terms.len() == 1 {
        terms.into_iter().next().unwrap()
    } else {
        BooleanExpression::And(terms)
    }
}
```

Two things make `parse_and_expr` special. First, the **implicit AND**:
adjacent words with no operator at all (`coffee shop`) still become an
`And([coffee, shop])`. That arm is what makes `harry potter` mean
`harry AND potter`, which is the single most important UX decision in
the whole syntax — users type space-separated words and get intersection
semantics for free.

Second, the **single-element collapse**: if the loop collected only one
term, it returns the term itself, not `And([term])`. A one-word query
`coffee` is `Term(Word("coffee"))`, *not* `And([Term(...)])`. This
matters because the tsquery emitter special-cases empty `And`s, and
because tests assert the exact shape — the parser's contract with its
own tests is that a single term unwraps to itself. Every `parse_*`
function ends with this same collapse-or-wrap decision, and the whole
parser stays consistent because of it.

Then the highest-precedence level, NOT:

```rust
/// Parse NOT expressions (highest precedence — prefix unary)
fn parse_not_expr(tokens: &[Token], pos: &mut usize) -> BooleanExpression {
    if *pos >= tokens.len() {
        return BooleanExpression::And(vec![]);
    }

    match &tokens[*pos] {
        Token::OpNot => {
            *pos += 1;
            let inner = parse_primary(tokens, pos);
            BooleanExpression::Not(Box::new(inner))
        }
        // A lone '-' word glues to the NEXT token as an exclusion.
        // Handles `-angst` and `-"major character death"`.
        Token::Word(w) if w == "-" => {
            *pos += 1;
            let inner = parse_primary(tokens, pos);
            match inner {
                BooleanExpression::Term(QueryTerm::Word(w)) => {
                    BooleanExpression::Not(Box::new(BooleanExpression::Term(QueryTerm::Excluded(w))))
                }
                BooleanExpression::Term(QueryTerm::Phrase(p)) => {
                    BooleanExpression::Not(Box::new(BooleanExpression::Term(QueryTerm::Excluded(p))))
                }
                other => BooleanExpression::Not(Box::new(other)),
            }
        }
        _ => parse_primary(tokens, pos),
    }
}
```

Here's where the tokenizer's lone-`-` decision pays off. The `Word("-")`
arm consumes the minus, parses the *next* primary, and rewraps it as an
`Excluded` term inside a `Not`. One code path serves `-angst` (next
primary is a word) *and* `-"major character death"` (next primary is a
phrase). And because `Excluded` is *always* produced wrapped in `Not`,
the tsquery emitter can render `Excluded` bare — the `Not` supplies the
`!`. That division of labor is the reason the tsquery tests assert
`!(angst:*)` and never `!(!(angst:*))`.

Finally, `parse_primary` — parens, words, phrases, and the fielded-term
detection:

```rust
/// Parse primary expressions (parenthesized groups, words, phrases)
fn parse_primary(tokens: &[Token], pos: &mut usize) -> BooleanExpression {
    if *pos >= tokens.len() {
        return BooleanExpression::And(vec![]);
    }

    match &tokens[*pos] {
        Token::ParenOpen => {
            *pos += 1;
            let expr = parse_or_expr(tokens, pos);
            // Expect closing paren
            if *pos < tokens.len() && tokens[*pos] == Token::ParenClose {
                *pos += 1;
            }
            expr
        }
        Token::Word(w) => {
            *pos += 1;
            // Check for fielded search (word:value), including word:"phrase"
            if let Some(colon_pos) = w.find(':') {
                if colon_pos > 0 {
                    let field = w[..colon_pos].to_lowercase();
                    let trailing = &w[colon_pos + 1..];
                    // Only treat as fielded if it's a known field
                    if matches!(
                        field.as_str(),
                        "title" | "author" | "fandom" | "character" | "relationship" | "source"
                    ) {
                        // Case 1: title:harry — value is in the same word
                        if !trailing.is_empty() {
                            return BooleanExpression::Term(QueryTerm::Fielded {
                                field,
                                value: trailing.to_string(),
                            });
                        }
                        // Case 2: title:"harry potter" — value is the next Phrase token
                        if let Some(Token::Phrase(p)) = tokens.get(*pos) {
                            *pos += 1;
                            return BooleanExpression::Term(QueryTerm::Fielded {
                                field,
                                value: p.clone(),
                            });
                        }
                        // Case 3: title: with nothing after — treat as empty fielded
                        return BooleanExpression::Term(QueryTerm::Fielded {
                            field,
                            value: String::new(),
                        });
                    }
                }
            }
            // Check for exclusion prefix
            if w.starts_with('-') && w.len() > 1 {
                let inner = w[1..].to_string();
                return BooleanExpression::Not(Box::new(BooleanExpression::Term(
                    QueryTerm::Excluded(inner),
                )));
            }
            BooleanExpression::Term(QueryTerm::Word(w.clone()))
        }
        Token::Phrase(p) => {
            *pos += 1;
            BooleanExpression::Term(QueryTerm::Phrase(p.clone()))
        }
        ...
    }
}
```

Read the `Word` arm as a three-question decision tree. **Question 1: does
it contain a colon?** If yes, is the part before the colon a *known
field*? Only then does it become `Fielded` — and there are three
sub-cases: value inline (`title:harry`), value in the next phrase token
(`title:"harry potter"`), or empty value (`title:` — tolerated rather
than erroring). **Question 2: does it start with `-`?** This arm handles
the *inline* exclusion (`-angst` written without a space), which the
tokenizer left as a single word — a second path to `Excluded`, and the
reason both `-angst` and `- angst` parse identically. **Question 3:
neither?** Then it's a plain `Word`.

The `ParenOpen` arm is the recursion engine: parse everything inside the
parens at the *lowest* precedence (`parse_or_expr`), then expect a
closing paren — and note the `if` rather than an assert: a missing
closing paren is tolerated, not fatal. The parser is forgiving by design,
because a search box should never crash on user input; it should
interpret what it can and drop the rest.

🧪 **Try It Yourself — feed the parser garbage.** The forgiving design
isn't hypothetical. Queries like `(` (bare open paren), `-` (lone
minus), `title:` (empty field), and `"unclosed phrase` (unterminated
quote) all parse to *something* rather than erroring. Run the test
suite and then add a few of your own cases to `src/search/parser.rs`:

```bash
cd /personal/documents/code/rust/fichub && cargo test -p fichub search::parser
```

Then in `tests/mod.rs` or a scratch test, assert what
`parse_query("(")` produces. You'll find it degrades to an empty `And`
— the parser's "nothing matched" sentinel. That's the contract: no
panic, ever, on any input.

### 24.4 Compiling the tree to tsquery

The AST is only half the story — the whole point is the PostgreSQL
string. Here's the compiler:

```rust
/// Convert a parsed BooleanExpression to a PostgreSQL tsquery string.
///
/// e.g. `"enemies to lovers" AND angst -fluff` → `'enimies' <-> 'to' <-> 'lover' & 'angst' & !('fluff':*)`
pub fn expr_to_tsquery(expr: &BooleanExpression) -> String {
    match expr {
        BooleanExpression::And(terms) => {
            if terms.is_empty() {
                return String::new();
            }
            let parts: Vec<String> = terms.iter().map(expr_to_tsquery).collect();
            parts.join(" & ")
        }
        BooleanExpression::Or(terms) => {
            if terms.is_empty() {
                return String::new();
            }
            let parts: Vec<String> = terms.iter().map(expr_to_tsquery).collect();
            if parts.len() == 1 {
                parts.into_iter().next().unwrap()
            } else {
                format!("({})", parts.join(" | "))
            }
        }
        BooleanExpression::Not(inner) => {
            let inner_ts = expr_to_tsquery(inner);
            if inner_ts.is_empty() {
                return String::new();
            }
            format!("!({})", inner_ts)
        }
        BooleanExpression::Term(term) => term_to_tsquery(term),
    }
}
```

This is a straight structural recursion — the tree's shape *is* the
output's shape. `And` joins children with ` & `, `Or` joins with ` | `
wrapped in parens (unless there's a single child — the same collapse
rule the parser uses), `Not` wraps in `!(...)`, and a term goes to the
leaf compiler. Every branch returns `String::new()` for empty input,
which means a `Not` of an empty group *disappears* rather than emitting
`!()` — malformed tsquery avoided before PostgreSQL ever sees it.

The leaf compiler is where the real translation decisions live:

```rust
fn term_to_tsquery(term: &QueryTerm) -> String {
    match term {
        QueryTerm::Word(w) => {
            // tsquery format with stemming and prefix matching
            // Use lowercase for consistency with tsvector
            format!("{}:*", w.to_lowercase())
        }
        QueryTerm::Phrase(p) => {
            // For phrases, use the <-> (followed by) operator between words
            let words: Vec<&str> = p.split_whitespace().collect();
            if words.is_empty() {
                return String::new();
            }
            let parts: Vec<String> = words
                .iter()
                .map(|w| format!("{}:*", w.to_lowercase()))
                .collect();
            parts.join(" <-> ")
        }
        QueryTerm::Excluded(e) => {
            // Renders bare — the enclosing Not() supplies the leading '!'.
            // (Excluded is always wrapped in Not by the parser.)
            let words: Vec<&str> = e.split_whitespace().collect();
            if words.len() > 1 {
                // Excluded phrase: -"major character death" → major:* <-> character:* <-> death:*
                let parts: Vec<String> = words
                    .iter()
                    .map(|w| format!("{}:*", w.to_lowercase()))
                    .collect();
                parts.join(" <-> ")
            } else {
                format!("{}:*", e.to_lowercase())
            }
        }
        QueryTerm::Fielded { value, .. } => {
            // Fielded search value becomes part of the text tsquery
            // Field-specific filtering is handled separately as a WHERE clause
            format!("{}:*", value.to_lowercase())
        }
    }
}
```

Let's unpack the three big ideas, because each one encodes a PostgreSQL
lesson you'll use for the rest of your career.

**Idea 1: the `:*` prefix-match suffix.** A word emits as `word:*`, not
`word`. In tsquery syntax, `:*` means "this lexeme, prefix-matched" —
so `enemies:*` matches `enemies`, `enemy`, `enemies'` etc. at the
lexeme level. Critically, the emission is the *raw* word plus `:*`; the
parser does **not** stem it. `enemies` becomes `enemies:*`, never
`enemi:*` — PostgreSQL's `to_tsquery('english', ...)` does the stemming
at query time, when it lexes the tsquery against the dictionary. If the
parser pre-stemmed, it would fight the dictionary and break words like
`running` (→ `run`). Raw in, stemmed at the database. This is the
prefix-match rule from our pitfalls list, and the tests assert the raw
form verbatim.

**Idea 2: `<->` is "immediately followed by".** A quoted phrase compiles
to its words joined with the tsquery distance operator `<->`, meaning
"the next lexeme follows directly". `"enemies to lovers"` becomes
`enemies:* <-> to:* <-> lovers:*` — and PostgreSQL's english dictionary
drops stopwords like *to* when lexing, so the phrase match stays
accurate. Why not just AND them? Because `enemies AND to AND lovers`
would match a fic where the three words appear *anywhere* in the text
separately; the `<->` chain requires them adjacent in order. That's the
difference between "contains these words" and "contains this phrase" —
and it's exactly what the quotes promise the user.

**Idea 3: `Excluded` renders bare.** An excluded term emits its words
(joined with `<->` if multi-word) *without* any negation — because the
parser guarantees `Excluded` only ever appears inside a `Not`, which
supplies the `!`. If `Excluded` emitted its own `!(...)`, you'd get
double negation (`!(!(angst:*))`), which in tsquery is a *different*
expression than `!(angst:*)` and would silently exclude everything. The
invariant "Excluded is always wrapped in Not" is maintained in exactly
one place (parse_not_expr) and relied on in exactly one place
(term_to_tsquery) — a clean split that makes the double-negation bug
structurally impossible.

⚠️ **Watch Out — a `Phrase` in tsquery is distance-based, not literal.**
`"enemies to lovers"` compiles to `enemies:* <-> to:* <-> lovers:*`,
not a quoted string literal. That means a phrase match is really an
adjacency match over *lexemes* — stemming applies, stopwords get
dropped, and punctuation inside the phrase is discarded. For a
fanfiction search that's exactly right (users want `"enemies to lovers"`
as a trope, and AO3 itself normalizes the phrase), but don't copy this
design to a system where users expect byte-exact literal matching —
there you'd need a different operator entirely.

### 24.5 Walking the tree: extract_fielded_terms and friends

The tsquery handles the *text* side of the search. But fielded terms
(`title:harry`) and exclusions (`-angst`) need to become *other* kinds
of clauses — ILIKE filters and tag NOT EXISTS — in the final SQL. That's
the job of the extractors, which walk the tree and pull out what the
builder needs:

```rust
/// Extract fielded search terms from an expression.
/// Returns a list of (field, value) pairs.
/// This separates fielded terms so they can be applied as WHERE clauses,
/// while non-fielded terms continue as text search.
pub fn extract_fielded_terms(expr: &BooleanExpression) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    extract_fielded_recurse(expr, &mut fields);
    fields
}

fn extract_fielded_recurse(expr: &BooleanExpression, fields: &mut Vec<(String, String)>) {
    match expr {
        BooleanExpression::And(terms) | BooleanExpression::Or(terms) => {
            for t in terms {
                extract_fielded_recurse(t, fields);
            }
        }
        BooleanExpression::Not(inner) => {
            extract_fielded_recurse(inner, fields);
        }
        BooleanExpression::Term(QueryTerm::Fielded { field, value }) => {
            fields.push((field.clone(), value.clone()));
        }
        _ => {}
    }
}
```

And its sibling for exclusions:

```rust
/// Extract excluded terms from an expression (for NOT-based exclusion).
pub fn extract_excluded_terms(expr: &BooleanExpression) -> Vec<String> {
    let mut excluded = Vec::new();
    extract_excluded_recurse(expr, &mut excluded);
    excluded
}

fn extract_excluded_recurse(expr: &BooleanExpression, excluded: &mut Vec<String>) {
    match expr {
        BooleanExpression::Not(inner) => {
            match inner.as_ref() {
                BooleanExpression::Term(QueryTerm::Excluded(e)) => {
                    excluded.push(e.clone());
                }
                BooleanExpression::Term(QueryTerm::Word(w)) => {
                    excluded.push(w.clone());
                }
                _ => {
                    // Nested NOT — recurse
                    extract_excluded_recurse(inner, excluded);
                }
            }
        }
        BooleanExpression::And(terms) | BooleanExpression::Or(terms) => {
            for t in terms {
                extract_excluded_recurse(t, excluded);
            }
        }
        _ => {}
    }
}
```

Two walkers, one shape: recurse through every container node, collect
the leaves of the target kind. Note that the exclusion walker catches
*both* `Excluded` terms *and* bare `Word`s sitting under a `Not` —
because `NOT angst` (keyword form) parses as `Not(Term(Word("angst")))`
while `-angst` parses as `Not(Term(Excluded("angst")))`. Both mean the
same thing to the user, and both must reach the exclusion filter.

Then the keystone: `extract_text_tsquery`, which produces the tsquery
*without* the fielded terms — the exact string the builder hands to
PostgreSQL:

```rust
/// Get the full-text search tsquery from an expression (excluding fielded terms).
pub fn extract_text_tsquery(expr: &BooleanExpression) -> String {
    // Reconstruct expression without fielded terms
    let cleaned = remove_fielded_terms(expr);
    expr_to_tsquery(&cleaned)
}

/// Remove fielded terms from an expression tree, returning a clean tree.
fn remove_fielded_terms(expr: &BooleanExpression) -> BooleanExpression {
    match expr {
        BooleanExpression::And(terms) => {
            let cleaned: Vec<BooleanExpression> = terms
                .iter()
                .filter_map(|t| {
                    let c = remove_fielded_terms(t);
                    match &c {
                        BooleanExpression::And(v) if v.is_empty() => None,
                        _ => Some(c),
                    }
                })
                .collect();
            if cleaned.is_empty() {
                BooleanExpression::And(vec![])
            } else if cleaned.len() == 1 {
                cleaned.into_iter().next().unwrap()
            } else {
                BooleanExpression::And(cleaned)
            }
        }
        BooleanExpression::Or(terms) => { /* same shape, OR-flavored */ }
        BooleanExpression::Not(inner) => {
            let cleaned = remove_fielded_terms(inner);
            match &cleaned {
                BooleanExpression::And(v) if v.is_empty() => BooleanExpression::And(vec![]),
                _ => BooleanExpression::Not(Box::new(cleaned)),
            }
        }
        BooleanExpression::Term(QueryTerm::Fielded { .. }) => {
            // Remove fielded terms from text search
            BooleanExpression::And(vec![])
        }
        other => other.clone(),
    }
}
```

`remove_fielded_terms` is a *filtering* traversal: fielded leaves become
empty `And`s, empty `And`s get dropped from parents, and a parent left
with a single child collapses to that child (the same invariant as the
parser). The result: `title:harry AND angst` filters to just `angst`,
compiles to `angst:*`, and the fielded term rides along in the
`fielded_terms` vec for the ILIKE clause. Two destinations, one tree,
zero ambiguity.

Why is the separation so careful? Because of the **fielded-only
trap**. If a query is *entirely* fielded — `title:harry` — then
`extract_text_tsquery` returns `""`, and if the route handler blindly
fell back to `plainto_tsquery('english', "title:harry")`, PostgreSQL
would tokenize the colon into `title & harry` and match nothing. The
route handler (which we'll meet in 24.6) must therefore check: empty
tsquery *and* non-empty fielded terms means "clear the text query
entirely and rely on the field clauses alone." The parser gives the
handler the information to make that call — `extract_text_tsquery` empty
is *meaningful*, not just "no text".

💡 **Key Concept — emptiness is a message.** `And(vec![])` isn't a bug
or a crash; it's the parser's way of saying "nothing here." Empty tsquery
→ no text constraint. Empty fielded list → no field constraints. The
pipeline treats emptiness as data and reacts differently in each case.
When you design your own parsers, give your AST a first-class "empty"
state and handle it explicitly everywhere — sentinel strings that
pretend to be real values are how silent wrong-result bugs are born.

### 24.6 The tests: the parser's own spec

The file ends with 30+ unit tests, and they're not an afterthought —
they *are* the spec. Every feature of the syntax has an exact expected
tsquery string:

```rust
#[test]
fn test_parse_quoted_phrase() {
    let expr = parse_query("\"coffee shop\"");
    let ts = expr_to_tsquery(&expr);
    assert_eq!(ts, "coffee:* <-> shop:*");
}

#[test]
fn test_parse_exclusion_dash() {
    let expr = parse_query("-angst");
    let ts = expr_to_tsquery(&expr);
    assert_eq!(ts, "!(angst:*)");
}

#[test]
fn test_parse_or_operator() {
    let expr = parse_query("coffee OR tea");
    let ts = expr_to_tsquery(&expr);
    assert_eq!(ts, "(coffee:* | tea:*)");
}

#[test]
fn test_parse_fielded_search_title() {
    let expr = parse_query("title:harry");
    let fields = extract_fielded_terms(&expr);
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0], ("title".to_string(), "harry".to_string()));
    // Text tsquery should be empty (fielded term removed)
    let text_ts = extract_text_tsquery(&expr);
    assert!(text_ts.is_empty());
}

#[test]
fn test_parse_mixed_fielded_and_text() {
    let expr = parse_query("title:harry AND angst");
    let fields = extract_fielded_terms(&expr);
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0], ("title".to_string(), "harry".to_string()));
    let text_ts = extract_text_tsquery(&expr);
    assert_eq!(text_ts, "angst:*");
}

#[test]
fn test_parse_complex_nested_parentheses() {
    let expr = parse_query("(fluff OR humor OR angst) AND (\"slow burn\" OR \"enemies to lovers\")");
    let ts = expr_to_tsquery(&expr);
    assert!(ts.contains("fluff:* | humor:* | angst:*"));
    // The other side has phrases with <-> operator
    assert!(ts.contains("slow:* <-> burn:*"));
    assert!(ts.contains("enemies:* <-> to:* <-> lovers:*"));
}

#[test]
fn test_exclusion_with_phrase() {
    // -"major character death" — this requires careful tokenization
    // The '-' is tokenized as a separate Word("-") then Phrase("major character death")
    // Then parse_not_expr should handle the '-' prefix
    let expr = parse_query("-\"major character death\"");
    // This should parse as Not(Phrase("major character death"))
    let ts = expr_to_tsquery(&expr);
    assert!(ts.contains("!"));
    assert!(ts.contains("major:* <-> character:* <-> death:*"));
}

#[test]
fn test_not_a_colon_field() {
    // "1:Harry" is not a fielded search — it doesn't match known field names
    let expr = parse_query("1:Harry");
    let fields = extract_fielded_terms(&expr);
    assert_eq!(fields.len(), 0);
    // Should be treated as a plain word
    let ts = expr_to_tsquery(&expr);
    assert_eq!(ts, "1:harry:*");
}
```

Read these as a *conversation* between the tests and the design. The
phrase test pins the `<->` adjacency. The exclusion tests pin the
single-`!` rule. The fielded test pins the *empty text tsquery* for
fielded-only queries. The `1:Harry` test pins the known-field gate. The
complex-expression test pins the OR flattening (`fluff:* | humor:* |
angst:*` verbatim — the flat shape we fought for in 24.3). Every
design rule from our pitfalls list has a test that would fail if the
rule were broken, which is exactly what makes refactoring this parser
safe.

🧪 **Try It Yourself — validate against real PostgreSQL.** A tsquery
string that *looks* right can still be invalid SQL syntax. The parser's
output should be fed to the database itself:

```sql
SELECT to_tsquery('english', 'enemies:* <-> to:* <-> lovers:* & angst:* & !(fluff:*)');
```

If that returns a tsquery instead of an error, the string is valid.
Script it for all the trickiest generated strings — excluded phrases,
parenthesized ORs, empty queries — and you'll catch malformed-syntax
bugs the unit tests can't (they only check string equality, not
PostgreSQL validity).

That's the parser: tokenizer → recursive-descent parser → flattened AST →
tsquery compiler + extractors, with the tests as the executable spec.
But a parser is only half a search engine — the other half is what the
route handler *does* with the tree, which is where we're headed next:
how `src/search/routes.rs` turns `extract_fielded_terms` into ILIKE
clauses and excluded terms into tag filters, and how `SearchQueryBuilder`
assembles it all into one battle-tested SQL statement.

## Chapter 25 — main_char_attr: The "Dark Harry" Semantics

Some search features are pure engineering — you build them, you move on.
And then there are the ones where the *domain* is the hard part: you
can't write the SQL until you've answered a question about how the world
works. This chapter is about one of those. The question: **what does it
mean for a tag to apply to a character?**

Consider the search "dark Harry". A naive implementation looks for fics
that have both a Harry tag and a Dark tag. That returns... a lot of
noise: fics where Harry shows up for three paragraphs as a side
character while the *actual* main character, say Draco, carries the Dark
tag. The fic isn't "about" Dark Harry at all — it just contains him. If
you've ever used a fanfiction archive's tag search, you've felt this
pain: tags are per-*fic*, but the interesting questions are per-*character*.

FicHub's answer is `main_char_attr`, a single query parameter whose
whole job is to make "attribute applies to the *main* character" a
first-class, SQL-level truth:

```
main_char_attr=Harry Potter|Dark Harry Potter
```

`Character | Attribute`. And the semantics, straight from the docs at
`docs/src/searching.md`:

> **Main-Character Attribute** — `main_char_attr=Character|Attribute` finds
> stories where the **main character** (the first-listed character) has a
> specific attribute tag. Example: `Harry Potter|Dark Harry Potter` finds fics
> **starring** Harry with the Dark tag — not fics where Harry is a side
> character and the Dark tag belongs to someone else.

Everything in this chapter hangs off the phrase "the first-listed
character" — a *scraping-time* decision that becomes a *query-time*
superpower. Let's see how.

### 25.1 Where the "main" comes from: scrape-time scoring

Back in Part 4, when a scraper extracts a story's metadata, it doesn't
just dump character names into the database — it attaches a *score* that
encodes relative importance. The model lives in `src/scrape/mod.rs`:

```rust
/// FicHub's tag model: i16 scores (DB column `fic_tags.score` smallint).
/// The crate uses f64; convert at the boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub tag_type_id: i16,
    /// Relative importance for "main" semantics: the first character in the
    /// metadata is the MAIN character (score 10), secondary characters 1;
    /// the first ship is the primary pairing (5), others 1; freeforms etc. 0.
    pub score: i16,
}

impl ExtractedTag {
    pub fn fandom(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 1, score: 0 } }
    pub fn character(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 2, score: 1 } }
    pub fn character_main(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 2, score: 10 } }
    pub fn relationship(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 3, score: 1 } }
    pub fn relationship_primary(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 3, score: 5 } }
    pub fn freeform(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 4, score: 0 } }
    pub fn warning(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 5, score: 0 } }
    pub fn category(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 6, score: 0 } }
}
```

Read the doc comment carefully, because it's the entire domain model
compressed into one sentence:

- The **first-listed character** in the metadata is the main character:
  score `10`.
- Every other character: score `1`.
- The **first-listed relationship** is the primary pairing: score `5`;
  others `1`.
- Fandoms, freeforms, warnings, categories: score `0` — they're
  per-fic facts, not ranked.

These numbers are stored per-fic in the `fic_tags` join table
(`url_id`, `tag_id`, `score`), and they're the raw material for every
"main" concept in FicHub. Notice the deliberate *scale*: 10 vs 1 is a
huge gap, and it's on purpose. Scores aren't meant to be subtle — they're
a yes/no signal with margin, so "is this the main character?" is
answerable with a simple `ORDER BY score DESC LIMIT 1` instead of a
threshold guess. When the FanFicFare crate hands back f64 scores, the
`From` impl scales them (`score: (t.score * 10.0).round() as i16`) so
the crate's own "main" semantics (1.0) land on FicHub's 10.

💡 **Key Concept — write data once, in a shape future queries can read.**
The score column is written at scrape time and *never* touched by the
search code. The search feature is free because the scraping layer made
a cheap decision (first-listed = main) and persisted it as a number.
This is the meta-lesson of Chapter 25: when you're building the
ingestion layer of a system, every little structured fact you store is a
future query you won't have to build. The inverse is also true — flatten
everything to free text at ingestion, and you'll be writing gnarly
parsing code in every feature forever.

### 25.2 The parameter: one pipe-separated string

The search API accepts `main_char_attr` as a single string parameter,
and the route handler splits it. The parsing lives in
`src/search/routes.rs`, inside `into_search_params`:

```rust
    /// Main-character attribute: "Character|Attribute" — finds fics where the
    /// highest-scored character tag is Character (the MAIN character) AND the
    /// freeform tag Attribute is present. E.g. "Harry Potter|Dark Harry Potter".
    pub main_char_attr: Option<String>,
```

and its conversion:

```rust
            main_char_attr: self.main_char_attr.as_deref().and_then(|s| {
                // Format "Character|Attribute" — split on the LAST '|' so
                // character names containing '|' (rare) still work.
                s.split_once('|').map(|(c, a)| (c.trim().to_string(), a.trim().to_string()))
                    .filter(|(c, a)| !c.is_empty() && !a.is_empty())
            }),
```

Two details are doing real work. First, `split_once('|')` on the *last*
pipe: a character name containing a pipe (rare, but this is a
user-typed string — never assume) still splits correctly, because the
attribute half is everything after the final `|`. Second, the
`.filter(...)` drops the pair unless *both* halves are non-empty after
trimming: `"Harry Potter|"` or `"|Dark"` or just `"Harry Potter"` all
silently become `None` — no error, no weird query, the filter just
doesn't apply. Forgiveness again: a malformed advanced parameter should
no-op, never 500.

The result is `main_char_attr: Option<(String, String)>` on
`SearchParams` — a tuple, not two separate fields, because the two
halves only mean something *together*. The doc comment on the struct
field in `src/search/builder.rs` says it best:

```rust
    /// Main-character attribute: format "Character|Attribute" — finds fics
    /// where the HIGHEST-SCORED character tag is `Character` (i.e. the main
    /// character, per scrape-time scoring) AND the freeform tag `Attribute`
    /// is present. E.g. "Harry Potter|Dark Harry Potter" finds fics starring
    /// Harry with the Dark tag — not fics where Harry is a side character.
    pub main_char_attr: Option<(String, String)>,
```

### 25.3 The SQL: "highest-scored character is X, and Y is present"

Now the payoff — the WHERE clause. It lives in `SearchQueryBuilder::push_where_clauses` in `src/search/builder.rs`, and it's two
subqueries in a trench coat:

```rust
        // main_char_attr — the fic's highest-scored character tag must be
        // `Character` AND the freeform tag `Attribute` must be present.
        // This is "attribute applies to the MAIN character", not just any
        // fic with both tags. The main character is the char tag with the
        // highest score (scrape-time: first-listed = score 10, others 1).
        if let Some((character, attribute)) = &self.params.main_char_attr {
            // 1) The main (highest-scored) character tag equals `character`
            qb.push(" AND (SELECT ft2.tag_id FROM fic_tags ft2");
            qb.push(" JOIN tags t2 ON t2.id = ft2.tag_id");
            qb.push(" WHERE ft2.url_id = fi.id AND t2.tag_type_id = 2");
            qb.push(" ORDER BY ft2.score DESC LIMIT 1) IN (");
            qb.push("SELECT t3.id FROM tags t3 WHERE t3.name ILIKE ");
            qb.push_bind(format!("%{}%", character));
            qb.push(" AND t3.tag_type_id = 2)");
            // 2) The freeform attribute tag is present
            qb.push(" AND EXISTS (SELECT 1 FROM fic_tags ft4");
            qb.push(" JOIN tags t4 ON t4.id = ft4.tag_id");
            qb.push(" WHERE ft4.url_id = fi.id AND t4.name ILIKE ");
            qb.push_bind(format!("%{}%", attribute));
            qb.push(" AND t4.tag_type_id = 4)");
        }
```

Read clause 1 inside-out, because it's the heart of the "Dark Harry"
semantics:

1. **Pick the fic's main character tag.** The inner subquery grabs the
   fic's character tags (`tag_type_id = 2`), orders them by `score
   DESC`, and takes the first: `LIMIT 1`. Thanks to the scrape-time
   scoring, that's the first-listed character — the fic's protagonist,
   with its score-10 badge.
2. **Check it matches the requested character.** The single tag id from
   step 1 must be `IN` the set of tag ids whose name ILIKEs the
   character half of the parameter. `ILIKE '%Harry Potter%'` rather than
   `=` is a deliberate looseness: users type from memory, and the
   parameter should match *Harry Potter*, *Harry James Potter*, or any
   spelling the archive uses. (Tag names are title-cased in the
   database, and the parser lowercases query text — case-insensitive
   matching is mandatory, exactly as our tag-lookup pitfall warns.)
3. **The attribute must exist as a freeform tag.** Clause 2 is a plain
   `EXISTS`: the fic carries *some* freeform tag (`tag_type_id = 4`)
   whose name ILIKEs the attribute half. Dark, Dark Harry Potter,
   Morally Grey Harry Potter — whichever the author actually used.

The conjunction is what makes it *semantic*. The fic must (a) *star*
Harry — his tag is literally the top-scored character tag — and (b)
carry a Dark freeform tag *somewhere*. A fic where Draco is the main
character and Harry is a score-1 extra, no matter how many Dark tags it
has, fails clause 1 and never appears. That's the "Dark Harry" filter,
and it's correct in a way that `include_tags=2:Harry Potter,4:Dark` can
never be.

⚠️ **Watch Out — ILIKE is loose by design, and that's a trade-off.**
`ILIKE '%Harry Potter%'` also matches "Harry Potter & Related Fandoms"
(a *fandom* tag — but the subquery constrains `tag_type_id = 2`, so
fandoms can't sneak in) and could match a character tag named
"Harry Potter's Aunt" if one existed. FicHub accepts this looseness
because the alternative — exact-name matching — silently returns zero
results for `main_char_attr=harry potter|dark` when the stored tag is
title-cased `Harry Potter`. Case-insensitive substring matching is the
pragmatic middle ground: it errs toward false positives (which users can
see and refine) instead of false negatives (which look like the feature
is broken). If you copy this pattern, remember the trade-off is a
product decision, not a free win.

### 25.4 Why it's a parameter, not parser syntax

You might wonder: why does "Dark Harry" ship as a *query parameter*
(`main_char_attr=...`) instead of a search-box syntax like
`character:"harry potter" AND freeform:"dark"`? Two honest reasons,
both worth internalizing.

First, **the semantics don't decompose.** The whole point is that the
attribute applies to the *main* character — a relationship *between*
two filters, not two independent filters. The boolean parser has no
way to express "the tag that wins the max-score contest must be X".
(And indeed, when the parser sees `character:harry`, it emits an
`include_any_tags` filter — the naive, any-character version, which is
exactly the wrong semantics for this feature.) The parameter carries the
*compound* intent as a single unit, so the builder can emit the
correlated subqueries.

Second, **it's a UI affordance.** The advanced-filters panel in the
frontend renders a dedicated "Main-Character Attribute" input with a
placeholder like `Harry Potter|Dark Harry Potter`. A structured
parameter maps 1:1 to a form field. You *can* teach users
`character:"harry potter"` syntax; you can't teach them a two-line
subquery in a search box.

That's not to say the parser is useless here — remember from Chapter 24
that excluded terms are *also* resolved to tags. The two systems are
complementary: the parser handles text-level boolean logic, and the
structured parameters handle the filters that *mean* something compound.
The builder assembles both into one WHERE clause, which is the elegant
part: `?q="enemies to lovers" -angst&main_char_attr=Harry Potter|Dark
Harry Potter` is a single SQL statement where the phrase filter, the
exclusion, and the main-character semantics all coexist.

🧪 **Try It Yourself — run the query by hand.** Fire up `psql` and
recreate the subquery shape against your local database to *see* the
semantics:

```sql
-- The fic's main character tag, per fic:
SELECT ft.url_id, t.name, ft.score
FROM fic_tags ft JOIN tags t ON t.id = ft.tag_id
WHERE t.tag_type_id = 2
ORDER BY ft.url_id, ft.score DESC;
```

You'll see score-10 rows sitting at the top of each fic's block —
those are the protagonists. Then add the two clauses from the builder
and count fics where the top row is Harry *and* a Dark freeform tag
exists. That count is the exact answer the API returns for
`main_char_attr=Harry Potter|Dark Harry Potter`.

### 25.5 The scoring system's other children

`main_char_attr` is the star, but the same `score` column powers a whole
family of "main" features, and one of them shares the builder:

```rust
        // primary_tag — fic must have this tag as its HIGHEST-scored fic_tag
        // (the tag with the max score among all the fic's tags must BE this
        // tag). Mirrors the main_char_attr "main character" subquery.
        if let Some(ref tag) = self.params.primary_tag {
            qb.push(" AND (SELECT ft2.tag_id FROM fic_tags ft2");
            qb.push(" WHERE ft2.url_id = fi.id");
            qb.push(" ORDER BY ft2.score DESC LIMIT 1) IN (");
            qb.push("SELECT t3.id FROM tags t3 WHERE t3.name = ");
            qb.push_bind(&tag.tag_name);
            qb.push(" AND t3.tag_type_id = ");
            qb.push_bind(tag.tag_type_id);
            qb.push(")");
        }
```

`primary_tag` asks a broader question: "is this tag the fic's *main*
tag, period?" — not just among characters, but among *all* its tags.
Same subquery shape, one column wider: order *every* `fic_tags` row by
score, take the max, and check it's the requested tag. The comment even
says it *"mirrors the main_char_attr 'main character' subquery"* — the
codebase is honest about the pattern it's reusing. When you see a second
subquery that looks like the first, that's not copy-paste debt; it's a
pattern that earned its keep. (And the facet queries in Chapter 24's
handler use `score >= hidden_threshold` to keep low-scored noise tags
out of the facet sidebar — one more child of the same score column.)

⚠️ **Watch Out — `hidden_threshold` gates *visibility*, not *main-ness*.**`
The `ft.score >= hidden_threshold` filter appears in almost every
tag-related clause of the builder. It's FicHub's spam/SEO defense: tags
scored below the threshold (config `tag_hidden_threshold`) are treated
as noise and ignored for filtering and display. But notice the
`main_char_attr` clause deliberately does *not* apply it to the
character subquery — a main character is identified by *rank*, not by
absolute score, so a threshold could never disqualify a protagonist's
tag. The threshold and the rank-based logic answer different questions,
and mixing them would silently break one of them.

That's the "Dark Harry" story: a scrape-time scoring decision, a
pipe-separated parameter, a correlated subquery pair, and a whole class
of "main" features growing from the same score column. It's the
single best example in FicHub of domain modeling paying off at the
query layer — and it's why the search engine feels *smart* to users
instead of just functional.

Next up in Chapter 26, we switch from finding fics to *reading* them:
the web reader that serves the cached HTML bundle through
`/api/reader/{url_id}` and renders it chapter by chapter.

## Chapter 26 — The Web Reader: routes/reader.rs

We've built the whole loop of FicHub except one thing: actually reading
the fic. Downloads are great, but sometimes you want to read right here,
in the browser, with your font size remembered and your place saved. That
job belongs to the reader endpoint — and it's the perfect bridge between
Part 5's file cache and Part 6's JSON world, because it does something
clever: it *reuses the export cache as a database*.

The route is tiny — 64 lines, the smallest handler in this part. Don't
let the size fool you. `src/routes/reader.rs` is a masterclass in
composition: it layers the database (fic info), the config (versioning),
the query layer (export logs), and the disk cache (the HTML zip) into
one response. Let's read it in full.

### 26.1 The handler: fetch, locate, unzip, serve

```rust
use axum::extract::{Path, State};
use axum::Json;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::AppError;
use crate::server::AppState;
use crate::db::queries;
use crate::cache;

pub async fn reader_handler(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    // Get fic info
    let fic = queries::get_fic_info(&state.db, &url_id).await?
        .ok_or_else(|| AppError::NotFound("Fic not found".into()))?;

    // Determine version and hash for cache lookup
    let version_bump = queries::get_fic_version_bump(&state.db, &url_id).await?.unwrap_or(0);
    let version = state.config.export_version + version_bump;
    let input_hash = fic.content_hash.clone().unwrap_or_else(|| "upstream".to_string());

    // Find the export log for HTML format
    let log = queries::find_export_log(&state.db, &url_id, version, "html", &input_hash).await?
        .ok_or_else(|| AppError::NotFound("No HTML cache. Export the fic first.".into()))?;

    // Build cache path and read the zip
    let cache_path = cache::disk::cache_path(&state.config.cache_dir, &cache::EType::Html, &url_id, &log.export_hash);

    if !cache_path.exists() {
        return Err(AppError::NotFound("HTML file not found on disk".into()));
    }

    // Read the zip and extract index.html
    let zip_data = std::fs::read(&cache_path)?;
    let cursor = std::io::Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| AppError::Internal(format!("Failed to read HTML cache zip: {}", e)))?;

    let html_content = archive.by_name("index.html")
        .map_err(|_| AppError::NotFound("index.html not found in cache bundle".into()))?;

    // Read the entry content into a string
    use std::io::Read;
    let mut html_string = String::new();
    let mut reader = html_content;
    reader.read_to_string(&mut html_string)?;

    // Look up work_id for reading_stats tracking
    let work_id = queries::get_work_by_source(&state.db, &url_id).await?
        .map(|w| w.id).unwrap_or(0);

    Ok(Json(json!({
        "err": 0,
        "url_id": url_id,
        "title": fic.title,
        "author": fic.author,
        "work_id": work_id,
        "html": html_string,
        "words": fic.words,
        "chapters": fic.chapters,
    })))
}
```

Now walk it as a pipeline, because that's exactly what it is — five
stages, each one producing the key the next stage needs:

**Stage 1 — the fic.** `get_fic_info` fetches the `fic_info` row by
url_id. This is the same database-mode lookup we met in Chapter 23, and
it fails with a 404-ish `AppError::NotFound` if the fic has never been
scraped. Everything downstream depends on this row — including `fic.content_hash`, which we'll need in a second.

**Stage 2 — the version.** The reader must find the *right* HTML file
for this fic, and "right" is a three-part key:

```rust
    // Determine version and hash for cache lookup
    let version_bump = queries::get_fic_version_bump(&state.db, &url_id).await?.unwrap_or(0);
    let version = state.config.export_version + version_bump;
    let input_hash = fic.content_hash.clone().unwrap_or_else(|| "upstream".to_string());
```

Remember Part 5's export log? Every export records the (url_id, version,
format, input_hash) quadruple that produced a file. The reader
reconstructs that exact quadruple: `version` is the global export
version *plus* any per-fic bump (a way for admins to force re-exports of
specific fics — we'll meet version bumps properly in Part 11), and
`input_hash` is the fic's content hash, defaulting to the string
`"upstream"` when the fic was never fingerprinted (an older row). This
default matters: `find_export_log` must match *something*, and a NULL
hash would never match a logged row. By normalizing NULL to a sentinel
string, the lookup always has a value to compare.

**Stage 3 — the log.** `find_export_log` is the cache index:

```rust
    let log = queries::find_export_log(&state.db, &url_id, version, "html", &input_hash).await?
        .ok_or_else(|| AppError::NotFound("No HTML cache. Export the fic first.".into()))?;
```

This is the moment the reader reveals its dependency: **the fic must
have been exported in HTML format at least once.** There is no
on-demand generation here — no semaphore, no scraper, no builder. If the
export log has no HTML row for this exact version+hash, the reader
answers `"No HTML cache. Export the fic first."` and the frontend shows
an "Export to read" button. That's a deliberate product decision: the
reader is a *consumer of the cache*, not a second export pipeline. One
pipeline (Part 5) builds files; every other feature reads them. When the
user clicks export in the reader UI, they're routed through the *same*
export endpoint we studied — and the very next read of the reader finds
the log row and serves instantly.

**Stage 4 — the file.** The log row carries `export_hash` — remember
the hash-is-the-contract lesson from Part 5. The cache path is built
from that hash:

```rust
    let cache_path = cache::disk::cache_path(&state.config.cache_dir, &cache::EType::Html, &url_id, &log.export_hash);

    if !cache_path.exists() {
        return Err(AppError::NotFound("HTML file not found on disk".into()));
    }
```

Two failure modes live here, and they're different. The *log* can exist
while the *file* is missing — a deleted cache directory, a pruned file,
a disk hiccup. That's why the handler checks `exists()` explicitly
instead of trusting the index. The error message — `"HTML file not
found on disk"` — is distinct from `"No HTML cache"`, and that
distinction is for operators: one says "export it", the other says
"your cache is inconsistent with your database." Log row present, file
absent: that's a cache-integrity symptom worth a glance at the disk.

**Stage 5 — the unzip.** The HTML cache entry is a *zip* (Part 5's HTML
bundle: `index.html` plus assets), so reading it is a three-step:

```rust
    // Read the zip and extract index.html
    let zip_data = std::fs::read(&cache_path)?;
    let cursor = std::io::Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| AppError::Internal(format!("Failed to read HTML cache zip: {}", e)))?;

    let html_content = archive.by_name("index.html")
        .map_err(|_| AppError::NotFound("index.html not found in cache bundle".into()))?;
```

`std::fs::read` slurps the whole zip into memory (fics are big but not
*that* big — and the reader is single-fic, so this is fine), wraps it in
a `Cursor` (because `ZipArchive` wants a `Read + Seek`), opens the
archive, and pulls the `index.html` entry. Note the error mapping
discipline: a corrupt zip is an `AppError::Internal` (server problem),
but a zip missing `index.html` is an `AppError::NotFound` (cache problem).
Same crate, two different semantics — and the HTTP status codes differ
accordingly.

Then, the response — and here's the design decision that makes the
frontend's life easy:

```rust
    Ok(Json(json!({
        "err": 0,
        "url_id": url_id,
        "title": fic.title,
        "author": fic.author,
        "work_id": work_id,
        "html": html_string,
        "words": fic.words,
        "chapters": fic.chapters,
    })))
```

The *entire fic*, as one HTML string, in one JSON payload. Not a chapter
list, not a paginated API — the full bundle, plus its metadata
(title/author/words/chapters for headers and stats) and the `work_id`
(for reading-stats tracking — note it's `unwrap_or(0)` when the fic
isn't linked to a work yet, so the endpoint never fails on an
un-merged fic).

💡 **Key Concept — serve files as JSON, not as files.** The reader
endpoint is a file-serving endpoint wearing a JSON costume. It could
have returned a redirect to `/cache/<hash>` (the way download links
work), but instead it *embeds* the file content in a structured payload.
Why? Because the frontend needs the metadata *and* the content in one
round-trip, and because the HTML string is the *input* to a
transformation (chapter splitting, rendering) rather than the final
artifact. When your frontend must process a file, an API that returns
the bytes inside a typed envelope is often friendlier than a raw file
URL. The `Content-Type` is `application/json`, but the payload is a
document — and that's fine, because the client decides what to do with
it.

### 26.2 The frontend contract: ReaderApiResponse

The frontend's side of this contract is spelled out in
`frontend/src/routes/read/[urlId]/reader-lib.ts` — a TypeScript
interface that mirrors the Rust JSON exactly:

```typescript
/** Response contract of GET /api/reader/{url_id} (backend: src/routes/reader.rs). */
export interface ReaderApiResponse {
  err: number;
  msg?: string;
  url_id: string;
  title: string;
  author: string;
  work_id: number | null;
  html: string;
  words: number;
  chapters: number;
}
```

Look at the two files side by side and you'll see the contract in
action: every field the Rust `json!` emits has a TS type here, and the
`msg` is the optional error-text slot (populated on `err != 0`). The
`err` field is the shared envelope we met in Chapter 23 — 0 means
success, anything else carries `msg`. When you write a client against a
JSON API, this mirror-image interface is the *documentation* — it tells
you what to expect and what might be missing, and it lets the compiler
catch field-name typos that would otherwise surface as `undefined` at
runtime.

### 26.3 Splitting the bundle: one HTML string becomes chapters

The raw HTML bundle is one giant document — the whole fic, every
chapter, in reading order. The frontend's job is to carve it into
per-chapter pieces. That logic lives in `reader-lib.ts` in a pure,
framework-free function (the file's header comment brags about this
deliberately: *"Kept free of Svelte so it can be unit-tested directly
with vitest"*):

```typescript
/**
 * Split a fic's full HTML (as produced by the html bundle: one `<h2 id="chN">`
 * per chapter) into per-chapter fragments. Returns [] if no chapter headings
 * are found. The returned fragments are wrapped in <div class="chapter"> so
 * each can be rendered independently.
 */
export function splitChapters(html: string, titles: string[] = []): ReaderChapter[] {
  const h2Re = /<h2\b[^>]*id="ch(\d+)"[^>]*>(.*?)<\/h2>/gi;
  const headings: { id: number; title: string; h2Start: number; bodyStart: number }[] = [];
  let match: RegExpExecArray | null;
  while ((match = h2Re.exec(html)) !== null) {
    const id = parseInt(match[1], 10);
    const rawTitle = match[2].replace(/<[^>]*>/g, '').trim();
    const title = rawTitle || titles[id - 1] || `Chapter ${id}`;
    headings.push({
      id,
      title,
      h2Start: match.index,
      bodyStart: h2Re.lastIndex, // just past the closing </h2>
    });
  }
  if (headings.length === 0) return [];

  // Body of chapter N runs from the end of its </h2> to the start of the next
  // chapter's <h2> tag (or the end of the document for the last chapter).
  const chapters: ReaderChapter[] = [];
  for (let i = 0; i < headings.length; i++) {
    const start = headings[i].bodyStart;
    const end = i + 1 < headings.length ? headings[i + 1].h2Start : html.length;
    const body = html.slice(start, end).trim();
    const words = countWords(stripTags(body));
    chapters.push({
      title: headings[i].title,
      content: `<div class="chapter">${body}</div>`,
      words,
    });
  }
  return chapters;
}
```

This is the quiet contract between Part 5's HTML generator and Part 6's
reader: **every chapter heading in the bundle is an `<h2 id="chN">`**.
The generator emits that exact shape (we saw the HTML family of
generators in Part 5), and the reader leans on it with a regex:
`/<h2\b[^>]*id="ch(\d+)"[^>]*>(.*?)<\/h2>/gi`. Each match records the
chapter number, the stripped title, and the *offsets* — `h2Start` (where
the heading begins) and `bodyStart` (where its body begins, i.e.
`lastIndex` just past the closing `</h2>`).

Then the slicing: chapter *N*'s body runs from its `bodyStart` to the
*next* heading's `h2Start` (or the document end for the last chapter).
That's the whole trick — the headings are the bookmarks, and everything
between them is a chapter. The fragment gets wrapped in
`<div class="chapter">` so the reader can render each piece
independently inside the same page, and the word count comes from
`countWords(stripTags(body))` — strip tags, count word-like runs.

Two robustness details are worth your attention. First, the **fallback
path**: if no headings match at all (an old bundle, a different
generator), `splitChapters` returns `[]`, and the caller falls back to
a single pseudo-chapter containing the whole body:

```typescript
export function readerWorkFromPayload(json: ReaderApiResponse, urlId: string): ReaderWork {
  const chapters = splitChapters(json.html);
  if (chapters.length === 0) {
    // No per-chapter headings — fall back to a single "chapter" with the
    // whole body so the reader still works.
    chapters.push({
      title: json.title,
      content: `<div class="chapter">${stripTags(json.html)}</div>`,
      words: countWords(json.html),
    });
  }
  ...
}
```

The reader never breaks on unexpected markup — it degrades to
one-giant-chapter mode, which is ugly but *works*. Second, the title
fallback chain: `rawTitle || titles[id - 1] || \`Chapter ${id}\`` — use
the heading's own text, else a provided titles array (from the API, for
bodies that skip titles), else a generic "Chapter N". Three layers of
graceful degradation, one regex.

⚠️ **Watch Out — the chapter splitter trusts a format invariant.** If
the HTML generator ever changes its heading shape (say, `<h3 class="ch">`
instead of `<h2 id="chN">`), `splitChapters` silently stops matching and
every fic falls into the single-chapter fallback — the reader *works*,
but chapter navigation, per-chapter word counts, and position restoring
all degrade at once. There's no error, no log, nothing. This is the
cost of coupling two components through an implicit markup contract:
the regex is the API. When you build similar pairs, make the contract
explicit — a shared constant, a doc comment on both sides (as here, in
reader-lib.ts), or a schema — and add a test that pins the generated
markup shape so a generator change breaks loudly in CI.

### 26.4 Loading, caching, and the offline story

The chapter-splitting is pure logic, but the *loading* is where the
reader earns its "works offline" reputation:

```typescript
/**
 * Fetch the full fic HTML from the reader API and split it into chapters.
 * Throws on network failure, non-OK responses, and err != 0 — unless a
 * previously cached reader payload exists in localStorage, in which case the
 * cached HTML is returned so previously-opened fics keep reading offline.
 */
export async function loadReaderData(urlId: string): Promise<ReaderLoadResult> {
  try {
    const res = await fetch(`/api/reader/${encodeURIComponent(urlId)}`);
    if (!res.ok) throw new Error(`Failed to load reader (HTTP ${res.status})`);
    const json = (await res.json()) as ReaderApiResponse;
    if (json.err !== 0) throw new Error(json.msg || 'Failed to load reader');
    saveReaderHtml(urlId, json);
    return { work: readerWorkFromPayload(json, urlId), fromCache: false };
  } catch (err) {
    // Offline / server error — fall back to the last-cached reader HTML so
    // previously opened fics render without a network connection.
    const cached = tryCachedReaderHtml(urlId);
    if (cached) return { work: readerWorkFromPayload(cached, urlId), fromCache: true };
    throw err;
  }
}
```

The strategy is a classic **cache-aside read with write-through**: on a
successful fetch, the raw API payload is stored in localStorage
(`saveReaderHtml` — keyed `fichub:reader:html:<urlId>`); on *any*
failure — network down, 500, `err != 0` — the loader checks the cache and
renders the last-known payload instead, flagging `fromCache: true` so
the UI can show an offline banner. Every previously opened fic is
therefore readable with zero connectivity. That's the PWA promise we'll
see fully fleshed out in Part 12 — but notice the seed of it here: one
`try/catch`, one `localStorage.setItem`, and the reader survives a
dropped connection.

The storage helpers are all written to *never throw*:

```typescript
/** Store the raw reader API JSON so the fic re-renders offline. */
export function saveReaderHtml(urlId: string, payload: ReaderApiResponse): void {
  try {
    localStorage.setItem(htmlCacheKey(urlId), JSON.stringify(payload));
  } catch {
    /* storage full/blocked — the SW cache may still cover offline */
  }
}

/** Safely read the last-cached reader payload (null on any failure). */
export function tryCachedReaderHtml(urlId: string): ReaderApiResponse | null {
  try {
    const raw = localStorage.getItem(htmlCacheKey(urlId));
    if (!raw) return null;
    const parsed = JSON.parse(raw) as ReaderApiResponse;
    return parsed && typeof parsed === 'object' && typeof parsed.html === 'string' ? parsed : null;
  } catch {
    return null;
  }
}
```

localStorage can throw (quota exceeded, disabled entirely in private
mode) and `JSON.parse` can throw (corrupt blob). Both helpers swallow
everything and degrade: save fails → the service worker may still cover
offline; load fails → `null`, and the caller falls through to the real
error. The comments even document *why* each swallow is safe. That's
the pattern for any optional persistence layer: the cache is a
convenience, never a dependency, so it must be incapable of breaking
the feature it serves.

And the reader state — font size, theme, chapter index, scroll
position — rides the same rails:

```typescript
export interface ReaderState {
  fontSize?: number;
  lineHeight?: number;
  maxWidth?: number;
  theme?: 'light' | 'sepia' | 'dark';
  indentParagraphs?: boolean;
  chapterIndex?: number;
  scrollPos?: number;
  finished?: boolean;
  lastReadAt?: number;
}

/** Safely read a reader state blob from localStorage (returns {} on any failure). */
export function restoreReaderState(urlId: string): ReaderState {
  try {
    const raw = localStorage.getItem(stateKey(urlId));
    if (!raw) return {};
    const parsed = JSON.parse(raw) as ReaderState;
    return typeof parsed === 'object' && parsed !== null ? parsed : {};
  } catch {
    return {};
  }
}

/** Persist reader state (swallows quota/storage errors). */
export function saveReaderState(urlId: string, state: ReaderState): void {
  try {
    localStorage.setItem(stateKey(urlId), JSON.stringify(state));
  } catch {
    /* storage full/blocked — non-fatal */
  }
}
```

Note the `restoreReaderState` type-check on the *parsed* value: `typeof
parsed === 'object' && parsed !== null ? parsed : {}`. A corrupt blob
that parses to a string or number would otherwise flow downstream as
"state" — this guard normalizes anything weird back to `{}`. When you
read untrusted-shaped data (and localStorage content is untrusted —
users edit it, old versions wrote different shapes), validate the shape,
don't trust the type.

### 26.5 The reader page: preferences, progress, and Next Up

The page component is `frontend/src/routes/read/[urlId]/+page.svelte` —
788 lines of Svelte 5 runes (the `$state`/`$derived` syntax you'll
master in Chapter 27). Its job: render one chapter at a time, remember
everything, and never lose the reader's place. The state is declared
up front in a beautifully organized block:

```svelte
  // ── Core load state ──────────────────────────────────────────────────────
  let loading = $state(true);
  let error = $state('');
  let work = $state<ReaderWork | null>(null);
  // Browser connectivity for the offline banner (mirrors OfflineIndicator).
  let navigatorOnLine = $state(typeof navigator !== 'undefined' ? navigator.onLine : true);
  let chapterList = $state<{ title: string; url_id: string }[]>([]);
  let chapterIndex = $state(0);
  let chapterTitle = $state('');
  let htmlContent = $state('');
  let words = $state(0);
  let totalWords = $state(0);
  let isLastChapter = $state(false);

  // ── Reader preferences (persisted in localStorage) ───────────────────────
  let fontSize = $state(18);
  let lineHeight = $state(1.6);
  let maxWidth = $state(720);
  let theme = $state<'light' | 'sepia' | 'dark'>('light');
  let indentParagraphs = $state(true);
```

Every user-facing preference has a `$state` variable with a sane
default, and every one of them gets persisted on change. The
preference toggles are tiny pure functions with clamp guards — the
pattern is identical for each:

```svelte
  function changeSize(delta: number) {
    fontSize = Math.max(12, Math.min(32, fontSize + delta));
    savePosition();
  }
  function changeLineHeight(delta: number) {
    lineHeight = Math.max(1.2, Math.min(2.4, Math.round((lineHeight + delta) * 10) / 10));
    savePosition();
  }
  function changeWidth(delta: number) {
    maxWidth = Math.max(480, Math.min(1100, maxWidth + delta));
    savePosition();
  }
  function cycleTheme() {
    theme = theme === 'light' ? 'sepia' : theme === 'sepia' ? 'dark' : 'light';
    savePosition();
  }
```

`Math.max(min, Math.min(max, value))` is the clamp idiom — font size
bounded 12–32, line height 1.2–2.4 (rounded to one decimal so the value
stays tidy), width 480–1100. Every mutation calls `savePosition()`, the
single funnel that persists state *and* syncs the server:

```svelte
  function savePosition() {
    if (!work) return;
    const state = restoreReaderState(urlId);
    const saved = {
      ...state,
      fontSize,
      lineHeight,
      maxWidth,
      theme,
      indentParagraphs,
      chapterIndex,
      scrollPos: window.scrollY,
      finished: markSaved,
      lastReadAt: Date.now(),
    };
    saveReaderState(urlId, saved);
    // Server sync (best effort, only when signed in)
    void syncServerProgress('reading');
  }
```

Note the merge pattern: `restoreReaderState(urlId)` first, then spread
with the current values — so a partial write (say, only `fontSize`) never
clobbers the other fields. `lastReadAt: Date.now()` stamps every save,
making "continue where you left off" possible across devices (we'll meet
the full reading-stats sync in Part 7). And the server sync is fire-and-
forget (`void syncServerProgress(...)`) — best-effort, throttled, and
silently skipped when signed out.

Scroll progress drives both the top progress bar and the auto-save:

```svelte
  function syncToScroll() {
    const doc = document.documentElement;
    const total = doc.scrollHeight - window.innerHeight;
    const pct = total > 0 ? Math.min(100, Math.round((window.scrollY / total) * 100)) : 0;
    scrollProgress = pct;
    progress = pct;
    // Save reading position (debounced by caller)
    savePosition();
  }
```

`(scrollY / (scrollHeight - innerHeight)) * 100` — the standard
"how far through the page am I" formula, guarded against divide-by-zero
(`total > 0`) and clamped to 100. (The `debounced by caller` comment
points at the scroll listener: `syncToScroll` fires on every scroll
event, but `savePosition`'s heavy lifting — localStorage writes — is
throttled upstream, so a fast scroll doesn't hammer the disk.)

Chapter navigation is deliberately simple — a `chapterIndex` plus
`renderChapter`, which swaps the HTML and resets the scroll:

```svelte
  function goToChapter(index: number) {
    if (!work || index < 0 || index >= chapterList.length) return;
    chapterIndex = index;
    scrollTo(0, 0);
    const saved = restoreReaderState(urlId);
    saved.chapterIndex = index;
    saved.scrollPos = 0;
    saveReaderState(urlId, saved);
    void syncServerProgress('reading');
    // Re-render content for the new chapter
    htmlContent = '';
    renderChapter(index);
  }

  function renderChapter(index: number) {
    if (!work) return;
    const ch = work.chapters[index];
    if (!ch) return;
    chapterTitle = ch.title;
    htmlContent = ch.content;
    words = ch.words;
    chapterIndex = index;
    isLastChapter = index >= work.chapters.length - 1;
    showNextUp = false;
    nextUpCandidates = [];
  }
```

Setting `htmlContent` to the chapter's fragment triggers Svelte's
reactivity: the markup block re-renders, and the reader is on the next
chapter. `isLastChapter` is computed by comparing against the array
length — that flag is what turns the "next" button into the "Next Up"
panel at the end of a fic.

🧪 **Try It Yourself — the keyboard is a reader.** Open any fic in the
reader, then press `ArrowRight` to go forward, `ArrowLeft` to go back,
and `f` to increase font size. All three come from one handler:

```svelte
  function handleKeydown(e: KeyboardEvent) {
    // Don't hijack typing in inputs / selects
    const tag = (e.target as HTMLElement)?.tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || (e.target as HTMLElement)?.isContentEditable) return;
    switch (e.key) {
      case 'ArrowRight': e.preventDefault(); nextChapter(); break;
      case 'ArrowLeft': e.preventDefault(); prevChapter(); break;
      case 'f': case 'F': e.preventDefault(); changeSize(2); break;
    }
  }
```

The guard on the first line is the detail worth copying: keyboard
shortcuts must never hijack typing — if the user is in an input,
textarea, select, or contenteditable, the handler returns immediately.
Without that guard, pressing `f` while typing in a comments box would
jump the font size. (And the `prevChapter` at the first chapter does
something friendly: it navigates back to the fic's own page rather than
looping.)

The end-of-fic experience is the "Next Up" panel — three recommendation
sources, each fetched best-effort, merged and deduped:

```svelte
  async function buildNextUp() {
    if (!work) return;
    const candidates: { url_id: string; title: string; author: string; reason: string }[] = [];

    // 1. Next-in-series heuristic: same author, title looks like a sequel
    //    (contains a numeral / "II" / "2" / ordinal). Prefer the one that
    //    sorts right after the current title.
    const sequel = await findSequel(work);
    if (sequel) {
      candidates.push({ ...sequel, reason: 'Next in series' });
    }

    // 2. Top community suggestion (via /api/recommendations/votes)
    try {
      const res = await fetch(`/api/recommendations/votes?url_id=${encodeURIComponent(urlId)}`);
      const json = await res.json();
      if (json.err === 0 && json.suggestions?.length > 0) {
        const top = json.suggestions[0];
        const info = await fetchFicInfo(top.suggested_url_id);
        if (info) {
          candidates.push({
            url_id: top.suggested_url_id,
            title: info.title,
            author: info.author,
            reason: 'Top community suggestion',
          });
        }
      }
    } catch { /* skip gracefully */ }

    // 3. Readers also bookmarked (co-occurrence) — best effort
    try {
      const res = await fetch(`/api/reader/${encodeURIComponent(urlId)}/related`);
      const json = await res.json();
      if (json.err === 0 && Array.isArray(json.related)) {
        for (const rel of json.related.slice(0, 2)) {
          candidates.push({
            url_id: rel.url_id,
            title: rel.title,
            author: rel.author,
            reason: 'Readers also bookmarked',
          });
        }
      }
    } catch { /* skip gracefully */ }

    // Dedupe by url_id
    const seen = new Set<string>();
    nextUpCandidates = candidates.filter((c) => {
      if (seen.has(c.url_id)) return false;
      seen.add(c.url_id);
      return true;
    });
  }
```

Every `fetch` here is wrapped in its own `try/catch { /* skip
gracefully */ }` — a single recommendation source failing (votes API
down, related endpoint 500ing) must not prevent the others from
showing, and must never break the reading experience. The sources are
ordered by *confidence*: the sequel heuristic (same author + numeral in
title — a *very* strong signal) first, then the community's top vote,
then the co-occurrence picks. The dedupe at the end uses a `Set` —
cheap, linear, and order-preserving for the survivors.

🧪 **Try It Yourself — simulate the offline reader.** Load any fic in
the reader (so its HTML lands in localStorage), then open DevTools →
Network → Offline, and reload the page. The fic renders anyway — from
the localStorage cache — and the offline banner appears. Now look at
the `syncServerProgress` function and notice the throttle: `if (status
!== 'completed' && now - lastServerSync < 30_000) return;`. The reader
won't spam the server with a reading-status update more than once every
30 seconds while you scroll — only "completed" bypasses the throttle.

⚠️ **Watch Out — the reader redirects signed-out users.** The mount
handler is blunt about it:

```svelte
  onMount(async () => {
    await auth.init();
    // The reader works offline for previously opened fics; only redirect when
    // there is truly no session.
    if (!auth.isLoggedIn) { goto('/'); return; }
```

The reader is a *logged-in* feature (reading progress ties to your
account). If you're signed out, you get bounced to the home page — but
notice the comment's nuance: the offline cache *would* still work, so
the redirect is a product choice ("the reader is for members"), not a
technical necessity. When you build gated features, keep this
distinction in mind — it's the difference between a product decision
and a bug.

That's the reader: five pipeline stages on the server, one big JSON
payload, and a frontend that splits, persists, and remembers. From
"export first" to "read offline", it's the payoff of everything we
built in Parts 4 and 5 — the cache isn't just for downloads anymore, it
powers the reading experience itself.

Next up in Chapter 27, we climb to the top of the frontend: the SvelteKit
shell that hosts all of this — `+layout.svelte`, the home `+page.svelte`,
and the API client in `client.ts` that ties the browser to every
endpoint we've built in this part.

## Chapter 27 — The SPA Frontend: Layout, Home, and the API Client

For five chapters we've been reading Rust. Time to flip sides. Everything
you've built in Part 6 — the meta endpoint, the search engine, the
reader — is consumed by a SvelteKit single-page app living in
`frontend/`. This chapter is the tour of the shell that holds it all:
the root layout (`+layout.svelte`), the home page (`+page.svelte`), and
the small but mighty API client in `frontend/src/lib/api/client.ts`.

If you've never seen Svelte before, don't panic. This chapter doubles as
your first Svelte lesson, taught in the friendliest possible setting:
a real production app. The version here is Svelte 5, which introduced
the *runes* syntax you'll see everywhere — `$state`, `$derived`, `$props`,
`$effect`. Runes look like compiler magic (a `$` prefix that's *not*
jQuery!), but they're really just declarations: `$state` says "this
variable is reactive", `$derived` says "this value recomputes when its
inputs change", and Svelte's compiler rewrites the assignments
underneath. If you've used React, the mapping is roughly `$state` →
`useState`, `$derived` → `useMemo`, `$effect` → `useEffect` — but
without the hook rules, because Svelte's reactivity is language-level,
not function-level.

### 27.1 The API client: one function to rule them all

Let's start with the foundation — `client.ts`, the frontend's door to
every Rust endpoint. It's 81 lines, and the whole file is built around
one generic helper:

```typescript
// API client for the FicHub Rust backend (v0 API).
// All requests go to relative /api/* paths so they work behind the
// same-origin Rust server or any nginx proxy.

import type { ExportResponse, RecommendationsResponse } from './types';

const BASE = '/api';

// Client ID management for anonymous usage tracking
function getClientId(): string {
  const STORAGE_KEY = 'fichub_client_id';
  let clientId = localStorage.getItem(STORAGE_KEY);
  if (!clientId) {
    // Generate UUID v4
    clientId = 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
      const r = (Math.random() * 16) | 0;
      const v = c === 'x' ? r : (r & 0x3) | 0x8;
      return v.toString(16);
    });
    localStorage.setItem(STORAGE_KEY, clientId);
  }
  return clientId;
}
```

The header comment sets the architectural rule: **relative `/api/*`
paths**. The frontend never hardcodes a host — the app is served by the
same origin as the Rust server (or behind one nginx proxy), so
`fetch('/api/epub?...')` just works in dev and prod alike. No CORS
configuration, no environment-specific base URLs, no `localhost:8080`
leaking into a production bundle. Same-origin requests are the
simplest API client there is.

The `getClientId` function is the client-side half of the anonymous
usage tracking we saw logged in `insert_search_query` back in Chapter
24: every visitor gets a UUID v4 stored in localStorage (generated
client-side with a tiny regex-based UUID generator — no library needed),
and it rides along on every request as the `X-Client-ID` header. The
`(r & 0x3) | 0x8` bit-twiddle is the UUID spec's version-4 variant
marker — a fun one-liner you'll see in a thousand codebases.

Now the core — the `request` helper that *every* endpoint function goes
through:

```typescript
class ApiError extends Error {
  status: number;
  body: string;
  constructor(status: number, body: string) {
    super(`API error ${status}: ${body}`);
    this.name = 'ApiError';
    this.status = status;
    this.body = body;
  }
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  // Add client ID header to all requests
  const headers = new Headers(init?.headers);
  headers.set('X-Client-ID', getClientId());

  const res = await fetch(`${BASE}${path}`, { ...init, headers });
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new ApiError(res.status, text);
  }
  return (await res.json()) as T;
}
```

Three decisions here are worth internalizing. First, **the client ID is
injected at the chokepoint, not at each call site** — every request
automatically carries it, and nobody can forget. This is the same
"single funnel" philosophy we saw in the reader's `savePosition`:
cross-cutting concerns belong in one wrapper, not scattered through
callers. Second, **non-OK responses become typed errors**: `ApiError`
carries `status` and `body` so the UI can distinguish "404, fic not
found" from "500, server exploded" — a raw `fetch` only gives you a
`Response` object, and forgetting to check `res.ok` is how silent
failures creep in. Third, the **generic return**: `request<T>` parses
the JSON and casts it, so callers get typed results for free:

```typescript
function buildQuery(params: Record<string, string | number | undefined>): string {
  const entries = Object.entries(params).filter(
    ([, v]) => v !== undefined && v !== null && v !== '',
  );
  if (entries.length === 0) return '';
  const qs = entries
    .map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(String(v))}`)
    .join('&');
  return `?${qs}`;
}

/** GET /api/epub?q=<url> — export a fic and return download URLs. */
export async function fetchExport(url: string): Promise<ExportResponse> {
  return request<ExportResponse>(`/epub${buildQuery({ q: url })}`);
}

/** GET /api/meta?q=<url> — fetch fic metadata without downloading. */
export async function fetchMeta(url: string): Promise<ExportResponse> {
  return request<ExportResponse>(`/meta${buildQuery({ q: url })}`);
}

/** GET /api/recommendations?q=<url>&n=<n> — get recommendations for a fic. */
export async function fetchRecommendations(
  q?: string,
  url_id?: string,
  n = 20,
): Promise<RecommendationsResponse> {
  return request<RecommendationsResponse>(
    `/recommendations${buildQuery({ q, url_id, n })}`,
  );
}

export { ApiError };
```

`buildQuery` is the anti-footgun: it *filters out* `undefined`, `null`,
and empty-string values before encoding, so `fetchMeta(url)` produces
`/api/meta?q=<url>` and never a trailing `?q=` — and a caller passing
`{ q: undefined }` gets no query string at all. Every value is
`encodeURIComponent`'d on both key and value, which is the rule that
makes URLs with spaces, `&`, or unicode (fic titles! author names!)
safe. And note how thin the endpoint functions are: one path, one
query build, one typed return. The pattern scales to dozens of
endpoints with zero duplication — the whole file is a template for your
own API clients.

💡 **Key Concept — the client is a thin typed mirror of the server.**
Notice what `client.ts` does *not* do: no business logic, no fallback
semantics, no state. It's a 1:1 typed mirror of the Rust routes —
`fetchMeta` exists because `meta_handler` exists. That's a feature, not
a limitation. Keeping the client dumb means the server stays the single
source of truth, and the client's job is just *transport + types*.
When you build an SPA, resist the urge to make the client clever; make
it a transparent window into the API, and put the cleverness in the
server (where you can test it) or in dedicated modules like
`reader-lib.ts` (where you can unit-test it).

🧪 **Try It Yourself — read the other API modules.** `client.ts` is the
foundation, but the frontend has a whole `frontend/src/lib/api/`
directory — `search.ts` (which we saw building query strings for
`/api/search`), `social.ts`, `authors.ts`, and more. Open `search.ts`
and notice how `buildSearchQuery` there is the *same pattern* as
`client.ts`'s `buildQuery` — filter empties, encode everything, join
with `&` — applied to the twenty-parameter search filter object. When
you see the same shape in two places, that's the pattern earning its
keep; when you see it in three, that's a signal to extract a shared
helper.

### 27.2 The home page: +page.svelte

The home page is the most opinionated file in this chapter, because it
contains the strongest design opinion in the whole codebase. Here it
is, in its entirety:

```svelte
<script lang="ts">
  // The root page is intentionally empty: the tabbed UI lives in +layout.svelte.
  // This page renders nothing so the layout's Download tab is the default view.
</script>
```

That's it. Four lines. The root route renders *nothing*, and the reason
is explained in the comment: **the tabbed UI lives in `+layout.svelte`**.
In SvelteKit, the layout wraps every page — so the layout can render
the chrome (nav bar, tabs, footer) and the page slot is just the
*tab content*. By making the root page empty, the app's default view is
whatever the layout shows when no page content exists — the Download
tab, which is FicHub's primary entry point.

This is a real architectural pattern worth copying: **the layout is the
app; pages are content.** The home page has no business duplicating the
download form when the layout already hosts the tabs. When you see a
page component this empty, it's not a placeholder — it's a deliberate
division of responsibility. (And the comment isn't apologizing; it's
*teaching* the next developer why the file exists at all. Write comments
like this one.)

### 27.3 The layout: +layout.svelte

The layout is where FicHub actually lives. At 461 lines it's the app
shell: imports, stores, nav structure, and the SvelteKit `{children}`
slot that renders whatever page the router chose. The script starts by
importing the whole component ecosystem:

```svelte
<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import DownloadTab from '$lib/components/DownloadTab.svelte';
  import HomeDashboard from '$lib/components/HomeDashboard.svelte';
  import AuthBar from '$lib/components/AuthBar.svelte';
  import LocaleSelector from '$lib/components/LocaleSelector.svelte';
  import NotificationBell from '$lib/components/NotificationBell.svelte';
  import NavDropdown from '$lib/components/NavDropdown.svelte';
  import OfflineIndicator from '$lib/components/OfflineIndicator.svelte';
  import HelpModal from '$lib/components/HelpModal.svelte';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import DocLink from '$lib/components/DocLink.svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { exportUserData } from '$lib/api/social';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { i18n, t, initI18n, setLocale } from '$lib/i18n/index.svelte';

  let { children } = $props();
```

The `$lib` alias is SvelteKit's shortcut for `src/lib` — the shared
code directory. Every component in `$lib/components/` is a piece of
chrome: `DownloadTab` (the export form), `AuthBar` (login/signup),
`NotificationBell`, `NavDropdown`, `OfflineIndicator`, and more. The
`import { auth } from '$lib/stores/auth.svelte'` line pulls in the
global auth store (a Svelte 5 class-based store — we'll meet its
internals in Part 7). And `let { children } = $props()` is Svelte 5's
way of receiving the layout slot: `children` is the rendered page
content that this layout wraps.

The first piece of logic is the route-page decision, and it's a clever
bit of SPA plumbing:

```svelte
  // Determine if we're on a route page (needs {children}) or a tab page
  const routePages = ['/search', '/ask', '/leaderboard', '/bookmarks', '/fic/', '/notifications', '/follows', '/updates', '/badges', '/trending', '/stats', '/roadmap', '/tropes', '/blind-date', '/feed', '/quests', '/read/', '/work/', '/curator', '/requests', '/series', '/authors', '/lists', '/shelves', '/admin', '/admin/auto-tag', '/admin/comment-triage', '/admin/blacklist', '/admin/stats', '/curator/flags', '/work-proposals', '/recommendations', '/download'];
  let isRoutePage = $derived(routePages.some(p => $page.url.pathname.startsWith(p)));
```

FicHub's home screen is a *two-tab* UI: **Home** (dashboard) and
**Download**. Those tabs live in the layout. But the app also has
dozens of *route* pages — `/search`, `/read/...`, `/bookmarks`, the
admin panel — which are full pages, not tab content. The `routePages`
array is the registry that tells the layout which mode it's in: if the
current pathname starts with any route prefix, `isRoutePage` is true,
and the layout renders the page content (`{children}`); otherwise it
renders the tabbed Home/Download interface. The `$derived` rune
recomputes the boolean automatically whenever `$page.url.pathname`
changes — no manual event wiring. (And notice `startsWith` — that's why
`/fic/` matches `/fic/<url_id>` and `/read/` matches `/read/<url_id>`.
Prefix matching is what makes dynamic segments work in a hardcoded list.)

The nav structure is data-driven — arrays of links rendered with
`{#each}`:

```svelte
  const discoverLinks = [
    { href: '/trending', label: t('nav.trending'), icon: '🔥' },
    { href: '/leaderboard', label: t('nav.rankings'), icon: '🏆' },
    { href: '/tropes', label: t('nav.tropes'), icon: '🧭' },
    { href: '/ask', label: t('nav.askTheArchive'), icon: '🗣️' },
    { href: '/roadmap', label: t('nav.roadmap'), icon: '🗺️' },
    { href: '/blind-date', label: t('nav.blindDate'), icon: '🎲' },
    { href: '/requests', label: t('nav.requests'), icon: '🙋' },
    { href: '/work-proposals', label: t('nav.workProposals'), icon: '🗳️' },
    { href: '/recommendations', label: t('nav.recommendations'), icon: '★' },
  ];
  const libraryLinks = [
    { href: '/bookmarks', label: t('nav.bookmarks'), icon: '🔖' },
    { href: '/lists', label: t('nav.lists'), icon: '📚' },
    { href: '/shelves', label: t('nav.shelves'), icon: '🗄️' },
    { href: '/follows', label: t('nav.following'), icon: '📋' },
    { href: '/updates', label: t('nav.updates'), icon: '📬' },
    { href: '/feed', label: t('nav.feed'), icon: '📰' },
  ];
  const userLinks = [
    { href: '/stats', label: t('nav.profileStats'), icon: '📊' },
    { href: '/modlog', label: t('nav.modlog'), icon: '🛡️' },
    { href: '/badges', label: t('nav.badges'), icon: '🏅' },
    { href: '/quests', label: t('nav.quests'), icon: '🎯' },
  ];
```

Every label goes through `t('nav.xxx')` — the i18n translation function
we'll explore in Part 10. The pattern — *arrays of {href, label, icon}
objects rendered with `{#each}`* — is the cleanest way to build a nav:
adding a link is a one-line change, the mobile menu can reuse the same
arrays, and the structure is inspectable at a glance. (And the emoji
icons are a delightfully pragmatic choice: zero icon-font dependencies,
renders everywhere.)

The search box in the nav bar is a live link to everything we built in
Chapters 24–25:

```svelte
  function onSearchKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && searchQuery.trim()) {
      goto(`/search?q=${encodeURIComponent(searchQuery.trim())}`);
    }
  }
```

Press Enter in the nav search and the SPA navigates to `/search?q=<your
query>` — the search page's `onMount` (which we saw in the search page
snippet) reads `?q=` and fires `doSearch()`, which calls the
`search()` API helper, which builds the query string and hits
`/api/search`. One keystroke, and the whole Part 6 stack — parser,
builder, facets — runs. That's the through-line of this part: every
layer we studied connects to every other, and the layout is where the
connections are wired.

Auth and locale bootstrapping happen on mount, in a specific order with
a specific reason:

```svelte
  onMount(async () => {
    initI18n({ userLocale: auth.user?.locale ?? null });
    auth.init().then(() => {
      // User locale may only be known after auth resolves.
      if (!localStorage.getItem('fichub_locale') && auth.user?.locale) {
        setLocale(auth.user.locale);
      }
    });
  });

  // Keep <html lang="..."> in sync with the active locale.
  $effect(() => {
    document.documentElement.lang = i18n.locale;
  });
```

`initI18n` seeds translations immediately (with the *current* user
locale, if any), then `auth.init()` resolves the session asynchronously
— and only then, if the user never picked a locale explicitly, does the
layout adopt their server-side preference. The `$effect` keeps the
document's `lang` attribute in sync with the active locale — an
accessibility detail (screen readers use `lang`) that most apps forget.
That's the layout's quiet job: not just chrome, but the *correct
environment* for every page beneath it.

The mobile experience is handled by a hamburger toggle and a second
rendering of the same nav data:

```svelte
  // Mobile hamburger menu
  let mobileOpen = $state(false);
```

```svelte
  {#if mobileOpen}
    <nav class="mobile-menu" aria-label={t('nav.mobileAria')}>
      <span class="mobile-group-label">{t('nav.discover')}</span>
      {#each discoverLinks as link}
        <a class="dd-item" href={link.href} onclick={onMobileNavClick}>{link.icon} {link.label}</a>
      {/each}
      ...
```

One boolean (`mobileOpen`), one conditional block, and the same
`discoverLinks`/`libraryLinks`/`userLinks` arrays render the mobile
menu — with `onMobileNavClick` closing the drawer after navigation.
Data-driven nav pays off twice: desktop dropdowns and mobile menu from
the same source of truth. (And note the accessibility: `aria-label`s on
the nav, `aria-expanded` on the hamburger button — a screen-reader user
gets the same structure a sighted user gets.)

The signed-in vs. signed-out split is a single `{#if auth.isLoggedIn}`
at the auth area:

```svelte
    <div class="auth-area">
      {#if auth.isLoggedIn}
        <NavDropdown label={auth.username ?? t('nav.account')} icon="👤" align="right">
          {#each userLinks as link}
            <a class="dd-item" href={link.href}>{link.icon} {link.label}</a>
          {/each}
          <button class="dd-item" type="button" onclick={handleExportData} disabled={exporting}>
            📦 {exporting ? t('nav.exporting') : t('nav.exportMyData')}
          </button>
          {#if exportError}
            <span class="dd-row dd-error">{exportError}</span>
          {/if}
          <span class="dd-row"><NotificationBell /></span>
          {#if isCurator}
            <a class="dd-item" href="/admin">🛠️ {t('nav.adminDashboard')}</a>
            <a class="dd-item" href="/curator">🛡️ {t('nav.curatorHub')}</a>
            <a class="dd-item" href="/curator/flags">🚩 {t('nav.flagQueue')}</a>
          {/if}
          ...
      {:else}
        <AuthBar />
      {/if}
    </div>
```

`auth.isLoggedIn` is reactive — the moment the store flips (login or
logout), the whole nav re-renders without a page reload. Signed-in
users get the account dropdown (user links, the one-click data export,
the notification bell, and — for role ≥ 10 — the admin and curator
entries); signed-out visitors get `AuthBar`, the login/signup
component. The `isCurator` check is a `$derived` from the user's role:

```svelte
  const isCurator = $derived((auth.user?.role ?? 0) >= 10);
```

That role tier — 10 = curator — is the exact role system we'll build in
Part 7. The frontend already knows how to hide and show for it.

The one-click data export is a nice end-to-end example of the layout's
API usage — it's the "download everything about me" button:

```svelte
  async function handleExportData() {
    if (exporting) return;
    exporting = true;
    exportError = '';
    try {
      const res = await exportUserData();
      const blob = await res.blob();
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      // The server names the archive fichub-user-data-<username>.zip; match
      // it on the client so the download lands with a clear filename.
      a.href = url;
      a.download = 'fichub-user-data.zip';
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      exportError = 'Export failed. Please try again.';
      console.error('User data export failed', e);
    } finally {
      exporting = false;
    }
  }
```

The classic browser-download dance: fetch the blob, create an object
URL, synthesize an `<a download>` click, revoke the URL. The guard
`if (exporting) return;` prevents double-clicks, the `try/catch/finally`
keeps the button from getting stuck in the loading state, and the
`a.download` attribute gives the file a friendly name. The comment
explains *why* the client hardcodes the filename (the server names it
`fichub-user-data-<username>.zip`; matching it client-side keeps the
download recognizable) — a tiny cross-stack contract, documented on the
client side.

Finally, the shell:

```svelte
<div class="app">
  <header class="topbar">
    <div class="brand">
      <a href="/" class="brand-link" aria-label="FicHub home">
        <span class="logo">📚</span>
        <span class="title">FicHub</span>
      </a>
    </div>

    <nav class="main-nav" aria-label={t('nav.primaryAria')}>
      <NavDropdown label={t('nav.discover')} icon="🧭">
        {#each discoverLinks as link}
          <a class="dd-item" href={link.href}>{link.icon} {link.label}</a>
        {/each}
      </NavDropdown>

      {#if auth.isLoggedIn}
        <NavDropdown label={t('nav.library')} icon="📚">
          {#each libraryLinks as link}
            <a class="dd-item" href={link.href}>{link.icon} {link.label}</a>
          {/each}
        </NavDropdown>
      {/if}
    </nav>

    <div class="search-area">
      <input
        class="nav-search"
        type="search"
        placeholder={t('nav.searchPlaceholder')}
        bind:value={searchQuery}
        onkeydown={onSearchKeydown}
        aria-label={t('nav.searchAria')}
      />
      <a class="adv-link" href="/search?advanced=1" title={t('nav.advancedSearch')}>⚙</a>
    </div>

    <a class="btn cta-download" href="/download">
      <span aria-hidden="true">⬇</span> {t('nav.download')}
    </a>
    ...
```

Brand, nav, search, download CTA, auth — the classic SaaS top bar, with
SvelteKit niceties: `bind:value` for two-way input binding (the search
input writes `searchQuery` on every keystroke), `{@html}`-free markup,
and the `⚙` advanced-search link that we saw the search page read as
`?advanced=1` back in Chapter 24's page snippet. And the `<svelte:head>`
block declares the PWA manifest and the Atom feed link — the whole app
shell announces its capabilities (offline via the manifest, syndication
via feed.xml) right at the top:

```svelte
<svelte:head>
  <link rel="manifest" href="/manifest.webmanifest" />
  <meta name="theme-color" content="#171a23" />
  <link rel="alternate" type="application/atom+xml" title="FicHub — New Arrivals" href="/feed.xml" />
</svelte:head>
```

`<svelte:head>` is Svelte's way of injecting into the document head
from any component — the manifest (Part 12's PWA story), the
theme-color for mobile browser chrome, and the Atom feed (Part 8's RSS
chapter) are all declared where they conceptually belong: the shell.

💡 **Key Concept — the layout is the app, pages are content.** The
single most important idea in this chapter is the division of labor
between `+layout.svelte` (chrome, nav, auth, i18n, mode-switching) and
`+page.svelte` (route-specific content). FicHub pushes this to an
extreme — the root page is literally empty, and the layout owns the
default view. This is the SPA architecture in miniature: state and
chrome at the top, content injected below, and the router deciding what
fits where. When you design your own app shells, ask *"what's true for
every page?"* — that's what belongs in the layout, and nothing else.

🧪 **Try It Yourself — trace a search keystroke end to end.** This is
the capstone exercise of the whole part. Put all five chapters
together:

1. Type `"enemies to lovers" AND (fluff OR humor) -angst` in the nav
   search box and press Enter. `onSearchKeydown` navigates to
   `/search?q=...`.
2. The search page's `onMount` reads `?q=`, sets `filters.q`, and calls
   `doSearch()` → `search(filters)` → `buildSearchQuery` → `fetch('/api/search?...')`.
3. Rust's `search_handler` runs `parse_query` (Chapter 24), extracts the
   tsquery, fielded terms, and exclusions, and builds the SQL with
   `SearchQueryBuilder` (Chapters 24–25).
4. PostgreSQL evaluates the `@@ to_tsquery` match plus the facet
   queries, and the JSON envelope comes back.
5. The search page renders results with snippets and facets; clicking a
   result's "Read" link goes to `/read/<url_id>`, whose `+page.svelte`
   calls `loadReaderData` → `/api/reader/<url_id>` (Chapter 26), splits
   the HTML bundle, and starts you reading.

That's the full stack — TypeScript to Rust to PostgreSQL and back —
built entirely from the pieces you now own.

⚠️ **Watch Out — Svelte 5 runes are not magic; they're declarations.**
When you see `let x = $state(0)`, the `$state` is a *compiler hint*,
not a runtime function — Svelte rewrites the assignment `x = 1` into a
reactive update under the hood. The corollaries: don't destructure
`$state` values into plain variables (the reactivity is tied to the
variable itself), don't use `$state` inside plain (non-rune) `.ts`
files (it only works in `.svelte` files and rune-enabled `.svelte.ts`
modules), and treat `$derived` as read-only — you can't assign to it.
The rule of thumb: **if a value changes over time and the UI must
react, it's `$state`; if it's computed from other reactive values, it's
`$derived`.** FicHub's code follows this consistently, and your
reading will go smoother if you do too.

That's the SPA shell: a typed API client, an empty home page that
delegates to the layout, and a layout that composes auth, i18n, nav,
search, and the route/tab mode-switch into one reactive app. It's the
frontend half of everything in Part 6 — the thin, friendly face over
the meta endpoint, the boolean search engine, and the reader.

---

That's Part 6 done — the API layer is now yours. You know how the meta
endpoint answers "what is this URL?" without touching the export
pipeline; how the boolean parser turns user text into a typed AST and
then into a PostgreSQL tsquery with AND/OR/NOT, phrases, fields, and
parens; why "Dark Harry" needs scrape-time scores and a correlated
subquery instead of a naive tag intersection; how the reader reuses the
HTML cache as its database and splits one bundle into chapters that
remember your place; and how the SvelteKit shell ties it all together
through one thin, typed API client. The search box, the metadata card,
and the reader page are no longer black boxes — they're code you could
reproduce from memory.

Next, in **Part 7 — Authentication, Users & Social**, we add the people:
JWTs and the `AuthUser` extractor, the role tiers (0/1/5/10) that the
layout's `isCurator` check already anticipates, bookmarks and ratings,
comments and moderation, and the follows-and-updates feed that turns a
download tool into a community. See you there.




