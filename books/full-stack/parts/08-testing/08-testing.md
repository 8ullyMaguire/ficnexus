# Part 8: Testing and Polish

---

# Chapter 34: Backend Testing (cargo test)

## Why Test? (Checking Your Homework Before Submitting)

Think about the last time you turned in homework without checking your answers. Maybe the math problems looked right, the essay seemed complete, and you were pretty confident — until you got the paper back with red marks everywhere.

Software without tests is like homework you never checked. It *looks* right. The happy path works. But as soon as someone types something unexpected, passes a weird URL, or your scraper hits a page layout you've never seen before — boom. Your production server returns a 500 error, or worse, silently produces wrong data.

Testing is checking your homework before you submit it. And in Rust, the tools are built right into the language.

Here's the beautiful thing about testing in Rust: **the test framework ships with the compiler**. There's no `npm install`, no separate test runner to configure, no YAML file with CI pipelines. You write a function, add `#[test]` above it, and run `cargo test`. That's it. The test harness, the assertion macros, the parallel test execution, the filtering — all of it is part of the standard library and the `cargo` toolchain.

But testing is more than just "does the code work?" Good tests:

1. **Document your code** — When you forget what `generate_slug` does in six months, the tests show you exactly what inputs produce what outputs. They're executable documentation that never goes stale.

2. **Catch regressions** — You refactor something, run the tests, and immediately know if you broke something. Without tests, every refactor is a roll of the dice.

3. **Enable confident refactoring** — Want to restructure the entire export pipeline? With good tests, you can do it fearlessly. If the tests pass, you know you haven't broken anything. This is the biggest practical benefit — it lets you improve your code without being terrified of change.

4. **Speed up development** — Instead of manually clicking through a browser to check if EPUB generation works, you run `cargo test` and get an answer in seconds. Tests are faster than manual verification by orders of magnitude.

5. **Serve as living specifications** — The tests describe what your code does. A new team member can read the test suite and understand the system's behavior without reading implementation code.

Let's look at how the FicHub backend uses tests — from simple unit tests to full integration tests that spin up a real database.

### The Testing Pyramid

Before we dive into code, let's talk about the shape of a good test suite. The "testing pyramid" is a mental model that helps you think about what kinds of tests to write:

```
        /  E2E  \          ← Few: slow, expensive, fragile
       /──────────\
      / Integration \      ← Some: test multiple components together
     /────────────────\
    /    Unit Tests     \   ← Many: fast, focused, cheap
   /──────────────────────\
```

- **Unit tests** are at the base — you should have lots of them. They test individual functions in isolation. They're fast (microseconds), cheap to write, and easy to debug when they fail. In FicHub, the unit tests for `generate_slug`, `build_info_string`, and `build_meta_json` run instantly and give you high confidence in those functions.

- **Integration tests** are in the middle — you should have a reasonable number. They test how multiple components work together. A database round-trip test, for example, tests the query function AND the database schema AND the connection pool AND the migration system. FicHub's `tests/integration.rs` has four modules covering database, API routing, export logic, and tag operations.

- **End-to-end tests** are at the top — you should have a few key ones. They test the entire system from HTTP request to response. They're slow, brittle (they break when anything changes), and expensive to maintain. FicHub's curl commands serve as lightweight E2E tests.

The key insight: **more tests at the base means faster feedback**. If you have 100 unit tests that run in 100ms, you get instant feedback on every change. If you have 10 integration tests that run in 5 seconds, you still get fast feedback. But if you only have E2E tests that take 30 seconds each, you'll stop running them and start shipping bugs.

The ratio matters: aim for roughly 70% unit tests, 20% integration tests, and 10% end-to-end tests. FicHub follows this pattern: the most tests are for pure functions (unit), then API routes (integration), then the curl verification (E2E).

## Unit Tests in Rust: #[cfg(test)] Modules

Rust's convention for unit tests is elegant: you put them right next to the code they're testing, inside a special `#[cfg(test)]` module. The `cfg` stands for "configuration" — the `#[cfg(test)]` attribute tells the compiler to only include this module when you run `cargo test`, never in production code.

This means your test code adds **zero** to the binary size of your production build. It's not compiled, not included, not even parsed. It literally doesn't exist outside of `cargo test`.

Here's the pattern you'll see everywhere in FicHub:

```rust
// src/routes/export.rs (simplified)

pub fn generate_slug(title: &str, url_id: &str) -> String {
    let sanitized: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' {
            c
        } else {
            '_'
        })
        .collect();
    let re = regex_lite::Regex::new(r"_+").unwrap();
    let slug = re.replace_all(&sanitized, "_").to_string();
    let slug = slug.trim_matches('_').to_string();
    format!("{}-{}", slug, url_id)
}

#[cfg(test)]
mod tests {
    use super::*;  // Import everything from the parent module

    #[test]
    fn test_generate_slug_basic() {
        let slug = generate_slug("Harry Potter", "abc123");
        assert_eq!(slug, "Harry_Potter-abc123");
    }
}
```

A few things to notice:

1. **`#[cfg(test)]`** — This entire `mod tests` block is stripped out of production builds. Zero runtime cost.

2. **`use super::*;`** — This imports everything from the parent module, giving the test access to all the functions (including private ones!). This is a Rust convention — tests in the same module can test private functions. This is different from many other languages where you can only test public APIs.

3. **`#[test]`** — Marks a function as a test. `cargo test` finds all functions with this attribute and runs them. The function must take no arguments and return nothing (or `Result<(), impl Error>`).

4. **No test runner needed** — Just `cargo test` and you're off.

### Where Do Tests Live?

In Rust, there are two places tests live:

1. **In the source file** — Inside `#[cfg(test)] mod tests { ... }` at the bottom of the file. These are unit tests that have access to private functions. This is the most common pattern in Rust. You'll see it in every FicHub source file that has testable logic.

2. **In the `tests/` directory** — Files like `tests/integration.rs`. These are integration tests that can only test your library's public API (they import your crate as an external dependency). They're compiled separately from the main crate and can't access private functions.

The difference matters: if you want to test a private helper function, you *must* put the test in the source file. If you're testing the public interface of a module, either location works.

FicHub uses both. The unit tests in `src/routes/export.rs` test `generate_slug` directly (even though it's used only internally). The integration tests in `tests/integration.rs` test the same functions through the public API, plus they test database queries, API routing, and the tag system.

There's a third, less common pattern: **test utilities in a separate module**. If you need shared test helpers (like `make_test_meta()` or `TestDb`), you can put them in a `test_helpers` module or in the integration test file. FicHub puts `mock_ficmeta()` in the integration test file so it's available to all four test modules.

🧪 **Try It Yourself:** Create a new Rust project with `cargo new testing-demo`. Add a simple function like `fn add(a: i32, b: i32) -> i32 { a + b }`, write a test for it, and run `cargo test`. Watch the test pass in under a second. Then intentionally write a failing test and see what the error output looks like.

## The assert!, assert_eq!, assert_ne! Macros

Rust gives you three core assertion macros for tests. Think of them as your testing vocabulary.

### assert!

The simplest assertion — it checks that a condition is `true`. If it's `false`, the test panics with a message.

```rust
#[test]
fn test_slug_has_no_colons() {
    let slug = generate_slug("Hello: World?", "xyz");
    assert!(!slug.contains(':'));  // Passes if no colon in slug
    assert!(!slug.contains('?'));  // Passes if no question mark
}

#[test]
fn test_slug_is_always_non_empty() {
    let slug = generate_slug("Any", "id");
    assert!(!slug.is_empty());
}
```

You can also provide a custom failure message:

```rust
#[test]
fn test_slug_format() {
    let slug = generate_slug("Test", "abc");
    assert!(
        slug.ends_with("-abc"),
        "Slug '{}' should end with '-abc'", slug
    );
}
```

If the test fails, Rust shows you the exact expression that failed and a diff if possible. With Rust 2021 edition, you get nice assertion messages:

```
thread 'test_slug_has_no_colons' panicked at
  'assertion failed: !slug.contains(':')'
```

### assert_eq!

Checks that two values are equal. The left and right sides must implement `PartialEq` (for comparison) and `Debug` (for error messages).

```rust
#[test]
fn test_slug_basic() {
    let slug = generate_slug("Harry Potter", "abc123");
    assert_eq!(slug, "Harry_Potter-abc123");
    //  ^^^ expected                    ^^^ actual
}
```

When this test fails, Rust shows both the expected and actual values, which makes debugging a breeze:

```
assertion failed: `(left == right)`
  left: `"Harry_Potter-abc123"`,
 right: `"Harry_Harry_Potter-abc123"`
```

You can see at a glance what went wrong — the slug has "Harry" twice, which means the regex is replacing something it shouldn't.

### assert_ne!

The opposite of `assert_eq!` — checks that two values are NOT equal.

```rust
#[test]
fn test_slug_is_not_empty() {
    let slug = generate_slug("Any Title", "id1");
    assert_ne!(slug, "");  // Slug should never be empty
}

#[test]
fn test_slug_differs_for_different_titles() {
    let slug1 = generate_slug("Title A", "same_id");
    let slug2 = generate_slug("Title B", "same_id");
    assert_ne!(slug1, slug2);
}
```

This is useful for testing that something changed, or that a function doesn't return a default/empty value when it shouldn't. The second example above is a great pattern — it tests that the function actually uses its input, rather than always returning the same value.

### Additional Assert Macros

Rust also provides `assert_matches!` (nightly/unstable) and the ability to use `Result` returns from test functions:

```rust
#[test]
fn test_slug_result() -> Result<(), String> {
    let slug = generate_slug("Test", "id");
    if slug.is_empty() {
        return Err("Slug should not be empty".into());
    }
    Ok(())
}
```

When a test returns `Result`, the test framework treats `Err` as a failure and `Ok` as a pass. This is useful when your test involves operations that can fail (like parsing), since you can use the `?` operator.

⚠️ **Watch Out:** The convention in Rust is `assert_eq!(actual, expected)`, NOT `assert_eq!(expected, actual)`. Some people find this confusing because in other languages like JUnit, the expected value goes first. But in Rust, think of it as "assert that `slug` equals `Harry_Potter-abc123`" — the thing you're testing goes on the left.

## Testing the Slug Generator: generate_slug

The `generate_slug` function takes a fanfiction title and turns it into a URL-safe string. It's a perfect example of a pure function — no database, no network, no file I/O, just string manipulation. And that makes it trivially easy to test.

Here are the tests from the actual FicHub codebase (in `src/routes/export.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_slug_basic() {
        let slug = generate_slug("The Best Story", "abc123");
        assert!(slug.contains("The_Best_Story"));
        assert!(slug.contains("abc123"));
        assert!(!slug.starts_with('_'));
        assert!(!slug.ends_with('_'));
    }

    #[test]
    fn test_generate_slug_special_chars() {
        let slug = generate_slug("Hello: World? (Part 1/2)", "xyz789");
        assert!(!slug.contains(':'));
        assert!(!slug.contains('?'));
        assert!(!slug.contains('('));
        assert!(!slug.contains(')'));
        assert!(!slug.contains('/'));
        assert!(slug.contains("Hello_World_Part_1_2"));
    }

    #[test]
    fn test_generate_slug_collapse_underscores() {
        let slug = generate_slug("A___B___C", "id1");
        // Multiple consecutive underscores should collapse to single ones
        assert_eq!(slug.chars().filter(|&c| c == '_').count(), 2);
    }

    #[test]
    fn test_generate_slug_empty_title() {
        let slug = generate_slug("", "id1");
        assert!(slug.contains("id1"));
        assert!(slug.starts_with("id1") || slug.ends_with("id1"));
    }
}
```

And the integration tests in `tests/integration.rs` add even more cases:

```rust
// tests/integration.rs (export_tests module)

#[test]
fn test_slug_basic() {
    let slug = generate_slug("Harry Potter", "abc123");
    assert_eq!(slug, "Harry_Potter-abc123");
}

#[test]
fn test_slug_special_characters() {
    let slug = generate_slug("Hello, World! @#$%", "id001");
    // Commas, spaces, !@#$% → _, then consecutive _ collapsed
    assert_eq!(slug, "Hello_World-id001");
}

#[test]
fn test_slug_unicode() {
    let slug = generate_slug("Mäßig Hëlló", "u002");
    // Rust's is_alphanumeric() is Unicode-aware, so accented chars pass through
    assert_eq!(slug, "Mäßig_Hëlló-u002");
}

#[test]
fn test_slug_leading_trailing_underscores_collapsed() {
    let slug = generate_slug("___Title___", "t003");
    assert_eq!(slug, "Title-t003");
}

#[test]
fn test_slug_multiple_underscores_collapsed() {
    let slug = generate_slug("A   B___C---D", "x007");
    // Spaces → _, multiple _ collapsed, but --- preserved (hyphen is allowed)
    assert_eq!(slug, "A_B_C---D-x007");
}

#[test]
fn test_slug_empty_title() {
    let slug = generate_slug("", "empty");
    // Empty sanitized → all collapsed away → slug is just "-empty"
    assert_eq!(slug, "-empty");
}
```

Notice how the tests cover different categories:

- **Basic functionality** — Does it work for a normal title?
- **Special characters** — What happens with colons, question marks, parentheses, slashes?
- **Unicode** — Do accented characters pass through or get mangled?
- **Edge cases** — Empty titles, titles that are only special characters
- **Formatting** — Are leading/trailing underscores cleaned up? Are multiple underscores collapsed?

This is the art of writing good tests. You don't just test the happy path — you test the weird, the extreme, and the unexpected. Each test has a clear purpose and a descriptive name. When one fails, you know exactly what broke.

## Testing the Info String Builder: build_info_string

The `build_info_string` function takes metadata about a fic and produces a human-readable summary like:

```
The Testing of the Rings by tolkien_fan
123456 words in 42 chapters
Status: ongoing
Updated: 2023-11-15 06:13:20 - 240 days ago
```

Let's look at the actual tests:

```rust
fn make_test_meta() -> FicMetadata {
    FicMetadata {
        url_id: "test123".into(),
        title: "Test Fic".into(),
        author: "Test Author".into(),
        chapters: 10,
        words: 50000,
        desc: "<p>A great story</p>".into(),
        published: 1700000000000,
        updated: 1700000000000,
        status: "complete".into(),
        source: "https://archiveofourown.org/works/123456".into(),
        source_id: 1,
        author_id: 42,
        author_url: "https://archiveofourown.org/users/TestAuthor".into(),
        author_local_id: "123456".into(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    }
}

#[test]
fn test_build_info_string() {
    let meta = make_test_meta();
    let (info, notes) = build_info_string(&meta);
    assert!(info.contains("Test Fic"));
    assert!(info.contains("Test Author"));
    assert!(info.contains("50000"));
    assert!(info.contains("10"));
    assert!(info.contains("complete"));
    assert!(notes.is_empty());
}
```

Notice the **test helper pattern** — `make_test_meta()` creates a known, controlled piece of data. Instead of copying the same 15-field struct literal into every test, you call one helper. If the `FicMetadata` struct changes (say, a new field is added), you only update one place. This is a huge time-saver as your codebase grows.

The integration tests add a more thorough version using `mock_ficmeta()`:

```rust
fn mock_ficmeta() -> FicMetadata {
    FicMetadata {
        url_id: "a1b2c3d4e5f6".into(),
        title: "The Testing of the Rings".into(),
        author: "tolkien_fan".into(),
        chapters: 42,
        words: 123_456,
        desc: "A thrilling tale of test-driven development in Middle-earth.".into(),
        published: 1_700_000_000_000,
        updated: 1_700_100_000_000,
        status: "ongoing".into(),
        source: "https://example.test/story/a1b2c3d4e5f6".into(),
        source_id: 1,
        author_id: 1001,
        author_url: "https://example.test/u/tolkien_fan".into(),
        author_local_id: "tolkien_fan".into(),
        content_hash: Some("abc123def456".into()),
        extra_meta: Some(r#"{"fandom":"Middle-earth"}"#.into()),
        raw_extended_meta: None,
    }
}

#[test]
fn test_build_info_string_format() {
    let meta = mock_ficmeta();
    let (info, notes) = build_info_string(&meta);

    assert!(info.contains("The Testing of the Rings"));
    assert!(info.contains("tolkien_fan"));
    assert!(info.contains("123456"));
    assert!(info.contains("42"));
    assert!(info.contains("ongoing"));
    assert!(notes.is_empty());
}

#[test]
fn test_build_info_string_updated_time_rendered() {
    let meta = mock_ficmeta();
    let (info, _) = build_info_string(&meta);
    // The updated timestamp is 1_700_100_000_000 millis → about 2023-11-25
    assert!(info.contains("2023-11"));
}

#[test]
fn test_info_string_word_count_formatting() {
    let meta = FicMetadata {
        words: 1234567,
        ..make_test_meta()
    };  // The .. syntax copies all other fields from make_test_meta()
    let (info, _) = build_info_string(&meta);
    assert!(info.contains("1234567"));
}
```

That last test uses the **struct update syntax** (`..make_test_meta()`), which is incredibly useful in tests. You create a default metadata object, then override just the one field you care about. It keeps your tests focused — if you're testing word count formatting, you shouldn't need to set up author names and URLs.

Here's a key insight about the `..` syntax: it copies all fields from the base struct except the ones you explicitly set. So `FicMetadata { words: 1234567, ..make_test_meta() }` creates a metadata with `words: 1234567` and everything else from `make_test_meta()`. It's like Python's `dataclasses.replace()` or Kotlin's `copy()`.

## Testing the Meta JSON Builder: build_meta_json

The `build_meta_json` function converts `FicMetadata` into a `serde_json::Value` for API responses. This is a critical function — if the JSON structure is wrong, the frontend breaks. Every field must be present, correctly named, and correctly typed.

```rust
#[test]
fn test_build_meta_json() {
    let meta = make_test_meta();
    let json = build_meta_json(&meta);
    assert_eq!(json["id"], "test123");
    assert_eq!(json["title"], "Test Fic");
    assert_eq!(json["author"], "Test Author");
    assert_eq!(json["chapters"], 10);
    assert_eq!(json["words"], 50000);
    assert_eq!(json["status"], "complete");
    assert_eq!(json["source_id"], 1);
    assert_eq!(json["author_id"], 42);
}

#[test]
fn test_build_meta_json_iso_dates() {
    let meta = mock_ficmeta();
    let json = build_meta_json(&meta);
    // Both "created" and "updated" should be RFC 3339 strings
    let created = json["created"].as_str().unwrap();
    let updated = json["updated"].as_str().unwrap();
    assert!(!created.is_empty(), "created should not be empty");
    assert!(!updated.is_empty(), "updated should not be empty");
    // Quick check they look like ISO timestamps
    assert!(created.contains('T'), "created should contain T separator");
    assert!(updated.contains('T'), "updated should contain T separator");
}

#[test]
fn test_build_meta_json_extra_meta_passthrough() {
    let meta = mock_ficmeta();
    let json = build_meta_json(&meta);
    assert_eq!(json["extra_meta"], r#"{"fandom":"Middle-earth"}"#);
}
```

The key insight here is **testing the contract, not the implementation**. You're not checking how the JSON is built (whether it uses `json!` macro or manual construction) — you're checking that the JSON has the right fields with the right values. This is what matters to the frontend consumer.

The "contains T" check on dates is a smart pattern — rather than comparing to an exact timestamp (which would make the test brittle and time-dependent), it verifies the format is correct. This is a good lesson: **test structure and format, not exact values, when the values are dynamic**.

The integration tests also verify the full JSON structure more comprehensively:

```rust
#[test]
fn test_build_meta_json_contains_expected_keys() {
    let meta = mock_ficmeta();
    let json: Value = build_meta_json(&meta);

    assert_eq!(json["id"], "a1b2c3d4e5f6");
    assert_eq!(json["title"], "The Testing of the Rings");
    assert_eq!(json["author"], "tolkien_fan");
    assert_eq!(json["chapters"], 42);
    assert_eq!(json["words"], 123_456);
    assert_eq!(json["status"], "ongoing");
    assert_eq!(json["source_id"], 1);
    assert_eq!(json["author_id"], 1001);
    assert_eq!(json["author_url"], "https://example.test/u/tolkien_fan");
    assert_eq!(json["author_local_id"], "tolkien_fan");
    assert!(json["description"]
        .as_str()
        .unwrap()
        .contains("Middle-earth"));
}
```

## Testing the Export Response: build_metadata_response

The `build_metadata_response` function is used when a fic is greylisted — meaning it can show metadata (title, author, word count) but cannot provide download links. This is an important moderation feature.

```rust
#[test]
fn test_build_metadata_response_greylisted() {
    let meta = make_test_meta();
    let resp = super::build_metadata_response(
        &meta,
        &[],       // No extra notes
        &1,        // Export version
        None,      // No cached export
        true,      // is_greylisted = true
    );
    let json = resp.0; // Unwrap the Json wrapper

    // Response is not an error — metadata is available
    assert_eq!(json["err"], 0);

    // Should include a note about greylisting
    assert!(json["notes"][0].as_str()
        .unwrap_or("")
        .contains("greylisted"));

    // Download URLs should be empty
    assert!(json["urls"].as_object().unwrap().is_empty());
    assert!(json["epub_url"].is_null());
    assert!(json["html_url"].is_null());
}
```

This test verifies three things:
1. The response has `err: 0` (not an error — we're just hiding the download links)
2. A note explains why downloads aren't available
3. The download URLs are null/empty

This is a great example of testing business logic through function outputs. The greylisting feature is a moderation decision — this test ensures the decision is correctly communicated to the frontend.

## Testing Error Handling: AppError Variants

FicHub uses an `AppError` enum for error handling. The API convention uses negative error codes to signal different error conditions:

- `-1` — Missing query parameter
- `-5` — Unsupported URL (no scraper found)
- `-7` — Fic or author is blacklisted
- `-10` — Automated requests are blocked

The integration tests verify that these error responses are returned correctly:

```rust
#[tokio::test]
async fn test_epub_no_query_returns_error() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v0/epub")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    assert_eq!(body["err"], -1);
    assert_eq!(body["msg"], "no query");
}

#[tokio::test]
async fn test_epub_invalid_query_returns_error() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v0/epub?q=invalid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    // Our mock returns -5 for any query that is non-empty
    assert!(body["err"].as_i64().unwrap() < 0);
    assert_eq!(body["q"], "invalid");
}

#[tokio::test]
async fn test_cache_nonexistent_returns_error() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/cache/epub/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    assert_eq!(body["err"], -5);
    assert_eq!(body["msg"], "file not found");
}
```

Notice something interesting: these tests use `StatusCode::OK` (200) even for error responses. This is because FicHub's API design returns errors as JSON in the response body with a negative `err` code, rather than using HTTP status codes. This is a common pattern for APIs consumed by JavaScript frontends — it simplifies error handling on the client side since you always get a 200 response and check the `err` field.

⚠️ **Watch Out:** This design choice means your HTTP layer doesn't distinguish between success and failure. If you ever need to add monitoring or logging based on HTTP status codes, you'll miss error cases. Some teams prefer using HTTP 4xx/5xx codes alongside the JSON error body. Choose the pattern that fits your monitoring needs.

## Integration Tests: tests/integration.rs

Unit tests are great for testing individual functions in isolation. But what about testing how those functions work *together*? That's where integration tests come in.

FicHub has a comprehensive integration test file at `tests/integration.rs` with **four modules**, each testing a different layer of the system. Let's explore each one in detail.

### Module 1: Database Tests (Live PostgreSQL)

These tests create a real PostgreSQL connection, spin up a temporary schema, run migrations, and test actual database queries. They require a running database, so they're marked `#[ignore]` by default.

```rust
mod db_tests {
    use super::*;
    use sqlx::PgPool;

    // Global mutex that serialises ALL database tests so they never
    // step on each other.
    static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    fn db_lock() -> &'static Mutex<()> {
        DB_LOCK.get_or_init(|| Mutex::new(()))
    }

    struct TestDb {
        pool: PgPool,
        schema: String,
    }

    impl TestDb {
        async fn new() -> Self {
            let database_url = std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set for db tests");

            let pool = PgPool::connect(&database_url)
                .await
                .expect("failed to connect to test database");

            // Unique schema name per invocation
            let schema = format!(
                "test_{}_{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );

            // Create the schema and set search_path
            sqlx::raw_sql(format!(
                "CREATE SCHEMA IF NOT EXISTS \"{}\"", schema
            ))
            .execute(&pool).await
            .expect("failed to create test schema");

            sqlx::raw_sql(format!(
                "SET search_path TO \"{}\"", schema
            ))
            .execute(&pool).await
            .expect("failed to set search_path");

            // Run migrations from the project's migration directory
            let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            let migrations_path = manifest_dir.join("migrations");
            let migrator = sqlx::migrate::Migrator::new(migrations_path)
                .await
                .expect("failed to load migrations");
            migrator.run(&pool).await
                .expect("failed to run migrations");

            TestDb { pool, schema }
        }

        async fn truncate_all(&self) {
            let tables = [
                "fic_info", "export_log", "fic_blacklist",
                "author_blacklist", "fic_version_bump",
                "request_log", "request_source",
            ];
            for table in &tables {
                sqlx::raw_sql(format!(
                    "TRUNCATE TABLE \"{}\".\"{}\" CASCADE",
                    self.schema, table
                ))
                .execute(&self.pool).await
                .unwrap_or_else(|e| panic!("failed to truncate {table}: {e}"));
            }
        }

        async fn cleanup(&self) {
            sqlx::raw_sql(format!(
                "DROP SCHEMA IF EXISTS \"{}\" CASCADE", self.schema
            ))
            .execute(&self.pool).await
            .unwrap_or_else(|e| {
                panic!("failed to drop schema {}: {e}", self.schema)
            });
        }
    }
```

Now let's look at the actual test functions:

```rust
    #[ignore]
    #[tokio::test]
    async fn test_upsert_and_get_fic_info_round_trip() {
        let _guard = db_lock().lock().unwrap();
        let td = TestDb::new().await;

        let fic = FicInfo {
            id: "a1b2c3d4e5f6".into(),
            created: None,
            updated: None,
            title: "Integration Test Fic".into(),
            author: "test_author".into(),
            author_url: Some("https://example.test/u/test_author".into()),
            author_local_id: Some("test_author".into()),
            chapters: 10,
            words: 50_000,
            description: "A test fic for integration testing purposes.".into(),
            fic_created: Utc::now(),
            fic_updated: Utc::now(),
            status: "ongoing".into(),
            source: "https://example.test/story/a1b2c3d4e5f6".into(),
            extra_meta: None,
            raw_extended_meta: None,
            source_id: Some(1),
            author_id: Some(1001),
            content_hash: Some("hash001".into()),
        };

        queries::upsert_fic_info(&td.pool, &fic).await
            .expect("upsert_fic_info failed");

        let fetched = queries::get_fic_info(&td.pool, "a1b2c3d4e5f6")
            .await
            .expect("get_fic_info failed")
            .expect("expected Some row");

        assert_eq!(fetched.id, fic.id);
        assert_eq!(fetched.title, fic.title);
        assert_eq!(fetched.author, fic.author);
        assert_eq!(fetched.chapters, 10);
        assert_eq!(fetched.words, 50_000);

        td.cleanup().await;
    }

    #[ignore]
    #[tokio::test]
    async fn test_upsert_fic_info_update_existing() {
        let _guard = db_lock().lock().unwrap();
        let td = TestDb::new().await;

        let mut fic = insert_fic_info(&td).await;
        // Update the title
        fic.title = "Updated Title".into();
        queries::upsert_fic_info(&td.pool, &fic).await
            .expect("second upsert failed");

        let fetched = queries::get_fic_info(&td.pool, "a1b2c3d4e5f6")
            .await
            .expect("get_fic_info failed")
            .expect("expected Some row after update");

        assert_eq!(fetched.title, "Updated Title");

        td.cleanup().await;
    }

    #[ignore]
    #[tokio::test]
    async fn test_insert_export_log_and_find_export_log() {
        let _guard = db_lock().lock().unwrap();
        let td = TestDb::new().await;

        insert_fic_info(&td).await;

        queries::insert_export_log(
            &td.pool, "a1b2c3d4e5f6", 1, "epub",
            "inputhash001", "exporthash001",
        )
        .await
        .expect("insert_export_log failed");

        let found = queries::find_export_log(
            &td.pool, "a1b2c3d4e5f6", 1, "epub", "inputhash001",
        )
        .await
        .expect("find_export_log failed")
        .expect("expected Some export_log");

        assert_eq!(found.url_id, "a1b2c3d4e5f6");
        assert_eq!(found.version, 1);
        assert_eq!(found.etype, "epub");
        assert_eq!(found.input_hash, "inputhash001");
        assert_eq!(found.export_hash, "exporthash001");

        td.cleanup().await;
    }

    #[ignore]
    #[tokio::test]
    async fn test_check_fic_blacklist() {
        let _guard = db_lock().lock().unwrap();
        let td = TestDb::new().await;

        insert_fic_info(&td).await;

        // Should be empty initially
        let entries = queries::check_fic_blacklist(&td.pool, "a1b2c3d4e5f6")
            .await
            .expect("check_fic_blacklist failed");
        assert!(entries.is_empty(), "blacklist should be empty initially");

        // Insert a blacklist entry
        sqlx::query(
            "INSERT INTO fic_blacklist (url_id, reason) VALUES ($1, $2)"
        )
        .bind("a1b2c3d4e5f6")
        .bind(5i32)
        .execute(&td.pool)
        .await
        .expect("insert into fic_blacklist failed");

        // Now it should return one row
        let entries = queries::check_fic_blacklist(&td.pool, "a1b2c3d4e5f6")
            .await
            .expect("check_fic_blacklist failed");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].reason, 5);

        td.cleanup().await;
    }

    #[ignore]
    #[tokio::test]
    async fn test_truncate_after_test() {
        let _guard = db_lock().lock().unwrap();
        let td = TestDb::new().await;

        insert_fic_info(&td).await;

        let before = queries::get_fic_info(&td.pool, "a1b2c3d4e5f6")
            .await
            .expect("get_fic_info failed");
        assert!(before.is_some(), "data should exist before truncate");

        td.truncate_all().await;

        let after = queries::get_fic_info(&td.pool, "a1b2c3d4e5f6")
            .await
            .expect("get_fic_info failed");
        assert!(after.is_none(), "data should be gone after truncate");

        td.cleanup().await;
    }
}
```

Key patterns in the database tests:

1. **`#[ignore]`** — Database tests are skipped by default. Run them with `cargo test -- --include-ignored`. This keeps your normal test run fast.

2. **Global mutex** — All DB tests run serially via `OnceLock<Mutex<()>>`. This prevents two tests from trying to create schemas or run migrations simultaneously.

3. **Temporary schemas** — Each test creates its own PostgreSQL schema with a unique name (PID + nanosecond timestamp). This means tests don't interfere with each other even if they run in parallel.

4. **Cleanup** — Every test drops its schema when done with `td.cleanup().await`. This prevents test data from accumulating.

5. **Round-trip testing** — The tests insert data and then read it back, verifying the full write-read cycle. This is the most common database test pattern.

⚠️ **Watch Out:** Database tests are slow (they involve actual network calls to PostgreSQL) and require a running database. For day-to-day development, you'll usually skip them and only run them before deploying. The `#[ignore]` pattern keeps your `cargo test` fast for the common case. Set `DATABASE_URL` as an environment variable before running these tests.

### Module 2: API Endpoint Smoke Tests

These tests build a mock Axum router with simplified handlers and verify that the routing, request parsing, and response formatting work correctly — without needing a real database or scraper:

```rust
mod api_tests {
    use axum::{
        body::Body,
        extract::Query,
        http::{Request, StatusCode},
        response::Json,
        routing::get,
        Router,
    };
    use serde::Deserialize;
    use serde_json::{json, Value};
    use tower::ServiceExt; // oneshot

    // Mock handlers that mirror the real routes
    async fn mock_api_docs() -> Json<Value> {
        Json(json!({
            "name": "fichub-rs API",
            "version": "0.1.0",
            "endpoints": {}
        }))
    }

    #[derive(Debug, Deserialize)]
    struct MockExportQuery {
        q: Option<String>,
    }

    async fn mock_epub_handler(
        Query(params): Query<MockExportQuery>,
    ) -> Json<Value> {
        let query = params.q.as_deref().unwrap_or("");
        if query.is_empty() {
            return Json(json!({"err": -1, "msg": "no query", "q": ""}));
        }
        Json(json!({"err": -5, "msg": "unsupported URL", "q": query}))
    }

    fn test_router() -> Router {
        Router::new()
            .route("/api/", get(mock_api_docs))
            .route("/api/v0/epub", get(mock_epub_handler))
            .route("/api/v0/meta", get(mock_meta_handler))
            .route("/cache/{etype}/{url_id}", get(mock_cache_download))
    }

    #[tokio::test]
    async fn test_get_api_root_returns_valid_json() {
        let app = test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();

        assert_eq!(body["name"], "fichub-rs API");
        assert_eq!(body["version"], "0.1.0");
    }

    #[tokio::test]
    async fn test_epub_no_query_returns_error() {
        let app = test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v0/epub")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();

        assert_eq!(body["err"], -1);
        assert_eq!(body["msg"], "no query");
    }
}
```

The `tower::ServiceExt::oneshot` method is the secret sauce here — it lets you send a single HTTP request through the Axum router without actually starting a server. It's perfect for testing because:

- No network overhead (everything stays in-process)
- No port conflicts (no real TCP binding)
- No cleanup needed (no server to shut down)
- Tests are fast (microseconds per request)

### Module 3: Export Logic Pure-Function Tests

These duplicate and extend the unit tests for `generate_slug`, `build_info_string`, and `build_meta_json`, but in the integration test context where they're testing the library's public API:

```rust
mod export_tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_etype_versions_contains_expected_formats() {
        let versions = export::etype_versions();
        assert_eq!(versions.get("epub"), Some(&1));
        assert_eq!(versions.get("html"), Some(&1));
        assert_eq!(versions.get("mobi"), Some(&0));
        assert_eq!(versions.get("pdf"), Some(&0));
    }

    #[test]
    fn test_compute_version_sums_components() {
        let v = export::compute_version(2, 1, 3);
        assert_eq!(v, 6); // 2 + 1 + 3
    }

    #[test]
    fn test_compute_version_zero_defaults() {
        let v = export::compute_version(0, 0, 0);
        assert_eq!(v, 0);
    }
}
```

These tests verify the export utility functions — the version computation and the export type registry. The `compute_version` function sums three version components (epub version, html version, cache version) to produce a single version number used for cache invalidation.

### Module 4: Tag API + Search Mock Handler Tests

These test the tag submission, voting, flagging, listing, and search endpoints using mock handlers:

```rust
mod tag_api_tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::{get, post},
        Router,
    };
    use serde_json::{json, Value};
    use tower::ServiceExt;

    fn test_tag_router() -> Router {
        Router::new()
            .route("/api/v0/tags/submit", post(mock_tag_submit))
            .route("/api/v0/tags/vote", post(mock_tag_vote))
            .route("/api/v0/tags/flag", post(mock_tag_flag))
            .route("/api/v0/tags", get(mock_tag_list))
            .route("/api/v0/search", get(mock_search))
    }

    #[tokio::test]
    async fn test_tag_submit_returns_success() {
        let app = test_tag_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v0/tags/submit")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&json!({})).unwrap()
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["err"], 0);
        assert_eq!(body["tag"]["id"], 42);
    }

    #[tokio::test]
    async fn test_tag_vote_returns_success() {
        let app = test_tag_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v0/tags/vote")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&json!({})).unwrap()
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["err"], 0);
        assert_eq!(body["new_score"], 2);
    }

    #[tokio::test]
    async fn test_tag_list_returns_tags() {
        let app = test_tag_router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v0/tags")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["err"], 0);
        assert_eq!(body["tags"][0]["name"], "Angst");
    }

    #[tokio::test]
    async fn test_search_returns_empty_results() {
        let app = test_tag_router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v0/search?q=test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["total"], 0);
        assert_eq!(body["page"], 1);
        assert!(body["results"].as_array().unwrap().is_empty());
    }
}
```

These tests verify the routing and response format for the tag system and search. The mock handlers return predictable responses, allowing the tests to focus on request routing, parameter parsing, and response structure.

## Running Tests: cargo test

Here are the commands you'll use daily:

```bash
# Run all unit tests (fast, no database needed)
cargo test

# Run only integration tests
cargo test --test integration

# Run all tests including #[ignore] database tests
cargo test -- --include-ignored

# Run a specific test by name
cargo test test_slug_basic

# Run all tests in a specific module
cargo test export_tests::

# Show println! output (tests capture stdout by default)
cargo test -- --nocapture

# Run tests in a specific file
cargo test --test integration export_tests

# Run tests in parallel (default behavior)
cargo test -- --test-threads=4

# List all tests without running them
cargo test -- --list
```

The output looks like this:

```
running 8 tests
test export_tests::test_slug_basic ... ok
test export_tests::test_slug_special_characters ... ok
test export_tests::test_slug_unicode ... ok
test export_tests::test_build_meta_json_contains_expected_keys ... ok
test export_tests::test_build_meta_json_iso_dates ... ok
test export_tests::test_etype_versions ... ok
test export_tests::test_compute_version_sums_components ... ok
test export_tests::test_compute_version_zero_defaults ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

🧪 **Try It Yourself:** Run `cargo test` in the FicHub directory and watch all the tests pass. Then intentionally break something (change `generate_slug` to always return `"test"`) and watch the tests fail. Fix it and see them pass again. This is the "red-green-refactor" cycle that makes test-driven development so powerful.

### Test Output and Debugging

When a test fails, Rust gives you detailed output:

```
thread 'test_slug_basic' panicked at
  'assertion failed: `(left == right)`
   left: `"test-abc123"`,
  right: `"Harry_Potter-abc123"`',
  src/routes/export.rs:405:8
```

This tells you:
- Which thread failed (the test name)
- The expression that failed
- The left and right values
- The exact file and line number

You can also use `cargo test -- --nocapture` to see `println!` output from your tests. By default, Rust captures stdout/stderr from test threads to keep the output clean.

## Test Coverage with cargo-tarpaulin

How do you know if you're testing enough? Test coverage tools show you what percentage of your code is actually exercised by tests.

`cargo-tarpaulin` is the standard Rust coverage tool:

```bash
# Install it (once)
cargo install cargo-tarpaulin

# Generate a text coverage report
cargo tarpaulin

# Generate HTML report for browsing
cargo tarpaulin --out Html

# Include integration tests
cargo tarpaulin --include-tests

# Show line-by-line coverage for a specific file
cargo tarpaulin --include-files src/routes/export.rs --out stdout
```

The output shows something like:

```
|| Tested/Total Lines:
|| src/routes/export.rs: 85/98 (86.7%)
|| src/scrape/mod.rs: 120/150 (80.0%)
|| src/db/queries.rs: 200/250 (80.0%)
|| Total coverage: 82.5%
```

The HTML report generates a browsable view of your codebase where each line is colored green (tested) or red (not tested). It's incredibly useful for finding untested code paths.

⚠️ **Watch Out:** Don't obsess over getting 100% coverage. A test that tests `assert_eq!(1 + 1, 2)` gives you coverage but provides no value. Aim for meaningful coverage — test your core logic, error paths, and edge cases. Coverage is a tool for finding blind spots, not a metric to maximize.

A good rule of thumb: if a function's coverage is below 70%, you probably need more tests. If it's above 90%, you're in good shape. Between 70-90% depends on how critical the function is.

## Writing Testable Code: Separation of Concerns

The best code is testable code. And testable code follows a simple principle: **separate the things that are hard to test from the things that are easy to test**.

Look at how the export module is structured in FicHub:

```rust
// Pure functions — easy to test, no dependencies
pub fn generate_slug(title: &str, url_id: &str) -> String { ... }
pub fn build_info_string(meta: &FicMetadata) -> (String, Vec<String>) { ... }
pub fn build_meta_json(meta: &FicMetadata) -> Value { ... }
pub fn compute_version(epub: i32, html: i32, cache: i32) -> i32 { ... }

// Handler — hard to test directly (needs DB, scraper, cache)
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> { ... }
```

The `epub_handler` is a 100-line async function that connects to a database, calls scrapers, generates files, and handles caching. Testing *that* directly would be painful — you'd need to mock the database, the scraper, the cache, the HTTP client, and the semaphore system.

But the helper functions (`generate_slug`, `build_info_string`, `build_meta_json`) are standalone functions that take data in and produce data out. No hidden state, no dependencies. You can test them with a simple function call and an assertion.

This separation is what makes FicHub's test suite practical:

1. **Pure functions** (slug, info string, meta JSON) — tested directly with unit tests
2. **Database queries** — tested through the `TestDb` integration test pattern
3. **API routing** — tested through mock Axum routers with `oneshot`
4. **Full handler logic** — would need end-to-end tests (curl against a running server)

The takeaway: **when you write a function, ask yourself: "how would I test this?"** If the answer involves a lot of setup, mocks, or external dependencies, consider whether you can extract the core logic into a pure function and test that instead.

Another testability pattern: **dependency injection through state**. The `epub_handler` receives `AppState` as an Axum extractor, not by creating it internally. This means in tests, you could (in theory) construct a `AppState` with a test database and mock scrapers. This is the "seams" pattern — places where you can substitute test implementations.

---

# Chapter 35: Frontend Testing (vitest)

## What is Vitest? (A Fast Test Runner for Vite)

Remember when testing JavaScript felt like pulling teeth? You needed Jest, which needed Babel, which needed a config file, which needed another config file, and somehow your `import` statements still didn't work. Then someone told you to try Mocha. Then you tried Jasmine. Then you gave up and just manually clicked through the browser.

Vitest is the answer to all that pain. It's a test runner built by the Vite team that works natively with your Vite configuration. No extra Babel setup, no transform pipeline headaches, no fighting with module resolution. If your Vite project works, your Vitest tests work.

The name is a portmanteau of "Vite" and "test" — and it's exactly what you'd expect: a test runner that speaks Vite's language.

Key advantages for FicHub:

- **Native ESM support** — No CommonJS transform needed. Your `import` statements just work. This alone saves hours of configuration headaches.

- **Vite-powered transforms** — TypeScript, Svelte, CSS imports — all handled automatically by your existing Vite config. No need to configure Babel plugins or TypeScript separately for tests.

- **Fast** — It runs tests in the same process as Vite's dev server, so startup is near-instant. Your 51 frontend tests run in about 1 second.

- **Compatible with Jest** — Same `describe`, `it`, `expect` API. If you know Jest, you know Vitest. The migration path is minimal.

- **Snapshot testing** — Built-in support for snapshot tests, which are great for testing component output.

- **Coverage support** — Built-in coverage via c8 or istanbul.

Vitest is part of a broader trend in the JavaScript ecosystem: tools that work *with* your build system instead of fighting it. Vite handles your dev server, Vitest handles your tests, and they share the same configuration. Clean, simple, fast.

## Setting Up vitest, @testing-library/svelte, jsdom

FicHub's frontend uses Vitest with two companion libraries:

1. **`@testing-library/svelte`** — Utilities for rendering Svelte components and interacting with them in tests (clicking buttons, filling inputs, querying the DOM). It follows the Testing Library philosophy: "The more your tests resemble the way your software is used, the more confidence they can give you."

2. **`jsdom`** — A JavaScript implementation of the browser DOM. Since your tests run in Node.js, not a browser, you need jsdom to simulate the DOM environment. It provides `document`, `window`, `HTMLElement`, and all the other browser APIs that Svelte components expect.

The setup lives in two files. First, the test configuration in `vite.config.ts`:

```typescript
// vite.config.ts (test section)
export default defineConfig({
  // ... other config
  test: {
    globals: true,         // describe, it, expect available globally
    environment: 'jsdom',  // Simulate browser DOM
    include: ['src/**/*.{test,spec}.{js,ts}'],
    setupFiles: ['./src/test-setup.ts'],
  },
});
```

And the test setup file:

```typescript
// src/test-setup.ts
// This runs before every test file
// Sets up global fetch mock and any other test infrastructure
```

The `environment: 'jsdom'` setting is crucial — it creates a fake DOM so `@testing-library/svelte` can render your components and inspect their output. Without it, Svelte components would fail because they expect `document.createElement` to exist.

The `globals: true` setting means you don't need to import `describe`, `it`, and `expect` in every test file (though you still can if you prefer explicit imports — FicHub does this in most test files for clarity and IDE autocompletion).

The `setupFiles` option points to a file that runs before every test. This is where you put global setup like mocking `fetch` or setting up test utilities.

🧪 **Try It Yourself:** Create a test file at `src/lib/example.test.ts` with `describe('hello', () => { it('works', () => { expect(1+1).toBe(2); }); });` and run `npx vitest run`. Watch it pass in milliseconds.

### Package Dependencies

To set up frontend testing, you need these packages:

```bash
npm install -D vitest @testing-library/svelte jsdom @testing-library/jest-dom
```

- `vitest` — The test runner (fast, Vite-native)
- `@testing-library/svelte` — Svelte component testing utilities (render, fireEvent, screen)
- `jsdom` — Browser DOM simulation for Node.js (provides document, window, etc.)
- `@testing-library/jest-dom` — Custom matchers like `toBeInTheDocument()`, `toHaveClass()`, `toHaveTextContent()`

And add a test script to `package.json`:

```json
{
  "scripts": {
    "test": "vitest run",
    "test:watch": "vitest",
    "test:coverage": "vitest run --coverage"
  }
}
```

The three scripts cover the main use cases:
- `npm run test` — Run all tests once (good for CI/CD and pre-commit checks)
- `npm run test:watch` — Run tests in watch mode (re-runs when files change, great for development)
- `npm run test:coverage` — Run tests with coverage report (see what's tested and what's not)

### How Vitest Differs from Jest

If you've used Jest before, Vitest will feel familiar — same `describe`, `it`, `expect` API. But there are key differences:

1. **Configuration** — Vitest uses your `vite.config.ts`, not a separate `jest.config.js`. This means your test setup inherits all your Vite aliases, transforms, and plugins.

2. **Speed** — Vitest is typically 2-5x faster than Jest for Vite projects because it reuses Vite's transform pipeline instead of running its own.

3. **ESM-first** — Vitest handles ES modules natively. No need for `transformIgnorePatterns` or `--experimental-vm-modules`.

4. **Watch mode** — Vitest's watch mode is built-in and smarter about which tests to re-run based on file changes.

5. **In-source testing** — Vitest supports writing tests inside your source files (like Rust's `#[cfg(test)]`), though FicHub uses separate test files for clarity.

⚠️ **Watch Out:** If your `@testing-library/svelte` version is too old, it may not support Svelte 5 runes. Check compatibility and upgrade if needed. The FicHub project uses versions that support the latest Svelte features.

## Testing the API Client (client.test.ts)

The API client is the bridge between the Svelte frontend and the Rust backend. It handles all the HTTP requests, response parsing, and error handling. Testing it thoroughly is critical because it's the single point of communication with the server.

### Mocking fetch: globalThis.fetch = mockFetch

The first thing the test file does is mock the global `fetch` function:

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
});
```

`vi.fn()` creates a mock function — a fake function that records every call it receives and lets you control what it returns. `globalThis.fetch = mockFetch` replaces the real `fetch` with our mock. And `beforeEach` resets the mock between tests so they don't interfere with each other.

This pattern — mock the network boundary, test the logic — is universal in frontend testing. You're not testing whether `fetch` works (the browser does that); you're testing whether your client correctly constructs requests and handles responses.

### Dynamic Imports for Mocking

Notice the `importClient()` helper:

```typescript
async function importClient() {
  return await import('./client');
}
```

And tests use it like:

```typescript
it('omits empty/undefined params', async () => {
  const { fetchRecommendations } = await importClient();
  mockFetch.mockResolvedValue({
    ok: true,
    json: async () => ({ err: 0, recommendations: [] }),
  });
  await fetchRecommendations('https://ao3.org/works/1', undefined, 20);
  const called = mockFetch.mock.calls[0][0] as string;
  expect(called).toContain('q=https%3A%2F%2Fao3.org%2Fworks%2F1');
  expect(called).not.toContain('url_id');
  expect(called).toContain('n=20');
});
```

The dynamic import is important — the test re-imports the module after setting up the mock, so the module picks up the mocked `fetch`. If you imported at the top of the file with `import { fetchExport } from './client'`, the original `fetch` would be captured at module load time, before the mock is set up.

### Testing fetchExport(): success and error

```typescript
describe('fetchExport', () => {
  it('returns parsed ExportResponse on success', async () => {
    const { fetchExport } = await importClient();
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        err: 0,
        url_id: 'x1',
        meta: {
          title: 'Test', author: 'A', words: 100,
          chapters: 1, status: 'complete'
        },
        epub_url: '/cache/epub/x1?h=abc',
      }),
    });
    const res = await fetchExport('https://ao3.org/works/1');
    expect(res.err).toBe(0);
    expect(res.url_id).toBe('x1');
    expect(res.epub_url).toBe('/cache/epub/x1?h=abc');
  });

  it('throws ApiError on HTTP failure', async () => {
    const { fetchExport, ApiError } = await importClient();
    mockFetch.mockResolvedValue({
      ok: false,
      status: 500,
      text: async () => 'boom'
    });
    await expect(fetchExport('u')).rejects.toBeInstanceOf(ApiError);
  });
});
```

The first test verifies the happy path: the API returns a successful response, and `fetchExport` correctly parses it into an `ExportResponse` object with all the expected fields.

The second test verifies error handling: when the server returns a 500, the function throws an `ApiError` instead of crashing. The `.rejects.toBeInstanceOf(ApiError)` matcher is specific to async error testing — it awaits the rejected promise and checks the error type.

### Testing submitSuggestion(): POST body

```typescript
describe('submitSuggestion', () => {
  it('POSTs json body with url_id and suggested_url', async () => {
    const { submitSuggestion } = await importClient();
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, suggestion_id: 42 }),
    });
    const res = await submitSuggestion(
      'seed1', 'https://ao3.org/works/9', 'great'
    );
    expect(res.err).toBe(0);
    expect(res.suggestion_id).toBe(42);

    // Verify the request itself
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toContain('/recommendations/suggest');
    expect(init.method).toBe('POST');
    const body = JSON.parse(init.body as string);
    expect(body.url_id).toBe('seed1');
    expect(body.suggested_url).toBe('https://ao3.org/works/9');
    expect(body.comment).toBe('great');
  });
});
```

This test goes beyond checking the response — it verifies the **request itself**. `mockFetch.mock.calls[0]` gives you the first call to the mock, and you can inspect the URL, HTTP method, and request body. This is how you test that your client is actually sending the right data.

The destructuring `[url, init]` extracts the two arguments that `fetch` receives: the URL string and the `RequestInit` object (which contains method, headers, body, etc.).

### Testing castVote(): vote values

```typescript
describe('castVote', () => {
  it('POSTs suggestion_id and vote', async () => {
    const { castVote } = await importClient();
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, new_score: 3 }),
    });
    const res = await castVote(7, 1);
    expect(res.new_score).toBe(3);
    const body = JSON.parse(mockFetch.mock.calls[0][1].body as string);
    expect(body.suggestion_id).toBe(7);
    expect(body.vote).toBe(1);
  });
});
```

The pattern is consistent: mock the response, call the function, verify the output, and inspect the request.

### The Full Test Coverage of the API Client

Here's a summary of what the client tests cover:

| Test | What it verifies |
|------|-----------------|
| `buildQuery omits empty params` | No empty query parameters sent |
| `buildQuery includes url_id` | URL ID parameter works |
| `fetchExport returns on success` | Happy path parsing |
| `fetchExport throws ApiError` | Error handling |
| `submitSuggestion POSTs body` | Correct request format |
| `castVote POSTs values` | Vote payload correct |

This covers the main API operations the frontend performs. For a small client module, this is thorough coverage.

## Testing the Search API (search.test.ts)

The search API module builds query strings from filter objects. These are pure functions — no network calls, no side effects — making them perfect candidates for unit testing.

### buildSearchQuery(): All Parameter Types

```typescript
import {
  buildSearchQuery, defaultFilters,
  SORT_OPTIONS, COMPLETE_OPTIONS, SOURCE_OPTIONS
} from './search';

describe('defaultFilters', () => {
  it('returns empty/null defaults', () => {
    const f = defaultFilters();
    expect(f.q).toBe('');
    expect(f.include_tags).toBe('');
    expect(f.min_words).toBeNull();
    expect(f.complete).toBeNull();
    expect(f.page).toBe(1);
    expect(f.per_page).toBe(20);
  });
});

describe('buildSearchQuery', () => {
  it('omits empty params', () => {
    const qs = buildSearchQuery(defaultFilters());
    expect(qs).toBe('');
  });

  it('includes q when set', () => {
    const f = defaultFilters();
    f.q = 'Harry Potter';
    expect(buildSearchQuery(f)).toContain('q=Harry+Potter');
  });

  it('includes min_words when set', () => {
    const f = defaultFilters();
    f.min_words = 10000;
    expect(buildSearchQuery(f)).toContain('min_words=10000');
  });

  it('includes complete=true', () => {
    const f = defaultFilters();
    f.complete = true;
    expect(buildSearchQuery(f)).toContain('complete=true');
  });

  it('includes sort', () => {
    const f = defaultFilters();
    f.sort = 'updated';
    expect(buildSearchQuery(f)).toContain('sort=updated');
  });

  it('includes source', () => {
    const f = defaultFilters();
    f.source = 'archiveofourown.org';
    expect(buildSearchQuery(f))
      .toContain('source=archiveofourown.org');
  });

  it('includes date_from in ISO format', () => {
    const f = defaultFilters();
    f.date_from = '2024-01-01T00:00:00Z';
    expect(buildSearchQuery(f))
      .toContain('date_from=2024-01-01T00%3A00%3A00Z');
  });

  it('includes include_tags', () => {
    const f = defaultFilters();
    f.include_tags = '1:Harry Potter,4:Fluff';
    expect(buildSearchQuery(f))
      .toContain('include_tags=1%3AHarry+Potter%2C4%3AFluff');
  });

  it('omits page when 1', () => {
    const f = defaultFilters();
    f.page = 1;
    expect(buildSearchQuery(f)).not.toContain('page=');
  });

  it('includes page when > 1', () => {
    const f = defaultFilters();
    f.page = 3;
    expect(buildSearchQuery(f)).toContain('page=3');
  });
});

describe('constants', () => {
  it('SORT_OPTIONS has entries', () => {
    expect(SORT_OPTIONS.length).toBeGreaterThan(0);
  });
  it('COMPLETE_OPTIONS has 3 entries', () => {
    expect(COMPLETE_OPTIONS).toHaveLength(3);
  });
  it('SOURCE_OPTIONS has entries', () => {
    expect(SOURCE_OPTIONS.length).toBeGreaterThan(0);
  });
});
```

Each test covers a single parameter type. This is the **one assertion per test** philosophy — each test has a clear, specific purpose. If the test fails, you know exactly what broke.

Note the first test: `buildSearchQuery(defaultFilters())` returns an empty string. This verifies that default filters produce no query string, which is correct — you don't want to send empty parameters to the API. The API treats missing parameters as "no filter," which is the right default behavior.

The URL encoding tests (like `%3A` for `:` and `%2C` for `,`) verify that special characters are properly encoded for HTTP query strings. This is critical — if encoding is wrong, the API will parse parameters incorrectly.

The constants tests verify that the option arrays have the expected sizes. This is a lightweight way to catch accidental deletions — if someone removes a sort option from the array, the test catches it.

## Testing Utility Functions (util.test.ts)

Utility functions are the workhorses of any frontend — formatting text, detecting patterns, cleaning up HTML. They're pure functions with clear inputs and outputs, making them ideal for testing.

```typescript
import {
  formatWords, detectSite, stripHtml, relativeTime, cacheUrl
} from './util';

describe('formatWords', () => {
  it('adds thousands separators', () => {
    expect(formatWords(1234567)).toBe('1,234,567');
    expect(formatWords(50000)).toBe('50,000');
    expect(formatWords(0)).toBe('0');
  });
});

describe('detectSite', () => {
  it('detects AO3', () => {
    expect(detectSite('https://archiveofourown.org/works/1'))
      .toBe('AO3');
  });
  it('detects FanFiction.net', () => {
    expect(detectSite('https://www.fanfiction.net/s/1/1/Title'))
      .toBe('FanFiction.net');
  });
  it('detects XenForo forums', () => {
    expect(detectSite('https://forums.spacebattles.com/threads/x.1'))
      .toBe('Forum');
  });
  it('returns Unknown for unrecognized', () => {
    expect(detectSite('https://example.com/story')).toBe('Unknown');
  });
});

describe('stripHtml', () => {
  it('removes tags and decodes entities', () => {
    expect(stripHtml('<p>Hello &amp; welcome</p>'))
      .toBe('Hello & welcome');
    expect(stripHtml('<br>line1<br>line2'))
      .toBe('line1 line2');
    expect(stripHtml('')).toBe('');
  });
});

describe('relativeTime', () => {
  it('returns recent for now', () => {
    expect(relativeTime(new Date().toISOString()))
      .toContain('minute');
  });
  it('returns empty for empty input', () => {
    expect(relativeTime('')).toBe('');
  });
});

describe('cacheUrl', () => {
  it('builds correct cache path', () => {
    expect(cacheUrl('epub', 'abc', 'def'))
      .toBe('/cache/epub/abc?h=def');
  });
});
```

Notice the testing strategy:

- **`formatWords`** — Tests large numbers, round thousands, and zero. These are the values that formatting bugs typically affect — large numbers that need commas, exact thousands, and the edge case of zero.

- **`detectSite`** — Tests each supported site (AO3, FFN, XenForo) plus the fallback case. The fallback is important — if someone passes a URL from an unsupported site, the function should degrade gracefully, not crash.

- **`stripHtml`** — Tests tag removal, HTML entity decoding (`&amp;` → `&`), and empty input. The empty input test is a common pattern — it verifies the function handles the absence of data.

- **`relativeTime`** — Tests current time (should say "X minutes ago") and empty input (should return empty string, not crash). Note the `toContain('minute')` instead of an exact match — relative time tests should be fuzzy because they depend on when the test runs.

- **`cacheUrl`** — Tests URL construction with all three parts (type, ID, hash). This is a simple format string, but getting it wrong would break all downloads.

⚠️ **Watch Out:** The `relativeTime` test uses `toContain('minute')` rather than an exact match because the output depends on when the test runs. If the test takes more than 60 seconds (unlikely but possible under heavy load), the output would change from "less than a minute" to "1 minutes". Use fuzzy assertions for time-dependent tests.

## Testing the Syntax Parser (syntax.test.ts)

The search syntax parser is one of FicHub's most interesting frontend features. It takes a query string like `fandom:Harry Potter tag:Fluff -tag:Angst words:10000-50000` and parses it into structured filter objects. With **28 tests**, it's the most thoroughly tested frontend module.

```typescript
import { parseSearchQuery } from './syntax';

describe('parseSearchQuery', () => {
  // ── Bare words ──
  it('parses bare words into q', () => {
    const f = parseSearchQuery('hello world');
    expect(f.q).toBe('hello world');
    expect(f.include_tags).toBe('');
  });

  // ── Key:value pairs ──
  it('parses title:', () => {
    const f = parseSearchQuery('title:The Best Story');
    expect(f.q).toBe('The Best Story');
  });

  it('parses author:', () => {
    const f = parseSearchQuery('author:J.K. Rowling');
    expect(f.q).toBe('J.K. Rowling');
  });

  it('parses creator: as alias for author:', () => {
    const f = parseSearchQuery('creator:SomeAuthor');
    expect(f.q).toBe('SomeAuthor');
  });

  it('parses fandom:', () => {
    const f = parseSearchQuery('fandom:Harry Potter');
    expect(f.include_tags).toBe('1:Harry Potter');
  });

  it('parses char:', () => {
    const f = parseSearchQuery('char:Harry Potter');
    expect(f.include_tags).toBe('2:Harry Potter');
  });

  it('parses character: as alias', () => {
    const f = parseSearchQuery('character:Draco Malfoy');
    expect(f.include_tags).toBe('2:Draco Malfoy');
  });

  it('parses rel:', () => {
    const f = parseSearchQuery('rel:Harry/Hermione');
    expect(f.include_tags).toBe('3:Harry/Hermione');
  });

  it('parses tag:', () => {
    const f = parseSearchQuery('tag:Fluff');
    expect(f.include_tags).toBe('4:Fluff');
  });

  it('parses freeform: as alias for tag:', () => {
    const f = parseSearchQuery('freeform:Angst');
    expect(f.include_tags).toBe('4:Angst');
  });

  // ── Exclusions ──
  it('parses exclusion with -', () => {
    const f = parseSearchQuery('-tag:Major Character Death');
    expect(f.exclude_tags).toBe('4:Major Character Death');
  });

  it('parses -fandom:', () => {
    const f = parseSearchQuery('-fandom:Harry Potter');
    expect(f.exclude_tags).toBe('1:Harry Potter');
  });

  // ── Numeric ranges ──
  it('parses words:10000-50000', () => {
    const f = parseSearchQuery('words:10000-50000');
    expect(f.min_words).toBe(10000);
    expect(f.max_words).toBe(50000);
  });

  it('parses words:>10000', () => {
    const f = parseSearchQuery('words:>10000');
    expect(f.min_words).toBe(10000);
    expect(f.max_words).toBeNull();
  });

  it('parses words:<5000', () => {
    const f = parseSearchQuery('words:<5000');
    expect(f.min_words).toBeNull();
    expect(f.max_words).toBe(5000);
  });

  it('parses chapters:5-20', () => {
    const f = parseSearchQuery('chapters:5-20');
    expect(f.min_chapters).toBe(5);
    expect(f.max_chapters).toBe(20);
  });

  // ── Boolean values ──
  it('parses complete:true', () => {
    const f = parseSearchQuery('complete:true');
    expect(f.complete).toBe(true);
  });

  it('parses complete:false', () => {
    const f = parseSearchQuery('complete:false');
    expect(f.complete).toBe(false);
  });

  it('parses complete:yes', () => {
    const f = parseSearchQuery('complete:yes');
    expect(f.complete).toBe(true);
  });

  // ── Site aliases ──
  it('parses site:ao3', () => {
    const f = parseSearchQuery('site:ao3');
    expect(f.source).toBe('archiveofourown.org');
  });

  it('parses site:ffn', () => {
    const f = parseSearchQuery('site:ffn');
    expect(f.source).toBe('fanfiction.net');
  });

  it('parses site:sv', () => {
    const f = parseSearchQuery('site:sv');
    expect(f.source).toBe('forums.sufficientvelocity.com');
  });

  // ── Sort and date filters ──
  it('parses sort:updated', () => {
    const f = parseSearchQuery('sort:updated');
    expect(f.sort).toBe('updated');
  });

  it('parses after:2024-01-01', () => {
    const f = parseSearchQuery('after:2024-01-01');
    expect(f.date_from).toBe('2024-01-01T00:00:00Z');
  });

  it('parses before:2024-12-31', () => {
    const f = parseSearchQuery('before:2024-12-31');
    expect(f.date_to).toBe('2024-12-31T23:59:59Z');
  });

  // ── Complex and edge cases ──
  it('handles complex queries', () => {
    const f = parseSearchQuery(
      'fandom:Harry Potter tag:Fluff -tag:Angst ' +
      'words:10000-50000 complete:true sort:updated'
    );
    expect(f.include_tags).toBe('1:Harry Potter,4:Fluff');
    expect(f.exclude_tags).toBe('4:Angst');
    expect(f.min_words).toBe(10000);
    expect(f.max_words).toBe(50000);
    expect(f.complete).toBe(true);
    expect(f.sort).toBe('updated');
  });

  it('handles empty string', () => {
    const f = parseSearchQuery('');
    expect(f.q).toBe('');
    expect(f.include_tags).toBe('');
  });

  it('handles mixed bare words and tokens', () => {
    const f = parseSearchQuery('best story fandom:Harry Potter');
    expect(f.q).toBe('best story');
    expect(f.include_tags).toBe('1:Harry Potter');
  });
});
```

### The Greedy Tokenizer: Multi-Word Values

One of the parser's trickiest jobs is handling multi-word values. When a user types `fandom:Harry Potter tag:Fluff`, the parser needs to know that "Harry Potter" is the value for `fandom:`, not just "Harry".

The parser uses a **greedy tokenizer** — when it encounters `fandom:`, it reads everything until the next recognized token prefix (like `tag:`, `-tag:`, `words:`, etc.) or the end of the string. This is what allows `fandom:Harry Potter` to correctly capture "Harry Potter" as the fandom value.

The tests verify this implicitly through the complex query test, which mixes bare words, key:value pairs, exclusions, ranges, booleans, and sort — all in a single query string. If the tokenizer wasn't greedy, `fandom:Harry Potter tag:Fluff` would only capture "Harry" as the fandom value.

Here's how the tokenizer works conceptually:

```
Input:  "fandom:Harry Potter tag:Fluff words:10000-50000"
Tokens:
  fandom: → reads until next keyword "tag:" → value: "Harry Potter"
  tag:    → reads until next keyword "words:" → value: "Fluff"
  words:  → reads until end of string → value: "10000-50000"
```

The tag type IDs (1=fandom, 2=character, 3=relationship, 4=freeform) are hardcoded in the parser. These map to the database's tag type system.

⚠️ **Watch Out:** The tag type IDs are hardcoded in the parser. If you change these IDs on the backend, you need to update the parser tests too. This is a good argument for sharing constants between frontend and backend, or at least documenting the mapping clearly in both codebases.

## Testing Components (DownloadTab.test.ts)

Testing Svelte components is where things get interesting. You're not just testing functions anymore — you're testing a living, interactive UI.

The `DownloadTab` component is the main interface for downloading fanfiction. It has an input field, a download button, and displays results. Testing it involves rendering the component, simulating user interactions, and verifying the output.

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
});

async function loadDownloadTab() {
  return await import('$lib/components/DownloadTab.svelte');
}

describe('DownloadTab', () => {
  it('shows an error when URL is empty', async () => {
    const { default: DownloadTab } = await loadDownloadTab();
    render(DownloadTab);
    await fireEvent.click(screen.getByText('Download'));
    expect(screen.getByText(/paste a fanfiction URL/i)).toBeTruthy();
  });

  it('renders download links on success', async () => {
    const { default: DownloadTab } = await loadDownloadTab();
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        err: 0,
        url_id: 'abc123',
        meta: {
          id: 'abc123',
          title: 'My Story',
          author: 'Author',
          chapters: 10,
          words: 50000,
          description: '<p>desc</p>',
          status: 'complete',
          source: 'https://archiveofourown.org/works/1',
          created: '2024-01-01T00:00:00Z',
          updated: '2024-01-02T00:00:00Z',
          extra_meta: null,
          raw_extended_meta: null,
          author_url: 'https://archiveofourown.org/users/Author',
          author_local_id: 'a1',
          source_id: 1,
          author_id: 2,
        },
        epub_url: '/cache/epub/abc123?h=xyz',
        html_url: '/cache/html/abc123?h=html1',
      }),
    });
    render(DownloadTab);

    // Type a URL into the input
    const input = screen.getByLabelText('Fanfiction URL')
      as HTMLInputElement;
    await fireEvent.input(input, {
      target: { value: 'https://archiveofourown.org/works/1' }
    });

    // Click the Download button
    await fireEvent.click(screen.getByText('Download'));

    // Wait for results to appear
    await waitFor(() =>
      expect(screen.getByText('My Story')).toBeTruthy()
    );
    expect(screen.getByText('EPUB')).toBeTruthy();
    expect(screen.getByText('HTML')).toBeTruthy();
  });
});
```

Let's break down the key concepts:

### render()

`render(DownloadTab)` mounts the Svelte component into a fake DOM (provided by jsdom). The component is now "alive" — it has state, it responds to events, and its HTML is in the test DOM. The `render` function returns utility functions for cleanup and debugging.

### screen queries

`screen` is your way of finding elements in the rendered DOM. Some common queries:

- `screen.getByText('Download')` — Find an element containing this text (throws if not found or if multiple match)
- `screen.getByLabelText('Fanfiction URL')` — Find an input by its label (great for accessibility)
- `screen.getByText(/paste a fanfiction URL/i)` — Find by regex pattern (case-insensitive)

These queries are part of `@testing-library/svelte`'s philosophy: **test the way users interact with your app**. Users don't care about CSS classes or DOM structure — they see text, labels, and buttons. Your tests should mirror that.

Query priority (from recommended to least):
1. `getByRole` — Accessible roles (button, textbox, etc.)
2. `getByLabelText` — Form elements with labels
3. `getByPlaceholderText` — Placeholder text
4. `getByText` — Visible text
5. `getByTestId` — Data attributes (last resort)

### fireEvent

`fireEvent.click()`, `fireEvent.input()`, `fireEvent.change()` — these simulate user interactions. They dispatch DOM events that trigger Svelte's reactive updates.

`fireEvent.input` is specifically for input fields — it fires the `input` event that Svelte listens on for two-way bindings. Using `fireEvent.change` instead might not trigger the binding update.

### waitFor

`waitFor(() => expect(...))` polls the DOM until the assertion passes. This is essential for testing asynchronous behavior — the fetch call returns a promise, and the DOM updates after the promise resolves. Without `waitFor`, your test would check the DOM before the data arrives, and the assertion would fail.

The polling interval is configurable (default 50ms), and it has a timeout (default 1000ms). If the assertion doesn't pass within the timeout, the test fails.

### Lazy Component Loading

Notice the `loadDownloadTab()` helper:

```typescript
async function loadDownloadTab() {
  return await import('$lib/components/DownloadTab.svelte');
}
```

Components are loaded lazily (with `await import()`) inside each test function, not at the top of the file. This is because the `$lib` alias needs the Vite build configuration to be set up, which happens after the test setup files run. Loading at the top of the file would try to resolve the alias before it's configured.

The first test (empty URL) verifies **client-side validation** — when the user clicks Download without entering a URL, the component shows an error message. This is a UX test as much as a code test — it verifies that the user gets helpful feedback.

The second test verifies the **happy path** — entering a valid URL, clicking Download, and seeing the results. Notice the `mockFetch.mockResolvedValue(...)` — this sets up the mock to return a realistic response, including all the metadata fields that the component needs to render.

🧪 **Try It Yourself:** Write a test for the SearchTab component. Mock the search API to return a few results, render the component, type a query, click Search, and verify that result cards appear. This will exercise the same patterns: mock fetch, render, fireEvent, waitFor.

## Running All Tests: npm run test

Here are the commands you'll use daily:

```bash
# Run all tests once (good for CI)
npm run test

# Run tests in watch mode (re-runs when files change)
npx vitest

# Run a specific test file
npx vitest run src/lib/api/client.test.ts

# Run tests matching a pattern
npx vitest run -t "fetchExport"

# Run with coverage
npx vitest run --coverage

# Run in CI mode (no watch, with reporters)
npx vitest run --reporter=junit

# Run with verbose output
npx vitest run --reporter=verbose
```

The output looks like:

```
 ✓ src/lib/api/client.test.ts (5 tests)
 ✓ src/lib/api/search.test.ts (11 tests)
 ✓ src/lib/util.test.ts (5 tests)
 ✓ src/lib/search/syntax.test.ts (28 tests)
 ✓ src/lib/components/DownloadTab.test.ts (2 tests)

 Test Files  5 passed (5)
      Tests  51 passed (51)
   Start at  14:23:07
   Duration  1.24s
```

51 tests, all passing, in 1.24 seconds. That's the beauty of Vitest — it's fast enough to run on every save. When you're in watch mode (`npx vitest` without `run`), it re-runs only the tests affected by the file you just changed, which is often even faster.

### Understanding the Watch Mode

When you run `npx vitest` (without `run`), it enters watch mode. This is incredibly useful during development:

```
> npx vitest

 ✓ src/lib/util.test.ts (5 tests)
   Waiting for file changes...

   press h to show help, q to quit
```

Every time you save a file, Vitest re-runs the relevant tests. This gives you instant feedback on whether your changes broke anything. It's like having a constant safety net while you code.

⚠️ **Watch Out:** If your tests fail with "Cannot find module '$lib/...", you probably need to check that your `vite.config.ts` has the correct aliases. The `$lib` alias needs to be defined both in Vite config and in the Vitest config section. FicHub handles this by lazy-importing components with `await import('$lib/components/DownloadTab.svelte')` inside each test function, which ensures the alias is resolved after the build configuration is loaded.

---

# Chapter 36: Full-Stack Integration and What's Next

## How Everything Fits Together

If you've been reading this book in order, you've now built a complete fanfiction download and management platform. Let's take a step back and appreciate the full picture.

Here's what lives inside the `fichub` repository:

```
fichub/
├── src/                        # Rust backend
│   ├── main.rs                 # Entry point — starts the server
│   ├── server.rs               # Axum server setup, shared state
│   ├── error.rs                # AppError enum
│   ├── routes/
│   │   ├── export.rs           # /api/v0/epub — the main download endpoint
│   │   ├── meta.rs             # /api/v0/meta — metadata-only endpoint
│   │   ├── search.rs           # /api/v0/search — full-text search
│   │   └── tags.rs             # /api/v0/tags — tag CRUD
│   ├── scrape/
│   │   ├── mod.rs              # Scraper trait + registry
│   │   ├── ao3.rs              # Archive of Our Own scraper
│   │   ├── ffn.rs              # FanFiction.net scraper
│   │   └── xenforo.rs          # XenForo forum scraper
│   ├── export/
│   │   ├── mod.rs              # Version computation, format registry
│   │   ├── epub.rs             # EPUB generation
│   │   └── html_bundle.rs      # HTML bundle generation
│   ├── db/
│   │   ├── models.rs           # FicInfo, ExportLog, etc.
│   │   └── queries.rs          # SQLx query functions
│   ├── cache/                  # Disk cache management
│   ├── tags/                   # Tag resolution system
│   └── recommendations/        # Recommendation engine
├── migrations/                 # SQL migrations (001_init.sql, etc.)
├── tests/
│   └── integration.rs          # 4-module integration test suite
├── frontend/                   # SvelteKit frontend
│   └── src/
│       ├── lib/
│       │   ├── api/
│       │   │   ├── client.ts       # API client functions
│       │   │   ├── client.test.ts  # API client tests
│       │   │   ├── search.ts       # Search query builder
│       │   │   └── search.test.ts  # Search API tests
│       │   ├── components/
│       │   │   ├── DownloadTab.svelte
│       │   │   ├── DownloadTab.test.ts
│       │   │   └── SearchTab.svelte
│       │   ├── search/
│       │   │   ├── syntax.ts       # Search syntax parser
│       │   │   └── syntax.test.ts  # Parser tests (28 tests)
│       │   ├── util.ts             # Utility functions
│       │   └── util.test.ts        # Utility tests
│       └── routes/
│           ├── +page.svelte        # Main page
│           └── +layout.svelte      # Layout wrapper
├── Cargo.toml                  # Rust dependencies
├── package.json                # Node.js dependencies
└── vite.config.ts              # Vite + Vitest config
```

## The Complete Request Flow

When a user pastes an AO3 URL and clicks "Download," here's exactly what happens, step by step:

### 1. Browser → SvelteKit

The user types `https://archiveofourown.org/works/123456` into the input field and clicks "Download". The `DownloadTab.svelte` component captures the input and calls `fetchExport(url)` from `client.ts`.

### 2. SvelteKit → API Client

The API client constructs a URL:
```
/api/v0/epub?q=https://archiveofourown.org/works/123456
```
It calls `globalThis.fetch()` with this URL, adding headers for CORS and content type.

### 3. API Client → Rust Backend

The request hits the Axum server, which routes it to `epub_handler` in `export.rs`. Axum deserializes the query parameters into an `ExportQuery` struct.

### 4. Rust → Scraper Registry

The handler asks the `ScraperRegistry` to find a scraper that handles `archiveofourown.org`. The registry pattern-matches on the URL hostname and returns an `Ao3Scraper`. If no scraper matches, it returns `AppError::BadRequest(-5, "unsupported URL")`.

### 5. Scraper → External Site

The `Ao3Scraper` makes HTTP requests to `archiveofourown.org/works/123456` using `reqwest`. It parses the HTML with `scraper` (the Rust crate), extracts title, author, chapters, word count, tags, and other metadata. This produces a `FicMetadata` struct.

### 6. Database Check

The handler looks up the fic in `fic_info` (inserts or updates via `upsert_fic_info`). It checks blacklists (`check_fic_blacklist`, `check_author_blacklist`). It checks for a version bump (`get_fic_version_bump`). It checks the cache (`find_export_log`).

### 7. Cache Hit or Export

If the EPUB is cached, it returns immediately with the cache URL (`/cache/epub/{url_id}?h={hash}`). If not, it:
  - Acquires a semaphore (prevents duplicate concurrent exports)
  - Fetches all chapter content via the scraper
  - Generates an EPUB using the `epub` crate
  - Generates an HTML bundle
  - Moves both files to the cache directory
  - Records them in `export_log`

### 8. Response → Browser

The response JSON includes `epub_url`, `html_url`, `meta`, `info`, `slug`, and `notes`. The frontend parses this JSON and displays the metadata and download links.

### 9. Download

When the user clicks the EPUB link, the browser requests `/cache/epub/{url_id}?h={hash}`, and the server reads the cached file from disk and streams it to the browser.

That's the entire flow — from user click to EPUB download. Every piece we built in Parts 1-7 participates.

## End-to-End Testing: curl Commands to Verify

The simplest way to test the full stack is with `curl`. Here are the commands that verify every major feature:

```bash
# 1. Check the server is running and responding
curl http://localhost:3000/api/

# Expected: {"name":"fichub-rs API","version":"0.1.0",...}

# 2. Export a fic from AO3
curl "http://localhost:3000/api/v0/epub?q=https://archiveofourown.org/works/123456"

# Expected: {"err":0,"url_id":"...","epub_url":"/cache/epub/...",...}

# 3. Get metadata only (no EPUB generation)
curl "http://localhost:3000/api/v0/meta?q=https://archiveofourown.org/works/123456"

# Expected: {"err":0,"url_id":"...","meta":{...}}

# 4. Search for fics
curl "http://localhost:3000/api/v0/search?q=harry+potter"

# Expected: {"total":N,"page":1,"per_page":20,"results":[...]}

# 5. Search with filters
curl "http://localhost:3000/api/v0/search?q=harry+potter&complete=true&min_words=10000"

# 6. List available tags
curl "http://localhost:3000/api/v0/tags"

# Expected: {"err":0,"tags":[...]}

# 7. Submit a tag (POST)
curl -X POST http://localhost:3000/api/v0/tags/submit \
  -H "Content-Type: application/json" \
  -d '{"url_id":"abc123","tag_name":"Fluff","tag_type_id":4}'

# 8. Download a cached EPUB
curl -O http://localhost:3000/cache/epub/abc123?h=def456

# 9. Test error handling — no query
curl "http://localhost:3000/api/v0/epub"

# Expected: {"err":-1,"msg":"no query"}

# 10. Test error handling — invalid URL
curl "http://localhost:3000/api/v0/epub?q=not-a-url"

# Expected: {"err":-5,"msg":"unsupported URL"}
```

Each curl command tests a different aspect of the system. If they all return the expected responses, your stack is working end-to-end.

### Interpreting curl Output

When you run `curl` against the API, the response is JSON. Let's look at what a successful export looks like:

```json
{
  "err": 0,
  "q": "https://archiveofourown.org/works/123456",
  "fixits": [],
  "info": "My Story by Author\n50000 words in 10 chapters\nStatus: complete\nUpdated: 2024-01-15 10:30:00 - 45 days ago\n",
  "url_id": "abc123def456",
  "slug": "My_Story-abc123def456",
  "meta": {
    "id": "abc123def456",
    "title": "My Story",
    "author": "Author",
    "chapters": 10,
    "words": 50000,
    "description": "<p>A great story</p>",
    "status": "complete",
    "source": "https://archiveofourown.org/works/123456",
    "created": "2024-01-01T00:00:00Z",
    "updated": "2024-01-15T10:30:00Z",
    "source_id": 1,
    "author_id": 2
  },
  "epub_url": "/cache/epub/abc123def456?h=hash123",
  "html_url": "/cache/html/abc123def456?h=hash456",
  "notes": []
}
```

And an error response:

```json
{
  "err": -1,
  "msg": "no query",
  "q": ""
}
```

Notice the consistent structure: `err` is always present, negative means error, `q` echoes back the original query. This consistency makes the frontend's job easy — it always checks `err` first, then accesses the data fields.

The `fixits` array is a placeholder for future auto-correction suggestions (like fixing malformed URLs). The `notes` array carries informational messages (like greylisting warnings).

🧪 **Try It Yourself:** Set up a local instance of FicHub (or use the production URL), then run through all 10 curl commands. Pay special attention to the error cases (#9 and #10) — a robust system handles errors gracefully and returns helpful messages.

## The Deployment Checklist

Before declaring your system production-ready, verify every item:

- [ ] **Backend compiles without warnings:** `cargo build --release` completes clean — no warnings, no errors
- [ ] **Backend tests pass:** `cargo test` — all unit tests green
- [ ] **Integration tests pass:** `cargo test --test integration` — API smoke tests green
- [ ] **Database migrations run:** `sqlx migrate run` succeeds against your production database
- [ ] **Frontend builds:** `npm run build` in the `frontend/` directory
- [ ] **Frontend tests pass:** `npm run test` — all 51 tests green
- [ ] **Server starts:** The Axum server binds to the correct port without errors
- [ ] **CORS configured:** The frontend can make API requests to the backend without CORS errors
- [ ] **Cache directory exists:** The server has read/write access to the cache directory
- [ ] **Static files served:** The frontend's build output is served correctly by nginx or the server
- [ ] **SSL/TLS:** HTTPS works (either through nginx reverse proxy or the server itself)
- [ ] **Logging configured:** You can see server logs for debugging (tracing + env_logger)
- [ ] **Error responses are consistent:** All error cases return proper JSON with `err` codes
- [ ] **Database connection pool:** The pool size is appropriate for your expected load
- [ ] **Rate limiting:** If needed, rate limiting is configured to prevent abuse
- [ ] **Monitoring:** Basic health checks and error tracking are in place

This checklist isn't just busywork — each item has prevented real production incidents. The "compiles without warnings" check catches unused variables that might indicate logic errors. The "CORS configured" check prevents the "it works locally but not in production" problem. The "error responses" check ensures users get helpful feedback instead of cryptic 500 errors.

## What We Built Together

Let's take a moment to appreciate what you've created. Across eight parts of this book, you built:

1. **A web scraper** that can parse fanfiction from multiple sites (Archive of Our Own, FanFiction.net, XenForo forums). Each scraper understands the site's HTML structure, extracts metadata and chapter content, and handles errors gracefully.

2. **An EPUB generator** that produces clean, well-formatted ebooks from scraped content. Proper metadata, chapter navigation, cover images — everything a good ebook needs.

3. **A full REST API** with endpoints for exporting (EPUB generation), searching (full-text search with filters), tagging (community-driven content classification), and recommendations (suggest similar fics).

4. **A PostgreSQL database** with migrations, connection pooling, and async queries via SQLx. The schema supports fic metadata, export logs, blacklists, version bumps, request logging, and a tag system.

5. **A caching layer** with disk storage, hash-based cache keys, and deduplication. Once a fic is exported, it's served from cache — no re-scraping needed.

6. **A tagging system** with community voting and moderation. Users can submit tags, vote on them, and flag inappropriate ones. Greylisting shows metadata without download links.

7. **A recommendation engine** that suggests similar fics based on shared tags, authors, and fandoms.

8. **A modern SvelteKit frontend** with dark mode, responsive design, a clean UI, search with a custom syntax parser, and a download interface with real-time feedback.

9. **A comprehensive test suite** — 8 Rust backend tests (unit + integration), 51 frontend JavaScript tests, covering pure functions, API clients, search builders, syntax parsing, and component rendering.

That's not a toy project. That's a production-grade platform built with the same tools and patterns used by companies like Cloudflare (Rust + Axum), Vercel (SvelteKit), and Discord (Rust backend).

### What Makes This Stack Special

Let's appreciate the technical choices:

**Rust for the backend** — You get memory safety without garbage collection, a type system that catches bugs at compile time, and performance that rivals C++. The `?` operator makes error handling ergonomic, and `async/await` with Tokio makes concurrent I/O straightforward. The result is a backend that's fast, reliable, and maintainable.

**SQLx for the database** — Compile-time checked SQL queries mean you can't write a query with a wrong column name. If the migration adds a column, your Rust code knows about it. If you mistype a table name, the build fails. This is a level of safety that most frameworks don't offer.

**Axum for the HTTP layer** — Built on Tower (middleware), Tokio (async runtime), and Hyper (HTTP), Axum gives you a modern, performant web framework without the bloat. The extractor pattern (Query, State, Path) makes request handling clean and composable.

**SvelteKit for the frontend** — No virtual DOM, no heavy runtime. Svelte compiles your components to efficient JavaScript that updates the DOM directly. The result is a frontend that's fast to load and fast to interact with.

**Vitest for frontend testing** — Works natively with Vite, runs tests in the same process, and provides the familiar Jest API. Your tests run in milliseconds, not seconds.

**PostgreSQL for the database** — Battle-tested, feature-rich, and handles everything from full-text search to JSON storage to materialized views. FicHub uses it for metadata storage, export logging, blacklisting, tagging, and search.

Each choice was deliberate. Each one contributes to a system that's fast, reliable, and pleasant to work on.

## Ideas for Improvement

Every project can grow. Here are directions you might take FicHub next:

### Adding More Fanfiction Sites

The scraper architecture is designed for extensibility. Adding a new site means:
1. Create a new file: `src/scrape/wattpad.rs`
2. Implement the `Scraper` trait: `lookup()`, `fetch_chapters()`, `extract_tags()`
3. Register it in the scraper registry
4. Add tests

Popular targets: **Wattpad** (huge user base, different HTML structure), **RoyalRoad** (LitRPG/progression fantasy), **Questionable Questing** (adult fiction), **SpaceBattles** (different from Sufficient Velocity, despite similar platforms).

Each new scraper is an opportunity to improve the `Scraper` trait — maybe adding better error recovery, rate limiting, or content normalization.

### User Accounts and Bookmarks

Add authentication (JWT or session-based) and let users:
- Save their reading history
- Bookmark fics for later
- Create reading lists
- Track which fics they've downloaded
- Rate and review fics

This would require a `users` table, session management, password hashing (argon2), and protected API routes. It's a significant feature but follows well-established patterns.

### A Mobile App

The API is already REST-based, which makes it trivial to build a mobile client. Consider:
- **React Native** — If you're comfortable with JavaScript (you already know React concepts from Svelte)
- **Flutter** — If you want a single codebase for iOS and Android with good performance
- **Tauri Mobile** — If you want to reuse your existing Rust code as the core

A mobile app could add offline reading (cache EPUBs locally), push notifications (when bookmarked fics update), and native sharing.

### Community Features

The tag system is already a step toward community features. You could add:
- **Comments** on fics (threaded, with moderation)
- **Ratings** (star ratings or upvote/downvote)
- **Reading lists** that are shareable
- **Collections** — curated lists of fics by theme (e.g., "Best Harry Potter Crossovers")
- **Discussion forums** (maybe using the XenForo scraper's knowledge)

### Contributing to Open Source

FicHub is built on open-source technologies. Consider:
- **Contributing to Axum** — Writing middleware, improving documentation, fixing bugs
- **Contributing to SvelteKit** — Bug fixes, new features, documentation
- **Contributing to SQLx** — New database adapters, performance improvements
- **Contributing to the epub crate** — Better EPUB 3 support, metadata handling
- **Creating your own libraries** — Extract reusable patterns from FicHub into standalone crates

Open source contribution is one of the best ways to grow as a developer. You'll learn from experienced maintainers, get code review from experts, and build a public track record.

## Learning Resources

You've learned a lot in this book, but there's always more. Here are the best resources for going deeper:

### Rust
- **The Rust Book** (doc.rust-lang.org/book) — The official guide. Reread it now that you have context — concepts that were abstract will make concrete sense.
- **Rust by Example** (doc.rust-lang.org/rust-by-example) — Learn by doing. Great for understanding patterns you've seen but not fully grasped.
- **SQLx Documentation** (docs.rs/sqlx) — Deep dive into async database queries. Pay attention to the "Compile-time checked queries" feature.
- **Axum Examples** (github.com/tokio-rs/axum/tree/main/examples) — Real-world API patterns. Study the "TODO" and "channels" examples.
- **"Zero to Production in Rust"** by Luca Palmieri — A full book on building production APIs. Covers testing, error handling, and deployment.
- **Rust Design Patterns** (rust-unofficial.github.io/patterns) — Common patterns for idiomatic Rust code.

### Svelte/SvelteKit
- **SvelteKit Documentation** (kit.svelte.dev) — The official guide. Focus on the "Concepts" section for understanding, not just the "Reference" for looking things up.
- **Svelte Tutorial** (svelte.dev/tutorial) — Interactive lessons. Great for understanding Svelte 5 runes deeply.
- **Joy of Code** (joyofcode.com) — Excellent Svelte tutorials with real-world examples.

### General Software Engineering
- **"Designing Data-Intensive Applications"** by Martin Kleppmann — Understanding the distributed systems you're building on top of. This book will change how you think about databases, replication, and consistency.
- **"The Pragmatic Programmer"** by Hunt & Thomas — Timeless software craftsmanship advice. Chapter on "Don't Repeat Yourself" alone is worth the read.
- **"Refactoring"** by Martin Fowler — How to improve code structure without changing behavior. Essential for maintaining a growing codebase.
- **"The Clean Coder"** by Robert Martin — Professionalism in software development. How to be the kind of developer teams want to hire.
- **"Working Effectively with Legacy Code"** by Michael Feathers — How to add tests and make changes to code that wasn't designed for testing. You'll face this eventually.

## The Power of Building Things Yourself

Here's a secret about software development: the best way to learn is to build something you actually want to use. Not a tutorial project. Not a homework assignment. Something that *you* would open every day and find useful.

You didn't build FicHub because someone told you to. You built it because you wanted a better way to download and organize fanfiction. That personal motivation carried you through every obstacle:

- The CORS errors that took three hours to debug
- The database migration that broke everything
- The scraper that stopped working when the site changed its HTML
- The EPUB generator that produced malformed files
- The search parser that couldn't handle Unicode
- The deployment that worked locally but failed on the server

Every one of those problems taught you something. Not just about the technology, but about the *process* of building software: how to read error messages, how to search for solutions, how to break a big problem into small pieces, and how to keep going when nothing works.

Here's what you built, step by step:

- Setting up a Rust project from scratch
- Writing async code with Tokio
- Managing a PostgreSQL database with SQLx
- Building a scraping engine with multiple site parsers
- Generating EPUBs from raw HTML content
- Creating a REST API with Axum
- Deploying to a real server with nginx
- Building a SvelteKit frontend with reactive components
- Writing tests for both backend and frontend

That's not textbook knowledge. That's *experienced* knowledge. You know how these pieces fit together because you put them together yourself. You know the pitfalls — the CORS issues, the database connection timeouts, the scraper parsing failures — because you encountered them and solved them.

Every time you run `cargo test` and all the tests pass, you feel a small rush of satisfaction. That's the feeling of craftsmanship. That's what building things gives you — not just the knowledge of how to build, but the confidence that you *can* build.

## Congratulations: You Are Now a Full-Stack Developer!

Look at what you can do now:

- **Backend:** Write Rust code that compiles fast, runs fast, and handles errors gracefully. You understand ownership, lifetimes, async/await, and the type system well enough to build real applications. You can write a web server from scratch in under 100 lines of code.

- **Database:** Design schemas, write migrations, and query data with type-safe SQL. You know about connection pooling, transactions, and the difference between `SELECT` and `INSERT ... ON CONFLICT`. You can debug slow queries and add indexes where they matter.

- **API:** Build REST endpoints that return consistent JSON responses. You know about query parameters, path parameters, request bodies, and error response conventions. You understand how to version an API and how to design responses that frontend developers will thank you for.

- **Scraping:** Parse HTML from real websites and extract structured data. You know about CSS selectors, attribute parsing, and handling different page layouts. You understand rate limiting, user agents, and the ethics of web scraping.

- **Export:** Generate EPUBs from raw content. You understand the EPUB format, metadata handling, and file organization. You can also generate HTML bundles for browser reading.

- **Frontend:** Build reactive UIs with SvelteKit, including forms, async data loading, responsive design, and dark mode. You understand component composition, state management, and client-side routing. You know how to build interfaces that users actually enjoy using.

- **Testing:** Write unit tests, integration tests, and end-to-end tests. You know how to mock dependencies, test async code, and verify both happy paths and error cases. You understand the testing pyramid and can write tests that actually provide value.

- **Deployment:** Ship a real application to a real server. You know about nginx configuration, systemd services, database management, and SSL certificates. You can debug production issues from server logs.

That's the full stack. You're not a "frontend developer" or a "backend developer" — you're a **full-stack developer** who can build, test, and deploy complete web applications.

The tools you've learned — Rust, SvelteKit, PostgreSQL, Axum, Vitest — are the same tools used by companies like Cloudflare, Discord, and Vercel. You're not using toy technologies. You're using production-grade tools that scale to millions of users.

So what's next? Build something else. Find another problem that annoys you and solve it with code. Take what you've learned and apply it to a new domain. The skills transfer. The patterns repeat. The confidence compounds.

And if you ever get stuck, remember: the community is here. The Rust Discord, the Svelte Discord, Stack Overflow, GitHub issues — people love helping people who are building things.

You started this book with a question: "How do I build a fanfiction download platform?" You end it with a complete, tested, deployed application. That's remarkable. That's not something most people accomplish.

Now go build something amazing. 🚀

---

# Summary

This final part brought our FicHub book to a close by covering the essential topic of testing — both backend and frontend — and stepping back to appreciate the full system we've built.

**Chapter 34: Backend Testing (cargo test)** covered:
- Why testing matters: regression prevention, documentation, confident refactoring, faster development
- Rust's built-in test framework: `#[cfg(test)]` modules and `#[test]` attributes (zero production cost)
- The three assertion macros: `assert!` for boolean checks, `assert_eq!` for equality, `assert_ne!` for inequality
- Testing the slug generator with 10+ edge cases: special chars, unicode, empty titles, consecutive underscores
- Testing the info string builder with the struct update syntax (`..make_test_meta()`) for focused tests
- Testing the meta JSON builder: verifying the contract (field names, types, formats) rather than implementation
- Testing error handling through API smoke tests: verifying error codes and messages
- The four-module integration test architecture:
  - DB tests: temporary schemas, global mutex, migrations, cleanup (marked `#[ignore]`)
  - API smoke tests: mock Axum routers with `tower::ServiceExt::oneshot`
  - Export logic tests: version computation, format registry
  - Tag API tests: CRUD operations, search endpoint
- Running tests with `cargo test` and its many flags
- Test coverage with `cargo-tarpaulin` (text and HTML reports)
- Designing testable code through separation of concerns and dependency injection

**Chapter 35: Frontend Testing (vitest)** covered:
- Vitest as a fast, Vite-native test runner (compatible with Jest API)
- Setting up jsdom, @testing-library/svelte, and the test environment
- Mocking `globalThis.fetch` with `vi.fn()` for API client tests
- Dynamic imports (`await importClient()`) to ensure mocks are picked up
- Testing `fetchExport` success and error paths, `submitSuggestion` POST body, `castVote` vote values
- Testing `buildSearchQuery` with all parameter types (q, min_words, complete, sort, source, tags, pagination)
- Testing `defaultFilters` correct defaults and option constants
- Testing utility functions: `formatWords` (thousands separators), `detectSite` (URL pattern matching), `stripHtml` (tag removal + entity decoding), `relativeTime` (fuzzy time matching), `cacheUrl` (path construction)
- The 28-test syntax parser suite: bare words, key:value pairs, aliases, exclusions, ranges, booleans, site shortcuts, dates, complex queries, empty strings
- Component testing: `render()`, `screen` queries, `fireEvent`, `waitFor` for async updates
- Running frontend tests with `npx vitest`, watch mode, and coverage

**Chapter 36: Full-Stack Integration and What's Next** covered:
- The complete project structure (Rust backend + SvelteKit frontend)
- The 9-step request flow: browser → API client → Axum → scraper → database → cache → EPUB → response → download
- End-to-end curl testing commands for all major features
- The deployment checklist (16 items covering compilation, tests, migrations, CORS, SSL, monitoring)
- What we built: scraper, EPUB generator, REST API, database, cache, tagging, recommendations, frontend, test suite
- Ideas for improvement: more sites, user accounts, mobile app, community features, open source contributions
- Learning resources for Rust, Svelte, and general software engineering
- The value of building real projects and the transition from learner to builder

---

# The Last 500 Words

Testing is not a chore — it's a gift you give your future self. Every test you write is a promise: "I will never have to debug this particular bug twice." Every passing test is a green light that says, "This part of the system works, and I can build on top of it with confidence."

When you first started this book, the idea of building a full-stack web application from scratch might have felt overwhelming. Database migrations, HTTP servers, web scraping, EPUB generation, SvelteKit frontends, integration tests — each piece is complex on its own. But here's what you discovered along the way: each piece is also *simple* when you take it one step at a time.

The slug generator is just string manipulation. The EPUB generator is just HTML wrapped in XML. The API is just a function that takes a request and returns a response. The frontend is just a form that calls an API and displays the result. The tests are just functions that call other functions and check the answers.

Complexity is just simplicity, stacked.

You've also learned something that no tutorial can teach: **the taste of a real project**. Real projects have weird edge cases. Real projects have to handle errors gracefully. Real projects need caching strategies and database optimizations and deployment pipelines. Real projects need tests — lots of them — because real users will find every bug you leave behind.

You've tasted that reality. You've dealt with the Unicode slug edge case. You've debugged why the EPUB metadata was missing the author. You've figured out why the Svelte component wasn't re-rendering after the API call. You've written 28 tests for a search syntax parser because every combination of keywords could break in a different way.

And you fixed all of it. You pushed through, iterated, and shipped a working application.

That's the real skill. Not knowing every framework or memorizing every API. The real skill is knowing that when something breaks, you can figure it out. You can read the error message, trace the code path, write a test to reproduce the bug, fix it, and verify the fix. That loop — break, diagnose, fix, verify — is the core of software engineering.

You now own that loop. Use it well.

The fanfiction community has a saying: "Don't like, don't read." The software community has a similar energy: "Don't like, don't use — or build something better." You chose the second option. You looked at existing tools, saw room for improvement, and built your own.

That's what developers do. Not just consume technology, but *create* it. You have the skills to build web applications, the testing discipline to ensure they work, and the deployment knowledge to put them in the world.

So here's your final assignment: **ship something**. Not a tutorial project, not a copy of someone else's app — something that matters to you. Something you'd use every day. Something that solves a problem you care about.

You have every tool you need. The rest is just building.

Happy coding. 🚀
