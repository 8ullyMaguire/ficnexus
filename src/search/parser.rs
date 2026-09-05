/// Boolean query parser for the FicNexus search system.
///
/// Supports:
/// - Simple words: `coffee`
/// - Quoted phrases: `"enemies to lovers"`
/// - AND operator: `coffee AND angst`
/// - OR operator: `coffee OR tea`
/// - NOT operator: `NOT major character death` or `-major character death`
/// - Exclusion prefix: `-angst`
/// - Fielded search: `title:harry` or `author:jk` or `fandom:harry`
/// - Parenthesized grouping: `(fluff OR humor) AND -angst`
/// - Mixed: `"enemies to lovers" AND (fluff OR humor) -angst`
///
/// Search v2 (structured [`FieldQuery`] expressions, per-token — no
/// backreferences, so `regex-lite`'s limitations are a non-issue):
/// - Role modifier: `@char:Harry` / `@fandom:Marvel` / `primary_fandom:X`
///   (requires a main/primary tag via `fic_tags.role_confidence`).
/// - Contains/fuzzy: `title~harry`, `description~gay`.
/// - Wildcards: `attr:cozy*` (prefix), `attr:*burn` (contains ending).
/// - Ship polarity: `romship:A/B` (romantic), `platship:A&B` (platonic).
/// - Counters & ranges: `words:>50k`, `words:10k-100k`, `words:5000`,
///   `kudos:>=100`, `chapters:5-50`, `published:2018-2021`, `published:2020`,
///   `updated:>2019`, `chapters:=7`.
/// - Crossover: `crossover:1`.
/// - `with` acts as an implicit AND; `NOT with attr:X` negates X.
///   e.g. `char:Harry with attr:"Slow Burn" NOT with attr:Dark`.
use std::fmt;

/// A parsed query leaf term.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum QueryTerm {
    /// A simple word (will be stemmed for tsquery)
    Word(String),
    /// A quoted phrase: "coffee shop" → distance-based tsquery
    Phrase(String),
    /// An excluded term: -angst or -"major character death"
    Excluded(String),
    /// A fielded search: title:something → applies as a separate WHERE clause
    Fielded { field: String, value: String },
    /// A v2 structured field expression: `@field:val`, `field~val`,
    /// `field:val*`, ranges, comparisons, romship/platship, crossover, etc.
    FieldQuery(FieldQuery),
}

/// A structured v2 field expression.
///
/// Represented as a typed [`Field`] (knows whether it is a text column, a tag,
/// a numeric counter, or a date) plus an operator and, for tag fields, an
/// optional role (`@`) / polarity. Parsed per-token — the tokenizer never
/// needs backreferences, so `regex-lite`'s lack of backreference support is a
/// non-issue (we hand-roll the small scanner instead).
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FieldQuery {
    pub field: Field,
    /// Raw value (tag name, text fragment, number, year). Semantically None
    /// only for a bare flag with no payload.
    pub value: Option<String>,
    /// How to apply the value.
    pub op: FieldOp,
    /// `@` role modifier: require the matched tag to be main/primary.
    pub role: bool,
    /// True when the expression sits under a `NOT` / `-` (e.g. `-char:X`,
    /// `NOT with attr:Y`) — the builder inverts the clause.
    pub negated: bool,
}

/// The typed set of searchable fields. Each variant knows how it resolves in
/// SQL (see [`Field::tag_type_id`], [`Field::is_count`], [`Field::is_tag`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Field {
    Title,
    Author,
    Description,
    /// Fandom tag, type_id 1 (any role).
    Fandom,
    /// Fandom tag that is the fic's primary/main fandom (`@fandom`).
    PrimaryFandom,
    /// Character tag, type_id 2.
    Char,
    /// Relationship tag, type_id 3, any polarity.
    Ship,
    /// Relationship tag, type_id 3, romantic polarity (rel_polarity = 1).
    Romship,
    /// Relationship tag, type_id 3, platonic polarity (rel_polarity = 2).
    Platship,
    /// Freeform/attribute tag, type_id 4.
    Attr,
    /// Archive warning tag, type_id 5.
    Warning,
    /// Category tag, type_id 6.
    Category,
    /// Rating tag, type_id 7.
    Rating,
    /// `fic_info.status` column (complete / in-progress / etc).
    Status,
    /// `fic_info.words` counter.
    Words,
    /// `fic_info.chapters` counter.
    Chapters,
    /// `works.kudos_count` counter.
    Kudos,
    /// `works.comments_count` counter.
    Comments,
    /// `works.bookmarks_count` counter.
    Bookmarks,
    /// `works.hit_count` counter.
    Hits,
    /// Publish year range over `fic_info.fic_created`.
    Published,
    /// Update year range over `fic_info.fic_updated`.
    Updated,
    /// `works.language_code`.
    Language,
    /// `works.beta_status`.
    Beta,
    /// `fic_info.source`.
    Source,
    /// Crossover = the fic carries > 1 distinct fandom tag.
    Crossover,
}

/// How a field value is applied.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum FieldOp {
    /// `field:value` / `field~value` — contains (ILIKE `%v%`), or a tag-name
    /// match for tag fields.
    Contains,
    /// `field:val*` — prefix wildcard (ILIKE `val%`).
    Prefix,
    /// `field:*val` — contains ending match (ILIKE `%val`).
    ContainsWide,
    /// Numeric / date range or comparison (`field:>N`, `field:a-b`, `50k`…).
    Range(RangeExpr),
}

/// A numeric / date interval.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum RangeExpr {
    Eq(i64),
    Ge(i64),
    Gt(i64),
    Le(i64),
    Lt(i64),
    Between(i64, i64),
}

impl Field {
    /// Human-readable field name.
    pub fn as_str(&self) -> &'static str {
        match self {
            Field::Title => "title",
            Field::Author => "author",
            Field::Description => "description",
            Field::Fandom => "fandom",
            Field::PrimaryFandom => "primary_fandom",
            Field::Char => "char",
            Field::Ship => "ship",
            Field::Romship => "romship",
            Field::Platship => "platship",
            Field::Attr => "attr",
            Field::Warning => "warning",
            Field::Category => "category",
            Field::Rating => "rating",
            Field::Status => "status",
            Field::Words => "words",
            Field::Chapters => "chapters",
            Field::Kudos => "kudos",
            Field::Comments => "comments",
            Field::Bookmarks => "bookmarks",
            Field::Hits => "hits",
            Field::Published => "published",
            Field::Updated => "updated",
            Field::Language => "language",
            Field::Beta => "beta",
            Field::Source => "source",
            Field::Crossover => "crossover",
        }
    }

    /// For tag-backed fields, the `tags.tag_type_id` (1 fandom, 2 character,
    /// 3 relationship, 4 freeform/attr, 5 warning, 6 category, 7 other).
    pub fn tag_type_id(&self) -> Option<i16> {
        match self {
            Field::Fandom | Field::PrimaryFandom => Some(1),
            Field::Char => Some(2),
            Field::Ship | Field::Romship | Field::Platship => Some(3),
            Field::Attr => Some(4),
            Field::Warning => Some(5),
            Field::Category => Some(6),
            Field::Rating => Some(7),
            _ => None,
        }
    }

    /// Relationship polarity required by romship (1) / platship (2);
    /// None for fields that don't pin a polarity.
    pub fn rel_polarity(&self) -> Option<i16> {
        match self {
            Field::Romship => Some(1),
            Field::Platship => Some(2),
            _ => None,
        }
    }

    /// Numeric counter columns (compare against a bound).
    pub fn is_count(&self) -> bool {
        matches!(
            self,
            Field::Words
                | Field::Chapters
                | Field::Kudos
                | Field::Comments
                | Field::Bookmarks
                | Field::Hits
        )
    }

    /// Year-range fields.
    pub fn is_date(&self) -> bool {
        matches!(self, Field::Published | Field::Updated)
    }

    /// Whether the field lives on a plain text/value column (rather than a
    /// tag or a counter).
    pub fn is_text(&self) -> bool {
        matches!(
            self,
            Field::Title
                | Field::Author
                | Field::Description
                | Field::Status
                | Field::Language
                | Field::Beta
                | Field::Source
        )
    }

    /// Whether the field is resolved through a `tags`/`fic_tags` filter.
    pub fn is_tag(&self) -> bool {
        self.tag_type_id().is_some()
    }

    /// Whether the filter needs a `works` join (works columns are only joined
    /// on demand to keep the default search surface unchanged).
    pub fn needs_works(&self) -> bool {
        matches!(
            self,
            Field::Kudos
                | Field::Comments
                | Field::Bookmarks
                | Field::Hits
                | Field::Language
                | Field::Beta
        )
    }
}

impl fmt::Display for QueryTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryTerm::Word(w) => write!(f, "{}", w),
            QueryTerm::Phrase(p) => write!(f, "\"{}\"", p),
            QueryTerm::Excluded(e) => write!(f, "-{}", e),
            QueryTerm::Fielded { field, value } => write!(f, "{}:{}", field, value),
            QueryTerm::FieldQuery(q) => {
                let role = if q.role { "@" } else { "" };
                match &q.op {
                    FieldOp::Contains => write!(
                        f,
                        "{}{}:{}",
                        role,
                        q.field.as_str(),
                        q.value.as_deref().unwrap_or("")
                    ),
                    FieldOp::Prefix => write!(
                        f,
                        "{}{}:{}*",
                        role,
                        q.field.as_str(),
                        q.value.as_deref().unwrap_or("")
                    ),
                    FieldOp::ContainsWide => write!(
                        f,
                        "{}{}:*{}",
                        role,
                        q.field.as_str(),
                        q.value.as_deref().unwrap_or("")
                    ),
                    FieldOp::Range(r) => write!(f, "{}{}:{:?}", role, q.field.as_str(), r),
                }
            }
        }
    }
}

/// A boolean expression tree.
#[derive(Debug, Clone, serde::Serialize)]
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

// ─── Tokenizer ───────────────────────────────────────────────────────────────

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
                    if ch == ' '
                        || ch == '\t'
                        || ch == '\n'
                        || ch == '\r'
                        || ch == '('
                        || ch == ')'
                        || ch == '"'
                    {
                        break;
                    }
                    word.push(ch);
                    chars.next();
                }
                // Check for operators (case-insensitive)
                let upper = word.to_uppercase();
                match upper.as_str() {
                    "AND" | "WITH" => tokens.push(Token::OpAnd),
                    "OR" => tokens.push(Token::OpOr),
                    "NOT" => tokens.push(Token::OpNot),
                    _ => tokens.push(Token::Word(word)),
                }
            }
        }
    }

    tokens
}

// ─── Parser (recursive descent, Pratt-style for precedence) ────────────────

/// Parse a raw query string into a BooleanExpression tree.
///
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
        }
    }

    if terms.len() == 1 {
        terms.into_iter().next().unwrap()
    } else {
        BooleanExpression::And(terms)
    }
}

/// Parse NOT expressions (highest precedence — prefix unary)
fn parse_not_expr(tokens: &[Token], pos: &mut usize) -> BooleanExpression {
    if *pos >= tokens.len() {
        return BooleanExpression::And(vec![]);
    }

    match &tokens[*pos] {
        Token::OpNot => {
            *pos += 1;
            // `NOT with X` / `NOT AND X` — skip the `with` separator so the
            // negation binds to the following primary (`not with attr:C` =>
            // NOT(attr:C)), not to an empty group.
            if *pos < tokens.len() && matches!(tokens[*pos], Token::OpAnd) {
                *pos += 1;
            }
            let inner = parse_primary(tokens, pos);
            BooleanExpression::Not(Box::new(inner))
        }
        // A lone '-' word glues to the NEXT token as an exclusion.
        // Handles `-angst` and `-"major character death"`.
        Token::Word(w) if w == "-" => {
            *pos += 1;
            let inner = parse_primary(tokens, pos);
            match inner {
                BooleanExpression::Term(QueryTerm::Word(w)) => BooleanExpression::Not(Box::new(
                    BooleanExpression::Term(QueryTerm::Excluded(w)),
                )),
                BooleanExpression::Term(QueryTerm::Phrase(p)) => BooleanExpression::Not(Box::new(
                    BooleanExpression::Term(QueryTerm::Excluded(p)),
                )),
                other => BooleanExpression::Not(Box::new(other)),
            }
        }
        _ => parse_primary(tokens, pos),
    }
}

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

            // ── v2 field expressions (everything the v1 block doesn't capture):
            // `@char:Harry`, `romship:A/B`, `words:>50k`, `words:10k-100k`,
            // `title~phrase`, `attr:cozy*`, `crossover:1`, `primary_fandom:X`…
            // The leading `@` role modifier guarantees this is a field query
            // even for names the v1 block would otherwise absorb as plain words.
            if w.starts_with('@') || safe_field_candidate(w) {
                if let Some(mut fq) = parse_field_word(w) {
                    // field:"quoted phrase", or tag/text field with an empty
                    // payload followed by a Phrase token.
                    if fq.value.as_deref().unwrap_or("").is_empty()
                        && !fq.field.is_count()
                        && !fq.field.is_date()
                    {
                        if let Some(Token::Phrase(p)) = tokens.get(*pos) {
                            *pos += 1;
                            fq.value = Some(p.clone());
                            fq.op = FieldOp::Contains;
                        }
                    }
                    return BooleanExpression::Term(QueryTerm::FieldQuery(fq));
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
            // Check for exclusion prefix on phrase (handled differently - phrase is already quoted)
            // Actually, -"phrase" would be tokenized as '-' then Phrase("phrase")
            // That case is handled by Word('-') followed by Phrase
            BooleanExpression::Term(QueryTerm::Phrase(p.clone()))
        }
        Token::ParenClose => {
            // Shouldn't happen normally, but handle gracefully
            *pos += 1;
            BooleanExpression::And(vec![])
        }
        _ => {
            *pos += 1;
            BooleanExpression::And(vec![])
        }
    }
}

// ─── PostgreSQL tsquery conversion ────────────────────────────────────────────

/// Cheap gate before attempting a full v2 field parse: name the leading field
/// token, so we only spend effort on words that could be a field (`name:`/`~`
/// with a payload, or a `@` role prefix). The v1 block runs first, so fields
/// it owns (title/author/fandom/character/relationship/source) never reach
/// this path — they stay `Fielded` for backward compatibility.
fn safe_field_candidate(word: &str) -> bool {
    let rest = word.strip_prefix('@').unwrap_or(word);
    let mut field_len = 0;
    for c in rest.chars() {
        if c.is_ascii_alphabetic() || c == '_' {
            field_len += c.len_utf8();
        } else {
            break;
        }
    }
    if field_len == 0 {
        return false;
    }
    let field_name = &rest[..field_len.min(rest.len())];
    if normalize_field(field_name).is_none() {
        return false;
    }
    let after = &rest[field_len..];
    // A delimiter with a socket — allow an empty payload so `attr:"phrase"`
    // (empty value + following Phrase token) is still recognised as a field
    // expression; the primary parser fills the value from the quoted token.
    // (Bare field names — e.g. the word "crossover" — are NOT field queries,
    // so plain-text searches for such words keep working.)
    after.len() >= 1 && (after.starts_with(':') || after.starts_with('~'))
}

/// Map a leading field name to its typed [`Field`], including aliases.
fn normalize_field(name: &str) -> Option<Field> {
    match name.to_ascii_lowercase().as_str() {
        "title" => Some(Field::Title),
        "author" => Some(Field::Author),
        "description" | "desc" => Some(Field::Description),
        "fandom" | "fandoms" | "universe" => Some(Field::Fandom),
        "primary_fandom" | "primaryfandom" | "main_fandom" | "mainfandom" | "primary" => {
            Some(Field::PrimaryFandom)
        }
        "char" | "character" | "characters" => Some(Field::Char),
        "ship" | "relationship" | "relationships" => Some(Field::Ship),
        "romship" | "romantic" | "rom" => Some(Field::Romship),
        "platship" | "platonic" | "plat" => Some(Field::Platship),
        "attr" | "attribute" | "attributes" | "freeform" | "freeforms" | "tag" | "tags" => {
            Some(Field::Attr)
        }
        "warning" | "warnings" | "archive_warning" => Some(Field::Warning),
        "category" | "categories" => Some(Field::Category),
        "rating" | "ratings" => Some(Field::Rating),
        "status" => Some(Field::Status),
        "words" | "wordcount" => Some(Field::Words),
        "chapters" => Some(Field::Chapters),
        "kudos" => Some(Field::Kudos),
        "comments" | "comment" => Some(Field::Comments),
        "bookmarks" | "bookmark" => Some(Field::Bookmarks),
        "hits" | "hit" => Some(Field::Hits),
        "published" | "pub" => Some(Field::Published),
        "updated" | "last_updated" => Some(Field::Updated),
        "language" | "lang" => Some(Field::Language),
        "beta" | "beta_status" => Some(Field::Beta),
        "source" => Some(Field::Source),
        "crossover" => Some(Field::Crossover),
        _ => None,
    }
}

/// Parse a single v2 field token into a structured [`FieldQuery`].
/// Returns `None` when the token is not a recognised field expression, so the
/// caller can fall back to treating it as a plain word.
fn parse_field_word(word: &str) -> Option<FieldQuery> {
    let rest = word.strip_prefix('@').unwrap_or(word);
    let role = word.starts_with('@');

    let mut field_len = 0;
    for c in rest.chars() {
        if c.is_ascii_alphabetic() || c == '_' {
            field_len += c.len_utf8();
        } else {
            break;
        }
    }
    if field_len == 0 {
        return None;
    }
    if field_len > rest.len() {
        field_len = rest.len();
    }
    let field = normalize_field(&rest[..field_len])?;
    let after = &rest[field_len..];
    let delim = after.chars().next()?;
    if delim != ':' && delim != '~' {
        return None;
    }
    let raw = &after[delim.len_utf8()..];
    parse_field_value(field, delim, raw, role)
}

/// Given a field, its delimiter (`:`/`~`) and the raw value, build a
/// `FieldQuery` with the right [`FieldOp`].
fn parse_field_value(field: Field, delim: char, raw: &str, role: bool) -> Option<FieldQuery> {
    // A `@` role modifier only makes sense on tag-backed fields; the
    // `primary_fandom` field always implies the primary/main role.
    let role = (role && field.is_tag()) || matches!(field, Field::PrimaryFandom);

    if field == Field::Crossover {
        // crossover:1 / crossover:true / crossover — presence means "on".
        // value("1") => fandom count > 1; value("0") => fandom count <= 1
        // (i.e. a single-fandom fic). We carry the toggle in `value`.
        let on = match raw.trim().to_ascii_lowercase().as_str() {
            "" | "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => true,
        };
        return Some(FieldQuery {
            field,
            value: Some(if on { "1".to_string() } else { "0".to_string() }),
            op: FieldOp::Contains,
            role: false,
            negated: false,
        });
    }

    if field.is_count() || field.is_date() {
        let expr = parse_numeric(raw, field.is_date())?;
        return Some(FieldQuery {
            field,
            value: Some(raw.to_string()),
            op: FieldOp::Range(expr),
            role,
            negated: false,
        });
    }

    // Text / status / tag fields.
    let op = match delim {
        '~' => FieldOp::Contains,
        ':' => {
            let v = raw;
            if v.ends_with('*') && v.len() > 1 {
                FieldOp::Prefix
            } else if v.starts_with('*') && v.len() > 1 {
                FieldOp::ContainsWide
            } else {
                FieldOp::Contains
            }
        }
        _ => FieldOp::Contains,
    };
    let clean = raw.trim_matches('*').to_string();
    Some(FieldQuery {
        field,
        value: Some(clean),
        op,
        role,
        negated: false,
    })
}

/// Parse a numeric interval / comparison, with `k`/`m` shorthand (50k = 50000).
/// `is_date` is true for the year-range fields: a bare 4-digit payload pins the
/// exact year, whereas a bare value on a counter means "at least N".
fn parse_numeric(raw: &str, is_date: bool) -> Option<RangeExpr> {
    let s = raw.trim();
    // `>=`, `<=` must be tested before the single-char `>`/`<`.
    if let Some(v) = s.strip_prefix(">=") {
        return Some(RangeExpr::Ge(parse_num(v)?));
    }
    if let Some(v) = s.strip_prefix("<=") {
        return Some(RangeExpr::Le(parse_num(v)?));
    }
    // Exact form: `field:=N`.
    if let Some(v) = s.strip_prefix('=') {
        return Some(RangeExpr::Eq(parse_num(v)?));
    }
    if let Some(v) = s.strip_prefix('>') {
        return Some(RangeExpr::Gt(parse_num(v)?));
    }
    if let Some(v) = s.strip_prefix('<') {
        return Some(RangeExpr::Lt(parse_num(v)?));
    }
    // `a-b` range.
    if let Some(dash) = find_range_dash(s) {
        let a = parse_num(&s[..dash])?;
        let b = parse_num(&s[dash + 1..])?;
        return Some(RangeExpr::Between(a, b));
    }
    // Bare number: date fields pin the exact year; counters mean "at least".
    let v = parse_num(s)?;
    if is_date && s.len() == 4 && s.bytes().all(|b| b.is_ascii_digit()) {
        Some(RangeExpr::Between(v, v))
    } else {
        Some(RangeExpr::Ge(v))
    }
}

/// Find an inner `-` that separates two numeric operands (for `a-b` ranges),
/// ignoring a leading minus and `>=`/`<=`-style comparisons already handled.
fn find_range_dash(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'-' && i == 0 {
            continue;
        }
        if bytes[i] == b'-' {
            let before = &s[..i];
            let after = &s[i + 1..];
            let numeral = |t: &str| -> bool {
                !t.is_empty()
                    && t.bytes()
                        .all(|b| b.is_ascii_digit() || matches!(b, b'k' | b'K' | b'm' | b'M'))
            };
            if numeral(before) && numeral(after) {
                return Some(i);
            }
        }
    }
    None
}

/// Parse an integer with an optional `k` (×1000) / `m` (×1,000,000) suffix.
fn parse_num(s: &str) -> Option<i64> {
    let t = s.trim().to_ascii_lowercase();
    if t.is_empty() {
        return None;
    }
    let (num, mult) = if let Some(n) = t.strip_suffix('k') {
        (n, 1000)
    } else if let Some(n) = t.strip_suffix('m') {
        (n, 1_000_000)
    } else {
        (t.as_str(), 1)
    };
    let n: i64 = num.trim().parse().ok()?;
    Some(n * mult)
}

/// Convert a parsed BooleanExpression to a PostgreSQL tsquery string.
///
/// e.g. `"enemies to lovers" AND angst -fluff` → `'enimies' <-> 'to' <-> 'lover' & 'angst' & !('fluff':*)`
pub fn expr_to_tsquery(expr: &BooleanExpression) -> String {
    match expr {
        BooleanExpression::And(terms) => {
            if terms.is_empty() {
                return String::new();
            }
            let parts: Vec<String> = terms
                .iter()
                .map(expr_to_tsquery)
                .filter(|p| !p.is_empty())
                .collect();
            parts.join(" & ")
        }
        BooleanExpression::Or(terms) => {
            if terms.is_empty() {
                return String::new();
            }
            let parts: Vec<String> = terms
                .iter()
                .map(expr_to_tsquery)
                .filter(|p| !p.is_empty())
                .collect();
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

/// Strip characters that are meaningful to PostgreSQL tsquery syntax but not
/// intended as part of a plain word.
///
/// Prevents e.g. `Holtzmann:` (a title token with a trailing colon) from
/// becoming `holtzmann::*` — a tsquery syntax error that made
/// `/api/search?q=Jillian+Holtzmann:+Ace+Attorney` return a database error.
fn sanitize_tsquery_word(w: &str) -> String {
    w.chars()
        .filter(|c| {
            !matches!(
                c,
                ':' | '\'' | '"' | '\\' | '(' | ')' | '&' | '|' | '!' | '<' | '>' | ',' | ';'
            )
        })
        .collect::<String>()
        .to_lowercase()
}

fn term_to_tsquery(term: &QueryTerm) -> String {
    match term {
        QueryTerm::Word(w) => {
            // tsquery format with stemming and prefix matching
            // Use lowercase for consistency with tsvector
            let clean = sanitize_tsquery_word(w);
            if clean.is_empty() {
                return String::new();
            }
            format!("{}:*", clean)
        }
        QueryTerm::Phrase(p) => {
            // For phrases, use the <-> (followed by) operator between words
            let words: Vec<&str> = p.split_whitespace().collect();
            if words.is_empty() {
                return String::new();
            }
            let parts: Vec<String> = words
                .iter()
                .map(|w| sanitize_tsquery_word(w))
                .filter(|w| !w.is_empty())
                .map(|w| format!("{}:*", w))
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
                    .map(|w| sanitize_tsquery_word(w))
                    .filter(|w| !w.is_empty())
                    .map(|w| format!("{}:*", w))
                    .collect();
                parts.join(" <-> ")
            } else {
                let clean = sanitize_tsquery_word(e);
                if clean.is_empty() {
                    return String::new();
                }
                format!("{}:*", clean)
            }
        }
        QueryTerm::Fielded { value, .. } => {
            // Fielded search value becomes part of the text tsquery
            // Field-specific filtering is handled separately as a WHERE clause
            let clean = sanitize_tsquery_word(value);
            if clean.is_empty() {
                return String::new();
            }
            format!("{}:*", clean)
        }
        QueryTerm::FieldQuery(_) => {
            // v2 field expressions are applied as dedicated WHERE clauses,
            // never as part of the general text search.
            String::new()
        }
    }
}

/// Extract the structured v2 field expressions from an expression tree,
/// carrying their polarity (a `NOT`/`-` inverts a term — the builder uses
/// `negated` to negate the clause rather than drop it).
pub fn extract_field_queries(expr: &BooleanExpression) -> Vec<FieldQuery> {
    let mut queries = Vec::new();
    extract_field_query_recurse(expr, &mut queries, false);
    queries
}

fn extract_field_query_recurse(
    expr: &BooleanExpression,
    queries: &mut Vec<FieldQuery>,
    negated: bool,
) {
    match expr {
        BooleanExpression::And(terms) | BooleanExpression::Or(terms) => {
            for t in terms {
                extract_field_query_recurse(t, queries, negated);
            }
        }
        BooleanExpression::Not(inner) => {
            // Flip the polarity for everything nested under NOT.
            extract_field_query_recurse(inner, queries, !negated);
        }
        BooleanExpression::Term(QueryTerm::FieldQuery(q)) => {
            let mut q = q.clone();
            q.negated = q.negated != negated;
            queries.push(q);
        }
        _ => {}
    }
}

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
        BooleanExpression::Or(terms) => {
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
                BooleanExpression::Or(cleaned)
            }
        }
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
        BooleanExpression::Term(QueryTerm::FieldQuery(_)) => {
            // v2 field expressions are also removed from the general text
            // search — they become dedicated WHERE clauses instead.
            BooleanExpression::And(vec![])
        }
        other => other.clone(),
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_word() {
        let expr = parse_query("coffee");
        assert!(matches!(expr, BooleanExpression::Term(QueryTerm::Word(_))));
        let ts = expr_to_tsquery(&expr);
        assert_eq!(ts, "coffee:*");
    }

    #[test]
    fn test_parse_two_words_implicit_and() {
        let expr = parse_query("coffee shop");
        let ts = expr_to_tsquery(&expr);
        assert_eq!(ts, "coffee:* & shop:*");
    }

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
    fn test_parse_exclusion_not_keyword() {
        let expr = parse_query("NOT angst");
        let ts = expr_to_tsquery(&expr);
        assert_eq!(ts, "!(angst:*)");
    }

    #[test]
    fn test_parse_and_operator() {
        let expr = parse_query("coffee AND angst");
        let ts = expr_to_tsquery(&expr);
        assert_eq!(ts, "coffee:* & angst:*");
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
    fn test_parse_fielded_search_author() {
        let expr = parse_query("author:jk");
        let fields = extract_fielded_terms(&expr);
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0], ("author".to_string(), "jk".to_string()));
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
    fn test_parse_complex_expression() {
        let expr = parse_query("\"enemies to lovers\" AND (fluff OR humor) -angst");
        let ts = expr_to_tsquery(&expr);
        assert!(ts.contains("<->"));
        assert!(ts.contains("&"));
        assert!(ts.contains("|"));
        assert!(ts.contains("!"));
    }

    #[test]
    fn test_extract_excluded_terms() {
        let expr = parse_query("coffee -angst NOT fluff");
        let excluded = extract_excluded_terms(&expr);
        assert!(excluded.contains(&"angst".to_string()));
        assert!(excluded.contains(&"fluff".to_string()));
    }

    #[test]
    fn test_parse_not_keyword_exclusion() {
        let expr = parse_query("coffee NOT angst");
        let ts = expr_to_tsquery(&expr);
        assert_eq!(ts, "coffee:* & !(angst:*)");
    }

    #[test]
    fn test_parse_complex_nested_parentheses() {
        let expr =
            parse_query("(fluff OR humor OR angst) AND (\"slow burn\" OR \"enemies to lovers\")");
        let ts = expr_to_tsquery(&expr);
        assert!(ts.contains("fluff:* | humor:* | angst:*"));
        // The other side has phrases with <-> operator
        assert!(ts.contains("slow:* <-> burn:*"));
        assert!(ts.contains("enemies:* <-> to:* <-> lovers:*"));
    }

    #[test]
    fn test_parse_empty() {
        let expr = parse_query("");
        assert!(matches!(expr, BooleanExpression::And(_)));
        let ts = expr_to_tsquery(&expr);
        assert_eq!(ts, "");
    }

    #[test]
    fn test_parse_whitespace() {
        let expr = parse_query("   ");
        assert!(matches!(expr, BooleanExpression::And(_)));
        let ts = expr_to_tsquery(&expr);
        assert_eq!(ts, "");
    }

    #[test]
    fn test_to_tsquery_fielded_falls_through() {
        let expr = parse_query("coffee AND title:harry");
        let ts = expr_to_tsquery(&expr);
        // Fielded values still produce a tsquery fragment but this is
        // stripped in extract_text_tsquery before the actual query
        assert!(ts.contains("coffee:*"));
        assert!(ts.contains("harry:*"));
    }

    #[test]
    fn test_extract_text_tsquery_removes_fielded() {
        let expr = parse_query("coffee AND title:harry AND author:rowling");
        let text_ts = extract_text_tsquery(&expr);
        assert_eq!(text_ts, "coffee:*");
    }

    #[test]
    fn test_fielded_search_detection() {
        // "source:ao3" should be treated as fielded
        let expr = parse_query("source:ao3");
        let fields = extract_fielded_terms(&expr);
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0], ("source".to_string(), "ao3".to_string()));
    }

    #[test]
    fn test_not_a_colon_field() {
        // "1:Harry" is not a fielded search — it doesn't match known field names
        let expr = parse_query("1:Harry");
        let fields = extract_fielded_terms(&expr);
        assert_eq!(fields.len(), 0);
        // The colon is stripped so the word stays valid tsquery
        let ts = expr_to_tsquery(&expr);
        assert_eq!(ts, "1harry:*");
    }

    #[test]
    fn test_trailing_colon_word_is_sanitized() {
        // Regression: "Holtzmann:" (title token ending in colon) must not
        // produce `holtzmann::*` — that is a Postgres tsquery syntax error.
        let expr = parse_query("Holtzmann:");
        let ts = expr_to_tsquery(&expr);
        assert_eq!(ts, "holtzmann:*");
    }

    #[test]
    fn test_colon_title_query_does_not_error() {
        // Regression for the live bug:
        // `/api/search?q=Jillian+Holtzmann:+Ace+Attorney` previously returned
        // "database error" because `Holtzmann:` → `holtzmann::*`.
        let expr = parse_query("Jillian Holtzmann: Ace Attorney");
        let text_ts = extract_text_tsquery(&expr);
        assert_eq!(text_ts, "jillian:* & holtzmann:* & ace:* & attorney:*");
    }

    #[test]
    fn test_tsquery_special_chars_stripped() {
        // Words containing tsquery metacharacters are sanitized so they can
        // never produce invalid syntax.
        let expr = parse_query("don't stop! (really) & more");
        let ts = expr_to_tsquery(&expr);
        assert_eq!(ts, "dont:* & stop:* & really:* & more:*");
    }

    #[test]
    fn test_empty_word_after_sanitize_is_dropped() {
        // A word that sanitizes to nothing must not join as a stray ' & '.
        let expr = parse_query("::: !!!");
        let ts = expr_to_tsquery(&expr);
        assert_eq!(ts, "");
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
    fn test_phrase_with_fielded_search() {
        let expr = parse_query("title:\"harry potter\"");
        let fields = extract_fielded_terms(&expr);
        // title:"harry potter" should parse as a single fielded term
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0], ("title".to_string(), "harry potter".to_string()));
    }

    #[test]
    fn test_boolean_expression_display() {
        let expr = parse_query("coffee AND (tea OR milk) -sugar");
        let display = expr.to_string();
        assert!(display.contains("AND"));
        assert!(display.contains("OR"));
        assert!(display.contains("NOT"));
    }

    // ─── v2 field-expression tests ────────────────────────────────────────

    /// Pull a single field query of the given field out of an expression.
    fn one(expr: &BooleanExpression, field: Field) -> FieldQuery {
        let qs = extract_field_queries(expr);
        qs.into_iter()
            .find(|q| q.field == field)
            .unwrap_or_else(|| panic!("no field query for {:?} in {:?}", field, expr))
    }

    #[test]
    fn test_v2_at_char_main_role() {
        let expr = parse_query("@char:Harry");
        let q = one(&expr, Field::Char);
        assert!(q.role, "main character role");
        assert_eq!(q.field, Field::Char);
        assert_eq!(q.value.as_deref(), Some("Harry"));
        assert_eq!(q.op, FieldOp::Contains);
    }

    #[test]
    fn test_v2_at_char_quoted_two_words() {
        let expr = parse_query("@char:\"Harry Potter\"");
        let q = one(&expr, Field::Char);
        assert!(q.role);
        assert_eq!(q.value.as_deref(), Some("Harry Potter"));
    }

    #[test]
    fn test_v2_at_fandom_role() {
        let expr = parse_query("@fandom:Harry Potter");
        let q = one(&expr, Field::Fandom);
        assert!(q.role);
    }

    #[test]
    fn test_v2_primary_fandom_forces_role() {
        let expr = parse_query("primary_fandom:harry");
        let q = one(&expr, Field::PrimaryFandom);
        assert!(q.role, "primary_fandom always implies main role");
        assert_eq!(q.value.as_deref(), Some("harry"));
    }

    #[test]
    fn test_v2_romship_polarity() {
        let expr = parse_query("romship:Harry/Ginny");
        let q = one(&expr, Field::Romship);
        assert_eq!(q.field, Field::Romship);
        assert_eq!(Field::Romship.tag_type_id(), Some(3));
        assert_eq!(Field::Romship.rel_polarity(), Some(1));
        assert_eq!(q.value.as_deref(), Some("Harry/Ginny"));
    }

    #[test]
    fn test_v2_platship_polarity() {
        let expr = parse_query("platship:Harry&Ron");
        let q = one(&expr, Field::Platship);
        assert_eq!(Field::Platship.rel_polarity(), Some(2));
        assert_eq!(q.value.as_deref(), Some("Harry&Ron"));
    }

    #[test]
    fn test_v2_ship_any_polarity() {
        let expr = parse_query("ship:Harry/Ginny");
        let _q = one(&expr, Field::Ship);
        assert_eq!(Field::Ship.rel_polarity(), None);
    }

    #[test]
    fn test_v2_words_gt_50k() {
        let expr = parse_query("words:>50k angst");
        let q = one(&expr, Field::Words);
        assert_eq!(q.op, FieldOp::Range(RangeExpr::Gt(50000)));
        // the words filter is stripped from text tsquery
        let text = extract_text_tsquery(&expr);
        assert_eq!(text, "angst:*");
    }

    #[test]
    fn test_v2_words_bare_is_at_least() {
        let expr = parse_query("words:5000");
        let q = one(&expr, Field::Words);
        assert_eq!(q.op, FieldOp::Range(RangeExpr::Ge(5000)));
    }

    #[test]
    fn test_v2_words_range() {
        let expr = parse_query("words:10k-100k");
        let q = one(&expr, Field::Words);
        assert_eq!(q.op, FieldOp::Range(RangeExpr::Between(10000, 100000)));
    }

    #[test]
    fn test_v2_words_cmp_leq() {
        let expr = parse_query("words:<=5k");
        let q = one(&expr, Field::Words);
        assert_eq!(q.op, FieldOp::Range(RangeExpr::Le(5000)));
    }

    #[test]
    fn test_v2_chapters_range() {
        let expr = parse_query("chapters:5-50");
        let q = one(&expr, Field::Chapters);
        assert_eq!(q.op, FieldOp::Range(RangeExpr::Between(5, 50)));
    }

    #[test]
    fn test_v2_published_year() {
        let expr = parse_query("published:2020");
        let q = one(&expr, Field::Published);
        assert_eq!(q.field, Field::Published);
        assert_eq!(q.op, FieldOp::Range(RangeExpr::Between(2020, 2020)));
    }

    #[test]
    fn test_v2_published_year_range() {
        let expr = parse_query("published:2018-2021");
        let q = one(&expr, Field::Published);
        assert_eq!(q.op, FieldOp::Range(RangeExpr::Between(2018, 2021)));
    }

    #[test]
    fn test_v2_updated_gt_year() {
        let expr = parse_query("updated:>2019");
        let q = one(&expr, Field::Updated);
        assert_eq!(q.op, FieldOp::Range(RangeExpr::Gt(2019)));
    }

    #[test]
    fn test_v2_crossover_on_and_off() {
        let on = one(&parse_query("crossover:1"), Field::Crossover);
        assert_eq!(on.value.as_deref(), Some("1"));
        let off = one(&parse_query("crossover:0"), Field::Crossover);
        assert_eq!(off.value.as_deref(), Some("0"));
    }

    #[test]
    fn test_v2_bare_crossover_word_is_not_field() {
        // A plain word "crossover" must remain a normal text word —
        // it must NOT be hijacked as the crossover filter.
        let expr = parse_query("crossover");
        assert!(extract_field_queries(&expr).is_empty());
    }

    #[test]
    fn test_v2_contains_tilde() {
        let expr = parse_query("title~harry");
        let q = one(&expr, Field::Title);
        assert_eq!(q.op, FieldOp::Contains);
        assert_eq!(q.value.as_deref(), Some("harry"));
    }

    #[test]
    fn test_v2_prefix_wildcard() {
        let expr = parse_query("attr:cozy*");
        let q = one(&expr, Field::Attr);
        assert_eq!(q.op, FieldOp::Prefix);
        assert_eq!(q.value.as_deref(), Some("cozy"));
    }

    #[test]
    fn test_v2_suffix_wildcard() {
        let expr = parse_query("attr:*burn");
        let q = one(&expr, Field::Attr);
        assert_eq!(q.op, FieldOp::ContainsWide);
        assert_eq!(q.value.as_deref(), Some("burn"));
    }

    #[test]
    fn test_v2_negated_field_query() {
        let expr = parse_query("-words:>100");
        // `-` glues: Not(FieldQuery(words>100)) -> negated=true
        let q = one(&expr, Field::Words);
        assert!(q.negated, "negated words filter");
        assert_eq!(q.op, FieldOp::Range(RangeExpr::Gt(100)));
    }

    #[test]
    fn test_v2_negated_char_field() {
        let expr = parse_query("-char:Voldemort");
        let q = one(&expr, Field::Char);
        assert!(q.negated);
    }

    #[test]
    fn test_v2_extraction_count() {
        let expr = parse_query("fandom:harry words:>50k @char:Hermione romance:fake");
        let qs = extract_field_queries(&expr);
        // fandom is a v1 Fielded (not FieldQuery); words, @char are FieldQuery.
        assert_eq!(qs.len(), 2);
    }

    #[test]
    fn test_v2_with_operator_is_and() {
        let expr = parse_query("char:Harry with attr:\"Slow Burn\" NOT with attr:Dark");
        let qs = extract_field_queries(&expr);
        let harry = qs.iter().find(|q| q.field == Field::Char).unwrap();
        assert!(!harry.negated);
        let slow = qs
            .iter()
            .find(|q| q.field == Field::Attr && q.value.as_deref() == Some("Slow Burn"))
            .unwrap();
        assert!(!slow.negated);
        let dark = qs
            .iter()
            .find(|q| q.field == Field::Attr && q.value.as_deref() == Some("Dark"))
            .unwrap();
        assert!(dark.negated, "NOT with attr:Dark must be negated");
    }

    #[test]
    fn test_v2_char_multiple_and_polarity() {
        let expr = parse_query("@char:Harry Potter romship:Harry/Ginny words:>100k");
        let qs = extract_field_queries(&expr);
        assert!(qs.iter().any(|q| q.field == Field::Char && q.role));
        assert!(qs.iter().any(|q| q.field == Field::Romship));
        assert!(
            qs.iter()
                .any(|q| q.field == Field::Words && q.op == FieldOp::Range(RangeExpr::Gt(100000)))
        );
    }

    #[test]
    fn test_v2_rating_status_to_fields() {
        let _r = one(&parse_query("rating:Explicit"), Field::Rating);
        assert_eq!(Field::Rating.tag_type_id(), Some(7));
        let s = one(&parse_query("status:wip"), Field::Status);
        assert_eq!(s.value.as_deref(), Some("wip"));
    }

    #[test]
    fn test_v2_kudos_from_works_column() {
        let q = one(&parse_query("kudos:>=100"), Field::Kudos);
        assert_eq!(q.op, FieldOp::Range(RangeExpr::Ge(100)));
        assert!(Field::Kudos.needs_works());
    }

    #[test]
    fn test_v2_description_and_language() {
        let d = one(&parse_query("description~gay"), Field::Description);
        assert_eq!(d.value.as_deref(), Some("gay"));
        let l = one(&parse_query("language:en"), Field::Language);
        assert_eq!(l.value.as_deref(), Some("en"));
        assert!(Field::Language.needs_works());
    }

    #[test]
    fn test_v2_field_with_quoted_phrase() {
        // attr:"coffee shop" — value comes from the next Phrase token.
        let expr = parse_query("attr:\"coffee shop\"");
        let q = one(&expr, Field::Attr);
        assert_eq!(q.value.as_deref(), Some("coffee shop"));
        assert_eq!(q.op, FieldOp::Contains);
    }

    #[test]
    fn test_v2_text_tsquery_strips_field_queries() {
        let expr = parse_query("angst words:>50k crossover:1");
        let text = extract_text_tsquery(&expr);
        assert_eq!(text, "angst:*");
    }

    #[test]
    fn test_v2_field_str_and_type_mapping() {
        assert_eq!(Field::Romship.as_str(), "romship");
        assert_eq!(Field::Char.tag_type_id(), Some(2));
        assert_eq!(Field::Attr.tag_type_id(), Some(4));
        assert!(Field::Words.is_count());
        assert!(Field::Published.is_date());
        assert!(Field::Title.is_text());
        assert!(!Field::Crossover.is_text());
    }

    #[test]
    fn test_v2_eq_comparison() {
        let expr = parse_query("chapters:=7");
        let q = one(&expr, Field::Chapters);
        assert_eq!(q.op, FieldOp::Range(RangeExpr::Eq(7)));
    }
}
