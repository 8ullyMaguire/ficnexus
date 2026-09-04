# Supplementary Content: Comprehensive Rust Tutorial

---

# Rust for FicHub Developers

## Variables and Binding

In Rust, `let` creates a binding. The binding is immutable by default:

```rust
let name = "FicHub";       // immutable string slice
let version = 1;            // immutable integer
let pi = 3.14;              // immutable float
```

To make a variable mutable, add `mut`:

```rust
let mut counter = 0;
counter += 1;               // OK — variable is mutable
```

Why immutable by default? Because immutability makes your code safer. When you see `let x = 5`, you know `x` will always be `5`. No function, no thread, no other code can change it. This eliminates an entire class of bugs.

Shadowing is when you re-declare a variable with the same name:

```rust
let x = 5;
let x = x + 1;     // x is now 6
let x = x * 2;     // x is now 12
```

Shadowing creates a new variable that happens to have the same name. This is useful for transforming values:

```rust
let name = "  FicHub  ";
let name = name.trim();    // "FicHub"
```

Type annotations are optional when Rust can infer the type:

```rust
let x = 5;              // Rust infers i32
let y = 5.0;            // Rust infers f64
let z: i64 = 5;         // Explicit annotation
let words: Vec<i32> = vec![1, 2, 3];  // Explicit annotation needed for collections
```

## Functions

Functions are defined with `fn`:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b      // Last expression (no semicolon) is the return value
}
```

Key rules:
- Parameters must have type annotations
- Return type comes after `->`
- The last expression (without semicolon) is the return value
- You can use `return` explicitly, but idiomatic Rust omits it

Functions can be public (accessible from other modules) or private:

```rust
pub fn public_function() -> String { "visible everywhere".to_string() }
fn private_function() -> String { "only in this module".to_string() }
```

## Closures

Closures are anonymous functions:

```rust
let add_one = |x: i32| -> i32 { x + 1 };
let result = add_one(5);   // 6

// If the body is a single expression, omit braces:
let add_one = |x| x + 1;

// Closures can capture variables from their environment:
let greeting = "Hello";
let greet = |name| format!("{}, {}!", greeting, name);
println!("{}", greet("World"));  // "Hello, World!"
```

Closures are used extensively with iterators:

```rust
let numbers = vec![1, 2, 3, 4, 5];

// Map: transform each element
let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();

// Filter: keep elements matching a predicate
let evens: Vec<&i32> = numbers.iter().filter(|x| *x % 2 == 0).collect();

// Fold: accumulate a single value
let sum: i32 = numbers.iter().fold(0, |acc, x| acc + x);
```

## Structs

Structs create custom types with named fields:

```rust
struct Person {
    name: String,
    age: u32,
    email: String,
}

let person = Person {
    name: "Alice".to_string(),
    age: 30,
    email: "alice@example.com".to_string(),
};

// Access fields with dot notation
println!("{} is {} years old", person.name, person.age);

// Mutable struct fields
let mut person = person;
person.age = 31;
```

Struct methods are defined in `impl` blocks:

```rust
impl Person {
    // Associated function (like a constructor)
    fn new(name: &str, age: u32, email: &str) -> Self {
        Person {
            name: name.to_string(),
            age,
            email: email.to_string(),
        }
    }

    // Method (takes &self)
    fn introduction(&self) -> String {
        format!("Hi, I'm {} and I'm {} years old.", self.name, self.age)
    }

    // Mutable method (takes &mut self)
    fn have_birthday(&mut self) {
        self.age += 1;
    }
}

let mut person = Person::new("Alice", 30, "alice@example.com");
println!("{}", person.introduction());
person.have_birthday();
```

## Enums

Enums represent a value that can be one of several variants:

```rust
enum Direction {
    North,
    South,
    East,
    West,
}

let dir = Direction::North;
```

Enums can carry data with each variant:

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    Color(u8, u8, u8),
}

let msg = Message::Write("hello".to_string());
let msg = Message::Color(255, 0, 0);
let msg = Message::Move { x: 10, y: 20 };
```

Pattern matching with `match`:

```rust
match msg {
    Message::Quit => println!("Quitting"),
    Message::Move { x, y } => println!("Moving to ({}, {})", x, y),
    Message::Write(text) => println!("Writing: {}", text),
    Message::Color(r, g, b) => println!("Color: #{:02x}{:02x}{:02x}", r, g, b),
}
```

`match` must be exhaustive — every variant must be handled. The `_` pattern catches anything not explicitly matched:

```rust
match msg {
    Message::Quit => println!("Quitting"),
    _ => println!("Something else"),
}
```

## Option and Result

Rust doesn't have null. Instead, it has `Option`:

```rust
fn find_user(id: u32) -> Option<String> {
    if id == 1 {
        Some("Alice".to_string())
    } else {
        None
    }
}

// Handling Option
match find_user(1) {
    Some(name) => println!("Found: {}", name),
    None => println!("User not found"),
}

// Shorthand
if let Some(name) = find_user(1) {
    println!("Found: {}", name);
}

// Chaining
let upper = find_user(1).map(|name| name.to_uppercase());
```

`Result` represents success or failure:

```rust
fn parse_number(s: &str) -> Result<i32, String> {
    s.parse::<i32>().map_err(|e| format!("Parse error: {}", e))
}

// Handling Result
match parse_number("42") {
    Ok(n) => println!("Parsed: {}", n),
    Err(e) => println!("Error: {}", e),
}

// The ? operator propagates errors
fn calculate() -> Result<i32, String> {
    let x = parse_number("42")?;
    let y = parse_number("10")?;
    Ok(x + y)
}
```

## Traits

Traits define shared behavior:

```rust
trait Summary {
    fn summarize(&self) -> String;
}

struct Article {
    title: String,
    author: String,
    content: String,
}

impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{} by {}", self.title, self.author)
    }
}

// Trait objects for dynamic dispatch
fn print_summary(item: &dyn Summary) {
    println!("{}", item.summarize());
}

// Generic functions with trait bounds
fn summarize_all(items: &[&dyn Summary]) {
    for item in items {
        println!("{}", item.summarize());
    }
}
```

## Ownership and Borrowing

Every value has exactly one owner. When the owner goes out of scope, the value is dropped:

```rust
{
    let s1 = String::from("hello");
    let s2 = s1;      // Ownership moves to s2
    // s1 is no longer valid
    println!("{}", s2);
}   // s2 is dropped here
```

Borrowing lets you use a value without taking ownership:

```rust
fn print_length(s: &String) {    // Borrows s
    println!("Length: {}", s.len());
}

let s = String::from("hello");
print_length(&s);                 // Pass a reference
println!("{}", s);                // s is still valid
```

Two kinds of borrows:
- `&T` — Shared borrow (many readers)
- `&mut T` — Mutable borrow (one writer)

```rust
fn push_world(s: &mut String) {
    s.push_str(", world!");
}

let mut s = String::from("hello");
push_world(&mut s);
println!("{}", s);   // "hello, world!"
```

## Enums with Data — AppError

FicHub's error type demonstrates enums with data:

```rust
enum AppError {
    BadRequest(i32, String),    // Code + message
    RateLimited(u64),           // Retry-after seconds
    NotFound(String),           // What wasn't found
    Internal(String),           // Error message
    ScrapeError(String),        // Scraping error
    ExportError(String),        // Export error
    Database(String),           // Database error
    CacheError(String),         // Cache error
}
```

Each variant can have different data. `BadRequest` has a code and message. `RateLimited` has just a number. This is much more expressive than a generic "error code + message" approach.

## Iterators

Rust iterators are lazy — they don't do anything until consumed:

```rust
let numbers = vec![1, 2, 3, 4, 5];

// Lazy — does nothing yet
let doubled = numbers.iter().map(|x| x * 2);

// Consuming — actually runs the iterator
let result: Vec<i32> = doubled.collect();
```

Common iterator methods:

```rust
let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// map: transform each element
let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();

// filter: keep matching elements
let evens: Vec<&i32> = numbers.iter().filter(|x| *x % 2 == 0).collect();

// fold: accumulate a value
let sum: i32 = numbers.iter().fold(0, |acc, x| acc + x);

// any: check if any element matches
let has_even = numbers.iter().any(|x| x % 2 == 0);

// find: find first matching element
let first_even = numbers.iter().find(|x| x % 2 == 0);

// count: count matching elements
let even_count = numbers.iter().filter(|x| x % 2 == 0).count();

// take and skip
let first_three: Vec<&i32> = numbers.iter().take(3).collect();
let skip_three: Vec<&i32> = numbers.iter().skip(3).collect();

// chain: combine two iterators
let a = vec![1, 2];
let b = vec![3, 4];
let combined: Vec<&i32> = a.iter().chain(b.iter()).collect();
```

## Concurrency with Threads

Rust's ownership system prevents data races at compile time:

```rust
use std::thread;

let mut handles = vec![];

for i in 0..10 {
    let handle = thread::spawn(move || {
        // `move` takes ownership of `i`
        println!("Thread {}: Hello!", i);
    });
    handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}
```

Shared state with `Arc` (atomic reference counting):

```rust
use std::sync::Arc;
use std::thread;

let counter = Arc::new(std::sync::atomic::AtomicI64::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    let handle = thread::spawn(move || {
        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    });
    handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}

println!("Counter: {}", counter.load(std::sync::atomic::Ordering::SeqCst));
```

## Error Handling Best Practices

1. **Use `?` for error propagation** — Don't unwrap in production code
2. **Create custom error types** — Use enums, not strings
3. **Implement `From` for automatic conversion** — Makes `?` work seamlessly
4. **Log errors at the handler level** — Don't duplicate logging
5. **Return user-friendly messages** — Don't expose internal details

```rust
// Bad: unwrap everywhere
let config = Config::from_env();  // Panics if env vars missing
let pool = db::init_pool(&url).await.unwrap();  // Panics on connection error

// Good: propagate errors
let config = Config::from_env();  // Panics are OK for required config
let pool = db::init_pool(&url).await?;  // Error propagated to caller
```

## Summary

This Rust tutorial covered the language features that FicHub uses most: variables, functions, structs, enums, Option, Result, traits, ownership, borrowing, iterators, and concurrency. Master these concepts and you'll be able to read and modify any part of the FicHub codebase.

---

# Extended Guide: The reqwest HTTP Client

## Building a Client

```rust
let client = reqwest::Client::builder()
    .user_agent("fichub.net/0.1.0")          // Identify ourselves
    .timeout(Duration::from_secs(30))         // Request timeout
    .connect_timeout(Duration::from_secs(10)) // Connection timeout
    .pool_max_idle_per_host(10)               // Connection pooling
    .build()?;
```

## Making Requests

```rust
// GET request
let response = client.get(url).send().await?;

// GET with query parameters
let response = client.get(url)
    .query(&[("key", "value"), ("page", "1")])
    .send().await?;

// POST with JSON body
let response = client.post(url)
    .json(&json!({"name": "FicHub", "version": 1}))
    .send().await?;

// POST with form data
let response = client.post(url)
    .form(&[("username", "user"), ("password", "pass")])
    .send().await?;

// Custom headers
let response = client.get(url)
    .header("User-Agent", "fichub.net/0.1.0")
    .header("Accept", "text/html")
    .send().await?;
```

## Handling Responses

```rust
// Check status
if response.status().is_success() {
    println!("Success!");
} else if response.status().is_client_error() {
    println!("Client error: {}", response.status());
} else if response.status().is_server_error() {
    println!("Server error: {}", response.status());
}

// Get response as text
let html = response.text().await?;

// Get response as JSON
let json: serde_json::Value = response.json().await?;

// Get response as bytes
let bytes = response.bytes().await?;

// Stream response (for large files)
let mut stream = response.bytes_stream();
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    // Process chunk...
}
```

## Connection Pooling

`reqwest::Client` automatically manages a connection pool:

```rust
// Reuse the client for multiple requests
let client = reqwest::Client::new();

// These share the same connection pool
let r1 = client.get("https://example.com/page1").send().await?;
let r2 = client.get("https://example.com/page2").send().await?;
let r3 = client.get("https://example.com/page3").send().await?;
```

The pool reuses TCP connections, avoiding the overhead of creating new connections for each request.

## Error Handling

```rust
match client.get(url).send().await {
    Ok(response) => {
        // Handle successful response
    }
    Err(e) => {
        if e.is_timeout() {
            println!("Request timed out");
        } else if e.is_connect() {
            println!("Connection failed: {}", e);
        } else if e.is_redirect() {
            println!("Too many redirects");
        } else {
            println!("Other error: {}", e);
        }
    }
}
```

## Summary

`reqwest` provides a ergonomic API for HTTP requests with automatic connection pooling, timeout handling, and streaming support. FicHub uses it for all communication with fanfiction sites.

---

# Extended Guide: HTML Parsing with the Scraper Crate

## The Basics

The `scraper` crate parses HTML and lets you query it with CSS selectors:

```rust
use scraper::{Html, Selector};

let html = r#"
<html>
<body>
    <h1 class="title">Hello World</h1>
    <p id="intro">Welcome to FicHub</p>
    <div class="content">
        <p>First paragraph</p>
        <p>Second paragraph</p>
    </div>
</body>
</html>"#;

let document = Html::parse_document(html);
```

## CSS Selectors

```rust
// Element selector
let sel = Selector::parse("h1").unwrap();

// Class selector
let sel = Selector::parse("h1.title").unwrap();

// ID selector
let sel = Selector::parse("#intro").unwrap();

// Attribute selector
let sel = Selector::parse("a[href]").unwrap();
let sel = Selector::parse("a[href='https://example.com']").unwrap();
let sel = Selector::parse("a[rel='author']").unwrap();

// Descendant selector
let sel = Selector::parse("div p").unwrap();

// Child selector
let sel = Selector::parse("div > p").unwrap();

// Multiple selectors (comma-separated)
let sel = Selector::parse("h1, h2, h3").unwrap();
```

## Extracting Data

```rust
// Find first matching element
let title = document
    .select(&Selector::parse("h1.title").unwrap())
    .next()
    .map(|el| el.text().collect::<String>())
    .unwrap_or_default();

// Find all matching elements
let paragraphs: Vec<String> = document
    .select(&Selector::parse("p").unwrap())
    .map(|el| el.text().collect::<String>())
    .collect();

// Get attribute value
let href = document
    .select(&Selector::parse("a[href]").unwrap())
    .next()
    .and_then(|el| el.value().attr("href"));

// Get inner HTML (preserves tags)
let content = document
    .select(&Selector::parse("div.content").unwrap())
    .next()
    .map(|el| el.inner_html());
```

## Text vs Inner HTML

```rust
// .text() — Extracts only text content, strips HTML tags
let text: String = element.text().collect();
// Result: "Hello World" (no HTML tags)

// .inner_html() — Returns the raw HTML inside the element
let html = element.inner_html();
// Result: "<p>Hello</p> <p>World</p>" (preserves tags)
```

Use `.text()` for metadata (title, author, word count).
Use `.inner_html()` for content that needs HTML formatting (chapter text).

## Handling Malformed HTML

The `scraper` crate is tolerant of malformed HTML:

```rust
let html = r#"
<html>
<body>
    <h1>Unclosed tag
    <p>Paragraph without closing tag
    <div>Div with <b>bold <i>and italic</b></i></div>
</body>
</html>"#;

let document = Html::parse_document(html);
// Parser handles the errors gracefully
```

## Performance Tips

1. **Parse once, query many** — Parse the HTML once, then run multiple selectors
2. **Cache selectors** — If you use the same selector repeatedly, parse it once
3. **Use specific selectors** — ID selectors are faster than class selectors
4. **Limit results** — Use `.next()` instead of `.collect()` when you only need the first match

## Summary

The `scraper` crate provides a clean, Rust-idiomatic API for parsing HTML and extracting data with CSS selectors. It handles malformed HTML gracefully and is the foundation of FicHub's scraping system.
