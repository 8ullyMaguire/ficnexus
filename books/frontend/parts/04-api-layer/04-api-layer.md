# Part 4: Building the API Layer

> *This is Part 4 of a 10-part book about building FicHub — a self-hosted fanfiction platform. In this part, we'll learn how our frontend talks to our Rust backend, and build the TypeScript code that makes it all work.*

---

# Chapter 14: Understanding REST APIs

Welcome back! In the previous parts, we built a Rust backend that scrapes fanfiction, generates EPUBs, and stores metadata in a database. We also set up a SvelteKit frontend with routes and pages. But there's a missing piece — how does the frontend *talk* to the backend?

The answer is something called an **API**. And in this chapter, we're going to understand exactly how it works.

## What Is an API?

API stands for **Application Programming Interface**. Think of it like a waiter at a restaurant. You (the frontend) don't go into the kitchen (the backend) and start grabbing food. Instead, you tell the waiter what you want, the waiter goes to the kitchen, and comes back with your food.

In tech terms, the frontend sends a *request* to the backend, the backend processes it, and sends back a *response*. That's all an API is — a way for two pieces of software to talk to each other.

FicHub's Rust backend runs on port 8004. When your SvelteKit frontend needs fanfiction metadata, it sends a request to `http://localhost:8004/api/v0/meta?q=https://archiveofourown.org/works/123456`, and the backend responds with JSON data about that story.

## What Is REST?

REST stands for **Representational State Transfer**. Don't worry about the fancy name — it's really just a set of rules for how web APIs should work. The term was coined by Roy Fielding in his 2000 doctoral dissertation, and since then it has become the dominant style for building web APIs.

REST is popular because it's **simple** and **scalable**. It works with any programming language, any database, and any client (browser, mobile app, command line tool). It's built on top of HTTP, which means it uses the same protocol that powers the web.

Here are the main rules:

1. **Use URLs as addresses** — Every piece of data has a URL. Like `/api/v0/epub` or `/api/v0/meta`.
2. **Use HTTP methods** — Use different verbs for different actions: GET for reading, POST for creating, PUT for updating, DELETE for removing.
3. **Return JSON** — Responses should be in JSON format (JavaScript Object Notation).
4. **Be stateless** — Each request should contain all the information needed. The server doesn't "remember" you between requests.

REST is the most common style of web API. When you hear someone say "REST API," they just mean "a web API that follows these rules."

## HTTP Methods: The Four Verbs

HTTP (HyperText Transfer Protocol) gives us several methods, but REST APIs mainly use four:

### GET — "Give me data"

GET is the most common method. It asks the server for data without changing anything. It's like reading a book — you're not modifying it, just looking at it.

```
GET /api/v0/meta?q=https://archiveofourown.org/works/123456
```

This says: "Hey server, please give me the metadata for this fanfiction story."

GET requests are **safe** — they don't change anything on the server. You can make the same GET request 100 times and get the same result every time (assuming the data hasn't changed). This also means GET requests can be **cached** by browsers and CDNs.

GET requests should never have a body. All the information goes in the URL and headers.

### POST — "Here's new data"

POST sends data *to* the server. It's usually used for creating something new or triggering an action.

```
POST /api/v0/recommendations/suggest
Content-Type: application/json

{
  "url_id": "abc123def456",
  "suggested_url": "https://fanfiction.net/s/789012/1/",
  "comment": "Similar themes and writing style"
}
```

This says: "I'd like to suggest a new recommendation. Here's the data."

POST requests are **not safe** — they change things on the server. That's why you should never bookmark a POST endpoint or refresh a page that made a POST request. The browser will warn you: "Are you sure you want to resubmit this form?"

In FicHub, POST is used for:
- Submitting recommendation suggestions
- Casting votes on suggestions

### PUT — "Update this"

PUT replaces or updates an existing piece of data. FicHub doesn't use PUT much in its v0 API, but you'll see it in more complex APIs:

```
PUT /api/v0/fics/abc123
Content-Type: application/json

{ "title": "Updated Title" }
```

PUT is **idempotent** — making the same PUT request multiple times should have the same effect as making it once. If you set a fic's title to "Updated Title" ten times, it's still "Updated Title" at the end.

### DELETE — "Remove this"

DELETE tells the server to remove something. Again, not used much in FicHub's v0 API, but common in other APIs:

```
DELETE /api/v0/bookmarks/456
```

DELETE is also idempotent — deleting something that's already deleted is a no-op.

### PATCH — "Partially update this"

There's actually a fifth method worth knowing: PATCH. While PUT replaces the entire resource, PATCH only updates specific fields:

```
PATCH /api/v0/fics/abc123
Content-Type: application/json

{ "title": "New Title" }
```

This only changes the `title` field, leaving everything else untouched. It's more efficient than PUT when you're updating a single field.

> **Watch Out!** Always be careful with DELETE requests. Once you delete something from a server, it might be gone forever!

## URLs as Addresses

Every API endpoint has a URL (Uniform Resource Locator). The URL tells the server *where* to find the data you want. Let's break down a FicHub URL:

```
https://fichub.net/api/v0/epub?q=https://archiveofourown.org/works/123456
│        │       │     │  │    │
│        │       │     │  │    └─ Query parameter
│        │       │     │  └────── Endpoint name
│        │       │     └───────── API version
│        │       └─────────────── API prefix
│        └─────────────────────── Domain
└──────────────────────────────── Protocol
```

The **API prefix** (`/api`) separates API endpoints from regular website pages. If FicHub also served a blog at `/blog`, the API at `/api` wouldn't conflict with it.

The **version** (`/v0`) lets us update the API without breaking old clients. When FicHub adds new features, it can create `/api/v1/` while keeping `/api/v0/` working. This is called **versioning**, and it's essential for maintaining backward compatibility.

The **endpoint** (`/epub`) identifies what resource or action we want. Each endpoint is like a specific "page" in the API documentation.

FicHub uses `/api/v0/` for its legacy API and will eventually have `/api/v1/` for the new Go backend. This gradual migration is common in growing projects — you don't rewrite everything at once.

### URL Encoding

URLs can only contain certain characters. Characters like spaces, `&`, `?`, and `=` have special meaning, so they need to be encoded:

| Character | Encoded | Meaning |
|-----------|---------|---------|
| Space | `%20` or `+` | Separator |
| `&` | `%26` | Parameter separator |
| `?` | `%3F` | Query string start |
| `=` | `%3D` | Key-value separator |
| `/` | `%2F` | Path separator |

For example, the URL `https://example.com/search?q=hello world` becomes `https://example.com/search?q=hello%20world` when properly encoded.

In JavaScript, use `encodeURIComponent()` to encode values:

```javascript
const query = encodeURIComponent('hello world')  // "hello%20world"
const url = `https://example.com/search?q=${query}`
```

> **Watch Out!** Never put user input directly into a URL without encoding it. If a user types `?foo=bar` as a search query, it could break the URL structure. Always use `encodeURIComponent()`!

## Request Headers: Extra Information

Headers are like sticky notes attached to your request. They tell the server extra things about what you're sending or what you expect back.

### Content-Type

This header tells the server what format your request body is in:

```
Content-Type: application/json
```

This means "the body of my request is JSON." You'll always send this header with POST and PUT requests.

### Accept

This header tells the server what format you want the response in:

```
Accept: application/json
```

This means "please send me JSON back." Most APIs default to JSON anyway, but it's good practice to be explicit.

### Authorization

If the server requires authentication, you include a token:

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

FicHub's v0 API doesn't require authentication, but the v1 API (and the `FichubClient` class) does use JWT tokens for logged-in features.

## Request Body: JSON Payloads

When you send data with a POST request, the data goes in the **body** (the payload). The body is formatted as JSON:

```json
{
  "url_id": "abc123def456",
  "suggested_url": "https://fanfiction.net/s/789012/1/",
  "comment": "Great similar fic!"
}
```

JSON is just a text format that looks like JavaScript objects:

- **Strings** are in quotes: `"hello"`
- **Numbers** are plain: `42` or `3.14`
- **Booleans** are `true` or `false`
- **Null** means "no value": `null`
- **Objects** use curly braces: `{"key": "value"}`
- **Arrays** use square brackets: `["item1", "item2"]`

Why JSON? Because it's **human-readable** (you can look at it in a text editor), **lightweight** (smaller than XML), and **easy to parse** in every programming language. JavaScript has built-in functions to convert JSON to objects and back:

```javascript
// String → Object (parsing)
const obj = JSON.parse('{"name": "Test Fic", "words": 50000}')
console.log(obj.name)  // "Test Fic"
console.log(obj.words) // 50000

// Object → String (serialization)
const json = JSON.stringify({ name: "Test Fic", words: 50000 })
console.log(json)  // '{"name":"Test Fic","words":50000}'
```

JSON also has strict rules:
- Property names must be in double quotes: `"name"` not `name`
- Strings must use double quotes: `"hello"` not `'hello'`
- No trailing commas: `{"a": 1}` not `{"a": 1,}`
- No comments: you can't add `// this is a comment` inside JSON

> **Watch Out!** One of the most common JavaScript errors is trying to `JSON.parse()` a response that isn't valid JSON. This happens when the server returns an HTML error page instead of JSON. Always check `response.ok` before parsing!

> **Try It Yourself!** Open your browser's developer console (F12 or Ctrl+Shift+J) and type `JSON.stringify({name: "test", count: 5})`. You'll see it convert a JavaScript object to a JSON string. Now try `JSON.parse('{"name": "test", "count": 5}')` to go the other way.
>
> **Challenge:** Create a JSON string for an array of three favorite books. Then parse it back and access the title of the second book. If you get `undefined`, check your quotes!

## Response Status Codes: How Did It Go?

Every API response comes with a **status code** — a three-digit number that tells you what happened. Here are the ones you'll see most often in FicHub:

### 200 — OK 🟢

Everything worked! The server found what you asked for and sent it back.

```
HTTP/1.1 200 OK
Content-Type: application/json

{
  "err": 0,
  "meta": { "title": "My Favorite Fic", "author": "SomeWriter" },
  "urls": { "epub": "/cache/epub/abc123?h=..." }
}
```

### 400 — Bad Request 🟡

You sent something wrong. Maybe a required parameter is missing, or the URL is invalid.

```
HTTP/1.1 400 Bad Request

{
  "err": -1,
  "msg": "no query"
}
```

### 404 — Not Found 🔴

The server couldn't find what you're looking for. In FicHub, this might mean the fanfiction story doesn't exist or the URL isn't supported.

### 429 — Too Many Requests 🟡

You're making requests too fast. FicHub has a rate limiter that uses Redis to track how many requests each IP makes. If you exceed the limit, the server says "slow down" and tells you how long to wait.

### 500 — Internal Server Error 🔴

Something broke on the server side. This isn't your fault — the server encountered an unexpected problem. In FicHub, the `AppError::Internal` variant returns this code.

### FicHub Error Codes

FicHub also has its own custom error codes in the JSON response body:

- `err: 0` — Success
- `err: -1` — Generic error (bad request, missing data)
- `err: -5` — Not found or unsupported URL
- `err: -6` — Scraping error (the upstream site had a problem)
- `err: -7` — Content is blacklisted
- `err: -10` — Blocked automated request
- `err: -429` — Rate limited

This two-layer error system (HTTP status + JSON error code) gives you fine-grained control. The HTTP status tells you the *category* of error, and the JSON error code tells you the *specific reason*.

> **Watch Out!** Always check for errors! A common beginner mistake is to assume every response will be successful. Network errors, rate limits, and invalid input will happen — your code needs to handle them gracefully.

## Query Parameters: Customizing Your Request

Query parameters let you customize what data you get. They appear after a `?` in the URL, and multiple parameters are separated by `&`:

```
/api/v0/epub?q=https://archiveofourown.org/works/123456
/api/v0/recommendations?url_id=abc123&n=10&site_domain=fanfiction.net
/api/v0/search?q=harry+potter&complete=true&sort=-words&page=1
```

The pattern is always `key=value`. Here's what each FicHub endpoint accepts:

### Export Endpoint: GET /api/v0/epub

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | URL of the fanfiction to export |
| `format` | string | No | Export format (epub, mobi, pdf) |
| `automated` | string | No | Set to "true" to test automated request blocking |

### Meta Endpoint: GET /api/v0/meta

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | URL of the fanfiction |

### Recommendations Endpoint: GET /api/v0/recommendations

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Either `q` or `url_id` | URL of the fic |
| `url_id` | string | Either `q` or `url_id` | Internal fic ID |
| `n` | number | No | Number of recommendations (default: 20, max: 100) |
| `site_domain` | string | No | Filter by source site |

### Votes Endpoint: GET /api/v0/recommendations/votes

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `url_id` | string | Yes | Fic to get suggestions for |

> **Try It Yourself!** If you have a local FicHub instance running, try opening these URLs in your browser:
> - `http://localhost:8004/api/v0/meta?q=https://archiveofourown.org/works/123456`
> - `http://localhost:8004/api/v0/recommendations?url_id=abc123&n=5`
>
> You'll see the JSON responses right in your browser! Browsers display JSON natively, making GET endpoints easy to test.

## The FicHub API: All Endpoints Explained

Let's put it all together. Here's a complete map of FicHub's v0 API. This is your cheat sheet — keep it handy when building the frontend.

### Core Endpoints

```
GET  /api/v0/epub?q=<url>          → Export EPUB (main feature)
GET  /api/v0/meta?q=<url>          → Get fic metadata
GET  /api/v0/remote                → Remote info (IP, port, is_automated)
```

The **export endpoint** is the star of the show. You give it a fanfiction URL, it scrapes the site, generates an EPUB file, caches it, and returns download links. This is FicHub's core feature.

The **meta endpoint** is like the export endpoint but lighter — it only returns metadata without generating a file.

### Recommendation Endpoints

```
GET  /api/v0/recommendations?url_id=&n=&site_domain=  → Get recommendations
POST /api/v0/recommendations/suggest                  → Submit a suggestion
POST /api/v0/recommendations/vote                     → Vote on a suggestion
GET  /api/v0/recommendations/votes                    → Get votes for a fic
```

These endpoints power the recommendation engine. Users can suggest similar fics and vote on other people's suggestions. The engine uses collaborative filtering — "people who liked this also liked that."

### Cache Endpoints

```
GET  /cache/{etype}/{url_id}/{fname}  → Download cached file with hash
GET  /cache/{etype}/{url_id}          → Download or trigger export
```

Once an EPUB is generated, it's cached on disk. These endpoints serve the cached files. The URL includes the export type (epub, mobi, pdf, html) and the file's hash for cache-busting.

The cache system uses a two-level directory tree: `{cache_dir}/{etype}/{url_id[:2]}/{url_id[2:]}/`. This prevents having too many files in a single directory, which would slow down the filesystem.

### Search Endpoint (v0)

```
GET  /api/v0/search?q=&include_tags=&exclude_tags=&complete=&sort=&page=
```

The search endpoint uses PostgreSQL's full-text search to find fics. It supports tag filtering, word count ranges, completion status, and pagination. This is the most powerful endpoint in the v0 API — we'll build an entire chapter around it in Chapter 18.

### API Design Notes

FicHub's API follows a few important patterns:

1. **Consistent error format** — All errors return `{ "err": <code>, "msg": "<message>" }`
2. **Consistent success format** — Most responses include `"err": 0` on success
3. **Query parameters for GET** — All filter/sort/pagination goes in the URL
4. **JSON body for POST** — Data is sent as JSON, not form-encoded
5. **URL encoding for special chars** — Always encode user input in URLs

## Testing APIs with curl

**curl** is a command-line tool for making HTTP requests. It's the developer's Swiss Army knife for testing APIs. You can use it from any terminal — Linux, macOS, or Windows (with Git Bash or WSL).

### Why Use curl?

When you're building an API client, you need to verify that the server returns what you expect. curl lets you:

1. **Test endpoints** — Make requests without writing any code
2. **Inspect responses** — See the exact JSON structure
3. **Debug issues** — Check status codes, headers, and error messages
4. **Document behavior** — Save curl commands as examples for other developers

### Making a GET Request

```bash
curl http://localhost:8004/api/v0/meta?q=https://archiveofourown.org/works/123456
```

This sends a GET request to the meta endpoint and prints the JSON response. The response appears directly in your terminal. By default, curl uses the GET method, so you don't need to specify `-X GET`.

If the server is running on a different port or host, adjust the URL accordingly. For example, on the Orange Pi deployment: `curl http://192.168.1.138:8004/api/v0/meta?q=...`

### Pretty-Printing JSON

Raw JSON is hard to read — it's all on one line with no indentation. Pipe it through `jq` for pretty formatting:

```bash
curl -s http://localhost:8004/api/v0/meta?q=https://archiveofourown.org/works/123456 | jq .
```

The `-s` flag (silent) suppresses curl's progress meter. The `jq .` command formats the JSON with indentation and colors.

If you don't have `jq`, you can use Python:

```bash
curl -s http://localhost:8004/api/v0/meta?q=https://archiveofourown.org/works/123456 | python3 -m json.tool
```

Both produce nicely formatted output:

```json
{
  "err": 0,
  "q": "https://archiveofourown.org/works/123456",
  "meta": {
    "id": "abc123def456",
    "title": "My Favorite Fic",
    "author": "SomeWriter",
    "words": 50000,
    "chapters": 10
  }
}
```

### Making a POST Request

POST requests send data in the request body:

```bash
curl -X POST http://localhost:8004/api/v0/recommendations/suggest \
  -H "Content-Type: application/json" \
  -d '{"url_id": "abc123", "suggested_url": "https://fanfiction.net/s/789012/1/"}'
```

The `-X POST` flag tells curl to use the POST method. The `-H` flag adds a header. The `-d` flag provides the request body.

Notice the `\` at the end of the first line — this is a line continuation character. It lets you split a long command across multiple lines for readability.

### Checking Response Headers

```bash
curl -I http://localhost:8004/api/v0/meta?q=https://archiveofourown.org/works/123456
```

The `-I` flag (capital I) shows only the response headers, including the status code:

```
HTTP/1.1 200 OK
content-type: application/json
content-length: 512
```

This is useful for checking if a request was successful without parsing the entire response body.

### Verbose Mode

For debugging, use the `-v` (verbose) flag to see the full request and response:

```bash
curl -v http://localhost:8004/api/v0/meta?q=https://archiveofourown.org/works/123456
```

This shows:
- The request method and URL
- All request headers
- The response status code
- All response headers
- The response body

It's like opening the hood of your car — you see everything that's happening under the surface.

### Saving Responses to a File

Sometimes you want to save a response for later analysis:

```bash
# Save to a file
curl -s http://localhost:8004/api/v0/meta?q=https://example.com > response.json

# Then inspect it
cat response.json | jq .
```

### Testing with Different HTTP Methods

```bash
# GET (default)
curl http://localhost:8004/api/v0/meta?q=https://example.com

# POST
curl -X POST -H "Content-Type: application/json" -d '{"key":"value"}' http://localhost:8004/api/v0/some-endpoint

# PUT
curl -X PUT -H "Content-Type: application/json" -d '{"key":"new-value"}' http://localhost:8004/api/v0/some-resource/123

# DELETE
curl -X DELETE http://localhost:8004/api/v0/some-resource/123
```

> **Try It Yourself!** If your FicHub instance is running, open a terminal and try:
> ```bash
> # Basic metadata request
> curl -s http://localhost:8004/api/v0/meta?q=https://archiveofourown.org/works/123456 | python3 -m json.tool
>
> # Pretty-print the meta response
> curl -s http://localhost:8004/api/v0/meta?q=https://archiveofourown.org/works/123456 | jq '.meta.title'
>
> # Check just the status code
> curl -s -o /dev/null -w "%{http_code}" http://localhost:8004/api/v0/meta?q=https://example.com
> ```
> You should see a JSON object with `err: 0` and metadata about the story. If you see `err: -5`, it means the URL isn't supported or the story wasn't found.
>
> **Bonus experiment:** Try accessing the endpoint with no query parameter. What error do you get?

## Reading API Documentation

Good API documentation is like a user manual for developers. FicHub's API documentation is embedded in the server itself — visiting `GET /api/` returns an HTML page listing all available endpoints. This is auto-generated from the Rust code, so it's always up to date.

But for the frontend developer, the real documentation is the **Rust source code**. Here's how to read it, even if you're not a Rust expert.

1. **Find the route** — Look in `src/routes/` for the handler file (e.g., `export.rs` for `/api/v0/epub`).
2. **Read the query struct** — The `ExportQuery` struct shows what parameters the endpoint accepts.
3. **Read the handler function** — The `epub_handler` function shows what the endpoint does and what it returns.
4. **Look at the JSON response** — The `json!({...})` macro shows the exact shape of the response.

For example, looking at `src/routes/export.rs`, we can see:

```rust
pub struct ExportQuery {
    pub q: Option<String>,        // The fic URL
    pub automated: Option<String>, // "true" to test blocking
    pub format: Option<String>,    // Export format
}
```

This tells us exactly what parameters the export endpoint accepts. The `Option<String>` means each parameter is optional (the server will handle missing values).

Then looking at the response:

```rust
Ok(Json(json!({
    "err": 0,
    "q": query,
    "fixits": [],
    "info": info_str,
    "url_id": meta.url_id,
    "slug": slug,
    "meta": build_meta_json(&meta),
    "hashes": hashes,
    "urls": urls,
    "epub_url": urls.get("epub"),
    // ... more fields
})))
```

This tells us exactly what fields will appear in the JSON response. We can use this to build our TypeScript types!

## Summary

In this chapter, you learned:

- **REST** is a set of rules for building web APIs — simple, scalable, and universal
- **HTTP methods** (GET, POST, PUT, DELETE, PATCH) indicate what action you want
- GET is **safe** and **idempotent** — it doesn't change anything
- POST is **unsafe** — it creates or modifies data
- PUT and DELETE are **idempotent** — repeating them has the same effect
- **URLs** are the addresses of API endpoints, with API prefix, version, and endpoint name
- **URL encoding** converts special characters to safe formats
- **Headers** carry extra information like content type and authentication
- **Request bodies** contain data you're sending (JSON format)
- **JSON** is the universal language of web APIs — lightweight, human-readable, and easy to parse
- **Status codes** tell you if your request succeeded or failed:
  - 200 = OK, 400 = Bad Request, 404 = Not Found, 429 = Rate Limited, 500 = Server Error
- **FicHub error codes** provide fine-grained control: 0 = success, -5 = not found, -6 = scrape error, -7 = blacklisted, -429 = rate limited
- **Query parameters** let you customize GET requests with `?key=value&key2=value2` syntax
- **curl** is a powerful command-line tool for testing APIs — use `-s` for silent, `| jq .` for pretty-printing, `-v` for verbose
- **Source code** is the ultimate API documentation — read the Rust handler to understand response shapes

In the next chapter, we'll take what we learned about API responses and turn it into TypeScript types that our frontend can use. Let's go!

---

# Chapter 15: Designing TypeScript Types from API Responses

In the last chapter, we learned how REST APIs work — HTTP methods, URLs, status codes, and JSON responses. Now it's time to take the next step: turning those JSON responses into **TypeScript types** that our frontend code can understand and use safely.

This is one of the most important skills in frontend development. When your types match your API responses exactly, you catch bugs at compile time instead of at runtime. It's like having a safety net that catches mistakes before they reach your users.

## Reading the Rust Backend Code

Before we can write TypeScript types, we need to understand *exactly* what shape the JSON responses have. The best way to do this is to read the Rust backend code. This might sound intimidating if you're not familiar with Rust, but don't worry — you don't need to understand every line. You just need to find the response shape.

### How to Find the Response Shape

There are three steps:

**Step 1: Find the route handler.** Look in `src/routes/` for the handler file. The export endpoint (`/api/v0/epub`) is in `export.rs`. The meta endpoint (`/api/v0/meta`) is in `meta.rs`. The recommendation endpoints are in `recommender/routes.rs`.

**Step 2: Find the `json!` macro.** Rust's `json!` macro creates JSON values. It looks like this:

```rust
Ok(Json(json!({
    "err": 0,
    "q": query,
    // ... more fields
})))
```

Each key-value pair in the `json!` block is a field in the response. The left side is the field name (what the frontend sees), and the right side is the value (what goes in that field).

**Step 3: Trace the values.** Some values come directly from the database or scraper. Others go through helper functions like `build_meta_json()`. Trace them to understand the type:

- `meta.url_id` → a `String` (comes from the `FicMetadata` struct)
- `meta.words` → an `i64` (a 64-bit integer)
- `meta.content_hash` → an `Option<String>` (either a string or null)
- `hashes` → a `HashMap<String, String>` (object with string keys and values)

This three-step process works for any Rust/Axum API. Let's apply it to the export endpoint.

Let's look at the export handler in `src/routes/export.rs`. The response is built with the `json!` macro:

```rust
Ok(Json(json!({
    "err": 0,
    "q": query,
    "fixits": [],
    "info": info_str,
    "url_id": meta.url_id,
    "slug": slug,
    "meta": build_meta_json(&meta),
    "hashes": hashes,
    "urls": urls,
    "epub_url": urls.get("epub"),
    "html_url": urls.get("html"),
    "mobi_url": urls.get("mobi"),
    "pdf_url": urls.get("pdf"),
    "notes": notes,
})))
```

From this, we can see the response has these fields:
- `err` — a number (0 for success, negative for errors)
- `q` — the original query URL
- `fixits` — an array (empty in practice)
- `info` — a human-readable string with fic info
- `url_id` — the unique identifier for this fic
- `slug` — a URL-friendly version of the title
- `meta` — an object with detailed metadata
- `hashes` — an object mapping format names to hash strings
- `urls` — an object mapping format names to download URLs
- `epub_url`, `html_url`, `mobi_url`, `pdf_url` — direct links (or null)
- `notes` — an array of warning/info messages

Now let's look at the `build_meta_json` function to understand the `meta` object:

```rust
pub fn build_meta_json(meta: &FicMetadata) -> Value {
    json!({
        "id": meta.url_id,
        "title": meta.title,
        "author": meta.author,
        "chapters": meta.chapters,
        "words": meta.words,
        "description": meta.desc,
        "status": meta.status,
        "source": meta.source,
        "created": chrono::DateTime::from_timestamp_millis(meta.published)
            .map(|d| d.to_rfc3339())
            .unwrap_or_default(),
        "updated": chrono::DateTime::from_timestamp_millis(meta.updated)
            .map(|d| d.to_rfc3339())
            .unwrap_or_default(),
        "extra_meta": meta.extra_meta,
        "raw_extended_meta": meta.raw_extended_meta,
        "author_url": meta.author_url,
        "author_local_id": meta.author_local_id,
        "source_id": meta.source_id,
        "author_id": meta.author_id,
    })
}
```

And the `FicMetadata` struct from `src/scrape/mod.rs`:

```rust
pub struct FicMetadata {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub desc: String,
    pub published: i64,
    pub updated: i64,
    pub status: String,
    pub source: String,
    pub source_id: i64,
    pub author_id: i64,
    pub author_url: String,
    pub author_local_id: String,
    pub content_hash: Option<String>,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
}
```

By reading the Rust code, we now know the *exact* shape of every field. This is much better than guessing or relying on outdated documentation!

## The ExportResponse: err, meta, urls, hashes

Let's build our first TypeScript type. The export endpoint returns the most complex response, so let's start there:

```typescript
/**
 * Response from GET /api/v0/epub
 *
 * The main export endpoint. Returns metadata and download URLs
 * for a fanfiction story.
 */
export interface ExportResponse {
  /** Error code: 0 = success, negative = error */
  err: number

  /** The original query URL */
  q: string

  /** Fix-it suggestions (usually empty) */
  fixits: string[]

  /** Human-readable info string (title by author, word count, etc.) */
  info: string

  /** Unique identifier for this fic (SHA-256 hash, 12 hex chars) */
  url_id: string

  /** URL-friendly slug: "Title-Slug-url_id" */
  slug: string

  /** Detailed metadata about the fic */
  meta: FicMeta

  /** Map of format → file hash (e.g., { epub: "abc123...", html: "def456..." }) */
  hashes: Record<string, string>

  /** Map of format → download URL (e.g., { epub: "/cache/epub/...", html: "/cache/html/..." }) */
  urls: Record<string, string>

  /** Direct URL to EPUB download, or null if not available */
  epub_url: string | null

  /** Direct URL to HTML bundle, or null if not available */
  html_url: string | null

  /** Direct URL to MOBI download, or null if not available */
  mobi_url: string | null

  /** Direct URL to PDF download, or null if not available */
  pdf_url: string | null

  /** Info/warning messages */
  notes: string[]
}
```

Notice a few things:

1. **`Record<string, string>`** — This is TypeScript's way of saying "an object where all keys are strings and all values are strings." It's perfect for our `hashes` and `urls` maps.

2. **`string | null`** — This means the field is either a string or null. The pipe `|` means "or." This is TypeScript's way of handling nullable fields.

3. **JSDoc comments** — The `/** ... */` comments explain what each field means. This is documentation that other developers (and future you) will thank you for.

> **Watch Out!** Don't confuse `null` and `undefined` in TypeScript. `null` means "explicitly no value." `undefined` means "the property doesn't exist." FicHub's API uses `null` for missing URLs (like when there's no PDF available), not `undefined`.

## The FicMeta Struct

Now let's define the `FicMeta` type based on the `build_meta_json` function:

```typescript
/**
 * Detailed metadata about a fanfiction story.
 * Appears inside ExportResponse.meta and MetaResponse.meta.
 */
export interface FicMeta {
  /** Unique fic identifier (SHA-256 hash) */
  id: string

  /** Story title */
  title: string

  /** Author name */
  author: string

  /** Number of chapters */
  chapters: number

  /** Total word count */
  words: number

  /** HTML-formatted description/summary */
  description: string

  /** Completion status: "ongoing", "complete", "hiatus", or "cancelled" */
  status: string

  /** Original URL where the story was scraped from */
  source: string

  /** ISO 8601 timestamp of when the story was first published */
  created: string

  /** ISO 8601 timestamp of when the story was last updated */
  updated: string

  /** Extra metadata (site-specific, often null) */
  extra_meta: string | null

  /** Raw extended metadata (site-specific, often null) */
  raw_extended_meta: string | null

  /** Author's profile URL on the source site */
  author_url: string

  /** Author's ID on the source site */
  author_local_id: string

  /** Source site ID (1 = AO3, 2 = FF.net, etc.) */
  source_id: number

  /** Author's numeric ID on the source site */
  author_id: number
}
```

Look at the types carefully:

- `chapters: number` — TypeScript uses `number` for both integers and floats. In JavaScript/TypeScript, there's no separate `int` type.
- `words: number` — Same thing. The Rust backend uses `i64` (a 64-bit integer), but TypeScript just calls it `number`.
- `status: string` — This could be a union type like `'ongoing' | 'complete' | 'hiatus' | 'cancelled'`, but the Rust backend doesn't constrain it, so we use `string` to be safe.

> **Try It Yourself!** Look at the `FicMetadata` struct in `src/scrape/mod.rs`. Can you match every Rust field to its TypeScript equivalent? Notice how `Option<String>` in Rust becomes `string | null` in TypeScript.

### Rust → TypeScript Type Mapping Cheat Sheet

Here's a quick reference for translating Rust types to TypeScript:

| Rust Type | TypeScript Type | Example |
|-----------|----------------|---------|
| `String` | `string` | `"hello"` |
| `i32`, `i64` | `number` | `42`, `50000` |
| `f32`, `f64` | `number` | `3.14`, `0.8` |
| `bool` | `boolean` | `true`, `false` |
| `Option<String>` | `string \| null` | `"hello"` or `null` |
| `Option<i32>` | `number \| null` | `42` or `null` |
| `Vec<String>` | `string[]` | `["a", "b", "c"]` |
| `HashMap<String, String>` | `Record<string, string>` | `{"key": "value"}` |
| `HashMap<String, Value>` | `Record<string, any>` | `{"key": anything}` |
| `serde_json::Value` | `any` | anything |

This mapping works because JSON has a limited set of types. There's no `i32` vs `i64` distinction in JSON — everything is just a "number." The TypeScript type system adds structure on top of the JSON.

## The MetaResponse

The meta endpoint (`GET /api/v0/meta`) returns a simpler version of the export response — same structure, but with empty hashes and URLs:

```typescript
/**
 * Response from GET /api/v0/meta
 *
 * Returns metadata for a fanfiction story without generating
 * any export files.
 */
export interface MetaResponse {
  err: number
  q: string
  fixits: string[]
  info: string
  url_id: string
  slug: string
  meta: FicMeta
  hashes: Record<string, string>  // empty object
  urls: Record<string, string>    // empty object
  epub_url: null
  html_url: null
  mobi_url: null
  pdf_url: null
  notes: string[]
}
```

Since `MetaResponse` and `ExportResponse` share the same shape, you could use a single type for both:

```typescript
export type FicResponse = ExportResponse  // or MetaResponse — they're the same shape!
```

But keeping them separate is clearer. When a developer sees `MetaResponse`, they know "this is the lighter metadata-only response."

## The RecResult: Recommendations

Now let's look at the recommendation system. The `GET /api/v0/recommendations` endpoint returns recommendations — a list of similar fics based on collaborative filtering.

Looking at the Rust code in `src/recommender/routes.rs`:

```rust
Ok(Json(json!({
    "err": 0,
    "url_id": url_id,
    "site_domain": params.site_domain,
    "recommendations": recommendations,
    "generated_at": chrono::Utc::now().to_rfc3339(),
})))
```

The `recommendations` field contains an array of recommendation objects. Based on the engine code, each recommendation looks like:

```typescript
/**
 * A single recommendation — a similar fic suggested by the engine.
 */
export interface RecResult {
  /** The url_id of the recommended fic */
  url_id: string

  /** Title of the recommended fic */
  title: string

  /** Algorithmic similarity score (0.0 to 1.0) */
  score: number

  /** Community score based on user votes (can be negative) */
  community_score: number
}

/**
 * Response from GET /api/v0/recommendations
 */
export interface RecommendationsResponse {
  err: number
  url_id: string
  site_domain: string | null
  recommendations: RecResult[]
  generated_at: string
}
```

## The Suggestion: Community-Submitted Recommendations

Users can submit their own recommendation suggestions. The `POST /api/v0/recommendations/suggest` endpoint handles this:

```typescript
/**
 * Request body for POST /api/v0/recommendations/suggest
 */
export interface SuggestRequest {
  /** url_id of the fic being recommended TO */
  url_id: string
  /** URL of the fic being suggested AS a recommendation */
  suggested_url: string
  /** Optional comment explaining why this is a good recommendation */
  comment?: string
}

/**
 * Response from POST /api/v0/recommendations/suggest
 */
export interface SuggestResponse {
  err: number
  /** The ID of the newly created suggestion (for voting) */
  suggestion_id: number
}
```

And for voting:

```typescript
/**
 * Request body for POST /api/v0/recommendations/vote
 */
export interface VoteRequest {
  /** The suggestion ID to vote on */
  suggestion_id: number
  /** Vote value: 1 for upvote, -1 for downvote */
  vote: 1 | -1
}

/**
 * Response from POST /api/v0/recommendations/vote
 */
export interface VoteResponse {
  err: number
  /** The new total score after this vote */
  new_score: number
}
```

Notice the `1 | -1` type — that's a **union type**. It means `vote` can only be exactly `1` or exactly `-1`. TypeScript will catch it if you accidentally pass `0` or `2`!

### Votes Response

The `GET /api/v0/recommendations/votes` endpoint lists all suggestions and their vote scores for a given fic:

```typescript
/**
 * A single community suggestion with its vote score.
 */
export interface Suggestion {
  /** The suggestion's unique ID */
  id: number
  /** The url_id of the fic being suggested */
  suggested_url_id: string
  /** Net vote score (upvotes minus downvotes) */
  net_votes: number
}

/**
 * Response from GET /api/v0/recommendations/votes
 */
export interface VotesResponse {
  err: number
  url_id: string
  suggestions: Suggestion[]
}
```

## Building the types.ts File Step by Step

Now let's put all our types together in a single file. In FicHub, this is `src/lib/api/fichub-types.ts`:

```typescript
// =============================================================================
// fichub-types.ts — TypeScript types for the FicHub v0 API
//
// These types are derived from the Rust backend source code.
// Always verify against the actual response shapes!
// =============================================================================

// --- Error codes -----------------------------------------------------------

/** FicHub error codes (the "err" field in responses) */
export const ErrorCode = {
  SUCCESS: 0,
  BAD_REQUEST: -1,
  NOT_FOUND: -5,
  SCRAPE_ERROR: -6,
  BLACKLISTED: -7,
  BLOCKED_AUTOMATED: -10,
  RATE_LIMITED: -429,
} as const

export type ErrorCode = (typeof ErrorCode)[keyof typeof ErrorCode]

// --- Core types ------------------------------------------------------------

export interface FicMeta {
  id: string
  title: string
  author: string
  chapters: number
  words: number
  description: string
  status: string
  source: string
  created: string
  updated: string
  extra_meta: string | null
  raw_extended_meta: string | null
  author_url: string
  author_local_id: string
  source_id: number
  author_id: number
}

// --- Export ----------------------------------------------------------------

export interface ExportResponse {
  err: number
  q: string
  fixits: string[]
  info: string
  url_id: string
  slug: string
  meta: FicMeta
  hashes: Record<string, string>
  urls: Record<string, string>
  epub_url: string | null
  html_url: string | null
  mobi_url: string | null
  pdf_url: string | null
  notes: string[]
}

// --- Meta ------------------------------------------------------------------

export interface MetaResponse {
  err: number
  q: string
  fixits: string[]
  info: string
  url_id: string
  slug: string
  meta: FicMeta
  hashes: Record<string, string>
  urls: Record<string, string>
  epub_url: null
  html_url: null
  mobi_url: null
  pdf_url: null
  notes: string[]
}

// --- Recommendations -------------------------------------------------------

export interface RecResult {
  url_id: string
  title: string
  score: number
  community_score: number
}

export interface RecommendationsResponse {
  err: number
  url_id: string
  site_domain: string | null
  recommendations: RecResult[]
  generated_at: string
}

// --- Suggestions & Votes ---------------------------------------------------

export interface Suggestion {
  id: number
  suggested_url_id: string
  net_votes: number
}

export interface VotesResponse {
  err: number
  url_id: string
  suggestions: Suggestion[]
}

export interface SuggestRequest {
  url_id: string
  suggested_url: string
  comment?: string
}

export interface SuggestResponse {
  err: number
  suggestion_id: number
}

export interface VoteRequest {
  suggestion_id: number
  vote: 1 | -1
}

export interface VoteResponse {
  err: number
  new_score: number
}

// --- Search ----------------------------------------------------------------

export interface SearchTag {
  name: string
  type: string
  type_id: number
  score: number
}

export interface SearchResult {
  url_id: string
  title: string
  author: string
  source: string
  words: number
  chapters: number
  status: string
  description: string
  updated: string | null
  rank: number | null
  tags: SearchTag[]
  total_freeform: number
}

export interface SearchResponse {
  total: number
  page: number
  per_page: number
  results: SearchResult[]
}
```

## Naming Conventions

Every codebase has naming conventions. Following them makes your code consistent and easier to read:

### PascalCase for Types

TypeScript interfaces and types use **PascalCase** — each word starts with a capital letter:

```typescript
✅ ExportResponse      // PascalCase
✅ FicMeta             // PascalCase
✅ SearchResponse      // PascalCase
❌ export_response     // snake_case — wrong for TypeScript!
❌ exportResponse      // camelCase — wrong for types!
```

### camelCase for Fields

Object properties use **camelCase** — the first word is lowercase, subsequent words are capitalized:

```typescript
export interface ExportResponse {
  url_id: string       // ✅ camelCase
  epub_url: string     // ✅ camelCase
  urlId: string        // Also valid camelCase
  EPUB_URL: string     // ❌ SCREAMING_CASE — wrong!
}
```

> **Watch Out!** FicHub's API uses `snake_case` in JSON responses (`url_id`, `epub_url`) because that's what the Rust backend uses. In TypeScript, you have two choices: keep `snake_case` to match the API, or use `camelCase` and add a `@ts-ignore` or transform layer. FicHub keeps `snake_case` in its types for simplicity — it's less work and fewer bugs.

### UPPERCASE for Constants

Constant values that never change use **UPPER_SNAKE_CASE**:

```typescript
export const SORT_OPTIONS = {
  RELEVANCE: '-relevance',
  DATE: '-date',
  WORDS: '-words',
} as const
```

## Handling Nullable Fields

In TypeScript, there are several ways to handle "this value might not exist":

### `string | null` — Explicit Null

The value is either a string or explicitly null. This is what FicHub's API uses:

```typescript
epub_url: string | null  // Could be a URL or null
```

### `string | undefined` — Missing Property

The property might not exist on the object at all:

```typescript
epub_url?: string  // Could be a URL or missing entirely
```

### `string` — Always Present

The value is always a string. No nulls, no missing values:

```typescript
url_id: string  // Always present, always a string
```

### Optional Parameters with Defaults

When a parameter has a default value on the server, you can make it optional in TypeScript:

```typescript
interface SearchParams {
  q?: string              // Optional — defaults to empty
  sort?: string           // Optional — defaults to "-date"
  per_page?: number       // Optional — defaults to 20
}
```

> **Try It Yourself!** Open your browser's developer tools, go to the Network tab, and visit a page that makes an API call (like the search page). Click on the API request and look at the Response tab. Can you match each field in the JSON response to a type in `fichub-types.ts`?

## Practice: Adding a New Type

Let's practice by adding a type for a new API endpoint. Imagine the FicHub backend adds a new endpoint:

```
GET /api/v0/stats
```

It returns:

```json
{
  "total_fics": 12345,
  "total_words": 9876543210,
  "total_users": 5678,
  "fics_by_source": {
    "archiveofourown.org": 8000,
    "fanfiction.net": 3000,
    "fictionpress.com": 1345
  },
  "recent_fics": [
    { "url_id": "abc123", "title": "New Story", "author": "Writer123" }
  ]
}
```

Let's design TypeScript types for this:

```typescript
// Step 1: Define the inner types first

/**
 * A recent fic summary (lighter than FicMeta — no word count, etc.)
 */
export interface RecentFic {
  url_id: string
  title: string
  author: string
}

/**
 * Stats response from GET /api/v0/stats
 */
export interface StatsResponse {
  /** Total number of fics in the database */
  total_fics: number

  /** Total word count across all fics */
  total_words: number

  /** Total registered users */
  total_users: number

  /** Breakdown of fics by source site */
  fics_by_source: Record<string, number>

  /** Most recently scraped fics (last 10) */
  recent_fics: RecentFic[]
}
```

Notice how we built this step by step:
1. First, we defined the inner type (`RecentFic`) that appears inside an array.
2. Then we defined the outer type (`StatsResponse`) that contains everything.
3. We used `Record<string, number>` for the `fics_by_source` map.
4. We used `RecentFic[]` for the array of recent fics.

### Practice Exercise: Design Types for a New Endpoint

Now it's your turn! Here's a new endpoint the backend might add:

```
GET /api/v0/fics/{url_id}/chapters
```

It returns:

```json
{
  "err": 0,
  "url_id": "abc123",
  "chapters": [
    {
      "position": 1,
      "title": "Chapter 1: The Beginning",
      "word_count": 5000
    },
    {
      "position": 2,
      "title": "Chapter 2: The Journey",
      "word_count": 7500
    }
  ],
  "total_words": 12500,
  "total_chapters": 2
}
```

Before reading the answer below, try designing the types yourself!

<details>
<summary>Click to reveal the answer</summary>

```typescript
/**
 * A single chapter in a fanfiction story.
 */
export interface ChapterSummary {
  /** Chapter position (1-indexed) */
  position: number

  /** Chapter title */
  title: string

  /** Word count for this chapter */
  word_count: number
}

/**
 * Response from GET /api/v0/fics/{url_id}/chapters
 */
export interface ChaptersResponse {
  /** Error code: 0 = success */
  err: number

  /** The fic's unique identifier */
  url_id: string

  /** List of chapters in order */
  chapters: ChapterSummary[]

  /** Total word count across all chapters */
  total_words: number

  /** Total number of chapters */
  total_chapters: number
}
```

Key decisions:
- We named it `ChapterSummary` (not `Chapter`) because it doesn't include the chapter content — just metadata
- We used `position` (not `chapter_number`) to match common fanfiction terminology
- We added `total_words` and `total_chapters` as convenience fields
- We kept `err` for consistency with other FicHub responses

</details>

### Practice Exercise: Design Types for an Error Response

What about error responses? When something goes wrong, the server returns:

```json
{
  "err": -5,
  "msg": "unsupported URL: https://not-a-fanfiction-site.com/story"
}
```

Design a type for this:

<details>
<summary>Click to reveal the answer</summary>

```typescript
/**
 * Standard error response from any FicHub API endpoint.
 *
 * All endpoints return this shape on error. Check `err` === 0 for success.
 */
export interface ErrorResponse {
  /** Error code: 0 = success, negative = error */
  err: number

  /** Human-readable error message */
  msg: string
}
```

This is useful as a base type. You could even make all responses extend it:

```typescript
/**
 * Generic API response wrapper.
 * Every endpoint returns err + msg on error, and additional fields on success.
 */
export interface ApiResponse<T> {
  err: number
  msg?: string
  data?: T
}
```

This pattern is common in APIs that want to wrap all responses in a consistent envelope.

</details>

## Summary

In this chapter, you learned:

- **Read the Rust source** to understand exact response shapes — it's the ultimate source of truth
- Use the **three-step process**: find the route handler, find the `json!` macro, trace the values
- **ExportResponse** is the most complex type, with metadata, download URLs, and hashes
- **FicMeta** contains all the metadata fields from the Rust `FicMetadata` struct
- **MetaResponse** is the same shape as ExportResponse but with empty hashes and URLs
- **RecResult**, **Suggestion**, and **SuggestRequest** types power the recommendation system
- **VotesResponse** and **VoteRequest** handle community voting
- **PascalCase** for type names, **camelCase** for fields, **UPPER_SNAKE_CASE** for constants
- Use `string | null` for nullable fields, `type?:` for optional parameters
- The **Rust → TypeScript mapping** is straightforward: `String` → `string`, `Option<T>` → `T | null`, `Vec<T>` → `T[]`
- Build complex types step by step, starting with inner types
- Use **JSDoc comments** (`/** ... */`) to document what each field means
- **Type assertions** (`as T`) tell TypeScript to trust your type knowledge
- **Union types** (`1 | -1`) restrict values to specific literal values

### Why Types Matter

TypeScript types aren't just documentation — they're a **safety net**. When your types match your API responses exactly:

- Your IDE provides **autocompletion** for all fields
- **Compile-time errors** catch typos and wrong types before they reach users
- **Refactoring** is safe — rename a field and TypeScript shows you everywhere it's used
- **Documentation** is always in sync — the type IS the documentation

Without types, you'd find bugs like `result.meta.titel` (typo!) at runtime — when your user sees a blank page. With types, TypeScript catches it immediately and underlines it in red.

In the next chapter, we'll use these types to build an API client — the code that actually *makes* the requests to the server. Let's go!

---

# Chapter 16: Building the API Client

We've learned about REST APIs and designed TypeScript types for all our responses. Now it's time to build the **API client** — the JavaScript code that actually makes HTTP requests to the backend and returns typed data.

This is where the rubber meets the road. Our types are the blueprint; the API client is the building.

## What Is an API Client?

An API client is a translator between your frontend application and the backend server. It takes care of all the HTTP details — URLs, headers, JSON parsing, error checking — so your components don't have to.

It:

1. **Takes simple function calls** — like `fetchExport(url)`
2. **Converts them to HTTP requests** — `GET /api/v0/epub?q=<url>`
3. **Handles errors** — network failures, bad responses, rate limits
4. **Returns typed data** — objects that match our TypeScript types

Think of it like a universal remote control. Instead of manually constructing URLs, setting headers, and parsing JSON, you just press a button: `fichub.fetchExport(url)`. The API client does all the dirty work.

Without an API client, every component would need to repeat the same boilerplate:

```typescript
// ❌ Without API client — repetitive and error-prone
const response = await fetch(`/api/v0/epub?q=${encodeURIComponent(url)}`, {
  headers: { 'Content-Type': 'application/json' },
})
const body = await response.json()
if (!response.ok) {
  throw new Error(`HTTP ${response.status}: ${body.msg}`)
}
// ... more error handling
```

```typescript
// ✅ With API client — clean and simple
const result = await fetchExport(url)
```

The difference is dramatic. The API client encapsulates 10+ lines of boilerplate into a single, readable function call.

## The Fetch API: Making HTTP Requests

Every modern browser has a built-in function for making HTTP requests: `fetch()`. It's the standard way to talk to servers from JavaScript.

Here's the basics:

```typescript
// A simple GET request
const response = await fetch('http://localhost:8004/api/v0/meta?q=https://example.com')

// Check if the request succeeded
if (!response.ok) {
  throw new Error(`HTTP ${response.status}`)
}

// Parse the JSON body
const data = await response.json()
console.log(data)
```

The `fetch()` function returns a **Promise** — a special object that represents a future value. We use `await` to wait for the Promise to resolve.

### Anatomy of a Response

The `response` object contains several useful properties:

```typescript
const response = await fetch(url)

response.ok          // true if status is 200-299
response.status      // HTTP status code (200, 404, 500, etc.)
response.statusText  // "OK", "Not Found", "Internal Server Error"
response.headers     // Response headers (Map-like object)
response.url         // The final URL (after redirects)
```

### Response Methods

The response body isn't available immediately — you need to read it:

```typescript
// Read as JSON
const data = await response.json()

// Read as text
const text = await response.text()

// Read as Blob (for binary data like images)
const blob = await response.blob()

// Read as ArrayBuffer (for binary data)
const buffer = await response.arrayBuffer()
```

For FicHub, we always use `response.json()` because the API returns JSON.

### Request Options

The second argument to `fetch()` is an options object:

```typescript
const response = await fetch(url, {
  method: 'POST',          // HTTP method
  headers: {               // Request headers
    'Content-Type': 'application/json',
    'Authorization': 'Bearer token123',
  },
  body: JSON.stringify(data),  // Request body (for POST/PUT)
  signal: AbortSignal.timeout(10000),  // Timeout after 10 seconds
})
```

### AbortController: Timeouts

By default, `fetch()` waits indefinitely. For better UX, add a timeout:

```typescript
const controller = new AbortController()
const timeoutId = setTimeout(() => controller.abort(), 10000) // 10 seconds

try {
  const response = await fetch(url, { signal: controller.signal })
  const data = await response.json()
  // ... process data
} catch (error) {
  if (error.name === 'AbortError') {
    console.error('Request timed out')
  }
} finally {
  clearTimeout(timeoutId)
}
```

This is especially important for the export endpoint, which can take a while if the fic has many chapters.

> **Watch Out!** `fetch()` only throws an error for *network* failures (like no internet connection). A 404 or 500 status code does NOT throw an error — you need to check `response.ok` yourself! This is a common source of bugs.

## The request() Helper Function

In FicHub's API client, we don't call `fetch()` directly for every request. Instead, we create a **helper function** that handles all the common stuff:

```typescript
const BASE_URL = ''  // Same origin — no need for full URL

/**
 * Make an HTTP request to the FicHub API.
 *
 * @param path - The API path (e.g., '/api/v0/epub')
 * @param options - Fetch options (method, body, etc.)
 * @returns The parsed JSON response
 * @throws ApiError if the response is not OK
 */
async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const url = `${BASE_URL}${path}`

  const response = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
  })

  // Always parse the body, even on error
  const body = await response.json()

  if (!response.ok) {
    throw new ApiError(response.status, body)
  }

  return body as T
}
```

Let's break this down:

1. **`<T>` is a generic type parameter** — This means the caller can specify what type the response should be. If we call `request<ExportResponse>(...)`, TypeScript knows the return type is `ExportResponse`.

2. **We always set `Content-Type: application/json`** — This tells the server we're sending JSON. We merge any additional headers from the caller.

3. **We always parse the body** — Even on error responses, we parse the JSON so we can extract error messages.

4. **We throw `ApiError` on failure** — This is a custom error class that wraps the status code and response body.

5. **We return `body as T`** — The `as T` is a type assertion. We're telling TypeScript "trust me, this body matches the type T."

## Error Handling: try/catch and the ApiError Class

Error handling is critical for a good user experience. Let's define our error class:

```typescript
/**
 * Custom error class for API errors.
 * Wraps the HTTP status code and parsed response body.
 */
export class ApiError extends Error {
  constructor(
    public status: number,
    public body: any
  ) {
    super(`API Error ${status}: ${body?.msg || 'Unknown error'}`)
    this.name = 'ApiError'
  }

  /** Get the FicHub error code from the body (e.g., -5 for not found) */
  get errCode(): number {
    return this.body?.err ?? -1
  }

  /** Get the human-readable error message */
  get message(): string {
    return this.body?.msg || 'Unknown error'
  }
}
```

Now let's see how we use try/catch with it:

```typescript
try {
  const result = await request<ExportResponse>('/api/v0/epub?q=' + encodeURIComponent(url))
  console.log('Success!', result.meta.title)
} catch (error) {
  if (error instanceof ApiError) {
    if (error.status === 404) {
      console.log('Fic not found:', error.message)
    } else if (error.status === 429) {
      console.log('Rate limited! Try again later.')
    } else {
      console.log('API error:', error.status, error.message)
    }
  } else {
    console.log('Network error:', error)
  }
}
```

The `try` block wraps the code that might fail. If an error is thrown, execution jumps to the `catch` block. We use `instanceof` to check if it's an `ApiError` (server responded with an error) or a plain `Error` (network failure).

## The buildQuery() Helper

API URLs need properly formatted query parameters. Special characters need to be encoded, and arrays need special handling. Let's write a helper:

```typescript
/**
 * Convert an object of parameters to a URL query string.
 *
 * @example
 * buildQuery({ q: 'test', complete: true, n: 10 })
 * // Returns: "q=test&complete=true&n=10"
 *
 * @example
 * buildQuery({ q: 'hello world' })
 * // Returns: "q=hello%20world"
 */
function buildQuery(params: Record<string, string | number | boolean | null | undefined>): string {
  const entries = Object.entries(params)
    .filter(([_, value]) => value !== null && value !== undefined && value !== '')
    .map(([key, value]) => `${encodeURIComponent(key)}=${encodeURIComponent(String(value))}`)

  return entries.length > 0 ? `?${entries.join('&')}` : ''
}
```

Key things about this helper:

1. **We filter out null, undefined, and empty strings** — No point sending `?q=&sort=` to the server. Better to send just `?sort=-date`.
2. **We use `encodeURIComponent`** — This converts spaces to `%20`, `&` to `%26`, etc. This is essential for safety.
3. **We return an empty string for no params** — This makes concatenation easy: `'/api/v0/search' + buildQuery(filters)`.

> **Try It Yourself!** Test the `buildQuery` function in your browser console:
> ```javascript
> function buildQuery(params) {
>   const entries = Object.entries(params)
>     .filter(([_, v]) => v !== null && v !== undefined && v !== '')
>     .map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(String(v))}`)
>   return entries.length > 0 ? `?${entries.join('&')}` : ''
> }
>
> buildQuery({ q: 'harry potter', complete: true, sort: '-words' })
> // What does this output?
>
> // Challenge: What happens with these inputs?
> buildQuery({ q: 'hello&world', sort: '-relevance' })  // Special characters!
> buildQuery({ q: '', sort: '-date', page: 1 })          // Empty string filter
> buildQuery({})                                          // Empty object
> ```
>
> Try to predict the output before running each one!

## fetchExport(): GET /api/v0/epub

Now let's build the actual API functions. Starting with the most important one:

```typescript
/**
 * Export a fanfiction story as an EPUB.
 *
 * @param url - The URL of the fanfiction story
 * @returns Export response with metadata and download URLs
 * @throws ApiError on failure
 */
export async function fetchExport(url: string): Promise<ExportResponse> {
  const query = buildQuery({ q: url })
  return request<ExportResponse>(`/api/v0/epub${query}`)
}
```

That's it! Just 3 lines of actual code. The `request()` helper handles everything else — headers, JSON parsing, error checking.

Usage:

```typescript
try {
  const result = await fetchExport('https://archiveofourown.org/works/123456')
  console.log(`Title: ${result.meta.title}`)
  console.log(`Download: ${result.epub_url}`)
} catch (error) {
  if (error instanceof ApiError) {
    console.error('Export failed:', error.message)
  }
}
```

## fetchMeta(): GET /api/v0/meta

The meta endpoint is similar but lighter:

```typescript
/**
 * Get metadata for a fanfiction story (without generating an export).
 *
 * @param url - The URL of the fanfiction story
 * @returns Metadata response
 */
export async function fetchMeta(url: string): Promise<MetaResponse> {
  const query = buildQuery({ q: url })
  return request<MetaResponse>(`/api/v0/meta${query}`)
}
```

## fetchRecommendations(): GET /api/v0/recommendations

```typescript
/**
 * Get recommendations for a fanfiction story.
 *
 * @param params - Query parameters (url or url_id, count, site filter)
 * @returns List of recommended fics
 */
export async function fetchRecommendations(params: {
  q?: string
  url_id?: string
  n?: number
  site_domain?: string
}): Promise<RecommendationsResponse> {
  const query = buildQuery(params)
  return request<RecommendationsResponse>(`/api/v0/recommendations${query}`)
}
```

## fetchVotes(): GET /api/v0/recommendations/votes

```typescript
/**
 * Get community suggestions and their vote scores for a fic.
 *
 * @param url_id - The fic's unique identifier
 * @returns List of suggestions with net vote scores
 */
export async function fetchVotes(url_id: string): Promise<VotesResponse> {
  const query = buildQuery({ url_id })
  return request<VotesResponse>(`/api/v0/recommendations/votes${query}`)
}
```

## submitSuggestion(): POST /api/v0/recommendations/suggest

POST endpoints are different — they send data in the request body instead of query parameters:

```typescript
/**
 * Submit a recommendation suggestion.
 *
 * @param data - The suggestion data (url_id, suggested_url, optional comment)
 * @returns The suggestion ID (for voting)
 */
export async function submitSuggestion(data: SuggestRequest): Promise<SuggestResponse> {
  return request<SuggestResponse>('/api/v0/recommendations/suggest', {
    method: 'POST',
    body: JSON.stringify(data),
  })
}
```

Notice how we pass `{ method: 'POST', body: JSON.stringify(data) }` as the second argument to `request()`. This tells the helper to make a POST request with a JSON body.

## castVote(): POST /api/v0/recommendations/vote

```typescript
/**
 * Cast a vote (upvote or downvote) on a recommendation suggestion.
 *
 * @param suggestion_id - The suggestion to vote on
 * @param vote - 1 for upvote, -1 for downvote
 * @returns The new total score
 */
export async function castVote(suggestion_id: number, vote: 1 | -1): Promise<VoteResponse> {
  return request<VoteResponse>('/api/v0/recommendations/vote', {
    method: 'POST',
    body: JSON.stringify({ suggestion_id, vote }),
  })
}
```

## Putting It All Together

Here's what the complete API client file looks like:

```typescript
// =============================================================================
// api.ts — FicHub API Client
//
// All functions for communicating with the FicHub v0 backend.
// =============================================================================

import type {
  ExportResponse,
  MetaResponse,
  RecommendationsResponse,
  VotesResponse,
  SuggestRequest,
  SuggestResponse,
  VoteRequest,
  VoteResponse,
} from './types'

// --- Error handling --------------------------------------------------------

export class ApiError extends Error {
  constructor(
    public status: number,
    public body: any
  ) {
    super(`API Error ${status}: ${body?.msg || 'Unknown error'}`)
    this.name = 'ApiError'
  }

  get errCode(): number {
    return this.body?.err ?? -1
  }

  get message(): string {
    return this.body?.msg || 'Unknown error'
  }
}

// --- Helpers ---------------------------------------------------------------

const BASE_URL = ''

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const url = `${BASE_URL}${path}`

  const response = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
  })

  const body = await response.json()

  if (!response.ok) {
    throw new ApiError(response.status, body)
  }

  return body as T
}

function buildQuery(params: Record<string, string | number | boolean | null | undefined>): string {
  const entries = Object.entries(params)
    .filter(([_, v]) => v !== null && v !== undefined && v !== '')
    .map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(String(v))}`)

  return entries.length > 0 ? `?${entries.join('&')}` : ''
}

// --- API functions ---------------------------------------------------------

export async function fetchExport(url: string): Promise<ExportResponse> {
  const query = buildQuery({ q: url })
  return request<ExportResponse>(`/api/v0/epub${query}`)
}

export async function fetchMeta(url: string): Promise<MetaResponse> {
  const query = buildQuery({ q: url })
  return request<MetaResponse>(`/api/v0/meta${query}`)
}

export async function fetchRecommendations(params: {
  q?: string
  url_id?: string
  n?: number
  site_domain?: string
}): Promise<RecommendationsResponse> {
  const query = buildQuery(params)
  return request<RecommendationsResponse>(`/api/v0/recommendations${query}`)
}

export async function fetchVotes(url_id: string): Promise<VotesResponse> {
  const query = buildQuery({ url_id })
  return request<VotesResponse>(`/api/v0/recommendations/votes${query}`)
}

export async function submitSuggestion(data: SuggestRequest): Promise<SuggestResponse> {
  return request<SuggestResponse>('/api/v0/recommendations/suggest', {
    method: 'POST',
    body: JSON.stringify(data),
  })
}

export async function castVote(suggestion_id: number, vote: 1 | -1): Promise<VoteResponse> {
  return request<VoteResponse>('/api/v0/recommendations/vote', {
    method: 'POST',
    body: JSON.stringify({ suggestion_id, vote }),
  })
}
```

## Practice: Adding a New API Function

Let's practice by adding a function for the stats endpoint we designed types for in the previous chapter:

```typescript
// Given this endpoint:
// GET /api/v0/stats

export async function fetchStats(): Promise<StatsResponse> {
  return request<StatsResponse>('/api/v0/stats')
}
```

That's literally one line of code! The `request()` helper does everything.

Now let's add a more complex one — the search endpoint:

```typescript
// Given this endpoint:
// GET /api/v0/search?q=harry+potter&complete=true&sort=-words&page=1&per_page=20

export async function search(params: {
  q?: string
  include_tags?: string
  exclude_tags?: string
  complete?: boolean
  sort?: string
  page?: number
  per_page?: number
}): Promise<SearchResponse> {
  const query = buildQuery(params)
  return request<SearchResponse>(`/api/v0/search${query}`)
}
```

### Common Patterns

Notice the patterns emerging:

1. **GET requests** use `buildQuery()` to convert parameters to a URL string
2. **POST requests** use `JSON.stringify()` to convert data to a JSON body
3. **All functions** return a Promise of a specific type
4. **All functions** throw `ApiError` on failure

### Error Handling Patterns

Here are common error handling patterns you'll use:

```typescript
// Pattern 1: Let the error bubble up
async function fetchAndDisplay(url: string) {
  const result = await fetchExport(url)  // Throws ApiError on failure
  displayResult(result)
}

// Pattern 2: Handle locally
async function fetchWithErrorCard(url: string) {
  try {
    const result = await fetchExport(url)
    displayResult(result)
  } catch (error) {
    showErrorCard(error)
  }
}

// Pattern 3: Return null on error (for optional data)
async function fetchMaybe(url: string): Promise<ExportResponse | null> {
  try {
    return await fetchExport(url)
  } catch {
    return null
  }
}

// Pattern 4: Return a result tuple (like Rust's Result)
async function fetchSafe(url: string): Promise<[ExportResponse | null, ApiError | null]> {
  try {
    const result = await fetchExport(url)
    return [result, null]
  } catch (error) {
    return [null, error instanceof ApiError ? error : new ApiError(0, { msg: 'Unknown' })]
  }
}
```

Each pattern serves a different use case. Pattern 1 is simplest but requires a try/catch at the call site. Pattern 2 handles errors immediately. Pattern 3 is good for optional data. Pattern 4 is explicit about both success and failure.

> **Try It Yourself!** The `GET /api/v0/remote` endpoint returns IP, port, and automated status. Try writing both the TypeScript type and the API function for it. Remember: the type goes in `types.ts` and the function goes in `api.ts`.
>
> Here's a hint — the Rust code looks like:
> ```rust
> Json(json!({
>     "remote": {
>         "ip": client_ip,
>         "port": 8004,
>         "is_automated": false,
>     }
> }))
> ```
>
> Can you design the TypeScript types? Can you write the API function?

## Summary

In this chapter, you learned:

- An **API client** is a translator between your app and the server — it handles HTTP details so your components don't have to
- The **`fetch()` API** is the browser's built-in HTTP client — powerful but low-level
- **`fetch()` only throws for network errors** — a 404 or 500 does NOT throw; you must check `response.ok`
- The **`request()` helper** handles common concerns: headers, JSON parsing, error checking
- **Generic types** (`request<T>()`) ensure type safety — TypeScript knows the return type
- **`ApiError`** wraps status codes and response bodies for meaningful error messages
- **`buildQuery()`** converts objects to URL query strings safely — always use `encodeURIComponent()`
- **GET functions** use `buildQuery()` for query parameters
- **POST functions** pass `JSON.stringify(data)` as the body
- **`AbortController`** lets you cancel stale requests and add timeouts
- There are **four error handling patterns**: bubble up, handle locally, return null, return result tuple
- The **factory function pattern** (`defaultFilters()`) prevents shared state bugs
- **`as const`** makes objects immutable and provides literal types for constants

### The API Client Architecture

Our API client follows a clean layered architecture:

```
Component (Svelte)
    ↓
API Function (fetchExport, search, etc.)
    ↓
Request Helper (request<T>)
    ↓
Fetch API (browser built-in)
    ↓
HTTP (TCP/IP)
    ↓
Server (Rust/Axum)
```

Each layer has a single responsibility:
- **Components** handle UI rendering and user interaction
- **API functions** define the public interface — what operations are possible
- **Request helper** handles HTTP mechanics — headers, JSON, errors
- **Fetch API** handles the actual network communication

This separation means you can:
- Test API functions without a browser
- Change the HTTP library without affecting components
- Mock the API for testing
- Add logging, caching, or retry logic in one place

### API Client Best Practices

1. **One file per concern** — `types.ts` for types, `api.ts` for functions, `errors.ts` for error classes
2. **Export everything** — Types and functions should be importable by components
3. **Document public functions** — JSDoc comments explain parameters and return values
4. **Use consistent naming** — `fetchX` for GET, `createX` for POST, `updateX` for PUT, `deleteX` for DELETE
5. **Handle errors at the boundary** — Let the API client throw, let the component catch
6. **Never hardcode URLs** — Use a base URL constant that can be changed for different environments
7. **Always encode user input** — Never put raw strings in URLs

In the next chapter, we'll dive deep into error handling patterns — how to show loading states, display error messages, and handle the error lifecycle gracefully. Let's keep going!

---

# Chapter 17: Error Handling Patterns

We've built an API client that can talk to the backend. But here's a hard truth: **things will go wrong**. The network will fail. The server will return errors. Users will enter invalid data. The API will be rate-limited.

The difference between a good app and a bad app isn't whether errors happen — it's how the app *handles* them. In this chapter, we'll learn the patterns that make FicHub resilient and user-friendly.

## Why Error Handling Matters

Let's play out a scenario. A user visits FicHub and types a URL into the search box. They click "Export." Here's what can go wrong:

1. **No internet connection** — `fetch()` throws a TypeError
2. **Server is down** — `fetch()` throws a TypeError (connection refused)
3. **Invalid URL** — Server returns `{ "err": -5, "msg": "unsupported URL" }` with status 400
4. **Rate limited** — Server returns status 429 with a retry-after header
5. **Scraping failed** — Server returns status 502 with `{ "err": -6, "msg": "scrape timeout" }`
6. **Content blacklisted** — Server returns `{ "err": -7, "msg": "fic is blacklisted" }`
7. **Server bug** — Server returns status 500 with `{ "err": -1, "msg": "internal server error" }`
8. **Timeout** — The request takes too long (maybe a big fic with many chapters)
9. **CORS error** — The browser blocks a cross-origin request
10. **Malformed response** — The server returns something that isn't valid JSON

That's *ten* different failure modes for a single button click! If we don't handle any of these, the user sees a blank page, a frozen button, or a cryptic JavaScript error in the console. That's terrible UX.

But if we handle them gracefully — showing clear error messages, offering retry buttons, and degrading gracefully — users understand what happened and can take action.

### The Cost of Not Handling Errors

Let's look at what happens when error handling is missing:

```typescript
// ❌ NO error handling — dangerous!
async function handleExport(url: string) {
  const result = await fetchExport(url)
  // If the request fails, the app CRASHES here
  // The user sees a blank page or a frozen UI
  displayResult(result)
}
```

```typescript
// ✅ Proper error handling — safe!
async function handleExport(url: string) {
  try {
    const result = await fetchExport(url)
    displayResult(result)
  } catch (error) {
    displayError(error)
  }
}
```

The difference is one try/catch block. That's all it takes to prevent your app from crashing.

### Error Handling Is Not Optional

Some developers treat error handling as "nice to have" — something they'll add "later." But errors aren't optional! Your users *will* encounter them:

- **Mobile users** frequently lose connectivity on trains and subways
- **Slow connections** cause timeouts
- **Server updates** can introduce temporary bugs
- **Invalid input** happens constantly (typos, wrong URLs, etc.)

Every API call needs error handling. Every single one.

> **Watch Out!** Unhandled promise rejections (errors in async code without try/catch) can crash your entire app. In Node.js, they crash the process. In the browser, they show up in the console as "Uncaught (in promise)" errors and can break your UI. Always wrap async calls in try/catch!

## The ApiError Class: Status Code + Body

We introduced `ApiError` in the last chapter. Let's expand on it:

```typescript
export class ApiError extends Error {
  public status: number
  public body: any

  constructor(status: number, body: any) {
    super(`API Error ${status}: ${body?.msg || 'Unknown error'}`)
    this.name = 'ApiError'
    this.status = status
    this.body = body
  }

  /** The FicHub-specific error code (-1, -5, -6, -7, -10, -429) */
  get errCode(): number {
    return this.body?.err ?? -1
  }

  /** Human-readable error message from the server */
  get message(): string {
    return this.body?.msg || 'Unknown error'
  }

  /** Whether this error is retryable (e.g., rate limit, server error) */
  get retryable(): boolean {
    return this.status === 429 || this.status >= 500
  }

  /** How long to wait before retrying (for rate limits) */
  get retryAfter(): number | null {
    if (this.status === 429 && this.body?.retry_after) {
      return this.body.retry_after
    }
    return null
  }

  /** User-friendly error title */
  get title(): string {
    switch (this.status) {
      case 400:
        return 'Bad Request'
      case 404:
        return 'Not Found'
      case 429:
        return 'Too Many Requests'
      case 502:
        return 'Server Error'
      default:
        if (this.status >= 500) return 'Server Error'
        return 'Error'
    }
  }

  /** User-friendly error description */
  get description(): string {
    switch (this.errCode) {
      case -5:
        return 'This URL is not supported or the story could not be found.'
      case -6:
        return 'The upstream site is having issues. Please try again later.'
      case -7:
        return 'This content is not available on FicHub.'
      case -10:
        return 'Automated requests are not allowed.'
      case -429:
        return `You're making too many requests. Please wait ${this.retryAfter || 30} seconds.`
      default:
        return this.message || 'An unexpected error occurred.'
    }
  }
}
```

This class gives us everything we need to display helpful error information. The `title`, `description`, and `retryable` properties make it easy to show user-friendly messages without hardcoding them in every component.

## try/catch Blocks: The Safety Net

The `try/catch` block is JavaScript's error handling mechanism:

```typescript
try {
  // Code that might throw an error
  const result = await fetchExport(url)
  // If we get here, no error was thrown
  console.log('Success!', result.meta.title)
} catch (error) {
  // Code that runs when an error is thrown
  console.error('Something went wrong:', error)
}
```

Here are the patterns we use in FicHub:

### Pattern 1: Simple try/catch

```typescript
async function handleExport(url: string) {
  try {
    const result = await fetchExport(url)
    displayResult(result)
  } catch (error) {
    displayError(error)
  }
}
```

### Pattern 2: Check error type

```typescript
async function handleExport(url: string) {
  try {
    const result = await fetchExport(url)
    displayResult(result)
  } catch (error) {
    if (error instanceof ApiError) {
      // Server responded with an error
      if (error.retryable) {
        showRetryableError(error)
      } else {
        showError(error.title, error.description)
      }
    } else {
      // Network error (no internet, server down, etc.)
      showNetworkError()
    }
  }
}
```

### Pattern 3: Multiple API calls

```typescript
async function loadFicPage(url_id: string) {
  let meta: MetaResponse | null = null
  let recs: RecommendationsResponse | null = null
  let votes: VotesResponse | null = null

  // Try each request independently
  try {
    meta = await fetchMetaById(url_id)
  } catch (error) {
    console.error('Failed to load metadata:', error)
  }

  try {
    recs = await fetchRecommendations({ url_id, n: 10 })
  } catch (error) {
    console.error('Failed to load recommendations:', error)
  }

  try {
    votes = await fetchVotes(url_id)
  } catch (error) {
    console.error('Failed to load votes:', error)
  }

  // Display whatever we got
  displayPage(meta, recs, votes)
}
```

This pattern is called **graceful degradation** — we try to load everything, but if some requests fail, we still show what we have.

> **Watch Out!** Don't use `try/catch` as a substitute for proper validation. If you know a URL is empty, check it *before* making the API call:
> ```typescript
> // ❌ Bad: let the server reject it
> try {
>   const result = await fetchExport('')
> } catch (error) { ... }
>
> // ✅ Good: check first
> if (!url.trim()) {
>   showError('Please enter a URL')
>   return
> }
> ```

## Displaying Errors to Users: Error Cards

Users shouldn't see raw error messages or JavaScript stack traces. They should see friendly, helpful error cards. Here's a Svelte component pattern:

```svelte
{#if error}
  <div class="error-card">
    <div class="error-icon">⚠️</div>
    <h3 class="error-title">{error.title}</h3>
    <p class="error-description">{error.description}</p>
    {#if error.retryable}
      <button onclick={retry} class="retry-button">
        Try Again
      </button>
    {/if}
  </div>
{/if}
```

And the corresponding CSS:

```css
.error-card {
  background: var(--color-error-bg);
  border: 1px solid var(--color-error-border);
  border-radius: 8px;
  padding: 1.5rem;
  text-align: center;
  max-width: 400px;
  margin: 2rem auto;
}

.error-icon {
  font-size: 2rem;
  margin-bottom: 0.5rem;
}

.error-title {
  color: var(--color-error-text);
  margin-bottom: 0.5rem;
}

.error-description {
  color: var(--color-muted);
  margin-bottom: 1rem;
}

.retry-button {
  background: var(--color-primary);
  color: white;
  border: none;
  padding: 0.5rem 1.5rem;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 500;
}

.retry-button:hover {
  background: var(--color-primary-hover);
}
```

The key principles are:
1. **Show a friendly icon** — Visual indicator that something went wrong
2. **Clear title** — "Bad Request" or "Not Found" (not "Error -5")
3. **Helpful description** — Explain what happened in plain language
4. **Action when possible** — Show a "Try Again" button for retryable errors
5. **Don't overwhelm** — One clear message, not a wall of text

## Loading States: Spinners and Disabled Buttons

While an API call is in progress, the user needs feedback. A frozen page with no response is confusing. Here are the patterns:

### Spinner for Content Loading

```svelte
{#if loading}
  <div class="spinner-container">
    <div class="spinner"></div>
    <p>Loading metadata...</p>
  </div>
{:else if error}
  <ErrorCard {error} />
{:else if data}
  <FicDisplay {data} />
{/if}
```

### Disabled Button During Submission

```svelte
<button
  onclick={handleSubmit}
  disabled={submitting}
  class="export-button"
>
  {#if submitting}
    <span class="spinner-small"></span>
    Exporting...
  {:else}
    Export EPUB
  {/if}
</button>
```

### Progress Bar for Multi-Step Operations

```svelte
{#if loading}
  <div class="progress-bar">
    <div class="progress-fill" style="width: {progress}%"></div>
  </div>
  <p class="progress-text">{statusMessage}</p>
{/if}
```

> **Try It Yourself!** Create a simple loading state in Svelte:
> ```svelte
> <script>
>   let loading = $state(false)
>
>   async function loadData() {
>     loading = true
>     try {
>       await new Promise(r => setTimeout(r, 2000)) // Simulate API call
>       // ... handle data
>     } finally {
>       loading = false
>     }
>   }
> </script>
>
> <button onclick={loadData} disabled={loading}>
>   {loading ? 'Loading...' : 'Load Data'}
> </button>
> ```
>
> **Challenge:** Add an error state. If the "API call" randomly fails (throw an error 50% of the time), show an error message with a retry button.

### The Error State Lifecycle: Error → Retry → Success

Error handling isn't just about catching errors — it's about managing a **lifecycle**. Here's the typical flow:

```
User clicks button
    ↓
[Loading state] → Show spinner, disable button
    ↓
API call fails
    ↓
[Error state] → Show error card with retry button
    ↓
User clicks "Try Again"
    ↓
[Loading state] → Show spinner again
    ↓
API call succeeds
    ↓
[Success state] → Show results
```

This lifecycle is the same whether you're exporting a fic, loading recommendations, or searching. The states are:
1. **Idle** — Nothing is happening. The UI is ready for user interaction.
2. **Loading** — A request is in progress. Show feedback (spinner, disabled buttons).
3. **Error** — The request failed. Show error message with retry option.
4. **Success** — The request succeeded. Show the results.

In Svelte 5, we can model this with state variables:

```svelte
<script>
  import { ApiError } from '$lib/api/api'

  let data = $state(null)
  let error = $state(null)
  let loading = $state(false)

  async function fetchData(url) {
    loading = true
    error = null

    try {
      const result = await fetchExport(url)
      data = result
    } catch (err) {
      if (err instanceof ApiError) {
        error = err
      } else {
        error = new ApiError(0, { msg: 'Network error — please check your connection.' })
      }
    } finally {
      loading = false
    }
  }

  function handleRetry() {
    // Re-fetch with the same URL
    fetchData(lastUrl)
  }
</script>

{#if loading}
  <Spinner message="Loading..." />
{:else if error}
  <ErrorCard
    title={error.title}
    description={error.description}
    retryable={error.retryable}
    onretry={handleRetry}
  />
{:else if data}
  <FicDisplay {data} />
{/if}
```

The three states (`loading`, `error`, `data`) are mutually exclusive — only one is shown at a time. The `finally` block ensures `loading` is set to `false` whether the request succeeds or fails.

## Network Errors vs API Errors

There's an important distinction:

**Network errors** happen when `fetch()` itself fails:
- No internet connection
- DNS resolution failure
- Server is unreachable
- CORS errors
- Timeout

These throw plain `Error` or `TypeError` objects — not `ApiError`.

**API errors** happen when the server responds with a non-2xx status:
- 400 Bad Request
- 404 Not Found
- 429 Rate Limited
- 500 Internal Server Error

These are wrapped in `ApiError` by our `request()` helper.

Here's how to handle both:

```typescript
async function safeFetch(url: string) {
  try {
    return await fetchExport(url)
  } catch (error) {
    if (error instanceof ApiError) {
      // Server responded — we know what went wrong
      return { error: error }
    } else {
      // Network failure — server never responded
      return {
        error: new ApiError(0, {
          msg: 'Could not connect to the server. Please check your internet connection.',
        }),
      }
    }
  }
}
```

> **Watch Out!** CORS errors are network errors that happen when the browser blocks a cross-origin request. If you see "CORS error" in the console, it means the server isn't configured to accept requests from your frontend's origin. FicHub handles this by serving the frontend from the same origin as the API (both on port 8004).

## Graceful Degradation: Showing Partial Data

Sometimes, part of a page loads successfully and part fails. Instead of showing nothing, show what you have:

```svelte
<script>
  let meta = $state(null)
  let recs = $state(null)
  let metaError = $state(null)
  let recsError = $state(null)

  async function loadPage(url_id) {
    // Load metadata
    try {
      meta = await fetchMetaById(url_id)
    } catch (err) {
      metaError = err
    }

    // Load recommendations (independent)
    try {
      recs = await fetchRecommendations({ url_id, n: 10 })
    } catch (err) {
      recsError = err
    }
  }
</script>

{#if meta}
  <FicHeader {meta} />
{:else if metaError}
  <p class="warning">Could not load fic details.</p>
{/if}

{#if recs}
  <RecommendationList items={recs.recommendations} />
{:else if recsError}
  <p class="warning">Recommendations unavailable.</p>
{/if}
```

The user sees the fic's title and author (metadata loaded fine) even though recommendations failed to load. This is much better than showing a completely blank page!

### Parallel Requests with Promise.allSettled

For loading multiple independent resources, use `Promise.allSettled`:

```typescript
async function loadPage(url_id: string) {
  const [metaResult, recsResult, votesResult] = await Promise.allSettled([
    fetchMetaById(url_id),
    fetchRecommendations({ url_id, n: 10 }),
    fetchVotes(url_id),
  ])

  return {
    meta: metaResult.status === 'fulfilled' ? metaResult.value : null,
    recs: recsResult.status === 'fulfilled' ? recsResult.value : null,
    votes: votesResult.status === 'fulfilled' ? votesResult.value : null,
  }
}
```

`Promise.allSettled` waits for all promises to complete (success or failure) and returns the result of each one. This is better than `Promise.all`, which rejects as soon as *any* promise fails.

## Toast Notifications: Brief Messages

For non-critical messages (success confirmations, warnings), **toast notifications** are better than error cards. They appear briefly and disappear:

```typescript
import { toast } from '$lib/ui/shared/toast/toasts'

// Success toast
toast.success('Export started! Your EPUB will be ready shortly.')

// Warning toast
toast.warning('This fic is greylisted — download links are not available.')

// Error toast
toast.error('Failed to submit suggestion. Please try again.')

// Info toast
toast.info('Rate limited. Please wait 30 seconds.')
```

Toast notifications work well for:
- **Success confirmations** — "Vote recorded!"
- **Transient warnings** — "Cache miss — generating EPUB..."
- **Non-blocking errors** — "Could not load recommendations"

They don't work well for:
- **Critical errors** — "Could not load fic metadata" (the page is broken)
- **Action-required errors** — "Rate limited" (user needs to wait)

### Toast Positioning and Timing

Toasts typically appear in the bottom-right or top-right corner of the screen. They auto-dismiss after a few seconds (typically 3-5 seconds for success, 8-10 seconds for errors).

```typescript
// Custom toast with longer timeout
toast.error('Something went wrong!', { duration: 10000 })

// Toast with action button
toast.success('Export complete!', {
  action: {
    label: 'Download',
    onClick: () => window.open(downloadUrl),
  },
})
```

### Toast Stacking

When multiple toasts appear at once, they "stack" — new toasts push older ones up or down. Most toast libraries handle this automatically. The typical limit is 3-5 visible toasts at a time.

> **Try It Yourself!** Think about a feature you use daily (like a social media app). When you "like" a post, what feedback do you get? A toast notification? A counter incrementing? A color change? Think about what kind of feedback your FicHub features should give.

### When to Use Each Feedback Type

| Feedback Type | Use When | Example |
|---------------|----------|---------|
| Toast (success) | Non-critical success | "Vote recorded!" |
| Toast (warning) | Transient issue, no action needed | "Cache miss — generating..." |
| Toast (error) | Non-blocking failure | "Could not load recs" |
| Error card | Critical failure, page is broken | "Failed to load fic" |
| Inline error | Field validation failed | "URL is required" |
| Spinner | Request in progress | "Loading..." |
| Disabled button | Preventing duplicate submissions | Button greyed out during request |
| Progress bar | Multi-step operation | "Step 2 of 4: Generating EPUB..." |

Each type of feedback serves a specific purpose. The key is matching the severity and duration of the feedback to the importance of the event.

## Putting It All Together: A Complete Component

Let's build a complete component that demonstrates all these patterns:

```svelte
<script>
  import { fetchExport, ApiError } from '$lib/api/api'
  import Spinner from '$lib/ui/shared/loader/Spinner.svelte'
  import ErrorCard from '$lib/ui/info/ErrorContainer.svelte'

  let { url = '' } = $props()

  let result = $state(null)
  let error = $state(null)
  let loading = $state(false)
  let lastUrl = $state('')

  async function handleExport() {
    if (!url.trim()) {
      error = new ApiError(0, { msg: 'Please enter a URL.' })
      return
    }

    loading = true
    error = null
    result = null
    lastUrl = url

    try {
      result = await fetchExport(url)
    } catch (err) {
      if (err instanceof ApiError) {
        error = err
      } else {
        error = new ApiError(0, {
          msg: 'Could not connect to the server.',
        })
      }
    } finally {
      loading = false
    }
  }

  function handleRetry() {
    url = lastUrl
    handleExport()
  }
</script>

<div class="export-panel">
  <div class="input-row">
    <input
      type="url"
      bind:value={url}
      placeholder="Enter fanfiction URL..."
      disabled={loading}
    />
    <button
      onclick={handleExport}
      disabled={loading || !url.trim()}
    >
      {#if loading}
        <span class="spinner-small"></span>
        Exporting...
      {:else}
        Export EPUB
      {/if}
    </button>
  </div>

  {#if loading}
    <Spinner message="Fetching story metadata..." />
  {:else if error}
    <ErrorCard
      title={error.title}
      description={error.description}
      retryable={error.retryable}
      onretry={handleRetry}
    />
  {:else if result}
    <div class="result-card">
      <h3>{result.meta.title}</h3>
      <p>by {result.meta.author}</p>
      <p>{result.meta.words.toLocaleString()} words · {result.meta.chapters} chapters</p>

      {#if result.epub_url}
        <a href={result.epub_url} class="download-button">
          Download EPUB
        </a>
      {:else}
        <p class="warning">Download not available (greylisted or blacklisted).</p>
      {/if}
    </div>
  {/if}
</div>
```

This component demonstrates:
- **Loading state** — spinner + disabled input
- **Error handling** — error card with retry
- **Success state** — result display with download link
- **Input validation** — checking for empty URL
- **Retry mechanism** — using `lastUrl` to remember the failed request
- **Graceful degradation** — showing a warning when download isn't available

## Practice: Add Error Handling to a Component

Find a component in your project that makes an API call but doesn't handle errors. Add:

1. A `loading` state variable
2. An `error` state variable
3. A `try/catch` block around the API call
4. Conditional rendering for loading, error, and success states
5. A retry button for retryable errors

```svelte
<script>
  import { ApiError } from '$lib/api/api'

  let data = $state(null)
  let error = $state(null)
  let loading = $state(false)

  async function loadData() {
    loading = true
    error = null

    try {
      // Your API call here
      data = await someApiCall()
    } catch (err) {
      error = err instanceof ApiError ? err : new ApiError(0, { msg: 'Network error' })
    } finally {
      loading = false
    }
  }
</script>

{#if loading}
  <p>Loading...</p>
{:else if error}
  <p>Error: {error.description}</p>
  <button onclick={loadData}>Try Again</button>
{:else}
  <!-- Display your data here -->
{/if}
```

### Exercise: Build a Complete Error-Handled Export Page

Here's a more comprehensive exercise. Build a complete export page with:

1. An input field for the URL
2. A submit button (disabled during loading)
3. Loading spinner during the API call
4. Error card with retry button on failure
5. Result display with download links on success
6. Input validation (check for empty URL before calling API)

```svelte
<script>
  import { fetchExport, ApiError } from '$lib/api/api'

  let url = $state('')
  let result = $state(null)
  let error = $state(null)
  let loading = $state(false)

  async function handleExport() {
    // Step 1: Validate input
    if (!url.trim()) {
      error = new ApiError(0, { msg: 'Please enter a URL.' })
      return
    }

    // Step 2: Set loading state
    loading = true
    error = null
    result = null

    // Step 3: Make the API call
    try {
      result = await fetchExport(url)
    } catch (err) {
      error = err instanceof ApiError ? err : new ApiError(0, { msg: 'Network error' })
    } finally {
      loading = false
    }
  }

  function handleKeydown(event) {
    if (event.key === 'Enter') handleExport()
  }
</script>

<div>
  <input
    type="url"
    bind:value={url}
    placeholder="Enter fanfiction URL..."
    onkeydown={handleKeydown}
    disabled={loading}
  />
  <button onclick={handleExport} disabled={loading || !url.trim()}>
    {loading ? 'Exporting...' : 'Export'}
  </button>

  {#if loading}
    <p>Fetching story metadata...</p>
  {:else if error}
    <div class="error-card">
      <p>{error.description}</p>
      {#if error.retryable}
        <button onclick={handleExport}>Try Again</button>
      {/if}
    </div>
  {:else if result}
    <div class="result">
      <h3>{result.meta.title}</h3>
      <p>by {result.meta.author}</p>
      <p>{result.meta.words.toLocaleString()} words · {result.meta.chapters} chapters</p>
      {#if result.epub_url}
        <a href={result.epub_url}>Download EPUB</a>
      {/if}
    </div>
  {/if}
</div>
```

This exercise demonstrates every error handling pattern from this chapter:
- **Input validation** — checking before the API call
- **Loading state** — disabling the button and showing a message
- **Error handling** — try/catch with error classification
- **Retry mechanism** — retry button for retryable errors
- **Success display** — showing the result with download link

## Summary

In this chapter, you learned:

- **Error handling is not optional** — things WILL go wrong, and your app needs to handle them gracefully
- There are **ten common failure modes** for API calls — network errors, server errors, rate limits, invalid input, and more
- The **`ApiError` class** wraps status codes, error codes, and user-friendly messages
- **`try/catch` blocks** are the safety net for async operations — always use them!
- **Error cards** show friendly, helpful messages instead of raw errors
- **Loading states** (spinners, disabled buttons) keep users informed during waits
- The **error lifecycle** (error → retry → success) is a pattern for resilient UIs
- **Network errors** (no internet) are different from **API errors** (server responded with error) — handle them differently
- **Graceful degradation** means showing partial data when some requests fail
- **Toast notifications** work for brief, non-critical messages — use the right feedback type for each situation
- Always **validate input** before making API calls
- Use **`Promise.allSettled`** for parallel independent requests
- **Unset error states** when retrying — clear the old error before starting a new request
- **AbortController** lets you cancel stale requests and add timeouts

### Error Handling Checklist

Before shipping any feature, run through this checklist:

- [ ] Every API call is wrapped in try/catch
- [ ] Loading states are shown during requests
- [ ] Error states display user-friendly messages
- [ ] Retry buttons are shown for retryable errors
- [ ] Input is validated before API calls
- [ ] Error states are cleared when retrying
- [ ] Partial data is shown when some requests fail
- [ ] Toast notifications are used for non-critical feedback
- [ ] No raw error messages are shown to users
- [ ] Network errors are handled differently from API errors

### The Psychology of Error Messages

Error messages are a conversation with your user. Bad error messages blame the user or the server. Good error messages explain what happened and what the user can do.

| ❌ Bad Message | ✅ Good Message |
|---------------|----------------|
| "Error -5" | "This URL is not supported" |
| "undefined is not a function" | "Something went wrong. Please try again." |
| "Internal Server Error" | "The server is having issues. Try again in a few minutes." |
| "Failed to fetch" | "Could not connect. Check your internet connection." |
| "Invalid input" | "Please enter a valid fanfiction URL." |

The key principles:
1. **Say what happened** — "This URL is not supported"
2. **Say what to do** — "Try a different URL"
3. **Be specific when possible** — "Rate limited. Wait 30 seconds."
4. **Be generic when necessary** — "Something went wrong" (for unexpected errors)
5. **Never show technical details** — No stack traces, no error codes, no "undefined"

In the next chapter, we'll build the search API client — the most complex part of our API layer. We'll design filters, handle pagination, and work with tag types. Let's keep going!

---

# Chapter 18: The Search API Client

In the previous chapters, we learned about REST APIs, built TypeScript types, created an API client, and mastered error handling. Now let's apply all of that to the most complex feature in FicHub: **search**.

Search is where everything comes together. It has the most parameters, the most complex response structure, and the most interesting UI patterns. If you can build a search client, you can build anything.

## The Search Endpoint: GET /api/v0/search

FicHub's search endpoint uses PostgreSQL's full-text search engine to find fanfiction stories. Here's the endpoint:

```
GET /api/v0/search?q=harry+potter&include_tags=1:Harry+Potter&complete=true&sort=-words&page=1&per_page=20
```

That's a lot of parameters! Let's break them down by looking at the Rust code in `src/search/routes.rs`:

```rust
pub struct SearchQueryParams {
    pub q: Option<String>,               // Full-text search query
    pub include_tags: Option<String>,     // Comma-separated "type_id:name" pairs
    pub exclude_tags: Option<String>,     // Comma-separated "type_id:name" pairs
    pub include_any_tags: Option<String>, // Comma-separated "type_id:name" pairs
    pub min_words: Option<i64>,           // Minimum word count
    pub max_words: Option<i64>,           // Maximum word count
    pub min_chapters: Option<i32>,        // Minimum chapter count
    pub max_chapters: Option<i32>,        // Maximum chapter count
    pub complete: Option<bool>,           // Only complete fics?
    pub source: Option<String>,           // Source site filter
    pub date_from: Option<String>,        // ISO 8601 datetime
    pub date_to: Option<String>,          // ISO 8601 datetime
    pub sort: Option<String>,             // Sort field
    pub page: Option<usize>,              // Page number
    pub per_page: Option<usize>,          // Results per page
}
```

That's 15 parameters! This is why we need a well-designed TypeScript interface.

## Designing SearchFilters: All Parameters

Let's create a TypeScript interface that matches all these parameters:

```typescript
/**
 * Search filters for GET /api/v0/search
 *
 * All fields are optional. The server provides sensible defaults
 * for missing values.
 */
export interface SearchFilters {
  /** Full-text search query */
  q?: string

  /** Tags that ALL must be present (AND filter) — format: "type_id:name,type_id:name" */
  include_tags?: string

  /** Tags that NONE can be present (exclude filter) — format: "type_id:name,type_id:name" */
  exclude_tags?: string

  /** Tags where at least ONE must be present (OR filter) — format: "type_id:name,type_id:name" */
  include_any_tags?: string

  /** Minimum word count */
  min_words?: number

  /** Maximum word count */
  max_words?: number

  /** Minimum chapter count */
  min_chapters?: number

  /** Maximum chapter count */
  max_chapters?: number

  /** Only show completed fics? */
  complete?: boolean

  /** Source site filter (e.g., "archiveofourown.org") */
  source?: string

  /** Only show fics updated after this date (ISO 8601) */
  date_from?: string

  /** Only show fics updated before this date (ISO 8601) */
  date_to?: string

  /** Sort order (see SORT_OPTIONS) */
  sort?: string

  /** Page number (1-indexed) */
  page?: number

  /** Results per page (max: configured server limit) */
  per_page?: number
}
```

> **Watch Out!** The `include_tags` format is tricky! It's a comma-separated string of `type_id:name` pairs, like `"1:Harry Potter,2:Hermione Granger"`. The `type_id` is a number that identifies the tag category. Don't mix this up with regular query parameters.

## SORT_OPTIONS, COMPLETE_OPTIONS, SOURCE_OPTIONS Constants

To prevent typos and provide autocomplete, we define constants for the valid option values:

```typescript
/**
 * Valid sort options for search.
 * The "-" prefix means descending order.
 */
export const SORT_OPTIONS = {
  RELEVANCE: '-relevance',   // Best match first (default when q is present)
  DATE: '-date',             // Most recently updated first (default when no q)
  WORDS: '-words',           // Most words first
  CHAPTERS: '-chapters',     // Most chapters first
  TITLE: '-title',           // Alphabetical by title
} as const

export type SortOption = (typeof SORT_OPTIONS)[keyof typeof SORT_OPTIONS]

/**
 * Completion filter options.
 */
export const COMPLETE_OPTIONS = {
  ALL: null,        // Show all fics (default)
  COMPLETE: true,   // Only completed fics
  INCOMPLETE: false, // Only incomplete/ongoing fics
} as const

/**
 * Known source sites.
 * Each source is identified by its domain name.
 */
export const SOURCE_OPTIONS = {
  AO3: 'archiveofourown.org',
  FF_NET: 'fanfiction.net',
  FICTIONPRESS: 'fictionpress.com',
  ADULT_FANFICTION: 'adultfanfiction.org',
  HP_FANFIC: 'hpfanfic.com',
} as const

export type SourceOption = (typeof SOURCE_OPTIONS)[keyof typeof SOURCE_OPTIONS]
```

### The `as const` Keyword

The `as const` keyword is a TypeScript feature that makes objects **immutable** and their values **literal types**:

```typescript
const SORT_OPTIONS = {
  RELEVANCE: '-relevance',
  DATE: '-date',
} as const

// Without as const:
// type: { RELEVANCE: string, DATE: string }
// SORT_OPTIONS.RELEVANCE is type: string

// With as const:
// type: { readonly RELEVANCE: "-relevance", readonly DATE: "-date" }
// SORT_OPTIONS.RELEVANCE is type: "-relevance" (a literal!)
```

This means TypeScript knows *exactly* which values are valid. If you try to use `sort: 'relevance'` (missing the `-`), TypeScript will catch it!

### Why `as const` Matters for Constants

Without `as const`, TypeScript widens the types:

```typescript
// ❌ Without as const — too loose
const COLORS = { RED: 'red', BLUE: 'blue' }
// COLORS.RED is type: string (too broad!)

// ✅ With as const — precise
const COLORS = { RED: 'red', BLUE: 'blue' } as const
// COLORS.RED is type: 'red' (exactly 'red'!)
```

This prevents typos and ensures you only use valid values.

## defaultFilters() Factory Function

When the user opens the search page, we need sensible defaults. A factory function creates a fresh copy of default filters:

```typescript
/**
 * Create a fresh set of default search filters.
 *
 * Using a factory function (not a constant object) ensures
 * each call returns a NEW object, preventing shared state bugs.
 */
export function defaultFilters(): SearchFilters {
  return {
    q: '',
    include_tags: undefined,
    exclude_tags: undefined,
    include_any_tags: undefined,
    min_words: undefined,
    max_words: undefined,
    min_chapters: undefined,
    max_chapters: undefined,
    complete: undefined,
    source: undefined,
    date_from: undefined,
    date_to: undefined,
    sort: SORT_OPTIONS.DATE,
    page: 1,
    per_page: 20,
  }
}
```

> **Watch Out!** Why not just use a constant like `const DEFAULT_FILTERS = { ... }`? Because objects are passed by reference in JavaScript. If you did `let filters = DEFAULT_FILTERS`, then `filters.q = 'test'` would also change `DEFAULT_FILTERS`! The factory function gives you a fresh copy every time.

## buildSearchQuery(): Converting Filters to URL Params

Now we need to convert our `SearchFilters` object into a URL query string. But there's a twist — tag filters need special formatting:

```typescript
/**
 * Build a tag filter string from individual tag entries.
 *
 * @param tags - Array of tag objects with type_id and name
 * @returns Comma-separated "type_id:name" string
 *
 * @example
 * buildTagFilter([
 *   { type_id: 1, name: 'Harry Potter' },
 *   { type_id: 2, name: 'Hermione Granger' }
 * ])
 * // Returns: "1:Harry Potter,2:Hermione Granger"
 */
export function buildTagFilter(
  tags: Array<{ type_id: number; name: string }>
): string {
  return tags.map(t => `${t.type_id}:${t.name}`).join(',')
}

/**
 * Convert search filters to a URL query string for GET /api/v0/search.
 *
 * Handles special cases:
 * - Omits undefined/null/empty values
 * - Formats tag arrays as "type_id:name" strings
 * - Encodes special characters properly
 */
export function buildSearchQuery(filters: SearchFilters): string {
  const params: Record<string, string> = {}

  // Text search
  if (filters.q) {
    params.q = filters.q
  }

  // Tag filters (already formatted strings)
  if (filters.include_tags) {
    params.include_tags = filters.include_tags
  }
  if (filters.exclude_tags) {
    params.exclude_tags = filters.exclude_tags
  }
  if (filters.include_any_tags) {
    params.include_any_tags = filters.include_any_tags
  }

  // Word count range
  if (filters.min_words !== undefined) {
    params.min_words = String(filters.min_words)
  }
  if (filters.max_words !== undefined) {
    params.max_words = String(filters.max_words)
  }

  // Chapter count range
  if (filters.min_chapters !== undefined) {
    params.min_chapters = String(filters.min_chapters)
  }
  if (filters.max_chapters !== undefined) {
    params.max_chapters = String(filters.max_chapters)
  }

  // Completion status
  if (filters.complete !== undefined) {
    params.complete = String(filters.complete)
  }

  // Source site
  if (filters.source) {
    params.source = filters.source
  }

  // Date range
  if (filters.date_from) {
    params.date_from = filters.date_from
  }
  if (filters.date_to) {
    params.date_to = filters.date_to
  }

  // Sort order
  if (filters.sort) {
    params.sort = filters.sort
  }

  // Pagination
  if (filters.page !== undefined && filters.page > 1) {
    params.page = String(filters.page)
  }
  if (filters.per_page !== undefined && filters.per_page !== 20) {
    params.per_page = String(filters.per_page)
  }

  // Build the query string
  const entries = Object.entries(params)
    .map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`)

  return entries.length > 0 ? `?${entries.join('&')}` : ''
}
```

Notice a few design decisions:

1. **We skip default values** — If `page` is 1 and `per_page` is 20, we don't include them in the URL. This keeps URLs clean and avoids sending unnecessary parameters.

2. **We use `String()` to convert numbers** — This is safer than template literals for edge cases.

3. **We handle the tag format** — Tags are already formatted strings, so we just pass them through.

4. **We encode special characters** — `encodeURIComponent` handles spaces, ampersands, and other problematic characters.

## The search() Function: Executing the Request

Finally, the search function itself:

```typescript
/**
 * Search for fanfiction stories.
 *
 * @param filters - Search filters (all optional)
 * @returns Paginated search results with metadata and tags
 *
 * @example
 * // Basic text search
 * const results = await search({ q: 'harry potter' })
 *
 * // Filtered search
 * const results = await search({
 *   q: 'time travel',
 *   include_tags: '1:Harry Potter',
 *   complete: true,
 *   sort: '-words',
 *   min_words: 50000,
 * })
 */
export async function search(filters: SearchFilters = {}): Promise<SearchResponse> {
  const query = buildSearchQuery(filters)
  return request<SearchResponse>(`/api/v0/search${query}`)
}
```

That's the public API! The complexity is hidden behind `buildSearchQuery()`.

### Usage Examples

```typescript
// Simple search
const results = await search({ q: 'harry potter' })
console.log(`Found ${results.total} results`)
for (const fic of results.results) {
  console.log(`${fic.title} by ${fic.author} (${fic.words} words)`)
}

// Advanced search with multiple filters
const advancedResults = await search({
  q: 'time travel',
  include_tags: '1:Harry Potter',         // Must have Harry Potter fandom tag
  exclude_tags: '4:Character Bashing',    // Must NOT have Character Bashing tag
  complete: true,                          // Only completed fics
  sort: '-words',                          // Sort by word count (highest first)
  min_words: 50000,                        // At least 50k words
  per_page: 10,                            // Show 10 results per page
})

// Pagination
const page1 = await search({ q: 'harry potter', page: 1 })
const page2 = await search({ q: 'harry potter', page: 2 })
```

## SearchResponse and SearchResult Types

Let's look at the response types in detail:

```typescript
/**
 * A single search result — one fanfiction story.
 */
export interface SearchResult {
  /** Unique fic identifier */
  url_id: string

  /** Story title */
  title: string

  /** Author name */
  author: string

  /** Source site domain (e.g., "archiveofourown.org") */
  source: string

  /** Total word count */
  words: number

  /** Number of chapters */
  chapters: number

  /** Completion status: "ongoing", "complete", "hiatus", or "cancelled" */
  status: string

  /** Story description/summary (may contain HTML) */
  description: string

  /** ISO 8601 timestamp of last update, or null if unknown */
  updated: string | null

  /** Full-text search relevance rank (0.0 to 1.0), or null if not doing text search */
  rank: number | null

  /** Tags associated with this fic */
  tags: SearchTag[]

  /** Total number of freeform tags (for "show more" UI) */
  total_freeform: number
}

/**
 * A tag attached to a search result.
 */
export interface SearchTag {
  /** Tag name (e.g., "Harry Potter", "Time Travel", "Angst") */
  name: string

  /** Tag category name (e.g., "Fandom", "Character", "Freeform") */
  type: string

  /** Tag category ID (see TAG_TYPES) */
  type_id: number

  /** Tag confidence score (higher = more certain) */
  score: number
}

/**
 * Paginated search response.
 */
export interface SearchResponse {
  /** Total number of matching fics (across all pages) */
  total: number

  /** Current page number (1-indexed) */
  page: number

  /** Results per page */
  per_page: number

  /** The search results for this page */
  results: SearchResult[]
}
```

### The `rank` Field

The `rank` field is interesting. It comes from PostgreSQL's full-text search ranking function (`ts_rank`). It measures how well a fic matches the search query:

- `rank: 0.8` — Very strong match
- `rank: 0.3` — Moderate match
- `rank: null` — Not doing text search (used tag filters only)

You can use this to show "best matches first" when doing text search.

### The `tags` Array

Each search result comes with its tags. Tags are grouped by type (Fandom, Character, Relationship, Freeform). The `score` field indicates confidence — tags with higher scores are more relevant to the fic.

This is important for the UI: you might want to show the top 5 tags and hide the rest, using the `total_freeform` count to show a "+3 more" link.

### The `updated` Field

The `updated` field is an ISO 8601 timestamp like `"2024-01-15T12:30:00Z"`. It tells you when the fic was last updated. This is useful for sorting by "most recent" and for displaying relative time like "3 days ago."

When this field is `null`, the fic's update date is unknown (this can happen for very old fics scraped from sites that don't provide dates).

### The `description` Field

The description field contains the fic's summary or synopsis. In FicHub, this is typically HTML text (like `<p>A story about...</p>`). When displaying this in the UI, you'll need to either:
1. Render it as HTML (with proper sanitization)
2. Strip the HTML tags and show plain text
3. Show a truncated version with "Read more" expand

> **Watch Out!** Never render raw HTML from the server without sanitization! Even though FicHub controls the content, scraped data from third-party sites could contain malicious HTML. Always sanitize or strip HTML before rendering.

## Tag Types: Fandom (1), Character (2), Relationship (3), Freeform (4)

FicHub uses a numeric tag type system. Each tag has a `type_id` that identifies its category:

```typescript
/**
 * Tag type IDs used by the FicHub tagging system.
 *
 * These match the Rust backend's tag_type_id values.
 */
export const TAG_TYPES = {
  /** Fandom tags (e.g., "Harry Potter", "Marvel", "Star Wars") */
  FANDOM: 1,

  /** Character tags (e.g., "Hermione Granger", "Tony Stark") */
  CHARACTER: 2,

  /** Relationship tags (e.g., "Hermione Granger/Harry Potter") */
  RELATIONSHIP: 3,

  /** Freeform tags (e.g., "Time Travel", "Angst", "Humor") */
  FREEFORM: 4,

  /** Warning tags (e.g., "Graphic Depictions Of Violence") */
  WARNING: 5,

  /** Category tags (e.g., "M/M", "F/M", "Gen") */
  CATEGORY: 6,
} as const

export type TagTypeId = (typeof TAG_TYPES)[keyof typeof TAG_TYPES]

/**
 * Human-readable names for tag types.
 * Use this to display tag categories in the UI.
 */
export const TAG_TYPE_NAMES: Record<number, string> = {
  [TAG_TYPES.FANDOM]: 'Fandom',
  [TAG_TYPES.CHARACTER]: 'Character',
  [TAG_TYPES.RELATIONSHIP]: 'Relationship',
  [TAG_TYPES.FREEFORM]: 'Freeform',
  [TAG_TYPES.WARNING]: 'Warning',
  [TAG_TYPES.CATEGORY]: 'Category',
}
```

### Building Tag Filter Strings

When the user selects tags in the UI, we need to build filter strings. The format is `type_id:name` pairs separated by commas. The `type_id` identifies the tag category (1=Fandom, 2=Character, etc.), and the `name` is the tag text.

Here are some examples:

```
# Single fandom filter
1:Harry Potter

# Multiple fandoms (AND — must have ALL of these)
1:Harry Potter,1:Marvel

# Character filter
2:Harry Potter,2:Hermione Granger

# Freeform filter (tags like "Time Travel", "Angst", etc.)
4:Time Travel,4:Angst

# Mixed types
1:Harry Potter,2:Hermione Granger,4:Time Travel
```

When the user selects tags in the UI, we convert them to this format:

```typescript
// User selects these tags in the UI:
const selectedTags = [
  { type_id: 1, name: 'Harry Potter' },
  { type_id: 4, name: 'Time Travel' },
  { type_id: 4, name: 'Fix-It Fic' },
]

// Build the filter string:
const filter = buildIncludeFilter(selectedTags)
// Result: "1:Harry Potter,4:Time Travel,4:Fix-It Fic"
```

And when we need to display selected tags back to the user, we parse the filter string:

```typescript
// Parse the filter string back to tag objects:
const tags = parseTagFilter("1:Harry Potter,4:Time Travel,4:Fix-It Fic")
// Result: [
//   { type_id: 1, name: "Harry Potter" },
//   { type_id: 4, name: "Time Travel" },
//   { type_id: 4, name: "Fix-It Fic" }
// ]
```

This bidirectional conversion is essential for the search UI — the user interacts with tag objects, but the API expects filter strings.

```typescript
/**
 * Build an include_tags filter from selected tags.
 *
 * @param tags - Array of selected tags with type_id and name
 * @returns Formatted filter string (e.g., "1:Harry Potter,4:Time Travel")
 *
 * @example
 * buildIncludeFilter([
 *   { type_id: 1, name: 'Harry Potter' },
 *   { type_id: 4, name: 'Time Travel' }
 * ])
 * // Returns: "1:Harry Potter,4:Time Travel"
 */
export function buildIncludeFilter(
  tags: Array<{ type_id: number; name: string }>
): string | undefined {
  if (tags.length === 0) return undefined
  return tags.map(t => `${t.type_id}:${t.name}`).join(',')
}

/**
 * Parse a tag filter string back into individual tags.
 *
 * @param filter - The filter string (e.g., "1:Harry Potter,2:Ron Weasley")
 * @returns Array of parsed tags
 *
 * @example
 * parseTagFilter("1:Harry Potter,2:Ron Weasley")
 * // Returns: [
 * //   { type_id: 1, name: "Harry Potter" },
 * //   { type_id: 2, name: "Ron Weasley" }
 * // ]
 */
export function parseTagFilter(
  filter: string | undefined
): Array<{ type_id: number; name: string }> {
  if (!filter) return []

  return filter.split(',').map(part => {
    const colonIndex = part.indexOf(':')
    if (colonIndex === -1) {
      throw new Error(`Invalid tag filter format: "${part}". Expected "type_id:name".`)
    }

    const type_id = parseInt(part.slice(0, colonIndex), 10)
    const name = part.slice(colonIndex + 1)

    if (isNaN(type_id)) {
      throw new Error(`Invalid type_id in tag filter: "${part}".`)
    }

    return { type_id, name }
  })
}
```

These helper functions make it easy to convert between user-friendly tag objects and the server's filter format.

## Putting It All Together: The Search Module

Here's the complete search module:

```typescript
// =============================================================================
// search.ts — FicHub Search API Client
// =============================================================================

import { request } from './api'
import type { SearchFilters, SearchResponse } from './types'

// --- Constants ------------------------------------------------------------

export const SORT_OPTIONS = {
  RELEVANCE: '-relevance',
  DATE: '-date',
  WORDS: '-words',
  CHAPTERS: '-chapters',
  TITLE: '-title',
} as const

export const TAG_TYPES = {
  FANDOM: 1,
  CHARACTER: 2,
  RELATIONSHIP: 3,
  FREEFORM: 4,
  WARNING: 5,
  CATEGORY: 6,
} as const

export const TAG_TYPE_NAMES: Record<number, string> = {
  1: 'Fandom',
  2: 'Character',
  3: 'Relationship',
  4: 'Freeform',
  5: 'Warning',
  6: 'Category',
}

export const SOURCE_OPTIONS = {
  AO3: 'archiveofourown.org',
  FF_NET: 'fanfiction.net',
  FICTIONPRESS: 'fictionpress.com',
  ADULT_FANFICTION: 'adultfanfiction.org',
  HP_FANFIC: 'hpfanfic.com',
} as const

// --- Filter helpers --------------------------------------------------------

export function defaultFilters(): SearchFilters {
  return {
    q: '',
    sort: SORT_OPTIONS.DATE,
    page: 1,
    per_page: 20,
  }
}

export function buildTagFilter(
  tags: Array<{ type_id: number; name: string }>
): string | undefined {
  if (tags.length === 0) return undefined
  return tags.map(t => `${t.type_id}:${t.name}`).join(',')
}

export function parseTagFilter(
  filter: string | undefined
): Array<{ type_id: number; name: string }> {
  if (!filter) return []
  return filter.split(',').map(part => {
    const colonIndex = part.indexOf(':')
    if (colonIndex === -1) throw new Error(`Invalid tag filter: "${part}"`)
    return {
      type_id: parseInt(part.slice(0, colonIndex), 10),
      name: part.slice(colonIndex + 1),
    }
  })
}

// --- Query builder ---------------------------------------------------------

export function buildSearchQuery(filters: SearchFilters): string {
  const params: Record<string, string> = {}

  if (filters.q) params.q = filters.q
  if (filters.include_tags) params.include_tags = filters.include_tags
  if (filters.exclude_tags) params.exclude_tags = filters.exclude_tags
  if (filters.include_any_tags) params.include_any_tags = filters.include_any_tags
  if (filters.min_words !== undefined) params.min_words = String(filters.min_words)
  if (filters.max_words !== undefined) params.max_words = String(filters.max_words)
  if (filters.min_chapters !== undefined) params.min_chapters = String(filters.min_chapters)
  if (filters.max_chapters !== undefined) params.max_chapters = String(filters.max_chapters)
  if (filters.complete !== undefined) params.complete = String(filters.complete)
  if (filters.source) params.source = filters.source
  if (filters.date_from) params.date_from = filters.date_from
  if (filters.date_to) params.date_to = filters.date_to
  if (filters.sort) params.sort = filters.sort
  if (filters.page !== undefined && filters.page > 1) params.page = String(filters.page)
  if (filters.per_page !== undefined && filters.per_page !== 20) params.per_page = String(filters.per_page)

  const entries = Object.entries(params)
    .map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`)

  return entries.length > 0 ? `?${entries.join('&')}` : ''
}

// --- API function ----------------------------------------------------------

/**
 * Search for fanfiction stories.
 *
 * @param filters - Search filters (all optional)
 * @returns Paginated search results
 */
export async function search(filters: SearchFilters = {}): Promise<SearchResponse> {
  const query = buildSearchQuery(filters)
  return request<SearchResponse>(`/api/v0/search${query}`)
}
```

## Practice: Test the Search API with curl

Let's practice by testing the search API with curl commands. This helps you understand exactly what the server returns and what each parameter does.

### Basic Text Search

```bash
curl -s "http://localhost:8004/api/v0/search?q=harry+potter" | python3 -m json.tool
```

This searches for fics containing "harry potter" in the title, author, or description. The `+` in the URL represents a space. PostgreSQL's full-text search handles word matching, stemming (so "running" matches "run"), and relevance ranking.

### Search with Tag Filter

```bash
curl -s "http://localhost:8004/api/v0/search?q=time+travel&include_tags=1:Harry+Potter" | python3 -m json.tool
```

This searches for fics that contain "time travel" AND have the Harry Potter fandom tag. The `include_tags` parameter uses AND logic — all specified tags must be present.

### Search with Multiple Filters

```bash
curl -s "http://localhost:8004/api/v0/search?q=harry+potter&complete=true&sort=-words&min_words=50000&per_page=5" | python3 -m json.tool
```

This searches for completed Harry Potter fics with at least 50k words, sorted by word count (highest first), showing only 5 results. Notice how multiple filters stack — the results must match ALL of them.

### Search with Exclusion

```bash
curl -s "http://localhost:8004/api/v0/search?q=harry+potter&exclude_tags=4:Character+Bashing" | python3 -m json.tool
```

This searches for Harry Potter fics but excludes any with the "Character Bashing" freeform tag. The `exclude_tags` parameter uses NOT logic — fics matching these tags are removed from results.

### Pagination

```bash
# Page 1 (first 5 results)
curl -s "http://localhost:8004/api/v0/search?q=harry+potter&page=1&per_page=5" | python3 -m json.tool

# Page 2 (next 5 results)
curl -s "http://localhost:8004/api/v0/search?q=harry+potter&page=2&per_page=5" | python3 -m json.tool
```

Notice how the `results` array changes between pages, but `total` stays the same. This is how pagination works — the `total` tells you how many results exist across all pages.

### Sort Options

```bash
# Sort by relevance (best match first)
curl -s "http://localhost:8004/api/v0/search?q=harry+potter&sort=-relevance" | python3 -m json.tool

# Sort by word count (longest first)
curl -s "http://localhost:8004/api/v0/search?q=harry+potter&sort=-words" | python3 -m json.tool

# Sort by date (most recently updated first)
curl -s "http://localhost:8004/api/v0/search?q=harry+potter&sort=-date" | python3 -m json.tool

# Sort alphabetically by title
curl -s "http://localhost:8004/api/v0/search?q=harry+potter&sort=-title" | python3 -m json.tool
```

The `-` prefix means descending order. For `-title`, descending means Z→A (reverse alphabetical).

### Word Count Range

```bash
# Only long fics (100k+ words)
curl -s "http://localhost:8004/api/v0/search?min_words=100000" | python3 -m json.tool

# Only short fics (under 10k words)
curl -s "http://localhost:8004/api/v0/search?max_words=10000" | python3 -m json.tool

# Medium-length fics (50k-100k words)
curl -s "http://localhost:8004/api/v0/search?min_words=50000&max_words=100000" | python3 -m json.tool
```

### Source Site Filter

```bash
# Only AO3 fics
curl -s "http://localhost:8004/api/v0/search?source=archiveofourown.org" | python3 -m json.tool

# Only FF.net fics
curl -s "http://localhost:8004/api/v0/search?source=fanfiction.net" | python3 -m json.tool
```

### No Results

```bash
# Search for something that probably doesn't exist
curl -s "http://localhost:8004/api/v0/search?q=xyzzy12345nonexistent" | python3 -m json.tool
```

You'll see `total: 0` and an empty `results` array. This is important for the UI — always check if `total` is zero before trying to display results!

> **Try It Yourself!** Run these curl commands and look at the response structure. Pay attention to:
> - The `total` field — how many results match?
> - The `page` and `per_page` fields — pagination info
> - The `tags` array in each result — what tag types are present?
> - The `rank` field — is it a number or null?
>
> Can you figure out the maximum number of pages? (Hint: it's `total / per_page`)
>
> Try these extra experiments:
> 1. What happens if you search with no `q` parameter? (All fics, sorted by date)
> 2. What happens if you set `per_page=100`? (Does the server cap it?)
> 3. What happens if you set `page=0` or `page=-1`? (Edge case handling)
> 4. Can you find fics with more than 1 million words? (`min_words=1000000`)

## Building a Search UI: Putting It All Together

Let's see how the search module works in a Svelte component:

```svelte
<script>
  import { search, defaultFilters, SORT_OPTIONS, TAG_TYPE_NAMES } from '$lib/api/search'
  import { ApiError } from '$lib/api/api'

  let filters = $state(defaultFilters())
  let results = $state(null)
  let loading = $state(false)
  let error = $state(null)
  let totalPages = $derived(
    results ? Math.ceil(results.total / results.per_page) : 0
  )

  async function handleSearch() {
    loading = true
    error = null
    filters.page = 1  // Reset to first page on new search

    try {
      results = await search(filters)
    } catch (err) {
      error = err instanceof ApiError ? err : new ApiError(0, { msg: 'Search failed' })
    } finally {
      loading = false
    }
  }

  async function goToPage(page) {
    filters.page = page
    await handleSearch()
  }

  function handleKeydown(event) {
    if (event.key === 'Enter') {
      handleSearch()
    }
  }
</script>

<div class="search-page">
  <!-- Search input -->
  <div class="search-bar">
    <input
      type="text"
      bind:value={filters.q}
      placeholder="Search fanfiction..."
      onkeydown={handleKeydown}
    />
    <button onclick={handleSearch} disabled={loading}>
      {loading ? 'Searching...' : 'Search'}
    </button>
  </div>

  <!-- Filters -->
  <div class="filters">
    <select bind:value={filters.sort}>
      <option value={SORT_OPTIONS.DATE}>Most Recent</option>
      <option value={SORT_OPTIONS.RELEVANCE}>Best Match</option>
      <option value={SORT_OPTIONS.WORDS}>Most Words</option>
      <option value={SORT_OPTIONS.CHAPTERS}>Most Chapters</option>
    </select>

    <label>
      <input type="checkbox" bind:checked={filters.complete} />
      Completed only
    </label>
  </div>

  <!-- Results -->
  {#if loading}
    <p>Searching...</p>
  {:else if error}
    <div class="error-card">
      <p>{error.description}</p>
      <button onclick={handleSearch}>Try Again</button>
    </div>
  {:else if results}
    <p class="result-count">
      Found {results.total.toLocaleString()} results
    </p>

    {#each results.results as fic}
      <div class="fic-card">
        <h3>
          <a href="/fics/{fic.url_id}">{fic.title}</a>
        </h3>
        <p class="author">by {fic.author}</p>
        <p class="meta">
          {fic.words.toLocaleString()} words · {fic.chapters} chapters · {fic.status}
        </p>

        {#if fic.tags.length > 0}
          <div class="tags">
            {#each fic.tags.slice(0, 5) as tag}
              <span class="tag tag-{tag.type_id}">
                {tag.name}
              </span>
            {/each}
            {#if fic.tags.length > 5}
              <span class="tag-more">+{fic.tags.length - 5} more</span>
            {/if}
          </div>
        {/if}
      </div>
    {/each}

    <!-- Pagination -->
    {#if totalPages > 1}
      <div class="pagination">
        <button
          onclick={() => goToPage(results.page - 1)}
          disabled={results.page <= 1}
        >
          ← Previous
        </button>

        <span>Page {results.page} of {totalPages}</span>

        <button
          onclick={() => goToPage(results.page + 1)}
          disabled={results.page >= totalPages}
        >
          Next →
        </button>
      </div>
    {/if}
  {/if}
</div>
```

This component demonstrates:

1. **State management** — `filters`, `results`, `loading`, `error` using Svelte 5 runes
2. **Derived state** — `totalPages` computed from results
3. **Search execution** — calling `search()` with current filters
4. **Error handling** — showing error card with retry
5. **Pagination** — Previous/Next buttons, page display
6. **Tag display** — showing top 5 tags with "+N more" overflow
7. **Keyboard support** — Enter key triggers search

> **Try It Yourself!** Extend this component with:
> 1. A word count range filter (min/max inputs)
> 2. A source site dropdown using `SOURCE_OPTIONS`
> 3. An "exclude tags" section
> 4. URL persistence (save filters to the URL so users can share search links)
>
> **Bonus challenge:** Add a "Clear all filters" button that resets to `defaultFilters()`. This is a common UX pattern that users love — it lets them start fresh without refreshing the page.

## Summary

In this chapter, you learned:

- The **search endpoint** has 15 parameters for powerful filtering
- **SearchFilters** TypeScript interface matches all parameters
- **Constants** (`SORT_OPTIONS`, `TAG_TYPES`, `SOURCE_OPTIONS`) prevent typos and provide autocomplete
- **`defaultFilters()`** is a factory function that creates fresh filter objects — never share state!
- **`buildSearchQuery()`** converts filters to URL params, handling special cases like tag formatting
- **`search()`** is the main API function — one line of code with complex logic hidden behind helpers
- **Tag types** (Fandom=1, Character=2, Relationship=3, Freeform=4, Warning=5, Category=6) categorize the tagging system
- **`buildTagFilter()`** and **`parseTagFilter()`** convert between tag objects and filter strings
- **curl** is great for testing API endpoints before building UIs — it's faster than writing code
- **Pagination** uses `page`, `per_page`, and `total` fields

### Search Architecture Recap

Let's review the architecture of our search system:

```
User types in search box
    ↓
Svelte component calls search(filters)
    ↓
search() calls buildSearchQuery(filters)
    ↓
buildSearchQuery() returns URL string: "?q=harry+potter&complete=true&sort=-words"
    ↓
search() calls request<SearchResponse>('/api/v0/search?q=...')
    ↓
request() calls fetch() with the URL
    ↓
Server processes the request (PostgreSQL full-text search + tag filters)
    ↓
Server returns JSON response
    ↓
request() parses JSON and returns typed SearchResponse
    ↓
search() returns SearchResponse to the component
    ↓
Component renders the results
```

This layered architecture keeps concerns separate:
- **SearchFilters** defines what parameters are possible
- **buildSearchQuery()** handles URL encoding and special formatting
- **request()** handles HTTP mechanics (fetch, headers, error checking)
- **search()** is the public API — simple and clean
- **Component** handles UI rendering

Each layer is independently testable and replaceable.

### Performance Tips

When building search UIs, keep these performance tips in mind:

1. **Debounce text input** — Don't search on every keystroke. Wait 300ms after the user stops typing.
### Chapter 16: Building the API Client
- Created the `request()` helper with generic types for type-safe API calls
- Built `ApiError` class for meaningful error handling with status codes and messages
- Created `buildQuery()` for safe URL parameter encoding
- Implemented all API functions: `fetchExport`, `fetchMeta`, `fetchRecommendations`, `fetchVotes`, `submitSuggestion`, `castVote`
- Learned four error handling patterns: bubble up, handle locally, return null, return result tuple
- Understood the API client architecture: components → API functions → request helper → fetch → HTTP

### Chapter 17: Error Handling Patterns
- Mastered try/catch blocks and error classification (network vs API errors)
- Built error cards with friendly messages and retry buttons
- Implemented loading states with spinners and disabled buttons
- Used `Promise.allSettled` for graceful degradation with partial data
- Applied toast notifications for non-critical feedback
- Learned the psychology of error messages — be specific, say what to do, never show technical details
- Built a complete error-handled export page with all patterns combined

### Chapter 18: The Search API Client
- Designed comprehensive SearchFilters with 15 parameters for powerful filtering
- Built constants for sort options, tag types, and source sites with `as const` for type safety
- Created `buildSearchQuery()` with special tag formatting (`type_id:name` pairs)
- Implemented `buildTagFilter()` and `parseTagFilter()` for bidirectional tag conversion
- Tested everything with curl before building the UI — a best practice for API development
- Built a complete search UI with pagination, loading states, and error handling
- Learned performance tips: debounce, cache, cancel stale requests, show loading states

## The Bigger Picture

What we've built in Part 4 is the **communication layer** between frontend and backend. This is one of the most important parts of any web application. Without a solid API layer, even the most beautiful UI is useless — it can't get data from the server.

The patterns we learned here apply to **every** web project:

1. **Types first** — Define your data shapes before writing code
2. **Test with curl** — Verify the API works before building UI
3. **Handle errors everywhere** — Every API call can fail
4. **Show feedback** — Loading states, error messages, success toasts
5. **Layer your code** — Separation of concerns makes code maintainable

In Part 5, we'll take our API client and build actual UI pages — the search page, the export page, and the recommendation display. We'll see how all these types and functions come together in real Svelte components.

The API layer is the bridge between your frontend and backend. Now that we've built a solid, type-safe, well-tested bridge, we can build anything on top of it. Let's keep going! 🚀

---

## Quick Reference: Complete API Function Signatures

For easy reference, here are all the API functions we built:

```typescript
// Export — GET /api/v0/epub
fetchExport(url: string): Promise<ExportResponse>

// Meta — GET /api/v0/meta
fetchMeta(url: string): Promise<MetaResponse>

// Recommendations — GET /api/v0/recommendations
fetchRecommendations(params: {
  q?: string
  url_id?: string
  n?: number
  site_domain?: string
}): Promise<RecommendationsResponse>

// Votes — GET /api/v0/recommendations/votes
fetchVotes(url_id: string): Promise<VotesResponse>

// Suggest — POST /api/v0/recommendations/suggest
submitSuggestion(data: {
  url_id: string
  suggested_url: string
  comment?: string
}): Promise<SuggestResponse>

// Vote — POST /api/v0/recommendations/vote
castVote(suggestion_id: number, vote: 1 | -1): Promise<VoteResponse>

// Search — GET /api/v0/search
search(filters: SearchFilters): Promise<SearchResponse>
```

## Quick Reference: Error Codes

| HTTP Status | FicHub err | Meaning | Retryable? |
|-------------|------------|---------|------------|
| 200 | 0 | Success | N/A |
| 400 | -1 | Bad request / missing data | No |
| 400 | -5 | Not found / unsupported URL | No |
| 400 | -7 | Content blacklisted | No |
| 400 | -10 | Automated request blocked | No |
| 429 | -429 | Rate limited | Yes (wait) |
| 500 | -1 | Internal server error | Maybe |
| 502 | -6 | Upstream scrape error | Yes (retry later) |

## Quick Reference: Tag Type IDs

| ID | Type | Example |
|----|------|---------|
| 1 | Fandom | Harry Potter, Marvel, Star Wars |
| 2 | Character | Hermione Granger, Tony Stark |
| 3 | Relationship | Hermione/Harry, Tony/Steve |
| 4 | Freeform | Time Travel, Angst, Humor |
| 5 | Warning | Graphic Depictions Of Violence |
| 6 | Category | M/M, F/M, Gen |

---

*End of Part 4: Building the API Layer*
