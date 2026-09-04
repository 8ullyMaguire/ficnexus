# Part 1: Welcome to the Project

---

# Chapter 1: What We're Building

## Meet FicHub

Imagine this: you're reading a fanfiction story online — maybe it's an epic *Harry Potter* reimagining, or a *Star Wars* character study that made you cry — and you think, "I want to read this on my phone, on the train, without needing Wi-Fi." So you open a tab, paste the URL, and within seconds you have an ebook file you can send to your Kindle, open on your phone, or save to your computer forever.

That's FicHub.

FicHub is a self-hosted web application that lets you download fanfiction from six major sites and save them as beautifully formatted ebooks. Paste a URL, click a button, get an EPUB. Simple on the surface, powerful underneath.

Why would you want this? Because fanfiction deserves better than being trapped in a browser tab. You might be reading on a phone with a spotty data connection. You might want to read offline on a flight. You might want to keep a personal archive of your favorite stories — because fanfiction sites have been known to go down, take stories offline, or change their formats without warning.

FicHub gives you a copy of the story that's yours forever. The EPUB file works on Kindles, Kobo e-readers, Apple Books, Google Play Books, and dozens of other reading apps. It's formatted with proper chapters, metadata, and (when available) cover art.

But FicHub isn't just about downloading. It's also a discovery engine. It can recommend stories you'll love based on what other fans are reading. It lets people suggest stories to the community, vote on those suggestions, and build a curated library of the best fanfiction out there.

And the coolest part? You can run the whole thing on your own computer. No one else's server. No tracking. No ads. Just you, your stories, and a tiny piece of the internet that belongs to you.

## The Two Halves

FicHub has two main parts, and we're going to build both of them from scratch.

**The Backend (Rust + Axum):** This is the engine room. It handles all the heavy lifting — fetching stories from fanfiction sites, converting them into ebook files, managing the database, running the recommendation engine, and serving up data to anyone who asks. We'll write this in Rust, a language that's fast, safe, and genuinely fun to use once you get the hang of it. We'll use a web framework called Axum, which is like a well-organized kitchen where every ingredient has its place.

**The Frontend (SvelteKit):** This is the face of FicHub — the part you see and interact with. It's a sleek, dark-themed interface with tabs, search bars, and beautiful cards showing story recommendations. We'll build this with SvelteKit, a framework that makes creating web interfaces feel less like programming and more like drawing a picture. SvelteKit handles the routing, the page loading, and all the plumbing so we can focus on making things look and feel great.

Think of it like a restaurant. The backend is the kitchen — all the chopping, cooking, and assembling happens back there. The frontend is the dining room — the menu, the table settings, the atmosphere. Both parts work together to create the full experience.

## The Six Supported Sites

FicHub can fetch stories from six different fanfiction websites:

1. **Archive of Our Own (AO3)** — The biggest and most popular fanfiction archive. Clean HTML, well-organized metadata, and the source of a huge portion of all fanfiction ever written. This is our primary target.

2. **FanFiction.net (FFN)** — The original fanfiction giant. Been around since the early 2000s. The HTML is messier, the structure is less consistent, but it has a massive library of stories, especially older ones.

3. **FictionPress** — Sister site to FanFiction.net, but for original fiction rather than fanfiction. Same engine, same quirks.

4. **SpaceBattles** — A forum-style site where stories are written as thread posts. The format is completely different from a traditional archive — stories live inside forum threads, and each chapter is one or more posts.

5. **SufficientVelocity** — Very similar to SpaceBattles, same forum engine, but a separate community with its own stories and culture.

6. **AdultFanFiction** — A site for mature-rated fanfiction. The markup is different from AO3 and FFN, so it needs its own scraper.

Each site has its own HTML structure, its own way of organizing chapters, its own metadata format. That means we need a separate scraper for each one — a bit of code that knows exactly how to pull the story content out of that particular site's pages. We'll build a system where you can plug in a new scraper easily, so if a new fanfiction site appears in the future, adding support for it is straightforward.

## The Three Tabs

When you open FicHub in your browser, you'll see three main tabs at the top:

### Download Tab

This is the main event. You see a text input where you paste a fanfiction URL. Below it, a "Download" button. Click it, and FicHub does its thing — fetches the story, grabs all the chapters, collects the metadata (title, author, tags, summary), and bundles it all into an EPUB file. A moment later, the file downloads to your computer.

#### What's an EPUB?

EPUB (Electronic Publication) is an open standard for ebooks. Think of it as a specially organized zip file containing HTML files (one per chapter), a table of contents, metadata (title, author, description), and optionally a cover image. Almost every e-reader and reading app supports EPUB. It's the most portable ebook format available — unlike PDFs, which are fixed-layout, EPUBs reflow to fit any screen size.

When FicHub generates an EPUB, it creates clean HTML for each chapter, formats the table of contents with clickable links, embeds the story's metadata, and packages everything into a standards-compliant EPUB 3 file. The result is an ebook that reads beautifully on any device.

There's also a search bar where you can look up stories you've already downloaded or browse the recommendation catalog.

### Recommendations Tab

This is where FicHub gets smart. Based on what stories people have bookmarked across the community, the recommendation engine suggests stories you might enjoy. It uses a technique called collaborative filtering — basically, "people who liked these stories also liked these other stories."

You see a grid of story cards, each with a cover image (if available), the title, author, a short summary, and tags. Click a card to see more details, or click a button to download it directly.

### Suggestions Tab

This is the community feature. Anyone using FicHub can suggest stories they think are worth reading. Other users can upvote or downvote those suggestions. The most-voted stories rise to the top, creating a crowd-sourced "best of fanfiction" list.

You can browse suggestions, filter by tags, see vote counts, and add your own suggestions to the pile.

## The Search Feature

FicHub's search isn't just a simple text search. It supports a query syntax inspired by AO3's own search:

- `harry potter` — searches for stories with "harry" and "potter" in the tags
- `fandom:marvel` — filters by fandom
- `rating:explicit` — filters by content rating
- `words:>50000` — finds stories longer than 50,000 words
- `kudos:>1000` — finds stories with more than 1,000 kudos
- `-crossover` — excludes crossover stories
- `characters:hermione granger` — filters by character tags

You can combine these freely: `fandom:marvel rating:gen words:>10000 -crossover` gives you Marvel stories rated General that are at least 10,000 words long and aren't crossovers.

We'll build a parser for this syntax on the frontend, converting your search query into API calls that the Rust backend understands.

## The Recommendation Engine

Behind the Recommendations tab sits a recommendation engine that uses collaborative filtering. This is the same basic technology that powers "Recommended for you" on Netflix, "Customers who bought this also bought" on Amazon, and "People you may know" on LinkedIn. The core idea is simple: if two people have similar tastes, and one person likes something the other hasn't seen, recommend it.

Here's how it works in plain English:

1. **The Collection Worker** runs in the background and periodically scrapes the public bookmark lists (favorites) of FicHub users. When someone connects their AO3 account, we look at what stories they've bookmarked.

2. We build a big matrix: which users have bookmarked which stories. From this, we can compute similarity scores between stories. If lots of people who bookmarked Story A also bookmarked Story B, those two stories are probably similar.

3. When you open the Recommendations tab, we look at what you've bookmarked, find similar stories you haven't seen yet, and show them to you.

This is collaborative filtering — the same basic idea that powers "customers who bought this also bought..." on Amazon, except we're doing it with fanfiction bookmarks.

We'll use PostgreSQL to store the bookmark data and Redis to cache the similarity scores so recommendations are fast.

## Community Features

FicHub isn't just a solo tool — it's a community platform:

- **Suggestions:** Users can suggest stories they love. Each suggestion includes the URL, a short review, and tags.
- **Voting:** Other users can upvote or downvote suggestions. This creates a natural ranking system.
- **Comments:** A simple comment thread on each suggestion lets people discuss why a story is worth reading.

These features turn FicHub from a downloader into a fanfiction discovery platform.

## What the Finished App Looks Like

Let me paint you a picture of the final product:

The page loads with a dark background — think deep charcoal gray, not harsh black. At the top, a header with the FicHub logo and the three tabs: **Download**, **Recommendations**, **Suggestions**.

On the Download tab, there's a wide text input field with placeholder text that says "Paste a fanfiction URL here..." Below it, a big blue button that says "Download." When you click it, a loading spinner appears while FicHub fetches the story. After a few seconds, your EPUB file starts downloading.

The Recommendations tab shows a grid of story cards. Each card has a gradient background in a color derived from the story's tags, the title in bold, the author name below it, a two-line summary, and a row of tag pills. Hover over a card and it lifts slightly with a subtle shadow effect.

The Suggestions tab shows a list of suggested stories with vote counts on the left (up arrow, number, down arrow), the story title and description on the right, and tags below. A "Suggest a Story" button lets you add your own.

The whole interface is responsive — it works on desktop, tablet, and phone. The navigation tabs collapse into a hamburger menu on small screens.

### The Data Flow Behind the Scenes

Every interaction in FicHub follows a predictable data flow. When you click Download, here's what happens behind the scenes in about 200 milliseconds:

1. The frontend validates the URL format — is it from a supported site?
2. A POST request is sent to `/api/download` with the URL in the body
3. The backend parses the URL to determine the source site
4. The appropriate scraper is selected and invoked
5. The scraper fetches the HTML from the fanfiction site
6. HTML is parsed and the story content, chapters, and metadata are extracted
7. The auto-merge system checks if this work already exists (by title + author)
8. The EPUB generator creates an ebook file from the extracted data
9. The file is saved to a disk cache (so the next download is instant)
10. The metadata is stored in PostgreSQL — linked to the canonical work record
11. The EPUB file is sent back to the browser for download

This flow is deterministic and auditable. If something goes wrong, logs tell you exactly which step failed. That's the kind of reliability you get when you build things methodically.

## How the Pieces Connect

Let's trace the journey of a single request, from your browser to the database and back:

1. **You paste a URL and click Download.** Your browser sends an HTTP request to the SvelteKit server running on port 5173.

2. **SvelteKit processes the request.** The SvelteKit frontend has a page component with a form. When you submit it, JavaScript sends a fetch request to the backend API at `http://localhost:3000/api/download`.

3. **Axum receives the request.** The Rust backend is running on port 3000. The Axum router sees the URL path `/api/download` and matches it to a handler function.

4. **The handler processes the request.** It extracts the URL from the request body, figures out which scraper to use based on the domain (ao3.com → AO3 scraper, fanfiction.net → FFN scraper, etc.), and calls the appropriate scraper.

5. **The scraper fetches the story.** It sends an HTTP request to the fanfiction site, parses the HTML response, and extracts the story content, chapter structure, and metadata.

6. **FicHub checks for an existing work.** The auto-merge system looks up the title and author in the `works` table. If a match is found, the new source URL is added to `fic_info` with a link to the existing work. If not, a new work record is created.

7. **The EPUB generator builds the file.** It takes the scraped data and creates an EPUB file — a zip archive with HTML files for each chapter, a table of contents, metadata, and cover image.

8. **The response comes back.** The handler sends the EPUB file back through Axum → your browser → your Downloads folder.

9. **Meanwhile, PostgreSQL stores data.** The work metadata, source URLs, user bookmarks, and suggestion data all live in PostgreSQL. When you download a story, FicHub links the source to the appropriate work record in the database.

10. **Redis handles caching.** Frequently accessed data — like recent downloads, recommendation scores, and rate limits — gets cached in Redis for lightning-fast access.

That's the full loop. Browser → SvelteKit → API → Rust → Scraper → Auto-Merge → Fanfiction Site → EPUB Generator → PostgreSQL + Redis → Response → Your browser → Download.

It sounds like a lot, but each piece has a single clear job, and we'll build each piece one at a time. By the end of this book, you'll understand how every single line of code fits into this flow.

## What You'll Learn

By the time you finish this book, you'll know how to:

- Write a Rust web server from scratch with Axum
- Connect to PostgreSQL and run database queries safely
- Build web scrapers that handle messy, real-world HTML
- Generate EPUB files programmatically
- Implement collaborative filtering for recommendations
- Build a beautiful, responsive frontend with SvelteKit
- Deploy your application with Docker
- Cross-compile Rust binaries for different platforms

More importantly, you'll understand *why* each piece exists and how to make architectural decisions for your own projects.

Let's get started.

---

# Chapter 2: How the Web Works

## The Internet as a Postal System

Before we write a single line of code, let's make sure we all understand how the web actually works. Don't worry — we're not going to dive into TCP/IP packets or DNS resolution diagrams. Instead, think of the internet as the world's most efficient postal system.

When you visit a website, here's what's really happening:

- **You (the browser)** write a letter (HTTP request) saying "Please send me the page at www.example.com."
- **The letter travels** across the internet to the website's server.
- **The server reads your letter**, figures out what you want, and writes back (HTTP response) with the page content.
- **Your browser receives the letter** and reads it, rendering the page on your screen.

That's it. Every single thing that happens on the web — every click, every form submission, every image load — is just your browser writing letters to servers and servers writing letters back.

The "letters" are called **HTTP requests** (when you send them) and **HTTP responses** (when you receive them). The address on the envelope is a **URL** — something like `https://example.com/about`.

## HTML: The Building Frame

When a server sends back a web page, the content is mostly written in **HTML** — HyperText Markup Language. HTML is the structure of a web page, the skeleton that everything else hangs on.

Think of HTML like the frame of a house. It defines where the rooms are, where the doors go, where the windows sit. It doesn't say anything about what color the walls are or what furniture goes inside — that's someone else's job.

Here's what a simple HTML page looks like:

```html
<!DOCTYPE html>
<html>
<head>
    <title>My First Page</title>
</head>
<body>
    <h1>Hello, World!</h1>
    <p>This is a paragraph of text.</p>
    <a href="https://example.com">Click here</a>
</body>
</html>
```

Let's break that down:

- `<!DOCTYPE html>` — tells the browser "this is an HTML5 document"
- `<html>` — the root element, everything goes inside here
- `<head>` — invisible stuff: the page title, linked stylesheets, scripts
- `<body>` — visible stuff: headings, paragraphs, images, links
- `<h1>` — a top-level heading (there's `<h2>`, `<h3>`, etc. for sub-headings)
- `<p>` — a paragraph
- `<a>` — a link (the `href` attribute is the address it points to)

HTML uses **tags** — words inside angle brackets. Most tags come in pairs: an opening tag like `<p>` and a closing tag like `</p>`. The content goes between them.

💡 **Key Concept:** HTML is just text. A web page is a text file that the browser knows how to read and display. That's all the web is — text files being sent back and forth.

## CSS: The Paint on the Walls

If HTML is the frame of the house, **CSS** (Cascading Style Sheets) is the paint, the wallpaper, the furniture arrangement — everything that makes the house look good.

CSS uses a selector system to target specific elements. The most basic selector is the element name itself:

```css
p { color: red; }           /* All paragraphs become red */
#title { font-size: 2rem; } /* The element with id="title" */
.card { border: 1px solid; } /* All elements with class="card" */
```

Selectors are how you tell CSS "apply these styles to THIS element." It's like addressing an envelope — you need to be specific about who it's for.

Modern CSS also has powerful layout tools:

```css
.container {
    display: flex;            /* Lay children out in a row */
    justify-content: center;  /* Center horizontally */
    gap: 1rem;                /* Space between children */
}

.grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);  /* 3 equal columns */
    gap: 1.5rem;
}
```

`flexbox` and `grid` are the two layout systems you'll use constantly. Flexbox is great for one-dimensional layouts (a row or a column). Grid is great for two-dimensional layouts (rows AND columns). FicHub's interface uses both — flexbox for the tab navigation, grid for the story cards.

CSS tells the browser things like "make this heading red" or "put 20 pixels of space around this paragraph" or "make this box have rounded corners and a shadow."

Here's CSS in action:

```css
body {
    background-color: #1a1a2e;
    color: #e0e0e0;
    font-family: Arial, sans-serif;
}

h1 {
    color: #4fc3f7;
    font-size: 2rem;
}

p {
    margin: 1rem 0;
    line-height: 1.6;
}
```

This CSS turns our plain page into a dark-themed page with a blue heading and comfortable paragraph spacing. Notice how CSS uses the same names as HTML tags — it's saying "for every `<body>` element, apply these styles" and "for every `<h1>` element, apply these styles."

The `#1a1a2e` and `#4fc3f7` are **hex color codes** — they represent colors as combinations of red, green, and blue. `#1a1a2e` is a deep dark blue-purple (that's the background color of FicHub!), and `#4fc3f7` is a bright sky blue.

## JavaScript: The Light Switches

HTML is structure. CSS is style. **JavaScript** is behavior — the thing that makes a web page *do stuff*.

When you click a button and a menu appears, that's JavaScript. When you type in a search box and results appear below it, that's JavaScript. When you submit a form and the page updates without reloading, that's JavaScript.

Here's a tiny example:

```html
<button onclick="sayHello()">Click me!</button>

<script>
function sayHello() {
    alert('Hello from JavaScript!');
}
</script>
```

When you click the button, the `sayHello()` function runs, and a pop-up dialog appears saying "Hello from JavaScript!" The `<script>` tag is where JavaScript code lives.

In FicHub, JavaScript is doing most of the heavy lifting on the frontend. When you paste a URL and click Download, JavaScript is what sends the request to the backend, shows a loading spinner, and handles the response. You won't write raw JavaScript much though — SvelteKit handles the messy parts and lets us write in a much friendlier way.

## Web Servers: The Restaurant Kitchen

Now let's talk about the other side of the equation — the server.

A **web server** is a program that runs on a computer, listens for incoming requests, and sends back responses. When you type `example.com` into your browser, your request eventually arrives at a server somewhere in the world, and that server sends back the HTML for that page.

Think of a web server like a restaurant kitchen. You (the customer) don't go into the kitchen and cook your own food. You sit at a table, look at the menu, and tell the waiter what you want. The waiter takes your order to the kitchen, the chef prepares it, and the waiter brings it back to you.

In this analogy:

- **You** = the browser (sending requests)
- **The menu** = the API (the list of things you can ask for)
- **The waiter** = HTTP (the protocol that carries requests and responses)
- **The kitchen** = the web server (where the work happens)
- **The chef** = your application code (Axum, in our case)

FicHub's web server is written in Rust using Axum. It sits on your computer, listens on port 3000, and handles requests like "give me this story's metadata" or "download this story as an EPUB."

## APIs: Ordering from a Menu

An **API** (Application Programming Interface) is the set of things a server lets you ask for and the rules for asking.

When FicHub's frontend needs data from the backend, it doesn't just shout into the void and hope for the best. It sends a carefully structured request to a specific URL, with specific data, and expects a specific format back.

Here are some of FicHub's API endpoints:

| Endpoint | Method | What it does |
|----------|--------|-------------|
| `/api/download` | POST | Download a story as EPUB |
| `/api/search` | GET | Search the story catalog |
| `/api/recommendations` | GET | Get recommended stories |
| `/api/suggestions` | GET | Get community suggestions |
| `/api/suggestions` | POST | Submit a new suggestion |

Each endpoint is like an item on a restaurant menu. The frontend knows what's available, what to ask for, and what format the answer will come back in.

The **method** (GET, POST, etc.) tells the server what kind of action you want:
- **GET** = "Give me some data" (reading)
- **POST** = "Here's some new data, do something with it" (creating)

## The Request-Response Cycle

Let's trace the exact journey of a request through FicHub, step by step:

1. **You paste a URL into the search box and click Download.**

2. **SvelteKit's JavaScript intercepts the form submission.** Instead of the page reloading (like a traditional website), SvelteKit's JavaScript catches the submit event and takes over.

3. **JavaScript sends a POST request:**
   ```javascript
   fetch('http://localhost:3000/api/download', {
       method: 'POST',
       headers: { 'Content-Type': 'application/json' },
       body: JSON.stringify({ url: 'https://archiveofourown.org/works/12345' })
   })
   ```

4. **The request travels to the Axum server on port 3000.** Axum's router sees the path `/api/download` and the method POST, and routes the request to the download handler function.

5. **The handler processes the request.** It looks at the URL, determines it's an AO3 story, and calls the AO3 scraper. The scraper fetches the story HTML, parses it, extracts the content, and returns structured data.

6. **The handler builds a response.** It generates an EPUB file from the scraped data and sends it back as the response.

7. **The response travels back to your browser.** JavaScript receives it and triggers a file download.

The whole thing takes maybe two to three seconds for a typical story.

## HTTP Status Codes: Did It Work?

When a server sends back a response, it includes a **status code** — a number that tells you whether the request succeeded and what happened.

Here are the ones you'll see most often:

**200 OK** — "Here's what you asked for." Everything worked perfectly.

**301 Moved Permanently** — "The thing you're looking for moved to a new address." The browser automatically follows the redirect.

**404 Not Found** — "There's nothing at that address." You typed the wrong URL, or the page was deleted.

**500 Internal Server Error** — "Something went wrong on my end." The server crashed or hit an unexpected error. This is the server's way of saying "oops."

**429 Too Many Requests** — "Slow down, you're asking for too much too fast." This is a rate limit. We'll use this ourselves to prevent users from hammering our API.

**401 Unauthorized** — "Who are you?" You need to log in first.

**403 Forbidden** — "I know who you are, but you're not allowed here."

In FicHub, you'll see these in action all the time. If you paste a URL that doesn't point to a supported site, you'll get a 400 Bad Request with a message like "This URL isn't from a supported site." If the story was deleted, you'll get a 404. If something truly unexpected happens, you'll get a 500.

⚠️ **Watch Out!** HTTP 500 errors are the trickiest to debug because they're so generic. "Something went wrong" tells you almost nothing. In FicHub, we'll add detailed logging to our server so that when a 500 happens, we can look at the server logs and see exactly what broke.

## JSON: The Language Computers Speak

When your browser and server exchange data, they need a format they both understand. The most common format for web APIs is **JSON** — JavaScript Object Notation.

JSON looks a lot like a JavaScript object (or a Rust struct, or a Python dictionary). It's just text with a specific structure:

```json
{
    "title": "The Luminary",
    "author": "starweaver42",
    "fandom": "Harry Potter",
    "word_count": 85000,
    "rating": "Teen And Up",
    "tags": ["Adventure", "Slow Burn", "Time Travel"],
    "complete": true
}
```

See how it works? Each piece of data has a **key** (like `"title"`) and a **value** (like `"The Luminary"`). Values can be strings (text), numbers, booleans (`true`/`false`), arrays (lists in square brackets), or even nested objects.

JSON is the language of web APIs. When the frontend sends data to the backend, it sends JSON. When the backend responds, it sends JSON. Even when the data is complex — nested objects, arrays of arrays — JSON handles it gracefully.

💡 **Key Concept:** JSON is just structured text. It's human-readable, which makes it great for debugging. You can open your browser's developer tools (F12), go to the Network tab, and see the exact JSON being sent back and forth.

Here's what a FicHub recommendation response might look like:

```json
{
    "recommendations": [
        {
            "id": 1,
            "title": "The Black Queen",
            "author": "Silently Watches",
            "url": "https://fanfiction.net/s/12345678/1",
            "summary": "Harry finds an ancient artifact that changes everything...",
            "tags": ["Harry Potter", "Dark", "Independent Harry"],
            "score": 92
        },
        {
            "id": 2,
            "title": "Fate Is A Very Silly Thing",
            "author": "gnosticfury",
            "url": "https://archiveofourown.org/works/87654321",
            "summary": "What if the prophecy went differently?",
            "tags": ["Harry Potter", "Humor", "Crack Treated Seriously"],
            "score": 87
        }
    ],
    "total": 2,
    "page": 1
}
```

The backend sends this JSON, and the frontend reads it and renders each recommendation as a card on the page.

## Frameworks: Pre-Built Furniture

Could you build a web server from scratch without any frameworks? Absolutely. You'd write code to open a network socket, listen for incoming connections, parse HTTP requests byte by byte, and send back raw HTTP responses with HTML text. People did this in the early days of the web, and it worked — but it was slow, error-prone, and incredibly tedious.

The difference between frameworks and raw code is like the difference between building a house from raw lumber versus assembling prefabricated walls. Both result in a house, but one takes months and requires expertise in every trade, while the other takes days and lets you focus on what makes your house unique.

**Frameworks** are pre-built collections of code that handle the boring, repetitive parts of web development so you can focus on the interesting parts — your application's unique logic. They give you structure, conventions, and battle-tested solutions to common problems.

Think of it like furniture. You *could* cut down trees, saw lumber, carve joints, and build a chair from raw wood. Or you could buy a chair and customize it with paint and cushions. Frameworks are the chair — solid, tested, and ready to use.

Here are the frameworks we'll use:

- **Axum** (Rust) — A web framework for building HTTP servers. It handles routing (matching URLs to functions), request parsing (extracting data from incoming requests), middleware (layers that process requests before they reach your code), and response formatting. It's like a well-organized kitchen with everything in its place.

- **SvelteKit** (JavaScript/TypeScript) — A framework for building web applications. It handles page routing (URLs to pages), server-side rendering, form handling, and asset management. It's like having a skilled interior designer who handles the layout so you can focus on choosing the colors.

- **SQLx** (Rust) — A library for connecting to databases and running queries. It handles connection pooling, query formatting, and type safety (it checks your SQL queries at compile time!).

- **Serde** (Rust) — A library for converting data between formats (like converting a Rust struct to JSON, or parsing JSON into a Rust struct). It's the universal translator that lets your code speak JSON fluently.

## Databases: Filing Cabinets for Data

A **database** is where your application stores data that needs to persist — survive between restarts, be queryable, and be consistent.

Think of a database like a filing cabinet. Each drawer is a **table** (like "users" or "works" or "suggestions"). Inside each drawer, each folder is a **row** — one record (one user, one work, one suggestion). Each folder has labeled sections — those are the **columns** (name, email, word count, etc.).

Here's what FicHub's main tables look like:

**users**
| id | username | ao3_user | created_at |
|----|----------|----------|------------|
| 1 | readingrainbow | ao3_user_42 | 2026-01-15 |
| 2 | fictionfiend | ao3_user_87 | 2026-02-03 |

**works** (canonical stories — one row per unique work)
| id | canonical_title | canonical_author | description | default_source_id |
|----|-----------------|------------------|-------------|-------------------|
| 1 | The Luminary | starweaver42 | A story about light... | abc123 |
| 2 | Second Chances | phoenix_writer | A second chance romance... | def456 |

**fic_info** (sources — one row per URL from a fanfiction site)
| id (url_id) | work_id | title | author | source | words |
|-------------|---------|-------|--------|--------|-------|
| abc123 | 1 | The Luminary | starweaver42 | ao3/12345 | 85000 |
| def456 | 1 | The Luminary | starweaver42 | ffn/67890 | 84500 |
| ghi789 | 2 | Second Chances | phoenix_writer | ao3/98765 | 45000 |

**bookmarks**
| user_id | work_id | created_at |
|---------|---------|------------|
| 1 | 1 | 2026-03-01 |
| 2 | 1 | 2026-03-05 |

**work_proposals** (curator merge/split proposals)
| id | proposer_id | work_id_a | work_id_b | proposal_type | status | created_at |
|----|-------------|-----------|-----------|---------------|--------|------------|
| 1 | 1 | 1 | 3 | merge | pending | 2026-04-10 |

When you download a story, FicHub scrapes the source and checks if the work already exists in the `works` table. If it does, the new source URL is added to `fic_info` with a link to the existing work. If not, a new work is created. When someone bookmarks a work, a row goes into the `bookmarks` table using `work_id`. The recommendation engine queries the `bookmarks` table to find patterns.

## PostgreSQL: Our Filing Cabinet of Choice

There are dozens of databases you could use — MySQL, SQLite, MongoDB, Redis, and many more. For FicHub, we chose **PostgreSQL** (often called "Postgres"). Let's quickly compare the main options so you understand why:

- **SQLite** — A file-based database, great for simple apps and mobile apps. It doesn't require a server process. But it doesn't handle concurrent writes well, and it lacks advanced features like arrays and full-text search. Fine for a todo app, not ideal for FicHub.

- **MySQL** — The most popular database in the world (by number of installations). It's fast, reliable, and well-documented. But it lacks some of PostgreSQL's advanced features, and its default settings prioritize speed over data safety.

- **MongoDB** — A "NoSQL" document database that stores JSON-like documents instead of tables and rows. It's flexible and easy to get started with, but it can lose data if not configured carefully, and complex queries (like our recommendation engine) are harder to express.

- **PostgreSQL** — The "most advanced" open-source database. It supports SQL and advanced features like arrays, full-text search, JSON columns, and custom types. It prioritizes data safety (it won't silently lose your data) and extensibility.

Why PostgreSQL?

**It's battle-tested.** PostgreSQL has been in development since 1996 and powers some of the biggest websites in the world (Instagram, Spotify, Reddit). It's incredibly reliable.

**It handles complex queries.** We need to do things like "find works that users with similar tastes have bookmarked." PostgreSQL's query language (SQL) makes these kinds of queries straightforward.

**It has rich data types.** PostgreSQL supports arrays, JSON, full-text search, and more. We'll use arrays for tags and full-text search for the search feature.

**It's free and open source.** No licensing fees, no vendor lock-in. You own your data.

Here's what a SQL query looks like:

```sql
-- Find all works with the "Harry Potter" tag, sorted by word count
SELECT w.canonical_title, w.canonical_author, fi.words
FROM works w
JOIN fic_info fi ON fi.work_id = w.id
WHERE 'Harry Potter' = ANY(fi.tags)
ORDER BY fi.words DESC
```

Don't worry if that doesn't make sense yet — we'll learn SQL step by step in later chapters. For now, just know that this query joins the `works` and `fic_info` tables to find anything tagged "Harry Potter" and returns the results sorted from longest to shortest.

We'll also use **Redis** (or Valkey, the open-source Redis alternative) as a fast, temporary storage for caching. Think of Redis like a whiteboard — quick to write on, quick to read from, but erased when you power off. We use it to cache recommendation scores, rate limiting counters, and frequently accessed data.

## Putting It All Together

Let's zoom out and see how all these pieces fit:

- **HTML** gives structure to our pages
- **CSS** makes them look beautiful
- **JavaScript** makes them interactive
- **SvelteKit** organizes the frontend code into pages and components
- **Axum** organizes the backend code into routes and handlers
- **JSON** is the language between frontend and backend
- **PostgreSQL** stores all our data — works, sources, bookmarks, proposals, and user accounts
- **Redis** caches frequently accessed data for speed

The works model deserves special attention here. A single story might appear on AO3, FFN, and a forum simultaneously. Instead of treating each URL as a separate entry, FicHub groups them under one **work** record. The `works` table holds the canonical title and author. Each source URL lives in `fic_info` with a foreign key back to the work. When you bookmark a work, you bookmark the work itself — not any particular URL. This means your bookmarks stay valid even if one source goes offline.

FicHub also has an auto-merge system. When a new story is scraped, the system checks if it matches an existing work by title and author (case-insensitive). If the word counts are close (within 5%), it links the new source to the existing work automatically. If there's some doubt, it creates a proposal for curators to vote on.

When you use FicHub, you're seeing the culmination of all these technologies working together, silently and seamlessly, to deliver a great experience.

### What "Self-Hosted" Really Means

When we say FicHub is "self-hosted," we mean you run it on your own computer or server. You're not signing up for someone else's service. You're not trusting a company with your data. You own every piece of it.

Self-hosting means:
- **Your data stays with you.** Your download history, bookmarks, and reading preferences never leave your machine.
- **No subscriptions.** No monthly fees, no premium tiers, no ads.
- **No shutdown risk.** If the company behind your favorite service goes bankrupt, the service disappears. If YOU are the service, it's yours as long as you want it.
- **Full control.** You decide what features to add, what sites to support, and how the interface looks.

Of course, self-hosting requires some technical setup — which is exactly why this book exists. We'll walk you through every step.

### Why Rust?

You might wonder: why build the backend in Rust instead of Python, JavaScript, or Go?

- **Speed.** Rust compiles to native machine code. It's as fast as C and C++ but with a much nicer developer experience. For a web server that handles concurrent requests and parses lots of HTML, this matters.
- **Safety.** Rust's ownership system prevents memory bugs at compile time. No null pointer dereferences, no buffer overflows, no data races. The compiler catches these before you even run the code.
- **Concurrency.** Rust's async/await system with Tokio makes it easy to handle thousands of simultaneous connections efficiently.
- **Developer experience.** Rust's error messages are famously helpful. The compiler teaches you as you go. Cargo makes managing dependencies, testing, and building a breeze.

Rust is a great choice for a project like FicHub: a system that needs to be fast (fetching and parsing web pages), reliable (handling user requests without crashing), and maintainable (code you'll want to read and modify years from now).

In the next chapter, we're going to set up all these tools on your computer. Let's get building.

---

# Chapter 3: Setting Up Your Workshop

## Welcome to Your Workshop

Every craftsperson needs a workshop. A woodworker has their workbench, their saws, their clamps. A painter has their easels, their brushes, their canvases.

Today, we're setting up your programming workshop. By the end of this chapter, you'll have all the tools installed on your computer that you need to build FicHub. We'll install the programming languages, the databases, the editors, and set up the project directory.

Don't be intimidated by the number of tools. Each one has a specific job, and we'll use them all together seamlessly. Think of it like setting up a kitchen — you need a stove, a fridge, a sink, some pots and pans. None of them are complicated on their own, and together they let you cook anything.

## Installing Rust

First up: **Rust**, the programming language we'll use for the backend.

Rust is a modern programming language focused on speed, safety, and reliability. It's the language behind some of the fastest software in the world, and it's been voted the "most loved" programming language for several years running. Once you get used to it, you'll understand why.

### On Linux (Arch, Manjaro, CachyOS)

If you're on an Arch-based distro (which it looks like you might be!), Rust is in the official repositories:

```bash
sudo pacman -S rust
```

But we also want `rustup`, the official Rust installer and version manager, because it lets us easily update Rust and manage multiple versions:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

When it asks "How would you like to proceed?", just press Enter to choose the default (install).

### On Linux (Debian, Ubuntu)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Same installer, different distro. Rust's install script works everywhere.

### On macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Yes, the same command again! Rust's install experience is beautifully consistent.

### On Windows

Download and run the installer from [rustup.rs](https://rustup.rs). It will install everything you need.

### Verifying the Installation

After installing, close your terminal and open a new one (so the PATH updates), then check:

```bash
rustc --version
```

You should see something like:

```
rustc 1.78.0 (9b00956e5 2024-04-29)
```

And:

```bash
cargo --version
```

Should show:

```
cargo 1.78.0 (54d8815d0 2024-03-26)
```

If you see version numbers, you're golden! 🎉

⚠️ **Watch Out!** If `rustc` or `cargo` gives you a "command not found" error, you need to source the environment file. Run:

```bash
source "$HOME/.cargo/env"
```

Or just close your terminal and open a new one.

## Cargo: Your Build Buddy

**Cargo** is Rust's package manager and build tool. It does everything — creating new projects, building code, running tests, managing dependencies, and more. If Rust is the engine, Cargo is the steering wheel. If you've used Python, think of Cargo as pip + venv + build tool all rolled into one, but much better.

Cargo manages your project's **dependencies** — external libraries other people wrote that you can use. When you add a dependency to `Cargo.toml`, Cargo downloads it from [crates.io](https://crates.io) (Rust's package registry), compiles it, and makes it available in your code. It also handles versioning — if two libraries need different versions of the same dependency, Cargo figures it out.

Let's try it out:

```bash
cargo new hello_fichub
cd hello_fichub
```

This creates a new directory called `hello_fichub` with a basic Rust project inside. Let's look at what it created:

```bash
ls
```

You'll see:

```
Cargo.toml
src/
```

- **Cargo.toml** — The project's configuration file. It tells Cargo the project name, version, and what external packages (called "crates") the project depends on.
- **src/** — The source code directory. Inside it, there's a `main.rs` file.

Let's look at `Cargo.toml`:

```toml
[package]
name = "hello_fichub"
version = "0.1.0"
edition = "2021"

[dependencies]
```

The `[package]` section is project metadata. The `[dependencies]` section is empty right now — we'll add libraries here later.

Now let's build and run it:

```bash
cargo run
```

You'll see:

```
   Compiling hello_fichub v0.1.0
    Finished dev [unoptimized + debuginfo] target(s)
     Running `target/debug/hello_fichub`
Hello, world!
```

That's it! Cargo compiled the code, created an executable, and ran it. The default project prints "Hello, world!" — and that's exactly what we see.

### What Just Happened?

When you ran `cargo run`, here's the chain of events:

1. Cargo looked at `src/main.rs` and found the `main` function
2. It compiled the Rust source code into machine code (the `rustc` compiler)
3. It linked the machine code into an executable file
4. It ran the executable

The compiled output goes into the `target/` directory. You can ignore that — Cargo manages it automatically.

### Other Cargo Commands

Here are the Cargo commands you'll use most often:

```bash
cargo build          # Build the project (no running)
cargo build --release # Build optimized for speed (for deployment)
cargo run            # Build and run
cargo test           # Run the tests
cargo check          # Quick check: does the code compile? (no binary)
cargo fmt            # Format the code nicely
cargo clippy         # Lint the code for common mistakes
cargo add <crate>    # Add a dependency
```

🧪 **Try It Yourself:** Create a new Cargo project called `cargo_practice`. Add a `println!` that prints your name. Build and run it. Then change the message to something else and run it again.

## Installing Node.js and npm

**Node.js** is a JavaScript runtime that lets you run JavaScript outside of a web browser. **npm** (Node Package Manager) is included with Node.js and manages JavaScript packages — similar to how Cargo manages Rust crates.

We need Node.js and npm for the SvelteKit frontend.

### On Linux (Arch-based)

```bash
sudo pacman -S nodejs npm
```

### On Linux (Debian/Ubuntu)

```bash
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
```

### On macOS

```bash
brew install node
```

(You'll need [Homebrew](https://brew.sh) for this command.)

### Verifying the Installation

```bash
node --version
# Should show: v20.x.x or similar

npm --version
# Should show: 10.x.x or similar
```

Both should show version numbers. If you see those, you're good to go!

💡 **Key Concept:** Node.js isn't just for running SvelteKit's development server. We also use npm to install packages — JavaScript libraries that other people have written and shared. Just like Cargo has crates.io, npm has npmjs.com. You'll see us running `npm install` and `npx` a lot.

## Installing PostgreSQL

**PostgreSQL** is our database — the filing cabinet where all the story data, user information, and suggestions will live.

### On Linux (Arch-based)

```bash
sudo pacman -S postgresql
```

Then initialize and start the database:

```bash
sudo postgresql-setup --initdb
sudo systemctl enable --now postgresql
```

### On Linux (Debian/Ubuntu)

```bash
sudo apt-get install -y postgresql postgresql-contrib
sudo systemctl enable --now postgresql
```

### On macOS

```bash
brew install postgresql@16
brew services start postgresql@16
```

### Creating the FicHub Database

After PostgreSQL is running, we need to create a database for FicHub:

```bash
# Create a user and database
sudo -u postgres createuser --interactive --pwprompt fichub
sudo -u postgres createdb -O fichub fichub
```

When prompted for a password, enter something simple for development (like `fichub123`). When asked "Shall the new role be a superuser?", answer **No**. When asked "Shall the new role be allowed to create databases?", answer **Yes**.

Let's verify the connection:

```bash
psql -U fichub -d fichub -h localhost
```

Enter the password you set, and you should see something like:

```
psql (16.2)
Type "help" for help.

fichub=>
```

Type `\q` to quit.

### Configuring Authentication

If you get an authentication error, you may need to edit PostgreSQL's authentication config:

```bash
# Find the config file
sudo find / -name "pg_hba.conf" 2>/dev/null
```

It's usually at `/var/lib/postgres/data/pg_hba.conf` or `/etc/postgresql/16/main/pg_hba.conf`. Edit it and find the line that says:

```
local   all   all   peer
```

Change it to:

```
local   all   all   md5
```

Then restart PostgreSQL:

```bash
sudo systemctl restart postgresql
```

⚠️ **Watch Out!** PostgreSQL on Arch Linux runs as the `postgres` user, while on Ubuntu it uses a system account named `postgres`. The commands are slightly different, so make sure you're using the right ones for your system.

## Installing Redis (or Valkey)

**Redis** is an in-memory data store — super fast, perfect for caching. We use it to store recommendation scores, rate limiting counters, and temporary data that needs to be accessed quickly.

**Valkey** is a drop-in open-source replacement for Redis (after Redis changed its license). Either one works.

### On Linux (Arch-based)

```bash
sudo pacman -S redis
sudo systemctl enable --now redis
```

Or for Valkey:

```bash
sudo pacman -S valkey
sudo systemctl enable --now valkey
```

### On Linux (Debian/Ubuntu)

```bash
sudo apt-get install -y redis-server
sudo systemctl enable --now redis-server
```

### On macOS

```bash
brew install redis
brew services start redis
```

### Verifying the Installation

```bash
redis-cli ping
```

Should respond with:

```
PONG
```

If you see `PONG`, Redis is running and ready to go.

### Connecting to Redis

The basic Redis commands:

```bash
redis-cli                # Open the Redis command line
SET mykey "hello"        # Store a value
GET mykey                # Retrieve a value → "hello"
DEL mykey                # Delete a value
PING                    # Check if Redis is alive → PONG
```

Redis is like a dictionary — you store values with string keys and retrieve them instantly. It's incredibly fast because everything lives in memory (RAM), not on disk.

💡 **Key Concept:** Redis is NOT a replacement for PostgreSQL. PostgreSQL stores all our permanent data (works, sources, users, bookmarks). Redis stores temporary, frequently-accessed data (caches, counters, session tokens). Think of PostgreSQL as a filing cabinet in a vault, and Redis as a whiteboard on your desk.

## A Code Editor

You need a good code editor — the tool you'll spend most of your time looking at. While you can technically write code in any text editor, a good editor with syntax highlighting, autocomplete, and error checking makes a huge difference.

### VS Code (Recommended)

**Visual Studio Code** (VS Code) is free, fast, and has excellent support for both Rust and Svelte/SvelteKit.

Install it from [code.visualstudio.com](https://code.visualstudio.com).

After installing, add these extensions:

1. **rust-analyzer** — Gives you autocomplete, error checking, and inline documentation for Rust code. It's almost essential for Rust development.
2. **Svelte for VS Code** — Syntax highlighting and autocomplete for Svelte components.
3. **Prettier** — Automatically formats your code on save.
4. **Error Lens** — Shows errors inline in the code, right next to the line that's wrong.

To install extensions, open VS Code, press `Ctrl+Shift+X` (or `Cmd+Shift+X` on macOS), search for each extension name, and click Install.

### Other Editors

If you prefer something else, that's totally fine! Some alternatives:

- **Helix** — A modern terminal-based editor (written in Rust!) with built-in LSP support
- **Neovim** — Powerful terminal editor with the right plugins
- **Zed** — A fast, collaborative editor
- **Sublime Text** — Lightweight and fast

The important thing is that you have something you're comfortable with. Don't spend three days configuring the "perfect" editor setup before writing any code. Pick something, start using it, and customize as you go.

## Terminal Basics

The **terminal** (also called the command line, shell, or console) is where you'll spend a lot of time as a developer. It's a text-based interface for interacting with your computer. Instead of clicking icons, you type commands.

Don't be scared of the terminal. It's just a text interface. You type a command, press Enter, and something happens. That's all.

Here are the essential commands:

### Navigating Directories

```bash
pwd                     # Print Working Directory — where am I?
ls                      # List files in the current directory
ls -la                  # List ALL files (including hidden ones) with details
cd ~/projects           # Change Directory to ~/projects
cd ..                   # Go up one directory
cd ~                    # Go to your home directory
```

### Creating Things

```bash
mkdir my_project        # Make a new directory called my_project
touch file.txt          # Create an empty file
```

### File Operations

```bash
cp file.txt backup.txt  # Copy a file
mv file.txt newname.txt # Rename/move a file
rm file.txt             # Delete a file (careful! No undo!)
rm -rf directory/       # Delete a directory and everything in it (VERY careful!)
```

### Viewing Files

```bash
cat file.txt            # Print the entire file contents
head -n 20 file.txt     # Print the first 20 lines
tail -n 20 file.txt     # Print the last 20 lines
less file.txt           # View file with scrolling (press q to quit)
```

### Finding Things

```bash
find . -name "*.rs"     # Find all .rs files in the current directory
grep "pattern" file.txt # Search for "pattern" inside file.txt
which cargo             # Find where cargo is installed
```

💡 **Key Concept:** Almost every command has flags (options) that change its behavior. The `man` command shows you the manual: `man ls` will tell you everything `ls` can do. When in doubt, `man <command>` is your friend.

🧪 **Try It Yourself:** Open your terminal and try these steps:

1. Run `pwd` to see where you are
2. Create a directory: `mkdir practice`
3. Change into it: `cd practice`
4. Create a file: `touch notes.txt`
5. List files: `ls -la`
6. Go back: `cd ..`
7. Remove the directory: `rm -rf practice`

If all that worked, you're ready for the next step!

## Creating the Project Directory

Now let's set up the FicHub project structure. We'll put everything in `~/code/rust/fichub/`:

```bash
# Create the main project directory
mkdir -p ~/code/rust/fichub
cd ~/code/rust/fichub

# Create the backend (Rust) project
cargo new backend
cd backend

# Add our main dependencies to Cargo.toml
# (We'll explain each one later)
cat > Cargo.toml << 'EOF'
[package]
name = "fichub-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.9", features = ["runtime-tokio", "postgres", "uuid", "chrono"] }
redis = { version = "0.27", features = ["tokio-comp"] }
reqwest = { version = "0.12", features = ["rustls-tls"] }
scraper = "0.22"
epub-builder = "0.8"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tower-http = { version = "0.6", features = ["cors", "trace"] }
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1"
thiserror = "2"
EOF

# Go back to the main directory
cd ..

# Create the frontend (SvelteKit) project
npx sv create frontend --template minimal --types ts --no-add-ons
cd frontend
npm install
cd ..

# Create supporting directories
mkdir -p book/parts
mkdir -p docs
```

Let me explain what each Rust dependency does:

| Crate | What it does |
|-------|-------------|
| `axum` | Web framework — handles HTTP requests and routes |
| `tokio` | Async runtime — lets Rust do multiple things at once |
| `serde` + `serde_json` | Serialization — convert data to/from JSON |
| `sqlx` | Database access — connect to PostgreSQL and run queries |
| `redis` | Redis client — connect to Redis |
| `reqwest` | HTTP client — make requests to fanfiction sites |
| `scraper` | HTML parser — extract data from web pages |
| `epub-builder` | EPUB generator — create ebook files |
| `uuid` | UUID generator — create unique IDs |
| `chrono` | Date/time handling |
| `tower-http` | HTTP middleware — CORS, logging |
| `tracing` + `tracing-subscriber` | Logging — see what's happening |
| `anyhow` + `thiserror` | Error handling — deal with things going wrong |

### Verify the Project Structure

```bash
ls -la ~/code/rust/fichub/
```

You should see something like:

```
backend/
  Cargo.toml
  src/
    main.rs
frontend/
  package.json
  src/
  svelte.config.js
  ...
book/
  parts/
docs/
```

### Test the Backend

```bash
cd ~/code/rust/fichub/backend
cargo build
```

The first time you run this, it will download and compile all the dependencies. This might take a few minutes — Rust is compiling a lot of code for the first time. Subsequent builds will be much faster thanks to Cargo's caching.

If it finishes with `Finished dev [unoptimized + debuginfo] target(s)`, everything is set up correctly.

### Test the Frontend

```bash
cd ~/code/rust/fichub/frontend
npm run dev
```

You should see something like:

```
VITE v6.x.x  ready in 300 ms

  ➜  Local:   http://localhost:5173/
```

Open that URL in your browser, and you should see the SvelteKit default page. Press `Ctrl+C` to stop the server.

## Summary: Your Workshop is Ready!

Let's recap what we installed:

### A Note About Versions

Throughout this book, we'll use specific versions of tools and libraries. You might have slightly different versions installed — and that's usually fine. Software evolves, and exact version numbers become outdated. What matters is that you have a recent enough version to support the features we use.

If you run into a version-related error, check the error message carefully. It often tells you which version is required. You can usually update with:

```bash
rustup update          # Update Rust
nvm install --lts      # Update Node.js (if using nvm)
sudo pacman -Syu       # Update system packages (Arch)
```

### Troubleshooting Common Issues

Here are some issues you might hit and how to fix them:

**"command not found: cargo"** — Rust isn't in your PATH. Run `source "$HOME/.cargo/env"` or restart your terminal.

**"error: failed to connect to PostgreSQL"** — PostgreSQL might not be running. Check with `systemctl status postgresql`.

**"error: password authentication failed"** — The pg_hba.conf file might need the `md5` change we described earlier.

**"EADDRINUSE: port already in use"** — Another process is using port 3000 or 5173. Find it with `lsof -i :3000` and either kill it or use a different port.

**npm permission errors** — On some systems, npm global installs need `sudo`. Better yet, install Node.js via nvm to avoid permission issues entirely:

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
nvm install 20
nvm use 20
```

These kinds of troubleshooting skills will serve you well throughout your programming career. When something doesn't work, read the error message, search for it online, and try the suggested fixes. Every programmer does this daily.

| Tool | Version | Purpose |
|------|---------|---------|
| Rust (`rustc`) | 1.78+ | Programming language (backend) |
| Cargo | 1.78+ | Rust package manager and build tool |
| Node.js | 20+ | JavaScript runtime (frontend) |
| npm | 10+ | JavaScript package manager |
| PostgreSQL | 16+ | Database (storing data) |
| Redis/Valkey | 7+ | Cache (fast temporary storage) |
| VS Code | latest | Code editor |

And our project structure:

```
~/code/rust/fichub/
├── backend/         # Rust Axum server
│   ├── Cargo.toml
│   └── src/main.rs
├── frontend/        # SvelteKit app
│   ├── package.json
│   └── src/
├── book/            # This book!
│   └── parts/
└── docs/            # Documentation
```

Everything is installed, everything compiles, and the project structure is in place. In the next chapter, we'll write our first real programs in both Rust and Svelte.

---

# Chapter 4: Your First Rust and SvelteKit Programs

## Time to Write Code!

Your workshop is set up. The tools are installed. The project structure exists. Now it's time to actually write some code.

We're going to start with Rust basics — just enough to understand the language before we dive into building the backend. Then we'll jump over to the frontend and get a SvelteKit page running. By the end of this chapter, you'll have written real, working code in both languages, and you'll understand enough to follow along when we start building FicHub's features.

Let's dive in.

## Hello, World! in Rust

We've already seen "Hello, World!" once, but let's actually understand it. Open `~/code/rust/fichub/backend/src/main.rs`:

```rust
fn main() {
    println!("Hello, world!");
}
```

Let's break this down piece by piece:

- `fn` — This means "function." In Rust, `fn` is how you declare a function.
- `main` — This is the name of the function. `main` is special — it's the entry point of every Rust program. When you run a Rust program, the `main` function is what gets executed first.
- `()` — The parentheses after the function name are for parameters. Empty parentheses means this function takes no arguments.
- `{ }` — The curly braces contain the function body — the code that runs when this function is called.
- `println!` — This is a **macro** (note the `!`). It's like a function but more powerful — it can take a variable number of arguments and does some compile-time magic. `println!` prints text to the terminal with a newline.
- `"Hello, world!"` — A string literal. In Rust, text is enclosed in double quotes.

### The Semicolon

Notice the semicolon `;` at the end of the `println!` line. In Rust, semicolons end **statements** (actions). Forgetting semicolons is one of the most common mistakes beginners make, and Rust will let you know with an error.

⚠️ **Watch Out!** Rust errors can look scary at first, but they're actually very helpful. If you forget a semicolon, Rust will say something like:

```
error: expected `;`
 --> src/main.rs:2:36
  |
2 |     println!("Hello, world!")
  |                                    ^ expected `;`
```

Read the error message carefully — Rust is telling you exactly what's wrong and where.

## Variables: Storing Things

Variables are how you store data. Think of them like labeled boxes — you put something inside, put a label on the box, and later you can look up the label to find what's inside.

### `let` — Creating Variables

```rust
fn main() {
    let name = "Alice";
    let age = 25;
    let height = 5.6;
    let is_student = true;

    println!("{} is {} years old", name, age);
}
```

Here we created four variables:
- `name` holds a **string slice** (`"Alice"`) — text
- `age` holds an **integer** (`25`) — a whole number
- `height` holds a **float** (`5.6`) — a decimal number
- `is_student` holds a **boolean** (`true`) — true or false

Rust figures out the type automatically based on the value. This is called **type inference**. You rarely need to write the type yourself.

### `mut` — Making Variables Mutable

By default, variables in Rust are **immutable** — once you set them, you can't change them. This is like writing on paper in pen. If you want to change a variable, you need to declare it as **mutable** with `mut`:

```rust
fn main() {
    let count = 0;       // immutable
    // count = 1;        // ERROR! Can't change this

    let mut counter = 0;  // mutable
    counter = 1;          // This works!
    counter = 2;          // And this!

    println!("Counter: {}", counter);
}
```

This might seem annoying at first, but it's actually a feature. Immutable variables are safer — you know they won't change unexpectedly. We'll use `mut` when we actually need to change a value.

💡 **Key Concept:** Rust defaults to immutability for safety. When you DO need to change something, you explicitly say so with `mut`. This is a core philosophy of Rust: make the safe thing easy and the dangerous thing require effort.

### Variable Types

Rust has several basic types:

```rust
fn main() {
    // Integers
    let whole_number: i32 = 42;          // signed 32-bit
    let big_number: i64 = 1_000_000;     // signed 64-bit (underscores are visual separators)
    let positive_only: u32 = 42;         // unsigned 32-bit (no negatives)

    // Floats
    let decimal: f64 = 3.14159;          // 64-bit float (default)
    let precise: f32 = 2.71;             // 32-bit float

    // Boolean
    let is_active: bool = true;

    // Character (single character in single quotes)
    let letter: char = 'A';

    // String (text in double quotes)
    let greeting: &str = "Hello";        // string slice (borrowed)
    let owned: String = String::from("Hello"); // owned string

    println!("{}, {}. π ≈ {}", greeting, letter, decimal);
}
```

The most common types you'll use are `i32` (or just `i32`), `f64`, `bool`, `char`, `&str`, and `String`. Don't worry about memorizing all of these — you'll get a feel for which ones to use.

🧪 **Try It Yourself:** Create a variable for your favorite book title, one for the number of pages, and one for whether you've finished it. Print all three in a single `println!` using `{}` placeholders.

## Functions: Reusable Blocks

Functions are reusable blocks of code. You write the code once, give it a name, and then call it whenever you need it — as many times as you want.

```rust
fn greet(name: &str) {
    println!("Hello, {}! Welcome to FicHub.", name);
}

fn add(a: i32, b: i32) -> i32 {
    a + b  // No semicolon! This is a return value.
}

fn main() {
    greet("Alice");
    greet("Bob");

    let sum = add(3, 5);
    println!("3 + 5 = {}", sum);
}
```

Let's break down the `add` function:

- `fn add(a: i32, b: i32) -> i32` — Takes two parameters `a` and `b`, both `i32`. Returns an `i32` (the `-> i32` part).
- `a + b` — The last expression in a function is the return value.
- **No semicolon** — This is important! If you put a semicolon after `a + b`, it becomes a statement, and the function returns `()` (nothing). This is one of the most confusing Rust quirks for beginners.

The `greet` function has no return value, so it doesn't have the `->` part. Functions that don't return anything are implicitly returning `()` (the "unit type" — Rust's way of saying "nothing").

⚠️ **Watch Out!** If you write:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b;  // ← semicolon here!
}
```

Rust will complain:

```
error: mismatched types
  |
  |     a + b;
  |           - help: remove this semicolon to return this value
```

The semicolon turns the expression into a statement, and the function returns nothing instead of the sum. Remove the semicolon!

## Structs: Grouping Related Data

Sometimes you need to group several pieces of data together under one name. A **struct** (structure) lets you do exactly that.

Think of a struct like a form. A library card has fields for name, address, phone number, and card number. A Rust struct does the same thing — it defines what fields a piece of data has.

```rust
struct Story {
    title: String,
    author: String,
    word_count: u32,
    is_complete: bool,
}

fn main() {
    let story = Story {
        title: String::from("The Luminary"),
        author: String::from("starweaver42"),
        word_count: 85000,
        is_complete: true,
    };

    println!(
        "\"{}\" by {} ({} words) - {}",
        story.title,
        story.author,
        story.word_count,
        if story.is_complete { "Complete" } else { "In Progress" }
    );
}
```

A few things to notice:

- `struct Story` defines the structure — it says "a Story has a title, author, word_count, and is_complete."
- We create an instance with `Story { ... }`, filling in each field.
- We access fields with a dot: `story.title`, `story.author`, etc.
- `String::from(...)` creates an owned String (we'll talk about this in a moment).

### Methods on Structs

You can add functions that belong to a struct using `impl`:

```rust
struct Story {
    title: String,
    author: String,
    word_count: u32,
}

impl Story {
    // A method — has &self as the first parameter
    fn summary(&self) -> String {
        format!("\"{}\" by {} ({} words)", self.title, self.author, self.word_count)
    }

    // A method that takes &mut self (can modify the story)
    fn update_word_count(&mut self, new_count: u32) {
        self.word_count = new_count;
    }
}

fn main() {
    let mut story = Story {
        title: String::from("The Luminary"),
        author: String::from("starweaver42"),
        word_count: 85000,
    };

    println!("{}", story.summary());       // "The Luminary" by starweaver42 (85000 words)
    story.update_word_count(86500);
    println!("{}", story.summary());       // Updated!
}
```

- `&self` in the method signature means "this method borrows the struct" (reads it without taking ownership)
- `&mut self` means "this method can modify the struct"
- `self` (without `&`) would mean "this method takes ownership of the struct" (destructive — rarely used)

### Lifetimes and Ownership — Don't Panic

You might have heard that Rust has "ownership" and "borrowing" rules. These are important concepts, but don't worry about mastering them right now. Here's the simple version:

- **Ownership:** Every value has exactly one owner. When the owner goes out of scope (leaves the function, for example), the value is dropped (freed from memory).
- **Borrowing:** You can let someone "read" your value (using `&`) without giving up ownership. This is called an "immutable reference."
- **Mutable Borrowing:** You can let someone "read and modify" your value (using `&mut`), but only if no one else is reading it at the same time.

These rules prevent memory bugs (use-after-free, double-free, data races) at compile time. It's Rust's superpower. For now, just follow the compiler's suggestions — when it tells you to add `&` or `&mut`, do it.

Here's a quick mental model: think of ownership like a library book. Only one person can have the book checked out at a time. You can let someone read it (borrowing with `&`), and you can even let someone take notes in it (mutable borrowing with `&mut`), but only one person can modify it at a time. And when you return the book (the owner goes out of scope), the library reclaims it.

You don't need to understand every edge case right now. Rust's compiler is incredibly helpful — it will suggest fixes for ownership errors, and most of the time, the fix is exactly what you'd expect. As you write more Rust, these rules become second nature.

## Enums: Choices and Options

An **enum** (enumeration) is a type that can be one of several variants. Think of it like a multiple-choice question — the answer can be A, B, C, or D, but never two at once.

```rust
enum Site {
    Ao3,
    FanfictionNet,
    FictionPress,
    SpaceBattles,
    SufficientVelocity,
    AdultFanFiction,
}

fn scraper_name(site: Site) -> &'static str {
    match site {
        Site::Ao3 => "AO3 Scraper",
        Site::FanfictionNet => "FFN Scraper",
        Site::FictionPress => "FictionPress Scraper",
        Site::SpaceBattles => "SpaceBattles Scraper",
        Site::SufficientVelocity => "SufficientVelocity Scraper",
        Site::AdultFanFiction => "AdultFanFiction Scraper",
    }
}

fn main() {
    let site = Site::Ao3;
    println!("Using {}", scraper_name(site));
}
```

- `enum Site` defines a type with six variants.
- `Site::Ao3` is one variant of the enum.
- `match site` is pattern matching — like a switch statement, but more powerful and safer.
- Rust **requires** you to handle every variant. If you forget one, the compiler will tell you. This prevents bugs where you forget to handle a case.

### Enums with Data

Enums get really powerful when variants can hold data:

```rust
enum DownloadStatus {
    Pending,
    Downloading { progress: u8 },  // holds a percentage
    Complete(String),               // holds the file path
    Error(String),                  // holds an error message
}

fn main() {
    let status = DownloadStatus::Downloading { progress: 42 };

    match status {
        DownloadStatus::Pending => println!("Waiting to start..."),
        DownloadStatus::Downloading { progress } => {
            println!("Downloading... {}%", progress);
        }
        DownloadStatus::Complete(path) => println!("Done! Saved to {}", path),
        DownloadStatus::Error(msg) => println!("Error: {}", msg),
    }
}
```

Each variant can hold different data — `Downloading` holds a progress percentage, `Complete` holds a file path, `Error` holds a message. The `match` expression handles each variant and can access the data inside.

💡 **Key Concept:** Enums in Rust are one of the most powerful features in the language. We use them extensively in FicHub — for site types, download states, error types, and more. Get comfortable with `match` because you'll use it everywhere.

### The `Option` Type

Rust doesn't have `null` — a special value that means "nothing." Instead, it has `Option<T>`, an enum that represents either something (`Some(value)`) or nothing (`None`):

```rust
fn find_story(id: i32) -> Option<String> {
    if id == 42 {
        Some(String::from("The Luminary"))
    } else {
        None
    }
}

fn main() {
    match find_story(42) {
        Some(title) => println!("Found: {}", title),
        None => println!("Story not found"),
    }

    // Or use if let for simpler cases
    if let Some(title) = find_story(42) {
        println!("Found: {}", title);
    }
}
```

The compiler forces you to handle the `None` case. You can't accidentally use a null value and crash. This is incredibly useful — FicHub uses `Option` everywhere: "did the scraper find a cover image? `Option<String>`" or "does this user have a linked AO3 account? `Option<String>`".

## Error Handling: Result and ?

Things go wrong. Networks fail, files don't exist, users paste invalid URLs. Rust handles this gracefully with the `Result<T, E>` type.

`Result` is an enum with two variants: `Ok(value)` (success) and `Err(error)` (failure).

```rust
use std::num::ParseIntError;

fn parse_word_count(text: &str) -> Result<u32, ParseIntError> {
    text.parse::<u32>()
}

fn main() {
    // Success case
    match parse_word_count("85000") {
        Ok(count) => println!("Word count: {}", count),
        Err(e) => println!("Parse error: {}", e),
    }

    // Failure case
    match parse_word_count("not a number") {
        Ok(count) => println!("Word count: {}", count),
        Err(e) => println!("Parse error: {}", e),
    }
}
```

`text.parse::<u32>()` tries to convert a string to a number. If the string is a valid number, it returns `Ok(85000)`. If not, it returns `Err(...)`.

### The `?` Operator

Writing `match` for every error gets tedious. The `?` operator makes it cleaner:

```rust
use std::num::ParseIntError;

fn parse_word_count(text: &str) -> Result<u32, ParseIntError> {
    text.parse::<u32>()
}

// Using ? to propagate errors
fn process_story_data(word_count_str: &str, title: &str) -> Result<String, ParseIntError> {
    let word_count = parse_word_count(word_count_str)?;  // ? = return Err if this fails
    Ok(format!("\"{}\" has {} words", title, word_count))
}

fn main() {
    match process_story_data("85000", "The Luminary") {
        Ok(summary) => println!("{}", summary),
        Err(e) => println!("Error: {}", e),
    }

    match process_story_data("not a number", "The Luminary") {
        Ok(summary) => println!("{}", summary),
        Err(e) => println!("Error: {}", e),
    }
}
```

The `?` means: "If this is `Ok`, unwrap the value and continue. If this is `Err`, return the error immediately." It's like saying "try this, and if it fails, just bail out."

We'll use the `?` operator constantly in FicHub. Every time we fetch a web page, parse HTML, or read from the database, errors are possible, and `?` lets us handle them cleanly.

### `anyhow` and `thiserror`

In FicHub, we use two helper crates for error handling:

- **`anyhow`** — Gives us `anyhow::Result<T>`, a convenience type that can hold any error. Great for application code where you don't need to distinguish between error types.
- **`thiserror`** — Helps us define custom error types for our library code.

You'll see these in action when we start building the backend.

## async/await: Doing Multiple Things at Once

Rust is a single-threaded language by default — it does one thing at a time. But web servers need to handle many requests simultaneously. **async/await** is how Rust does concurrency — running multiple tasks at the same time without blocking.

Think of it like a chef in a kitchen. A single-threaded chef cooks one dish at a time. An async chef starts boiling water, and while the water heats up, they start chopping vegetables. They don't wait for the water to boil before doing anything else — they do other work while waiting.

```rust
use tokio::time::{sleep, Duration};

async fn fetch_story(url: &str) -> String {
    // Simulate fetching a story (takes 2 seconds)
    sleep(Duration::from_secs(2)).await;
    format!("Content from {}", url)
}

async fn fetch_metadata(url: &str) -> String {
    // Simulate fetching metadata (takes 1 second)
    sleep(Duration::from_secs(1)).await;
    format!("Metadata from {}", url)
}

#[tokio::main]
async fn main() {
    // Sequential: 3 seconds total
    let start = std::time::Instant::now();
    let _story = fetch_story("example.com/story").await;
    let _meta = fetch_metadata("example.com/meta").await;
    println!("Sequential: {}ms", start.elapsed().as_millis());

    // Concurrent: 2 seconds total (the longer task)
    let start = std::time::Instant::now();
    let (story, meta) = tokio::join!(
        fetch_story("example.com/story"),
        fetch_metadata("example.com/meta")
    );
    println!("Concurrent: {}ms", start.elapsed().as_millis());
    println!("Story: {}", story);
    println!("Meta: {}", meta);
}
```

Key points:

- `async fn` — Declares an asynchronous function. It returns a future (a value that will be ready later).
- `.await` — Pauses until the async operation completes, but **yields control** so other tasks can run. It does NOT block the thread.
- `#[tokio::main]` — Sets up the async runtime (Tokio) that manages all the concurrent tasks.
- `tokio::join!` — Runs multiple async functions concurrently and waits for all of them to finish.

⚠️ **Watch Out!** The `.await` keyword only works inside `async` functions. If you try to use it in a regular `fn`, Rust will give you a clear error. Also, `#[tokio::main]` is required for async programs — without it, there's no runtime to execute async tasks.

### Why async Matters for FicHub

When FicHub downloads a story, it needs to:
1. Fetch the story's main page (network I/O — slow)
2. Fetch each chapter (network I/O — slow)
3. Parse the HTML (CPU work — fast)
4. Generate the EPUB (CPU work — fast)

Without async, these operations would happen one at a time. With async, we can fetch multiple chapters simultaneously, dramatically speeding up the download. When you download a 50-chapter story, instead of waiting 50 × 200ms = 10 seconds, we can fetch several chapters in parallel.

### The `tokio::spawn` Pattern

Sometimes you want to start a task and not wait for it — like logging a download event or updating a cache. `tokio::spawn` creates a background task:

```rust
use tokio::time::{sleep, Duration};

async fn log_download(url: &str) {
    sleep(Duration::from_millis(50)).await; // Simulate DB write
    println!("Logged download: {}", url);
}

async fn download_story(url: &str) -> String {
    // Start logging in the background — don't wait for it
    tokio::spawn(log_download(url.to_string()));

    // Do the actual download
    sleep(Duration::from_secs(2)).await;
    format!("Story from {}", url)
}

#[tokio::main]
async fn main() {
    let story = download_story("example.com/story").await;
    println!("Got: {}", story);
    // The log might print after this line — that's fine!
    sleep(Duration::from_millis(100)).await;
}
```

This fire-and-forget pattern is useful for side effects that shouldn't slow down the main request.

## Your First SvelteKit Page

Now let's switch gears and look at the frontend. Open your SvelteKit project:

```bash
cd ~/code/rust/fichub/frontend
```

Let's look at the project structure:

```bash
ls -la src/
```

You'll see:

```
src/
  lib/
    assets/
    components/
    ...
  routes/
    +page.svelte
    +layout.svelte
    +layout.ts
  app.html
```

The key file is `src/routes/+page.svelte` — this is the home page of your SvelteKit app. Let's replace its contents:

```svelte
<script>
    let title = 'FicHub';
    let tagline = 'Download fanfiction as ebooks';
    let url = '';
    let downloading = false;

    async function downloadStory() {
        if (!url) return;
        downloading = true;
        try {
            const response = await fetch('http://localhost:3000/api/download', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ url: url })
            });
            if (response.ok) {
                const blob = await response.blob();
                const a = document.createElement('a');
                a.href = URL.createObjectURL(blob);
                a.download = 'story.epub';
                a.click();
            }
        } catch (err) {
            console.error('Download failed:', err);
        } finally {
            downloading = false;
        }
    }
</script>

<div class="container">
    <h1>{title}</h1>
    <p>{tagline}</p>

    <div class="input-group">
        <input
            type="text"
            bind:value={url}
            placeholder="Paste a fanfiction URL..."
        />
        <button on:click={downloadStory} disabled={downloading}>
            {downloading ? 'Downloading...' : 'Download'}
        </button>
    </div>
</div>

<style>
    .container {
        max-width: 600px;
        margin: 4rem auto;
        text-align: center;
        font-family: sans-serif;
    }

    h1 {
        font-size: 3rem;
        color: #4fc3f7;
    }

    p {
        color: #999;
        font-size: 1.2rem;
    }

    .input-group {
        margin-top: 2rem;
        display: flex;
        gap: 0.5rem;
    }

    input {
        flex: 1;
        padding: 0.75rem 1rem;
        border: 2px solid #333;
        border-radius: 8px;
        background: #1a1a2e;
        color: white;
        font-size: 1rem;
    }

    button {
        padding: 0.75rem 1.5rem;
        border: none;
        border-radius: 8px;
        background: #4fc3f7;
        color: #1a1a2e;
        font-size: 1rem;
        font-weight: bold;
        cursor: pointer;
    }

    button:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
</style>
```

This is a Svelte component. Let's break it down:

### The `<script>` Block

```svelte
<script>
    let title = 'FicHub';
    let tagline = 'Download fanfiction as ebooks';
    let url = '';
    let downloading = false;
</script>
```

This is where your JavaScript lives. `let` creates variables (just like Rust!). These variables are **reactive** — when they change, the parts of the page that use them automatically update.

The `downloadStory` function handles the download — it sends the URL to our Rust backend, gets back an EPUB file, and triggers a browser download.

### The HTML Template

```svelte
<div class="container">
    <h1>{title}</h1>
    <p>{tagline}</p>
    ...
</div>
```

This looks almost like regular HTML, but with curly braces `{}` for inserting JavaScript values. `{title}` inserts the value of the `title` variable. When the variable changes, the page updates automatically.

- `bind:value={url}` creates two-way binding — when you type in the input, `url` updates, and when `url` changes, the input updates.
- `on:click={downloadStory}` calls the `downloadStory` function when the button is clicked.
- `{downloading ? 'Downloading...' : 'Download'}` is a ternary expression — it shows different text depending on whether we're currently downloading.

### The `<style>` Block

```svelte
<style>
    .container {
        max-width: 600px;
        margin: 4rem auto;
        text-align: center;
    }
</style>
```

This is CSS, scoped to this component. The styles only apply to elements inside this component — they won't leak out and affect other parts of the page. This is one of Svelte's great features.

Run the frontend to see it:

```bash
cd ~/code/rust/fichub/frontend
npm run dev
```

Open `http://localhost:5173` in your browser. You should see the FicHub title, tagline, a text input, and a download button. It won't actually download anything yet (the backend isn't running), but the interface is there!

## Svelte 5 Runes: The New Way

If you're using Svelte 5 (which the latest SvelteKit scaffolds with), you'll see **runes** — special syntax that makes reactivity explicit. Let's update our component to use runes:

```svelte
<script>
    let title = $state('FicHub');
    let tagline = $state('Download fanfiction as ebooks');
    let url = $state('');
    let downloading = $state(false);

    let buttonText = $derived(downloading ? 'Downloading...' : 'Download');

    $effect(() => {
        console.log('URL changed:', url);
    });

    async function downloadStory() {
        if (!url) return;
        downloading = true;
        try {
            const response = await fetch('http://localhost:3000/api/download', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ url: url })
            });
            if (response.ok) {
                const blob = await response.blob();
                const a = document.createElement('a');
                a.href = URL.createObjectURL(blob);
                a.download = 'story.epub';
                a.click();
            }
        } catch (err) {
            console.error('Download failed:', err);
        } finally {
            downloading = false;
        }
    }
</script>

<div class="container">
    <h1>{title}</h1>
    <p>{tagline}</p>

    <div class="input-group">
        <input
            type="text"
            bind:value={url}
            placeholder="Paste a fanfiction URL..."
        />
        <button on:click={downloadStory} disabled={downloading}>
            {buttonText}
        </button>
    </div>
</div>
```

Here are the three runes we used:

### `$state` — Declaring Reactive State

```svelte
let url = $state('');
```

`$state` declares a reactive variable. When this value changes, any part of the template that reads it will update. It's the Svelte 5 way of saying "this value might change, and when it does, the UI should re-render."

### `$derived` — Computed Values

```svelte
let buttonText = $derived(downloading ? 'Downloading...' : 'Download');
```

`$derived` creates a value that automatically recalculates whenever its dependencies change. `buttonText` depends on `downloading`. When `downloading` changes, `buttonText` updates automatically. It's like a formula in a spreadsheet — change one cell, and the formula recalculates.

### `$effect` — Side Effects

```svelte
$effect(() => {
    console.log('URL changed:', url);
});
```

`$effect` runs a function whenever its dependencies change. This is useful for things like logging, making API calls, or syncing with external systems. The function runs after the DOM updates, so you can safely access elements.

💡 **Key Concept:** Svelte 5 runes (`$state`, `$derived`, `$effect`) make reactivity explicit. In Svelte 4, any `let` was reactive. In Svelte 5, you choose which variables are reactive with `$state`. This is clearer and avoids subtle bugs.

## Components and Props

Real SvelteKit apps aren't one big file — they're split into **components**, reusable pieces of UI. Let's create a Story Card component:

```bash
mkdir -p src/lib/components
```

Create `src/lib/components/StoryCard.svelte`:

```svelte
<script>
    let { title, author, wordCount, tags = [] } = $props();
</script>

<div class="card">
    <h3>{title}</h3>
    <p class="author">by {author}</p>
    <p class="words">{wordCount.toLocaleString()} words</p>
    <div class="tags">
        {#each tags as tag}
            <span class="tag">{tag}</span>
        {/each}
    </div>
</div>

<style>
    .card {
        background: #16213e;
        border-radius: 12px;
        padding: 1.5rem;
        transition: transform 0.2s, box-shadow 0.2s;
    }

    .card:hover {
        transform: translateY(-4px);
        box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
    }

    h3 {
        margin: 0 0 0.5rem;
        color: white;
    }

    .author {
        color: #999;
        margin: 0 0 0.5rem;
    }

    .words {
        color: #4fc3f7;
        margin: 0 0 1rem;
    }

    .tags {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
    }

    .tag {
        background: #0f3460;
        color: #4fc3f7;
        padding: 0.25rem 0.75rem;
        border-radius: 20px;
        font-size: 0.85rem;
    }
</style>
```

Now use it in your main page:

```svelte
<script>
    import StoryCard from '$lib/components/StoryCard.svelte';

    let stories = [
        {
            title: 'The Luminary',
            author: 'starweaver42',
            wordCount: 85000,
            tags: ['Harry Potter', 'Adventure', 'Slow Burn']
        },
        {
            title: 'Fate Is A Very Silly Thing',
            author: 'gnosticfury',
            wordCount: 120000,
            tags: ['Harry Potter', 'Humor', 'Time Travel']
        },
        {
            title: 'The Black Queen',
            author: 'Silently Watches',
            wordCount: 250000,
            tags: ['Harry Potter', 'Dark', 'Independent Harry']
        }
    ];
</script>

<div class="container">
    <h1>FicHub</h1>
    <p>Download fanfiction as ebooks</p>

    <div class="stories-grid">
        {#each stories as story}
            <StoryCard
                title={story.title}
                author={story.author}
                wordCount={story.wordCount}
                tags={story.tags}
            />
        {/each}
    </div>
</div>
```

Key concepts:

- `$props()` receives values passed to the component from outside. Here, `StoryCard` receives `title`, `author`, `wordCount`, and `tags`.
- `{#each stories as story}` is a loop — it creates a `StoryCard` for each story in the list.
- `import StoryCard from '$lib/components/StoryCard.svelte'` imports the component. `$lib` is a shortcut for `src/lib`.

### Props in Detail

```svelte
let { title, author, wordCount, tags = [] } = $props();
```

This uses destructuring to pull out individual props. The `tags = []` means "if tags isn't provided, default to an empty array." You can also type your props:

```svelte
/** @type {{ title: string, author: string, wordCount: number, tags: string[] }} */
let { title, author, wordCount, tags = [] } = $props();
```

When using StoryCard, you pass props like HTML attributes:

```svelte
<StoryCard title="My Story" author="Me" wordCount={50000} tags={['Action', 'Drama']} />
```

Notice that for string values you use quotes (`title="My Story"`), but for JavaScript expressions (numbers, arrays, variables) you use curly braces (`wordCount={50000}`).

🧪 **Try It Yourself:** 

1. Create a new component called `TagBadge.svelte` that takes a `label` prop and displays it as a styled badge.
2. Update `StoryCard.svelte` to use `TagBadge` instead of the raw `<span>` for tags.
3. Make the badge color change based on the tag text (e.g., "Harry Potter" gets a blue badge, "Dark" gets a red badge).

## Running Both Servers: The Full Setup

Now we have both a Rust backend and a SvelteKit frontend. To run FicHub, you need both running at the same time.

### Why Two Servers?

You might wonder why we need two separate servers instead of just one. The answer is separation of concerns:

- **The SvelteKit dev server** (port 5173) serves the frontend code and provides hot module replacement (HMR) — when you change a file, the browser automatically refreshes to show the change. This is great for rapid development.

- **The Axum server** (port 3000) handles the API — the heavy lifting of scraping stories, generating EPUBs, and querying the database. It doesn't know or care about the frontend.

In production, SvelteKit will be built into static files that Axum serves directly — one server, one port. But during development, running them separately means you can work on the frontend and backend independently without one rebuild affecting the other.

### Terminal 1: The Backend

```bash
cd ~/code/rust/fichub/backend
cargo run
```

This compiles and runs the Rust server. It will listen on port 3000.

### Terminal 2: The Frontend

Open a new terminal window (or a new tab):

```bash
cd ~/code/rust/fichub/frontend
npm run dev
```

This starts the SvelteKit development server on port 5173.

Now open `http://localhost:5173` in your browser. You'll see the FicHub interface.

The frontend (port 5173) sends API requests to the backend (port 3000). The backend processes those requests and sends responses back. This is the client-server architecture in action.

### The CORS Issue

If you see a CORS (Cross-Origin Resource Sharing) error in the browser console, that's because the frontend and backend are on different ports. We need to configure the backend to accept requests from the frontend. We'll fix this in the next chapter by adding CORS middleware to Axum.

For now, let's focus on getting both servers running and talking to each other.

### Running Both with One Command

You can use a tool like `concurrently` to run both servers with a single command:

```bash
cd ~/code/rust/fichub
npx concurrently "cd backend && cargo run" "cd frontend && npm run dev"
```

Or add a script to the root `package.json`:

```json
{
    "scripts": {
        "dev": "npx concurrently \"cd backend && cargo run\" \"cd frontend && npm run dev\""
    }
}
```

Then just run `npm run dev` from the project root to start everything.

## What You've Learned

In this chapter, you've learned the fundamentals of two programming languages. That's a lot to absorb, and nobody expects you to have memorized everything. The important thing is that you've been exposed to these concepts — variables, functions, structs, enums, error handling, async, components, reactivity, props. When we use them in later chapters, you'll recognize them and understand what they do.

Programming is a skill, not a knowledge test. You learn by doing. Every time you type a code snippet from this book, run it, and see it work, you're building muscle memory. Every error message you fix teaches you something new. Every feature you add to the project reinforces the patterns.

Here's what you've learned:

### Rust Basics
- **Variables** with `let` and `mut`
- **Functions** with `fn`, parameters, and return values
- **Structs** for grouping related data
- **Enums** for choices and pattern matching with `match`
- **Error handling** with `Result`, `Option`, and `?`
- **async/await** for concurrent operations

### SvelteKit Basics
- **Components** — reusable UI pieces in `.svelte` files
- **Reactivity** with `$state`, `$derived`, and `$effect`
- **Props** with `$props()`
- **Loops** with `{#each}`
- **Two-way binding** with `bind:value`
- **Event handling** with `on:click`

These are the building blocks we'll use throughout the rest of the book. You don't need to be an expert in either language — we'll learn as we build. But having these fundamentals means you'll recognize the patterns when you see them.

## A Quick Reference

Here's a cheat sheet for the Rust syntax we covered:

```rust
// Variables
let x = 5;              // immutable
let mut y = 10;         // mutable
let z: i64 = 100;       // explicit type

// Functions
fn add(a: i32, b: i32) -> i32 { a + b }

// Structs
struct Story { title: String, word_count: u32 }
impl Story { fn summary(&self) -> String { ... } }

// Enums
enum Site { Ao3, FanfictionNet }
match site { Site::Ao3 => ..., Site::FanfictionNet => ... }

// Error handling
fn parse(text: &str) -> Result<u32, Error> { text.parse::<u32>()? }

// Async
async fn fetch(url: &str) -> String { ... }
let result = fetch("...").await;
```

And for Svelte:

```svelte
<!-- Variables -->
let count = $state(0);
let doubled = $derived(count * 2);

<!-- Effects -->
$effect(() => { console.log(count); });

<!-- Props -->
let { name, age } = $props();

<!-- Template -->
<h1>{name}</h1>
{#each items as item}<p>{item}</p>{/each}

<!-- Events -->
<button on:click={handler}>Click</button>

<!-- Binding -->
<input bind:value={name} />
```

In the next part of the book, we'll build our first real Rust web server with Axum — a server that actually listens for HTTP requests and sends back responses. That's when things start getting really exciting.

Welcome to the project. Let's build FicHub. 🚀

### Before We Move On

Take a moment to make sure everything is working:

1. ✅ Rust compiles and runs (`cargo run` in the backend directory)
2. ✅ Node.js runs (`node --version` shows a version number)
3. ✅ PostgreSQL accepts connections (`psql -U fichub -d fichub -h localhost`)
4. ✅ Redis responds to PING (`redis-cli ping` returns PONG)
5. ✅ The SvelteKit dev server starts (`npm run dev` in the frontend directory)
6. ✅ You can see the FicHub interface at `http://localhost:5173`

If all six of these work, you're in great shape for Part 2, where we'll build the Rust backend — a real, working web server that listens for requests, talks to a database, and returns JSON. It's where FicHub starts to come alive.

If something doesn't work, don't panic. Go back to Chapter 3 and re-read the relevant installation section. Most problems are simple: a service isn't running, a port is blocked, or a PATH variable is missing. The error message usually tells you what's wrong.

See you in Part 2! 🚀

# Part 2: The Rust Backend

---

*In Part 1, we set up our tools — Rust, Node.js, PostgreSQL, Redis — and ran our first Hello World. Now we build the real thing. Welcome to the engine room.*

*This part takes you from an empty project to a fully functional backend with database, configuration, error handling, and all the routes wired up. By the end, you'll understand every line of the Rust code that powers FicHub.*

*Let's fire up the editor and get cooking.* 🚀

---

# Chapter 5: Your First Rust Web Server (Axum)

## Why Axum?

There are several web frameworks for Rust — Actix Web, Rocket, Warp, and Axum among them. We chose Axum for FicHub because:

1. **It's built by the Tokio team** — the same people who built the async runtime. This means deep integration and fewer surprises.
2. **It's minimal and composable** — Axum doesn't force opinions on you. It gives you routing, extractors, and middleware, and lets you choose the rest.
3. **It's type-safe** — if you extract a parameter that doesn't exist, your code won't compile. Errors are caught at compile time, not in production.
4. **It's well-maintained** — active development, good documentation, and a growing ecosystem of middleware and extensions.
5. **It's fast** — Axum consistently ranks among the fastest Rust web frameworks in benchmarks.

For a self-hosted server that needs to handle concurrent requests, serve static files, connect to databases, and expose a clean API, Axum is an excellent choice.

## What Is Axum? (A Restaurant Analogy)

Imagine you're opening a restaurant. You need:

- **A building** — a place for customers to walk into (that's your web server)
- **A menu** — a list of what you serve (that's your routes)
- **Waiters** — people who take orders and bring food (those are your handler functions)
- **A kitchen** — where the actual cooking happens (that's your database, your cache, your scraping logic)
- **A reservation system** — so multiple customers can eat at the same time (that's the async runtime)

**Axum** is the whole restaurant package. It gives you the building, the menus, and the waiters. You just need to write the kitchen logic — the parts that actually *do* something.

Unlike some web frameworks that try to do everything (cook the food, wash the dishes, AND park the cars), Axum is focused on one thing: routing HTTP requests to the right handler function. It's like a small, efficient restaurant where every waiter knows exactly what to do and never gets in each other's way.

Here's what makes Axum special:

1. **It's built on Tokio** — the async runtime we learned about in Part 1. This means it can handle thousands of simultaneous requests without breaking a sweat.
2. **It's type-safe** — if you try to extract a parameter that doesn't exist, your code won't compile. The compiler catches mistakes before they reach production.
3. **It's composable** — middleware, extractors, and handlers plug together like LEGO bricks.
4. **It's minimal** — Axum doesn't have its own template engine, ORM, or session management. It expects you to use the best library for each job (SQLx for databases, Tera for templates, etc.).

The Axum framework was created by the Tokio team — the same people who built the async runtime underneath it. This means it's deeply integrated with the async ecosystem, and it handles edge cases that other frameworks might miss.

## Cargo.toml: Adding Dependencies

Let's look at our `Cargo.toml` to see what goes into building this restaurant:

```toml
[package]
name = "fichub"
version = "0.1.0"
edition = "2024"
description = "Self-hosted fanfiction download server (fichub.net replacement)"

[dependencies]
# Web framework & runtime
axum = "0.8"
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.7", features = ["cors", "compression-gzip", "trace", "fs"] }

# Database
sqlx = { version = "0.9", features = ["runtime-tokio", "postgres", "migrate", "derive"] }

# Redis
redis = { version = "1.4", features = ["aio", "tokio-comp"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Config
dotenvy = "0.15"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
```

Let's break this down line by line:

| Dependency | What It Does | Restaurant Analogy |
|------------|-------------|-------------------|
| `axum` | Web framework — routes, handlers, state | The restaurant building and staff |
| `tokio` | Async runtime — lets Rust do many things at once | The energy that powers everything |
| `tower` | Middleware — layers that wrap around handlers | The security guard at the door |
| `tower-http` | HTTP-specific middleware (CORS, logging, static files) | Extra security and comfort features |
| `sqlx` | PostgreSQL database driver | The filing cabinet where we store recipes |
| `redis` | Redis client for caching | The memory foam mattress — instant recall |
| `serde` | Serialize/deserialize data structures | The universal translator |
| `serde_json` | JSON-specific serialization | The JSON translation module |
| `dotenvy` | Load `.env` files | The settings clipboard |
| `tracing` | Structured logging | The security camera system |
| `tracing-subscriber` | Formatting and filtering logs | The camera monitor |

The `features` part is important — it's like ordering specific dishes rather than the whole menu:

- **`tokio` with `features = ["full"]`** means "give me everything." This includes timers, TCP/UDP networking, file I/O, channels, and synchronization primitives. You need the full feature set when building a web server.

- **`sqlx` features** are more selective:
  - `"runtime-tokio"` — Use Tokio as the async runtime
  - `"postgres"` — Include the PostgreSQL driver (we could also use MySQL or SQLite)
  - `"migrate"` — Migration support for database schema changes
  - `"derive"` — The `#[derive(FromRow)]` macro that auto-maps database rows to structs

- **`tower-http` features** — Each feature adds specific middleware:
  - `"cors"` — Cross-Origin Resource Sharing headers
  - `"compression-gzip"` — Gzip compression for responses
  - `"trace"` — Request/response logging
  - `"fs"` — Static file serving

You might wonder why we have both `tower` and `tower-http`. Tower is the general middleware framework; tower-http adds HTTP-specific middleware on top of it. Think of Tower as the security system and tower-http as the specific cameras and locks.

## The Main Function: `#[tokio::main]`

Here's the entry point of our entire application — the `main.rs` file. Every Rust program starts here:

```rust
// src/main.rs
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod frontend;
pub mod limiter;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod tags;

#[tokio::main]
async fn main() {
    // Load .env if present
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();

    // Load configuration
    let config = config::Config::from_env();

    tracing::info!("Starting fichub-rs server on port {}", config.app_port);

    // Run the server
    server::run(config).await;
}
```

Let's walk through this step by step, because every line does something important.

**The `pub mod` lines** at the top are like doors in a hallway. Each one opens into a different room of our application:

```rust
pub mod db;        // The database room
pub mod config;    // The settings room
pub mod routes;    // The handler room (where API endpoints live)
pub mod error;     // The error room
pub mod server;    // The server room
pub mod cache;     // The caching room
pub mod scrape;    // The web scraping room
pub mod export;    // The file generation room
pub mod frontend;  // The frontend serving room
pub mod limiter;   // The rate limiting room
pub mod recommender; // The recommendation engine room
pub mod search;    // The search room
pub mod tags;      // The tagging system room
```

The `pub` means these modules are public — other parts of the code can use them. If you left off `pub`, only code inside `main.rs` could access them. By making them `pub`, the entire application can import from any module.

**The `#[tokio::main]` attribute** is where the magic happens. When you write `async fn main()`, Rust needs someone to manage the async operations. Tokio volunteers to be that manager. It sets up:

1. A **thread pool** — multiple threads that can run tasks simultaneously
2. An **event loop** — a mechanism that watches for things to happen (data arriving on a socket, a timer firing, a file finishing reading)
3. A **task scheduler** — decides which async tasks to run when

```rust
#[tokio::main]  // ← This macro transforms your main function
async fn main() {
    // Tokio is now in charge of managing async tasks
    // You can use .await freely here
}
```

Think of `#[tokio::main]` as hiring a restaurant manager. Without it, the waiters (async tasks) would have no one coordinating them — they'd all try to serve the same table at once, or stand around waiting for orders that never come.

**The `.ok()` on `dotenvy::dotenv()`** means "try to load the `.env` file, but don't panic if it's missing." In Rust, `.ok()` converts a `Result<T, E>` into an `Option<T>` — if it was `Ok(value)`, you get `Some(value)`; if it was `Err(e)`, you get `None`. By calling `.ok()` and then discarding the result, we say "I don't care if this fails."

This is important because in production, you might not have a `.env` file — environment variables might be set directly in Docker, systemd, or your hosting platform.

**The `tracing_subscriber` block** sets up our logging system. Think of it as installing the security cameras:

```rust
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
    )
    .init();
```

The `EnvFilter` reads the `RUST_LOG` environment variable to decide what to log. If `RUST_LOG` is not set, it defaults to `info,fichub=debug` — this means:
- Log messages at info level and above for everything
- Log messages at debug level and above for our `fichub` crate

If you wanted to see EVERYTHING (including debug messages from all libraries), you could set `RUST_LOG=debug` in your `.env` file. If you wanted to silence everything except errors, you'd set `RUST_LOG=error`.

**Finally**, `server::run(config).await` hands everything off to the server module. This is where the actual work begins. The `config` variable holds all our settings, and `.await` means "wait for the server to finish" (which in practice means "run forever").

🧪 **Try It Yourself**: Create a minimal `main.rs` that just prints "Hello from Tokio!" and runs a timer. Use `#[tokio::main]` and `tokio::time::sleep`. Example:

```rust
#[tokio::main]
async fn main() {
    println!("Hello from Tokio!");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    println!("One second later!");
}
```

Run it with `cargo run` and watch the delay. That delay is Tokio's async runtime in action!

## Creating a Router with Routes

Now let's look at the heart of our application — the router. The router is like a switchboard operator — it looks at each incoming request and figures out which handler function should deal with it.

Here's a simplified version first:

```rust
use axum::{routing::get, Router};

// A simple handler function
async fn hello() -> &'static str {
    "Hello, world!"
}

#[tokio::main]
async fn main() {
    // Create a router with one route
    let app = Router::new()
        .route("/", get(hello));

    // Bind to a port and serve
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

The `Router::new()` creates an empty restaurant. `.route("/", get(hello))` adds a menu item: "When someone visits `/`, run the `hello` function." The `get` part means this only works for GET requests — if someone sends a POST to `/`, they'll get a 405 "Method Not Allowed" response.

You can add multiple routes:

```rust
let app = Router::new()
    .route("/", get(hello))
    .route("/about", get(about_page))
    .route("/api/data", get(get_data).post(create_data));
```

The `.route("/api/data", get(get_data).post(create_data))` line shows how to handle multiple HTTP methods on the same path. GET requests go to `get_data`, POST requests go to `create_data`.

### Understanding the `json!()` Macro

The `json!()` macro is one of the most useful tools in `serde_json`. It lets you build JSON values using Rust-like syntax:

```rust
use serde_json::json;

// Simple values
let j = json!("hello");           // "hello"
let j = json!(42);                // 42
let j = json!(true);              // true
let j = json!(null);              // null

// Objects
let j = json!({
    "name": "Alice",
    "age": 30,
    "active": true
});
// {"name": "Alice", "age": 30, "active": true}

// Arrays
let j = json!([1, 2, 3]);
// [1, 2, 3]

// Nested
let j = json!({
    "user": {
        "name": "Bob",
        "scores": [100, 95, 87]
    }
});

// Interpolating Rust variables
let title = "My Story";
let wordcount = 5000;
let j = json!({
    "title": title,
    "wordcount": wordcount,
    "status": "Complete"
});
// {"title": "My Story", "wordcount": 5000, "status": "Complete"}
```

The `json!()` macro is powerful because it:
- Handles escaping automatically (strings with quotes, newlines, etc.)
- Works with any Rust value that implements `Serialize`
- Produces a `serde_json::Value` that Axum can send as a response
- Is checked at compile time for syntax errors

### The `Json<T>` Extractor and Response

When a handler returns `Json<Value>`, Axum:
1. Serializes the value to JSON
2. Sets the `Content-Type: application/json` header
3. Sends the JSON body

When a handler takes `Json<T>` as a parameter, Axum:
1. Reads the request body
2. Parses it as JSON
3. Deserializes it into type `T`
4. If parsing fails, returns a 400 error automatically

```rust
// Sending JSON
async fn handler() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

// Receiving JSON
async fn create_fic(
    Json(payload): Json<CreateFicRequest>,
) -> Json<Value> {
    // payload is already parsed and type-checked
    Json(json!({"id": payload.id, "created": true}))
}
```

## GET /api/v0/epub Handler Returning JSON

Now let's look at a real handler from our codebase — the epub handler. This is the handler that powers the main API endpoint. Before we read it, let's understand the `serde` derive macros that make it work:

### The `serde` Derive Macros

Serde is Rust's serialization framework. It converts Rust data structures to and from other formats (JSON, YAML, TOML, etc.). The `#[derive(Serialize, Deserialize)]` macros do the heavy lifting:

```rust
use serde::{Deserialize, Serialize};

// This struct can be converted TO JSON (Serialize)
// and FROM JSON (Deserialize)
#[derive(Debug, Serialize, Deserialize)]
struct User {
    name: String,
    age: u32,
    email: Option<String>,
}

// Serialize: Rust struct → JSON
let user = User {
    name: "Alice".to_string(),
    age: 30,
    email: None,
};
let json = serde_json::to_string(&user).unwrap();
// {"name":"Alice","age":30,"email":null}

// Deserialize: JSON → Rust struct
let json = r#"{"name":"Bob","age":25}"#;
let user: User = serde_json::from_str(json).unwrap();
// User { name: "Bob", age: 25, email: None }
```

The `Option<String>` for `email` means it can be `null` in JSON. If the JSON doesn't include `email`, it becomes `None` in Rust.

### Query Parameter Structs

When you use `Query(params): Query<ExportQuery>`, Axum deserializes the URL query string into your struct:

```rust
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub q: Option<String>,        // ?q=... → Some("...") or None
    pub automated: Option<String>, // ?automated=true → Some("true")
    pub format: Option<String>,    // ?format=epub → Some("epub")
}
```

Every field must be `Option<T>` for optional parameters. If someone visits `/api/v0/epub` without any query parameters, all fields are `None`. If they visit `?q=test&format=epub`, `q` is `Some("test")` and `format` is `Some("epub")`.

Axum handles the parsing automatically. If the query string is malformed (like `?q=hello&q=hello`), Axum returns a 400 error. You never have to parse query strings manually.

```rust
// src/routes/export.rs (simplified)

use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

/// Query parameters for export requests
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub q: Option<String>,      // The URL to look up
    pub automated: Option<String>, // Whether it's a bot
    pub format: Option<String>,    // Export format
}

/// Main export handler: GET /api/v0/epub?q=<url>
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    // Get the query parameter (URL to look up)
    let query = params.q.as_deref().unwrap_or("");

    if query.is_empty() {
        return Ok(Json(json!({
            "err": -1,
            "msg": "no query",
            "q": ""
        })));
    }

    // Find the right scraper for this URL
    let scraper = state.scraper_registry.find_scraper(query)
        .ok_or_else(|| AppError::BadRequest(
            -5,
            format!("unsupported URL: {}", query)
        ))?;

    // Look up metadata from the source site
    let meta = scraper.lookup(&state.http_client, query).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;

    // Return the metadata as JSON
    Ok(Json(json!({
        "id": meta.url_id,
        "title": meta.title,
        "author": meta.author,
        "wordcount": meta.words,
        "chapters": meta.chapters,
        "description": meta.desc,
    })))
}
```

Several important things to notice here:

**1. The function takes `State` as a parameter.** This is how Axum gives your handler access to shared data. The handler needs the database pool, the HTTP client, the config — all of that lives in `AppState`. We extract it with `State(state)`.

**2. The function takes `Query` as a parameter.** This tells Axum to parse the URL query string into our `ExportQuery` struct. If the parsing fails (like if someone sends `?q=hello&q=hello` with duplicate keys), Axum automatically returns a 400 error. You don't have to handle that case!

**3. The return type is `Result<Json<Value>, AppError>`.** This means the handler can either succeed (return JSON) or fail (return an AppError). The `?` operator handles the conversion automatically.

**4. `json!({...})`** is a macro from `serde_json` that creates JSON values. It's like building a dictionary, but it produces valid JSON. The syntax looks like Rust's struct initialization but produces a `serde_json::Value` at runtime.

**5. The `as_deref().unwrap_or("")` pattern.** `params.q` is an `Option<String>`. `as_deref()` converts it to `Option<&str>` (a reference to the string, without copying). `unwrap_or("")` provides a default empty string. This is a common Rust pattern for handling optional strings.

## Axum Extractors: The Magic Behind Handler Parameters

Before we dive into path and query parameters, let's understand **extractors** — the mechanism that makes Axum so elegant. An extractor is any type that implements the `FromRequestParts` or `FromRequest` trait. When you add a parameter to a handler, Axum tries to extract it from the request.

```rust
async fn my_handler(
    State(state): State<Arc<AppState>>,      // Extractor 1: shared state
    Query(params): Query<MyQuery>,            // Extractor 2: query string
    Path(id): Path<String>,                   // Extractor 3: path parameter
    Json(body): Json<MyBody>,                 // Extractor 4: request body
) -> impl IntoResponse {
    // All four values are extracted automatically
}
```

Axum processes extractors in order. If any extractor fails (bad query string, missing path segment, invalid JSON), the handler is never called — Axum returns an error response instead. This means your handler code can assume all data is valid.

The most common extractors:

| Extractor | What It Extracts | Example |
|-----------|-----------------|---------|
| `State(s)` | Shared application state | Database pool, config |
| `Query(q)` | URL query parameters | `?key=value` |
| `Path(p)` | URL path segments | `/users/{id}` |
| `Json(body)` | JSON request body | POST data |
| `ConnectInfo(addr)` | Client's IP address | Remote socket address |

You can use multiple extractors in any order — Axum figures out how to combine them. The only rule is that `Json<T>` must be the last extractor because it consumes the request body.

🧪 **Try It Yourself**: Write a handler that uses three extractors: `State`, `Query`, and `Path`. What happens if you send a request with a malformed query string? What if the path doesn't match?

## Path Parameters and Query Parameters

Axum has two main ways to get data from URLs:

### Query Parameters

These come after the `?` in a URL:
```
GET /api/v0/epub?q=https://archiveofourown.org/works/12345
```

You define a struct with `#[derive(Deserialize)]` and use `Query(params)`:

```rust
use serde::Deserialize;
use axum::extract::Query;

#[derive(Deserialize)]
pub struct ExportQuery {
    pub q: Option<String>,    // "q" from the URL
    pub format: Option<String>, // "format" from the URL
}

pub async fn epub_handler(
    Query(params): Query<ExportQuery>,
) -> ... {
    let url = params.q.unwrap_or_default();
    let format = params.format.unwrap_or_else(|| "epub".to_string());
}
```

Every field in the struct must be `Option<T>` if it might not be present in the URL. If someone visits `/api/v0/epub` without any query parameters, `params.q` will be `None`.

You can also make fields required by not using `Option`:

```rust
#[derive(Deserialize)]
pub struct RequiredQuery {
    pub q: String,  // REQUIRED — missing = 400 error
}
```

### Path Parameters

These are parts of the URL path itself:
```
GET /cache/epub/abc123/myfile.epub
```

You define them with `{name}` in the route and use `Path(...)`:

```rust
use axum::extract::Path;

pub async fn download_handler(
    Path((etype, url_id, fname)): Path<(String, String, String)>,
) -> ... {
    // etype = "epub", url_id = "abc123", fname = "myfile.epub"
}
```

The type annotation `Path<(String, String, String)>` tells Axum "I expect three path segments, and they should all be strings." If someone visits `/cache/epub/abc123` with only two segments, they get a 404.

You can also use structs for path parameters:

```rust
#[derive(Deserialize)]
struct DownloadParams {
    etype: String,
    url_id: String,
    fname: String,
}

async fn download_handler(
    Path(params): Path<DownloadParams>,
) -> ... {
    // params.etype, params.url_id, params.fname
}
```

The tuple version is more concise; the struct version is more readable. Pick whichever makes sense for your use case.

## State: Sharing Data Across Routes

In a real application, many routes need access to the same resources — the database pool, the HTTP client, the config. You don't want to create a new database connection for every request. That would be like hiring a new waiter for every customer — expensive and slow!

This is where **State** comes in. State is a single value that all handlers share:

```rust
use std::sync::Arc;

// Define what state looks like
struct AppState {
    db: sqlx::PgPool,           // Database connection pool
    redis: redis::aio::MultiplexedConnection,  // Redis connection
    http_client: reqwest::Client,               // HTTP client
    config: Config,                             // All settings
}

#[tokio::main]
async fn main() {
    // Create the state ONCE, at startup
    let state = Arc::new(AppState {
        db: db_pool,
        redis: redis_conn,
        http_client,
        config,
    });

    // Give it to the router
    let app = Router::new()
        .route("/api/v0/epub", get(epub_handler))
        .with_state(state);  // ← State is attached here!

    // Every handler can now access it via State(state)
}
```

The `Arc` (Atomic Reference Count) is Rust's way of sharing ownership safely. Multiple handlers can all have a reference to the same `AppState` without worrying about who "owns" it. When the last reference is dropped, the state is cleaned up. Think of it as giving every waiter a key to the same kitchen — they can all use it, and when the restaurant closes, the key is returned.

The `Arc` is needed because Axum requires state to be cloneable and shareable across threads. Without it, you'd have ownership conflicts — Rust's borrow checker would prevent you from passing the same state to multiple handlers.

## The AppState Struct

Here's our actual `AppState` from `server.rs`:

```rust
/// Shared application state accessible by all handlers
pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    pub http_client: reqwest::Client,
    pub scraper_registry: Arc<ScraperRegistry>,
    pub cache_semaphores: CacheSemaphores,
    pub rate_limiter: Box<dyn limiter::RateLimiter>,
    pub recommender_engine: RecommendationEngine,
    pub collection_worker: CollectionWorker,
}
```

This is the "kitchen" of our restaurant. Every piece of state is something that multiple handlers might need:

| Field | Type | Purpose | Created Once |
|-------|------|---------|-------------|
| `config` | `Config` | All our settings (port, database URL, etc.) | ✓ |
| `db` | `sqlx::PgPool` | The database connection pool | ✓ |
| `redis` | `MultiplexedConnection` | Redis connection for caching | ✓ |
| `http_client` | `reqwest::Client` | For fetching web pages | ✓ |
| `scraper_registry` | `Arc<ScraperRegistry>` | Which sites we know how to scrape | ✓ |
| `cache_semaphores` | `CacheSemaphores` | Prevents duplicate exports | ✓ |
| `rate_limiter` | `Box<dyn RateLimiter>` | Limits how fast users can hit us | ✓ |
| `recommender_engine` | `RecommendationEngine` | The recommendation system | ✓ |
| `collection_worker` | `CollectionWorker` | Background data collection | ✓ |

The "Created Once" column is key — all of these are created at startup and shared across all requests. Creating them on every request would be incredibly wasteful.

The `Box<dyn limiter::RateLimiter>` is a **trait object** — it means the rate limiter can be any type that implements the `RateLimiter` trait. This lets us swap implementations (like switching from a Redis-based limiter to an in-memory one for testing) without changing the rest of the code.

## Running the Server and Testing with curl

Let's see how the server starts up. The `run()` function in `server.rs` does all the heavy lifting:

```rust
pub async fn run(config: Config) {
    // 1. Connect to PostgreSQL
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // 2. Connect to Redis
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");

    // 3. Build HTTP client
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");

    // 4. Create shared state
    let state = Arc::new(AppState { /* ... */ });

    // 5. Build router
    let app = build_router(state).await;

    // 6. Bind and serve
    let addr = format!("0.0.0.0:{}", config.app_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app).await.expect("Server error");
}
```

Notice the order: database first, then Redis, then HTTP client, then state, then router, then serve. Each step can fail, and we use `.expect()` to crash with a clear message if anything goes wrong. In production, you might want to handle these errors more gracefully (retry, log, send alerts), but for development, crashing loudly is the right choice — it tells you immediately what's broken.

The `reqwest::Client::builder()` creates a reusable HTTP client with custom settings. The `.user_agent("fichub.net/0.1.0")` sets a custom User-Agent header so web servers know who's making the request. The `.timeout(Duration::from_secs(30))` means we give up on any HTTP request that takes longer than 30 seconds.

Now let's test it! Start your server in one terminal:

```bash
cd ~/code/rust/fichub
cargo run
```

You should see output like:
```
INFO Starting fichub-rs server on port 3000
INFO Listening on 0.0.0.0:3000
```

In another terminal, use `curl` to talk to it:

```bash
# See the API docs (self-documenting!)
curl http://localhost:3000/api/

# Get metadata for a fanfic
curl "http://localhost:3000/api/v0/meta?q=https://archiveofourown.org/works/12345"

# Check remote info (who am I?)
curl http://localhost:3000/api/v0/remote

# Search for fanfics
curl "http://localhost:3000/api/v0/search?q=harry+potter"

# Try an error — empty query
curl "http://localhost:3000/api/v0/epub?q="
```

The `curl` command is your best friend for testing APIs. It sends HTTP requests from the command line and shows you the response. The `-v` flag (verbose) shows you all the headers and details:

```bash
curl -v "http://localhost:3000/api/v0/meta?q=test"
```

🧪 **Try It Yourself**: Start the server and hit each endpoint with curl. What do the JSON responses look like? Can you make the server return an error by sending an empty query? What happens if you try to access a route that doesn't exist?

## What Happens When a Request Arrives (The Flow)

Let's trace exactly what happens when someone visits `GET /api/v0/meta?q=some-url`. This is important to understand because it shows how all the pieces fit together:

```
Step 1: Browser sends: GET /api/v0/meta?q=some-url HTTP/1.1
        ↓
Step 2: Tokio receives the TCP connection on port 3000
        ↓
Step 3: Axum parses the HTTP request into parts:
        - Method: GET
        - Path: /api/v0/meta
        - Query: q=some-url
        ↓
Step 4: Router matches: /api/v0/meta → meta_handler
        ↓
Step 5: TraceLayer logs: "GET /api/v0/meta → 200 OK (45ms)"
        ↓
Step 6: Axum extracts from the request:
        - State(state) → from the router's shared state
        - Query(params) → parsed from "?q=some-url"
        ↓
Step 7: Axum calls meta_handler(state, params)
        ↓
Step 8: Handler uses state.http_client to fetch the web page
        (this is an async operation — other requests can proceed)
        ↓
Step 9: Handler uses state.db to store/retrieve data
        (another async operation)
        ↓
Step 10: Handler returns Ok(Json({...}))
        ↓
Step 11: Axum serializes the JSON and sends HTTP 200 OK
         Content-Type: application/json
        ↓
Step 12: Browser receives the response and renders it
```

This whole journey takes about 50-200 milliseconds, depending on network speed and database performance. And because of Tokio's async runtime, the server can handle thousands of these simultaneously — while one handler waits for a database query, other handlers are free to do their work.

The key insight is that **async doesn't mean parallel**. It means the server can do other work while waiting. When handler A calls `.await` on a database query, Tokio pauses handler A and runs handler B. When the database responds, Tokio resumes handler A. It's like a chef who starts cooking one dish, puts it in the oven, and while it bakes, starts preparing the next dish.

⚠️ **Watch Out**: If your handler blocks (does slow synchronous work), it blocks all other handlers too. This is called "blocking the runtime." Always use `.await` for async operations and avoid `.unwrap()` on network calls that might hang. If you need to do CPU-intensive work, use `tokio::task::spawn_blocking` to move it to a separate thread.

## Building a Complete Simple Server

Let's put everything together into a complete, runnable example. This isn't our full FicHub server — it's a simplified version that demonstrates all the key concepts:

```rust
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

// Our shared state
struct AppState {
    visitor_count: std::sync::atomic::AtomicU64,
}

// A query parameter struct
#[derive(Deserialize)]
struct GreetQuery {
    name: Option<String>,
}

// Handler 1: Root route
async fn index() -> Json<Value> {
    Json(json!({
        "message": "Welcome to our server!",
        "endpoints": ["/greet", "/hello/{name}", "/count"]
    }))
}

// Handler 2: Greet with query parameter
async fn greet(
    Query(params): Query<GreetQuery>,
) -> Json<Value> {
    let name = params.name.unwrap_or_else(|| "stranger".to_string());
    Json(json!({
        "message": format!("Hello, {}!", name)
    }))
}

// Handler 3: Greet with path parameter
async fn hello_name(
    Path(name): Path<String>,
) -> Json<Value> {
    Json(json!({
        "message": format!("Hello, {}!", name)
    }))
}

// Handler 4: Uses shared state
async fn count(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let count = state.visitor_count
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Json(json!({
        "visitor_number": count + 1,
        "message": format!("You are visitor #{}!", count + 1)
    }))
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        visitor_count: std::sync::atomic::AtomicU64::new(0),
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/greet", get(greet))
        .route("/hello/{name}", get(hello_name))
        .route("/count", get(count))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
```

Try running this and testing all the endpoints:

```bash
# Welcome message
curl http://localhost:3000/

# Greet with default name
curl http://localhost:3000/greet

# Greet with query parameter
curl "http://localhost:3000/greet?name=Alice"

# Greet with path parameter
curl http://localhost:3000/hello/Bob

# Visitor counter (run multiple times!)
curl http://localhost:3000/count
curl http://localhost:3000/count
curl http://localhost:3000/count
```

The visitor counter demonstrates shared state — the `AtomicU64` counter persists across requests because it lives in `AppState`, which is shared via `Arc`. Without state, each request would start with a fresh counter.

🧪 **Try It Yourself**: Modify the server above to add a `/time` endpoint that returns the current time as JSON. Hint: use `chrono::Utc::now()`.

---

# Chapter 6: Configuration and Error Handling

## Environment Variables: Settings for Your App

Every application needs settings. Where's the database? What port should we listen on? What's the Redis URL? These are things that change between your laptop, your test server, and your production server.

You *could* hardcode them:

```rust
let database_url = "postgres://user:password@localhost:5432/fichub";
```

But that's terrible for three reasons:

1. **You'd have to change the source code** every time you move to a different server
2. **You'd accidentally commit your password** to git (and everyone would see it)
3. **You can't have different settings** for development, testing, and production

Instead, we use **environment variables** — settings that live outside your code, in the environment where the program runs:

```rust
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");
```

This reads the `DATABASE_URL` environment variable. If it's not set, the program crashes with a clear error message: "DATABASE_URL must be set."

Environment variables are key-value pairs in your shell:

```bash
# Linux/Mac
export DATABASE_URL=postgres://user:password@localhost:5432/fichub

# Windows
set DATABASE_URL=postgres://user:password@localhost:5432/fichub
```

They're set before running your program and last until the terminal closes. Different terminals can have different values.

## The .env File: Keeping Secrets Safe

Setting environment variables every time you run the server is annoying. Imagine typing this every morning:

```bash
export DATABASE_URL=postgres://...
export REDIS_URL=redis://...
export CACHE_DIR=./cache
export PORT=3000
# ... 20 more variables ...
cargo run
```

That's where `.env` files come in. Create a file called `.env` in your project root:

```bash
# .env file in the project root
DATABASE_URL=postgres://user:password@localhost:5432/fichub
REDIS_URL=redis://localhost:6379
CACHE_DIR=./cache
PORT=3000
```

We load it at the very start of `main.rs`:

```rust
dotenvy::dotenv().ok();
```

The `dotenvy` crate reads the `.env` file and sets each variable in the environment — as if you had typed `export` for each one. The `.ok()` means "don't panic if the file doesn't exist."

Now when you run `cargo run`, the `.env` file is automatically loaded. No typing required!

⚠️ **Watch Out**: Never commit your `.env` file to git! It contains passwords and secrets. Add it to `.gitignore`:

```bash
echo ".env" >> .gitignore
```

In production, you'd set these variables directly (in Docker, systemd, or your hosting platform) rather than using a `.env` file. The `.env` file is just for development convenience.

You can also have different `.env` files for different environments:

```
.env              # Default (used when no other file is specified)
.env.development  # Development settings
.env.production   # Production settings
.env.testing      # Test settings
```

## Config Struct: Loading All Settings at Once

Instead of scattering `std::env::var()` calls throughout your code (which makes it hard to know what settings exist and what they do), collect everything into a single `Config` struct.

Here's our actual `Config` from `config.rs`:

```rust
use std::collections::HashMap;
use std::path::PathBuf;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub secondary_cache_dir: Option<PathBuf>,
    pub export_version: i32,
    pub dynamic_rate_limit: bool,
    pub node_name: String,
    pub calibre_container: String,
    pub tmp_dir: PathBuf,
    pub app_port: u16,
    pub frontend_dir: PathBuf,
    pub trusted_proxies: Vec<String>,
    pub ip_tag_sources: Vec<(String, String, String)>,
    // Recommender settings
    pub rec_default_delay_secs: u64,
    pub rec_max_favourite_pages: u32,
    pub rec_max_recommendations: usize,
    pub rec_cache_ttl_hours: u32,
    pub rec_suggest_limit_per_hour: u32,
    pub rec_vote_limit_per_hour: u32,
    pub rec_precompute_enabled: bool,
    // ... more fields ...
}
```

The `#[derive(Debug, Clone)]` is two derive macros in one line:
- `Debug` lets you print the struct with `{:?}` for debugging — essential when things go wrong
- `Clone` lets you copy the struct (Config is cheap to clone since it's mostly strings and numbers)

Notice the variety of types:
- `String` — for text values like URLs
- `PathBuf` — for file paths (like `./cache`)
- `i32`, `u16`, `u32`, `u64` — for numbers of different sizes
- `bool` — for true/false settings
- `Vec<String>` — for lists (like trusted proxy IPs)
- `Option<PathBuf>` — for optional settings
- `HashMap<String, u64>` — for key-value maps (like per-site rate limits)

Each type has a purpose. `PathBuf` is better than `String` for file paths because it handles OS-specific separators automatically. `u16` is the right size for a port number (0-65535).

## Config::from_env() Function Explained

The magic happens in the `from_env()` method. Let's read it carefully:

```rust
impl Config {
    pub fn from_env() -> Self {
        // REQUIRED: crash immediately if these are missing
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");

        // OPTIONAL with defaults: use fallback if missing
        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());

        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);

        let node_name = std::env::var("NODE_NAME")
            .unwrap_or_else(|_| "orion".to_string());

        // OPTIONAL: might not be set
        let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);

        // ... load everything else ...

        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            app_port,
            node_name,
            secondary_cache_dir,
            // ... other fields ...
        }
    }
}
```

There are three patterns here, and they're important:

**Pattern 1: `expect()` — Required variables**
```rust
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");
```
If `DATABASE_URL` isn't set, this crashes with a clear message. Use this for things the app can't function without. The server would crash immediately on startup — and that's the right behavior! It's better to crash loudly than to silently fail later.

**Pattern 2: `unwrap_or_else()` — Optional with defaults**
```rust
let cache_dir = std::env::var("CACHE_DIR")
    .unwrap_or_else(|_| "./cache".to_string());
```
If `CACHE_DIR` isn't set, use `"./cache"`. The server continues normally. Use this for settings that have reasonable defaults.

**Pattern 3: Chain with `.parse()` — String to number conversion**
```rust
let app_port = std::env::var("PORT")
    .unwrap_or_else(|_| "3000".to_string())  // String
    .parse()                                   // Result<u16>
    .unwrap_or(3000);                          // u16
```
Let's trace through this chain:
```
std::env::var("PORT")                    → Result<String, VarError>
    .unwrap_or_else(|_| "3000".to_string()) → String (either from env or "3000")
    .parse()                               → Result<u16, ParseIntError>
    .unwrap_or(3000)                        → u16 (either parsed or 3000)
```

This chain is **safe** — it can never fail. If `PORT` isn't set, or if it's set to something that isn't a valid port number (like "banana"), we get 3000. This is defensive programming at its finest.

**Pattern 4: Optional with `.ok()` and `.filter()`**
```rust
let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
    .ok()                                    // Option<String>
    .filter(|s| !s.is_empty())              // Option<String> (None if empty)
    .map(PathBuf::from);                     // Option<PathBuf>
```
The `.ok()` converts `Result` to `Option`. The `.filter()` removes empty strings. The `.map()` converts the type. This chain handles three cases:
- Variable not set → `None`
- Variable set to empty string → `None`
- Variable set to a real path → `Some(PathBuf)`

🧪 **Try It Yourself**: Create a `.env` file with `DATABASE_URL`, `REDIS_URL`, and `PORT=8080`. Run the server and verify it starts on port 8080. Then change the port to `banana` and see what happens (it should default to 3000).

## Making Nice Error Pages

When something goes wrong, we don't want to dump a stack trace on the user. We don't want to show them our database password or the internal structure of our code. We want a nice, structured JSON error that our frontend can display nicely.

Here's what a successful response looks like:
```json
{
  "id": "12345",
  "title": "A Great Fanfic",
  "author": "Some Author",
  "wordcount": 15000
}
```

And here's what an error looks like:
```json
{
  "err": -5,
  "msg": "unsupported URL: not-a-real-site.com"
}
```

The `err` field is always a negative number that tells the frontend what kind of error occurred. The `msg` field is a human-readable description. This consistent format means our frontend can always handle errors the same way.

Why negative numbers? Because positive numbers could be confused with HTTP status codes (200, 404, 500, etc.). By using negatives, we make it clear these are application-specific codes, not HTTP codes.

## The AppError Enum: Different Kinds of Errors

Our `error.rs` file defines every possible error our app can produce:

```rust
/// Application-wide error type
#[derive(Debug)]
pub enum AppError {
    /// Bad request with error code and message
    BadRequest(i32, String),
    /// Rate limited - wait N seconds
    RateLimited(u64),
    /// Resource not found
    NotFound(String),
    /// Internal server error
    Internal(String),
    /// Scraper error
    ScrapeError(String),
    /// Export/generation error
    ExportError(String),
    /// Database error
    Database(String),
    /// Cache error
    CacheError(String),
}
```

Each variant represents a different *category* of failure. Let's use our restaurant analogy:

| Variant | HTTP Status | Meaning | Restaurant Analogy |
|---------|-------------|---------|-------------------|
| `BadRequest(code, msg)` | 400 | The user sent bad data | Customer ordered something not on the menu |
| `RateLimited(secs)` | 429 | Too many requests, wait N seconds | Customer tried to order 100 meals at once |
| `NotFound(msg)` | 404 | The resource doesn't exist | Customer asked for a dish we ran out of |
| `Internal(msg)` | 500 | Something broke on our end | The oven exploded |
| `ScrapeError(msg)` | 502 | Couldn't fetch from external site | The supplier didn't deliver |
| `ExportError(msg)` | 500 | Couldn't generate the file | The chef burned the food |
| `Database(msg)` | 500 | Database problem | The filing cabinet jammed |
| `CacheError(msg)` | 500 | Redis problem | The memory foam mattress deflated |

The `Display` implementation makes errors printable:

```rust
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::BadRequest(code, msg) => write!(f, "BadRequest({}): {}", code, msg),
            AppError::RateLimited(retry_after) => write!(f, "RateLimited: retry after {}s", retry_after),
            AppError::NotFound(msg) => write!(f, "NotFound: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal: {}", msg),
            // ... more variants ...
        }
    }
}
```

This is used when logging errors — the tracing system calls `Display` to get a human-readable message.

## IntoResponse: Converting Errors to JSON

The magic that makes errors into nice JSON is the `IntoResponse` implementation. Axum knows that if a handler returns a type that implements `IntoResponse`, it should call `into_response()` to get the HTTP response:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::BadRequest(code, msg) => {
                (StatusCode::BAD_REQUEST, json!({"err": code, "msg": msg}))
            }
            AppError::RateLimited(retry_after) => {
                (StatusCode::TOO_MANY_REQUESTS, json!({
                    "err": -429,
                    "msg": "rate limited",
                    "retry_after": retry_after
                }))
            }
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, json!({"err": -5, "msg": msg}))
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -1,
                    "msg": "internal server error"
                }))
            }
            AppError::ScrapeError(msg) => {
                (StatusCode::BAD_GATEWAY, json!({"err": -6, "msg": msg}))
            }
            AppError::ExportError(msg) => {
                tracing::error!("Export error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -5,
                    "msg": "export failed"
                }))
            }
            AppError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -1,
                    "msg": "database error"
                }))
            }
            AppError::CacheError(msg) => {
                tracing::error!("Cache error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -1,
                    "msg": "cache error"
                }))
            }
        };

        (status, Json(body)).into_response()
    }
}
```

Each error variant maps to:
1. An **HTTP status code** (400, 404, 429, 500, 502) — tells the browser what happened
2. A **JSON body** with `err` and `msg` — tells our frontend how to display it

The `tracing::error!` call on `Internal` errors is crucial — it logs the *real* error message to our server logs (for debugging) but sends a *generic* message to the user (for security). You never want to leak internal details like "connection refused at 127.0.0.1:5432" to users.

## Error Codes: What -1, -5, -6, -7, -429 Mean

Our error codes are negative numbers that the frontend can use to show the right message:

| Code | HTTP Status | Meaning | What the Frontend Does |
|------|-------------|---------|----------------------|
| `-1` | 500 | Internal error | "Something went wrong on our end. Try again later." |
| `-5` | 404 / 400 | Not found or bad request | "We couldn't find that fanfic. Check the URL?" |
| `-6` | 502 | Scraper error | "We couldn't reach the source site. It might be down." |
| `-7` | 500 | Export error | "We couldn't create your download. Try again?" |
| `-10` | 400 | Automated request blocked | "Automated requests are not allowed." |
| `-429` | 429 | Rate limited | "Please slow down. Try again in {N} seconds." |
| `400` | 400 | Custom bad request | Varies by message |

The frontend can switch on these codes to show the right UI:
```javascript
if (response.err === -429) {
    showRateLimitMessage(response.retry_after);
} else if (response.err === -5) {
    showNotFoundMessage();
} else {
    showGenericError();
}
```

## Error Recovery Patterns

Sometimes you don't want an error to crash the whole request — you want to recover gracefully. Here are some patterns:

### Pattern 1: Log and Continue

```rust
// Try to log the request, but don't fail if logging breaks
if let Err(e) = queries::insert_request_log(&pool, ...).await {
    tracing::warn!("Failed to log request: {}", e);
    // Continue anyway — logging failure shouldn't break the API
}
```

The `if let Err(e)` pattern means "if this fails, do something with the error but don't propagate it."

### Pattern 2: Fallback Values

```rust
// Try to get from cache, fall back to database
let fic = cache::get(&redis, &url_id).await
    .unwrap_or_else(|_| {
        tracing::debug!("Cache miss for {}", url_id);
        None
    })
    .or_else(|| {
        // Try database as fallback
        block_on(queries::get_fic_info(&pool, &url_id)).ok().flatten()
    });
```

This tries Redis first, then the database. If both fail, `fic` is `None`.

### Pattern 3: Partial Results

```rust
// Collect results, ignoring failures
let mut results = Vec::new();
for url in urls {
    match queries::get_fic_info(&pool, &url).await {
        Ok(Some(fic)) => results.push(fic),
        Ok(None) => tracing::debug!("Not found: {}", url),
        Err(e) => tracing::warn!("Error fetching {}: {}", url, e),
    }
}
// Return whatever we got
Ok(results)
```

This collects successful results and skips failures, rather than failing the entire batch.

### Pattern 4: Retry on Transient Errors

```rust
use std::time::Duration;

async fn retry_query(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let mut last_err = None;

    for attempt in 0..3 {
        match queries::get_fic_info(pool, id).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                tracing::warn!("Attempt {} failed: {}", attempt + 1, e);
                last_err = Some(e);
                tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
            }
        }
    }

    Err(last_err.unwrap())
}
```

This retries up to 3 times with increasing delays (100ms, 200ms, 300ms). Useful for transient network issues.

## The From Trait: Automatic Error Conversion

One of Rust's most powerful features is automatic error conversion with the `From` trait. This is what makes the `?` operator so convenient:

```rust
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::CacheError(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::ScrapeError(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}
```

This means anywhere you use `?` in a function that returns `AppResult<T>`, the error is automatically converted:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)  // Returns Result<..., sqlx::Error>
    .await?;               // ← ? automatically converts sqlx::Error to AppError
    Ok(row)
}
```

Without the `From` implementations, you'd have to write this everywhere:
```rust
.await.map_err(|e| AppError::Database(e.to_string()))?;
```

With `From`, just `?` works. It's like having a universal adapter — any error type plugs into our `AppError` automatically. This is one of Rust's best features for error handling.

There's also a type alias that makes function signatures cleaner:

```rust
/// Standard API result type
pub type AppResult<T> = Result<T, AppError>;
```

Now instead of writing `Result<Option<FicInfo>, AppError>`, you write `AppResult<Option<FicInfo>>`. Shorter, clearer, and easier to read.

---

# Chapter 7: Connecting to PostgreSQL

## The Database URL

Before we connect to PostgreSQL, let's understand the connection URL format:

```
postgres://username:password@hostname:port/database_name
```

For example:
```
postgres://fichub:secretpass@localhost:5432/fichub
```

This tells SQLx:
- **Username**: `fichub`
- **Password**: `secretpass`
- **Host**: `localhost` (the same machine)
- **Port**: `5432` (PostgreSQL's default port)
- **Database**: `fichub` (the specific database to use)

You can also use environment variables in the URL:
```
postgres://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${DB_NAME}
```

The URL is stored in your `.env` file and read by `Config::from_env()`.

## What Is a Database? (A Giant Filing Cabinet)

Imagine you have a massive filing cabinet with thousands of folders. Each folder contains information about one fanfic — title, author, word count, when it was published. When you want to find a specific fanfic, you open the drawer, flip through the folders, and pull out the one you need.

A **database** is exactly that, except:
- It lives on a computer, not in a room
- It can hold millions of records
- It can find things in milliseconds (not hours of flipping)
- Multiple people can use it at the same time
- It keeps backups so nothing gets lost
- You can ask complex questions like "show me all fics with more than 10,000 words by authors whose names start with 'A'"

The filing cabinet we're using is called **PostgreSQL** (often shortened to "Postgres"). It's been around since 1996 and is one of the most reliable, feature-rich databases in the world. It's used by Apple, Instagram, Spotify, and thousands of other companies.

PostgreSQL speaks **SQL** (Structured Query Language) — a language specifically designed for working with databases. Here's a simple SQL query:

```sql
SELECT title, author FROM fic_info WHERE words > 10000;
```

This says "go to the fic_info table, find all rows where the words column is greater than 10,000, and return just the title and author columns."

## SQLx: A Rust Library for Talking to PostgreSQL

**SQLx** is how Rust talks to PostgreSQL. It's like a translator who speaks both "Rust" and "SQL" and can carry messages between them.

What makes SQLx special:

1. **Compile-time checked queries** — if your SQL has errors, Rust catches them when you compile. Not at runtime when a user hits the endpoint. At *compile time*, before you even deploy.

2. **Async** — it doesn't block while waiting for the database. Your server keeps handling other requests while SQLx waits for PostgreSQL to respond.

3. **Type-safe** — database rows automatically map to Rust structs. You get the benefits of Rust's type system even for database operations.

4. **Connection pooling** — built-in support for sharing connections efficiently.

We added it to our `Cargo.toml`:
```toml
sqlx = { version = "0.9", features = [
    "runtime-tokio",  # Use Tokio as the async runtime
    "postgres",       # PostgreSQL driver (could also be "mysql" or "sqlite")
    "migrate",        # Migration support
    "derive",         # Derive macros (FromRow for automatic row-to-struct mapping)
    "macros",         # Compile-time checked SQL
    "tls-rustls-ring", # TLS support for encrypted connections
]}
```

## Connection Pools: Sharing One Connection Among Many Users

Here's a problem: if you create a new database connection for every request, and you get 1,000 requests per second, you'd need 1,000 connections. PostgreSQL can handle that, but it's wasteful — each connection uses memory and takes time to set up (about 20-50 milliseconds).

Instead, we use a **connection pool** — a shared collection of connections. When a request comes in, we borrow a connection from the pool. When it's done, we return it. It's like a library checkout system — there are 20 books (connections), and 100 people (requests) take turns reading them.

Here's how we create our pool in `db/mod.rs`:

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::path::Path;
use std::time::Duration;

/// Initialize the database connection pool and run migrations
pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)                        // Up to 20 connections
        .acquire_timeout(Duration::from_secs(10))   // Wait up to 10 seconds
        .connect(database_url)                      // Connect to PostgreSQL
        .await?;

    // Run migrations from the migrations directory
    let migrations_path = if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            exe_dir.join("migrations")
        } else {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")
        }
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")
    };

    if migrations_path.exists() {
        sqlx::migrate::Migrator::new(migrations_path)
            .await?
            .run(&pool)
            .await?;
        tracing::info!("Database migrations applied");
    } else {
        tracing::warn!("Migrations directory not found at {:?}", migrations_path);
    }

    Ok(pool)
}
```

Let's break down the pool configuration:

- **`max_connections(20)`** — We allow up to 20 simultaneous database connections. This is like having 20 clerks at the filing cabinet. More clerks = more people served simultaneously, but each clerk uses resources. For a typical web app, 10-20 is a good number.

- **`acquire_timeout(Duration::from_secs(10))`** — If all 20 connections are busy, wait up to 10 seconds for one to become available. If it takes longer, fail with an error. This prevents requests from hanging forever if the database is overloaded.

- **`connect(database_url)`** — Actually connect to PostgreSQL using the URL. The URL format is: `postgres://username:password@host:port/database_name`

The `PgPool` type is the pool itself. It's designed to be shared — you create it once and pass it to every handler. In our `AppState`, it's the `db` field. Handlers access it like this:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    // SQLx automatically borrows a connection from the pool
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)  // ← pool is passed here
    .await?;
    Ok(row)
}
```

You never manually open or close connections. SQLx handles it all through the pool.

🧪 **Try It Yourself**: What happens if you set `max_connections(1)` and try to handle two requests at the same time? Try it — start the server, and in two terminal windows, run `curl` commands simultaneously. The second request should wait until the first one finishes.

## Creating Tables with CREATE TABLE

PostgreSQL stores data in **tables** — like spreadsheets with rows and columns. Each table has a specific structure (which columns exist and what type of data they hold).

You create tables with SQL:

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,                    -- Auto-incrementing ID
    name TEXT NOT NULL,                       -- Text that can't be empty
    email TEXT UNIQUE NOT NULL,               -- Text that must be unique
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP  -- When it was created
);
```

Let's break down each part:

- **`CREATE TABLE users`** — "Create a new table called 'users'"
- **`id SERIAL PRIMARY KEY`** — An auto-incrementing integer that uniquely identifies each row. The first row gets id=1, the second gets id=2, etc. `PRIMARY KEY` means this column must be unique and non-null.
- **`name TEXT NOT NULL`** — A text column that cannot be empty
- **`email TEXT UNIQUE NOT NULL`** — A text column that must be unique across all rows AND cannot be empty
- **`created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP`** — A timestamp that automatically fills in with the current time when a row is inserted

Here are the SQL types you'll encounter most often:

| SQL Type | Rust Type | What It Means | Example |
|----------|-----------|---------------|---------|
| `SERIAL` | `i32` | Auto-incrementing integer | 1, 2, 3, ... |
| `BIGSERIAL` | `i64` | Like SERIAL but bigger | For tables with millions of rows |
| `INT4` / `INT8` | `i32` / `i64` | Regular integer | Fixed-size numbers |
| `SMALLINT` | `i16` | Small integer | For values 0-32767 |
| `TEXT` | `String` | Any text, any length | Titles, descriptions |
| `VARCHAR(128)` | `String` | Text with max length | URLs, IDs |
| `BOOLEAN` | `bool` | True or false | is_automated |
| `TIMESTAMPTZ` | `DateTime<Utc>` | Date + time + timezone | created, updated |
| `REAL` | `f32` | Floating-point number | Scores, ratings |
| `INET` | `IpAddr` | An IP address | 192.168.1.1 |

The difference between `INT4` and `INT8` matters. `INT4` (4 bytes) stores numbers up to about 2 billion. `INT8` (8 bytes) stores numbers up to about 9 quintillion. For a word count, `INT8` is safe — some fanfics are over 2 billion words... well, maybe not, but better safe than sorry!

## The fic_info Table: Our Main Data Store

Here's the heart of our database — the `fic_info` table. Every fanfic we know about gets a row here:

```sql
-- Fic metadata cache
CREATE TABLE IF NOT EXISTS fic_info (
    id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    author_url TEXT,
    author_local_id TEXT,
    chapters INT4 NOT NULL,
    words INT8 NOT NULL,
    description TEXT NOT NULL,
    fic_created TIMESTAMPTZ NOT NULL,
    fic_updated TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL,
    source TEXT NOT NULL,
    extra_meta TEXT,
    raw_extended_meta TEXT,
    source_id INT8,
    author_id INT8,
    content_hash VARCHAR(256)
);
```

Let's read this like a recipe:

- **`id VARCHAR(128) PRIMARY KEY`** — The unique identifier for each fic. For AO3 fics, this is the work ID (like "12345"). For FFN, it's the story ID. This is the primary key — every row must have a unique id.

- **`title TEXT NOT NULL`** — The fic's title. Cannot be empty.

- **`author TEXT NOT NULL`** — The author's name. Cannot be empty.

- **`author_url TEXT`** — The author's profile URL. Can be NULL (some fics don't have this).

- **`chapters INT4 NOT NULL`** — Number of chapters. A regular integer.

- **`words INT8 NOT NULL`** — Word count. A big integer to handle very long fics.

- **`description TEXT NOT NULL** — The fic's summary/description.

- **`fic_created TIMESTAMPTZ NOT NULL`** — When the fic was originally published on the source site.

- **`fic_updated TIMESTAMPTZ NOT NULL`** — When the fic was last updated on the source site.

- **`status TEXT NOT NULL`** — Completion status ("Complete", "In Progress", "Hiatus").

- **`source TEXT NOT NULL`** — Which site the fic is from ("ao3", "ffn", etc.).

- **`content_hash VARCHAR(256)`** — A hash of the fic's content. Used to detect when a fic has been updated. NULL if we haven't fetched the content yet.

The `IF NOT EXISTS` is important — it means this command is safe to run multiple times. If the table already exists, it does nothing. If it doesn't exist, it creates it. This is what makes migrations idempotent (safe to re-run).

Notice the difference between `created` (when we added it to our database) and `fic_created` (when the fic was published on the source site). These are often different — a fic published in 2015 might not have been added to our database until 2024.

## Creating a Database Migration File

Instead of running SQL manually (which is error-prone and hard to track), we use **migration files** — numbered SQL files that get applied in order. Our migrations live in `~/code/rust/fichub/migrations/`:

```
migrations/
├── 001_initial_schema.sql    ← Creates the basic tables
├── 002_recommender.sql       ← Adds recommendation engine tables
├── 003_tagging.sql           ← Adds tagging system tables
└── 004_shelves.sql           ← Adds OPDS shelf tables
```

The naming convention is `NNN_description.sql` where `NNN` is a three-digit number. This ensures they run in order (001 before 002, etc.).

Migration files are just SQL. They look like any other `.sql` file, but they're managed by SQLx's migration system. The key difference is that SQLx tracks which migrations have been applied, so running the server again won't re-apply old ones.

🧪 **Try It Yourself**: Create a new migration file `005_test.sql` with a simple `CREATE TABLE test_table (id SERIAL PRIMARY KEY, name TEXT);`. Restart the server and check if the table was created. You can check with `psql`:

```bash
psql -U your_user -d fichub -c "\dt test_table"
```

## Running Migrations with sqlx::migrate!

Here's how we run migrations on startup, from `db/mod.rs`:

```rust
// Find the migrations directory
let migrations_path = if let Ok(exe_path) = std::env::current_exe() {
    if let Some(exe_dir) = exe_path.parent() {
        exe_dir.join("migrations")
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")
    }
} else {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")
};

// Apply any new migrations
if migrations_path.exists() {
    sqlx::migrate::Migrator::new(migrations_path)
        .await?
        .run(&pool)
        .await?;
    tracing::info!("Database migrations applied");
}
```

The `sqlx::migrate!` macro (or `Migrator::new`) does something clever:

1. It creates a special table called `_sqlx_migrations` in your database (if it doesn't exist)
2. It reads all `.sql` files from the migrations directory, sorted by name
3. It checks which ones have already been applied (by looking at the tracking table)
4. It applies any new ones, in order
5. It marks them as done in the tracking table

So every time you start the server, it automatically applies any new migrations. You never have to run them manually!

The path resolution logic handles two cases:
- **Running from the compiled binary** — migrations are next to the executable
- **Running with `cargo run`** — migrations are in `CARGO_MANIFEST_DIR` (your project root)

## SQL Basics: The Language of Databases

Before we go further, let's make sure you understand the basic SQL commands. Think of SQL as a recipe language — you tell the database exactly what to do.

### SELECT: Reading Data

```sql
-- Get everything from a table
SELECT * FROM fic_info;

-- Get specific columns
SELECT title, author, words FROM fic_info;

-- Filter with WHERE
SELECT * FROM fic_info WHERE words > 10000;

-- Combine conditions
SELECT * FROM fic_info WHERE words > 10000 AND status = 'Complete';

-- Sort by a column
SELECT * FROM fic_info ORDER BY words DESC LIMIT 10;

-- Count rows
SELECT COUNT(*) FROM fic_info WHERE source = 'ao3';
```

The `*` means "all columns." The `WHERE` clause filters rows. `ORDER BY` sorts results. `LIMIT` caps the number of results.

### INSERT: Adding Data

```sql
-- Insert one row
INSERT INTO fic_info (id, title, author, words, status, source)
VALUES ('12345', 'A Great Story', 'SomeAuthor', 50000, 'Complete', 'ao3');

-- Insert multiple rows
INSERT INTO fic_info (id, title, author, words, status, source) VALUES
    ('11111', 'Story One', 'Author A', 10000, 'Complete', 'ao3'),
    ('22222', 'Story Two', 'Author B', 25000, 'In Progress', 'ffn');
```

### UPDATE: Changing Data

```sql
-- Update one row
UPDATE fic_info SET status = 'Complete' WHERE id = '12345';

-- Update multiple rows (CAREFUL!)
UPDATE fic_info SET status = 'Hiatus' WHERE source = 'ffn';

-- Update with a condition
UPDATE fic_info SET words = words + 1000 WHERE id = '12345';
```

⚠️ **Watch Out**: Never run UPDATE or DELETE without a WHERE clause! `UPDATE fic_info SET status = 'Hiatus'` (without WHERE) changes EVERY row in the table.

### DELETE: Removing Data

```sql
-- Delete one row
DELETE FROM fic_info WHERE id = '12345';

-- Delete rows matching a condition
DELETE FROM export_log WHERE created < NOW() - INTERVAL '30 days';
```

### JOIN: Combining Tables

```sql
-- Get fics with their tags
SELECT fic_info.title, tags.name
FROM fic_info
JOIN fic_tags ON fic_tags.url_id = fic_info.id
JOIN tags ON tags.id = fic_tags.tag_id
WHERE fic_info.id = '12345';
```

JOINs let you combine data from multiple tables. The `ON` clause specifies how the tables relate to each other.

## Connection Pool Tuning

The connection pool is one of the most important settings in your application. Too few connections and requests wait; too many and PostgreSQL runs out of resources.

```rust
let pool = PgPoolOptions::new()
    .max_connections(20)                         // How many connections to create
    .min_connections(5)                          // Keep at least 5 warm connections
    .acquire_timeout(Duration::from_secs(10))    // How long to wait for a connection
    .idle_timeout(Duration::from_secs(300))      // Close idle connections after 5 minutes
    .max_lifetime(Duration::from_secs(1800))     // Close connections after 30 minutes
    .connect(database_url)
    .await?;
```

Here's what each setting means:

| Setting | Default | What It Does |
|---------|---------|-------------|
| `max_connections` | 10 | Max connections in the pool |
| `min_connections` | 0 | Min connections to keep warm |
| `acquire_timeout` | 30s | How long to wait for a free connection |
| `idle_timeout` | 10min | Close connections idle longer than this |
| `max_lifetime` | 30min | Close connections older than this |

**Rule of thumb**: Set `max_connections` to about 2-4x the number of CPU cores on your database server. For a small PostgreSQL instance with 2 cores, 10-20 connections is reasonable.

The `acquire_timeout` is critical — if all connections are busy and a new request comes in, it waits up to this duration. If the timeout expires, the request fails with an error. Set this to match your user's patience — 5-10 seconds is usually reasonable.

🧪 **Try It Yourself**: Set `max_connections(2)` and `acquire_timeout(Duration::from_secs(5))`. Then fire off 10 simultaneous curl requests. Watch the logs — you should see some requests waiting, and possibly timing out if they take too long.

## Common Query Patterns in FicHub

Let's look at some real query patterns from our codebase and understand why they're written the way they are:

### Pattern 1: Upsert (Insert or Update)

```sql
INSERT INTO fic_info (id, title, author, ...)
VALUES ($1, $2, $3, ...)
ON CONFLICT (id) DO UPDATE SET
    title = EXCLUDED.title,
    author = EXCLUDED.author
```

This is the most common pattern in FicHub. When we scrape a fic, we either add it new or update it if it already exists. `EXCLUDED` refers to the values we tried to insert.

### Pattern 2: Optional Fetch

```rust
.fetch_optional(pool)  // Returns Option<T> — None if no rows match
```

This is safer than `fetch_one()` because it doesn't panic when there are no results. Use it when "not found" is a valid outcome.

### Pattern 3: Batch Operations

```sql
INSERT INTO fic_bookmarks (user_hash, url_id, site_domain)
VALUES ($1, $2, $3), ($1, $4, $3), ($1, $5, $3)
ON CONFLICT DO NOTHING
```

Instead of inserting one row at a time (which requires a round-trip to the database for each), batch them into a single query. This is much faster for bulk operations.

### Pattern 4: Conditional Queries

```sql
SELECT * FROM fic_info
WHERE ($1::text IS NULL OR title ILIKE '%' || $1 || '%')
AND ($2::int IS NULL OR words >= $2)
```

This lets you build dynamic filters without creating separate queries for each combination. If a parameter is NULL, it's ignored.

## Testing the Connection

Let's write a simple function to test that the database connection works:

```rust
pub async fn test_connection(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Simple query to verify the connection works
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await?;
    Ok(())
}
```

If this succeeds, the connection pool is working. If it fails, you'll get an error like "connection refused" (PostgreSQL isn't running) or "authentication failed" (wrong password).

You could call this in `main.rs` after creating the pool:

```rust
if let Err(e) = db::test_connection(&db_pool).await {
    tracing::error!("Database connection failed: {}", e);
    std::process::exit(1);
}
```

⚠️ **Watch Out**: The `init_pool` function runs migrations automatically. If you're setting up a fresh database, make sure the database user has permission to create tables and run migrations. On most PostgreSQL installations, the default user has these permissions.

---

# Chapter 8: Database Migrations and Models

## Why Migrations Matter

Your database schema will change over time — you'll add tables, modify columns, create indexes. Without migrations, you'd have to manually track and apply these changes on every server. Migrations automate this process, ensuring every database is in the same state.

## What Are Migrations? (Version Control for Your Database)

You know how Git tracks changes to your source code? Migrations are the same thing, but for your database.

Imagine you have a database with 10,000 rows of data. You can't just `DROP TABLE` and recreate it — you'd lose everything! Instead, you make **small, incremental changes**:

- Migration 1: Create the initial tables (001_initial_schema.sql)
- Migration 2: Add new tables for the recommender system (002_recommender.sql)
- Migration 3: Add tagging tables (003_tagging.sql)
- Migration 4: Add shelf tables (004_shelves.sql)
- Migration 5: Add unified works model (002_unified_works.sql)

Each migration is a numbered SQL file. SQLx tracks which ones have been applied, so running the server again won't re-apply old migrations.

Why not just edit the SQL directly? Because:
1. **You can't go back** — if you drop a column, the data is gone
2. **You can't collaborate** — two people editing the same SQL is a nightmare
3. **You can't deploy safely** — you need to know exactly what changed between versions
4. **You can't test** — you can't easily run the exact same changes on a test database

Migrations solve all these problems. Each one is a self-contained, numbered, versioned change.

## The 001_initial_schema.sql File Explained Line by Line

Let's read our first migration like a recipe, understanding every line:

```sql
-- FicHub database schema
-- Migration 001: Initial schema
```

The `--` is a SQL comment. It's ignored by the database — just for humans to read. Always add comments at the top of your migrations explaining what they do.

```sql
-- Request source tracking
CREATE TABLE IF NOT EXISTS request_source (
    id BIGSERIAL PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT,
    UNIQUE(is_automated, route, description)
);
```

This creates the `request_source` table — it tracks where requests come from (browser? bot? API?). Let's understand each line:

- **`id BIGSERIAL PRIMARY KEY`** — `BIGSERIAL` is like `SERIAL` but uses 8 bytes instead of 4, allowing for billions of rows. `PRIMARY KEY` means this uniquely identifies each row.

- **`created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP`** — When the record was created. `TIMESTAMPTZ` includes timezone info. `DEFAULT CURRENT_TIMESTAMP` means it auto-fills with the current time.

- **`is_automated BOOLEAN DEFAULT FALSE`** — Whether this is a bot. Default is `FALSE` (not a bot).

- **`route TEXT`** — The API route that was hit (like "/api/v0/epub"). Can be NULL.

- **`description TEXT`** — A description of the source. Can be NULL.

- **`UNIQUE(is_automated, route, description)`** — This means you can't have two rows with the exact same combination of these three columns. If someone tries to insert a duplicate, PostgreSQL raises an error.

```sql
-- Request log
CREATE TABLE IF NOT EXISTS request_log (
    id BIGSERIAL PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    source_id BIGINT REFERENCES request_source(id),
    etype TEXT NOT NULL,
    query TEXT NOT NULL,
    info_request_ms INT4 NOT NULL,
    url_id TEXT,
    fic_info TEXT,
    export_ms INT4,
    export_file_name TEXT,
    export_file_hash TEXT,
    url TEXT
);
```

The `request_log` table records every API request. Key points:

- **`source_id BIGINT REFERENCES request_source(id)`** — This is a **foreign key**. It links each log entry to a source. The `REFERENCES` clause means `source_id` must match an existing `id` in the `request_source` table. If you try to reference a source that doesn't exist, PostgreSQL rejects the insert.

- **`info_request_ms INT4 NOT NULL`** — How long the metadata lookup took (in milliseconds). This helps us track performance.

- **`url_id TEXT`** — The fic's ID, if we found one. NULL if the lookup failed.

```sql
CREATE INDEX IF NOT EXISTS idx_request_log_url_id_etype_created
    ON request_log(url_id, etype, created);
```

An **index** is like a table of contents — it makes certain queries much faster. Without this index, searching for all logs with a specific `url_id` would require scanning every row in the table (a "sequential scan"). With the index, PostgreSQL can jump straight to the right rows (an "index scan").

The index covers three columns (`url_id`, `etype`, `created`) because our most common query filters by all three. This is called a "composite index."

```sql
CREATE INDEX IF NOT EXISTS idx_request_log_date_export
    ON request_log(created)
    WHERE export_file_name IS NOT NULL AND etype = 'epub';
```

This is a **partial index** — it only indexes rows that match the WHERE clause. This is more efficient than indexing everything because:
1. The index is smaller (fewer rows)
2. Queries that match the WHERE clause are faster
3. Less disk space is used

Now the main table:

```sql
CREATE TABLE IF NOT EXISTS fic_info (
    id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    author_url TEXT,
    author_local_id TEXT,
    chapters INT4 NOT NULL,
    words INT8 NOT NULL,
    description TEXT NOT NULL,
    fic_created TIMESTAMPTZ NOT NULL,
    fic_updated TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL,
    source TEXT NOT NULL,
    extra_meta TEXT,
    raw_extended_meta TEXT,
    source_id INT8,
    author_id INT8,
    content_hash VARCHAR(256)
);
```

This is the core table. Every fanfic we know about gets a row here. Notice:

- The `id` is a `VARCHAR(128)`, not a `SERIAL`. That's because different sites use different ID formats — AO3 uses numeric IDs, but we might need room for other formats.

- `content_hash VARCHAR(256)` stores an MD5 or SHA-256 hash of the fic's content. When we check if a fic has been updated, we compare this hash to a new hash of the current content. If they differ, the fic has been modified.

- We have both `created` (when we added it to our database) and `fic_created` (when it was published on the source site). These are different dates!

The export log and blacklist tables follow similar patterns:

```sql
-- Export log (cache tracking)
CREATE TABLE IF NOT EXISTS export_log (
    url_id VARCHAR(128) REFERENCES fic_info(id),
    version INT NOT NULL,
    etype TEXT NOT NULL,
    input_hash TEXT NOT NULL,
    export_hash TEXT NOT NULL,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(url_id, version, etype, input_hash)
);
```

The `UNIQUE(url_id, version, etype, input_hash)` constraint means you can't have two exports with the same fic, version, format, and input. This is what enables cache hits — if the same export exists, we serve the cached version instead of regenerating it.

## Creating the export_log Table

We already covered this above! The key insight is the unique constraint and the `REFERENCES fic_info(id)` foreign key. The foreign key ensures that `export_log` entries always reference a valid fic — you can't have an export for a fic that doesn't exist.

## The FicInfo Struct: A Rust Struct That Matches the Database Row

This is where SQLx gets magical. Our `FicInfo` struct in `db/models.rs` is a **mirror** of the database table:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Fanfiction metadata as stored in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicInfo {
    pub id: String,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub title: String,
    pub author: String,
    pub author_url: Option<String>,
    pub author_local_id: Option<String>,
    pub chapters: i32,
    pub words: i64,
    pub description: String,
    pub fic_created: DateTime<Utc>,
    pub fic_updated: DateTime<Utc>,
    pub status: String,
    pub source: String,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
    pub source_id: Option<i64>,
    pub author_id: Option<i64>,
    pub content_hash: Option<String>,
}
```

Notice how every field maps to a database column:

| Rust Field | Rust Type | SQL Column | SQL Type | Nullable? |
|-----------|-----------|------------|----------|-----------|
| `id` | `String` | `id` | `VARCHAR(128)` | No (PRIMARY KEY) |
| `created` | `Option<DateTime<Utc>>` | `created` | `TIMESTAMPTZ` | Yes |
| `title` | `String` | `title` | `TEXT` | No |
| `author` | `String` | `author` | `TEXT` | No |
| `chapters` | `i32` | `chapters` | `INT4` | No |
| `words` | `i64` | `words` | `INT8` | No |
| `description` | `String` | `description` | `TEXT` | No |
| `fic_created` | `DateTime<Utc>` | `fic_created` | `TIMESTAMPTZ` | No |
| `status` | `String` | `status` | `TEXT` | No |
| `source` | `String` | `source` | `TEXT` | No |
| `extra_meta` | `Option<String>` | `extra_meta` | `TEXT` | Yes |
| `content_hash` | `Option<String>` | `content_hash` | `VARCHAR(256)` | Yes |

The `Option<...>` types correspond to nullable columns. In PostgreSQL, a column is nullable unless you say `NOT NULL`. In Rust, nullable means `Option<T>`. The mapping is automatic — SQLx's `FromRow` derive macro handles it.

The derive macros on the struct do the following:
- `Debug` — lets you print the struct with `{:?}` for debugging
- `Clone` — lets you make copies of the struct
- `Serialize` — lets you convert the struct to JSON (for API responses)
- `Deserialize` — lets you create the struct from JSON (for API requests)
- `FromRow` — lets SQLx convert a database row to this struct

## FromRow Derive: Automatically Mapping Rows to Structs

The `#[derive(FromRow)]` macro is the magic that connects Rust to PostgreSQL. It generates code that automatically converts a database row into a `FicInfo` struct:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
```

Without `FromRow`, you'd have to manually extract each field from the row:

```rust
// WITHOUT FromRow (DON'T DO THIS!)
let row = sqlx::query("SELECT * FROM fic_info WHERE id = $1")
    .bind(id)
    .fetch_optional(pool)
    .await?;

let fic = row.map(|r| FicInfo {
    id: r.get("id"),
    title: r.get("title"),
    author: r.get("author"),
    chapters: r.get("chapters"),
    words: r.get("words"),
    description: r.get("description"),
    // ... 15 more fields ...
});
```

That's tedious, error-prone, and hard to maintain. With `FromRow`:

```rust
// WITH FromRow (DO THIS!)
let fic: Option<FicInfo> = sqlx::query_as::<_, FicInfo>(
    "SELECT * FROM fic_info WHERE id = $1",
)
.bind(id)
.fetch_optional(pool)
.await?;
```

One line versus twenty. And if you add a column to the database but forget to add it to the struct, SQLx will give you a compile-time error. That's type safety in action.

## Creating the Migration for Recommendations: 002_recommender.sql

Our second migration adds the recommendation engine — a system that suggests similar fics based on user favourites. This migration creates several related tables:

```sql
-- fic_works: site-specific work metadata and favourite counts
CREATE TABLE IF NOT EXISTS fic_works (
    url_id VARCHAR(128) PRIMARY KEY REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    site_work_id VARCHAR(255) NOT NULL,
    favouriter_count INT4 NOT NULL DEFAULT 0,
    first_favourite_scraped TIMESTAMPTZ,
    last_favourite_scraped TIMESTAMPTZ,
    last_cooccur_update TIMESTAMPTZ,
    UNIQUE(site_domain, site_work_id)
);
```

The `REFERENCES fic_info(id) ON DELETE CASCADE` is important — if a fanfic is deleted from `fic_info`, all related `fic_works` rows are automatically deleted too. This is called "cascade delete" and it keeps our data consistent.

The `UNIQUE(site_domain, site_work_id)` constraint ensures each work is only tracked once per site.

```sql
-- fic_bookmarks: tracks which user favourited which work
CREATE TABLE IF NOT EXISTS fic_bookmarks (
    user_hash VARCHAR(64) NOT NULL,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    first_seen TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_hash, url_id)
);
```

We store a `user_hash` (anonymized user ID) rather than actual user information. This is a privacy measure — we can track patterns without knowing who users are. The hash is a one-way transformation (like a fingerprint) that can't be reversed to find the original user.

```sql
-- fic_bookmark_cooccur: pairwise co-occurrence counts
CREATE TABLE IF NOT EXISTS fic_bookmark_cooccur (
    work_a VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    work_b VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    cooccur_count INT4 NOT NULL DEFAULT 1,
    last_updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (work_a, work_b),
    CHECK (work_a < work_b)
);
```

This is the co-occurrence table — it counts how many users have favourited both Work A and Work B. The `CHECK (work_a < work_b)` constraint is clever — it ensures we always store the pair in alphabetical order. Without it, we might store both (A, B) and (B, A), which would double-count. With the constraint, only one ordering is allowed.

```sql
-- recommendation_suggestions: community-submitted recommendations
CREATE TABLE IF NOT EXISTS recommendation_suggestions (
    id BIGSERIAL PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    suggested_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    submitted_by_ip INET NOT NULL,
    comment TEXT,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(url_id, suggested_url_id, submitted_by_ip)
);

-- recommendation_votes: upvotes/downvotes on suggestions
CREATE TABLE IF NOT EXISTS recommendation_votes (
    suggestion_id BIGINT NOT NULL REFERENCES recommendation_suggestions(id) ON DELETE CASCADE,
    voter_ip INET NOT NULL,
    vote SMALLINT NOT NULL CHECK (vote IN (-1, 1)),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (suggestion_id, voter_ip)
);
```

The voting system uses `CHECK (vote IN (-1, 1))` to ensure votes are either -1 (downvote) or +1 (upvote). No other values are allowed.

## The Tagging System Migration (003_tagging.sql)

Our third migration adds a sophisticated tagging system — users can tag fics with categories like "fandom", "character", "relationship", and "freeform". Let's look at the key parts:

```sql
-- Tag types (fixed enum)
CREATE TABLE IF NOT EXISTS tag_types (
    id SMALLINT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

INSERT INTO tag_types (id, name) VALUES
    (1, 'fandom'),
    (2, 'character'),
    (3, 'relationship'),
    (4, 'freeform'),
    (5, 'warning'),
    (6, 'category'),
    (7, 'other')
ON CONFLICT (id) DO NOTHING;
```

This creates a "type" system for tags. Each tag belongs to a type (fandom, character, etc.). The `INSERT ... ON CONFLICT DO NOTHING` is important — it's idempotent, meaning running it multiple times doesn't create duplicates.

```sql
-- Canonical tags
CREATE TABLE IF NOT EXISTS tags (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE COLLATE "C",
    tag_type_id SMALLINT NOT NULL REFERENCES tag_types(id),
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

The `COLLATE "C"` is interesting — it makes tag name comparison case-sensitive and fast. "Harry Potter" and "harry potter" are different tags. This is intentional for a tagging system.

```sql
-- Fic-tag junction (which tags are on which fics)
CREATE TABLE IF NOT EXISTS fic_tags (
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    added_by_ip INET NOT NULL DEFAULT '0.0.0.0',
    score SMALLINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (url_id, tag_id)
);
```

This is a **junction table** — it connects fics to tags. Many-to-many relationships in SQL always use junction tables. A fic can have many tags, and a tag can be on many fics.

```sql
-- Function: update fic_tags.score when a vote is inserted/updated/deleted
CREATE OR REPLACE FUNCTION update_fic_tag_score()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE fic_tags SET score = score + NEW.value
        WHERE url_id = NEW.url_id AND tag_id = NEW.tag_id;
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' AND NEW.value <> OLD.value THEN
        UPDATE fic_tags SET score = score - OLD.value + NEW.value
        WHERE url_id = NEW.url_id AND tag_id = NEW.tag_id;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE fic_tags SET score = score - OLD.value
        WHERE url_id = OLD.url_id AND tag_id = OLD.tag_id;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;
```

This is a PostgreSQL **trigger function** — it runs automatically whenever votes are inserted, updated, or deleted. It keeps the `score` column in `fic_tags` synchronized with the votes. This is a classic database pattern: store the detail data (votes) and maintain a summary (score) automatically.

```sql
-- Add full-text search to fic_info
ALTER TABLE fic_info ADD COLUMN IF NOT EXISTS text_search tsvector
    GENERATED ALWAYS AS (
        setweight(to_tsvector('english', coalesce(title,'')), 'A') ||
        setweight(to_tsvector('english', coalesce(description,'')), 'B')
    ) STORED;
CREATE INDEX IF NOT EXISTS idx_fic_info_text_search ON fic_info USING GIN(text_search);
```

This adds PostgreSQL's built-in full-text search. The `tsvector` type stores pre-processed search terms. The `setweight` function assigns importance: title matches (weight 'A') rank higher than description matches (weight 'B'). The `GIN` index makes searches fast.

## The Unified Works Migration (002_unified_works.sql)

Our most important migration adds the unified works model — a system where the same story from different sites appears as one canonical entry.

```sql
-- works: canonical story entries
CREATE TABLE IF NOT EXISTS works (
    id SERIAL PRIMARY KEY,
    canonical_title TEXT NOT NULL,
    canonical_author TEXT NOT NULL,
    description TEXT DEFAULT '',
    default_source_id VARCHAR(128) REFERENCES fic_info(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

The `works` table is the heart of the unified model. Each row represents one story — regardless of how many sites it's posted on. The `default_source_id` points to the primary source (usually the original publication site).

```sql
-- Add work_id FK to fic_info
ALTER TABLE fic_info ADD COLUMN IF NOT EXISTS work_id INTEGER REFERENCES works(id);
CREATE INDEX IF NOT EXISTS idx_fic_info_work_id ON fic_info(work_id);
```

This links each source (fic_info row) to its parent work. A story posted on AO3 and FFN would have two fic_info rows, both pointing to the same works row.

```sql
-- work_proposals: curator merge/split proposals
CREATE TABLE IF NOT EXISTS work_proposals (
    id SERIAL PRIMARY KEY,
    proposer_id INTEGER NOT NULL REFERENCES users(id),
    action_type TEXT NOT NULL CHECK (action_type IN ('merge', 'split')),
    source_work_id INTEGER REFERENCES works(id),
    target_work_id INTEGER REFERENCES works(id),
    work_id INTEGER REFERENCES works(id),
    details JSONB,
    status TEXT DEFAULT 'pending' CHECK (status IN ('pending','accepted','rejected')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    closed_at TIMESTAMPTZ
);
```

Curators can propose merging two works (if they're the same story) or splitting a work (if it contains multiple stories). The `status` column tracks whether the proposal is pending, accepted, or rejected.

```sql
-- work_proposal_votes: voting on proposals
CREATE TABLE IF NOT EXISTS work_proposal_votes (
    proposal_id INTEGER NOT NULL REFERENCES work_proposals(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id),
    vote SMALLINT NOT NULL CHECK (vote IN (-1, 0, 1)),
    voted_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (proposal_id, user_id)
);
```

Each curator can vote once per proposal. A vote of 1 means approve, -1 means reject, 0 means retract. When 3 curators approve and at least 2 have voted, the proposal is accepted automatically.

```sql
-- reputation_events: track reputation changes
CREATE TABLE IF NOT EXISTS reputation_events (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    event_type TEXT NOT NULL,
    delta INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_reputation_events_user ON reputation_events(user_id);
```

Every reputation change is logged here. When a proposal is accepted, the proposer gets +5 and each approving voter gets +2. When rejected, the proposer gets -1. At 100 reputation, a reader is auto-promoted to curator.

The migration also bootstraps existing data: it scans fic_info for rows with the same title and author, groups them into works, and links them via work_id. This is how the unified model gets populated from the existing database.

## Schema Design Principles

Looking at all our migrations together, some patterns emerge:

### 1. Use Appropriate Data Types
Don't use `TEXT` for everything. Use `INT4` for small numbers, `INT8` for large ones, `BOOLEAN` for true/false, `TIMESTAMPTZ` for dates. This saves space and catches bugs.

### 2. Add Constraints
`NOT NULL`, `UNIQUE`, `FOREIGN KEY`, `CHECK` — constraints prevent bad data from entering your database. It's easier to enforce rules in the schema than in application code.

### 3. Create Indexes for Common Queries
Look at your `WHERE` clauses and `JOIN` conditions. If you're filtering by a column frequently, add an index on it.

### 4. Use Foreign Keys with CASCADE
`REFERENCES fic_info(id) ON DELETE CASCADE` ensures that when a fic is deleted, all related records (tags, bookmarks, exports) are automatically cleaned up. Without CASCADE, you'd have orphaned records.

### 5. Make Migrations Idempotent
Always use `IF NOT EXISTS` and `ON CONFLICT`. This makes migrations safe to run multiple times, which is essential for automated deployment.

## Running Migrations on Startup

As we saw in Chapter 7, migrations run automatically when the server starts. This is handled in `db::init_pool()`:

```rust
sqlx::migrate::Migrator::new(migrations_path)
    .await?
    .run(&pool)
    .await?;
```

The `Migrator` reads all `.sql` files from the migrations directory, sorts them by name, and applies any that haven't been run yet. It tracks which ones have been applied in a special `_sqlx_migrations` table that looks like:

```sql
SELECT * FROM _sqlx_migrations;

--  version |          description          |          installed_on          | success
----------+-------------------------------+-------------------------------+---------
       1 | 001_initial_schema            | 2024-01-15 10:30:00+00        | t
       2 | 002_recommender               | 2024-01-15 10:30:01+00        | t
       3 | 003_tagging                   | 2024-01-15 10:30:02+00        | t
       4 | 004_shelves                   | 2024-01-15 10:30:03+00        | t
       5 | 002_unified_works             | 2026-07-28 20:30:00+00        | t
```

⚠️ **Watch Out**: Never manually edit a migration that's already been applied! If you need to change a schema, create a new migration (e.g., `005_fix_column.sql`). Changing an applied migration can cause the tracking table to get out of sync, leading to confusing errors.

⚠️ **Watch Out**: Always use `IF NOT EXISTS` in your migrations. This makes them idempotent — safe to run multiple times. Without it, re-running a migration would fail with "table already exists."

🧪 **Try It Yourself**: Look at the `_sqlx_migrations` table in your database. What columns does it have? How does SQLx track which migrations have been run? You can check with:

```bash
psql -U your_user -d fichub -c "SELECT * FROM _sqlx_migrations;"
```

---

# Chapter 9: CRUD Operations

## Why CRUD Matters

Every application — from a simple todo list to a massive social network — fundamentally does four things with data: creates it, reads it, updates it, and deletes it. Mastering CRUD is the foundation of database programming. Once you can do these four things safely and efficiently, you can build anything.

## What Is CRUD? (Create, Read, Update, Delete)

CRUD stands for the four basic things you can do with data. Every database interaction falls into one of these categories:

| Operation | SQL | What It Does | Restaurant Analogy |
|-----------|-----|-------------|-------------------|
| **Create** | INSERT | Add new data | A new customer sits down and orders |
| **Read** | SELECT | Look up data | Waiter reads the order |
| **Update** | UPDATE | Change existing data | Customer changes their order |
| **Delete** | DELETE | Remove data | Customer leaves and their order is removed |

Understanding CRUD is fundamental to building any application. Let's look at how FicHub does each one.

## INSERT: Saving a Fic to the Database

Here's how we save a fanfic to the database — the `upsert_fic_info` function:

```rust
pub async fn upsert_fic_info(pool: &PgPool, fic: &FicInfo) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash, updated
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, NOW())
        ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            author = EXCLUDED.author,
            chapters = EXCLUDED.chapters,
            words = EXCLUDED.words,
            description = EXCLUDED.description,
            fic_updated = EXCLUDED.fic_updated,
            status = EXCLUDED.status,
            extra_meta = EXCLUDED.extra_meta,
            raw_extended_meta = EXCLUDED.raw_extended_meta,
            content_hash = EXCLUDED.content_hash,
            updated = NOW()"#,
    )
    .bind(&fic.id)
    .bind(&fic.title)
    .bind(&fic.author)
    .bind(&fic.author_url)
    .bind(&fic.author_local_id)
    .bind(fic.chapters)
    .bind(fic.words)
    .bind(&fic.description)
    .bind(fic.fic_created)
    .bind(fic.fic_updated)
    .bind(&fic.status)
    .bind(&fic.source)
    .bind(&fic.extra_meta)
    .bind(&fic.raw_extended_meta)
    .bind(fic.source_id)
    .bind(fic.author_id)
    .bind(&fic.content_hash)
    .execute(pool)
    .await?;
    Ok(())
}
```

This is an **upsert** (UPDATE + INSERT). Here's what happens:

1. **Try to INSERT** a new row with the fic's data
2. **If the id already exists** (`ON CONFLICT`), UPDATE the existing row instead
3. **Set `updated = NOW()`** to track when we last touched this record

The `EXCLUDED` keyword in the ON CONFLICT clause refers to the values you tried to insert. So `title = EXCLUDED.title` means "set title to the value we tried to insert."

The `$1, $2, $3...` are **placeholders** — they're replaced by the `.bind()` values. This is crucial for security! If you concatenated strings instead:

```rust
// DANGEROUS! NEVER DO THIS! SQL INJECTION VULNERABILITY!
let query = format!("INSERT INTO fic_info (id, title) VALUES ('{}', '{}')", id, title);
```

An attacker could inject SQL:
```
id = "'; DROP TABLE fic_info; --"
```

This would generate:
```sql
INSERT INTO fic_info (id, title) VALUES (''; DROP TABLE fic_info; --', 'some title')
```

Which would DROP your entire fic_info table! With `.bind()`, the database driver handles escaping. User input is always treated as data, never as SQL code.

🧪 **Try It Yourself**: Create a simple function that inserts a row into a test table. What happens if you try to insert a duplicate primary key without `ON CONFLICT`?

## SELECT: Finding a Fic by url_id

Reading data is the most common operation. Here's `get_fic_info`:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
```

Let's break this down piece by piece:

- **`sqlx::query_as::<_, FicInfo>`** — "Run a SQL query and convert the result into a `FicInfo` struct." The `_,` is a turbofish that lets you specify the output type.

- **`"SELECT * FROM fic_info WHERE id = $1"`** — The SQL query. `$1` is the first placeholder (like a blank in a fill-in-the-blank test).

- **`.bind(id)`** — "Put the `id` value into `$1`." This safely escapes the value.

- **`.fetch_optional(pool)`** — "Run the query and return at most one result. If nothing matches, return `None`."

The return type is `Option<FicInfo>` — either `Some(fic)` if found, or `None` if not. This is much safer than `fetch_one()`, which would panic if no rows are returned.

SQLx offers several fetch methods:

| Method | Returns | Use When |
|--------|---------|----------|
| `fetch_optional(pool)` | `Option<Row>` | You expect 0 or 1 results |
| `fetch_one(pool)` | `Row` | You expect exactly 1 result (panics if 0) |
| `fetch_all(pool)` | `Vec<Row>` | You expect multiple results |
| `fetch_scalar(pool)` | `Option<T>` | You want a single value (like COUNT) |
| `execute(pool)` | `QueryResult` | You don't need data back (INSERT/UPDATE/DELETE) |

For searching similar fics, we use `fetch_all`:

```rust
pub async fn search_similar_fics(pool: &PgPool, query: &str) -> AppResult<Vec<FicInfo>> {
    let rows = sqlx::query_as::<_, FicInfo>(
        r#"SELECT * FROM fic_info
           WHERE title ILIKE $1 OR author ILIKE $2
           LIMIT 5"#,
    )
    .bind(format!("%{}%", query))
    .bind(format!("%{}%", query))
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
```

The `ILIKE` operator is case-insensitive LIKE — it matches "Harry" and "harry" and "HARRY". The `%` wildcards mean "anything before and after." So `%harry%` matches any title containing "harry" anywhere.

## UPDATE: Changing a Fic's Status

Updating data uses the `UPDATE` SQL statement:

```sql
UPDATE fic_info SET status = $1, updated = NOW() WHERE id = $2
```

And in Rust:

```rust
pub async fn update_fic_status(pool: &PgPool, id: &str, status: &str) -> AppResult<()> {
    sqlx::query("UPDATE fic_info SET status = $1, updated = NOW() WHERE id = $2")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
```

The `WHERE id = $2` clause is critical — without it, you'd update **every row** in the table! Always double-check your WHERE clause before running an UPDATE.

Our export log uses an upsert pattern for updates:

```rust
pub async fn insert_export_log(
    pool: &PgPool,
    url_id: &str,
    version: i32,
    etype: &str,
    input_hash: &str,
    export_hash: &str,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO export_log (url_id, version, etype, input_hash, export_hash)
           VALUES ($1, $2, $3, $4, $5)
           ON CONFLICT (url_id, version, etype, input_hash) DO UPDATE SET
               export_hash = EXCLUDED.export_hash,
               created = NOW()"#,
    )
    .bind(url_id)
    .bind(version)
    .bind(etype)
    .bind(input_hash)
    .bind(export_hash)
    .execute(pool)
    .await?;
    Ok(())
}
```

This is an upsert — if the export already exists (same fic, version, format, and input), we update the export hash and timestamp. If it's new, we insert it.

## DELETE: Removing Old Cache Entries

Deleting data uses the `DELETE` SQL statement. Our tag management functions show safe deletion patterns:

```rust
pub async fn delete_tag(pool: &PgPool, tag_id: i32, force: bool) -> AppResult<()> {
    // Safety check: don't delete if fics are using this tag
    if !force {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM fic_tags WHERE tag_id = $1",
        )
        .bind(tag_id)
        .fetch_one(pool)
        .await?;

        if count.0 > 0 {
            return Err(AppError::BadRequest(
                -5,
                format!("tag {} is used by {} fics, use ?force=true", tag_id, count.0),
            ));
        }
    }

    // Delete in reverse dependency order (children first, then parent)
    sqlx::query("DELETE FROM fic_tags WHERE tag_id = $1")
        .bind(tag_id).execute(pool).await?;

    sqlx::query("DELETE FROM tag_aliases WHERE canonical_tag_id = $1")
        .bind(tag_id).execute(pool).await?;

    sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(tag_id).execute(pool).await?;

    Ok(())
}
```

Notice three important patterns:

1. **Safety check first** — we check if any fics use this tag before deleting. This prevents accidental data loss.

2. **Reverse dependency order** — we delete `fic_tags` first (the children), then `tag_aliases`, then `tags` (the parent). If we deleted `tags` first, the foreign key constraint on `fic_tags` would prevent the deletion.

3. **The `force` parameter** — when `force=true`, we skip the safety check and delete everything. This is for admin use when you really need to clean up.

⚠️ **Watch Out**: Always use a WHERE clause with DELETE! `DELETE FROM table` (without WHERE) deletes **everything** in the table. That's usually catastrophic. Always test your DELETE queries with a SELECT first:

```sql
-- First, check what would be deleted:
SELECT * FROM fic_tags WHERE tag_id = 1;

-- Then, when you're sure:
DELETE FROM fic_tags WHERE tag_id = 1;
```

## The queries.rs File: Database Functions as Rust Functions

Our `queries.rs` file is organized as a collection of async functions — one per operation. This is a clean pattern that makes database code easy to find and maintain:

| Function | Operation | What It Does |
|----------|-----------|-------------|
| `upsert_fic_info` | INSERT/UPDATE | Save or update a fanfic's metadata |
| `get_fic_info` | SELECT | Look up a fic by its ID |
| `insert_request_source` | INSERT | Record where a request came from |
| `insert_request_log` | INSERT | Log an API request |
| `find_export_log` | SELECT | Check if we have a cached export |
| `insert_export_log` | INSERT/UPDATE | Record a new export in the cache |
| `check_fic_blacklist` | SELECT | Check if a fic is blocked |
| `check_author_blacklist` | SELECT | Check if an author is blocked |
| `get_fic_version_bump` | SELECT | Check for cache invalidation signals |
| `search_similar_fics` | SELECT | Find fics with similar titles |
| `lookup_tag_by_name` | SELECT | Find a tag by its exact name |
| `create_tag` | INSERT | Create a new canonical tag |
| `upsert_fic_tag` | INSERT/UPDATE | Tag a fanfic with a tag |
| `get_fic_tags` | SELECT | Get all tags for a fic with scores |
| `upsert_tag_vote` | INSERT/UPDATE | Record a vote on a tag |
| `get_existing_vote` | SELECT | Check if a user already voted |
| `insert_tag_flag` | INSERT | Flag a tag for review |
| `list_unresolved_flags` | SELECT | Get flags needing curator attention |
| `resolve_flag` | UPDATE | Mark a flag as resolved |
| `check_rate_limit` | Redis | Check and increment rate limit counter |
| `create_tag_alias` | INSERT | Create an alias for a tag |
| `merge_tags` | UPDATE/DELETE | Merge one tag into another |
| `delete_tag` | DELETE | Remove a tag and its associations |

Each function follows the same pattern:
1. Accept the `PgPool` (or `redis::aio::MultiplexedConnection`) and the data needed
2. Build a SQL query with `.bind()` for parameters
3. Execute with the appropriate fetch method
4. Return `AppResult<T>` (which automatically converts database errors to `AppError`)

The consistent pattern makes the code predictable. When you see `pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>>`, you know exactly what it does without reading the implementation.

## Using sqlx::query_as with PostgreSQL

The `query_as` function is the key to type-safe database access:

```rust
// query_as: converts rows to structs automatically (PREFERRED)
let fic: Option<FicInfo> = sqlx::query_as::<_, FicInfo>(
    "SELECT * FROM fic_info WHERE id = $1",
)
.bind(id)
.fetch_optional(pool)
.await?;

// query: returns raw rows (less convenient, more manual work)
let row: Option<sqlx::postgres::PgRow> = sqlx::query(
    "SELECT * FROM fic_info WHERE id = $1",
)
.bind(id)
.fetch_optional(pool)
.await?;
```

The `<_, FicInfo>` part is a turbofish — it tells Rust "convert the result into `FicInfo`." The `_` means "figure out the row type automatically."

For simple queries where you just need one value, use `query_scalar`:

```rust
let count: Option<i64> = sqlx::query_scalar::<_, i64>(
    "SELECT COUNT(*) FROM fic_tags WHERE tag_id = $1",
)
.bind(tag_id)
.fetch_optional(pool)
.await?;
```

This is more efficient than `query_as` when you only need a single value — no need to create a whole struct.

For queries that return tuples of known types:

```rust
let row: (i32, String, i16) = sqlx::query_as(
    "SELECT id, name, tag_type_id FROM tags WHERE name = $1",
)
.bind(name)
.fetch_optional(pool)
.await?
.unwrap_or_default();
```

The tuple `(i32, String, i16)` matches the three columns selected. This is useful for quick lookups where you don't need a full struct.

## Understanding the `?` Operator and Error Propagation

The `?` operator is one of Rust's most elegant features for error handling. Let's understand it deeply because you'll use it everywhere in database code.

### How `?` Works

The `?` operator does two things:
1. If the value is `Ok(v)`, it unwraps to `v` and execution continues
2. If the value is `Err(e)`, it **returns early** from the function, converting the error

Here's a concrete example:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)  // Returns Result<Option<FicInfo>, sqlx::Error>
    .await?;               // ← ? converts sqlx::Error to AppError::Database
    Ok(row)
}
```

Let's trace what happens in two scenarios:

**Scenario 1: Success**
```
.query_as(...)
.bind(id)
.fetch_optional(pool)   → Ok(Some(fic))
.await                  → Ok(Some(fic))
?                       → Some(fic)  (unwrapped, execution continues)
Ok(row)                 → Ok(Some(fic))  (returned to caller)
```

**Scenario 2: Database error**
```
.query_as(...)
.bind(id)
.fetch_optional(pool)   → Err(sqlx::Error::ConnectionRefused)
.await                  → Err(sqlx::Error::ConnectionRefused)
?                       → EARLY RETURN: Err(AppError::Database("connection refused"))
                          (the Ok(row) line is never reached)
```

The `?` operator essentially says: "If this fails, convert the error and return it. Otherwise, give me the value."

### Without `?`

To understand why `?` is so valuable, look at what you'd write without it:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let result = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(row) => Ok(row),
        Err(e) => Err(AppError::Database(e.to_string())),
    }
}
```

That's 6 extra lines for every database call! With `?`, it's zero extra lines. The `From<sqlx::Error> for AppError` implementation handles the conversion.

### Chaining `?`

You can chain multiple `?` operators in a single function. Each one can return early:

```rust
pub async fn export_fic(pool: &PgPool, id: &str) -> AppResult<String> {
    // Step 1: Look up the fic (might fail)
    let fic = get_fic_info(pool, id).await?;   // ← Returns early if not found

    // Step 2: Check blacklist (might fail)
    let blacklist = check_fic_blacklist(pool, id).await?;  // ← Returns early on DB error

    if !blacklist.is_empty() {
        return Err(AppError::BadRequest(-7, "fic is blacklisted".into()));
    }

    // Step 3: Generate the file (might fail)
    let content = generate_epub(&fic).await?;  // ← Returns early on generation error

    Ok(content)
}
```

Each `?` is a potential early return point. If any step fails, the function returns immediately with the error. The caller never sees partial results.

### The `?` Operator in Closures

The `?` operator also works in closures and async blocks, but you need to be careful:

```rust
// This works:
let fic = sqlx::query_as::<_, FicInfo>("SELECT ...")
    .fetch_optional(pool)
    .await?;

// This also works in a closure:
let fics: Vec<FicInfo> = urls
    .iter()
    .filter_map(|url| {
        // ? works here because the closure returns Option
        let fic = lookup_fic(url).ok()?;
        Some(fic)
    })
    .collect();
```

## Error Handling in Database Operations

Every query can fail — the database might be down, the connection might time out, the query might have a syntax error. Our error handling uses the `?` operator with automatic conversion:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;  // ← If this fails, sqlx::Error is converted to AppError::Database
    Ok(row)
}
```

The `From<sqlx::Error> for AppError` implementation we saw in Chapter 6 makes this automatic. The `?` operator:
1. If the result is `Ok(value)`, unwrap it and continue
2. If the result is `Err(e)`, convert `e` to `AppError::Database` and return early from the function

This means every database function automatically handles errors cleanly — no `.unwrap()` that could panic in production. When a database error occurs, the handler returns a clean JSON response like `{"err": -1, "msg": "database error"}` instead of crashing.

For operations that need to handle "not found" gracefully:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)  // Returns Ok(None) if not found
    .await?;                // Converts sqlx::Error to AppError
    Ok(row)                 // Returns Ok(None) to the caller
}
```

The caller then decides what to do with `None`:
```rust
let fic = queries::get_fic_info(&state.db, &url_id).await?;

match fic {
    Some(fic) => Ok(Json(json!({ "title": fic.title, ... }))),
    None => Err(AppError::NotFound(format!("fic {} not found", url_id))),
}
```

🧪 **Try It Yourself**: Write a function that queries a nonexistent table. What error do you get? How does the error type get converted to `AppError`? Try adding a `.unwrap()` and see what happens when the query fails.

---

# Chapter 10: The Axum Router and Middleware

## Why Middleware Matters

Without middleware, every handler would need to log requests, check CORS headers, compress responses, and handle errors independently. That's a lot of duplicated code. Middleware solves this by wrapping handlers with reusable behavior — write once, apply everywhere. It's the difference between every waiter memorizing the health code versus having a health inspector check everyone uniformly.

## Building the Full Router with All Routes

Now let's see the complete router — the "menu" of our restaurant. This is where everything comes together: every endpoint, every handler, every piece of middleware.

Here's the full `build_router` function from `server.rs`:

```rust
async fn build_router(state: Arc<AppState>) -> Router {
    let frontend_dir = state.config.frontend_dir.clone();

    // A helper function for legacy redirects
    async fn redirect_to_root() -> Redirect {
        Redirect::to("/")
    }

    Router::new()
        // ── API routes ──────────────────────────────────────────────
        .route("/api/", get(routes::api_docs::api_docs_handler))
        .route("/api/v0/epub", get(routes::export::epub_handler))
        .route("/api/v0/meta", get(routes::meta::meta_handler))
        .route("/api/v0/remote", get(remote_handler))

        // ── Cache download routes ──────────────────────────────────
        .route("/cache/{etype}/{url_id}/{fname}",
               get(routes::cache_download::download_with_hash))
        .route("/cache/{etype}/{url_id}",
               get(routes::cache_download::download_or_export))

        // ── Recommender routes ─────────────────────────────────────
        .route("/api/v0/recommendations",
               get(crate::recommender::routes::recommendations_handler))
        .route("/api/v0/recommendations/suggest",
               post(crate::recommender::routes::suggest_handler))
        .route("/api/v0/recommendations/vote",
               post(crate::recommender::routes::vote_handler))
        .route("/api/v0/recommendations/votes",
               get(crate::recommender::routes::votes_handler))

        // ── Tag routes (v3) ────────────────────────────────────────
        .route("/api/v0/tags/submit",
               post(crate::tags::routes::submit_tag))
        .route("/api/v0/tags/vote",
               post(crate::tags::routes::vote_tag))
        .route("/api/v0/tags/flag",
               post(crate::tags::routes::flag_tag))
        .route("/api/v0/tags",
               get(crate::tags::routes::get_tags))

        // ── Curator routes (v3) ────────────────────────────────────
        .route("/api/v0/curator/alias",
               post(crate::tags::curator::create_alias))
        .route("/api/v0/curator/merge",
               post(crate::tags::curator::merge_tags))
        .route("/api/v0/curator/tags/{id}",
               delete(crate::tags::curator::delete_tag))
        .route("/api/v0/curator/flags",
               get(crate::tags::curator::list_flags))
        .route("/api/v0/curator/flags/{id}/resolve",
               post(crate::tags::curator::resolve_flag))

        // ── Search routes ──────────────────────────────────────────
        .route("/api/v0/search",
               get(crate::search::routes::search_handler))

        // ── OPDS catalog routes ────────────────────────────────────
        .route("/opds", get(crate::routes::opds::feeds::root_catalog))
        .route("/opds/new", get(crate::routes::opds::feeds::recent_feed))
        .route("/opds/popular", get(crate::routes::opds::feeds::popular_feed))
        .route("/opds/tags", get(crate::routes::opds::tags::tag_types))
        .route("/opds/tags/{type_id}",
               get(crate::routes::opds::tags::tags_by_type))
        .route("/opds/tags/{type_id}/{tag_name}",
               get(crate::routes::opds::tags::fics_by_tag))
        .route("/opds/authors",
               get(crate::routes::opds::authors::author_list))
        .route("/opds/recommendations/popular",
               get(crate::routes::opds::recommendations::popular_recommendations))
        .route("/opds/recommendations",
               get(crate::routes::opds::recommendations::fic_recommendations))
        .route("/opds/search", get(crate::routes::opds::search::search_feed))
        .route("/opds/shelves",
               get(crate::routes::opds::shelves::shelf_list))
        .route("/opds/shelf/{shelf_id}",
               get(crate::routes::opds::shelves::shelf_contents))

        // ── Legacy redirect routes ─────────────────────────────────
        .route("/legacy/epub_export", get(redirect_to_root))
        .route("/fic/{url_id}", get(redirect_to_root))
        .route("/changes", get(redirect_to_root))
        .route("/popular/", get(redirect_to_root))

        // ── Static frontend files (catch-all fallback) ─────────────
        .fallback_service(
            ServeDir::new(&frontend_dir)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(frontend_dir.join("index.html"))),
        )

        // ── Middleware (applied to ALL routes above) ────────────────
        .layer(TraceLayer::new_for_http())    // Request/response logging
        .layer(CorsLayer::permissive())       // CORS headers

        // ── Shared state ───────────────────────────────────────────
        .with_state(state)
}
```

That's a lot of routes! Let's organize them by category and understand the patterns:

### API Routes (`/api/...`)

| Route | Method | Handler | Purpose |
|-------|--------|---------|---------|
| `/api/` | GET | `api_docs_handler` | Self-documenting API docs (like a restaurant's menu) |
| `/api/v0/epub` | GET | `epub_handler` | Get metadata and download links |
| `/api/v0/meta` | GET | `meta_handler` | Get metadata only (no downloads) |
| `/api/v0/remote` | GET | `remote_handler` | Get request source info (IP, port) |
| `/api/v0/search` | GET | `search_handler` | Search for fanfics |

### Cache Routes (`/cache/...`)

| Route | Method | Handler | Purpose |
|-------|--------|---------|---------|
| `/cache/{etype}/{url_id}/{fname}` | GET | `download_with_hash` | Direct download with hash validation |
| `/cache/{etype}/{url_id}` | GET | `download_or_export` | Download if cached, otherwise trigger export |

The `{etype}` can be "epub", "html", "mobi", or "pdf". The `{url_id}` is the fic's identifier. The `{fname}` is the filename (optional).

### Tag Routes (`/api/v0/tags/...`)

| Route | Method | Handler | Purpose |
|-------|--------|---------|---------|
| `/api/v0/tags` | GET | `get_tags` | Get all tags for a fic |
| `/api/v0/tags/submit` | POST | `submit_tag` | Submit a new tag for a fic |
| `/api/v0/tags/vote` | POST | `vote_tag` | Upvote or downvote a tag |
| `/api/v0/tags/flag` | POST | `flag_tag` | Flag a tag for review |

### OPDS Routes (`/opds/...`)

OPDS (Open Publication Distribution System) is a standard for ebook catalog browsing. Our OPDS routes let ebook readers browse and download fics:

| Route | Method | Handler | Purpose |
|-------|--------|---------|---------|
| `/opds` | GET | `root_catalog` | Root catalog (starting point) |
| `/opds/new` | GET | `recent_feed` | Recently added fics |
| `/opds/popular` | GET | `popular_feed` | Popular fics |
| `/opds/tags` | GET | `tag_types` | Browse by tag type |
| `/opds/authors` | GET | `author_list` | Browse by author |
| `/opds/search` | GET | `search_feed` | Search the catalog |
| `/opds/shelves` | GET | `shelf_list` | Shared reading lists |

### Legacy Routes

```rust
.route("/legacy/epub_export", get(redirect_to_root))
.route("/fic/{url_id}", get(redirect_to_root))
.route("/changes", get(redirect_to_root))
.route("/popular/", get(redirect_to_root))
```

These redirect old URLs to the new frontend. If someone has a bookmark to `/fic/12345`, they'll be sent to the homepage instead of getting a 404. This is a nice UX touch.

## The run() Function: Starting the Server

The `run()` function orchestrates the entire startup sequence. Let's look at it again with more commentary:

```rust
pub async fn run(config: Config) {
    // STEP 1: Connect to PostgreSQL
    // This creates the connection pool and runs any pending migrations
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");
    // If PostgreSQL isn't running, we crash here with a clear message

    // STEP 2: Connect to Redis
    // Redis is our caching layer — instant lookups for frequently accessed data
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");

    // STEP 3: Build HTTP client
    // Reusable client for fetching web pages (fanfic metadata)
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")     // Identify ourselves
        .timeout(Duration::from_secs(30))   // Give up after 30 seconds
        .build()
        .expect("Failed to build HTTP client");

    // STEP 4: Initialize scraper registry
    // This knows how to extract metadata from AO3, FFN, etc.
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());

    // STEP 5: Initialize rate limiter
    // Prevents any single user from overwhelming the server
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::new(
        redis_conn.clone(),
        config.dynamic_rate_limit,
    )
    .await
    .expect("Failed to initialize rate limiter");

    // Load datacenter IPs if configured
    // (bots from datacenters get different rate limits)
    if !config.ip_tag_sources.is_empty() {
        rate_limiter.load_datacenter_ips(&config.ip_tag_sources).await;
    }

    // STEP 6: Create shared state
    // Everything handlers need, packed into one shared object
    let state = Arc::new(AppState {
        config: config.clone(),
        db: db_pool,
        redis: redis_conn,
        http_client,
        scraper_registry,
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine,
        collection_worker,
    });

    // STEP 7: Build the router with all routes
    let app = build_router(state).await;

    // STEP 8: Bind to a port and start accepting connections
    let addr = format!("0.0.0.0:{}", config.app_port);
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .expect("Server error");
}
```

The `into_make_service_with_connect_info::<std::net::SocketAddr>()` part is important — it tells Axum to pass the client's IP address to handlers that need it. This is how our `remote_handler` knows who's making the request:

```rust
async fn remote_handler(
    ConnectInfo(remote_addr): ConnectInfo<std::net::SocketAddr>,
) -> Json<Value> {
    Json(json!({
        "ip": remote_addr.ip().to_string(),
        "port": remote_addr.port(),
        "is_automated": false,
    }))
}
```

The `ConnectInfo` extractor pulls the client's socket address out of the request. Without `into_make_service_with_connect_info`, this information wouldn't be available.

The `.expect()` calls throughout are intentional — they cause the server to crash immediately if any dependency fails. In a real deployment, you'd want health checks and graceful shutdown, but for development, failing fast is the right behavior.

## Middleware: The Security Guards

**Middleware** is code that wraps around your handlers. It runs before (and sometimes after) the handler, adding extra behavior. Think of middleware as security guards at the door — they check your ID, log who came in, and maybe offer you a mint before you sit down.

Axum uses the **Layer** pattern for middleware:

```rust
use tower_http::trace::TraceLayer;
use tower_http::cors::CorsLayer;

Router::new()
    .route("/api/v0/epub", get(epub_handler))
    .route("/api/v0/meta", get(meta_handler))
    .layer(TraceLayer::new_for_http())  // ← Middleware 1
    .layer(CorsLayer::permissive())     // ← Middleware 2
```

Layers apply to **all routes above them**. So both `TraceLayer` and `CorsLayer` apply to every route in our router.

The order of layers matters! They wrap from outside in — the last layer added is the first to execute. So `CorsLayer` runs first (adds CORS headers), then `TraceLayer` runs (logs the request), then the actual handler runs.

You can also apply middleware to specific routes:

```rust
Router::new()
    .route("/public", get(public_handler))
    .route("/admin", get(admin_handler))
    .layer(auth_middleware)  // Only wraps these routes
```

This is how you protect certain routes (like admin pages) with authentication while leaving public routes open.

## TraceLayer: Logging Every Request

The `TraceLayer` logs every HTTP request that hits our server:

```rust
use tower_http::trace::TraceLayer;

.layer(TraceLayer::new_for_http())
```

When a request comes in, you'll see output like:

```
2024-01-15T10:30:00Z INFO request{method=GET uri=/api/v0/epub version=HTTP/1.1}: tower_http::trace::on_response: finished 45ms - 200 OK
```

This tells you:
- **When** the request happened (timestamp)
- **What** the request was (GET /api/v0/epub)
- **What version** of HTTP was used (HTTP/1.1)
- **How long** it took (45ms)
- **What the response was** (200 OK)

For debugging, this is invaluable. When something goes wrong, you can look at the logs and see exactly what happened. The timing information helps you spot slow queries or API calls.

For production monitoring, you might want more detailed logging:

```rust
.layer(
    TraceLayer::new_for_http()
        .on_request(|request: &Request<_>| {
            tracing::info!("Incoming: {} {}", request.method(), request.uri());
        })
        .on_response(|response: &Response<_>, latency: Duration| {
            tracing::info!("Response: {} in {:?}", response.status(), latency);
        })
)
```

## CorsLayer: Allowing Cross-Origin Requests

**CORS** (Cross-Origin Resource Sharing) is a browser security feature that prevents websites from making requests to other websites without permission.

By default, a website at `http://localhost:5173` (your SvelteKit frontend) can't make requests to `http://localhost:3000` (your Rust backend). The browser blocks it because they're different "origins" (different ports count as different origins!).

The `CorsLayer` fixes this:

```rust
use tower_http::cors::CorsLayer;

.layer(CorsLayer::permissive())
```

`CorsLayer::permissive()` allows all origins, all methods, and all headers. This is convenient for development but too permissive for production.

In production, you'd want to be more specific:

```rust
use tower_http::cors::{CorsLayer, Any, Method};
use axum::http::HeaderValue;

.layer(
    CorsLayer::new()
        .allow_origin("https://fichub.example.com".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .max_age(Duration::from_secs(3600)),
)
```

This allows:
- Only requests from `https://fichub.example.com`
- Only GET and POST methods
- Any headers
- Cached for 1 hour (browsers won't re-check CORS for an hour)

⚠️ **Watch Out**: Never use `CorsLayer::permissive()` in production with sensitive data. It allows any website on the internet to make requests to your API, which could leak user data or allow unauthorized actions.

## Serving Static Files with ServeDir

Our SvelteKit frontend builds static files (HTML, CSS, JavaScript). Axum serves them with `ServeDir`:

```rust
.fallback_service(
    ServeDir::new(&frontend_dir)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(frontend_dir.join("index.html"))),
)
```

Here's how it works, step by step:

1. **`ServeDir::new(&frontend_dir)`** — Serve files from the `./frontend/build` directory. If someone requests `/style.css`, Axum looks for `./frontend/build/style.css`.

2. **`append_index_html_on_directories(true)`** — If someone visits `/` or `/about/`, look for `index.html` in that directory. Without this, visiting `/about/` would look for a directory listing instead of the app.

3. **`.fallback(ServeFile::new(...))`** — If the file isn't found in the directory, serve `index.html` from the frontend build directory. This is crucial for Single Page Applications (SPAs) — the frontend handles routing client-side, so all unmatched paths should serve the main HTML file.

The `fallback_service` is important — it's the **last** thing Axum checks. The priority is:
1. Check all explicit routes first
2. If no route matches, try static file serving
3. If no file matches, serve `index.html` (the SPA fallback)

This means your API routes (`/api/v0/...`) are checked before static files. If someone visits `/api/v0/epub`, they hit the handler, not a file.

## Middleware Composition: The Tower Ecosystem

Axum's middleware system is built on **Tower** — a library that provides a universal interface for services, layers, and middleware. Think of Tower as the "universal remote control" for async services.

The Tower ecosystem includes many useful middleware layers:

### Request Body Size Limiting

```rust
use tower_http::limit::RequestBodyLimitLayer;

// Limit request bodies to 1MB
.layer(RequestBodyLimitLayer::new(1024 * 1024))
```

This prevents someone from sending a 1GB POST request and crashing your server.

### Gzip Compression

```rust
use tower_http::compression::CompressionLayer;

.layer(CompressionLayer::new())
```

This automatically compresses responses with gzip. For JSON responses, this can reduce bandwidth by 70-90%.

### Request Timeout

```rust
use tower::timeout::TimeoutLayer;
use std::time::Duration;

.layer(TimeoutLayer::new(Duration::from_secs(30)))
```

If a handler takes longer than 30 seconds, the connection is terminated. This prevents slow handlers from holding connections forever.

### Custom Middleware

You can write your own middleware. Here's a simple logging middleware:

```rust
use axum::middleware::{self, Next};
use axum::extract::Request;

async fn log_request(request: Request, next: Next) -> impl IntoResponse {
    let method = request.method().clone();
    let uri = request.uri().clone();

    let response = next.run(request).await;

    tracing::info!("{} {} → {}", method, uri, response.status());
    response
}

// Usage:
.layer(middleware::from_fn(log_request))
```

This middleware logs every request and its status code. The `next.run(request)` call passes the request to the next middleware (or the handler if there are no more middleware).

### Stacking Middleware

The order of `.layer()` calls matters. Middleware wraps from outside in:

```rust
Router::new()
    .route("/api", get(handler))
    .layer(TimeoutLayer::new(Duration::from_secs(30)))  // 3rd: timeout
    .layer(CompressionLayer::new())                      // 2nd: compress
    .layer(TraceLayer::new_for_http())                   // 1st: log
```

Execution order:
1. `TraceLayer` logs the incoming request
2. `CompressionLayer` prepares to compress the response
3. `TimeoutLayer` sets a 30-second timeout
4. The handler runs
5. Response goes back through CompressionLayer (compressed)
6. Response goes back through TraceLayer (logs the response)

Think of it like layers of an onion — each layer wraps the one inside it.

🧪 **Try It Yourself**: Add a `TimeoutLayer` to your router and test what happens when a handler sleeps for longer than the timeout:

```rust
async fn slow_handler() -> &'static str {
    tokio::time::sleep(Duration::from_secs(10)).await;
    "Done!"
}

// Route with 5-second timeout
.layer(TimeoutLayer::new(Duration::from_secs(5)))
```

Visit the endpoint — you should get a timeout error after 5 seconds, not after 10.

## The Complete Router Tree

Let's visualize how everything fits together in one diagram:

```
Router::new()
│
├── EXPLICIT ROUTES (checked first, in order)
│   ├── /api/                              → api_docs_handler (GET)
│   ├── /api/v0/epub                       → epub_handler (GET)
│   ├── /api/v0/meta                       → meta_handler (GET)
│   ├── /api/v0/remote                     → remote_handler (GET)
│   ├── /api/v0/search                     → search_handler (GET)
│   ├── /api/v0/recommendations            → recommendations_handler (GET)
│   ├── /api/v0/recommendations/suggest    → suggest_handler (POST)
│   ├── /api/v0/recommendations/vote       → vote_handler (POST)
│   ├── /api/v0/recommendations/votes      → votes_handler (GET)
│   ├── /api/v0/tags                       → get_tags (GET)
│   ├── /api/v0/tags/submit                → submit_tag (POST)
│   ├── /api/v0/tags/vote                  → vote_tag (POST)
│   ├── /api/v0/tags/flag                  → flag_tag (POST)
│   ├── /api/v0/curator/*                  → curator handlers
│   ├── /cache/{etype}/{url_id}/{fname}    → download_with_hash (GET)
│   ├── /cache/{etype}/{url_id}            → download_or_export (GET)
│   ├── /opds                              → root_catalog (GET)
│   ├── /opds/new, /opds/popular, etc.     → OPDS feed handlers
│   ├── /legacy/epub_export                → redirect_to_root (GET)
│   └── /fic/{url_id}                      → redirect_to_root (GET)
│
├── FALLBACK SERVICE (if no route matched)
│   └── ServeDir(frontend_dir)
│       ├── Try: /style.css → frontend/build/style.css
│       ├── Try: /about → frontend/build/about/index.html
│       └── Fallback: frontend/build/index.html (SPA)
│
├── MIDDLEWARE (applied to ALL of the above)
│   ├── CorsLayer::permissive()   → adds CORS headers
│   └── TraceLayer::new_for_http() → logs requests
│
└── STATE (available to all handlers)
    └── Arc<AppState>
        ├── config: Config
        ├── db: PgPool
        ├── redis: MultiplexedConnection
        ├── http_client: reqwest::Client
        ├── scraper_registry: ScraperRegistry
        ├── cache_semaphores: CacheSemaphores
        ├── rate_limiter: RateLimiter
        ├── recommender_engine: RecommendationEngine
        └── collection_worker: CollectionWorker
```

## The Cache System and Export Types

FicHub supports multiple export formats — EPUB, HTML, MOBI, and PDF. Each format has its own file extension, MIME type, and version number. The `EType` enum in `cache/mod.rs` defines these:

```rust
/// The type of export format
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum EType {
    Epub,
    Html,
    Mobi,
    Pdf,
}

impl EType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EType::Epub => "epub",
            EType::Html => "html",
            EType::Mobi => "mobi",
            EType::Pdf => "pdf",
        }
    }

    pub fn suffix(&self) -> &'static str {
        match self {
            EType::Epub => ".epub",
            EType::Html => ".zip",
            EType::Mobi => ".mobi",
            EType::Pdf => ".pdf",
        }
    }
}

impl std::str::FromStr for EType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "epub" => Ok(EType::Epub),
            "html" => Ok(EType::Html),
            "mobi" => Ok(EType::Mobi),
            "pdf" => Ok(EType::Pdf),
            _ => Err(()),
        }
    }
}
```

The `FromStr` implementation lets you parse strings into `EType` values:

```rust
let etype: EType = "epub".parse().unwrap();  // EType::Epub
let etype: EType = "invalid".parse();         // Err(())
```

This is used in the cache download route:

```rust
pub async fn download_with_hash(
    State(state): State<Arc<AppState>>,
    Path((etype_str, url_id, fname)): Path<(String, String, String)>,
    Query(params): Query<DownloadQuery>,
) -> Response {
    let etype = match etype_str.parse::<EType>() {
        Ok(e) => e,
        Err(_) => return Json(json!({"err": -1, "msg": "invalid format"})).into_response(),
    };
    // ... serve the file ...
}
```

The cache system uses a semaphore to prevent duplicate concurrent exports:

```rust
pub type CacheSemaphores = Arc<Mutex<HashMap<(String, EType), Arc<Semaphore>>>>;
```

When two users request the same export simultaneously, the semaphore ensures only one generates the file while the other waits. This prevents wasted work and potential race conditions.

## How Routes Connect to Handler Functions

The connection between routes and handlers is explicit — you write it yourself. This is one of Axum's strengths: there's no magic, no convention-over-configuration, no hidden routing. Everything is visible in one place.

```rust
.route("/api/v0/epub", get(routes::export::epub_handler))
```

This single line says: "When someone sends a GET request to `/api/v0/epub`, call the `epub_handler` function from the `routes::export` module."

For POST routes:

```rust
.route("/api/v0/tags/submit", post(crate::tags::routes::submit_tag))
```

For routes that handle multiple methods:

```rust
.route("/api/v0/tags",
    get(crate::tags::routes::get_tags)
     .post(crate::tags::routes::create_tag)
     .delete(crate::tags::routes::delete_tag)
)
```

The route matching is **first-match wins**. If you have overlapping routes:

```rust
.route("/api/v0/tags", get(get_tags))
.route("/api/v0/tags/submit", post(submit_tag))
```

When someone visits `/api/v0/tags/submit`, Axum first tries to match `/api/v0/tags` — but the method is POST and that route only handles GET, so it doesn't match. Then it tries `/api/v0/tags/submit` — method is POST and this route handles POST, so it matches.

⚠️ **Watch Out**: Route order matters! If you have a catch-all route, put it last. Otherwise, it will match everything and your specific routes will never be reached. The `.fallback_service()` in our router handles this correctly — it's at the very end.

⚠️ **Watch Out**: Path parameters like `{url_id}` must be at the end of a segment. You can't have `/api/{id}/epub` — Axum requires path segments to be at the end. Use `/api/v0/{id}` instead.

🧪 **Try It Yourself**: Add a new route that handles both GET and POST. GET should return `{"method": "GET"}` and POST should return `{"method": "POST"}`. Test with:

```bash
# GET request
curl http://localhost:3000/your-route

# POST request
curl -X POST http://localhost:3000/your-route
```

---

## What We Built

In Part 2, we built the entire Rust backend for FicHub. We went from an empty `main.rs` to a fully functional web server with database, configuration, error handling, middleware, and all the routes wired up. Let's recap what we accomplished:

**Chapter 5: Your First Rust Web Server (Axum)** — We learned what Axum is (a focused, composable web framework built on Tokio), how the `#[tokio::main]` macro sets up the async runtime, how to create a Router with routes, how handlers extract query parameters (`Query<...>`) and path parameters (`Path<...>`), how to share state across handlers using `Arc<AppState>`, and traced the complete 12-step journey of a request from TCP connection to JSON response. We built a complete simple server example and tested it with curl.

**Chapter 6: Configuration and Error Handling** — We built a configuration system using environment variables and `.env` files with `dotenvy`, created a comprehensive `Config` struct with `from_env()` that handles required settings (`.expect()`), optional settings with defaults (`.unwrap_or_else()`), and type conversion (`.parse()`). We built a robust `AppError` enum with eight error variants, each mapping to specific HTTP status codes and JSON error codes (-1, -5, -6, -7, -10, -429). We saw how the `From` trait enables automatic error conversion with the `?` operator, and learned four error recovery patterns: log-and-continue, fallback values, partial results, and retry on transient errors.

**Chapter 7: Connecting to PostgreSQL** — We connected to PostgreSQL using SQLx and connection pools (`PgPoolOptions`), learned how to create tables with CREATE TABLE, understood the fic_info table's schema with its 18 columns, created our first migration file, and learned how `sqlx::migrate!` runs migrations automatically on startup. We covered SQL basics (SELECT, INSERT, UPDATE, DELETE, JOIN), connection pool tuning (max_connections, acquire_timeout, idle_timeout), and common query patterns (upsert, optional fetch, batch operations, conditional queries).

**Chapter 8: Database Migrations and Models** — We understood migrations as version control for your database, read through our initial schema migration (`001_initial_schema.sql`) line by line, explored the recommender system tables (`002_recommender.sql`) with co-occurrence tracking and community voting, examined the tagging system (`003_tagging.sql`) with trigger functions and full-text search, saw how the `FicInfo` struct mirrors the database table using `#[derive(FromRow)]`, and learned about the `_sqlx_migrations` tracking table. We also covered schema design principles: appropriate data types, constraints, indexes, foreign keys with CASCADE, and idempotent migrations.

**Chapter 9: CRUD Operations** — We mastered INSERT (saving fics with upsert), SELECT (finding fics by ID, searching similar fics), UPDATE (changing statuses, recording exports), and DELETE (removing tags with safety checks). We deeply understood the `?` operator — how it unwraps `Ok` values and returns early on `Err`, how it converts between error types via `From` implementations, and how it chains across multiple fallible operations. We saw how `.bind()` prevents SQL injection, how `query_as` provides type-safe database access, and how `fetch_optional` safely handles "not found" cases.

**Chapter 10: The Axum Router and Middleware** — We assembled the complete router with all routes organized by category (API, cache, tags, OPDS, legacy), understood the `run()` function's eight-step startup sequence, learned about TraceLayer for request logging and CorsLayer for cross-origin requests, saw how ServeDir handles static files and SPA routing with fallbacks, explored the Tower middleware ecosystem (request body limiting, compression, timeouts, custom middleware), understood middleware composition (onion layers), visualized the complete router tree, and examined the cache system with the `EType` enum and semaphore-based deduplication.

### Skills You've Gained

By the end of Part 2, you can:

- **Build a web server** with Axum that handles GET and POST requests
- **Extract parameters** from URLs (path params) and query strings
- **Share state** across handlers using `Arc<AppState>`
- **Configure your app** with environment variables and `.env` files
- **Handle errors gracefully** with `AppError` and the `?` operator
- **Connect to PostgreSQL** with connection pools
- **Create and run migrations** to manage your database schema
- **Map database rows to Rust structs** with `#[derive(FromRow)]`
- **Perform CRUD operations** safely with parameterized queries
- **Add middleware** for logging, CORS, compression, and more
- **Serve static files** for a single-page application

These are fundamental skills for any Rust web developer. The patterns you've learned — extractors, state sharing, error handling, connection pooling — apply to any Axum project, not just FicHub.

### What's Coming in Part 3

In Part 3, we'll build the SvelteKit frontend that talks to this backend. You'll learn:

- How to create a SvelteKit project with TypeScript
- How to fetch data from our Rust API endpoints
- How to display fanfic metadata in a beautiful UI
- How to handle loading states, errors, and edge cases
- How to build a search interface
- How to connect the frontend and backend in development and production

The backend we built in this part is the foundation — Part 3 builds the house on top of it. See you there! 🏠

---
# Part 3: Scraping Fanfiction

*Building the system that reads stories from the web*

---

# Chapter 11: Understanding Fanfiction Sites

## The Landscape of Fanfiction

Imagine you're a librarian, but instead of organizing books on shelves, you're trying to catalog stories scattered across dozens of different websites—each with its own rules, its own layout, and its own way of presenting information. That's the challenge we face in Part 3 of this book: building scrapers that can read fanfiction from different sites and extract the information we need.

In Parts 1 and 2, we built the FicHub backend: an Axum server, a PostgreSQL database, CRUD endpoints for managing stories. But all that infrastructure is useless without data. Our scrapers are the bridge between the wild, messy world of fanfiction websites and our neatly organized database.

Think about what a scraper actually does at a high level. Someone gives it a URL like `https://archiveofourown.org/works/123456` and says, "What's the title of this story? Who wrote it? How many chapters does it have? Give me the content." The scraper has to visit that page, read the HTML, find the right elements, extract the text, and package it all into a neat data structure.

In this chapter, we'll tour the major fanfiction sites to understand how they work. By the end, you'll see why every site needs its own scraper—and why this is both the hardest and most interesting part of the project.

## Archive of Our Own (AO3)

Archive of Our Own, affectionately known as AO3, is the gold standard of modern fanfiction hosting. Run by the Organization for Transformative Works, it's a massive archive with millions of stories across thousands of fandoms. It's also one of the most scraper-friendly sites out there, with clean HTML and well-organized data.

**How AO3 organizes stories:**

Every story on AO3 is called a "work." Each work has:
- A **title** displayed prominently at the top
- An **author** with a profile link
- A **summary** (what they call the "summary" or description)
- **Tags**—lots and lots of tags. Fandoms, characters, relationships, warnings, ratings, and freeform tags
- **Stats**: word count, chapters, kudos, bookmarks, hits
- **Chapters**: each chapter is a separate section with its own title and content
- **Status**: whether the story is complete or still in progress
- **Dates**: published date and last update date

The URL structure is clean and predictable. A story lives at:
```
https://archiveofourown.org/works/123456
```

If the story has multiple chapters, you can link directly to a specific chapter:
```
https://archiveofourown.org/works/123456/chapters/789012
```

The work ID (123456) is a simple numeric identifier that's unique across the entire site. The chapter ID (789012) is also numeric. This makes extracting the ID from a URL straightforward—we just need a regex that matches `/works/(\d+)`.

Here's how the `Ao3Scraper` extracts the work ID:

```rust
fn extract_work_id(url: &str) -> Option<String> {
    // Matches /works/NUMBER or /works/NUMBER/chapters/NUMBER
    let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
    re.captures(url)?.get(1).map(|m| m.as_str().to_string())
}
```

This is pattern matching at its simplest: find `/works/`, grab the digits that follow. The `(\d+)` part is a capture group—it matches one or more digits and captures them as a group. The `?` after `ok()` handles the case where the regex fails to compile (which shouldn't happen with a hardcoded pattern, but Rust's API requires us to handle it).

And similarly, the chapter number extraction:

```rust
fn extract_chapter_number(url: &str) -> Option<i32> {
    let re = regex_lite::Regex::new(r"/chapters/(\d+)").ok()?;
    re.captures(url)?.get(1).and_then(|m| m.as_str().parse().ok())
}
```

Notice the difference: `extract_work_id` returns a `String` (the raw digits), while `extract_chapter_number` returns `Option<i32>` (the parsed integer). This is a design choice—we need the work ID as a string for the URL ID hash, but the chapter number as an integer for display.

**The HTML structure that scrapers look at:**

AO3 uses semantic HTML with clear CSS class names. This is one of the reasons it's so popular among developers—it's a pleasure to scrape (compared to some other sites we'll see shortly). The page structure for a story looks roughly like:

```html
<div id="workskin">
  <div class="preface group">
    <div class="title heading">
      <h2 class="title heading">My Amazing Fanfiction</h2>
      <h3 class="byline heading">
        <a href="/users/authorname/pseuds/authorname" rel="author">AuthorName</a>
      </h3>
    </div>

    <div class="stats group">
      <dl>
        <dt>Published:</dt>
        <dd class="published">2023-01-15</dd>
        <dt>Status:</dt>
        <dd class="status">Complete</dd>
        <dt>Chapters:</dt>
        <dd class="chapters">3 / 5</dd>
        <dt>Words:</dt>
        <dd class="words">42,156</dd>
      </dl>
    </div>

    <div class="summary module">
      <blockquote class="userstuff">
        <p>This is the summary of the story...</p>
      </blockquote>
    </div>
  </div>

  <div id="chapters">
    <div class="chapter">
      <h3 class="heading">
        <span class="chapter">1</span>
        <span class="chapter-title">The Beginning</span>
      </h3>
      <div class="userstuff module">
        <p>Once upon a time...</p>
      </div>
    </div>

    <div class="chapter">
      <h3 class="heading">
        <span class="chapter">2</span>
        <span class="chapter-title">The Middle</span>
      </h3>
      <div class="userstuff module">
        <p>And then things happened...</p>
      </div>
    </div>
  </div>
</div>
```

Notice how the class names are descriptive: `h2.title.heading` for the title, `a[rel='author']` for the author link, `dd.chapters` for chapter count, `dd.words` for word count. These selectors are what our CSS selector engine uses to find the right elements. The HTML is well-structured and predictable, which is exactly what a scraper needs.

**The magic query parameter:**

One of AO3's quirks is that by default, a story page only shows the first chapter. To get all chapters on a single page, you append `?view_full_work=true` to the URL. The AO3 scraper uses this trick:

```rust
let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
```

Without this parameter, fetching a 50-chapter story would require 50 separate HTTP requests—one for each chapter. With it, we get everything in a single request. AO3 is nice about this, and it's a huge efficiency win. The entire story content, including all chapters, metadata, and tags, loads on one page.

**AO3's tag structure:**

AO3 has the richest tagging system of any fanfiction site. Stories are tagged with:
- **Fandom tags** (Harry Potter, Marvel, Star Wars)
- **Character tags** (Harry Potter, Hermione Granger, Draco Malfoy)
- **Relationship tags** (Harry/Hermione, Steve/Tony)
- **Freeform tags** (time travel, fix-it, angst, fluff)
- **Warning tags** (major character death, non-con, underaged)
- **Rating** (General, Teen, Mature, Explicit)
- **Category** (M/M, F/M, F/F, Gen, Multi)

These tags are structured and searchable on the site itself. Our scrapers can extract them and store them in our database for advanced searching and filtering. More on this in Chapter 14.

🧪 **Try It Yourself:** Open your browser and navigate to an AO3 story. View the page source (Ctrl+U or right-click > View Page Source). Search for `class="title heading"` — can you find it? That's the element our scraper targets for the title. Now search for `rel="author"` — that's the author link. You're reading the page exactly like our scraper does.

## FanFiction.net (FF.net)

FanFiction.net is the granddaddy of fanfiction archives. It's been around since 1998, and it shows. The HTML is older, the layout is more complex, and the selectors are less intuitive. But it has an enormous catalog of stories that you won't find anywhere else—some stories have been on FF.net for over two decades.

**How FF.net organizes stories:**

Like AO3, stories are identified by numeric IDs. A story lives at:
```
https://www.fanfiction.net/s/1234567/1/
```

The `s/1234567` part identifies the story, and the `/1/` at the end is the chapter number. You change that number to navigate between chapters.

FF.net's HTML is denser and uses IDs more than classes. The metadata lives in a `#profile_top` section, and many elements share the same class name (`xcontrast_txt`), differentiated by their element type (bold tag for title, anchor tag for author, div tag for description).

```html
<div id="profile_top">
  <b class="xcontrast_txt">My Story Title</b>
  <a href="/u/12345/AuthorName" class="xcontrast_txt">AuthorName</a>
  <div class="xcontrast_txt">The summary text goes here...</div>
  <span class="xgray">Rated: T | Chapters: 3 | Words: 42,156 | Reviews: 52</span>
  <span data-xutitle="word count">42,156</span>
  <span data-xutitle="chapters">3 / 5</span>
</div>
```

Notice how the title is a bold tag (`b.xcontrast_txt`), the author is an anchor tag (`a.xcontrast_txt`), and the description is a div (`div.xcontrast_txt`). They all share the `xcontrast_txt` class, but our selectors distinguish them by element type.

The chapter content lives in a `div.storytext`:

```html
<div class="storytext xcontrast_txt">
  <p>The actual story content...</p>
  <p>More paragraphs...</p>
</div>
```

Notice the `xcontrast_txt` class everywhere? That's a quirk of FF.net's styling. The class seems to be about text contrast rather than semantic meaning. Our scraper needs to target these specific class names, which are less descriptive than AO3's.

**One request per chapter:**

Unlike AO3, FF.net doesn't have a "view full work" option. Each chapter is a separate page. So the FF.net scraper has to loop through all chapters:

```rust
for i in 1..=meta.chapters {
    let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
    // ... fetch and parse each chapter
}
```

This means a 20-chapter story requires 20 HTTP requests. We need to be mindful of rate limiting (more on that later). The trade-off is that each request is small and focused, which makes error handling easier—if chapter 3 fails, we can skip it and continue with chapter 4.

**FF.net's chapter title selector:**

One clever aspect of the FF.net scraper is how it gets chapter titles. FF.net has a dropdown menu (`select#chap_select`) that lists all chapters, with the currently selected option showing the current chapter's title:

```rust
let title = document
    .select(&Selector::parse("select#chap_select option[selected]").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| format!("Chapter {i}"));
```

This selector targets the `<option>` element inside the chapter dropdown that has the `selected` attribute. It's a neat trick—instead of looking for a heading element, we extract the title from the navigation UI.

**FictionPress: FF.net's twin:**

FictionPress.com is a sister site to FF.net—it was created for original fiction, but it uses the exact same codebase. The HTML structure is identical. In fact, the FicHub codebase handles this elegantly:

```rust
/// FictionPress scraper (site shares same structure as FF.net)
pub use FfNetScraper as FictionPressScraper;
```

That's the entire `fictionpress.rs` file. FictionPress is literally the same struct, just re-exported under a different name. The `can_handle` method checks for both domains:

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("fanfiction.net") || url.contains("fictionpress.com")
}
```

Smart, right? When two sites share the same structure, why write the same code twice? This is the DRY (Don't Repeat Yourself) principle in action. And because the registry calls `can_handle` first, there's no ambiguity—if a URL points to FictionPress, the same FF.net scraper handles it.

🧪 **Try It Yourself:** Visit a FF.net story and open the page source. Search for `#profile_top` — can you find the metadata section? Now search for `storytext` — that's where the actual story content lives. Compare the HTML structure to AO3's. Can you see why FF.net needs different CSS selectors?

## XenForo Forums (SpaceBattles, SufficientVelocity)

Now we get to the interesting case. SpaceBattles (spacebattles.com), SufficientVelocity (sufficientvelocity.com), and QuestionableQuesting (questionablequesting.com) are XenForo-based forums. They're not traditional fanfiction archives—they're general-purpose forums where people also write and share original fiction.

This creates a fundamental challenge: the sites weren't designed for fanfiction. They're designed for forum discussions. Stories are posted as forum threads, and each "chapter" is a forum post.

**How XenForo works:**

Stories are posted as forum threads. Each "chapter" is a forum post in the thread. The URL looks like:
```
https://forums.spacebattles.com/threads/story-title.12345/
```

That `12345` at the end is the thread ID. But the URL also contains a slug—the human-readable title of the thread. The regex for extracting the thread ID needs to handle this:

```rust
fn extract_thread_id(url: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"threads/.*\.(\d+)/?").ok()?;
    re.captures(url)?.get(1).map(|m| m.as_str().to_string())
}
```

The regex `threads/.*\.(\d+)/?` matches "threads/", then any characters (the slug), then a dot, then captures the digits before the optional trailing slash. The `.*` is greedy—it matches as many characters as possible, then backtracks to find the dot and digits.

**The HTML structure is very different:**

```html
<h1 class="p-title-value">Story Title by AuthorName</h1>

<article class="message" data-author="AuthorName">
  <div class="message-body">
    <div class="bbWrapper">
      <p>First post content (often the story intro or Chapter 1)...</p>
    </div>
  </div>
</article>

<article class="message" data-author="AuthorName">
  <div class="message-body">
    <div class="bbWrapper">
      <p>Second post content (often Chapter 2)...</p>
    </div>
  </div>
</article>
```

Each `article.message` is a forum post, and we treat each one as a chapter. The content lives inside `div.bbWrapper` (BBCode wrapper—XenForo uses BBCode internally).

**The author extraction:**

XenForo stores the author in two places: the `data-author` attribute on the article element, and in an `a.username` link:

```rust
let author_el = document
    .select(&Selector::parse("a.username").unwrap())
    .next();
let author = author_el
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Author".to_string());
```

The author URL needs special handling because XenForo uses relative URLs:

```rust
let author_url = author_el
    .and_then(|el| el.value().attr("href"))
    .map(|h| {
        if h.starts_with('/') {
            // Determine domain from the URL
            for domain in XENFORO_DOMAINS {
                if url.contains(domain) {
                    return format!("https://{domain}{h}");
                }
            }
        }
        h.to_string()
    })
    .unwrap_or_default();
```

This is more complex than AO3's author URL extraction because XenForo gives us a relative path like `/members/username.12345/`. We need to figure out which domain it belongs to and prepend it.

**The description extraction:**

For XenForo, the "description" is the first post's content:

```rust
let description = document
    .select(&Selector::parse("article.message-body").unwrap())
    .next()
    .map(|el| el.inner_html())
    .unwrap_or_default();
```

The first `article.message-body` is the opening post (OP) of the thread, which typically contains the story introduction or description. We use `inner_html()` to preserve any formatting.

**Pagination: XenForo's multi-page threads:**

Long XenForo threads are split across multiple pages. A 50-post thread might be spread across 5 pages (10 posts per page). The scraper currently only fetches the first page, which means it only gets the posts on that page.

The metadata always shows `chapters: 1` because we don't know the total chapter count without fetching all pages:

```rust
Ok(FicMetadata {
    // ...
    chapters: 1,
    words: 0,
    // ...
})
```

This is a known limitation. For a full implementation, the scraper would need to:
1. Parse the page navigation to find the total page count
2. Fetch each page sequentially
3. Combine posts from all pages
4. Handle rate limiting between page requests

For now, the scraper works well for threads that fit on a single page, and provides a reasonable approximation for longer threads by grabbing the first page's posts.

**Domain matching:**

XenForo scrapers need to know which domains to handle. The scraper uses a constant array:

```rust
const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];
```

And the `can_handle` method checks if the URL contains any of these domains:

```rust
fn can_handle(&self, url: &str) -> bool {
    Self::is_xenforo_url(url)
}

fn is_xenforo_url(url: &str) -> bool {
    XENFORO_DOMAINS.iter().any(|d| url.contains(d))
}
```

If you wanted to add another XenForo forum (there are hundreds out there), you'd just add its domain to the array. The scraper logic is generic enough to handle any XenForo site.

⚠️ **Watch Out:** XenForo forums often have CAPTCHAs, anti-bot measures, and require cookies for certain content. Our basic scraper won't handle authentication-protected threads. If a thread requires a login to view, our scraper will get an error or an empty page.

## Story IDs: The Universal Identifier

Every site uses some form of unique identifier for its stories. Understanding these IDs is crucial because they form the basis of our URL ID system.

| Site | URL Pattern | ID Extraction |
|------|-------------|---------------|
| AO3 | `/works/\d+` | Regex captures digits after `/works/` |
| FF.net | `/s/\d+` | Regex captures digits after `/s/` |
| XenForo | `threads/*.\d+` | Regex captures digits after last dot |
| AdultFanFiction | Numeric segments | First numeric path segment |
| HPFanFic | URL slug | Last URL segment |

Our `generate_url_id` function creates a deterministic, site-independent ID by combining the source ID with the story ID:

```rust
pub fn generate_url_id(source_id: i64, story_id: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(source_id.to_string().as_bytes());
    hasher.update(b":");
    hasher.update(story_id.as_bytes());
    let result = hasher.finalize();
    // Use first 12 hex chars for a compact but unique ID
    hex::encode(&result[..6])
}
```

The `source_id` is a numeric identifier for each site (AO3=1, FF.net=2, XenForo=3, AdultFanFiction=4, HPFanFic=5). The `story_id` is the site-specific identifier extracted from the URL. By hashing them together with a colon separator, we get a 12-character hex string that's unique across all sites.

The tests for this function are thorough:

```rust
#[test]
fn test_generate_url_id_deterministic() {
    let id1 = generate_url_id(1, "story_123");
    let id2 = generate_url_id(1, "story_123");
    assert_eq!(id1, id2);
}

#[test]
fn test_generate_url_id_different_source_id() {
    let id1 = generate_url_id(1, "story_123");
    let id2 = generate_url_id(2, "story_123");
    assert_ne!(id1, id2);
}

#[test]
fn test_generate_url_id_length() {
    let id = generate_url_id(42, "abc123");
    assert_eq!(id.len(), 12);
}

#[test]
fn test_generate_url_id_hex_chars() {
    let id = generate_url_id(7, "test_url");
    assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
}
```

The deterministic test verifies that the same inputs always produce the same output (essential for database lookups). The different-source-id test confirms that the source ID actually matters. The length and hex-char tests verify the output format.

🧪 **Try It Yourself:** Open a browser and navigate to three different fanfiction sites. Pick a story on each one. Look at the URL—can you spot the story ID in each URL? Try copying the URL into a text editor and highlighting the ID portion. That's exactly what our regex patterns are designed to extract.

## Being Polite: Robots.txt and Rate Limits

Web scraping comes with responsibilities. Just because we *can* fetch a page doesn't mean we should hammer the server with requests.

**User-Agent headers:**

Every request our scrapers make includes a User-Agent header that identifies us:

```rust
.header("User-Agent", "fichub.net/0.1.0")
```

This tells the target site who we are. It's basic etiquette—like introducing yourself before entering someone's house. Without a User-Agent, many sites will block your requests because they can't distinguish you from malicious bots.

The version number in the User-Agent (0.1.0) helps site administrators identify which version of our scraper is accessing their site. If we introduce a bug that sends too many requests, they can contact us and ask us to fix it.

**Rate limiting:**

AO3, FF.net, and other sites all have rate limits. If you make too many requests too quickly, you'll get blocked. Our scrapers don't implement rate limiting themselves (that's handled elsewhere in the system), but the architecture is designed to support it.

The `CollectionWorker` in our backend creates per-site rate limiters:

```rust
let mut rate_limiters = HashMap::new();
for fetcher in &fetchers {
    let domain = fetcher.site_domain().to_string();
    let delay = fetcher.rate_limit_delay(&config, &domain);
    rate_limiters.insert(domain, PerSiteRateLimiter::new(delay));
}
```

Each site gets its own delay configuration. AO3 might allow one request every 2 seconds, while FF.net might require 3 seconds between requests. This per-site configuration is essential because different sites have different tolerance levels.

**Timeouts:**

We set reasonable timeouts on our HTTP client:

```rust
let http_client = reqwest::Client::builder()
    .user_agent("fichub.net/0.1.0")
    .timeout(std::time::Duration::from_secs(30))
    .build()
    .expect("Failed to build HTTP client");
```

30 seconds is generous enough for slow sites but prevents our scraper from hanging indefinitely if a site is unresponsive. Without a timeout, a single slow response could block an entire async task.

**robots.txt:**

Most websites have a `robots.txt` file that tells automated tools which pages they're allowed to visit and how often. For example, AO3's robots.txt might say "don't scrape more than one page per second." Our scrapers should respect these guidelines.

In practice, we implement this through our rate limiter configuration rather than by parsing robots.txt directly. The rate limits are manually configured based on our understanding of each site's preferences.

**Error handling for blocked requests:**

Our `ScrapeError::Blocked` variant handles the case where a site refuses our request:

```rust
pub enum ScrapeError {
    NotFound,      // Story doesn't exist
    Blocked,       // Site blocked our request
    Network(String),  // HTTP/connection error
    ParseError(String),  // Couldn't parse HTML
}
```

If a site returns a 403 (Forbidden) or 429 (Too Many Requests), we map it to `ScrapeError::Blocked`. The caller can then decide whether to retry (after a delay) or give up.

⚠️ **Watch Out:** Some fanfiction sites explicitly prohibit scraping in their Terms of Service. Always check the ToS before scraping a site. AO3 has specific guidelines about automated access. Our scrapers are designed for personal use—building a public scraping service that hammers someone's server is a different matter entirely.

## The Challenge: Every Site Is Different

Here's the fundamental challenge: there's no standard way that fanfiction sites present their data. AO3 uses semantic HTML with clean class names. FF.net uses IDs and custom classes. XenForo uses its own templating system. Each site has evolved independently over years, and they all make different choices.

Consider the simple task of extracting a story title:

| Site | Selector | What it matches |
|------|----------|-----------------|
| AO3 | `h2.title.heading` | An `<h2>` with both `title` and `heading` classes |
| FF.net | `#profile_top b.xcontrast_txt` | A `<b>` inside the profile section |
| XenForo | `h1.p-title-value` | An `<h1>` with the XenForo title class |
| AdultFanFiction | `h1.story_title` | An `<h1>` with `story_title` class |
| HPFanFic | `h1` | Any `<h1>` on the page |

Each site uses a different element, different classes, and different hierarchy. There's no "one selector fits all" solution.

This is why we need a **trait-based architecture**. Instead of writing one monolithic scraper that handles everything, we define a contract (the `SiteScraper` trait) and let each site implement it differently:

```rust
#[async_trait]
pub trait SiteScraper: Send + Sync {
    fn can_handle(&self, url: &str) -> bool;
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError>;
}
```

Each scraper only needs to know about its own site's HTML structure. AO3 doesn't care about FF.net's selectors, and FF.net doesn't care about XenForo's forum layout. They all speak the same language at the trait level.

This separation of concerns is what makes the system maintainable. If AO3 redesigns its page, we only update the AO3 scraper. If we want to support a new site, we add a new scraper. The rest of the system—database, API, EPUB generation—never changes.

In the next chapter, we'll dig into how to build these scrapers using Rust's HTTP and HTML parsing tools.

---

# Chapter 12: Building Web Scrapers

## What Is Scraping?

Let's use an analogy. Imagine you walk into a library and pick up a book. You open it, find the table of contents, and read the title page. That's what web scraping is—but the library is a website, the book is a web page, and you're a Rust program.

More precisely, web scraping is the process of:
1. **Fetching** a web page (making an HTTP request)
2. **Parsing** the HTML content (reading the page structure)
3. **Extracting** specific information (pulling out the title, author, content, etc.)

In Rust, we have excellent tools for each step. The `reqwest` crate handles HTTP requests. The `scraper` crate parses HTML and lets us use CSS selectors. And our `SiteScraper` trait ties everything together.

Let's build a scraper from the ground up, step by step.

## The reqwest Crate: Making HTTP Requests

`reqwest` is the de facto HTTP client for Rust async code. It's like `fetch()` in JavaScript, but built for Rust's async/await model. It handles connection pooling, TLS, redirects, and all the details of HTTP so we can focus on what we care about: getting the page content.

**Building a client:**

We create a reusable HTTP client when the server starts. The client is expensive to create (it sets up connection pools and TLS configuration), so we make it once and share it:

```rust
let http_client = reqwest::Client::builder()
    .user_agent("fichub.net/0.1.0")
    .timeout(std::time::Duration::from_secs(30))
    .build()
    .expect("Failed to build HTTP client");
```

The `Client::builder()` pattern lets us configure:
- **User-Agent**: who we identify as (required by most sites)
- **Timeout**: how long to wait for a response (30 seconds)
- **TLS**: we use `rustls` (not OpenSSL) for HTTPS—this is a Rust-native TLS implementation
- **Features**: we enable JSON support for API calls

The `build()` method returns a `Result<Client, Error>`. We use `.expect()` here because if the client can't be built, the server can't function at all, so a panic is appropriate.

**Making a GET request:**

Every scraper request follows the same pattern. Let's trace through a real request to AO3:

```rust
let response = client
    .get(&fic_url)
    .header("User-Agent", "fichub.net/0.1.0")
    .send()
    .await
    .map_err(|e| ScrapeError::Network(e.to_string()))?;
```

Let's break this down line by line:
1. `client.get(&fic_url)` creates a GET request builder targeting the URL
2. `.header("User-Agent", "fichub.net/0.1.0")` adds our identification header
3. `.send()` actually sends the request over the network (this is the async part!)
4. `.await` suspends the current task until the response arrives
5. `.map_err(|e| ScrapeError::Network(e.to_string()))?` converts any error into our error type and propagates it

The `?` operator is Rust's error propagation shorthand. If `map_err` produces an `Err`, the function returns early with that error. If it produces an `Ok`, the inner value is unwrapped and assigned to `response`.

**Reading the response body:**

Once we have the response, we need to read its body as text (HTML):

```rust
if !response.status().is_success() {
    return Err(ScrapeError::NotFound);
}

let html = response.text().await
    .map_err(|e| ScrapeError::Network(e.to_string()))?;
```

First we check the HTTP status code using `response.status().is_success()`. This returns `true` for any 2xx status code (200 OK, 201 Created, etc.). If it's not a success, the story probably doesn't exist or has been deleted, so we return `ScrapeError::NotFound`.

Otherwise, `response.text().await` reads the entire response body as a UTF-8 string. This is where the actual HTML lives.

**Response status handling:**

Different status codes mean different things:
- **200 OK**: The story exists and we got the page
- **301/302 Redirect**: reqwest follows these automatically
- **403 Forbidden**: The site is blocking our request → `ScrapeError::Blocked`
- **404 Not Found**: The story doesn't exist → `ScrapeError::NotFound`
- **429 Too Many Requests**: We're rate-limited → `ScrapeError::Blocked`
- **500+ Server Error**: The site has a problem → `ScrapeError::Network`

Currently, our scrapers treat any non-2xx status as "not found." A more sophisticated approach would distinguish between 404 and 403, mapping them to different error variants.

⚠️ **Watch Out:** `response.text().await` reads the entire response body into memory. For a very large story with hundreds of chapters on a single page (like a full-work AO3 view), this could use significant memory. In practice, fanfiction pages rarely exceed a few megabytes, so this is fine. But if you were scraping a site that returns huge pages, you'd want to stream the response instead.

## The scraper Crate: Reading HTML with CSS Selectors

Once we have the HTML, we need to parse it and find specific elements. This is where the `scraper` crate comes in. It provides HTML parsing and CSS selector support, built on top of the `html5ever` parser and the `selectors` crate.

**Parsing HTML:**

```rust
let document = Html::parse_document(&html);
```

This takes our raw HTML string and turns it into a DOM-like tree structure that we can query. The `parse_document` function handles malformed HTML gracefully—it won't crash on unclosed tags or missing attributes.

**Using CSS selectors:**

CSS selectors are the language you use in stylesheets to target HTML elements. If you've ever written `div.content` or `#main-title` or `h2 a`, you've used CSS selectors. The `scraper` crate lets us use them to find elements in parsed HTML.

Here's a concrete example from the AO3 scraper:

```rust
let title = document
    .select(&Selector::parse("h2.title.heading").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());
```

Let's walk through this step by step:
1. `Selector::parse("h2.title.heading")` creates a selector that matches `<h2>` elements with both `title` and `heading` classes
2. `.unwrap()` is safe here because we know the selector string is valid
3. `document.select(...)` returns an iterator of all matching elements
4. `.next()` gets the first match (we expect only one)
5. `.map(|el| el.text().collect::<String>()...)` extracts all text content from the element and its children
6. `.unwrap_or_else(|| "Unknown Title".to_string())` provides a default if nothing was found

The `.text()` method returns an iterator over all text nodes in the element. We `.collect()` them into a single string. This means if the element contains nested elements (like `<h2>Story <em>Title</em></h2>`), we get "Story Title"—all the text, merged together.

**Common selector patterns in our scrapers:**

Here's a reference table of the selectors used across our scrapers:

| What we're looking for | Selector | Site |
|----------------------|----------|------|
| Story title | `h2.title.heading` | AO3 |
| Author link | `a[rel='author']` | AO3 |
| Chapter count | `dd.chapters` | AO3 |
| Word count | `dd.words` | AO3 |
| Story summary | `blockquote.userstuff` | AO3 |
| Chapter container | `div.chapter` | AO3 |
| Chapter content | `div.userstuff` | AO3 |
| Story title | `#profile_top b.xcontrast_txt` | FF.net |
| Author | `#profile_top a.xcontrast_txt` | FF.net |
| Description | `#profile_top div.xcontrast_txt` | FF.net |
| Chapter content | `div.storytext` | FF.net |
| Chapter title | `select#chap_select option[selected]` | FF.net |
| Thread title | `h1.p-title-value` | XenForo |
| Author | `a.username` | XenForo |
| Forum post | `article.message-body` | XenForo |
| Story title | `h1.story_title` | AdultFanFiction |
| Author | `a.author` | AdultFanFiction |
| Story description | `div.story_description` | AdultFanFiction |
| Story content | `div.story_content` | AdultFanFiction |
| Title | `h1` | HPFanFic |
| Author | `a[href*='author']` | HPFanFic |
| Content | `div.story-content, div.fic-content, article` | HPFanFic |

Notice the variety: ID selectors (`#profile_top`), class selectors (`.userstuff`), attribute selectors (`[rel='author']`, `[data-xutitle='word count']`), and even comma-separated fallback selectors (`div.story-content, div.fic-content, article`).

🧪 **Try It Yourself:** Open your browser's developer tools (F12 or Ctrl+Shift+I) and navigate to a fanfiction page. Use the "Inspector" or "Elements" tab to find the title element. Right-click it and select "Copy > Copy selector" to get the CSS selector. Then try typing that selector into the console with `document.querySelector('your-selector-here')`. You just did manually what our scraper does automatically!

**Selector strategies:**

When writing selectors, there's a trade-off between specificity and robustness:

- **Very specific** (e.g., `#profile_top b.xcontrast_txt`): Works perfectly until the site changes, then breaks completely
- **Very generic** (e.g., `h1`): More resilient to changes, but might match the wrong element
- **Fallback chains** (e.g., `div.story-content, div.fic-content, article`): Tries multiple specific selectors, falling back to more generic ones

Our scrapers generally use specific selectors because we're targeting known, stable sites. But the HPFanFic scraper demonstrates the fallback approach.

## The SiteScraper Trait: A Blueprint for Scrapers

In Rust, a **trait** is like a contract. It says: "Any type that implements this trait must provide these methods." It's how we achieve polymorphism—the ability to write code that works with any scraper, without knowing which specific scraper it is.

Think of it like a job description. The trait says "the applicant must be able to do X, Y, and Z." Different applicants (scrapers) implement those abilities differently, but they all satisfy the job description.

Here's our `SiteScraper` trait from `scrape/mod.rs`:

```rust
#[async_trait]
pub trait SiteScraper: Send + Sync {
    /// Returns true if this scraper can handle the given URL
    fn can_handle(&self, url: &str) -> bool;

    /// Extract metadata from a story URL
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;

    /// Fetch all chapters given metadata
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError>;

    /// Extract structured tags from a fic URL (optional, default empty).
    async fn extract_tags(&self, _client: &reqwest::Client, _url: &str) -> Result<Vec<ExtractedTag>, ScrapeError> {
        Ok(Vec::new())
    }
}
```

Let's look at each method:

**`can_handle(&self, url: &str) -> bool`**

This is the routing method. Given a URL, does this scraper know how to handle it? For AO3, it simply checks if the URL contains "archiveofourown.org":

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

For XenForo, it checks against multiple domains:

```rust
fn can_handle(&self, url: &str) -> bool {
    Self::is_xenforo_url(url)
}

fn is_xenforo_url(url: &str) -> bool {
    XENFORO_DOMAINS.iter().any(|d| url.contains(d))
}
```

The `can_handle` method is the first thing the registry calls. It's synchronous (not async) because it doesn't do any I/O—it's just string matching.

**`lookup(&self, client, url) -> Result<FicMetadata, ScrapeError>`**

This is the main extraction method. Given a story URL, it:
1. Extracts the story ID from the URL
2. Fetches the story page via HTTP
3. Parses the HTML
4. Extracts metadata (title, author, chapters, words, description, etc.)
5. Returns a `FicMetadata` struct

This is where most of the scraper's work happens. The `client` parameter is the shared HTTP client—we pass it in rather than creating one inside the scraper, which allows for connection pooling and shared configuration.

**`fetch_chapters(&self, client, meta) -> Result<Vec<Chapter>, ScrapeError>`**

Once we have metadata, we need to download the actual chapter content. This method takes the metadata (which includes the URL and chapter count) and fetches each chapter's HTML content.

The separation between `lookup` and `fetch_chapters` is intentional. You might want to look up metadata without downloading all the content (for example, to show a preview). Or you might want to fetch chapters separately (for example, to resume a failed download).

**`extract_tags(&self, client, url) -> Result<Vec<ExtractedTag>, ScrapeError>`**

This is an optional method with a default implementation that returns an empty vector. It lets scrapers that can extract structured tags (like AO3 with its rich tagging system) provide that data. Sites without rich tags simply don't override this method.

The default implementation is defined right in the trait:

```rust
async fn extract_tags(&self, _client: &reqwest::Client, _url: &str) -> Result<Vec<ExtractedTag>, ScrapeError> {
    Ok(Vec::new())
}
```

Note the underscore-prefixed parameters (`_client`, `_url`). This tells Rust "I know these parameters exist but I don't use them in this default implementation." It prevents unused-variable warnings.

**Why `Send + Sync`?**

The `Send + Sync` bounds on the trait mean that any scraper must be safe to send between threads and share references across threads. This is essential because we wrap our scraper registry in `Arc` and share it across async tasks. Without these bounds, the compiler would refuse to let us share scrapers across the `tokio` runtime.

## The FicMetadata Struct: What We Extract

Every scraper returns a `FicMetadata` struct—the standardized representation of a fanfiction story's metadata. This is the bridge between the scraper world and the database world.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,           // Deterministic ID from source_id + story_id
    pub title: String,            // "My Amazing Story"
    pub author: String,           // "AuthorName"
    pub chapters: i32,            // Number of chapters published
    pub words: i64,               // Total word count
    pub desc: String,             // Story description/summary (HTML)
    pub published: i64,           // Unix timestamp (milliseconds) when published
    pub updated: i64,             // Unix timestamp (milliseconds) when last updated
    pub status: String,           // "ongoing", "complete", "hiatus", "cancelled"
    pub source: String,           // Original URL
    pub source_id: i64,           // Numeric site identifier (1=AO3, 2=FF.net, etc.)
    pub author_id: i64,           // Database ID for the author (0 if unknown)
    pub author_url: String,       // Link to author's profile page
    pub author_local_id: String,  // Site-specific identifier for the author/story
    pub content_hash: Option<String>,  // Hash of content for change detection
    pub extra_meta: Option<String>,    // Additional metadata (JSON)
    pub raw_extended_meta: Option<String>,  // Raw extended metadata
}
```

Let's look at some design decisions:

**Why `i64` for timestamps?**

We store timestamps as Unix milliseconds (the number of milliseconds since January 1, 1970). Using `i64` gives us enough precision and range to handle any date we'll encounter. Millisecond precision is more than enough for fanfiction—we rarely need sub-second precision for publish dates.

When a scraper doesn't know the exact publish date, it uses the current time:

```rust
let now = Utc::now().timestamp_millis();
```

**Why `String` for status?**

We use simple strings for status: "ongoing", "complete", "hiatus", "cancelled". An enum would be more type-safe, but the database stores this as text, and the string representation is easier to work with across different parts of the system. The API returns strings, the EPUB generator reads strings, and the database stores strings.

**Why `Option<String>` for `content_hash`?**

The `content_hash` field is used for change detection. When we scrape a story, we can hash its content and compare it to a previously stored hash. If the hash is different, the story has been updated. This field is `Option` because we don't always compute it—sometimes we just want the metadata without doing a full content comparison.

**Why `extra_meta` and `raw_extended_meta`?**

These are escape hatches for site-specific data that doesn't fit into the standard fields. For example, AO3 has kudos, bookmarks, and hits—data that's unique to AO3 and doesn't have equivalents on other sites. We can store this as JSON in `extra_meta` without polluting the standard fields.

**The derive macros:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

- `Debug`: Lets us print the struct for debugging (e.g., `println!("{:?}", metadata)`)
- `Clone`: Lets us create copies (needed because we sometimes pass metadata to multiple functions)
- `Serialize` / `Deserialize`: Lets us convert to/from JSON (needed for API responses and database storage)

## The Chapter Struct

Chapters are simpler than metadata:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String, // HTML content
}
```

The `content` field stores HTML, not plain text. This is intentional—fanfiction often has formatting (bold, italic, blockquotes, images) that we want to preserve. When we generate EPUBs later, we'll convert this HTML to EPUB-compatible markup.

The `chapter_id` is 1-indexed (starts at 1, not 0) to match human numbering conventions.

## async_trait: Making Traits Work with Async

Rust's trait system has a limitation: trait methods can't natively be async. This is because async functions return a `Future`, and the compiler can't know what concrete future type a trait method will return at compile time.

The `async_trait` crate solves this by transforming async trait methods into regular methods that return a pinned boxed future. The `#[async_trait]` attribute macro handles the transformation:

```rust
#[async_trait]
pub trait SiteScraper: Send + Sync {
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;
}
```

Without `async_trait`, this would need to be written much more verbosely:

```rust
// Without async_trait (for illustration only):
pub trait SiteScraper: Send + Sync {
    fn lookup<'a>(
        &'a self,
        client: &'a reqwest::Client,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<FicMetadata, ScrapeError>> + Send + 'a>>;
}
```

The `async_trait` version looks and feels like a regular async method, while the manual version requires explicit lifetime annotations, `Pin<Box<dyn Future>>` types, and `Send + 'a` bounds. The attribute macro hides all this complexity.

There's a small performance cost: every async trait method call allocates a `Box` on the heap. For our scraper system, this cost is negligible compared to the network I/O. But it's worth knowing about if you're building something performance-critical.

## Error Handling in Scrapers: The ScrapeError Type

Every scraper operation can fail, and we need to know *how* it failed. Our `ScrapeError` enum covers the common cases:

```rust
#[derive(Debug)]
pub enum ScrapeError {
    NotFound,           // Story doesn't exist or was deleted
    Blocked,            // Site blocked our request
    Network(String),    // HTTP/connection error with message
    ParseError(String), // Couldn't parse the HTML with message
}
```

**Why four variants instead of one?**

Each variant represents a fundamentally different failure mode that the caller might want to handle differently:

- **NotFound**: The story was deleted or the URL is wrong. We should probably tell the user "this story doesn't exist" and not retry.
- **Blocked**: The site is rate-limiting us or blocking automated access. We should retry later, not immediately. Maybe after a 60-second delay.
- **Network**: Something went wrong with the HTTP request. Could be DNS failure, timeout, or connection refused. Maybe retry after a short delay, or maybe it's a permanent issue.
- **ParseError**: The HTML doesn't match our expectations. The site might have changed its layout. This needs developer attention, not a retry.

By distinguishing these cases, the caller can make smarter decisions about what to do next.

We also implement `Display` and `Error` for `ScrapeError` so it integrates with Rust's standard error handling:

```rust
impl std::fmt::Display for ScrapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrapeError::NotFound => write!(f, "fic not found"),
            ScrapeError::Blocked => write!(f, "blocked by site"),
            ScrapeError::Network(e) => write!(f, "network error: {}", e),
            ScrapeError::ParseError(e) => write!(f, "parse error: {}", e),
        }
    }
}

impl std::error::Error for ScrapeError {}
```

The `Display` implementation gives us human-readable error messages. The `Error` implementation lets us use `?` to propagate `ScrapeError` in functions that return `Result<T, Box<dyn Error>>`.

**The `?` operator in action:**

When a scraper method returns an error, we use the `?` operator to propagate it up the call stack:

```rust
let response = client
    .get(&fic_url)
    .header("User-Agent", "fichub.net/0.1.0")
    .send()
    .await
    .map_err(|e| ScrapeError::Network(e.to_string()))?;
```

The `.map_err(...)` converts `reqwest::Error` into our `ScrapeError::Network`, and the `?` operator returns early from the function if there's an error. This keeps the error handling clean and consistent across all scrapers.

**Why `.map_err()` instead of `?` directly?**

We can't use `?` directly because `reqwest::Error` is a different type than `ScrapeError`. The `?` operator needs a conversion (via `From` trait) or we need to do it explicitly with `.map_err()`. Using `.map_err()` is explicit and makes it clear what type of error we're wrapping.

**Test coverage for error display:**

The tests verify that our error messages are correct:

```rust
#[test]
fn test_scrape_error_display_not_found() {
    assert_eq!(format!("{}", ScrapeError::NotFound), "fic not found");
}

#[test]
fn test_scrape_error_display_network() {
    let err = ScrapeError::Network("connection refused".into());
    assert_eq!(format!("{}", err), "network error: connection refused");
}
```

This might seem like overkill, but error messages are user-facing (they appear in API responses), so it's worth verifying they're correct.

## Putting It All Together

Here's the complete flow when a user asks FicHub to scrape a story:

1. **User submits a URL** through the API (e.g., `GET /api/v0/meta?url=https://archiveofourown.org/works/123456`)
2. **Registry finds the scraper** by calling `find_scraper(url)` which checks each scraper's `can_handle` method
3. **Scraper's `lookup`** extracts the work ID, fetches the page, parses the HTML, and returns metadata
4. **Scraper's `fetch_chapters`** downloads all chapter content (one request for AO3, N requests for FF.net)
5. **Scraper's `extract_tags`** (if available) pulls structured tags
6. **Server stores everything** in the PostgreSQL database
7. **EPUB is generated** from the stored data when the user requests a download

Each step is independent, testable, and replaceable. If AO3 changes its HTML, we only need to update the AO3 scraper. If we want to support a new site, we just add a new scraper that implements the trait.

The beauty of this architecture is that the rest of the system doesn't know or care which scraper was used. The API handler calls `state.scraper_registry.lookup(&client, &url)`, and the registry handles the routing. Whether the story is from AO3, FF.net, or XenForo, the response is the same `FicMetadata` struct.

🧪 **Try It Yourself:** Think about a fanfiction site that isn't currently supported. What would its scraper need? Write down: (1) the URL pattern for stories, (2) what metadata you'd want to extract, (3) a rough idea of what CSS selectors you'd use, and (4) what the `can_handle` method would check for. This mental exercise helps you understand the scraper architecture.

---

# Chapter 13: The Scraper Registry

## What Is a Registry?

Imagine you're at a hotel, and you need to reach a specific department. You call the front desk, and the receptionist says "Let me transfer you." They look up your request, find the right department, and connect you. That's exactly what our ScraperRegistry does—it's a phone book for scrapers.

When the server receives a URL like `https://archiveofourown.org/works/123456`, it doesn't know which scraper should handle it. The registry takes that URL, checks each scraper's `can_handle` method, and routes the request to the right one.

This is a classic **Strategy Pattern** in software design. We have a collection of strategies (scrapers), each capable of handling a specific type of input (URLs from different sites), and a registry that selects the appropriate strategy at runtime.

Let's look at the code.

## The ScraperRegistry Struct

The registry is defined in `scrape/registry.rs`:

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}
```

That's it. A vector of boxed trait objects. Each element is a scraper that implements `SiteScraper`. We use `Box<dyn SiteScraper>` (dynamic dispatch) instead of concrete types because the scrapers are different types (`Ao3Scraper`, `FfNetScraper`, `XenForoScraper`, etc.) and we need to store them all in the same collection.

**Why `Vec` instead of `HashMap`?**

You might think a `HashMap<String, Box<dyn SiteScraper>>` would be more efficient—mapping domain names to scrapers directly. But the problem is that some scrapers handle multiple domains (like FF.net handling both `fanfiction.net` and `fictionpress.com`), and the matching logic isn't always a simple domain check.

Consider: FF.net's `can_handle` checks `url.contains("fanfiction.net") || url.contains("fictionpress.com")`. If we used a HashMap keyed on domain, we'd need to register the same scraper twice—once for each domain. And for sites like XenForo where we maintain a list of domains, a HashMap wouldn't capture the full matching logic.

The `can_handle` method gives us the flexibility to implement any matching logic we want. Whether it's a simple domain check, a regex match on the URL path, or something more complex, the trait-based approach handles it.

**Why `Box<dyn SiteScraper>` (dynamic dispatch)?**

In Rust, you can't have a vector of different types directly. `Vec<Ao3Scraper>` can only hold `Ao3Scraper` values. But we want to store `Ao3Scraper`, `FfNetScraper`, `XenForoScraper`, and others in the same vector.

`Box<dyn SiteScraper>` solves this. The `dyn SiteScraper` part is a **trait object**—it says "any type that implements SiteScraper." The `Box` part puts it on the heap so it has a known size (trait objects are unsized by default).

The trade-off: dynamic dispatch adds a small overhead (one extra indirection) compared to static dispatch. For our use case, this overhead is completely negligible—we're doing HTTP requests that take hundreds of milliseconds, and the dispatch overhead is nanoseconds.

## Creating the Registry

The `new()` method registers all known scrapers:

```rust
impl ScraperRegistry {
    pub fn new() -> Self {
        let mut scrapers: Vec<Box<dyn SiteScraper>> = Vec::new();
        scrapers.push(Box::new(sites::ao3::Ao3Scraper));
        scrapers.push(Box::new(sites::ffnet::FfNetScraper));
        scrapers.push(Box::new(sites::xenforo::XenForoScraper));
        scrapers.push(Box::new(sites::fictionpress::FictionPressScraper));
        scrapers.push(Box::new(sites::adultfanfiction::AdultFanFictionScraper));
        scrapers.push(Box::new(sites::hpfanfic::HpFanFicScraper));
        ScraperRegistry { scrapers }
    }
}
```

Each scraper is a **unit struct** (a struct with no fields, like `struct Ao3Scraper;`). This is a design choice that makes sense because our scrapers don't carry any state—they're stateless functions that operate on the passed-in client and URL.

Each scraper is wrapped in `Box::new(...)` to heap-allocate it, then stored as a `Box<dyn SiteScraper>`. This is how Rust achieves runtime polymorphism—we have a collection of different types that all share the same interface.

**The order matters (a little):**

The scrapers are checked in order. The first scraper that returns `true` from `can_handle` wins. In practice, this rarely matters because each scraper handles a distinct set of domains. But if you had overlapping scrapers (say, two scrapers that both claim to handle AO3), the first one registered would be used.

Currently, the FictionPress scraper is a re-export of FfNetScraper:

```rust
// In sites/fictionpress.rs:
pub use super::ffnet::FfNetScraper as FictionPressScraper;
```

This means both the FF.net scraper and the FictionPress scraper will match on `fictionpress.com` URLs (since FfNetScraper's `can_handle` checks for both domains). The FF.net scraper is registered first, so it handles FictionPress URLs. This is fine—they use the exact same code.

**The Default implementation:**

```rust
impl Default for ScraperRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

This is a Rust convention. Implementing `Default` lets you create an instance with `ScraperRegistry::default()` in addition to `ScraperRegistry::new()`. It's a small ergonomic improvement that makes the code more idiomatic.

## find_scraper: Matching a URL to the Right Scraper

The core method of the registry is `find_scraper`:

```rust
pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
    self.scrapers.iter().find(|s| s.can_handle(url))
}
```

This is a simple linear search. It iterates through all registered scrapers, calls `can_handle` on each one, and returns the first match. If no scraper can handle the URL, it returns `None`.

Let's trace through what happens with an AO3 URL:
1. Check `Ao3Scraper.can_handle("https://archiveofourown.org/works/123456")` → `true` (URL contains "archiveofourown.org")
2. Return `Some(Ao3Scraper)` immediately

And for an unknown URL:
1. Check `Ao3Scraper.can_handle("https://wattpad.com/story/12345")` → `false`
2. Check `FfNetScraper.can_handle(...)` → `false`
3. Check `XenForoScraper.can_handle(...)` → `false`
4. Check `FictionPressScraper.can_handle(...)` → `false`
5. Check `AdultFanFictionScraper.can_handle(...)` → `false`
6. Check `HpFanFicScraper.can_handle(...)` → `false`
7. Return `None`

**Is linear search fast enough?**

With six scrapers, a linear search takes at most six comparisons. Each comparison is a string `contains` check, which is essentially a substring search. This is measured in nanoseconds—completely negligible compared to the hundreds of milliseconds a network request takes.

If we had hundreds of scrapers, we might want to optimize with a HashMap or a trie-based lookup. But for six scrapers, simplicity wins.

**Return type: `Option<&Box<dyn SiteScraper>>`**

The return type looks intimidating, but let's break it down:
- `Option<...>`: Might return `Some(value)` or `None`
- `&Box<dyn SiteScraper>`: A reference to a boxed trait object

We return a *reference* rather than an owned value because we don't want to move the scraper out of the vector. We're just borrowing it for the duration of the call.

## The Convenience Methods

The registry provides two convenience methods that combine the lookup with the scraper call:

```rust
pub async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    match self.find_scraper(url) {
        Some(scraper) => scraper.lookup(client, url).await,
        None => Err(ScrapeError::NotFound),
    }
}

pub async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    match self.find_scraper(&meta.source) {
        Some(scraper) => scraper.fetch_chapters(client, meta).await,
        None => Err(ScrapeError::NotFound),
    }
}
```

The `lookup` method finds the right scraper for a URL and calls its `lookup` method. If no scraper is found, it returns `ScrapeError::NotFound`.

The `fetch_chapters` method is a bit different: it uses the `source` field from the metadata (which is the original URL) to find the scraper, not the URL the caller originally provided. This is because `fetch_chapters` might be called later, after the metadata has been stored in the database. The `source` field preserves the original URL.

**Why not just pass the scraper directly?**

You might wonder: why have the registry look up the scraper when the caller could just call the scraper directly? The answer is **decoupling**. The route handler doesn't need to know which scraper handles which site. It just says "look up this URL" and the registry handles the rest. This means:

1. Route handlers are simpler—they don't need site-specific logic
2. Adding new sites doesn't require changing route handlers
3. The registry can apply middleware (logging, metrics, rate limiting) to all scraper calls

**A small helper:**

```rust
pub fn scraper_count(&self) -> usize {
    self.scrapers.len()
}
```

This returns the number of registered scrapers, which we log at startup:

```rust
tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());
```

This is a simple but useful diagnostic. If you see "Registered 0 scrapers" at startup, something went wrong with the initialization.

## How the Server Uses the Registry

The registry is created once when the server starts and stored in the shared application state. Let's trace through the server initialization:

```rust
// In server.rs - the run() function
pub async fn run(config: Config) {
    // ... database and Redis connections ...

    // Build HTTP client
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");

    // Initialize scraper registry
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());

    // ... other initialization ...

    // Create shared state
    let state = Arc::new(AppState {
        config: config.clone(),
        db: db_pool,
        redis: redis_conn,
        http_client,
        scraper_registry,
        // ... other fields
    });

    // Build router and start serving
    let app = build_router(state).await;
    // ...
}
```

The registry is wrapped in `Arc` (Atomic Reference Counted pointer) so it can be shared across multiple async tasks without cloning the data. The registry itself lives on the heap, and `Arc` provides shared ownership with thread-safe reference counting.

The `AppState` struct includes the registry:

```rust
pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    pub http_client: reqwest::Client,
    pub scraper_registry: Arc<ScraperRegistry>,
    pub cache_semaphores: CacheSemaphores,
    pub rate_limiter: Box<dyn limiter::RateLimiter>,
    pub recommender_engine: RecommendationEngine,
    pub collection_worker: CollectionWorker,
}
```

When a route handler needs to scrape a story, it accesses the registry through the state:

```rust
// In a route handler:
let metadata = state.scraper_registry.lookup(&state.http_client, &url).await?;
let chapters = state.scraper_registry.fetch_chapters(&state.http_client, &metadata).await?;
```

The route handler doesn't need to know whether the URL is from AO3, FF.net, or XenForo. The registry handles the routing transparently.

## The Arc<ScraperRegistry> Pattern

Let's take a moment to understand why we use `Arc`. In Rust, data can be owned by one owner at a time. But our registry needs to be accessed by multiple async tasks simultaneously—different HTTP handlers might be scraping different stories at the same time.

`Arc` solves this by providing shared ownership. Multiple `Arc` pointers can point to the same data, and the data is only freed when the last `Arc` is dropped. It's reference-counted garbage collection, but thread-safe.

```rust
// When the server starts:
let scraper_registry = Arc::new(ScraperRegistry::new());

// Arc is cloned when shared to other parts of the system:
let worker_registry = scraper_registry.clone();
// Now both scraper_registry and worker_registry point to the same data
// The reference count is now 2

// When we need to use it:
state.scraper_registry.lookup(&client, &url).await?;
```

The key insight: cloning an `Arc` is cheap—it just increments an atomic counter. The underlying data isn't copied. This is why `Arc` is the standard pattern for sharing expensive-to-create resources in async Rust.

**When the last Arc is dropped:**

When all `Arc` pointers to the registry are dropped (the server shuts down, for example), the reference count reaches zero and the registry is freed. This is automatic—you don't need to manually clean up the scrapers.

⚠️ **Watch Out:** `Arc` provides shared *read* access but not shared *write* access. If you needed mutable shared state (like adding a scraper at runtime), you'd use `Arc<Mutex<T>>` or `Arc<RwLock<T>>`. Our registry is read-only after creation, so `Arc` alone is sufficient.

**Why not just pass the registry by value?**

You could theoretically pass the registry by value to each handler, but then you'd need to reconstruct it for each request. `Arc` lets us create the registry once and share it efficiently across all requests.

## The CollectionWorker Connection

The registry isn't just used by route handlers. The `CollectionWorker`—a background task that scrapes user favourites—also uses it:

```rust
pub struct CollectionWorker {
    db: PgPool,
    redis: Mutex<MultiplexedConnection>,
    client: Client,
    config: Config,
    registry: Arc<ScraperRegistry>,
    fetchers: Vec<Box<dyn SiteFetcher>>,
    rate_limiters: HashMap<String, PerSiteRateLimiterLimiter>,
}
```

The worker receives the same `Arc<ScraperRegistry>` that the server uses. This means both the HTTP handlers and the background worker share the same registry instance, with the same scrapers, and the same configuration. No duplication, no drift.

The `CollectionWorker` also maintains its own set of `SiteFetcher` objects and per-site rate limiters. These are separate from the `SiteScraper` trait—`SiteFetcher` is a different abstraction used for batch collection operations where rate limiting and progress tracking are more important.

## Adding a New Scraper: Step by Step

Let's walk through what it takes to add a new scraper to the registry. This is the most common maintenance task you'll do as a FicHub developer.

**Step 1: Create the scraper file**

Create a new file in `src/scrape/sites/`. Let's call it `newsite.rs`:

```rust
/// NewSite scraper
pub struct NewSiteScraper;

use async_trait::async_trait;
use crate::scrape::{FicMetadata, Chapter, SiteScraper, ScrapeError};
use scraper::{Html, Selector};
use chrono::Utc;

impl NewSiteScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/story/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for NewSiteScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("newsite.example.com")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let story_id = Self::extract_story_id(url)
            .ok_or_else(|| ScrapeError::ParseError("could not extract story ID".into()))?;

        let response = client
            .get(url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ScrapeError::NotFound);
        }

        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);

        let title = document
            .select(&Selector::parse("h1.title").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());

        let author = document
            .select(&Selector::parse("a.author-name").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Author".to_string());

        let url_id = crate::scrape::generate_url_id(6, &story_id);
        let now = Utc::now().timestamp_millis();

        Ok(FicMetadata {
            url_id,
            title,
            author,
            chapters: 1,
            words: 0,
            desc: String::new(),
            published: now,
            updated: now,
            status: "ongoing".to_string(),
            source: url.to_string(),
            source_id: 6,
            author_id: 0,
            author_url: String::new(),
            author_local_id: story_id,
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        let response = client
            .get(&meta.source)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);

        let content_sel = Selector::parse("div.story-content").unwrap();
        let mut chapters = Vec::new();

        for (i, div) in document.select(&content_sel).enumerate() {
            chapters.push(Chapter {
                chapter_id: (i + 1) as i32,
                title: format!("Chapter {}", i + 1),
                content: div.inner_html(),
            });
        }

        Ok(chapters)
    }
}
```

Note the source ID `6`—each site gets a unique number. Our current assignments are: 1=AO3, 2=FF.net, 3=XenForo, 4=AdultFanFiction, 5=HPFanFic. The next available ID is 6.

**Step 2: Register the module**

In `src/scrape/sites/mod.rs`, add the module:

```rust
pub mod ao3;
pub mod ffnet;
pub mod xenforo;
pub mod fictionpress;
pub mod adultfanfiction;
pub mod hpfanfic;
pub mod newsite;  // Add this line
```

**Step 3: Add to the registry**

In `src/scrape/registry.rs`, push your new scraper into the vector:

```rust
pub fn new() -> Self {
    let mut scrapers: Vec<Box<dyn SiteScraper>> = Vec::new();
    scrapers.push(Box::new(sites::ao3::Ao3Scraper));
    scrapers.push(Box::new(sites::ffnet::FfNetScraper));
    scrapers.push(Box::new(sites::xenforo::XenForoScraper));
    scrapers.push(Box::new(sites::fictionpress::FictionPressScraper));
    scrapers.push(Box::new(sites::adultfanfiction::AdultFanFictionScraper));
    scrapers.push(Box::new(sites::hpfanfic::HpFanFicScraper));
    scrapers.push(Box::new(sites::newsite::NewSiteScraper));  // Add this line
    ScraperRegistry { scrapers }
}
```

**Step 4: Test it**

Run the server and try scraping a story from the new site. Check the logs for the "Registered 7 scrapers" message.

That's it. Three steps, and your new scraper is ready to go. The architecture is designed so that adding new sites requires minimal boilerplate.

## Testing the Registry

The registry is straightforward to test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_has_scrapers() {
        let registry = ScraperRegistry::new();
        assert_eq!(registry.scraper_count(), 6);
    }

    #[test]
    fn test_find_ao3_scraper() {
        let registry = ScraperRegistry::new();
        let url = "https://archiveofourown.org/works/123456";
        assert!(registry.find_scraper(url).is_some());
    }

    #[test]
    fn test_find_ffnet_scraper() {
        let registry = ScraperRegistry::new();
        let url = "https://www.fanfiction.net/s/1234567/1/";
        assert!(registry.find_scraper(url).is_some());
    }

    #[test]
    fn test_find_xenforo_scraper() {
        let registry = ScraperRegistry::new();
        let url = "https://forums.spacebattles.com/threads/story.12345/";
        assert!(registry.find_scraper(url).is_some());
    }

    #[test]
    fn test_find_unknown_site() {
        let registry = ScraperRegistry::new();
        let url = "https://wattpad.com/story/12345";
        assert!(registry.find_scraper(url).is_none());
    }
}
```

These tests verify that all scrapers are registered and that routing works correctly. The unknown-site test is particularly important—it confirms that we don't accidentally claim to handle sites we don't support.

🧪 **Try It Yourself:** Modify the `test_registry_has_scrapers` test to check for a different number. If you've added your own scraper (from the step-by-step guide above), update the expected count. What happens if you push the same scraper twice? Try it and observe.

---

# Chapter 14: AO3 Scraper (Deep Dive)

## The Most Important Scraper

If FicHub had to pick just one site to support, it would be AO3. It's the largest, most active fanfiction archive, it has the most structured data, and its community is the most engaged. Our AO3 scraper is the most fully-featured in the codebase, and it's a great template for understanding how scrapers work.

Let's go through it line by line. Every selector, every extraction, every fallback—explained.

## The AO3 Page Structure

Before we can extract data from AO3, we need to understand what the page looks like. AO3 uses a well-organized HTML structure with semantic elements and descriptive CSS classes. This is one of the reasons it's so popular among developers—it's a pleasure to scrape.

**The work page (view_full_work=true):**

When you visit `https://archiveofourown.org/works/123456?view_full_work=true`, you see the entire story on one page. The HTML structure looks roughly like this:

```html
<div id="workskin">
  <!-- Header section with metadata -->
  <div class="preface group">
    <div class="title heading">
      <h2 class="title heading">The Title of the Story</h2>
      <h3 class="byline heading">
        <a href="/users/authorname/pseuds/authorname" rel="author">AuthorName</a>
      </h3>
    </div>

    <!-- Stats section -->
    <div class="stats group">
      <dl>
        <dt>Published:</dt>
        <dd class="published">2023-01-15</dd>
        <dt>Status:</dt>
        <dd class="status">Complete</dd>
        <dt>Chapters:</dt>
        <dd class="chapters">5 / 5</dd>
        <dt>Words:</dt>
        <dd class="words">42,156</dd>
      </dl>
    </div>

    <!-- Summary -->
    <div class="summary module">
      <blockquote class="userstuff">
        <p>This is the summary of the story.</p>
      </blockquote>
    </div>
  </div>

  <!-- Chapters -->
  <div id="chapters">
    <div class="chapter" id="chapter-1">
      <h3 class="heading">
        <span class="chapter">1</span>
        <span class="chapter-title">The Beginning</span>
      </h3>
      <div class="userstuff module">
        <p>Chapter 1 content goes here...</p>
      </div>
    </div>

    <div class="chapter" id="chapter-2">
      <h3 class="heading">
        <span class="chapter">2</span>
        <span class="chapter-title">The Middle</span>
      </h3>
      <div class="userstuff module">
        <p>Chapter 2 content goes here...</p>
      </div>
    </div>
  </div>
</div>
```

Notice the pattern: AO3 wraps content in `div.userstuff` elements. The title is in `h2.title.heading`. The author is in `a[rel='author']`. These selectors are what our scraper targets.

## CSS Selectors That Find the Metadata

Let's walk through each piece of metadata extraction in the AO3 scraper.

**Title:**

```rust
let title = document
    .select(&Selector::parse("h2.title.heading").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());
```

The selector `h2.title.heading` matches an `<h2>` element that has both the `title` and `heading` classes. AO3 uses this specific combination for the story title. The `.text().collect::<String>()` call extracts all text nodes within the element (including text inside child elements), and `.trim()` removes leading/trailing whitespace.

Why `h2` and not `h1`? AO3 uses `h1` for the site navigation and `h2` for story titles. This is a quirk of AO3's HTML—you'd only know this by examining the page source.

**Author:**

```rust
let author = document
    .select(&Selector::parse("a[rel='author']").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Author".to_string());
```

The selector `a[rel='author']` is a CSS attribute selector. It matches any `<a>` element that has `rel="author"`. This is how AO3 marks author links—they use the `rel="author"` attribute to indicate that the link points to the author's profile.

This is a robust selector because `rel="author"` is semantically meaningful. AO3 is unlikely to remove it because it's used for accessibility and SEO purposes.

**Author URL:**

```rust
let author_url = document
    .select(&Selector::parse("a[rel='author']").unwrap())
    .next()
    .and_then(|el| el.value().attr("href"))
    .map(|h| format!("{BASE_URL}{h}"))
    .unwrap_or_default();
```

This reuses the same selector as the author name, but instead of getting the text content, it gets the `href` attribute. AO3 uses relative URLs for author profiles (like `/users/authorname/pseuds/authorname`), so we prepend the base URL to make it absolute.

Note the `.and_then()` chain: it first gets the element, then gets the `href` attribute. If either step fails, the chain short-circuits to `None`.

**Description:**

```rust
let description = document
    .select(&Selector::parse("blockquote.userstuff").unwrap())
    .next()
    .map(|el| el.inner_html())
    .unwrap_or_default();
```

The summary lives in a `blockquote.userstuff` element. Note that we use `inner_html()` instead of `text()` here—we want to preserve any HTML formatting in the summary (like italic text, links, or paragraph breaks). This is important because AO3 summaries often contain rich formatting.

**Chapter count:**

```rust
let stats_text: String = document
    .select(&Selector::parse("dd.chapters").unwrap())
    .next()
    .map(|el| el.text().collect())
    .unwrap_or_default();

let chapters = if let Some(pos) = stats_text.find('/') {
    stats_text[..pos].trim().parse().unwrap_or(1)
} else {
    1
};
```

This is interesting. AO3's chapter count is displayed as "3 / 5" (3 chapters published out of 5 planned). We find the `/` character and take everything before it—the current chapter count. If there's no `/`, it's a one-shot story with a single chapter.

The `.parse().unwrap_or(1)` chain converts the string to an integer, defaulting to 1 if parsing fails. This handles edge cases like empty strings or unexpected formats.

**Word count:**

```rust
let words = document
    .select(&Selector::parse("dd.words").unwrap())
    .next()
    .and_then(|el| {
        el.text().collect::<String>().replace(',', "").parse().ok()
    })
    .unwrap_or(0);
```

The word count is in a `dd.words` element. Note the `.replace(',', "")`—AO3 formats large numbers with commas (like "42,156"), and we need to remove them before parsing as an integer. The `.parse().ok()` converts the result to `Option<i64>`, which we default to 0 if parsing fails.

**Status:**

```rust
let status_text = document
    .select(&Selector::parse("dd.status").unwrap())
    .next()
    .map(|el| el.text().collect::<String>())
    .unwrap_or_default();
let status = if status_text.contains("Complete") {
    "complete".to_string()
} else {
    "ongoing".to_string()
};
```

AO3's status field says either "Complete" or "In Progress." We normalize these to our internal format: "complete" or "ongoing." If the status element isn't found, we default to "ongoing"—a safe assumption since most stories are in progress.

**Generating the URL ID:**

```rust
let url_id = crate::scrape::generate_url_id(1, &work_id);
```

The `1` is AO3's source ID. Combined with the work ID, this generates a deterministic 12-character hex hash. This hash is used as the story's identifier across the FicHub system.

## The lookup() Function

The `lookup` method ties all these extractions together:

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract work ID from AO3 URL".into()))?;

    let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;

    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }

    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);

    // ... extract all metadata fields ...

    Ok(FicMetadata {
        url_id,
        title,
        author,
        chapters,
        words,
        desc: description,
        published: now,
        updated: now,
        status,
        source: fic_url,
        source_id: 1,
        author_id: 0,
        author_url,
        author_local_id: work_id.clone(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

The flow is:
1. Extract the work ID from the URL
2. Build the full URL with `?view_full_work=true`
3. Fetch the page
4. Check for errors
5. Parse the HTML
6. Extract each metadata field
7. Assemble and return the `FicMetadata` struct

**Note the `source` field:** We store the full URL with `?view_full_work=true` as the source. This means when `fetch_chapters` is called later, it can use this URL directly to get the full story in one request. We don't need to reconstruct the URL or add the parameter again.

**Note the `author_local_id` field:** We store the `work_id` as the `author_local_id`. This is a bit of a naming quirk—the field is used to store the site-specific identifier, which for AO3 is the work ID. We'll use this same field when fetching chapters.

## The fetch_chapters() Function

Once we have metadata, we need the actual chapter content:

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let url = format!("{BASE_URL}/works/{}?view_full_work=true", meta.author_local_id);
    let response = client
        .get(&url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;

    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);

    let chapter_sel = Selector::parse("div.chapter").unwrap();
    let title_sel = Selector::parse("h3.title").unwrap();

    let mut chapters = Vec::new();
    for (i, chapter_div) in document.select(&chapter_sel).enumerate() {
        let chapter_title = chapter_div
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {}", i + 1));

        let content = chapter_div
            .select(&Selector::parse("div.userstuff").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();

        chapters.push(Chapter {
            chapter_id: (i + 1) as i32,
            title: chapter_title,
            content,
        });
    }

    if chapters.is_empty() {
        // Fallback for one-shots: look for top-level content
        if let Some(body) = document.select(&Selector::parse("div.userstuff").unwrap()).next() {
            let content = body.inner_html();
            if !content.is_empty() {
                chapters.push(Chapter {
                    chapter_id: 1,
                    title: meta.title.clone(),
                    content,
                });
            }
        }
    }

    Ok(chapters)
}
```

**The `?view_full_work=true` trick again:**

We use the same parameter to get all chapters on one page. This means for a 20-chapter story, we make 1 HTTP request instead of 20. Huge efficiency win. The alternative—fetching each chapter separately—would be 20x slower and much more likely to trigger rate limits.

**Iterating through chapters:**

The `div.chapter` selector finds all chapter containers on the page. For each one, we extract:
- The chapter title from `h3.title` (or fall back to "Chapter N")
- The chapter content from `div.userstuff` (using `inner_html()` to preserve formatting)

The `enumerate()` method gives us the index `i`, which we use for the chapter ID (`i + 1` since we want 1-indexed chapters) and as a fallback title.

**The fallback for one-shots:**

If no `div.chapter` elements are found (common for one-shot stories), the code looks for a top-level `div.userstuff` element. This handles stories with a single chapter that aren't wrapped in chapter divs.

The fallback uses `meta.title.clone()` as the chapter title—the story title serves as the chapter title for one-shots. This makes sense because there's only one chapter, so the title is the same as the story title.

⚠️ **Watch Out:** The fallback logic checks `if chapters.is_empty()` after the main loop. This means even multi-chapter stories where the selectors don't match would fall back to treating the entire page as one chapter. It's a safety net, not a perfect solution. If AO3 changes its HTML significantly, we might get incorrect chapter counts.

## extract_tags: Getting Structured Tags

AO3 has the richest tagging system of any fanfiction site. Our `ExtractedTag` struct represents each tag:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub tag_type_id: i16,
}

impl ExtractedTag {
    pub fn fandom(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 1 } }
    pub fn character(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 2 } }
    pub fn relationship(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 3 } }
    pub fn freeform(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 4 } }
    pub fn warning(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 5 } }
    pub fn category(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 6 } }
}
```

The convenience methods (`fandom()`, `character()`, etc.) make it easy to construct tags with the right type ID. The type IDs map to database values: 1=fandom, 2=character, 3=relationship, 4=freeform, 5=warning, 6=category.

The `extract_tags` method is defined on the `SiteScraper` trait with a default implementation that returns an empty vector. AO3's scraper overrides this to extract tags from the page. The tag HTML on AO3 is organized by type, with each type in its own section:

```html
<div class="fandom tags">
  <h3 class="heading">Fandoms:</h3>
  <ul class="tags commas">
    <li><a href="/tags/Harry%20Potter">Harry Potter</a></li>
    <li><a href="/tags/Marvel">Marvel</a></li>
  </ul>
</div>

<div class="relationship tags">
  <h3 class="heading">Relationships:</h3>
  <ul class="tags commas">
    <li><a href="/tags/Harry%20Potter%2FDraco%20Malfoy">Harry/Draco</a></li>
  </ul>
</div>
```

Each tag type section uses the same HTML structure but different class names. The scraper targets each section with the appropriate selector and constructs `ExtractedTag` values with the correct type ID.

## Handling Different AO3 Page Formats

AO3 has a few page variants that our scraper needs to handle:

1. **Single chapter story**: No `div.chapter` elements, just a top-level `div.userstuff`. Handled by the fallback logic.
2. **Multi-chapter story**: Multiple `div.chapter` elements, each with content. The primary code path.
3. **Work with restricted chapters**: Some chapters might be marked as "restricted" (only visible to logged-in users). Our scraper gets an empty content div for these.
4. **Work with author notes**: Author notes appear before and after the story content, typically in `div.author` or `div.end` elements. Our current implementation doesn't extract these separately.

Our current implementation handles cases 1 and 2 well. Cases 3 and 4 are edge cases that might not work perfectly, but they don't crash—the scraper gracefully handles missing content.

**The `published` and `updated` fields:**

Notice that we set both `published` and `updated` to `Utc::now().timestamp_millis()`:

```rust
let now = Utc::now().timestamp_millis();

Ok(FicMetadata {
    // ...
    published: now,
    updated: now,
    // ...
})
```

This is a limitation. We're not currently extracting the actual publish and update dates from AO3's page. AO3 provides these in `dd.published` and `dd.updated` elements as date strings like "2023-01-15". A more complete scraper would parse these date strings into Unix timestamps using chrono's date parsing.

This is a common pattern in scraper development: start with the most important fields (title, author, chapters, words), and add refinements (exact dates, extended metadata) over time.

## Testing the AO3 Scraper

The AO3 scraper has some of the best tests in the codebase, particularly for the URL ID generation:

```rust
#[test]
fn test_generate_url_id_deterministic() {
    let id1 = generate_url_id(1, "story_123");
    let id2 = generate_url_id(1, "story_123");
    assert_eq!(id1, id2);
}

#[test]
fn test_generate_url_id_different_source_id() {
    let id1 = generate_url_id(1, "story_123");
    let id2 = generate_url_id(2, "story_123");
    assert_ne!(id1, id2);
}

#[test]
fn test_generate_url_id_different_story_id() {
    let id1 = generate_url_id(1, "story_123");
    let id2 = generate_url_id(1, "story_456");
    assert_ne!(id1, id2);
}

#[test]
fn test_generate_url_id_length() {
    let id = generate_url_id(42, "abc123");
    assert_eq!(id.len(), 12);
}

#[test]
fn test_generate_url_id_hex_chars() {
    let id = generate_url_id(7, "test_url");
    assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_generate_url_id_empty_story_id() {
    let id = generate_url_id(1, "");
    assert_eq!(id.len(), 12);
}
```

The deterministic test verifies that the same inputs always produce the same output (essential for database lookups). The different-source-id and different-story-id tests confirm that both inputs affect the output. The length and hex-char tests verify the output format.

The empty-story-id test is a boundary case—it confirms that even with an empty input, we get a valid 12-character hash. This is important for robustness.

🧪 **Try It Yourself:** Write a test for the `extract_work_id` method. Create test cases for these URLs:
- `https://archiveofourown.org/works/123456`
- `https://archiveofourown.org/works/123456/chapters/789012`
- `https://archiveofourown.org/works/123456?view_full_work=true`

What should the method return for each? (Hint: the `?view_full_work=true` parameter shouldn't affect the work ID extraction.)

---

# Chapter 15: Other Site Scrapers

## Beyond AO3: The Full Ecosystem

While AO3 is the most popular fanfiction site, it's far from the only one. FicHub supports six different scrapers, each tailored to a specific site's unique HTML structure. In this chapter, we'll tour the remaining scrapers and learn the patterns that make them work.

The key insight across all these scrapers is this: the **structure** of the scraper is always the same (extract ID, fetch page, parse HTML, extract metadata, fetch chapters), but the **specifics** change for each site. Different selectors, different URL patterns, different quirks.

## FF.net Scraper: Different HTML, Same Goal

FanFiction.net uses a completely different HTML structure than AO3. Where AO3 uses clean semantic HTML with descriptive classes, FF.net uses IDs and custom classes that are more about styling than meaning.

**Extracting the story ID:**

FF.net URLs follow the pattern `/s/NUMBER/CHAPTER/`. Our scraper extracts the story ID:

```rust
fn extract_story_id(url: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"/s/(\d+)").ok()?;
    re.captures(url)?.get(1).map(|m| m.as_str().to_string())
}
```

Simple and clean. The `/s/` prefix is FF.net's convention for "story."

**FF.net metadata extraction:**

The metadata section on FF.net uses a `#profile_top` container. Unlike AO3's descriptive class names, FF.net uses the same class (`xcontrast_txt`) for multiple element types, distinguished by the element itself:

```rust
let title = document
    .select(&Selector::parse("#profile_top b.xcontrast_txt").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());

let author = document
    .select(&Selector::parse("#profile_top a.xcontrast_txt").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Author".to_string());

let description = document
    .select(&Selector::parse("#profile_top div.xcontrast_txt").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_default();
```

Notice the selectors:
- `b.xcontrast_txt` — the title is a `<b>` (bold) element
- `a.xcontrast_txt` — the author is an `<a>` (link) element
- `div.xcontrast_txt` — the description is a `<div>` element

All share the `xcontrast_txt` class, but the element type tells us what each one represents.

**Word count and chapter count via data attributes:**

FF.net uses `data-` attributes to store specific metadata values:

```rust
let words = document
    .select(&Selector::parse("#profile_top span[data-xutitle='word count']").unwrap())
    .next()
    .and_then(|el| {
        el.text().collect::<String>().replace(',', "").parse().ok()
    })
    .unwrap_or(0);

let chapters = document
    .select(&Selector::parse("#profile_top span[data-xutitle='chapters']").unwrap())
    .next()
    .and_then(|el| {
        el.text().collect::<String>().trim().split('/').next()
            .and_then(|s| s.trim().parse().ok())
    })
    .unwrap_or(1);
```

The `data-xutitle` attribute is FF.net's custom way of marking specific data fields. The `xu` prefix is likely an internal FF.net convention. Our selectors target these attributes precisely with `[data-xutitle='word count']` and `[data-xutitle='chapters']`.

The chapter count uses the same "X / Y" format as AO3 (current / planned), so we apply the same parsing logic: split on `/`, take the first part.

**One request per chapter:**

Unlike AO3, FF.net doesn't support viewing the full work on one page. The scraper iterates through each chapter:

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    let story_id = &meta.author_local_id;

    for i in 1..=meta.chapters {
        let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);

        let chapter_sel = Selector::parse("div.storytext").unwrap();
        let content = document
            .select(&chapter_sel)
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();

        let title = document
            .select(&Selector::parse("select#chap_select option[selected]").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {i}"));

        chapters.push(Chapter {
            chapter_id: i,
            title,
            content,
        });
    }

    Ok(chapters)
}
```

The chapter content selector `div.storytext` targets FF.net's story content div. The chapter title selector `select#chap_select option[selected]` is particularly clever—it extracts the title from the currently selected option in the chapter dropdown menu, which shows the chapter name.

⚠️ **Watch Out:** FF.net's rate limits are stricter than AO3's. A 50-chapter story requires 50 HTTP requests. If you're scraping multiple stories, you need to space out requests to avoid getting blocked. Our `PerSiteRateLimiter` handles this, but it's worth understanding the constraint.

## XenForo Scraper: Forum Posts as Chapters

XenForo forums are the most different scraper in our collection. They're not purpose-built for fanfiction, so we have to adapt. Stories are posted as forum threads, and each "chapter" is a forum post.

**The domain list:**

```rust
const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];
```

These three forums are the most popular XenForo-based sites for fanfiction. SpaceBattles focuses on sci-fi and action, SufficientVelocity is more general, and QuestionableQuesting allows mature content.

**Thread ID extraction:**

XenForo URLs follow a pattern: `threads/slug.12345/`. The thread ID is the number after the last dot:

```rust
fn extract_thread_id(url: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"threads/.*\.(\d+)/?").ok()?;
    re.captures(url)?.get(1).map(|m| m.as_str().to_string())
}
```

The `.*` in the regex matches the slug (human-readable title), and `\.(\d+)` captures the numeric thread ID. The `.*` is greedy by default, so it matches as many characters as possible, then backtracks to find the dot and digits.

**Forum posts as chapters:**

The most interesting part of the XenForo scraper is how it treats forum posts as chapters:

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    let url = &meta.source;
    let response = client
        .get(url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;

    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);

    let article_sel = Selector::parse("article.message-body").unwrap();
    let mut chapter_idx = 0;

    for article in document.select(&article_sel) {
        chapter_idx += 1;
        let content = article.inner_html();
        let title = if chapter_idx == 1 {
            meta.title.clone()
        } else {
            format!("Chapter {chapter_idx}: Thread Page {chapter_idx}")
        };

        chapters.push(Chapter {
            chapter_id: chapter_idx,
            title,
            content,
        });
    }

    if chapters.is_empty() {
        return Err(ScrapeError::ParseError("no content found in XenForo thread".into()));
    }

    Ok(chapters)
}
```

Each `article.message-body` element is a forum post. The first post is treated as Chapter 1 (using the thread title), and subsequent posts get numbered chapters like "Chapter 2: Thread Page 2."

The first post is special because in XenForo, the OP (original poster) usually writes the story introduction and Chapter 1 in the first post. Subsequent posts by the same author are typically additional chapters.

**The pagination challenge:**

XenForo threads can span multiple pages. A thread with 50 posts might be spread across 5 pages (10 posts per page). Our current scraper only fetches the first page, which means it only gets the posts on that page.

This is a known limitation. To handle pagination properly, the scraper would need to:
1. Parse the page navigation to find the total page count
2. Fetch each page sequentially (with rate limiting)
3. Combine posts from all pages
4. Handle cases where non-story posts (replies from other users) are mixed in

For now, the scraper works well for threads that fit on a single page, and provides a reasonable approximation for longer threads.

**Author URL construction:**

XenForo uses relative URLs for member profiles:

```rust
let author_url = author_el
    .and_then(|el| el.value().attr("href"))
    .map(|h| {
        if h.starts_with('/') {
            for domain in XENFORO_DOMAINS {
                if url.contains(domain) {
                    return format!("https://{domain}{h}");
                }
            }
        }
        h.to_string()
    })
    .unwrap_or_default();
```

This is more complex than AO3's author URL extraction. XenForo gives us a relative path like `/members/username.12345/`, and we need to figure out which domain it belongs to by checking the original URL. We iterate through our known domains and match against the URL.

## FictionPress: The Clone

FictionPress is the simplest scraper because it doesn't exist as separate code:

```rust
// In sites/fictionpress.rs:
pub use super::ffnet::FfNetScraper as FictionPressScraper;
```

That's the entire file. FictionPress uses the exact same codebase as FF.net, so we just re-export the FF.net scraper under a different name. The `can_handle` method already checks for both domains:

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("fanfiction.net") || url.contains("fictionpress.com")
}
```

This is a great example of the DRY (Don't Repeat Yourself) principle. When two things are truly identical, don't duplicate—re-export. It also means any bug fix to the FF.net scraper automatically applies to FictionPress.

## AdultFanFiction Scraper

AdultFanFiction.org is a smaller archive with a simpler HTML structure:

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("adult-fanfiction.org") || url.contains("adultfanfiction.net")
}
```

It checks for two domain variants. The scraper uses straightforward selectors:

```rust
let title = document
    .select(&Selector::parse("h1.story_title").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());

let author = document
    .select(&Selector::parse("a.author").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Author".to_string());

let description = document
    .select(&Selector::parse("div.story_description").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_default();
```

The story ID extraction is a bit different—it finds the first numeric segment in the URL path:

```rust
let story_id = url.split('/').filter_map(|s| s.parse::<i64>().ok()).next()
    .ok_or_else(|| ScrapeError::ParseError("could not extract story ID".into()))?;
```

This is less precise than the regex-based approaches used by other scrapers, but it works for AdultFanFiction's URL structure. It splits the URL by `/`, tries to parse each segment as a number, and takes the first one that succeeds.

The chapter content uses `div.story_content`:

```rust
let content_sel = Selector::parse("div.story_content").unwrap();
let mut chapters = Vec::new();

for (i, content_div) in document.select(&content_sel).enumerate() {
    chapters.push(Chapter {
        chapter_id: (i + 1) as i32,
        title: format!("Chapter {}", i + 1),
        content: content_div.inner_html(),
    });
}
```

This is the simplest chapter extraction of all our scrapers—it just finds all `div.story_content` elements and treats each as a chapter.

## HPFanFic Scraper

The Harry Potter Fan Fiction Archive (hpfanficarchive.com / fanficauthors.net) is a niche site for HP-specific fanfiction:

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("hpfanficarchive.com") || url.contains("fanficauthors.net")
}
```

This scraper has the most generic selectors, reflecting the site's simpler and less standardized HTML structure:

```rust
let title = document
    .select(&Selector::parse("h1").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());

let author = document
    .select(&Selector::parse("a[href*='author']").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Author".to_string());
```

The `a[href*='author']` selector uses a substring match on the `href` attribute—any link whose URL contains "author" is treated as the author link. This is more flexible but also more fragile. If a non-author link happens to contain "author" in its URL, it would be selected instead.

For content, it tries multiple selectors with a comma-separated fallback:

```rust
let content_sel = Selector::parse("div.story-content, div.fic-content, article").unwrap();
```

This comma-separated selector tries three different selectors in order. If the site uses `div.story-content`, great. If it uses `div.fic-content`, that works too. If neither exists, it falls back to `article`. This defensive approach helps when a site's exact HTML structure isn't perfectly known.

For the ID, it uses the last segment of the URL:

```rust
let id = format!("hpfanfic_{}", url.split('/').last().unwrap_or("unknown"));
```

This is a different approach from the other scrapers. Instead of a numeric ID, it uses a string-based ID prefixed with the site name. The `generate_url_id` function still hashes this into a 12-character hex string.

## The Challenge of Site Changes

Here's the uncomfortable truth about web scraping: **sites change their HTML**. When AO3 redesigns its page layout, our selectors break. When FF.net updates its CSS classes, our scrapers can't find the elements they're looking for.

**How do we detect when selectors break?**

If a selector returns no results, the scraper falls back to default values:
```rust
.unwrap_or_else(|| "Unknown Title".to_string())
```

So instead of crashing, the scraper returns "Unknown Title" and an empty description. The story is still saved, but with degraded metadata. This is a graceful degradation strategy—we prefer incomplete data over no data.

**How do we know the defaults are being used?**

If you're logging scrape results, you can monitor for stories with "Unknown Title" or zero word counts. These are signals that the selectors might be broken.

**How do we fix broken selectors?**

The process is:
1. Detect that metadata is missing (title is "Unknown", chapters is 1, words is 0)
2. Visit the site manually in a browser
3. Open the browser's developer tools (F12)
4. Use the Elements tab to find where the data now lives
5. Copy the new CSS selector
6. Update the scraper code
7. Deploy the fix
8. Optionally re-scrape affected stories

This is why the `SiteScraper` trait is so valuable. When a site changes, we only need to update one scraper file. The rest of the system—database, API, EPUB generation—is unaffected. The trait boundary isolates the impact of changes.

**Preventive measures:**

Some things we can do to make scrapers more resilient:
- Use multiple fallback selectors (like HPFanFic's comma-separated selector)
- Check for data quality after scraping (is the title empty? are words count 0?)
- Log warnings when default values are used
- Monitor scrape success rates over time
- Write tests that verify selector strings are valid CSS

⚠️ **Watch Out:** If a site changes its HTML and your scraper silently returns default values, you might not notice the problem until users complain about missing data. Consider adding metrics or alerts for scrape failures. A simple "percentage of stories with non-default titles" metric can catch selector breakage early.

## Adding a New Scraper: Step by Step

Let's put everything together with a concrete walkthrough. Suppose you want to add a scraper for a new fanfiction site.

**Step 1: Research the site**

Visit the site and examine:
- URL structure: What does a story URL look like?
- Story ID: How are stories identified in the URL?
- Page structure: Where is the title, author, summary, chapters?
- Content structure: How is the story content laid out?

Use your browser's developer tools to inspect the HTML. Look for patterns: consistent class names, predictable element hierarchies, data attributes.

**Step 2: Create the scraper file**

Create a new file in `src/scrape/sites/`. Use the template we showed in Chapter 13:

```rust
pub struct NewSiteScraper;

use async_trait::async_trait;
use crate::scrape::{FicMetadata, Chapter, SiteScraper, ScrapeError};
use scraper::{Html, Selector};
use chrono::Utc;

impl NewSiteScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/story/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for NewSiteScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("newsite.example.com")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        // Extract ID, fetch page, parse HTML, extract metadata
        todo!()
    }

    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        // Fetch chapters, parse content
        todo!()
    }
}
```

**Step 3: Register the module and registry**

In `src/scrape/sites/mod.rs`:
```rust
pub mod newsite;
```

In `src/scrape/registry.rs`:
```rust
scrapers.push(Box::new(sites::newsite::NewSiteScraper));
```

**Step 4: Test it**

Run the server and try scraping a story from the new site. Check:
- Does `can_handle` return true for the right URLs?
- Does `lookup` return correct metadata?
- Does `fetch_chapters` return the content?

**Step 5: Handle edge cases**

Consider:
- What if the story doesn't exist? (Return `ScrapeError::NotFound`)
- What if the site blocks us? (Return `ScrapeError::Blocked`)
- What if the HTML is different than expected? (Return `ScrapeError::ParseError`)
- What if the story has 0 chapters? (Handle gracefully)

## Comparison: All Scrapers Side by Side

| Feature | AO3 | FF.net | XenForo | AdultFF | HPFanFic |
|---------|-----|--------|---------|---------|----------|
| ID Extraction | Regex `\d+` | Regex `\d+` | Regex after dot | Numeric segment | URL slug |
| Requests per story | 1 | N (one per chapter) | 1 (first page) | 1 | 1 |
| Tag extraction | Yes | Basic | No | No | No |
| Chapter titles | From page | From dropdown | Post number | Numbered | Numbered |
| Content selector | `div.userstuff` | `div.storytext` | `article.message-body` | `div.story_content` | Multiple fallbacks |
| Source ID | 1 | 2 | 3 | 4 | 5 |

Notice the trade-offs: AO3 has the best data quality (rich tags, chapter titles, full work view) but requires careful selector maintenance. FF.net has good data quality but requires multiple requests. XenForo has the worst data quality (no tags, limited chapter info) but is the simplest to implement.

## Wrapping Up the Scraper System

Let's zoom out and look at the full picture of what we've built across these five chapters:

1. **The `SiteScraper` trait** defines the contract every scraper must follow
2. **Individual scrapers** implement the trait for their specific site
3. **The `ScraperRegistry`** routes URLs to the right scraper
4. **The `FicMetadata` struct** standardizes what we extract from every site
5. **The `ScrapeError` enum** handles failures gracefully
6. **The `ExtractedTag` struct** lets scrapers provide structured tags
7. **The `generate_url_id` function** creates deterministic IDs across sites
8. **The `Chapter` struct** holds per-chapter content

This architecture is extensible. Adding a new site requires three things: a new scraper file, a module declaration, and a registry entry. The rest of the system—database, API, EPUB export—works automatically.

The scrapers are stateless (unit structs with no fields), which makes them easy to test and reason about. They receive the HTTP client as a parameter, which allows for connection pooling and shared configuration. They return standardized types (`FicMetadata`, `Vec<Chapter>`), which makes the rest of the system site-agnostic.

In the next part of the book, we'll use this scraper system to build the API endpoints that let users submit URLs and get back beautiful EPUB files. The scrapers are the foundation; the API is the house we build on top of it.

🧪 **Try It Yourself:** Take everything you've learned in Part 3 and imagine building a scraper for Wattpad. Write down: (1) the regex for extracting the story ID, (2) the CSS selectors for title, author, and content, (3) how you'd handle chapters (one-shot vs multi-chapter), and (4) what source ID you'd assign. You now have all the knowledge you need to build a real scraper.

---

*In Part 4, we'll explore the API layer that ties everything together.*# Part 4: Export and Caching

*The export pipeline turns scraped story data into downloadable files—EPUBs for e-readers, HTML bundles for browsers—and stores them so we never have to do the work twice. This part covers file generation, content-addressable caching, Redis-backed rate limiting, and the full end-to-end export flow.*

---

## Chapter 16: Generating EPUB Files

### What Is an EPUB?

An EPUB file is the closest thing the internet has to a universal e-book standard. It's what you send to a Kindle (with a quick conversion), a Kobo, a Nook, an iPad, or any modern reading app. And here's the thing that makes it interesting from an engineering perspective: an EPUB file is just a ZIP archive with a specific structure inside.

That's it. No magic, no proprietary binary format. A `.epub` file is a ZIP containing XHTML files, CSS stylesheets, images, and an XML manifest that tells reading software how to put it all together. The International Digital Publishing Forum (IDPF) standardized this format in 2007, and now it's maintained by the W3C as EPUB 3.

An EPUB 3 archive contains:

- `META-INF/container.xml` — tells the reader where the package document is
- `content.opf` — the Open Packaging Format document listing all files and metadata
- `toc.ncx` — navigation control for backward compatibility with EPUB 2 readers
- `stylesheet.css` — our custom styling
- `introduction.xhtml` — the story's title page with metadata
- `chapter_1.xhtml` through `chapter_N.xhtml` — one file per chapter

For FicHub, EPUB generation is the core export feature. When someone asks for a fanfiction story, we need to hand them a beautifully formatted book they can load onto their e-reader. That means taking the HTML content we scraped from Archive of Our Own, FanFiction.net, or Royal Road and wrapping it in a proper EPUB structure with metadata, navigation, and styling.

Why EPUB over other formats? Because it's the standard. Amazon's MOBI format is proprietary. PDF doesn't adapt to different screen sizes. Plain HTML lacks navigation and metadata support. EPUB gives readers everything they need in a format every device understands.

### Why EPUB Over Other Formats?

Before diving into the implementation, it's worth understanding why FicHub chose EPUB as the primary export format. Each format has trade-offs:

- **EPUB**: Open standard, works on virtually every e-reader and reading app, supports metadata and navigation, renders with the reader's own styling. This is the gold standard for long-form reading.
- **PDF**: Fixed layout, doesn't adapt to screen sizes, poor accessibility. Good for print but terrible for reading on phones.
- **MOBI/AZW3**: Amazon's proprietary format. Requires conversion from EPUB. No reason to generate natively.
- **HTML**: Universal but lacks the navigation structure and metadata support that EPUB provides.
- **Plain text**: Loses all formatting. No chapter navigation, no metadata, no styling.

EPUB wins because it's the standard that every device supports. If a user doesn't have an e-reader, the HTML bundle serves as a fallback. By generating both EPUB and HTML, FicHub covers all reading scenarios.

### The epub-builder Crate: Making EPUBs in Rust

Rather than building the ZIP archive and XML manifests by hand, we use the `epub-builder` crate. It handles all the internal EPUB plumbing—the OPF package document, the NCX navigation file, the container.xml—and gives us a clean API to work with.

Here's the dependency in `Cargo.toml`:

```toml
epub-builder = "0.7"
```

The core API follows a builder pattern. You create an `EpubBuilder` with a zip library, set metadata, add content pages, and generate the file. The `ZipLibrary` type tells `epub-builder` how to create the ZIP archive—there are different backends available, but `ZipLibrary::new()` uses the standard `zip` crate.

Here's the dependency list for the full export system:

```toml
epub-builder = "0.7"      # EPUB generation
zip = "0.6"                # ZIP compression for HTML bundles
md5 = "0.7"                # Content hashing
uuid = "1.0"               # Unique workspace identifiers
chrono = "0.4"             # Timestamp formatting
hex = "0.4"                # Hash encoding
```

The `md5` and `uuid` crates are also used elsewhere in the application (for API keys and content identification), so they're shared dependencies. The `zip` crate is used both by `epub-builder` (internally) and by the HTML bundle generator (directly).

Let's look at the full signature of our EPUB creation function:

```rust
use std::fs;
use std::path::{Path, PathBuf};

use epub_builder::{EpubBuilder, EpubContent, ZipLibrary};
use md5::{Digest, Md5};
use uuid::Uuid;

use crate::export::ExportError;
use crate::scrape::{Chapter, FicMetadata};

/// Generate an EPUB file for the given fic metadata and chapters.
///
/// Creates a UUID-named subdirectory inside `tmp_dir`, writes the EPUB there,
/// computes its MD5 hash, and returns `(path_to_epub, md5_hex)`.
pub async fn create_epub(
    meta: &FicMetadata,
    chapters: &[Chapter],
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    // ... implementation ...
}
```

Notice the function is `async`. While the EPUB generation itself is synchronous (CPU-bound file I/O), making it async allows the runtime to yield to other tasks during disk writes, and keeps the interface consistent with the rest of the async application.

### Creating a Temporary Directory with UUID

Every export starts with a clean workspace. We generate a UUID and create a temporary directory inside our configured `tmp_dir`:

```rust
let uuid = Uuid::new_v4();
let work_dir = tmp_dir.join(uuid.to_string());
fs::create_dir_all(&work_dir)?;
```

Why UUID? Because when ten people request the same story at once (and they will—we've seen it happen with popular updates), each export needs its own isolated workspace. The UUID guarantees uniqueness without any coordination. No locks, no file existence checks, no race conditions.

The temporary directory structure looks like:

```
/tmp/fichub/
  a1b2c3d4-e5f6-7890-abcd-ef1234567890/
    output.epub
  b2c3d4e5-f6a7-8901-bcde-f12345678901/
    output.epub
```

Each UUID-named directory contains exactly one EPUB file. After the file is generated, it's moved to the cache and the temporary directory is cleaned up (or left for the OS to clean later).

### Setting Metadata: Title, Author, Language

An EPUB without metadata is like a book without a cover. The `epub-builder` crate lets us set metadata fields that e-readers use to display story information in libraries:

```rust
let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

builder.metadata("title", &meta.title)?;
builder.metadata("author", &meta.author)?;
builder.metadata("lang", "en")?;
builder.metadata("description", &meta.desc)?;
```

The `lang` field is required by the EPUB specification. Without it, some reading apps refuse to open the file or default to whatever language their OS uses. We default to English, but the scraper could be extended to detect the language from the content.

The `description` field is interesting—we set it to the story's description from the source site. Some reading apps show this in search results or on the book's info page.

The `FicMetadata` struct provides all the fields we need:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,       // Unique identifier for the story
    pub title: String,        // Story title
    pub author: String,       // Author name
    pub chapters: i32,        // Number of chapters
    pub words: i64,           // Total word count
    pub desc: String,         // Story description (HTML)
    pub published: i64,       // Publication date (unix millis)
    pub updated: i64,         // Last update date (unix millis)
    pub status: String,       // "ongoing", "complete", "hiatus", "cancelled"
    pub source: String,       // Original URL
    pub source_id: i64,       // Numeric ID of the source site
    pub author_id: i64,       // Numeric ID of the author
    pub author_url: String,   // Author's profile URL
    pub author_local_id: String, // Author's ID on the source site
    pub content_hash: Option<String>, // Hash of the story content
    pub extra_meta: Option<String>,   // Additional metadata
    pub raw_extended_meta: Option<String>, // Raw extended metadata
}
```

### Adding Inline CSS

Every chapter needs styling. Rather than including a separate CSS file (which adds complexity and risks broken file references), we inline it:

```rust
let css = concat!(
    "body{font-family:serif;line-height:1.5;}",
    "h2{text-align:center;}",
    "p{margin:0.5em 0;}"
);
builder.stylesheet(css.as_bytes())?;
```

The `concat!` macro joins the three string literals at compile time, producing a single string. The `stylesheet()` method in `epub-builder` writes this as a file named `stylesheet.css` inside the EPUB and automatically links it to all content pages via `<link rel="stylesheet">`.

This gives readers a clean, readable layout: serif fonts for that bookish feel, centered chapter titles, and comfortable spacing between paragraphs. It's minimal, but it works across all reading apps—from Kindle's serif defaults to iBooks' more modern look.

### The Introduction Page: Showing Story Info

Before the first chapter, we add an introduction page that shows the story's metadata. This is the "title page" of our e-book:

```rust
let intro_html = format!(
    r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>Introduction</title>
    <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
    <h1>{title}</h1>
    <h2>by {author}</h2>
    <table>
        <tr><td>Words:</td><td>{words}</td></tr>
        <tr><td>Chapters:</td><td>{chapters}</td></tr>
        <tr><td>Status:</td><td>{status}</td></tr>
        <tr><td>Published:</td><td>{published}</td></tr>
        <tr><td>Updated:</td><td>{updated}</td></tr>
    </table>
    <hr/>
    <p>{desc}</p>
</body>
</html>"#,
    title = escape_html(&meta.title),
    author = escape_html(&meta.author),
    words = meta.words,
    chapters = meta.chapters,
    status = escape_html(&meta.status),
    published = format_timestamp(meta.published),
    updated = format_timestamp(meta.updated),
    desc = escape_html(&meta.desc),
);

builder.add_content(
    EpubContent::new("introduction.xhtml", intro_html.as_bytes())
        .title("Introduction"),
)?;
```

Notice two important things here.

First, we're using `escape_html()` on user-provided strings like the title and author. This prevents XSS attacks and EPUB parsing errors from special characters in story metadata. A story titled "He Said <she> & 'went' to the Shop" would break an XHTML document if the `<`, `>`, `&`, and `'` characters weren't escaped.

Second, we use `format_timestamp()` to convert Unix milliseconds to a human-readable date. The source sites store dates as Unix timestamps in milliseconds, but readers want to see "2024-03-15" not "1710460800000".

Here are those helper functions:

```rust
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn format_timestamp(unix_millis: i64) -> String {
    use chrono::DateTime;

    let secs = unix_millis / 1000;
    let nsecs = ((unix_millis % 1000) * 1_000_000) as u32;

    DateTime::from_timestamp(secs, nsecs)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}
```

The `escape_html` function replaces the five characters that have special meaning in HTML/XML. The order matters: we replace `&` first because the other replacements introduce `&` characters (like `&amp;`). If we replaced `<` first, then replaced `&`, we'd double-escape the ampersand.

### Adding Chapters as Separate XHTML Files

Each chapter becomes its own XHTML file inside the EPUB. This is how e-readers handle navigation—each chapter is a separate "page" that readers can flip through:

```rust
for chapter in chapters {
    let chapter_html = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{title}</title>
    <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
    <h2>{title}</h2>
    {content}
</body>
</html>"#,
        title = escape_html(&chapter.title),
        content = chapter.content,
    );

    let filename = format!("chapter_{}.xhtml", chapter.chapter_id);
    builder.add_content(
        EpubContent::new(filename.as_str(), chapter_html.as_bytes())
            .title(&chapter.title),
    )?;
}
```

The `chapter_id` comes from the scraper and ensures chapters are uniquely named. The `EpubContent::new()` call takes a filename and the HTML content as bytes. The `.title()` call sets what the e-reader shows in its table of contents.

Important: the `chapter.content` field is already HTML from the scraper. We don't escape it because the scraper has already ensured it's valid HTML (with proper paragraph tags, line breaks, etc.). If we escaped it, the HTML tags would show up as literal text in the reader.

The `Chapter` struct is simple:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,   // Sequential chapter number (1, 2, 3, ...)
    pub title: String,     // Chapter title
    pub content: String,   // HTML content of the chapter
}
```

The `chapter_id` is typically the chapter number as assigned by the source site. This ensures chapters maintain their original order and numbering.

### The Spine: The Order of Chapters

You might wonder: how does the EPUB reader know which chapter comes first? That's what the "spine" is for. In EPUB terminology, the spine defines the reading order of all the content documents. With `epub-builder`, this happens automatically—the spine follows the order in which you call `add_content()`.

That's why we add the introduction first, then loop through chapters in order. If we added them out of order, the reader would present them in a jumbled mess.

Under the hood, `epub-builder` builds the OPF manifest like this:

```xml
<spine toc="ncx">
  <itemref idref="introduction"/>
  <itemref idref="chapter_1"/>
  <itemref idref="chapter_2"/>
  <!-- ... more chapters ... -->
</spine>
```

Each `<itemref>` points to a content document. The reader processes them in order. If a reader supports "go to next chapter," it follows this spine.

### Writing the EPUB File

After adding all the content, we write the EPUB to disk:

```rust
let epub_path = work_dir.join("output.epub");
let file = fs::File::create(&epub_path)?;
builder.generate(file)?;
```

The `generate()` method does all the heavy lifting: it creates the EPUB's internal ZIP structure, writes the OPF manifest, generates the NCX navigation document, and packages everything together. The resulting `output.epub` is a valid EPUB 3 file that any compliant reader can open.

The output file is always named `output.epub`—the actual identity comes from its hash and the cache path. The filename is irrelevant once we move it to the cache.

### Computing the MD5 Hash

After generating the file, we compute an MD5 hash. This hash serves two purposes: it's a content-addressable identifier for the cache, and it lets us detect if the content has changed:

```rust
let epub_data = fs::read(&epub_path)?;
let md5_hex = Md5::digest(&epub_data)
    .iter()
    .map(|b| format!("{:02x}", b))
    .collect::<String>();

Ok((epub_path, md5_hex))
```

The `Md5::digest()` function returns a 16-byte array. We format each byte as two lowercase hex characters, producing a 32-character string like `ef92b24f0e9a8d3c12345678abcdef01`.

MD5 isn't cryptographically secure (there are known collision attacks), but it's perfectly fine for content-addressable caching. We don't need collision resistance—we just need a fast way to identify identical content. If two different stories ever produced the same MD5 hash (astronomically unlikely), the worst case is a cache collision, which the hash verification in the download handler would catch.

The function returns both the path to the EPUB and its hash. The caller will use the hash for cache storage and to give the client a download URL with integrity verification.

### The ExportError Type

All export functions return `Result<T, ExportError>`. This error enum wraps the various failure modes:

```rust
#[derive(Debug)]
pub enum ExportError {
    IoError(String),       // File system errors
    TemplateError(String), // Template formatting errors
    EpubError(String),     // epub-builder errors
    CalibreError(String),  // Calibre conversion errors
    ZipError(String),      // ZIP archive errors
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::IoError(e) => write!(f, "IO error: {e}"),
            ExportError::TemplateError(e) => write!(f, "template error: {e}"),
            ExportError::EpubError(e) => write!(f, "EPUB error: {e}"),
            ExportError::CalibreError(e) => write!(f, "Calibre error: {e}"),
            ExportError::ZipError(e) => write!(f, "zip error: {e}"),
        }
    }
}

impl std::error::Error for ExportError {}
```

The `From` implementations let us use the `?` operator to automatically convert library errors into our error type:

```rust
impl From<std::io::Error> for ExportError {
    fn from(e: std::io::Error) -> Self {
        ExportError::IoError(e.to_string())
    }
}

impl From<epub_builder::Error> for ExportError {
    fn from(e: epub_builder::Error) -> Self {
        ExportError::EpubError(e.to_string())
    }
}

impl From<zip::result::ZipError> for ExportError {
    fn from(e: zip::result::ZipError) -> Self {
        ExportError::ZipError(e.to_string())
    }
}
```

This pattern keeps the code clean—`fs::File::create(&epub_path)?` automatically wraps IO errors without any extra code.

### Version Tracking

The export module also tracks versions. Each export format has a version number that's used for cache invalidation:

```rust
pub fn etype_versions() -> HashMap<&'static str, i32> {
    let mut m = HashMap::new();
    m.insert(ETYPE_EPUB, 1);
    m.insert(ETYPE_HTML, 1);
    m.insert(ETYPE_MOBI, 0);
    m.insert(ETYPE_PDF, 0);
    m
}

pub fn compute_version(export_version: i32, etype_version: i32, fic_version_bump: i32) -> i32 {
    export_version + etype_version + fic_version_bump
}
```

The total version is the sum of the export module version, the format-specific version, and any content-level version bumps. If we change the EPUB template (fix a CSS bug, add a new metadata field), we bump `ETYPE_EPUB` from 1 to 2. This invalidates all cached EPUBs without deleting them—they just won't match the new version number.

MOBI and PDF are at version 0 because they're generated by converting from EPUB using Calibre (covered in `convert.rs`). Their templates are the EPUB template—when it changes, MOBI and PDF are regenerated automatically because they depend on the EPUB hash.

### Converting EPUBs to Other Formats

FicHub doesn't just generate EPUBs—it can convert them to MOBI and PDF using Calibre's `ebook-convert` tool. This is handled by `src/export/convert.rs`:

```rust
const CONVERT_TIMEOUT_SECS: u64 = 300;

pub async fn convert_epub(
    epub_path: &Path,
    output_format: &str,
    calibre_container: &str,
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    let uuid = Uuid::new_v4();
    let work_dir = tmp_dir.join(uuid.to_string());
    fs::create_dir_all(&work_dir)?;

    let output_filename = format!("output.{output_format}");
    let output_path = work_dir.join(&output_filename);

    // First attempt
    let first = run_conversion(epub_path, &output_path, calibre_container).await;

    match first {
        Ok(()) => {}
        Err(e) => {
            // Retry once on failure
            tracing::warn!("First conversion attempt failed: {e}. Retrying once ...");
            run_conversion(epub_path, &output_path, calibre_container)
                .await
                .map_err(|retry_err| {
                    ExportError::CalibreError(format!(
                        "Calibre conversion failed after retry: {retry_err}"
                    ))
                })?;
        }
    }

    // Verify output exists
    if !output_path.exists() {
        return Err(ExportError::CalibreError(format!(
            "Output file was not created: {}",
            output_path.display()
        )));
    }

    // Compute hash
    let output_data = fs::read(&output_path)?;
    let md5_hex = Md5::digest(&output_data)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    Ok((output_path, md5_hex))
}
```

Calibre can run either directly on the host or inside a Docker container. The `calibre_container` parameter controls this:

```rust
async fn run_conversion(
    epub_path: &Path,
    output_path: &Path,
    calibre_container: &str,
) -> Result<(), ExportError> {
    let timeout_dur = Duration::from_secs(CONVERT_TIMEOUT_SECS);

    if calibre_container.is_empty() {
        run_direct(epub_path, output_path, timeout_dur).await
    } else {
        run_docker(calibre_container, epub_path, output_path, timeout_dur).await
    }
}
```

The Docker approach is useful when Calibre isn't installed on the host system but is available as a container. The conversion is wrapped in a 300-second timeout to prevent hung processes from blocking the export pipeline. The one-retry pattern handles transient Calibre errors—sometimes the first attempt fails due to temporary resource constraints but succeeds on the second try.

### CSS Design Philosophy

The CSS in both the EPUB and HTML bundle follows a deliberate design philosophy:

1. **Serif fonts for reading**: Georgia (HTML) and generic serif (EPUB) are chosen for readability. Sans-serif fonts are easier to scan but harder to read for long periods.

2. **Generous line height**: 1.5-1.7 line height reduces eye strain. The default in most browsers is 1.2, which is too tight for long-form reading.

3. **Centered layout with max-width**: 800px is the sweet spot for line length. Studies show 50-75 characters per line is optimal for reading comprehension. 800px at 16px font size gives roughly 70-80 characters.

4. **Minimal decoration**: No background images, no gradients, no animations. The content is the star. The styling exists only to make the content comfortable to read.

5. **High contrast**: #333 text on #fafafa background is easier on the eyes than pure black on pure white. The contrast ratio is still well above WCAG accessibility requirements.

These choices aren't arbitrary—they're based on typographic research about what makes text comfortable to read for extended periods.

> 🧪 **Try It Yourself**: Take any HTML file and create a minimal EPUB with `epub-builder`. Set the metadata, add a single chapter, and write it out. Rename the `.epub` to `.zip` and open it—you'll see the internal structure: a `META-INF` directory, `content.opf`, and your XHTML files. Try removing `stylesheet.css` and see if the EPUB still opens (it should, but without styling).

> ⚠️ **Watch Out**: The `epub-builder` crate doesn't validate your HTML. If you pass in broken XHTML (unclosed tags, invalid entities), the EPUB might generate successfully but fail to open on certain readers. Always escape user-provided content with `escape_html()` and trust that the scraper provides valid HTML for chapter content.

---

## Chapter 17: HTML Bundles

The HTML bundle generator (`src/export/html_bundle.rs`) builds a self-contained HTML file with all chapters, styled with CSS for comfortable reading, and packaged in a ZIP. It follows the same UUID-workspace pattern as the EPUB generator. While EPUB is the gold standard for dedicated e-readers, the HTML bundle is the universal fallback—readable on any device with a web browser.

### What Is an HTML Bundle?

Not everyone wants an EPUB. Some users want to read a story right in their browser, or they want a single portable file they can open on any computer without special software. That's what the HTML bundle is for.

An HTML bundle is exactly what it sounds like: a complete, self-contained HTML file with all the chapters, styling, and navigation baked in. We package it into a ZIP file so it downloads as a single file and takes up less space on disk.

The HTML bundle is simpler than EPUB in some ways (no XML manifests, no navigation documents, no OPF packaging) but more complex in others (we need to build a complete, well-styled web page from scratch). Where EPUB delegates rendering to the reading app, the HTML bundle has to provide all the styling and layout itself.

### Building a Complete HTML Document

Let's walk through the `create_html_bundle` function:

```rust
pub async fn create_html_bundle(
    meta: &FicMetadata,
    chapters: &[Chapter],
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    // Unique workspace
    let uuid = Uuid::new_v4();
    let work_dir = tmp_dir.join(uuid.to_string());
    fs::create_dir_all(&work_dir)?;
```

Same pattern as the EPUB: create a UUID-named workspace. This consistency makes the code easier to understand and maintain. Both export functions follow the same lifecycle: create workspace, build output, compute hash, return results.

### Adding a Table of Contents with Links

The HTML bundle builds navigation and content in a single loop, but they go into different parts of the page. The navigation goes at the top; the full chapter content goes below. This separation is important—it lets us put the navigation in a fixed, easy-to-access location while the content scrolls freely:

```rust
let mut chapters_nav = String::new();
let mut chapters_content = String::new();

for chapter in chapters {
    chapters_nav.push_str(&format!(
        r##"<li><a href="#ch{ch}">{title}</a></li>"##,
        ch = chapter.chapter_id,
        title = escape_html(&chapter.title),
    ));

    chapters_content.push_str(&format!(
        r#"<h2 id="ch{ch}">{title}</h2>
{content}"#,
        ch = chapter.chapter_id,
        title = escape_html(&chapter.title),
        content = chapter.content,
    ));
}
```

Each chapter gets an anchor (`id="ch{ch}"`) that the table of contents links to. Click "Chapter 7" in the nav, and the browser jumps right to it. The `r##"..."##` syntax (a raw string with a delimiter) is used for the navigation links because the `#` in `href="#ch1"` would normally need escaping—but in raw strings, it's just a character.

The two-column layout for navigation uses CSS `columns: 2`, which automatically flows the list items across two columns. For a story with 20 chapters, this gives a compact navigation block at the top of the page.

### The Full HTML Structure

The complete HTML document includes styling, metadata, a chapter navigation section, and all the chapter content. It's designed to look good on both desktop and mobile:

```rust
let html = format!(
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
    <title>{title} — {author}</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: Georgia, 'Times New Roman', serif;
            line-height: 1.7;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            background: #fafafa;
        }}
        h1 {{ text-align: center; margin: 1.5em 0 0.3em; font-size: 1.8em; }}
        h2 {{
            text-align: center;
            margin: 1.5em 0 0.5em;
            font-size: 1.4em;
            border-bottom: 1px solid #ddd;
            padding-bottom: 0.3em;
        }}
        .meta {{ text-align: center; color: #666; margin-bottom: 2em; font-size: 0.95em; }}
        .meta td {{ padding: 2px 8px; }}
        .desc {{
            margin: 1em 0;
            padding: 1em;
            background: #fff;
            border-radius: 4px;
            border: 1px solid #eee;
        }}
        .nav {{
            background: #fff;
            border: 1px solid #ddd;
            border-radius: 4px;
            padding: 1em;
            margin: 1.5em 0;
        }}
        .nav h3 {{ margin-bottom: 0.5em; }}
        .nav ul {{ list-style: none; columns: 2; }}
        .nav li {{ padding: 2px 0; }}
        .nav a {{ color: #1a5276; text-decoration: none; }}
        .nav a:hover {{ text-decoration: underline; }}
        .content p {{ margin: 0.5em 0; text-indent: 1.5em; }}
        .content p:first-of-type {{ text-indent: 0; }}
        hr {{ border: none; border-top: 1px solid #ddd; margin: 2em 0; }}
        .footer {{ text-align: center; color: #999; font-size: 0.85em; margin: 3em 0; }}
        a.back-to-top {{ display: block; text-align: right; font-size: 0.85em; color: #1a5276; }}
    </style>
</head>
<body>
    <h1>{title}</h1>
    <div class="meta">
        <p>by <strong>{author}</strong></p>
        <table align="center">
            <tr><td>Words:</td><td>{words}</td></tr>
            <tr><td>Chapters:</td><td>{chapters}</td></tr>
            <tr><td>Status:</td><td>{status}</td></tr>
            <tr><td>Published:</td><td>{published}</td></tr>
            <tr><td>Updated:</td><td>{updated}</td></tr>
        </table>
    </div>
    <div class="desc">{desc_escaped}</div>
    <hr/>
    <div class="nav">
        <h3>Chapter Navigation</h3>
        <ul>{nav}</ul>
    </div>
    <hr/>
    <div class="content">{content}</div>
    <hr/>
    <div class="footer">
        <p>Generated by fICHub — {source}</p>
    </div>
</body>
</html>"#,
    title = escape_html(&meta.title),
    author = escape_html(&meta.author),
    words = meta.words,
    chapters = meta.chapters,
    status = escape_html(&meta.status),
    published = format_timestamp(meta.published),
    updated = format_timestamp(meta.updated),
    desc_escaped = escape_html(&meta.desc),
    nav = chapters_nav,
    content = chapters_content,
    source = escape_html(&meta.source),
);
```

Let's break down the styling decisions:

- **`max-width: 800px`** — Limits line length for comfortable reading. Too-wide text is hard to read.
- **`font-family: Georgia`** — A serif font that's available on virtually every system. It's the safe default for reading-focused pages.
- **`line-height: 1.7`** — Generous line spacing reduces eye strain during long reading sessions.
- **`background: #fafafa`** — Off-white is easier on the eyes than pure white, especially at night.
- **`text-indent: 1.5em`** — Paragraph indentation follows traditional book formatting. The first paragraph after a heading isn't indented.
- **`border-bottom` on h2** — Visual separation between chapters without taking up too much vertical space.

The `viewport` meta tag ensures the page looks good on mobile devices. Without it, mobile browsers would render the page at desktop width and zoom out.

The `.desc` section uses a card-style design (white background, border, border-radius) to visually separate the story description from the content. This creates a clear hierarchy: title → metadata → description → navigation → chapters.

### The zip Crate: Packaging It Up

A raw HTML file would work, but ZIP compression significantly reduces file size (typically 60-80% for text-heavy content). Plus, the `.zip` extension is universally recognized:

```rust
use std::io::Write;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

// Write the HTML file first
let html_path = work_dir.join("index.html");
fs::write(&html_path, &html)?;

// Bundle into ZIP
let zip_path = work_dir.join("bundle.zip");
let zip_file = fs::File::create(&zip_path)
    .map_err(|e| ExportError::IoError(e.to_string()))?;
let mut zip = ZipWriter::new(zip_file);

let options = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Deflated)
    .unix_permissions(0o644);

zip.start_file("index.html", options)
    .map_err(|e| ExportError::ZipError(e.to_string()))?;
zip.write_all(html.as_bytes())
    .map_err(|e| ExportError::ZipError(e.to_string()))?;

zip.finish()
    .map_err(|e| ExportError::ZipError(e.to_string()))?;
```

The `Deflated` compression method is a good balance between speed and ratio. We set Unix permissions to `0o644` (readable by everyone, writable by owner) so the file behaves correctly when extracted on Unix systems.

Why ZIP the HTML? A 500KB HTML file might compress to 150KB in a ZIP. For popular stories with hundreds of thousands of words, the savings are significant. And since the cache stores the ZIP, we save disk space too.

The ZIP contains a single file: `index.html`. This is different from the EPUB, which contains many files. The simplicity is intentional—it keeps the download small and the extraction trivial.

### Computing the Hash

Same pattern as the EPUB—we read the ZIP file and compute its MD5:

```rust
let zip_data = fs::read(&zip_path)?;
let md5_hex = Md5::digest(&zip_data)
    .iter()
    .map(|b| format!("{:02x}", b))
    .collect::<String>();

Ok((zip_path, md5_hex))
```

The hash is computed over the ZIP file, not the HTML. This means if we change the ZIP compression settings but keep the HTML identical, the hash would change. That's fine—the cache treats each format independently.

### The escape_html and format_timestamp Helpers

The HTML bundle uses the same helper functions as the EPUB generator:

```rust
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn format_timestamp(unix_millis: i64) -> String {
    use chrono::DateTime;

    let secs = unix_millis / 1000;
    let nsecs = ((unix_millis % 1000) * 1_000_000) as u32;

    DateTime::from_timestamp(secs, nsecs)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}
```

These are duplicated between the EPUB and HTML modules. In a larger codebase, you might extract them to a shared utility module. For now, the duplication is acceptable—it keeps each module self-contained and avoids tight coupling.

### Why the Double-Curly-Brace Syntax?

You might notice `{{ box-sizing: border-box; }}` in the Rust format string. This is a quirk of Rust's `format!()` macro: curly braces `{` and `}` are reserved for interpolation. To include literal braces in the output, you must double them: `{{` produces `{`, and `}}` produces `}`.

The CSS for the HTML bundle is significantly more elaborate than the EPUB stylesheet because:
1. EPUB styling is interpreted by the reading app—our CSS is a suggestion, not a command.
2. HTML styling is rendered directly by the browser—we control the full presentation.
3. The HTML bundle needs to work on mobile, desktop, and print—the CSS must handle all contexts.

### Responsive Design Considerations

The HTML bundle includes several responsive design patterns:

- **`max-width: 800px`** prevents lines from becoming too long on wide screens
- **`margin: 0 auto`** centers the content container
- **`padding: 20px`** provides breathing room on narrow screens
- **`meta viewport` tag** ensures mobile browsers don't zoom out to fit desktop width
- **Two-column navigation** uses CSS `columns: 2` which automatically reflows to one column on narrow screens

These patterns ensure the story is readable on a 4-inch phone, a 10-inch tablet, and a 27-inch monitor without any JavaScript.

### Performance Considerations

For very long stories (100+ chapters, millions of words), the HTML bundle can become quite large. A 2-million-word story might produce a 10MB HTML file. Compression typically reduces this to 2-3MB in the ZIP, but the uncompressed HTML still needs to be rendered by the browser.

Modern browsers handle large HTML files well, but there are limits. If a story exceeds about 5 million words, the HTML bundle might cause performance issues on mobile devices. For these cases, the EPUB format is better because reading apps handle pagination and lazy loading.

The EPUB format doesn't have this problem because reading apps load one chapter at a time. The HTML bundle loads everything at once.

### When to Use HTML vs EPUB

The export handler generates *both* formats for every request. But they serve different needs:

**EPUB is ideal for:**
- E-readers (Kindle, Kobo, Nook)
- Reading apps (Apple Books, Calibre, Moon+ Reader)
- Offline reading on mobile devices
- Large stories (better chapter navigation)
- Users who prefer adjustable font sizes and reader-controlled styling
- Preserving reading progress across sessions

**HTML bundle is ideal for:**
- Quick reading in a browser
- Sharing with others who might not have an e-reader
- Archiving (it's a single, self-contained file)
- Quick searches (Ctrl+F works on the entire story)
- Printing (the browser's print dialog works perfectly)
- Viewing on devices without reading apps

Both are cached independently, so subsequent requests for the same story (in either format) are instant. The choice doesn't affect performance—it's purely about user preference.

Some users prefer the HTML format because they can open it on any device without installing software. Others prefer EPUB because it syncs reading progress across devices via their reading app. By providing both, FicHub accommodates all preferences.

### Accessibility Considerations

The HTML bundle includes several accessibility features:

- **`lang="en"` on the `<html>` tag**: Tells screen readers what language to use for pronunciation.
- **Semantic HTML**: Uses `<h1>`, `<h2>`, `<table>`, `<ul>`, `<li>` elements correctly. Screen readers can navigate by heading level.
- **Sufficient color contrast**: The #333 on #fafafa combination exceeds WCAG AA requirements (4.5:1 ratio).
- **Responsive design**: Works on all screen sizes without horizontal scrolling.

The EPUB format inherits accessibility from the reading app—most modern readers support screen readers, adjustable fonts, and high-contrast modes. Our job is to provide valid XHTML and let the reader handle the rest.

### Testing the HTML Bundle

To verify the HTML bundle works correctly, try these tests:

1. **Open in Chrome/Firefox/Safari**: The page should render correctly in all major browsers.
2. **Check mobile rendering**: Resize the browser to 375px width (iPhone SE). The layout should adapt.
3. **Test the navigation**: Click each chapter link in the table of contents. The page should jump to the correct chapter.
4. **Search with Ctrl+F**: Type a word from the middle of the story. It should be found across all chapters.
5. **Extract the ZIP**: Use `unzip bundle.zip` and open `index.html`. It should work identically.
6. **Check the source**: View the HTML source. All special characters should be properly escaped.

> 🧪 **Try It Yourself**: Download an HTML bundle from FicHub, extract the ZIP, and open `index.html` in a browser. Try resizing the window—the layout should adapt. Now open the same ZIP on your phone's browser. It should be readable too. Try using Ctrl+F to search across all chapters—the HTML format excels at this.

> ⚠️ **Watch Out**: The HTML bundle uses `escape_html()` on the story description but not on chapter content. Chapter content is already HTML from the scraper. If a scraper returns raw, unescaped text instead of HTML, the bundle could break. Each scraper must ensure its chapter content is valid HTML. Always validate your scrapers' output before trusting it.

---

## Chapter 18: The Disk Cache

The disk cache module (`src/cache/disk.rs`) is the persistence layer for the export system. It provides content-addressable file storage with hash-based directory structures, atomic file moves, and cache integrity verification. Combined with the `export_log` database table, it ensures FicHub only generates each export once.

### Why Cache? Don't Redo Work You've Already Done

Generating an EPUB involves scraping a website (which can take seconds to minutes), parsing its content, building the EPUB structure, and writing it to disk. If someone requests the same story again—maybe they lost the file, or they're using a different device, or they want the HTML version this time—why redo all that work?

The answer is caching: store the generated file on disk and serve it directly on subsequent requests. FicHub's caching system is hash-based and content-addressable, which means if the story hasn't changed, we serve the same file. If it has, we generate a new one with a different hash.

The cache also serves another purpose: it provides integrity verification. When a client requests a cached file, we recompute the hash and compare it to the expected hash. If they don't match, we return an error instead of serving a potentially corrupted file.

Consider the math: without caching, a popular story with 100 chapters might take 60 seconds to scrape and generate. If 100 people request it in a day, that's 100 minutes of scraping. With caching, the first request takes 60 seconds, and all subsequent requests take milliseconds. The cache saves us 99+ minutes of work per day for a single story.

### Cache Invalidation Strategies

FicHub uses content-based cache invalidation. Instead of expiring cached files after a time period (TTL-based), we invalidate them when the content changes:

1. **Content hash from scraper**: When the scraper fetches metadata, it also computes a hash of the story's content (chapter count, word count, last update time). If this hash changes, we know the story has been updated.

2. **Export version numbers**: The export module has version numbers for each format. When we change the EPUB template (fix a CSS bug, add metadata), we bump the version. This invalidates all cached files without deleting them.

3. **Export log in PostgreSQL**: The `export_log` table records what we've generated. When a request comes in, we check the log for a matching version, format, and input hash. If it matches, we serve the cached file.

This triple-layered approach ensures we never serve stale content while maximizing cache hits.

### Hash-Based Directory Structure

The cache lives on disk with a carefully structured directory layout. The `cache_path` function computes where a file should go:

```rust
pub fn cache_path(cache_root: &Path, etype: &EType, url_id: &str, hash: &str) -> PathBuf {
    let mut path = cache_root.join(etype.as_str());

    // Split url_id into 3-char directory chunks (up to 9 chars = 3 levels deep)
    let chars: Vec<char> = url_id.chars().collect();
    for i in (0..chars.len()).step_by(3).take(3) {
        let chunk: String = chars.iter().skip(i).take(3).collect();
        if !chunk.is_empty() {
            path = path.join(chunk);
        }
    }

    // Full url_id directory
    path = path.join(url_id);

    // Actual file: hash + suffix
    path.join(format!("{}{}", hash, etype.suffix()))
}
```

This creates a path like `/cache/epub/abc/def/abcdefghijklm/ef92b24f0e9a8d3c.epub`. The three-character directory chunks prevent any single directory from having thousands of entries—which would slow down filesystem operations on many systems.

Let's trace through a few examples to understand the algorithm:

**Short url_id (`"abc"`):**
1. `cache/epub`
2. Chunk at index 0-2: `abc` → `cache/epub/abc`
3. Full url_id: `cache/epub/abc/abc`
4. File: `cache/epub/abc/abc/<hash>.epub`

**Medium url_id (`"abcdef"`):**
1. `cache/html`
2. Chunk at index 0-2: `abc` → `cache/html/abc`
3. Chunk at index 3-5: `def` → `cache/html/abc/def`
4. Full url_id: `cache/html/abc/def/abcdef`
5. File: `cache/html/abc/def/abcdef/<hash>.zip`

**Long url_id (`"abcdefghijklm"`):**
1. `cache/mobi`
2. Chunk at index 0-2: `abc` → `cache/mobi/abc`
3. Chunk at index 3-5: `def` → `cache/mobi/abc/def`
4. Chunk at index 6-8: `ghi` → `cache/mobi/abc/def/ghi`
5. Full url_id: `cache/mobi/abc/def/ghi/abcdefghijklm`
6. File: `cache/mobi/abc/def/ghi/abcdefghijklm/<hash>.mobi`

The `.take(3)` ensures we create at most three chunk directories (9 characters of the url_id). Beyond that, the full url_id is used as the final directory name. This gives us a good balance between directory depth and spread.

### The EType Enum: epub, html, mobi, pdf

FicHub supports four export formats, modeled as an enum:

```rust
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum EType {
    Epub,
    Html,
    Mobi,
    Pdf,
}

impl EType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EType::Epub => "epub",
            EType::Html => "html",
            EType::Mobi => "mobi",
            EType::Pdf => "pdf",
        }
    }

    pub fn suffix(&self) -> &'static str {
        match self {
            EType::Epub => ".epub",
            EType::Html => ".zip",     // HTML bundles are zipped
            EType::Mobi => ".mobi",
            EType::Pdf => ".pdf",
        }
    }

    pub fn version(&self) -> i32 {
        match self {
            EType::Epub => 1,
            EType::Html => 1,
            EType::Mobi => 0,
            EType::Pdf => 0,
        }
    }
}
```

The `version()` method is particularly clever. Each format has a version number that gets baked into the cache key. If we change the EPUB template (fix a CSS bug, add a new metadata field), we bump the version from 1 to 2, and all cached EPUBs are effectively invalidated without us having to delete anything.

MOBI and PDF are at version 0 because they're generated by converting from EPUB—they don't have their own templates. When the EPUB version changes, MOBI and PDF are regenerated automatically because their cache keys depend on the EPUB hash.

The `FromStr` implementation lets us parse format strings:

```rust
impl std::str::FromStr for EType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "epub" => Ok(EType::Epub),
            "html" => Ok(EType::Html),
            "mobi" => Ok(EType::Mobi),
            "pdf" => Ok(EType::Pdf),
            _ => Err(()),
        }
    }
}
```

Note the `.to_lowercase()` — this makes the parsing case-insensitive, so `"EPUB"`, `"Epub"`, and `"epub"` all parse to `EType::Epub`. This is important because the URL path might use different casing.

### move_to_cache: Moving Files to the Cache

After generating a file in a temporary directory, we move it to the cache:

```rust
pub fn move_to_cache(source: &Path, dest: &Path) -> AppResult<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(source, dest)?;
    Ok(())
}
```

This is a simple but important operation. We create the full directory structure (the three-character chunks, the url_id directory) and then move the file atomically. The `fs::rename` is an atomic operation on most filesystems—it either succeeds completely or fails completely. No half-written files in the cache.

We use `rename` instead of `copy` because the temporary file is no longer needed after moving. This saves disk space and avoids the overhead of copying large files.

The `create_dir_all` call is idempotent—if the directory already exists, it does nothing. This means concurrent requests can safely call `move_to_cache` without coordination.

### CacheSemaphores: Preventing Duplicate Work

Here's a subtle but critical problem: what if ten people request the same story simultaneously? Without coordination, we'd start ten concurrent exports—wasting CPU, bandwidth, and disk space—all generating the exact same file.

FicHub solves this with semaphores, one per (url_id, etype) pair:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

/// Semaphore map to prevent duplicate concurrent exports per (url_id, etype)
pub type CacheSemaphores = Arc<Mutex<HashMap<(String, EType), Arc<Semaphore>>>>;

pub async fn get_export_semaphore(
    semaphores: &CacheSemaphores,
    url_id: &str,
    etype: &EType,
) -> Arc<Semaphore> {
    let key = (url_id.to_string(), etype.clone());
    let mut map = semaphores.lock().await;
    map.entry(key)
        .or_insert_with(|| Arc::new(Semaphore::new(1)))
        .clone()
}
```

A `Semaphore::new(1)` is a mutex—only one task can acquire it at a time. The outer `Mutex<HashMap<...>>` protects the map of semaphores itself. The `Arc` wrapping ensures the semaphore can be shared across async tasks.

When a request comes in for a story that needs exporting, it acquires the semaphore. Subsequent requests for the same story wait. The first request generates the file, puts it in the cache, and releases the semaphore. The waiting requests then wake up—and find the file already in the cache.

The key insight is that the semaphore is *per story*. Exports for different stories proceed in parallel—only duplicate exports for the same story are serialized.

### clear_stale_cache: Cleaning Up

When a story is updated on the source site, the scraper detects a new content hash. We generate a new EPUB with a different hash—but the old one is still on disk. The `clear_stale_cache` function handles this:

```rust
pub fn clear_stale_cache(
    cache_root: &Path,
    etype: &EType,
    url_id: &str,
    keep_hash: &str,
) -> AppResult<()> {
    let dir = cache_root.join(etype.as_str()).join(url_id);
    if dir.exists() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(stem) = path.file_stem() {
                        if stem != keep_hash {
                            let _ = fs::remove_file(&path);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
```

It scans the cache directory for a given story and removes any file whose hash doesn't match the current one. The `let _ =` on `remove_file` is intentional—if a file can't be deleted (permission error, race condition), we log it but don't fail the entire operation.

Note that `clear_stale_cache` looks in `cache_root/etype/url_id/` directly (without the 3-char chunk directories). This is a simplification—the full `cache_path` function includes chunks, but for cleanup, we scan the flat url_id directory. In practice, this works because the cache directory structure is consistent.

### file_md5: Verifying Cache Integrity

The cache download handler verifies file integrity by recomputing the MD5 hash:

```rust
pub fn file_md5(path: &Path) -> AppResult<String> {
    use md5::{Md5, Digest};
    let data = fs::read(path)?;
    let hash = Md5::digest(&data);
    Ok(hex::encode(hash))
}
```

When a client requests a cached file, we verify the hash matches before serving it. If the file was corrupted on disk (filesystem error, accidental modification), we return an error instead of serving garbage.

The `hex::encode` function is from the `hex` crate—it converts the byte array to a lowercase hex string. This is more idiomatic than the manual formatting used in the EPUB/HTML generators, but produces the same result.

### Cache Size and Disk Usage

Cached files can accumulate over time. A typical EPUB is 100KB-5MB depending on story length. An HTML bundle is similar after ZIP compression. For a service with 10,000 cached stories, the total disk usage might be 1-50GB.

The `clear_stale_cache` function removes old versions of stories, but it only cleans up when a new version is generated. Stories that are deleted from the source site will remain in the cache indefinitely until manually cleaned up or until their directory is garbage-collected.

A production deployment might add a cron job that scans the cache directory and removes files older than a certain age. But for FicHub's current scale, manual cleanup is sufficient.

### The Double-Check Pattern in Detail

The export handler implements a classic concurrency pattern: double-checked locking. Here's the full flow:

1. **First check** (fast, no lock): Query the database for a cached export. This is a simple `SELECT` query against the `export_log` table, which should complete in under 5ms.

2. **Cache hit**: Return immediately. Total time: ~50ms. No lock contention, no file generation.

3. **Cache miss**: Acquire the semaphore (wait if another request is exporting the same story).

4. **Second check** (after lock): Query the database again. Another request might have finished while we waited.

5. **Second cache hit**: Return the result another request generated. We avoided redundant work.

6. **Second cache miss**: We're the first—generate the file, store it, release the semaphore.

This pattern ensures that:
- Cache hits are fast (no lock contention).
- Duplicate exports are avoided (only one generates at a time).
- Waiting requests don't redo work (they check again after the semaphore is released).
- The semaphore is automatically released when the request handler returns (RAII pattern).

The key insight is that the semaphore is *per story and per format*. Exporting story A's EPUB doesn't block exporting story B's EPUB. And exporting story A's EPUB doesn't block exporting story A's HTML. The granularity is exactly right.

### Why Not Use a Distributed Lock?

You might wonder: why use local semaphores instead of Redis-based distributed locks? The answer is that FicHub currently runs as a single instance. Local semaphores are simpler, faster, and don't require Redis for locking.

If FicHub scales to multiple instances, the semaphore approach would need to change. Options include:
1. **Redis-based locks** (e.g., Redlock): More complex but works across instances.
2. **Optimistic locking**: Use the database as the coordination point. The `export_log` table already serves this purpose.
3. **Message queue**: Route export requests through a queue so only one worker processes each request.

For now, the local semaphore combined with the database-based double-check provides sufficient coordination.

### Cache Statistics and Monitoring

In production, you'd want to monitor cache performance. Key metrics include:

- **Cache hit rate**: What percentage of requests are served from cache? A healthy system should have 80%+ hit rate for popular stories.
- **Cache size**: How much disk space is used? Set alerts for when it exceeds thresholds.
- **Stale cache entries**: How many old versions are sitting around? Run `clear_stale_cache` periodically.
- **Export latency**: How long do cache misses take? Track the 95th percentile.

You can get these metrics by logging cache hits/misses and monitoring disk usage. A simple approach is to add a metric counter in the export handler:

```rust
if cached.is_some() {
    tracing::info!("cache_hit url_id={} etype={}", meta.url_id, "epub");
} else {
    tracing::info!("cache_miss url_id={} etype={}", meta.url_id, "epub");
}
```

These logs can be aggregated into dashboards using tools like Grafana or Prometheus.

> 🧪 **Try It Yourself**: Run `ls -R /cache/epub/` (or wherever your cache directory is) to see the directory structure. Count how many stories are cached and how many have multiple versions (different hashes). The structure should match the `cache_path` algorithm: `<etype>/<3-chars>/<3-chars>/<3-chars>/<url_id>/<hash>.<suffix>`.

> ⚠️ **Watch Out**: The `move_to_cache` function uses `fs::rename`, which only works within the same filesystem. If your temporary directory and cache directory are on different mounts or volumes, the rename will fail with an "Invalid cross-device link" error. Make sure `tmp_dir` and `cache_dir` are on the same filesystem, or implement a copy-then-delete fallback.

---

## Chapter 19: Rate Limiting with Redis

The rate limiter module (`src/limiter/`) implements a Redis-backed token bucket algorithm that coordinates request rates across all FicHub instances. It protects upstream fanfiction sites from overload while keeping FicHub responsive for legitimate users.

### Why Be Polite to Websites?

FicHub scrapes content from Archive of Our Own, FanFiction.net, Royal Road, and other sites. These sites provide a free service—hosting stories for authors. Flooding them with rapid-fire requests is not only rude, it's a fast way to get our IP addresses blocked.

Rate limiting serves two purposes: it protects our upstream sites from overload, and it keeps FicHub functional by avoiding bans and temporary blocks. The rate limiter sits between FicHub's scraper and the external websites, ensuring we never exceed a safe threshold.

Consider what happens without rate limiting: a popular story update triggers 500 simultaneous requests. FicHub sends 500 concurrent HTTP requests to Archive of Our Own. AO3's servers see a sudden spike, flag it as suspicious, and temporarily block FicHub's IP range. Now *no one* can request stories from AO3 until the block expires. Rate limiting prevents this cascade.

But rate limiting isn't just about being polite. It's also about resilience. If AO3 starts returning 503 errors (server busy), the rate limiter automatically backs off by consuming extra tokens on failure. This means we naturally adapt to upstream conditions without manual intervention.

### Rate Limiting as a Shared Resource

The rate limiter is shared across all FicHub instances. If we run multiple servers (for load balancing or high availability), they all check against the same Redis buckets. This prevents the combined request rate from exceeding what upstream sites can handle.

Without shared state, each server would independently allow 30 requests/second. With 3 servers, that's 90 requests/second total—three times what we intended. Redis ensures that the global bucket is decremented atomically across all servers.

### What Is Redis? A Fast In-Memory Data Store

Redis is an in-memory data store that's incredibly fast (microsecond response times) and supports atomic operations. It's the perfect backing store for a rate limiter because:

1. **Speed**: Rate limit checks happen on every request. They need to be fast.
2. **Atomicity**: The rate limit check and update must happen as a single operation. Redis ensures this with single-threaded command execution.
3. **Persistence**: Redis can persist data to disk (via RDB snapshots or AOF logs), so rate limit state survives restarts.
4. **Shared state**: Multiple FicHub instances (if we scale horizontally) share the same rate limit state through Redis.
5. **Rich data structures**: Redis supports hashes, sorted sets, and Lua scripting—all useful for rate limiting.

FicHub connects to Redis in the server setup:

```rust
let redis_client = redis::Client::open(config.redis_url.as_str())
    .expect("Invalid Redis URL");
let redis_conn = redis_client.get_multiplexed_async_connection()
    .await
    .expect("Failed to connect to Redis");
```

The `MultiplexedConnection` is a single connection that can handle multiple concurrent requests without blocking. This is important in an async application where many tasks might need to check rate limits simultaneously.

### Token Buckets: An Analogy with Candy

The rate limiter uses the "token bucket" algorithm. Imagine you have a jar of candy:

- The jar has a **capacity** (maximum candy it can hold).
- Candy is added at a steady **flow rate** (e.g., 2 candies per second).
- Every request takes one candy from the jar.
- If the jar is empty, you have to wait until enough candy accumulates.
- If the jar is full, extra candy spills over (unused capacity doesn't carry forward beyond the cap).

In FicHub's terms, the "jar" is a Redis key, "candy" is tokens, and each HTTP request to an upstream site consumes one token.

Here are FicHub's actual token bucket parameters:

```rust
RedisBucketLimiter {
    global_capacity: 150.0,    // Max 150 tokens in global bucket
    global_flow: 30.0,         // Refill at 30 tokens/second
    ip_capacity: 30.0,         // Max 30 tokens per IP
    ip_flow: 0.116,            // ~1 token every 8.6 seconds per IP
}
```

The global bucket allows bursts of up to 150 requests across all users, refilling at 30 per second. This means we can handle a sudden spike of 150 requests, then sustain about 30 requests per second.

The per-IP bucket is much more restrictive: each user can make roughly one request every 8.6 seconds. This prevents any single user from monopolizing the global bucket. Even if a user makes 100 requests simultaneously, their per-IP bucket only allows one every 8.6 seconds.

### The Lua Script for Rate Limiting

The heart of the rate limiter is a Lua script that runs atomically inside Redis. Lua scripts in Redis execute as a single atomic operation—no other command can interfere:

```lua
local key = KEYS[1]
local requested = tonumber(ARGV[1])
local capacity = tonumber(ARGV[2])
local flow = tonumber(ARGV[3])

local bucket = redis.call('HMGET', key, 'value', 'last_drain')
local value = tonumber(bucket[1])
local last_drain = tonumber(bucket[2])

local now = redis.call('TIME')
local now_sec = tonumber(now[1]) + tonumber(now[2])/1000000

if value == nil then
    value = capacity
    last_drain = now_sec
    redis.call('HMSET', key, 'value', value, 'last_drain', last_drain)
end

local elapsed = now_sec - last_drain
local new_tokens = math.min(capacity, value + elapsed * flow)
local allowed = new_tokens - requested

if allowed >= 0 then
    redis.call('HMSET', key, 'value', allowed, 'last_drain', now_sec)
    return -1
else
    local wait = (requested - new_tokens) / flow
    return wait
end
```

Let's walk through this script step by step:

1. **Read the current state**: Get the token count (`value`) and the last time we drained (`last_drain`) from a Redis hash. A hash is like a dictionary—`HMGET` gets multiple fields at once.

2. **Get the current time**: Redis provides time with microsecond precision via the `TIME` command. We combine seconds and microseconds for floating-point precision.

3. **Initialize if needed**: If this is the first request (no state exists), fill the bucket to capacity. This gives new keys an immediate burst allowance.

4. **Calculate new tokens**: Based on how much time has elapsed since the last drain, add `elapsed * flow` tokens, capped at `capacity`. The `math.min` ensures we never exceed the bucket's capacity.

5. **Check if allowed**: Subtract the requested tokens from the new total.

6. **Decision**: If tokens remain (`allowed >= 0`), update the state and return `-1` (the convention for "allowed"). If not, calculate how many seconds to wait and return that value.

The script is loaded once when the rate limiter starts:

```rust
let lua_sha: String = redis::cmd("SCRIPT")
    .arg("LOAD")
    .arg(lua_script)
    .query_async(&mut conn)
    .await?;
```

Subsequent calls use `EVALSHA` (execute by SHA hash) instead of `EVAL` (execute by script text). This is faster because Redis caches the compiled script. The SHA is computed from the script text, so any change to the script would invalidate the SHA.

### The check_bucket Method

The `RedisBucketLimiter` uses the Lua script via this method:

```rust
async fn check_bucket(&self, key: &str, capacity: f64, flow: f64) -> Result<f64, redis::RedisError> {
    let mut conn = self.redis.clone();
    let result: f64 = redis::cmd("EVALSHA")
        .arg(&self.lua_sha[..])
        .arg(1)           // number of keys
        .arg(key)         // the Redis key
        .arg(1.0)         // requested tokens (1 per check)
        .arg(capacity)    // bucket capacity
        .arg(flow)        // refill rate
        .query_async(&mut conn)
        .await?;
    Ok(result)
}
```

The return value is either `-1` (allowed, no wait) or a positive float (wait this many seconds). The caller uses this to decide whether to proceed or sleep.

### Alternatives to Token Buckets

Other rate limiting algorithms exist, each with different trade-offs:

- **Fixed window**: Count requests in a time window (e.g., 100 requests per minute). Simple but has a burst problem—100 requests in the first second of the window is allowed.
- **Sliding window log**: Store timestamps of all requests and count those in the sliding window. Precise but memory-intensive.
- **Sliding window counter**: Combine the current and previous window counts. Good balance of accuracy and efficiency.
- **Leaky bucket**: Requests enter a queue and are processed at a fixed rate. Smooth but introduces latency.

Token buckets were chosen for FicHub because they allow bursts (important for legitimate traffic patterns) while maintaining a steady average rate. The burst capacity (150 global, 30 per-IP) absorbs normal traffic spikes without penalizing users, while the flow rate (30/second global, ~0.12/second per-IP) prevents sustained overload.

### Tuning the Rate Limits

The rate limit parameters in FicHub are carefully tuned:

```rust
global_capacity: 150.0,    // Burst allowance for the entire system
global_flow: 30.0,         // Sustained rate (30 req/sec)
ip_capacity: 30.0,         // Per-user burst allowance
ip_flow: 0.116,            // ~1 req per 8.6 seconds per user
```

The global flow of 30 requests/second is based on what AO3 and FFN can comfortably handle. The per-IP flow of 0.116 tokens/second means each user can sustain about 7 requests per minute—enough to export a 7-chapter story without hitting the limit, but slow enough that a single user can't monopolize the global bucket.

If upstream sites start blocking FicHub, the first adjustment is usually reducing `global_flow`. If a specific user is hammering the API, reducing `ip_flow` throttles them without affecting others.

### Monitoring Rate Limit State

Redis makes it easy to monitor rate limit state. You can inspect the current token count and last drain time:

```
redis-cli HMGET rate:global value last_drain
redis-cli HMGET rate:ip:192.168.1.1 value last_drain
```

This is useful for debugging—if users report slow response times, you can check whether the global bucket is depleted. If `value` is near zero, the system is running at capacity and requests are waiting.

You can also use `redis-cli MONITOR` to watch all rate limit operations in real time. This is invaluable for tuning the parameters and understanding traffic patterns.

### The Global vs Per-IP Rate Limits

The rate limiter checks two buckets in sequence:

```rust
async fn check_ip(&self, ip: IpAddr) -> RateLimitResult {
    if !self.dynamic_rate_limit {
        // Simple static delay for development
        let delay = self.static_delay_base + rand::random::<f64>() * self.static_delay_base;
        tokio::time::sleep(tokio::time::Duration::from_secs_f64(delay)).await;
        return RateLimitResult::Allowed;
    }

    // Check datacenter IPs first
    if self.is_datacenter_ip(ip) {
        return RateLimitResult::Blocked;
    }

    // Check global bucket first
    let global_wait = self.check_bucket(
        "rate:global", self.global_capacity, self.global_flow
    ).await.unwrap_or(-1.0);

    if global_wait > 0.0 {
        return RateLimitResult::Wait(global_wait.ceil() as u64);
    }

    // Then check per-IP bucket
    let ip_key = format!("rate:ip:{}", ip);
    let ip_wait = self.check_bucket(
        &ip_key, self.ip_capacity, self.ip_flow
    ).await.unwrap_or(-1.0);

    if ip_wait > 0.0 {
        return RateLimitResult::Wait(ip_wait.ceil() as u64);
    }

    RateLimitResult::Allowed
}
```

The global bucket protects the upstream sites overall. The per-IP bucket prevents any single user from consuming the entire global allocation. Both must pass for a request to proceed.

The `.unwrap_or(-1.0)` on the Redis calls is a fallback—if Redis is temporarily unavailable, we default to "allowed" rather than blocking all requests. This is a deliberate trade-off: we'd rather risk being too aggressive with upstream sites than block all FicHub users because Redis is down.

The `.ceil()` rounds the wait time up to the nearest whole second. We don't want to tell users "wait 0.3 seconds"—that's imprecise and could lead to premature retries.

### The RateLimiter Trait

FicHub defines a trait for rate limiters, making it easy to swap implementations:

```rust
#[async_trait::async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if a request is allowed for the given IP
    async fn check_ip(&self, ip: IpAddr) -> RateLimitResult;

    /// Report a failure (penalize)
    async fn report_failure(&self, ip: IpAddr);

    /// Check if IP is in a datacenter blocklist
    fn is_datacenter_ip(&self, ip: IpAddr) -> bool;
}

#[derive(Debug)]
pub enum RateLimitResult {
    Allowed,
    Wait(u64),     // Wait N seconds before retrying
    Blocked,        // IP is in datacenter blocklist
}
```

The `report_failure` method is called when a scraper encounters an error (HTTP 429, 503, connection reset). It penalizes the IP by consuming extra tokens:

```rust
async fn penalize(&self, key: &str, capacity: f64, flow: f64) -> Result<(), redis::RedisError> {
    let mut conn = self.redis.clone();
    let _: f64 = redis::cmd("EVALSHA")
        .arg(&self.lua_sha[..])
        .arg(1)
        .arg(key)
        .arg(1.5)        // penalize with 1.5 extra tokens
        .arg(capacity)
        .arg(flow)
        .query_async(&mut conn)
        .await?;
    Ok(())
}
```

Using 1.5 tokens instead of 1.0 means a failure costs 50% more than a success. This naturally backs off when a site starts returning errors. If a site gives us a 429, the next request will wait longer. The penalty decays over time as the bucket refills.

```rust
async fn report_failure(&self, ip: IpAddr) {
    let _ = self.penalize("rate:global", self.global_capacity, self.global_flow).await;
    let ip_key = format!("rate:ip:{}", ip);
    let _ = self.penalize(&ip_key, self.ip_capacity, self.ip_flow).await;
}
```

Both the global and per-IP buckets are penalized. The `let _ =` ignores Redis errors—we don't want a failed penalty to crash the application.

### Handling Rate Limit Exceeded: 429 Responses

When the rate limiter returns `Wait(n)`, the scraper respects it by sleeping for the specified duration before retrying. In development mode, the limiter uses a simpler approach:

```rust
if !self.dynamic_rate_limit {
    // Simple static delay
    let delay = self.static_delay_base + rand::random::<f64>() * self.static_delay_base;
    tokio::time::sleep(tokio::time::Duration::from_secs_f64(delay)).await;
    return RateLimitResult::Allowed;
}
```

When `dynamic_rate_limit` is disabled, the limiter just adds a small random delay (0.1 to 0.2 seconds). This is useful for local development when Redis isn't running or when you're testing scrapers and don't want to wait for real rate limits.

In production, the full token bucket algorithm handles everything dynamically. The delay adapts to the current load—if the upstream site is under heavy use, the global bucket depletes faster and requests wait longer.

### Datacenter IP Blocking

Some requests come from cloud servers, data centers, or known bots. The rate limiter maintains a set of datacenter IP addresses and blocks them outright:

```rust
pub async fn load_datacenter_ips(&self, sources: &[(String, String, String)]) {
    for (file_path, _type, _tag) in sources {
        match tokio::fs::read_to_string(file_path).await {
            Ok(content) => {
                let mut ips = self.datacenter_ips.write().await;
                for line in content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        if let Ok(ip) = line.parse::<IpAddr>() {
                            ips.insert(ip);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Could not load IP tag file {}: {}", file_path, e);
            }
        }
    }
    tracing::info!(
        "Loaded {} datacenter IPs",
        self.datacenter_ips.read().await.len()
    );
}
```

This prevents automated scrapers from using FicHub as a proxy to scrape upstream sites. The IP sets are loaded from external tag files that can be updated independently. The `RwLock<HashSet<IpAddr>>` allows concurrent reads (which is what `is_datacenter_ip` does) while ensuring exclusive access when updating the set.

Note that `is_datacenter_ip` is currently a placeholder:

```rust
fn is_datacenter_ip(&self, _ip: IpAddr) -> bool {
    // Synchronous check - this is a best-effort check
    // For production, use a proper prefix tree (ipnet crate)
    false
}
```

For production use, you'd want to use the `ipnet` crate to efficiently check CIDR ranges (like `104.16.0.0/12` for Cloudflare, `198.41.0.0/20` for Amazon AWS). A simple `HashSet` won't work because datacenter IPs are specified as ranges, not individual addresses.

> 🧪 **Try It Yourself**: Use Redis CLI to watch the rate limit buckets in real time. Run `redis-cli MONITOR` in one terminal and make a few requests to FicHub in another. You'll see `HMGET` and `HMSET` operations on the `rate:global` and `rate:ip:*` keys. Each request consumes tokens from both buckets.

> ⚠️ **Watch Out**: The `unwrap_or(-1.0)` fallback on Redis calls means the limiter defaults to "allowed" when Redis is unreachable. In production, you might want a stricter fallback—perhaps blocking all requests when Redis is down, or using a local in-memory limiter as a backup. The current approach prioritizes availability over safety.

### Redis Persistence and Recovery

Redis can persist rate limit state to disk in two ways:

1. **RDB snapshots**: Periodic snapshots of the entire dataset. Fast to load on restart, but might lose some data between snapshots.

2. **Append-only file (AOF)**: Every write operation is appended to a log file. More durable, but slower to load and uses more disk space.

For rate limiting, RDB snapshots are usually sufficient. If rate limit state is lost on restart, the worst case is a brief burst of requests that exceeds the intended limit. The buckets refill quickly enough that this is self-correcting.

If you need stricter persistence (e.g., for billing or abuse prevention), use AOF with `everysec` fsync policy. This gives you at most 1 second of data loss on a hard crash.

### Failure Modes and Edge Cases

The rate limiter handles several edge cases:

1. **Redis is down**: The `unwrap_or(-1.0)` fallback returns "allowed." This is a deliberate trade-off: we'd rather risk being too aggressive than blocking all users.

2. **Clock skew**: The Lua script uses Redis's `TIME` command, which returns the server's clock. If the Redis server's clock is wrong, the refill calculation will be inaccurate. This is rare in practice.

3. **Negative token counts**: The `math.min(capacity, value + elapsed * flow)` ensures tokens never exceed capacity. The `allowed >= 0` check ensures we never consume more tokens than available.

4. **Rapid key expiration**: Redis keys with no expiration never expire. The rate limit buckets persist until explicitly deleted. This is correct behavior—we want the state to survive for the lifetime of the application.

---

## Chapter 20: The Export Flow (End to End)

### Following a Request from Start to Finish

Now that we've explored each subsystem in isolation, let's follow a complete export request from the moment a URL hits the API to the moment a download link appears in the response. This is the `epub_handler` in `src/routes/export.rs`, and it orchestrates everything we've built—the scraper, the EPUB and HTML generators, the disk cache, the rate limiter, and the database.

The handler lives at `GET /api/v0/epub?q=<url>` and returns JSON containing download URLs for all available formats. It's the most complex endpoint in FicHub, touching nearly every subsystem. Let's trace through every step.

### Step 1: Receiving the Request

The request arrives and is immediately validated:

```rust
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    let start = std::time::Instant::now();

    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query", "q": ""})));
    }

    if params.automated.as_deref() == Some("true") {
        return Ok(Json(json!({"err": -10, "msg": "automated requests blocked"})));
    }
```

The `start` variable lets us track how long the entire export takes. We reject empty queries and explicitly automated requests. The `ExportQuery` struct captures the query parameters:

```rust
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub q: Option<String>,          // The URL to export
    pub automated: Option<String>,  // Flag indicating bot usage
    pub format: Option<String>,     // Optional format preference
}
```

All parameters are `Option` because they're not all required. The `q` parameter is the only one that matters for the basic flow.

### Step 2: Finding the Right Scraper

FicHub supports multiple fanfiction sites, each with its own scraper. The scraper registry matches the URL to the right scraper:

```rust
let scraper = state.scraper_registry.find_scraper(query)
    .ok_or_else(|| AppError::BadRequest(
        -5,
        format!("unsupported URL: {}", query)
    ))?;
```

The `ScraperRegistry` contains all registered scrapers (AO3, FFN, Royal Road, etc.). The `find_scraper` method examines the URL and returns the appropriate scraper. If the URL doesn't match any known pattern, we return error code -5.

This is the first point where the rate limiter comes into play—though in the current code, the rate limiter is checked separately before this handler is called (via middleware or in the scraper itself).

### Step 3: Looking Up Metadata

Before generating anything, we need to know what the story is. The scraper makes a lightweight HTTP request to get just the metadata:

```rust
let meta = scraper.lookup(&state.http_client, query).await
    .map_err(|e| AppError::ScrapeError(e.to_string()))?;

let info_request_ms = start.elapsed().as_millis() as i32;
```

This is intentionally separate from the chapter fetch. Most requests will find this story in the cache and never need to fetch chapters at all. The metadata lookup is the fast path that determines whether we need to do any heavy work.

The `info_request_ms` timestamp lets us track how long the metadata lookup took—useful for performance monitoring.

The handler then upserts the metadata into the database:

```rust
let fic_info_row = FicInfo {
    id: meta.url_id.clone(),
    title: meta.title.clone(),
    author: meta.author.clone(),
    author_url: Some(meta.author_url.clone()),
    author_local_id: Some(meta.author_local_id.clone()),
    chapters: meta.chapters,
    words: meta.words,
    description: meta.desc.clone(),
    fic_created: chrono::DateTime::from_timestamp_millis(meta.published)
        .unwrap_or_default(),
    fic_updated: chrono::DateTime::from_timestamp_millis(meta.updated)
        .unwrap_or_default(),
    status: meta.status.clone(),
    source: meta.source.clone(),
    extra_meta: meta.extra_meta.clone(),
    raw_extended_meta: meta.raw_extended_meta.clone(),
    source_id: Some(meta.source_id),
    author_id: Some(meta.author_id),
    content_hash: meta.content_hash.clone(),
};
queries::upsert_fic_info(&state.db, &fic_info_row).await?;
```

This keeps our database in sync with the source site. If the author changes their pen name or updates the description, we'll pick it up on the next request. The `upsert` operation (insert or update) handles both new and existing stories.

After saving the source metadata, the handler links it to a canonical work. The `find_or_create_work` function checks if a work with the same title and author already exists. If it does, and the word count is within 5%, the source is linked to that work. If not, a new work is created.

```rust
let work_result = works::find_or_create_work(&state.db, &meta).await?;
let work_id = match &work_result {
    works::AutoMergeResult::Merged { work_id, .. } => *work_id,
    works::AutoMergeResult::Created { work_id } => *work_id,
};
```

This is the heart of the unified works model. The same story posted on AO3 and FFN gets two `fic_info` rows, but both point to the same `works` row. The word count tolerance (within 5%) prevents merging stories that just happen to have similar titles.

### Step 4: Auto-Populating Tags

The handler also extracts tags from the scraper:

```rust
if let Ok(extracted_tags) = scraper.extract_tags(&state.http_client, query).await {
    for tag in &extracted_tags {
        if let Ok(resolution) = crate::tags::resolve::resolve_tag(
            &state.db, &tag.name, tag.tag_type_id,
        ).await {
            let _ = queries::upsert_fic_tag(
                &state.db, &meta.url_id, resolution.tag_id,
                &std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
            ).await;
        }
    }
}
```

Tags (genres, warnings, characters, etc.) are extracted from the source site and stored in our database. The `resolve_tag` function either finds an existing tag or creates a new one. The `UNSPECIFIED` IP address indicates this is an automated tag extraction, not a user-submitted tag.

### Step 5: Checking Blacklists

Before generating any files, the handler checks if the story or author is blacklisted:

```rust
let fic_blacklist = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;
if !fic_blacklist.is_empty() {
    // Greylist: show metadata but no download
    if fic_blacklist.iter().any(|b| b.reason == 6) {
        return Ok(build_metadata_response(
            &meta, &[], &state.config.export_version, None, true
        ));
    }
    // Hard blacklist
    if fic_blacklist.iter().any(|b| b.reason == 5 || b.reason == 7 || b.reason == 8) {
        return Ok(Json(json!({"err": -7, "msg": "fic is blacklisted", "q": query})));
    }
}

let author_blacklist = queries::check_author_blacklist(
    &state.db, meta.source_id, meta.author_id,
).await?;
if !author_blacklist.is_empty() {
    return Ok(Json(json!({"err": -7, "msg": "author is blacklisted", "q": query})));
}
```

Blacklists serve multiple purposes:
- **Reason 5**: DMCA takedown requests
- **Reason 6**: Greylist (show metadata, no download)
- **Reason 7**: Content policy violations
- **Reason 8**: Other legal requirements

The greylist response shows the story's metadata (title, author, description) but doesn't provide download links. This lets users know the story exists without facilitating distribution.

### Step 6: Computing the Cache Version

The cache version combines the export module's version with any content-level bumps:

```rust
let version_bump = queries::get_fic_version_bump(&state.db, &meta.url_id)
    .await?.unwrap_or(0);
let version = state.config.export_version + version_bump;

let input_hash = meta.content_hash.clone()
    .unwrap_or_else(|| "upstream".to_string());
```

The `export_version` is a global version number for the export module. The `version_bump` is per-story—incremented when the story's content changes on the source site. The `input_hash` is the story's content hash from the scraper, or `"upstream"` if no hash is available.

### Step 7: Checking the Cache

The cache check queries the database for a matching export:

```rust
let cached = queries::find_export_log(
    &state.db, &meta.url_id, version, "epub", &input_hash
).await?;
```

The `export_log` table stores previously generated exports. We look for a match on url_id, version, format, and input hash. If all four match, the cached file is still valid.

### Step 8: Cache Hit — Return Immediately

If the cache check finds a match, the handler builds URLs for all available formats and returns:

```rust
if let Some(export_log) = cached {
    let epub_hash = &export_log.export_hash;
    hashes.insert("epub".to_string(), epub_hash.clone());
    urls.insert("epub".to_string(),
        format!("/cache/epub/{}?h={}", meta.url_id, epub_hash));

    // Check other formats too
    for etype_str in &["html", "mobi", "pdf"] {
        if let Ok(Some(entry)) = queries::find_export_log(
            &state.db, &meta.url_id, version, etype_str,
            &format!("epub:{}", epub_hash),
        ).await {
            hashes.insert(etype_str.to_string(), entry.export_hash.clone());
            urls.insert(etype_str.to_string(),
                format!("/cache/{}/{}?h={}",
                    etype_str, meta.url_id, entry.export_hash));
        }
    }

    let slug = generate_slug(&meta.title, &meta.url_id);
    let (info_str, notes) = build_info_string(&meta);

    // Look up work_id for this source
    let work_id = queries::get_work_by_source(&state.db, &meta.url_id).await
        .ok().flatten().map(|w| w.id);

    return Ok(Json(json!({
        "err": 0,
        "q": query,
        "fixits": [],
        "info": info_str,
        "url_id": meta.url_id,
        "work_id": work_id,
        "slug": slug,
        "meta": build_meta_json(&meta),
        "hashes": hashes,
        "urls": urls,
        "epub_url": urls.get("epub"),
        "html_url": urls.get("html"),
        "mobi_url": urls.get("mobi"),
        "pdf_url": urls.get("pdf"),
        "notes": notes,
    })));
}
```

On cache hit, the total response time is typically under 50 milliseconds—a database lookup and a JSON response. No HTTP requests to upstream sites, no file generation, no disk I/O beyond reading cached files when the client downloads them.

The handler also looks up HTML, MOBI, and PDF versions. MOBI and PDF have `input_hash` of `epub:{epub_hash}`—they depend on the EPUB hash, not the raw content hash. This creates a dependency chain.

### Step 9: Cache Miss — Acquire Semaphore and Double-Check

When the cache misses, we enter the expensive path—but first, we protect against duplicate concurrent work:

```rust
let sem = cache::get_export_semaphore(
    &state.cache_semaphores, &meta.url_id, &EType::Epub
).await;
let _permit = sem.acquire().await
    .map_err(|e| AppError::Internal(e.to_string()))?;

// Double-check: did another request finish while we waited?
let cached = queries::find_export_log(
    &state.db, &meta.url_id, version, "epub", &input_hash
).await?;
if let Some(export_log) = cached {
    // Another request did the work — use their result
    let epub_hash = &export_log.export_hash;
    hashes.insert("epub".to_string(), epub_hash.clone());
    urls.insert("epub".to_string(),
        format!("/cache/epub/{}?h={}", meta.url_id, epub_hash));
    let slug = generate_slug(&meta.title, &meta.url_id);
    let (info_str, notes) = build_info_string(&meta);

    let work_id = queries::get_work_by_source(&state.db, &meta.url_id).await
        .ok().flatten().map(|w| w.id);

    return Ok(Json(json!({
        "err": 0,
        "q": query,
        "fixits": [],
        "info": info_str,
        "url_id": meta.url_id,
        "work_id": work_id,
        "slug": slug,
        "meta": build_meta_json(&meta),
        "hashes": hashes,
        "urls": urls,
        "epub_url": urls.get("epub"),
        "html_url": urls.get("html"),
        "mobi_url": urls.get("mobi"),
        "pdf_url": urls.get("pdf"),
        "notes": notes,
    })));
}
```

This is the double-check pattern from Chapter 18 in action. The first request to acquire the semaphore will proceed to generate the file. All others will wait, then find the file already cached when they check again.

The `_permit` variable is important—it holds the semaphore permit. When `_permit` goes out of scope (at the end of the function or the `return` statement), the semaphore is automatically released. This is RAII (Resource Acquisition Is Initialization) in action.

### Step 10: Fetching Chapters

Now we do the heavy lifting—actually scraping the chapter content:

```rust
let chapters = scraper.fetch_chapters(&state.http_client, &meta).await
    .map_err(|e| AppError::ScrapeError(e.to_string()))?;
```

This is where the scraper (covered in Part 3) does its work: making HTTP requests, parsing HTML, extracting chapter content, and returning a vector of `Chapter` structs. The rate limiter ensures we don't overwhelm the upstream site.

For a story with 50 chapters, this might take 30-60 seconds. The rate limiter's per-IP bucket (one request every 8.6 seconds) means we need to pace our requests carefully. The global bucket (30 requests/second) allows multiple FicHub users to scrape different stories simultaneously.

The `Chapter` struct contains everything we need for export:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,   // Sequential chapter number
    pub title: String,     // Chapter title (e.g., "Chapter 1: The Beginning")
    pub content: String,   // Full HTML content of the chapter
}
```

The `content` field is HTML, not plain text. The scraper converts the source site's markup to a consistent HTML format. This means we can use it directly in the EPUB and HTML templates without further processing.

If the scraper fails (network error, parse error, rate limit), it returns a `ScrapeError`. The `?` operator propagates this error, which Axum converts to an appropriate HTTP response. The client receives an error JSON indicating what went wrong.

### Step 11: Generating EPUB and HTML

With the chapters in hand, we generate both export formats:

```rust
// Generate EPUB
let (epub_path, epub_hash) = export::epub::create_epub(
    &meta, &chapters, &state.config.tmp_dir
).await.map_err(|e| AppError::ExportError(e.to_string()))?;

// Generate HTML bundle
let (html_path, html_hash) = export::html_bundle::create_html_bundle(
    &meta, &chapters, &state.config.tmp_dir
).await.map_err(|e| AppError::ExportError(e.to_string()))?;
```

Both functions follow the same pattern: create a UUID workspace, build the output file, compute the MD5 hash, and return the path and hash.

### Step 12: Saving to Cache

Move the generated files from temporary directories to the cache:

```rust
// Cache EPUB
let cache_dest = cache::disk::cache_path(
    &state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash
);
cache::disk::move_to_cache(&epub_path, &cache_dest)?;

// Record in export_log
queries::insert_export_log(
    &state.db, &meta.url_id, version, "epub", &input_hash, &epub_hash
).await?;

// Cache HTML
let html_cache_dest = cache::disk::cache_path(
    &state.config.cache_dir, &EType::Html, &meta.url_id, &html_hash
);
cache::disk::move_to_cache(&html_path, &html_cache_dest)?;

let html_input_hash = format!("epub:{}", epub_hash);
queries::insert_export_log(
    &state.db, &meta.url_id, version, "html", &html_input_hash, &html_hash
).await?;
```

Notice that the HTML bundle's input hash is `epub:{epub_hash}`. This creates a dependency chain: if the EPUB changes, the HTML bundle is also regenerated, even if the HTML template hasn't changed. This ensures consistency—the HTML and EPUB always represent the same version of the story.

The `insert_export_log` call records the export in the database. This is what the cache check looks for on subsequent requests. Without this record, every request would be a cache miss.

### Atomicity of Cache Operations

The cache operations are designed to be atomic:

1. **move_to_cache** uses `fs::rename`, which is atomic on the same filesystem.
2. **insert_export_log** is a single database INSERT (or UPSERT).
3. The semaphore ensures only one request performs these operations for a given story.

If the server crashes between `move_to_cache` and `insert_export_log`, the file is in the cache but not recorded in the database. The next request will regenerate the file (generating a new hash) and record it. The orphaned file will remain until manually cleaned up. This is acceptable because orphaned files don't cause errors—they just waste disk space.

### Step 13: Building the Response

The final response includes everything the client needs:

```rust
hashes.insert("epub".to_string(), epub_hash.clone());
hashes.insert("html".to_string(), html_hash.clone());
urls.insert("epub".to_string(),
    format!("/cache/epub/{}?h={}", meta.url_id, epub_hash));
urls.insert("html".to_string(),
    format!("/cache/html/{}?h={}", meta.url_id, html_hash));

let export_ms = start.elapsed().as_millis() as i32;
let slug = generate_slug(&meta.title, &meta.url_id);
let (info_str, notes) = build_info_string(&meta);

let work_id = queries::get_work_by_source(&state.db, &meta.url_id).await
    .ok().flatten().map(|w| w.id);

// Log the request
let fic_json = serde_json::to_string(&meta).ok();
queries::insert_request_log(
    &state.db, source_id, "epub", query, info_request_ms,
    Some(&meta.url_id), fic_json.as_deref(),
    Some(export_ms), Some(&format!("{}.epub", epub_hash)),
    Some(&epub_hash), Some(query),
).await?;

Ok(Json(json!({
    "err": 0,
    "q": query,
    "fixits": [],
    "info": info_str,
    "url_id": meta.url_id,
    "work_id": work_id,
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

The request log records everything: how long the metadata lookup took, how long the export took, the hashes, and the original query. This data powers analytics and helps identify slow scrapers or frequent requests.

### The Response Structure

The JSON response tells the client everything it needs:

| Field | Type | Description |
|-------|------|-------------|
| `err` | integer | 0 = success. Negative values indicate specific errors. |
| `q` | string | The original URL that was requested |
| `url_id` | string | Unique identifier for the story |
| `slug` | string | URL-safe title identifier for friendly URLs |
| `info` | string | Human-readable story summary |
| `meta` | object | Full story metadata (title, author, dates, etc.) |
| `hashes` | object | Format → hash mapping (e.g., `{"epub": "abc123"}`) |
| `urls` | object | Format → download URL mapping |
| `epub_url` | string | Direct download URL for EPUB (shorthand) |
| `html_url` | string | Direct download URL for HTML (shorthand) |
| `notes` | array | Warnings or notices about the story |
| `work_id` | integer | Canonical work ID for the unified works model | |

The `epub_url` and `html_url` fields are shorthand duplicates of what's in `urls`. They exist because many clients only need the EPUB and HTML URLs, and it's convenient to have them at the top level.

### The generate_slug Function

The slug makes download URLs human-friendly:

```rust
pub fn generate_slug(title: &str, url_id: &str) -> String {
    let sanitized: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let re = regex_lite::Regex::new(r"_+").unwrap();
    let slug = re.replace_all(&sanitized, "_").to_string();
    let slug = slug.trim_matches('_').to_string();
    format!("{}-{}", slug, url_id)
}
```

A title like "Harry Potter and the Methods of Rationality" becomes `Harry_Potter_and_the_Methods_of_Rationality-abc123`. Special characters are replaced with underscores, consecutive underscores are collapsed, and the url_id is appended for uniqueness.

The slug is purely cosmetic—it doesn't affect caching or downloads. But it makes the API response more human-readable and helps with debugging. When you see `slug: "My_Story-abc123"` in a log entry, you immediately know which story it refers to.

### The build_info_string Function

The info string is a human-readable summary:

```rust
pub fn build_info_string(meta: &FicMetadata) -> (String, Vec<String>) {
    let relative_time = {
        let now = chrono::Utc::now().timestamp_millis();
        let diff_ms = now - meta.updated;
        let diff_secs = diff_ms / 1000;
        if diff_secs < 60 {
            "less than a minute ago".to_string()
        } else if diff_secs < 3600 {
            format!("{} minutes ago", diff_secs / 60)
        } else if diff_secs < 86400 {
            format!("{} hours ago", diff_secs / 3600)
        } else {
            format!("{} days ago", diff_secs / 86400)
        }
    };

    let updated_str = chrono::DateTime::from_timestamp_millis(meta.updated)
        .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let info = format!(
        "{title} by {author}\n{words} words in {chapters} chapters\n\
         Status: {status}\nUpdated: {date} - {relative} ago\n",
        title = meta.title,
        author = meta.author,
        words = meta.words,
        chapters = meta.chapters,
        status = meta.status,
        date = updated_str,
        relative = relative_time,
    );

    (info, Vec::new())
}
```

The relative time ("3 hours ago", "2 days ago") makes it easy for users to see how recent the story is. The `Vec<String>` for notes is empty in the basic case—it's used for greylisting and other special conditions.

The function returns a tuple of `(String, Vec<String>)` because some stories have notes attached (e.g., "This fic is greylisted - download links are not available."). The notes array can contain multiple strings for different conditions.

### Request Source Tracking

The export handler tracks where requests come from:

```rust
let source_id = queries::insert_request_source(
    &state.db, false, "/api/v0/epub", "web request",
).await?;
```

This tells us whether requests are coming from the web interface, the API, or automated tools. The `automated` parameter in the query string is a hint, but the real source tracking happens at the database level. This data helps answer questions like:
- How many API requests do we get per day?
- Which stories are most popular?
- Are there any clients hammering the API?

### The Cache Download Path

When the client follows a download URL like `/cache/epub/abc123?h=ef92b24f`, the `download_with_hash` handler serves the file:

```rust
pub async fn download_with_hash(
    State(state): State<Arc<AppState>>,
    Path((etype_str, url_id, fname)): Path<(String, String, String)>,
    Query(params): Query<DownloadQuery>,
) -> Response {
    let etype = match etype_str.parse::<EType>() {
        Ok(e) => e,
        Err(_) => return Json(json!({"err": -1, "msg": "invalid format"})).into_response(),
    };

    let hash = match params.h {
        Some(h) => h,
        None => {
            let stem = fname.trim_end_matches(etype.suffix());
            stem.to_string()
        }
    };

    let cache_path = crate::cache::disk::cache_path(
        &state.config.cache_dir, &etype, &url_id, &hash
    );

    if !cache_path.exists() {
        return Json(json!({"err": -5, "msg": "file not found"})).into_response();
    }

    match crate::cache::disk::file_md5(&cache_path) {
        Ok(actual_hash) if actual_hash == hash => {
            let mime = match etype {
                EType::Epub => "application/epub+zip",
                EType::Html => "application/zip",
                EType::Mobi => "application/x-mobipocket-ebook",
                EType::Pdf => "application/pdf",
            };
            match tokio::fs::read(&cache_path).await {
                Ok(data) => {
                    let filename = format!("{}{}", url_id, etype.suffix());
                    let headers = [
                        ("Content-Type", mime),
                        ("Content-Disposition",
                            &format!("attachment; filename=\"{}\"", filename)),
                    ];
                    (headers, data).into_response()
                }
                Err(_) => Json(json!({"err": -1, "msg": "read error"})).into_response(),
            }
        }
        _ => Json(json!({"err": -5, "msg": "hash mismatch"})).into_response(),
    }
}
```

The hash verification is a security measure. Without it, an attacker could guess valid cache paths and download files that don't belong to them. The hash in the URL acts as a secret token—only someone with the hash (which is returned in the export response) can download the file.

The `Content-Disposition` header tells the browser to download the file rather than display it. The filename includes the url_id and the format suffix, giving the user a meaningful filename.

### The download_or_export Fallback

There's also a simpler download endpoint that handles cases where the client doesn't have a hash:

```rust
pub async fn download_or_export(
    State(state): State<Arc<AppState>>,
    Path((etype_str, url_id)): Path<(String, String)>,
    Query(params): Query<DownloadQuery>,
) -> Response {
    if let Some(ref hash) = params.h {
        if let Ok(etype) = etype_str.parse::<EType>() {
            let cache_path = crate::cache::disk::cache_path(
                &state.config.cache_dir, &etype, &url_id, hash
            );
            if cache_path.exists() {
                // ... serve file if hash matches ...
            }
        }
    }

    // No cached file - redirect to the main page
    Redirect::to(&format!("/?id={}", url_id)).into_response()
}
```

If the client has a hash and the file exists, serve it. Otherwise, redirect to the frontend with the story ID so the frontend can trigger an export. This handles the case where someone shares a direct link without the hash parameter.

The redirect pattern is important: instead of returning a 404 or an error, we redirect to a page that can handle the request. This provides a better user experience—the user sees a loading screen while the export runs, rather than an error message.

### The Request Source Tracking

The export handler tracks where requests come from using the `insert_request_source` function. This tells us whether requests are coming from the web interface, the API, or automated tools. The `automated` parameter in the query string is a hint, but the real source tracking happens at the database level.

This data helps answer important questions:
- How many API requests do we get per day?
- Which stories are most popular?
- Are there any clients hammering the API?
- What's the ratio of cache hits to cache misses?

### Request Logging

Every export request is logged for analytics:

```rust
queries::insert_request_log(
    &state.db, source_id, "epub", query, info_request_ms,
    Some(&meta.url_id), fic_json.as_deref(),
    Some(export_ms), Some(&format!("{}.epub", epub_hash)),
    Some(&epub_hash), Some(query),
).await?;
```

The log includes:
- The source (API vs. web interface)
- The original query
- Timing data (metadata lookup time, total export time)
- The story metadata (as JSON)
- The output file hash

This data helps identify popular stories, slow scrapers, and performance bottlenecks. It also helps with debugging—if a user reports a corrupt EPUB, we can look up the export log and check the hash.

### Error Codes Reference

The export handler returns specific error codes for different failure modes:

| Code | Message | Meaning |
|------|---------|---------|
| 0 | Success | Export completed successfully |
| -1 | No query / Invalid format | Missing or malformed request |
| -5 | Unsupported URL | No scraper matches the URL |
| -7 | Blacklisted | Story or author is blocked |
| -10 | Automated requests blocked | Bot detection triggered |

These codes are documented in the API but not formally specified. If FicHub were to publish a public API, these would be formalized with descriptions and suggested client handling.

### The Full Flow in Summary

Here's the complete flow for a cache miss:

1. **Receive request** → Validate query, reject automated requests
2. **Find scraper** → Match URL to the right scraper
3. **Lookup metadata** → Lightweight HTTP request for story info
4. **Upsert database** → Store/update metadata in PostgreSQL
5. **Extract tags** → Parse genres, warnings, characters from source
6. **Check blacklists** → Verify the story isn't blocked
7. **Compute version** → Combine export version with content version
8. **Check cache** → Database lookup for existing export
9. **Acquire semaphore** → Prevent duplicate concurrent exports
10. **Double-check cache** → Verify another request didn't finish first
11. **Fetch chapters** → Full scrape with rate limiting
12. **Generate EPUB** → Build the e-book file
13. **Generate HTML** → Build the browser-readable bundle
14. **Move to cache** → Atomic file move to permanent storage
15. **Record in database** → Store export log for future lookups
16. **Log request** → Record analytics data
17. **Return response** → JSON with download URLs and metadata

For a cache hit, steps 10-15 are skipped. The total time drops from minutes (scraping + generation) to milliseconds (database lookup + JSON response).

### Design Decisions and Trade-offs

The export flow makes several deliberate design decisions:

**Why generate both EPUB and HTML on cache miss?**
Generating both at once means the next request for either format is a cache hit. The marginal cost of generating HTML after EPUB is small (maybe 200ms) compared to the cost of scraping (30+ seconds). It's almost always worth it.

**Why store the export log in PostgreSQL instead of Redis?**
The export log is authoritative data. Redis could lose it on a crash. PostgreSQL with WAL (Write-Ahead Logging) guarantees durability. The log also powers analytics queries that are easier to write in SQL than Redis.

**Why use the MD5 hash as the cache key?**
Content-addressable storage is simple and correct. If the content changes, the hash changes, and the cache naturally serves the new version. There's no need for explicit cache invalidation—the hash handles it.

**Why not use a CDN for cached files?**
FicHub currently serves files from disk. A CDN would improve download speed for distant users but adds complexity and cost. For the current scale, disk I/O is fast enough.

### Future Improvements

Several improvements could enhance the export system:

1. **Streaming EPUB generation**: For very large stories, generate the EPUB incrementally instead of loading all chapters into memory.
2. **Background export queue**: Move export generation to a background worker so the API responds immediately with a "processing" status.
3. **Format-agnostic caching**: Store the raw chapter data in the cache and generate formats on demand, reducing initial cache miss cost.
4. **Compression optimization**: Try different ZIP compression levels to find the best size/speed trade-off for HTML bundles.
5. **Parallel chapter fetching**: Fetch multiple chapters simultaneously (within rate limits) to reduce total scrape time.

### Performance Breakdown

Here's a rough breakdown of how long each step takes for a typical 50-chapter story:

| Step | Time | Notes |
|------|------|-------|
| Metadata lookup | 1-3 seconds | Single HTTP request |
| Database upsert | 5-10 ms | PostgreSQL INSERT/UPDATE |
| Tag extraction | 1-3 seconds | Depends on scraper |
| Cache check | 1-5 ms | PostgreSQL SELECT |
| Chapter fetching | 30-90 seconds | Rate-limited HTTP requests |
| EPUB generation | 100-500 ms | CPU-bound file I/O |
| HTML generation | 50-200 ms | Simpler than EPUB |
| Cache write | 10-50 ms | Atomic rename |
| **Total (cache miss)** | **~1-2 minutes** | Mostly chapter fetching |
| **Total (cache hit)** | **~10-50 ms** | Database + JSON only |

The bottleneck is always the chapter fetching. Everything else is fast. This is why caching is so valuable—once a story is cached, subsequent requests are 1000x faster.

### Error Recovery

The export handler has several error recovery mechanisms:

1. **Semaphore timeout**: If the semaphore can't be acquired (e.g., a previous export crashed and didn't release it), the handler returns an internal error. The semaphore's permit is released when the `_permit` variable is dropped, even on error.

2. **Calibre retry**: The `convert_epub` function retries failed conversions once before giving up. This handles transient Calibre errors.

3. **Cache validation**: The download handler recomputes MD5 before serving. Corrupted files are rejected with a "hash mismatch" error.

4. **Graceful degradation**: If tag extraction fails, the export continues without tags. If blacklisting fails, the export proceeds (fail open for availability).

### The AppState: Everything Connected

The `AppState` struct holds all the shared resources that the export handler needs:

```rust
pub struct AppState {
    pub config: Config,                    // Cache dir, tmp dir, versions
    pub db: sqlx::PgPool,                  // PostgreSQL connection pool
    pub redis: redis::aio::MultiplexedConnection,  // Redis for rate limiting
    pub http_client: reqwest::Client,      // HTTP client for scraping
    pub scraper_registry: Arc<ScraperRegistry>,     // Scraper lookup
    pub cache_semaphores: CacheSemaphores,          // Export deduplication
    pub rate_limiter: Box<dyn limiter::RateLimiter>, // Rate limiting
    pub recommender_engine: RecommendationEngine,   // Story recommendations
    pub collection_worker: CollectionWorker,        // Background collection
}
```

Every handler receives `State(state): State<Arc<AppState>>` from Axum. The `Arc` (Atomic Reference Counting) allows multiple handlers to share the same state without copying. The connection pools (`db`, `redis`) are themselves concurrent—multiple requests can use them simultaneously.

The `rate_limiter` is boxed as a trait object (`Box<dyn RateLimiter>`). This allows swapping implementations at startup—the application could use `RedisBucketLimiter` in production and a mock limiter in tests.

The `config` field holds filesystem paths:

```rust
pub struct Config {
    pub cache_dir: PathBuf,   // Where cached exports live
    pub tmp_dir: PathBuf,     // Where temporary files are created
    pub export_version: i32,  // Current export module version
    // ... database URL, Redis URL, etc.
}
```

The `cache_dir` and `tmp_dir` paths must be on the same filesystem for `fs::rename` to work. If they're on different mounts, the move will fail. This is a common gotcha when deploying to containerized environments where `/tmp` might be a tmpfs mount.

> 🧪 **Try It Yourself**: Use `curl` to make an export request and trace the full flow:
> ```
> curl "http://localhost:3000/api/v0/epub?q=https://archiveofourown.org/works/123456" | python3 -m json.tool
> ```
> Note the `epub_url` and `html_url` in the response. Follow the URL to download the file. Now make the same request again—the second response should be much faster because it's a cache hit. Check the timing in the logs to see the difference.

> ⚠️ **Watch Out**: The export handler doesn't currently use the rate limiter for incoming requests—it only applies rate limiting to *outgoing* scraper requests. If FicHub becomes popular, you'd want to add incoming rate limiting to prevent abuse of the export endpoint itself. A simple middleware that checks the client's IP against a Redis-backed limit would work well here. The `RedisBucketLimiter` already has per-IP support—it just needs to be wired into the Axum middleware layer.

> ⚠️ **Watch Out**: The `Semaphore::acquire()` call can fail if the semaphore is closed (all permits have been dropped). In practice, this shouldn't happen with our code, but the `.map_err()` call handles it gracefully. If it ever does fail, it means the semaphore infrastructure has a bug—the application logs the error and returns an internal error response.

---

*Part 4 has shown how FicHub transforms raw scraped data into polished, downloadable files. The EPUB generator creates proper e-books with metadata, styling, and chapter navigation. The HTML bundle produces portable, browser-readable archives with responsive design. The disk cache ensures we only generate each file once, using content-addressable storage with hash-based directory structures. The Redis-backed token bucket rate limiter keeps our upstream sites happy with globally-coordinated, per-IP pacing. And the end-to-end export flow ties everything together with double-checked locking, semaphore-based deduplication, and comprehensive error handling. In Part 5, we'll explore how the database layer stores metadata, tracks exports, and powers the search features that help users discover new stories.*
# Part 5: The Recommendation Engine

*If the export pipeline is FicHub's factory, the recommendation engine is its brain. It watches what thousands of readers bookmark, learns the patterns of taste, and whispers to each visitor: "Based on what you're reading, you'll probably love this one too." This part covers collaborative filtering, background data collection, community suggestions, and voting—turning passive browsing data into genuinely useful recommendations.*

---

## Chapter 21: Collaborative Filtering Explained

### What Are Recommendations?

You know that friend who, after you finish a great book, immediately says "Oh, you'll love this next one"? They know your taste. They know what you just read. And they've read enough to connect the dots between your last five books and this new one you haven't heard of yet.

A recommendation engine does exactly the same thing, except it works at the scale of thousands of readers instead of one friend.

When a user opens a fanfiction on FicHub, we want to say: "Other people who read *this* story also loved *these* stories." That's the core promise. Not "here are stories with similar tags" (though that helps). Not "here's what's popular right now." But something deeper: "the community's reading patterns suggest you'll enjoy this."

The challenge is building that brain. And the technique we use is called **collaborative filtering**.

Why not just use tag matching? Because tags are noisy. Two stories might both be tagged "Harry Potter" but one is a 500-word comedy crack-fic and the other is a 500,000-word epic war drama. Tags alone can't capture the nuance of taste. But human behavior can. If 200 people bookmark both stories, there's a genuine connection there—regardless of what the tags say.

### Collaborative Filtering: Learning from What Others Liked

The word "collaborative" is the key. We're not analyzing the stories themselves—we're analyzing the *behavior* of readers. Specifically, we're looking at bookmarks.

Here's the intuition: if Reader A bookmarks Story X and Story Y, and Reader B also bookmarks Story X and Story Y, there's a connection between those two readers. If Reader B also bookmarks Story Z, then Story Z is a good recommendation for Reader A.

We don't need to know anything about the stories themselves. We don't need to read them. We don't need to understand their themes or plot points. We just need to know that the same people tend to bookmark the same things. That's the magic of collaborative filtering—the crowd's behavior teaches us about taste.

Think about how this works in practice. Imagine Alice bookmarks these stories:

- "Midnight Sun" by JaneAuthor
- "Eclipse Reimagined" by BobWriter
- "Bella's War" by CarolNerd
- "The Last Twilight" by DaveStories

And Bob bookmarks these:

- "Midnight Sun" by JaneAuthor
- "Eclipse Reimagined" by BobWriter
- "Phoenix Rising" by EveTales

Alice and Bob share two stories in common: "Midnight Sun" and "Eclipse Reimagined." That's a strong signal. When Alice discovers that Bob also liked "Phoenix Rising," that's a recommendation worth surfacing.

Now scale this up to thousands of readers. The patterns become even more powerful. If 500 readers bookmark both Story X and Story Y, that's not a coincidence—that's a genuine taste connection.

This is called **item-item collaborative filtering** because we're computing similarity between items (stories), not between users. "People who liked X also liked Y" is the item-item framing. We chose this approach because:

1. Items change more slowly than users—you write a story once, but a user's taste evolves. A story's bookmark set grows over time, but it doesn't fundamentally change. A user's preferences, on the other hand, shift constantly.
2. Item-item similarity is more stable and interpretable. "Stories A and B are similar because 50 readers bookmarked both" is easier to reason about than "Reader 123 and Reader 456 have similar tastes because they both liked 20 of the same stories."
3. We can precompute and cache item similarities efficiently. Story X's top 20 similar stories are the same for every reader who views Story X. Compute once, serve from cache.
4. New items get recommendations quickly as soon as a few people bookmark them. There's no cold-start problem for the recommendation *candidates*—only for the seed story.

There's also user-user collaborative filtering (finding similar readers), but item-item is better for our use case because we care about recommending *stories*, not finding *readers*.

### The Jaccard Coefficient: Overlap as Similarity

To measure how "similar" two stories are, we need a mathematical tool. Enter the **Jaccard coefficient**.

The Jaccard coefficient measures the overlap between two sets. If Story A was bookmarked by {Alice, Bob, Carol, Dave} and Story B was bookmarked by {Bob, Carol, Dave, Eve}, the overlap is {Bob, Carol, Dave}—that's 3 elements. The union is {Alice, Bob, Carol, Dave, Eve}—5 elements. The Jaccard coefficient is 3/5 = 0.6.

```
            Set A          Set B
          ┌───────┐     ┌───────┐
          │       │  ●●●│       │
          │  ●●   │●●●  │   ●●● │
          │       │  ●●●│       │
          └───────┘     └───────┘

          Jaccard(A, B) = |A ∩ B| / |A ∪ B|
                        = 3 / 5
                        = 0.6
```

A Jaccard coefficient of 1.0 means the stories have identical sets of bookmarkers (perfect overlap). A coefficient of 0.0 means no overlap at all. In practice, we see values between 0.0 and 0.3 for most story pairs—because most stories are bookmarked by different readers. A Jaccard of 0.2 is actually quite high and indicates a strong recommendation candidate.

Here's why Jaccard works well for us: it naturally handles stories with different numbers of bookmarks. A story with 100 bookmarks and a story with 50 bookmarks might share 10 readers. Raw count (10) seems low, but as a fraction of their combined reach, it might be very significant. Jaccard captures that significance.

Contrast this with a simpler metric like "shared bookmarkers count." Story A (1000 bookmarkers) and Story B (1000 bookmarkers) sharing 50 readers would score higher than Story C (10 bookmarkers) and Story D (10 bookmarkers) sharing 8 readers—even though the second pair is clearly more similar. Jaccard fixes this by normalizing by the union size.

Let's work through another example. Story X has 50 bookmarkers. Story Y has 30 bookmarkers. They share 15 bookmarkers. The Jaccard is 15 / (50 + 30 - 15) = 15/65 ≈ 0.23. That's a strong recommendation—23% of the combined audience overlaps. Now imagine Story Z also has 30 bookmarkers but shares only 3 with Story X. Jaccard = 3 / (50 + 30 - 3) = 3/77 ≈ 0.04. Much weaker. The co-occurrence count (15 vs. 3) directly translates to Jaccard similarity.

One subtlety: the Jaccard formula in the SQL query uses `cooccur_count::float` to ensure floating-point division. Without the `::float` cast, PostgreSQL would perform integer division, truncating the result to 0 for most pairs. This is a common gotcha in SQL—one that can silently produce incorrect rankings if you're not careful.

### Item-Item Collaborative Filtering

FicHub specifically uses **item-item** collaborative filtering rather than user-user. Here's why:

In user-user filtering, you'd find "readers similar to you" and recommend what they liked. But FicHub doesn't have user accounts—readers are anonymous (we only hash their profile URLs). So user-user filtering would require tracking individual reading histories, which raises privacy concerns and creates cold-start problems for new visitors.

Item-item filtering sidesteps this entirely. We don't need to know anything about the current reader. We just need to know: "This story is similar to that story, based on the community's bookmarking patterns." Then when someone views Story X, we show stories similar to Story X—regardless of who the reader is.

The item-item approach also has a practical advantage: we can precompute similarities and cache them. Story X's top 20 similar stories are the same for every reader who views Story X. So we compute once and serve from cache.

### The Co-occurrence Table: Tracking Which Fics Appear Together

The heart of the recommendation engine is the `fic_bookmark_cooccur` table. Every time a user bookmarks two stories, we increment the co-occurrence count for that pair.

```
work_a                  | work_b                  | cooccur_count
------------------------|-------------------------|---------------
"Midnight Sun"          | "Eclipse Reimagined"    | 47
"Midnight Sun"          | "Bella's War"           | 23
"Eclipse Reimagined"    | "Bella's War"           | 31
"Phoenix Rising"        | "The Last Twilight"     | 12
...
```

Notice the `CHECK (work_a < work_b)` constraint in the database schema. This ensures that each pair is stored only once—alphabetically ordered. Without this, we'd have both `(A, B)` and `(B, A)` and need to query both directions. The constraint guarantees that `(work_a, work_b)` is always in sorted order.

Let's trace through a concrete example. Say Alice bookmarks stories A, B, and C. Bob bookmarks stories A, B, D, and E. Carol bookmarks stories A, C, D, and F. After processing all three users, the co-occurrence table would contain:

```
work_a | work_b | cooccur_count
-------|--------|---------------
A      | B      | 2          (Alice + Bob)
A      | C      | 2          (Alice + Carol)
A      | D      | 1          (Bob + Carol)
A      | E      | 1          (Bob only)
A      | F      | 1          (Carol only)
B      | C      | 1          (Alice only)
B      | D      | 1          (Bob only)
B      | E      | 1          (Bob only)
C      | D      | 1          (Carol only)
C      | F      | 1          (Carol only)
```

Story A appears in the most pairs (5 pairs) because Alice, Bob, and Carol all bookmarked it. Stories B and C each appear in 4 pairs. The co-occurrence count tells us which stories are most connected to each other.

When someone views Story A, the engine looks up all rows where Story A appears (as either `work_a` or `work_b`), computes Jaccard for each candidate, and returns the top matches. In this example, Stories B and C would rank highest for Story A (both with co-occurrence of 2, and both having favorable Jaccard scores).

The full schema:

```sql
CREATE TABLE IF NOT EXISTS fic_bookmark_cooccur (
    work_a VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    work_b VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    cooccur_count INT4 NOT NULL DEFAULT 1,
    last_updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (work_a, work_b),
    CHECK (work_a < work_b)
);
CREATE INDEX IF NOT EXISTS idx_cooccur_a ON fic_bookmark_cooccur(work_a);
CREATE INDEX IF NOT EXISTS idx_cooccur_b ON fic_bookmark_cooccur(work_b);
```

The indexes on `work_a` and `work_b` are crucial. When we look up recommendations for a story, we need to find all rows where the story appears as either `work_a` or `work_b`. The two indexes make both lookups fast. Without them, every recommendation request would require a full table scan—unacceptable at scale.

The co-occurrence count is a raw number: how many users bookmarked both stories. But raw counts aren't enough. A story with 10,000 bookmarks will have high co-occurrence with almost everything. We need to normalize by the total number of bookmarks for each story—which is where the Jaccard coefficient comes in.

### The FicWorks Table: Tracking Bookmark Counts

To compute Jaccard, we need to know how many people bookmarked each story. The `fic_works` table tracks this:

```sql
CREATE TABLE IF NOT EXISTS fic_works (
    url_id VARCHAR(128) PRIMARY KEY REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    site_work_id VARCHAR(255) NOT NULL,
    favouriter_count INT4 NOT NULL DEFAULT 0,
    first_favourite_scraped TIMESTAMPTZ,
    last_favourite_scraped TIMESTAMPTZ,
    last_cooccur_update TIMESTAMPTZ,
    UNIQUE(site_domain, site_work_id)
);
```

The `favouriter_count` is incremented every time the collection worker discovers a new user who bookmarked this story. It's the `|A|` and `|B|` in the Jaccard formula.

### The SQL Query for Finding Similar Works

Here's the SQL that computes the Jaccard coefficient on the fly:

```sql
WITH seed AS (
    SELECT favouriter_count FROM fic_works WHERE url_id = $1
),
candidates AS (
    SELECT
        CASE WHEN work_a = $1 THEN work_b ELSE work_a END AS candidate_id,
        cooccur_count
    FROM fic_bookmark_cooccur
    WHERE work_a = $1 OR work_b = $1
)
SELECT
    c.candidate_id,
    c.cooccur_count,
    fw.favouriter_count,
    (c.cooccur_count::float
       / (s.favouriter_count + fw.favouriter_count - c.cooccur_count)
    ) AS jaccard
FROM candidates c
JOIN fic_works fw ON fw.url_id = c.candidate_id
CROSS JOIN seed s
WHERE c.candidate_id != $1
ORDER BY jaccard DESC
LIMIT $2
```

Let's break this down piece by piece:

1. **`seed` CTE**: Gets the favouriter count for the story we're recommending for (the "seed"). This is a single-row result that we'll use in the denominator.

2. **`candidates` CTE**: Finds all stories that share bookmarks with the seed. The `CASE WHEN work_a = $1 THEN work_b ELSE work_a END` expression is clever: since our seed could appear as either `work_a` or `work_b` (it's always stored alphabetically), the `CASE` picks out the *other* work in each pair. This gives us a clean list of candidate story IDs.

3. **The Jaccard calculation**: `cooccur_count::float / (s.favouriter_count + fw.favouriter_count - cooccur_count)`. This is the standard Jaccard formula. The denominator `|A| + |B| - |A ∩ B|` is mathematically equivalent to `|A ∪ B|`. The `::float` cast ensures we get decimal division instead of integer truncation.

4. **`JOIN fic_works fw`**: We need the candidate's favouriter count for the denominator.

5. **`CROSS JOIN seed s`**: Brings the seed's favouriter count into every row.

6. **`WHERE c.candidate_id != $1`**: Don't recommend a story to itself.

7. **`ORDER BY jaccard DESC`**: Most similar stories first.

8. **`LIMIT $2`**: Return the top N candidates.

This query is surprisingly efficient. The co-occurrence table has indexes on both `work_a` and `work_b`, so the `WHERE work_a = $1 OR work_b = $1` clause uses index scans. For a story with 50 bookmarkers, this query returns maybe 50-200 candidates and runs in under 50ms.

⚠️ **Watch Out**: The Jaccard formula can produce division by zero if both stories have zero bookmarkers. The `favouriter_count` column starts at 0, and the engine handles this by checking the count before doing collaborative filtering. If the seed has zero bookmarkers, we skip straight to tag-based fallback—there's nothing meaningful to compute.

### The RecQuery and RecResult Structs

The engine speaks through two clean data structures. `RecQuery` is what goes in:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecQuery {
    pub url_id: String,
    #[serde(default = "default_rec_n")]
    pub n: usize,
    pub site_domain: Option<String>,
}

const fn default_rec_n() -> usize {
    20
}

impl Default for RecQuery {
    fn default() -> Self {
        Self {
            url_id: String::new(),
            n: 20,
            site_domain: None,
        }
    }
}
```

`RecQuery` asks: "Give me N recommendations for this story, optionally filtered to a specific site." The `site_domain` filter is handy when someone only wants recommendations from the same platform—they're reading on AO3 and want more AO3 stories, not FanFiction.net ones. The default of 20 recommendations gives a good balance between variety and manageability.

`RecResult` is what comes out:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecResult {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub words: i64,
    pub chapters: i32,
    pub status: String,
    pub site_domain: String,
    pub summary: String,
    pub score: f64,
    pub community_score: i32,
    pub download_urls: HashMap<String, String>,
}
```

Each result carries the story's metadata (so the frontend can display it immediately without making another request) plus two scores: the engine's computed `score` (0.0 to ~1.0) and the `community_score` (net votes from users). The `download_urls` map is populated when the user has export-ready files, and left empty otherwise.

The `score` field is the blended result of collaborative filtering, tag matching, and voting boost. It's not a probability or a percentage—it's a relative ranking number. A score of 0.8 doesn't mean "80% likely to enjoy." It means "this candidate scored 0.8 on our blending formula." What matters is the *relative* ordering: a story with score 0.8 ranks above one with score 0.6.

### The RecommendationEngine Struct

The engine itself is beautifully simple—a single field:

```rust
pub struct RecommendationEngine {
    pub db: PgPool,
}

impl RecommendationEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { db: pool }
    }
}
```

It's a database wrapper. All the intelligence lives in SQL queries and a bit of Rust glue code. No machine learning models, no TensorFlow, no GPU clusters. Just clever queries and well-structured data.

This is a deliberate design choice. Machine learning models require training data, periodic retraining, hyperparameter tuning, and infrastructure to serve predictions. SQL queries are self-contained, debuggable, and run on the same PostgreSQL server we already have. For a project like FicHub—where the data is relatively simple (bookmarks and votes) and the recommendation problem is straightforward (find similar stories)—SQL-based collaborative filtering is the right level of complexity.

### The get_recommendations Function: Step by Step

The `get_recommendations` function follows a two-tier strategy:

```rust
pub async fn get_recommendations(
    &self,
    query: &RecQuery,
    config: &Config,
) -> Result<Vec<RecResult>, AppError> {
    let limit = query.n.min(config.rec_max_recommendations);
    if limit == 0 {
        return Ok(Vec::new());
    }

    // 1. Check precomputed cache
    let cached = check_cache(&self.db, query, config, limit).await?;
    if !cached.is_empty() {
        return Ok(cached);
    }

    // 2. Compute live
    compute_live(&self.db, query, config, limit).await
}
```

This is the classic **cache-or-compute** pattern. Precomputed results are fast (one query), but they get stale. Live computation is thorough but costs more database time. The cache TTL (time-to-live) is configurable via `config.rec_cache_ttl_hours`—typically a few hours.

**The Cache Check** queries the `precomputed_recommendations` table:

```sql
SELECT pr.recommended_url_id, pr.score,
       fi.title, fi.author, fi.words, fi.chapters,
       fi.status, fi.source, fi.description
FROM precomputed_recommendations pr
JOIN fic_info fi ON fi.id = pr.recommended_url_id
WHERE pr.url_id = $1
  AND pr.computed_at > NOW() - ($2 * INTERVAL '1 hour')
ORDER BY pr.rank ASC
LIMIT $3
```

The `computed_at > NOW() - ($2 * INTERVAL '1 hour')` clause ensures we don't serve stale results. If the cache is older than `rec_cache_ttl_hours` hours, we treat it as a miss and recompute. This is a simple TTL-based invalidation strategy—no complex cache coherence protocols, no event-driven invalidation. Just "recompute every few hours."

When the cache includes a `site_domain` filter, the query adds an extra condition:

```sql
JOIN fic_works fw ON fw.url_id = pr.recommended_url_id
WHERE pr.url_id = $1
  AND fw.site_domain = $2
  AND pr.computed_at > NOW() - ($3 * INTERVAL '1 hour')
```

This ensures that filtered recommendations only come from the requested site. The cache is stored *without* the site filter (all recommendations are cached together), and the filter is applied at query time. This means one cache entry serves all site-filtered requests—we just join with `fic_works` to filter.

After fetching from the cache, the results are enriched with community scores:

```rust
let community = get_community_scores(db).await?;

let results = rows
    .into_iter()
    .map(|r| {
        let cs = community.get(&r.recommended_url_id).copied().unwrap_or(0);
        RecResult {
            url_id: r.recommended_url_id,
            title: r.title,
            author: r.author,
            words: r.words,
            chapters: r.chapters,
            status: r.status,
            site_domain: r.source,
            summary: r.description,
            score: r.score as f64,
            community_score: cs,
            download_urls: HashMap::new(),
        }
    })
    .collect();
```

Notice that community scores are *not* cached—they're always fetched fresh. This is because votes change frequently (users can vote at any time), and we want the displayed vote counts to be current. The collaborative filtering scores are stable enough to cache, but vote counts should always reflect the latest state.

When the cache misses, `compute_live` does the heavy lifting. Let's walk through it step by step.

**Step 1: Get the seed story's metadata.**

```rust
let seed = sqlx::query_as::<_, (i32, String, String)>(
    r#"SELECT COALESCE(fw.favouriter_count, 0),
              COALESCE(fi.title, ''),
              COALESCE(fi.author, '')
       FROM fic_works fw
       JOIN fic_info fi ON fi.id = fw.url_id
       WHERE fw.url_id = $1"#,
)
.bind(&query.url_id)
.fetch_optional(db)
.await
.ok_or_else(|| AppError::NotFound(format!("Work {} not found", query.url_id)))?;
```

We need three things: the favouriter count (to compute Jaccard and decide whether to use tag fallback), the title (for keyword matching), and the author (for author matching). The `COALESCE` handles NULL values gracefully—defaulting to 0 for count and empty strings for text.

**Step 2: Find co-occurrence candidates.**

The big Jaccard query runs here, returning stories ranked by similarity. This is the core collaborative filtering signal.

**Step 3: Tag fallback if needed.**

If the seed story has very few bookmarkers (below `rec_min_favouriters_for_collab`), collaborative filtering won't have enough data. So we fall back to matching by author and title keywords. We'll cover this in detail in the "cold start" section.

**Step 4: Merge and blend scores.**

Collaborative and tag-based scores are combined using a weighted formula:

```rust
let mut scored: HashMap<String, CandidateScore> = HashMap::new();

for c in &cooccur {
    let jaccard = c.jaccard.unwrap_or(0.0);
    scored.insert(
        c.candidate_id.clone(),
        CandidateScore {
            url_id: c.candidate_id.clone(),
            score: jaccard,
            tag_score: 0.0,
            community_score: 0,
        },
    );
}

for (tag_id, tag_score_val) in &tag_candidates {
    let entry = scored.entry(tag_id.clone()).or_insert_with(|| CandidateScore {
        url_id: tag_id.clone(),
        score: 0.0,
        tag_score: 0.0,
        community_score: 0,
    });
    entry.tag_score = *tag_score_val;
}
```

The `HashMap` keyed by `url_id` ensures we don't have duplicate candidates. A story that appears in both collaborative and tag results gets both scores stored—collaborative in `score` and tag in `tag_score`.

**Step 5: Blend collaborative and tag scores.**

```rust
let weight = (favouriter_count as f64 / 5.0).min(1.0);

let mut candidates: Vec<CandidateScore> = scored.into_values().collect();
for c in &mut candidates {
    let collab = c.score;
    let tag = c.tag_score;
    c.score = collab * weight + tag * (1.0 - weight);
}
```

With 0 favouriters, `weight` is 0.0—pure tag-based. With 5+ favouriters, `weight` is 1.0—pure collaborative. This smooth transition means new stories still get reasonable recommendations while mature stories rely on real user behavior.

**Step 6: Apply voting boost.**

Community votes are fetched and applied as a logarithmic multiplier:

```rust
let net_votes = get_community_votes(db, &candidates).await?;

let gamma = config.rec_voting_boost_gamma;
for c in &mut candidates {
    let nv = net_votes.get(&c.url_id).copied().unwrap_or(0);
    c.community_score = nv;
    // Boost: score * (1.0 + gamma * ln(1 + max(0, net_votes)))
    let boost = 1.0 + gamma * (nv.max(0) as f64).ln_1p();
    c.score *= boost;
}
```

**Step 7: Sort and take the top N.**

```rust
candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
let top: Vec<CandidateScore> = candidates.into_iter().take(limit).collect();
```

**Step 8: Fetch metadata and build results.**

Each candidate's full metadata (title, author, word count, etc.) is fetched from `fic_info` and wrapped in a `RecResult`.

**Step 9: Post-filter by site domain.**

```rust
if let Some(ref domain) = query.site_domain {
    results.retain(|r| r.site_domain == *domain);
}
```

This filter is applied *after* scoring so that the ranking is still based on the full dataset, but the user only sees stories from their preferred site.

### The Cold Start Problem

"What if nobody has bookmarked this story yet?"

This is the classic **cold start problem** in recommendation systems. Without bookmark data, collaborative filtering can't compute similarities. A brand-new story on a small site might have zero bookmarkers. Even a popular story might have only 2-3 bookmarkers, which isn't enough for meaningful Jaccard coefficients.

FicHub handles this with a **tag-based fallback**. When the seed has fewer than `rec_min_favouriters_for_collab` bookmarkers (configurable, typically 5), the engine switches strategies:

```rust
let min_collab = config.rec_min_favouriters_for_collab as i32;
let use_tag_fallback = favouriter_count < min_collab;

let tag_candidates = if use_tag_fallback {
    fetch_tag_candidates(db, &query.url_id, &seed_title, &seed_author, limit).await?
} else {
    Vec::new()
};
```

The `fetch_tag_candidates` function does two things:

**1. Find stories by the same author.** This is the strongest signal—if you liked one story by an author, you'll probably like their others. We use an `ILIKE` query with wildcard patterns:

```rust
if !seed_author.is_empty() {
    let rows: Vec<FicInfoRow> = sqlx::query_as::<_, FicInfoRow>(
        r#"SELECT id, title, author, words, chapters, status, source, description
           FROM fic_info
           WHERE id != $1 AND author ILIKE $2
           ORDER BY words DESC
           LIMIT $3"#,
    )
    .bind(url_id)
    .bind(format!("%{}%", seed_author))
    .bind(limit as i64)
    .fetch_all(db)
    .await?;

    for row in rows {
        if seen.insert(row.id.clone()) {
            results.push((row.id, 0.8));
        }
    }
}
```

The `ILIKE` operator is PostgreSQL's case-insensitive `LIKE`. It matches "J.K. Rowling", "j.k. rowling", and "J.k. ROWLING" equally. The `%author%` wildcards catch partial matches too—important when authors have slightly different pen names across sites.

Same-author matches get a score of 0.8—high but not perfect. The author might write in very different styles or fandoms, so we can't assume identity.

**2. Find stories with matching title keywords.** This is a weaker signal, but it works surprisingly well for stories with distinctive words in their titles:

```rust
let keywords: Vec<&str> = seed_title
    .split_whitespace()
    .filter(|w| w.len() >= 3)
    .collect();

for kw in keywords {
    if results.len() >= limit {
        break;
    }
    let remaining = limit - results.len();
    let rows: Vec<FicInfoRow> = sqlx::query_as::<_, FicInfoRow>(
        r#"SELECT id, title, author, words, chapters, status, source, description
           FROM fic_info
           WHERE id != $1 AND title ILIKE $2
           ORDER BY words DESC
           LIMIT $3"#,
    )
    .bind(url_id)
    .bind(format!("%{}%", kw))
    .bind(remaining as i64)
    .fetch_all(db)
    .await?;

    for row in rows {
        if seen.insert(row.id.clone()) {
            results.push((row.id, 0.5));
        }
    }
}
```

The `filter(|w| w.len() >= 3)` skips tiny words like "the" or "a" that would match everything. We search one keyword at a time, stopping as soon as we have enough results. Title matches get a score of 0.5—meaningful but weaker than author matches.

The `seen` set prevents the same story from appearing twice. If a story matches both an author search and a title keyword search, it gets the higher score (0.8 from the author match, because author results are added first).

🧪 **Try It Yourself**: Think about the Jaccard coefficient. If Story A has 20 bookmarkers and Story B has 30 bookmarkers, and they share 8 bookmarkers, what's the Jaccard? Answer: 8 / (20 + 30 - 8) = 8/42 ≈ 0.19. That's actually a pretty strong similarity—19% overlap in a system with thousands of potential bookmarkers. In practice, anything above 0.1 is worth recommending.

### Blending Collaborative and Tag Scores

The blending formula is elegant and well-tuned:

```
final_score = collaborative × weight + tag × (1 - weight)
weight = min(1.0, favouriter_count / 5.0)
```

Think of it as a sliding scale. When a story is brand-new (0 favouriters), `weight` is 0.0 and we rely entirely on tag matching. As bookmarkers accumulate, the weight shifts toward collaborative filtering. By 5+ bookmarkers, collaborative data is fully trusted.

This transition is smooth and continuous—there's no hard cutoff where we suddenly switch from one strategy to another. A story with 2 bookmarkers gets 40% collaborative and 60% tag. A story with 3 gets 60% collaborative and 40% tag. It gracefully evolves as data accumulates.

Why 5 as the threshold? It's a heuristic, but it's based on the observation that Jaccard coefficients become meaningful once there are about 5 shared bookmarkers. Below that, the signal is too noisy to be reliable. Above that, collaborative filtering outperforms tag matching.

### The Voting Boost: Community Favorites

Beyond collaborative and tag signals, the engine incorporates community voting. When users upvote or downvote a recommendation, those votes create a `community_score` for each candidate.

The boost formula uses a logarithmic curve:

```
boost = 1.0 + gamma × ln(1 + max(0, net_votes))
```

Why logarithmic? Because a story with +100 votes is better than one with +10 votes, but not 10× better. The logarithm captures diminishing returns—the first few votes matter more than the hundredth.

The `gamma` parameter (from config) controls how much votes influence the final ranking. A higher `gamma` means votes matter more; a lower `gamma` means the engine relies more on collaborative filtering.

After all signals are blended and boosted, the candidates are sorted by final score, and the top N are returned. Each result includes both the computed score and the community vote count, giving the frontend everything it needs to display a rich recommendation card.

### Putting It All Together

Let's trace a complete recommendation request:

1. User A visits a story's page and clicks "Recommendations."
2. The frontend calls `GET /api/v0/recommendations?url_id=abc123&n=20`.
3. The handler resolves the URL, checks the database, and calls `get_recommendations`.
4. The engine checks the cache—miss.
5. `compute_live` runs: gets seed metadata, queries co-occurrence, runs Jaccard, applies tag fallback if needed, blends scores, applies voting boost, sorts, and fetches metadata.
6. 20 `RecResult` objects are returned as JSON.
7. The results are cached in `precomputed_recommendations` for next time.
8. The frontend renders 20 recommendation cards with titles, authors, word counts, and scores.

The whole thing takes under 100ms for a warm cache hit, and typically under 500ms for a live computation. That's fast enough to feel instant.

The cache is precomputed by the `compute_and_cache` method:

```rust
pub async fn compute_and_cache(
    &self,
    url_id: &str,
    config: &Config,
) -> Result<(), AppError> {
    let query = RecQuery {
        url_id: url_id.to_string(),
        n: config.rec_max_recommendations,
        site_domain: None,
    };

    let results = compute_live(&self.db, &query, config, config.rec_max_recommendations).await?;

    // Clear old cache entries for this work
    sqlx::query("DELETE FROM precomputed_recommendations WHERE url_id = $1")
        .bind(url_id)
        .execute(&self.db)
        .await?;

    // Insert new entries
    for (rank, result) in results.iter().enumerate() {
        sqlx::query(
            r#"INSERT INTO precomputed_recommendations
                   (url_id, recommended_url_id, score, rank, computed_at)
               VALUES ($1, $2, $3, $4, NOW())"#,
        )
        .bind(url_id)
        .bind(&result.url_id)
        .bind(result.score as f32)
        .bind(rank as i16)
        .execute(&self.db)
        .await?;
    }

    Ok(())
}
```

This method is called periodically (or on-demand) to refresh the cache. It deletes old entries and inserts new ones with the current timestamp. The `rank` column preserves the ordering, so the cache query can `ORDER BY rank ASC` without recomputing scores.

The `precomputed_recommendations` table schema:

```sql
CREATE TABLE IF NOT EXISTS precomputed_recommendations (
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    recommended_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    score REAL NOT NULL,
    rank SMALLINT NOT NULL,
    computed_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (url_id, recommended_url_id)
);
CREATE INDEX IF NOT EXISTS idx_precomputed_url ON precomputed_recommendations(url_id, rank);
```

The `score` column uses `REAL` (single-precision float) instead of `DOUBLE PRECISION` to save space. We don't need double precision for recommendation scores—a float has plenty of precision for ranking. The `rank` column is a `SMALLINT` (2 bytes) since we'll never have more than 32,767 recommendations for a single story.

The composite primary key `(url_id, recommended_url_id)` ensures no duplicate recommendations, and the index on `(url_id, rank)` makes the cache query fast—we can efficiently look up all recommendations for a story, ordered by rank.

⚠️ **Watch Out**: The cache stores recommendations *without* the site filter. If you want site-filtered recommendations, the filtering happens at query time by joining with `fic_works`. This means the cache might contain 100 precomputed recommendations, but only 15 are from the requested site. The `LIMIT` is applied *after* filtering, so you might get fewer results than requested. This is a trade-off: one cache entry serves all filter combinations, but you can't guarantee a specific number of results for filtered queries.

🧪 **Try It Yourself**: Write a SQL query that finds the top 5 stories most similar to a story with url_id `'abc123'`, using the Jaccard formula from this chapter. Hint: you'll need the `fic_bookmark_cooccur` and `fic_works` tables, and the formula is `cooccur_count / (favouriter_count_a + favouriter_count_b - cooccur_count)`.

---

## Chapter 22: The Collection Worker

### Why Collect Data?

The recommendation engine is only as good as its data. Without knowing which users bookmark which stories, collaborative filtering has nothing to work with. The co-occurrence table starts empty. The Jaccard coefficients are meaningless. It's like a restaurant with no reviews—the food might be great, but nobody knows.

We need a system that continuously collects bookmark data from fanfiction sites—a background worker that crawls user profiles, discovers which stories they've bookmarked, and updates the co-occurrence counts. This worker runs 24/7, quietly building the knowledge base that powers recommendations.

The worker answers a simple question: "If I visit this story's page on AO3 and look at who bookmarked it, and then visit each of those users' profiles to see what else they bookmarked—what can I learn about which stories go together?"

It's a lot of web scraping. And it needs to be done carefully—politely, efficiently, and with respect for user privacy.

Consider the scale: a popular AO3 story might have 5,000+ bookmarkers. Each bookmarker might have 50+ bookmarks of their own. That's potentially 250,000+ bookmark relationships to discover for a single story. We obviously can't crawl all of that—we'd be making requests for days. But we don't need to. Even crawling 5 pages of bookmarkers (maybe 250 users) and 3 pages of their favourites (maybe 150 works per user) gives us enough data to build meaningful co-occurrence patterns.

The key insight is that we don't need comprehensive data—we need *sufficient* data. A co-occurrence count of 5 between two stories is meaningful, even if the true count is 50. The relative ordering (which story pairs have higher co-occurrence) is what matters, and that ordering stabilizes with surprisingly few data points.

### The CollectionWorker: A Robot That Works While You Sleep

The `CollectionWorker` is FicHub's background data collector. It's not a web request handler—it's a long-running process that polls a queue and does work without anyone asking.

```rust
pub struct CollectionWorker {
    db: PgPool,
    redis: Mutex<MultiplexedConnection>,
    client: Client,
    config: Config,
    registry: Arc<ScraperRegistry>,
    fetchers: Vec<Box<dyn SiteFetcher>>,
    rate_limiters: HashMap<String, PerSiteRateLimiter>,
}
```

Let's unpack each field:

- **`db`**: The PostgreSQL connection pool for reading and writing bookmark data. This is how the worker interacts with the same database that the recommendation engine reads from.

- **`redis`**: A Redis connection wrapped in a `Mutex` (since we share it across async tasks). Redis serves as the work queue—a lightweight, fast message broker that doesn't require a separate service.

- **`client`**: An HTTP client for making requests to fanfiction sites. This is a `reqwest::Client` with sensible defaults (timeouts, redirect policies, connection pooling).

- **`config`**: Configuration values like rate limits, maximum page counts, and cache TTLs. These let operators tune the worker's behavior without changing code.

- **`registry`**: The scraper registry, so we can look up site-specific scrapers when needed.

- **`fetchers`**: A list of `SiteFetcher` implementations—one per supported site. This is the polymorphic core: each site has its own scraping logic, but the worker treats them all the same.

- **`rate_limiters`**: Per-domain rate limiters that ensure we don't hammer any single site. Stored in a `HashMap<String, PerSiteRateLimiter>` keyed by domain.

The constructor registers a fetcher for each known site:

```rust
pub fn new(
    db: PgPool,
    redis: MultiplexedConnection,
    client: Client,
    config: Config,
    registry: Arc<ScraperRegistry>,
) -> Self {
    let mut fetchers: Vec<Box<dyn SiteFetcher>> = Vec::new();
    fetchers.push(Box::new(Ao3Fetcher));
    fetchers.push(Box::new(FfNetFetcher));
    fetchers.push(Box::new(XenForoFetcher));
    fetchers.push(Box::new(FictionPressFetcher));
    fetchers.push(Box::new(AdultFanFictionFetcher));
    fetchers.push(Box::new(HpFanFicFetcher));

    let mut rate_limiters = HashMap::new();
    for fetcher in &fetchers {
        let domain = fetcher.site_domain().to_string();
        let delay = fetcher.rate_limit_delay(&config, &domain);
        rate_limiters.insert(domain, PerSiteRateLimiter::new(delay));
    }

    Self {
        db,
        redis: Mutex::new(redis),
        client,
        config,
        registry,
        fetchers,
        rate_limiters,
    }
}
```

Each fetcher knows how to collect bookmarks from its specific site. AO3's bookmark page structure is completely different from FanFiction.net's, so each site needs its own implementation. The `rate_limiters` map is pre-populated with the correct delay for each site.

### The SiteFetcher Trait: Collecting Bookmarks from Each Site

The `SiteFetcher` trait defines the interface that every site-specific scraper must implement:

```rust
#[async_trait]
pub trait SiteFetcher: Send + Sync {
    /// Canonical domain for this site.
    fn site_domain(&self) -> &str;

    /// Per-site rate-limit delay in seconds.
    fn rate_limit_delay(&self, config: &Config, domain: &str) -> u64 {
        config
            .rec_site_rate_limits
            .get(domain)
            .copied()
            .unwrap_or(config.rec_default_delay_secs)
    }

    /// Collect users who have favourited / bookmarked the given work URL.
    async fn collect_favouriters(
        &self,
        client: &Client,
        work_url: &str,
        max_pages: u32,
    ) -> Result<Vec<String>, ScrapeError>;

    /// Collect the set of works favourited by a user.
    async fn collect_user_favourites(
        &self,
        client: &Client,
        user_url: &str,
        max_pages: u32,
    ) -> Result<Vec<String>, ScrapeError>;

    /// Compute a deterministic hash for a user's profile URL.
    fn user_hash(&self, user_url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(user_url.to_lowercase().as_bytes());
        hex::encode(hasher.finalize())
    }
}
```

Two core methods:

1. **`collect_favouriters`**: Given a story URL, return a list of user profile URLs for everyone who bookmarked it. On AO3, this means paginating through the "Bookmarks" tab of a work. On FanFiction.net, it might mean checking the "Favorited by" page. Each site has its own way of presenting this information.

2. **`collect_user_favourites`**: Given a user profile URL, return a list of story URL IDs they've bookmarked. On AO3, this means paginating through a user's "Bookmarks" page. On FanFiction.net, it means checking their "Favorites" page.

The `max_pages` parameter limits how deep we crawl. We don't need *every* bookmarker of a popular story—that could be thousands of pages. We just need enough to build meaningful co-occurrence counts. Typically, 5-10 pages per query is plenty.

The `rate_limit_delay` method has a default implementation that checks `config.rec_site_rate_limits` for an override. If the config has a specific delay for this domain, use it. Otherwise, fall back to `config.rec_default_delay_secs`.

The current implementations are stubs—they return empty vectors:

```rust
struct Ao3Fetcher;

#[async_trait]
impl SiteFetcher for Ao3Fetcher {
    fn site_domain(&self) -> &str {
        "archiveofourown.org"
    }

    async fn collect_favouriters(
        &self,
        _client: &Client,
        _work_url: &str,
        _max_pages: u32,
    ) -> Result<Vec<String>, ScrapeError> {
        Ok(Vec::new())
    }

    async fn collect_user_favourites(
        &self,
        _client: &Client,
        _user_url: &str,
        _max_pages: u32,
    ) -> Result<Vec<String>, ScrapeError> {
        Ok(Vec::new())
    }
}
```

The architecture is the focus—the trait defines the contract, and real scraping logic can be added later per-site without changing the worker. This is the **strategy pattern** in action: the worker doesn't know how to scrape AO3, but it knows how to call a `SiteFetcher` that does.

### SHA-256 User Hashing for Privacy

Notice the `user_hash` method. We never store raw user profile URLs. Instead, we compute a SHA-256 hash:

```rust
fn user_hash(&self, user_url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(user_url.to_lowercase().as_bytes());
    hex::encode(hasher.finalize())
}
```

The `user_hash` column in the database is 64 hex characters—a one-way hash. We can't reverse it to find the original URL. We can't look up a user by their profile URL (we'd need to hash it first). And if the database were ever compromised, the user identities would be protected.

The `to_lowercase()` ensures that `User123` and `user123` produce the same hash. Consistency matters more than case sensitivity here—we don't want the same user treated as two different people because of capitalization differences.

SHA-256 is a cryptographic hash function. While it's not strictly necessary for privacy (we're not trying to prevent brute-force attacks), it's the standard choice for deterministic hashing. It's fast, widely supported (via the `sha2` crate), and produces a uniform distribution of outputs.

The privacy model is important: FicHub collects reading behavior data but doesn't identify individuals. We know that "user hash abc123" bookmarked stories X, Y, and Z. We don't know who that person is, where they live, or what their AO3 username is. This is a privacy-by-design approach.

### The PerSiteRateLimiter: Being Polite to Websites

Fanfiction sites are not big tech companies. They're often run by volunteers on modest hardware. AO3, for example, runs on donations and has limited server capacity. Hammering it with rapid-fire requests would be rude at best, and get our IP banned at worst.

The `PerSiteRateLimiter` ensures we wait between requests to each site:

```rust
pub struct PerSiteRateLimiter {
    last_request: AtomicI64,
    delay_secs: u64,
}

impl PerSiteRateLimiter {
    pub fn new(delay_secs: u64) -> Self {
        Self {
            last_request: AtomicI64::new(0),
            delay_secs,
        }
    }
}
```

The implementation uses an atomic compare-and-swap (CAS) loop to coordinate concurrent access without locks:

```rust
pub async fn wait_if_needed(&self) {
    let delay_nanos = (self.delay_secs as u64) * 1_000_000_000;

    loop {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;

        let last = self.last_request.load(Ordering::Acquire);
        let elapsed = now.wrapping_sub(last);

        if elapsed >= delay_nanos as i64 {
            // Enough time has passed — try to claim this slot.
            if self.last_request
                .compare_exchange(last, now, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                return;
            }
            // CAS failed — another thread claimed a slot. Retry.
        } else {
            // Still within cooldown — sleep for the remainder.
            let remaining = delay_nanos - elapsed as u64;
            tokio::time::sleep(Duration::from_nanos(remaining)).await;
        }
    }
}
```

Here's how it works:

1. Read the current time and the last request timestamp.
2. If enough time has elapsed, try to atomically claim the slot by updating `last_request`.
3. If the CAS succeeds, we're good—proceed with the request.
4. If the CAS fails (another thread jumped in between our load and our CAS), retry the whole check with a fresh timestamp.
5. If not enough time has elapsed, sleep until the cooldown expires, then loop back to try claiming a slot.

The `last_request` field is initialized to 0, which represents "never requested." Since any real timestamp is much larger than 0, the first caller always passes the check immediately.

Why use atomic CAS instead of a Mutex? Because we want the check-and-update to be as fast as possible. A Mutex would block other threads for the entire duration of the critical section. The CAS loop only blocks briefly—just long enough to check and update a single integer.

⚠️ **Watch Out**: The rate limiter uses `AtomicI64` and `Ordering::AcqRel`. This is important—without the correct memory ordering, the CAS loop could see stale values and send requests too quickly. The `Acquire` load ensures we see the latest write from any other thread, and `AcqRel` ensures our write is visible to subsequent loads by other threads. Getting the ordering wrong could cause the rate limiter to fail silently.

### The Redis Queue: A To-Do List for the Collector

Redis provides a simple, fast queue using lists. Each site gets its own queue:

```
collection_queue:archiveofourown.org
collection_queue:fanfiction.net
collection_queue:forums.spacebattles.com
collection_queue:fictionpress.com
collection_queue:adult-fanfiction.org
collection_queue:hpfanfic.com
```

Why separate queues per site? Because different sites have different rate limits and processing characteristics. AO3's bookmark pages are structured and parse quickly. FanFiction.net's pages are older and sometimes inconsistent. SpaceBattles uses XenForo, which has its own quirks. By separating queues, we can process each site at its own pace without one slow site blocking the others.

The `QueueItem` struct represents a single unit of work:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub url_id: String,
    pub site_domain: String,
    pub site_work_id: String,
}
```

When a new story is scraped or requested, we enqueue it for collection. The worker picks it up and processes it.

Why Redis and not PostgreSQL for the queue? Redis is an in-memory data store—it's orders of magnitude faster for queue operations. A `LPUSH`/`LPOP` pair completes in microseconds, while a PostgreSQL `INSERT` + `SELECT FOR UPDATE` + `DELETE` cycle takes milliseconds. For a queue that processes hundreds of items per minute, that difference matters.

Redis also doesn't need persistence for the queue. If the worker restarts, it just re-enqueues anything that was in progress (or accepts the loss—these aren't critical tasks). And Redis's `LPUSH`/`LPOP` operations are atomic, so we don't need additional locking. The `Mutex<MultiplexedConnection>` in the worker is for sharing the connection across async tasks, not for queue atomicity.

The queue is intentionally simple. No priorities, no retries, no dead-letter queues. Items are processed in FIFO order, and failures are logged and forgotten. This simplicity is a feature—it makes the system easy to understand, debug, and operate. For FicHub's scale (hundreds of items per day, not millions per second), this simplicity is the right trade-off.

### The Enqueue Function: Adding Work to the Queue

Adding work to the queue is a single Redis command:

```rust
pub async fn enqueue(
    &self,
    url_id: &str,
    site_domain: &str,
    site_work_id: &str,
) -> Result<(), redis::RedisError> {
    let item = QueueItem {
        url_id: url_id.to_string(),
        site_domain: site_domain.to_string(),
        site_work_id: site_work_id.to_string(),
    };
    let json = serde_json::to_string(&item)
        .expect("QueueItem serialisation should not fail");

    let key = format!("collection_queue:{}", site_domain);
    let mut conn = self.redis.lock().await;

    redis::cmd("LPUSH")
        .arg(&[key.as_str(), json.as_str()])
        .query_async(&mut *conn)
        .await
}
```

`LPUSH` pushes to the left of a Redis list. The worker uses `LPOP` (pop from the left) to consume items in FIFO order. It's the simplest queue possible—no message brokers, no consumer groups, no complexity.

The `QueueItem` is serialized to JSON before pushing. This makes the queue inspectable (you can `LRANGE` the key to see pending items) and debuggable (the JSON is human-readable).

### The Run Function: The Main Loop

The worker's `run` method is an infinite loop that polls all site queues:

```rust
pub async fn run(&self) {
    info!("Collection worker started — polling Redis queues for all known sites");

    loop {
        for fetcher in &self.fetchers {
            let domain = fetcher.site_domain();
            let key = format!("collection_queue:{}", domain);

            let item_str: Option<String> = {
                let mut conn = self.redis.lock().await;
                redis::cmd("LPOP")
                    .arg(&key)
                    .query_async(&mut *conn)
                    .await
                    .unwrap_or(None)
            };

            if let Some(item_str) = item_str {
                // Apply rate limit before making HTTP requests.
                if let Some(rl) = self.rate_limiters.get(domain) {
                    rl.wait_if_needed().await;
                }

                match serde_json::from_str::<QueueItem>(&item_str) {
                    Ok(item) => {
                        debug!("Processing {} from {}", item.url_id, domain);
                        if let Err(e) = self.process_work(item).await {
                            error!("Error processing work on {}: {}", domain, e);
                        }
                    }
                    Err(e) => {
                        warn!("Invalid queue item on {}: {}", domain, e);
                    }
                }
            }
        }

        // Brief sleep to avoid busy-looping Redis when queues are empty.
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
```

Notice the 100ms sleep at the end. Without it, the loop would spin as fast as possible, burning CPU while waiting for new items. The 100ms pause is short enough to feel responsive (a new item will be picked up within ~100ms of being enqueued) but long enough to avoid wasteful polling.

Each site's queue is checked independently. If AO3 has 5 items and FanFiction.net has 0, the worker processes all 5 AO3 items before moving on (well, interleaved—each loop iteration checks all queues once). This ensures we don't starve one site while another is busy.

Error handling is pragmatic: if a queue item fails to parse, we log a warning and skip it. If processing fails, we log an error and move on. We don't retry failed items immediately—they'd just fail again. In a production system, you might push failed items to a dead-letter queue for investigation, but for FicHub's scale, logging and moving on is sufficient.

### Processing a Work: Step by Step

The `process_work` method is where the real work happens. It's a multi-step procedure:

```rust
async fn process_work(&self, item: QueueItem) -> Result<(), Box<dyn std::error::Error>> {
    // Find the right fetcher for this site
    let fetcher = self.fetchers
        .iter()
        .find(|f| f.site_domain() == item.site_domain)
        .ok_or_else(|| format!("No SiteFetcher for domain {}", item.site_domain))?;

    let work_url = format!("https://{}/works/{}", item.site_domain, item.site_work_id);

    // Step A: Fetch favouriters (who bookmarked this work?)
    let favouriters = fetcher
        .collect_favouriters(&self.client, &work_url, self.config.rec_max_favourite_pages)
        .await?;

    let mut new_user_count: u32 = 0;

    for user_url in &favouriters {
        let user_hash = fetcher.user_hash(user_url);

        // Skip users already recorded for this work
        let already_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM fic_bookmarks WHERE user_hash = $1 AND url_id = $2)",
        )
        .bind(&user_hash)
        .bind(&item.url_id)
        .fetch_one(&self.db)
        .await
        .unwrap_or(false);

        if already_exists {
            continue;
        }

        new_user_count += 1;

        // Ensure the fic_works row exists (or update counter)
        sqlx::query(
            r#"INSERT INTO fic_works (url_id, site_domain, site_work_id, favouriter_count,
                                       first_favourite_scraped, last_favourite_scraped)
               VALUES ($1, $2, $3, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               ON CONFLICT (url_id) DO UPDATE SET
                   favouriter_count    = fic_works.favouriter_count + 1,
                   last_favourite_scraped = CURRENT_TIMESTAMP"#,
        )
        .bind(&item.url_id)
        .bind(&item.site_domain)
        .bind(&item.site_work_id)
        .execute(&self.db)
        .await?;

        // Record the bookmark
        sqlx::query(
            "INSERT INTO fic_bookmarks (user_hash, url_id, site_domain)
             VALUES ($1, $2, $3)
             ON CONFLICT DO NOTHING",
        )
        .bind(&user_hash)
        .bind(&item.url_id)
        .bind(&item.site_domain)
        .execute(&self.db)
        .await?;

        // Step B: Fetch this user's favourite works
        let user_favourites = fetcher
            .collect_user_favourites(
                &self.client,
                user_url,
                self.config.rec_max_user_favourite_pages,
            )
            .await?;

        // Step C: Update co-occurrence for all pairs in the set
        if user_favourites.len() >= 2 {
            self.update_cooccurrence(&item.site_domain, &user_favourites)
                .await?;
        }
    }

    // Step D: Ensure the fic_works row exists even with no new users
    if new_user_count == 0 {
        sqlx::query(
            r#"INSERT INTO fic_works (url_id, site_domain, site_work_id, favouriter_count)
               VALUES ($1, $2, $3, 0)
               ON CONFLICT (url_id) DO UPDATE SET
                   last_favourite_scraped = CURRENT_TIMESTAMP
               WHERE fic_works.first_favourite_scraped IS NULL"#,
        )
        .bind(&item.url_id)
        .bind(&item.site_domain)
        .bind(&item.site_work_id)
        .execute(&self.db)
        .await?;
    } else {
        info!(
            "Processed {} on {} — {} new user(s), {} favouriter(s) total",
            item.url_id, item.site_domain, new_user_count, favouriters.len(),
        );
    }

    Ok(())
}
```

Let's trace through the steps:

**Step A**: Call `collect_favouriters` to get user profile URLs from the site's bookmark page. The `max_pages` parameter from config limits how deep we crawl.

**For each new user** (one we haven't already recorded for this story):

**Step B**: Call `collect_user_favourites` to get the full list of stories this user has bookmarked. This is the expensive part—one HTTP request per page of their bookmarks, paginated.

**Step C**: Update co-occurrence counts for all pairs of works in that user's favourite set. If they bookmarked 10 stories, that's C(10,2) = 45 pairs to update.

**Step D**: If no new users were found (all already in the database), still ensure the `fic_works` row exists. This handles the case where a story was already fully processed—the row should still exist with accurate metadata.

The `ON CONFLICT DO NOTHING` on the bookmark insert prevents duplicate entries. The `ON CONFLICT DO UPDATE` on `fic_works` increments the favouriter count. These upsert patterns are essential—the worker might re-process a story if it's re-enqueued, and we need idempotency.

⚠️ **Watch Out**: The `process_work` method does a lot of database queries—one `EXISTS` check, one upsert, one insert, and one co-occurrence update *per user*. For a story with 200 bookmarkers, that's 800+ database queries. This is fine for a background worker running at moderate speed, but it would be problematic in a request handler. The worker's isolation from the API layer is important.

### Finding What Each User Bookmarked: The Co-occurrence Update

The `update_cooccurrence` method is a simple nested loop:

```rust
async fn update_cooccurrence(
    &self,
    site_domain: &str,
    works: &[String],
) -> Result<(), sqlx::Error> {
    for i in 0..works.len() {
        for j in (i + 1)..works.len() {
            let (work_a, work_b) = if works[i] < works[j] {
                (works[i].as_str(), works[j].as_str())
            } else {
                (works[j].as_str(), works[i].as_str())
            };

            sqlx::query(
                r#"INSERT INTO fic_bookmark_cooccur
                       (work_a, work_b, site_domain, cooccur_count)
                   VALUES ($1, $2, $3, 1)
                   ON CONFLICT (work_a, work_b) DO UPDATE SET
                       cooccur_count = fic_bookmark_cooccur.cooccur_count + 1,
                       last_updated  = CURRENT_TIMESTAMP"#,
            )
            .bind(work_a)
            .bind(work_b)
            .bind(site_domain)
            .execute(&self.db)
            .await?;
        }
    }
    Ok(())
}
```

For a user who bookmarked 5 stories, this creates C(5,2) = 10 pairs. Each pair gets its co-occurrence count incremented by 1. The `work_a < work_b` ordering is enforced by the code (sorting the pair) and validated by the `CHECK` constraint in the database schema.

The `ON CONFLICT DO UPDATE` means: if this pair already exists, add 1 to the count. If it doesn't, insert with count 1. This is how the co-occurrence table grows over time—each user's bookmark set contributes to the pairwise counts.

The complexity is O(n²) for n bookmarked stories. For typical users (10-50 bookmarks), this is fast. For power users with hundreds of bookmarks, it could take a while—but that's acceptable for a background worker.

### Site-Specific Rate Limits

Different sites have different tolerance for scraping. AO3 is relatively robust but still has rate limits. FanFiction.net is older and more fragile. XenForo forums (SpaceBattles, Sufficient Velocity) have their own conventions.

The rate limits are configured per domain:

```toml
[recommender]
default_delay_secs = 5
site_rate_limits = { "archiveofourown.org" = 3, "fanfiction.net" = 10 }
```

AO3 gets 3 seconds between requests (it's a modern Rails app with decent capacity). FanFiction.net gets 10 seconds (it's an older PHP app that struggles under load). The default for unknown sites is 5 seconds.

The `rec_max_favourite_pages` config option limits how many pages of bookmarks we crawl per query. More pages means more data but slower collection. For most sites, 5-10 pages captures the majority of bookmarkers.

The `rec_max_user_favourite_pages` config option limits how many pages of a user's bookmarks we crawl. This is typically lower (3-5 pages) because we don't need a user's complete reading history—just enough to establish co-occurrence patterns.

⚠️ **Watch Out**: Rate limits are per-process, not per-IP. If FicHub runs multiple worker instances (e.g., in a Docker Compose setup), each one independently enforces its own rate limit. This could result in a site seeing faster request rates than intended—for example, two workers each waiting 3 seconds could send requests 1.5 seconds apart on average. For production deployments, consider using a distributed rate limiter (Redis-based token bucket) instead of the per-process atomic CAS approach.

### How the Worker Fits in the System

The worker is triggered in a few ways:

1. **On new story scrape**: When the scraper fetches a new story, it can enqueue it for collection. This is the most common trigger—new stories need bookmark data to appear in recommendations.

2. **On recommendation request**: If a user requests recommendations for a story not yet in the database, the handler enqueues it. This is a lazy-loading approach: we don't proactively scrape every story, only those that users actually ask about.

3. **On suggestion submission**: If a user suggests a recommendation involving a story not in the database, both stories are enqueued. This ensures that community suggestions can eventually be processed even for stories we haven't scraped yet.

4. **Periodic batch jobs**: An admin might enqueue popular stories for re-scraping to refresh their bookmark data. Bookmark counts change over time as new readers discover old stories, so periodic refreshes keep the data current.

This creates a self-reinforcing cycle: more requests → more stories enqueued → more data collected → better recommendations → more requests.

The worker's design also supports incremental updates. When we re-process a story, we don't re-scrape everything from scratch. The `ON CONFLICT DO NOTHING` on bookmark inserts and `ON CONFLICT DO UPDATE` on co-occurrence inserts mean we only process new data. Existing bookmarks are skipped, and co-occurrence counts are incremented (not reset). This makes refreshes efficient—you only pay for the new data.

### Error Handling and Resilience

The worker is designed to be resilient. If a single story fails to process (network error, parsing error, database error), the error is logged and the worker moves on. It doesn't crash, it doesn't retry endlessly, and it doesn't block other stories.

```rust
match serde_json::from_str::<QueueItem>(&item_str) {
    Ok(item) => {
        debug!("Processing {} from {}", item.url_id, domain);
        if let Err(e) = self.process_work(item).await {
            error!("Error processing work on {}: {}", domain, e);
        }
    }
    Err(e) => {
        warn!("Invalid queue item on {}: {}", domain, e);
    }
}
```

The `if let Err(e)` pattern means: if processing fails, log the error and continue. The next iteration of the loop will pick up the next item. Failed items are not re-queued automatically—they're lost. For a production system, you might want a dead-letter queue for failed items, but for FicHub's scale, logging and moving on is sufficient. The stories will eventually be re-enqueued by other triggers.

Network errors are the most common failure mode. Fanfiction sites occasionally go down, return errors, or serve unexpected HTML. The `SiteFetcher` implementations should handle these gracefully—returning empty results rather than panicking. The worker treats empty results as "no data available" rather than "error"—the story simply won't contribute to co-occurrence counts until it's re-processed successfully.

🧪 **Try It Yourself**: If a user has bookmarked 20 stories, how many co-occurrence pairs are created? The answer is C(20,2) = 20 × 19 / 2 = 190 pairs. That's 190 database updates for a single user. Now imagine a power user with 50 bookmarks—that's 1,225 pairs. This is why the rate limiter and background processing matter. These updates happen asynchronously, not in the API request path.

---

## Chapter 23: Community Suggestions

### Letting Users Suggest Recommendations

Collaborative filtering is powerful, but it's passive. It watches what people do, not what they think. Sometimes a reader finishes a story and *knows* exactly what to recommend next—but there's no bookmark data connecting the two.

Maybe the two stories are in different fandoms, so the same readers don't overlap. Maybe one story is brand-new and hasn't accumulated enough bookmarks yet. Maybe the connection is thematic—similar pacing, similar character dynamics, similar emotional arc—and that connection only a human reader would notice.

FicHub lets users submit recommendations manually. This is the community suggestions feature—a way for readers to say "I read this, and I think you'd love that."

### The Suggestion Table in the Database

The `recommendation_suggestions` table stores user-submitted links between stories:

```sql
CREATE TABLE IF NOT EXISTS recommendation_suggestions (
    id BIGSERIAL PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    suggested_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    submitted_by_ip INET NOT NULL,
    comment TEXT,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(url_id, suggested_url_id, submitted_by_ip)
);
```

Key points:

- **`url_id`**: The story being recommended *for* (the seed). This is the story the user is currently reading.
- **`suggested_url_id`**: The story being recommended (the suggestion). This is the story they think others should read.
- **`submitted_by_ip`**: Who submitted it. We use IP addresses for identity, not user accounts (FicHub doesn't have a login system). This is a pragmatic choice for a tool that values simplicity. IP addresses aren't perfect identity—multiple people behind the same NAT share an IP—but they're good enough for a small community.
- **`comment`**: Optional text like "Great similar pacing" or "Same universe" or "If you liked the worldbuilding, this has even more." Comments add semantic context that the algorithm can't extract. They also help other voters decide whether to upvote the suggestion.
- **`UNIQUE(url_id, suggested_url_id, submitted_by_ip)`**: Prevents duplicate suggestions from the same IP. If someone tries to suggest the same story twice, it upserts (updates the comment and timestamp).

The `ON DELETE CASCADE` ensures that if a story is deleted from `fic_info`, all its suggestions and votes are automatically cleaned up. Referential integrity at the database level means we never have orphaned suggestions pointing to deleted stories.

The table is intentionally simple. There's no moderation status column, no visibility flag, no "approved" boolean. The philosophy is: let the community self-curate through voting. Bad suggestions don't need to be deleted—they just get downvoted and sink to the bottom. This keeps the system simple and avoids the need for a moderation queue.

### The Suggestion Struct

The `Suggestion` struct represents a suggestion in the API response:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub id: i64,
    pub suggested_url_id: String,
    pub comment: Option<String>,
    pub net_votes: i32,
    pub created: Option<DateTime<Utc>>,
}
```

It includes the suggestion's ID (for voting), the suggested story's ID, any comment, the net vote count (upvotes minus downvotes), and when it was created. The `net_votes` field is computed dynamically by joining with the votes table—it's not stored in the suggestions table itself.

### The Submit Suggestion Function

When a user submits a suggestion, the backend calls `submit_suggestion`:

```rust
pub async fn submit_suggestion(
    db: &PgPool,
    url_id: &str,
    suggested_url_id: &str,
    voter_ip: &str,
    comment: Option<&str>,
) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        r#"INSERT INTO recommendation_suggestions
               (url_id, suggested_url_id, submitted_by_ip, comment)
           VALUES ($1, $2, $3::inet, $4)
           ON CONFLICT (url_id, suggested_url_id, submitted_by_ip)
           DO UPDATE SET comment = EXCLUDED.comment, created = CURRENT_TIMESTAMP
           RETURNING id"#,
    )
    .bind(url_id)
    .bind(suggested_url_id)
    .bind(voter_ip)
    .bind(comment)
    .fetch_one(db)
    .await?;

    Ok(row.0)
}
```

The `ON CONFLICT ... DO UPDATE` means: if this IP already suggested this pair, update the comment and timestamp rather than rejecting it. This is a nice UX touch—users can revise their suggestions without creating duplicates.

The `::inet` cast is important. PostgreSQL's `INET` type validates that the value is a proper IP address (IPv4 or IPv6). If someone passes garbage, the database will reject it with a clear error rather than storing invalid data.

The function returns the suggestion's ID, which the frontend uses to allow immediate voting on the new suggestion.

### Validating That Both Fics Exist

Before storing a suggestion, the API handler validates that both stories exist in the database. This is the `suggest_handler` in `routes.rs`:

```rust
pub async fn suggest_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SuggestBody>,
) -> Result<Json<Value>, AppError> {
    if body.url_id.is_empty() || body.suggested_url.is_empty() {
        return Ok(Json(json!({
            "err": -1,
            "msg": "url_id and suggested_url are required"
        })));
    }

    // Resolve the suggested_url to a url_id via the scraper
    let scraper = state.scraper_registry.find_scraper(&body.suggested_url)
        .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", body.suggested_url)))?;
    let meta = scraper.lookup(&state.http_client, &body.suggested_url).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;
    let suggested_url_id = meta.url_id;

    let seed_exists = queries::get_fic_info(&state.db, &body.url_id).await?.is_some();
    let suggestion_exists = queries::get_fic_info(&state.db, &suggested_url_id).await?.is_some();

    if !seed_exists {
        state.collection_worker.enqueue(&body.url_id, "unknown", "unknown").await?;
        return Ok(Json(json!({
            "err": -5,
            "msg": "seed fic not found in database — enqueued for collection"
        })));
    }

    if !suggestion_exists {
        state.collection_worker.enqueue(&suggested_url_id, "unknown", "unknown").await?;
        return Ok(Json(json!({
            "err": -5,
            "msg": "suggested fic not yet collected — enqueued for processing"
        })));
    }

    // Submit the suggestion
    let suggestion_id = submit_suggestion(
        &state.db,
        &body.url_id,
        &suggested_url_id,
        "0.0.0.0",
        body.comment.as_deref(),
    ).await?;

    Ok(Json(json!({
        "err": 0,
        "suggestion_id": suggestion_id,
    })))
}
```

If either story isn't in the database, we don't reject the suggestion outright. Instead, we **enqueue the missing story** for background collection. Once the worker scrapes it and stores its metadata, the suggestion can be processed. This is a graceful degradation—the user's intent is preserved even if the data isn't ready yet.

The `find_scraper` + `lookup` pattern resolves the URL to a canonical `url_id`. The user provides a URL (like `https://archiveofourown.org/works/12345`), and the scraper extracts the canonical ID (`12345` in this case, though the actual format is more complex).

### The Suggestion Modal in the Frontend

The suggestion flow in the frontend is designed to be frictionless. Here's the detailed user experience:

1. **User is on a story's page.** They've just read "Midnight Sun" and want to recommend "Eclipse Reimagined" to future readers.

2. **They click "Suggest a Recommendation."** This is a secondary action button—visible but not prominent. It's usually placed below the main recommendation list, so users only see it after browsing the existing recommendations.

3. **A modal appears** with:
   - A title: "Suggest a Recommendation for Midnight Sun"
   - An input field for the suggested story's URL. Placeholder text: "Paste the URL of a story you'd recommend..."
   - A text area for an optional comment. Placeholder text: "Why do you think this is a good match? (optional)"
   - A character counter for the comment (max 500 characters).
   - A "Submit" button (disabled until a URL is entered).
   - A "Cancel" link.

4. **The user pastes a URL.** The frontend might do quick client-side validation—checking that the URL looks like a valid fanfiction URL (contains "archiveofourown.org/works/" or "fanfiction.net/s/" etc.). If the URL doesn't look right, a gentle hint appears: "This doesn't look like a fanfiction URL. Please check and try again."

5. **They write a comment.** The comment is optional but encouraged. Good comments help other voters decide whether to upvote. The frontend might show a subtle prompt: "Tip: explain why this is a good match—it helps others vote."

6. **On submit, the frontend calls `POST /api/v0/recommendations/suggest`.** While waiting for the response, the submit button shows a loading spinner and is disabled to prevent double-submission.

7. **The backend resolves the URL to a `url_id`, validates both stories exist, and stores the suggestion.** If either story is missing from the database, the backend enqueues it for collection and returns an error message like "This story isn't in our database yet—we've queued it for collection."

8. **The modal shows a success message.** "Your suggestion has been submitted! Other readers can now vote on it." The modal auto-closes after 2 seconds.

9. **The suggestion appears in the community suggestions list.** The user can immediately see their suggestion with 0 votes and start voting on it themselves (yes, you can upvote your own suggestion—it's a feature, not a bug).

The frontend uses the `SuggestBody` struct:

```rust
pub struct SuggestBody {
    pub url_id: String,
    pub suggested_url: String,
    pub comment: Option<String>,
}
```

Note that the user provides the *URL* of the suggested story, not a `url_id`. The backend resolves the URL to a `url_id` using the scraper registry—the same mechanism used when someone requests an export. This means users don't need to know FicHub's internal ID system. They just paste a URL from wherever they found the story.

The URL resolution is important for user experience. Fanfiction URLs come in many forms:
- `https://archiveofourown.org/works/12345`
- `https://www.fanfiction.net/s/12345/1/`
- `https://forums.spacebattles.com/threads/story-name.12345/`
- `https://www.fictionpress.com/s/12345/1/`

The scraper registry knows how to parse each format and extract the canonical `url_id`. The user doesn't need to worry about URL normalization—they just paste whatever they have.

### Preventing Spam

Without user accounts, how do we prevent spam? Several layers work together:

1. **IP-based deduplication**: The `UNIQUE(url_id, suggested_url_id, submitted_by_ip)` constraint prevents the same IP from submitting the same pair twice. If someone wants to change their comment, the suggestion is updated rather than duplicated.

2. **Rate limiting** (in the frontend): The suggestion button can be throttled—e.g., one suggestion per 30 seconds. This prevents rapid-fire spam without being too restrictive for normal use.

3. **Vote-based filtering**: Low-voted suggestions appear at the bottom. High-voted ones rise to the top. Spammy suggestions that nobody upvotes simply sink into obscurity. The community self-curates.

4. **Manual moderation**: Admins can delete suggestions. The `ON DELETE CASCADE` handles cleanup—deleting a suggestion automatically removes its votes.

This isn't perfect, but it's sufficient for a small-to-medium community. The voting system does most of the heavy lifting—spam that nobody upvotes simply doesn't matter. And the IP-based deduplication prevents the most obvious abuse: one person flooding the system with hundreds of duplicate suggestions.

### How Suggestions Feed the Engine

Community suggestions don't just appear in a list—they feed back into the recommendation engine's scoring. When computing live recommendations, the engine queries `get_community_votes` to find net votes for each candidate:

```rust
async fn get_community_votes(
    db: &PgPool,
    candidates: &[CandidateScore],
) -> Result<HashMap<String, i32>, AppError> {
    if candidates.is_empty() {
        return Ok(HashMap::new());
    }

    let url_ids: Vec<String> = candidates.iter().map(|c| c.url_id.clone()).collect();

    let rows: Vec<(String, Option<i32>)> = sqlx::query_as(
        r#"SELECT s.suggested_url_id, SUM(v.vote)::INT AS net_votes
           FROM recommendation_suggestions s
           JOIN recommendation_votes v ON v.suggestion_id = s.id
           WHERE s.suggested_url_id = ANY($1)
           GROUP BY s.suggested_url_id"#,
    )
    .bind(&url_ids)
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(url_id, net)| (url_id, net.unwrap_or(0)))
        .collect())
}
```

The `ANY($1)` operator is PostgreSQL's way of doing `IN` with a dynamic list. The `JOIN` between suggestions and votes ensures we only count suggestions that have been voted on. The `GROUP BY` aggregates votes per suggestion.

This creates a feedback loop: good suggestions get upvoted → votes boost the recommendation's score → higher ranking → more visibility → more upvotes. Bad suggestions sink to the bottom naturally.

🧪 **Try It Yourself**: What happens if a user suggests a story that's already been recommended by the collaborative filtering engine? Nothing special—it's stored as a separate suggestion. But if other users upvote it, it gets a voting boost that might push it higher in the results. The collaborative and community signals reinforce each other.

---

## Chapter 24: Voting and Scoring

### Upvotes and Downvotes

The voting system lets the community curate recommendations. Anyone can upvote or downvote a suggestion, and those votes affect how prominently the suggestion appears in results.

It's a simple binary: **+1** for upvote, **-1** for downvote. No scales, no stars, no nuanced "3.5 out of 5" judgments. Binary voting is easy to implement, easy to understand, and produces clear signals.

Why binary? Because nuanced ratings are hard. Asking someone to rate a recommendation on a 5-star scale introduces decision fatigue—"Is this a 3 or a 4? What's the difference?" Binary is instant: thumbs up or thumbs down. The aggregate signal is clear, and the user experience is frictionless.

The voting system also has a subtle psychological effect: because votes are visible to everyone, they create social proof. A suggestion with +7 votes looks more trustworthy than one with +1. This helps voters make quick decisions—they can see what the community thinks before forming their own opinion. It's the same mechanism that makes Reddit upvotes and YouTube likes so influential.

There's also an important asymmetry in how we use votes. Upvotes *boost* recommendations—the boost formula uses `max(0, net_votes)`, so only positive votes increase the score. Downvotes *don't punish*—they just prevent the boost. A suggestion with -5 votes gets no boost (multiplier of 1.0), but it doesn't get penalized below its collaborative filtering baseline. This design choice reflects the philosophy that downvotes are a signal of "this isn't a good recommendation," not "this is a bad story."

### The Vote Table

The `recommendation_votes` table stores every vote:

```sql
CREATE TABLE IF NOT EXISTS recommendation_votes (
    suggestion_id BIGINT NOT NULL REFERENCES recommendation_suggestions(id) ON DELETE CASCADE,
    voter_ip INET NOT NULL,
    vote SMALLINT NOT NULL CHECK (vote IN (-1, 1)),
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (suggestion_id, voter_ip)
);
```

Key design choices:

- **`PRIMARY KEY (suggestion_id, voter_ip)`**: Each IP can only have one vote per suggestion. If someone changes their mind, the vote is updated, not duplicated. This prevents vote-stuffing while allowing opinion changes.

- **`CHECK (vote IN (-1, 1))`**: The database rejects any value that isn't -1 or 1. No 0s, no 2s, no fractions. This is a hard constraint that prevents invalid data at the database level.

- **`ON DELETE CASCADE`**: If a suggestion is deleted, its votes are automatically removed. No orphaned votes.

- **`SMALLINT`**: The vote value is a 2-byte integer. Overkill for storing -1 or 1, but it's the smallest integer type PostgreSQL supports. The `SMALLINT` type saves space compared to `INT4` at scale—millions of votes add up.

The `INET` type for `voter_ip` ensures we store valid IP addresses. PostgreSQL's `INET` supports both IPv4 and IPv6, so it's future-proof.

### The Cast Vote Function

Casting a vote is a two-step operation:

```rust
pub async fn cast_vote(
    db: &PgPool,
    suggestion_id: i64,
    voter_ip: &str,
    vote_value: i16,
) -> Result<i32, AppError> {
    let vote = vote_value.clamp(-1, 1);

    sqlx::query(
        r#"INSERT INTO recommendation_votes (suggestion_id, voter_ip, vote)
           VALUES ($1, $2::inet, $3)
           ON CONFLICT (suggestion_id, voter_ip)
           DO UPDATE SET vote = EXCLUDED.vote, created = CURRENT_TIMESTAMP"#,
    )
    .bind(suggestion_id)
    .bind(voter_ip)
    .bind(vote)
    .execute(db)
    .await?;

    let row: (Option<i32>,) = sqlx::query_as(
        r#"SELECT SUM(vote)::INT FROM recommendation_votes WHERE suggestion_id = $1"#,
    )
    .bind(suggestion_id)
    .fetch_one(db)
    .await?;

    Ok(row.0.unwrap_or(0))
}
```

The `clamp(-1, 1)` ensures only valid values are stored—defense in depth. The `ON CONFLICT ... DO UPDATE` means: if this IP already voted on this suggestion, replace the old vote with the new one. This is the "change your vote" behavior—users aren't locked into their first impression.

After storing the vote, the function immediately queries the new net score (`SUM(vote)`) and returns it. This gives the frontend the updated score in the same response, enabling optimistic UI updates.

The net score calculation is straightforward:

```sql
SELECT SUM(vote)::INT FROM recommendation_votes WHERE suggestion_id = $1
```

If 5 people upvoted (+1 each) and 2 downvoted (-1 each), the sum is 3. The `COALESCE` isn't needed here because the suggestion_id is guaranteed to exist (it's a foreign key), but the `unwrap_or(0)` handles the edge case of a suggestion with zero votes.

### Calculating Net Scores

The net score for a suggestion is simply:

```
net_votes = SUM of all votes for this suggestion
          = upvotes - downvotes
```

If 5 people upvoted and 2 downvoted, the net score is 3. If 3 upvoted and 7 downvoted, it's -4. The net score is always an integer, and it can be negative.

The net score is computed in two places:

1. **In `cast_vote`**: After storing a vote, we compute the new total and return it immediately.
2. **In `get_community_suggestions`**: When listing suggestions, we join with the votes table to get each suggestion's net score.

The `get_community_suggestions` function:

```rust
pub async fn get_community_suggestions(
    db: &PgPool,
    url_id: &str,
) -> Result<Vec<Suggestion>, AppError> {
    let rows: Vec<SuggestionRow> = sqlx::query_as::<_, SuggestionRow>(
        r#"SELECT s.id, s.suggested_url_id, s.comment,
                  COALESCE(v.net, 0) AS net_votes, s.created
           FROM recommendation_suggestions s
           LEFT JOIN (
               SELECT suggestion_id, SUM(vote)::INT AS net
               FROM recommendation_votes
               GROUP BY suggestion_id
           ) v ON v.suggestion_id = s.id
           WHERE s.url_id = $1
           ORDER BY net_votes DESC, s.created DESC"#,
    )
    .bind(url_id)
    .fetch_all(db)
    .await?;

    let suggestions = rows
        .into_iter()
        .map(|r| Suggestion {
            id: r.id,
            suggested_url_id: r.suggested_url_id,
            comment: r.comment,
            net_votes: r.net_votes,
            created: r.created,
        })
        .collect();

    Ok(suggestions)
}
```

The `LEFT JOIN` is crucial—it ensures suggestions with zero votes still appear (with `net_votes = 0`). The `COALESCE(v.net, 0)` handles the case where no votes exist yet. And `ORDER BY net_votes DESC, s.created DESC` puts the most upvoted suggestions first, with newest as the tiebreaker.

The subquery `(SELECT suggestion_id, SUM(vote)::INT AS net FROM recommendation_votes GROUP BY suggestion_id)` pre-aggregates all votes, and the outer query joins it with the suggestions. This is more efficient than computing the sum for each suggestion individually.

### The Voting Boost in Recommendations

Community votes don't just affect the suggestion list—they feed into the recommendation engine's scoring.

When computing live recommendations, the engine fetches net votes for all candidate stories and applies a logarithmic boost:

```rust
let net_votes = get_community_votes(db, &candidates).await?;

let gamma = config.rec_voting_boost_gamma;
for c in &mut candidates {
    let nv = net_votes.get(&c.url_id).copied().unwrap_or(0);
    c.community_score = nv;
    // Boost: score * (1.0 + gamma * ln(1 + max(0, net_votes)))
    let boost = 1.0 + gamma * (nv.max(0) as f64).ln_1p();
    c.score *= boost;
}
```

The `get_community_votes` function aggregates votes across all suggestions for each candidate:

```rust
async fn get_community_votes(
    db: &PgPool,
    candidates: &[CandidateScore],
) -> Result<HashMap<String, i32>, AppError> {
    if candidates.is_empty() {
        return Ok(HashMap::new());
    }

    let url_ids: Vec<String> = candidates.iter().map(|c| c.url_id.clone()).collect();

    let rows: Vec<(String, Option<i32>)> = sqlx::query_as(
        r#"SELECT s.suggested_url_id, SUM(v.vote)::INT AS net_votes
           FROM recommendation_suggestions s
           JOIN recommendation_votes v ON v.suggestion_id = s.id
           WHERE s.suggested_url_id = ANY($1)
           GROUP BY s.suggested_url_id"#,
    )
    .bind(&url_ids)
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(url_id, net)| (url_id, net.unwrap_or(0)))
        .collect())
}
```

The logarithmic boost formula deserves detailed explanation:

```
boost = 1.0 + gamma × ln(1 + max(0, net_votes))
```

- **`max(0, net_votes)`**: Negative votes don't *reduce* the score—they just don't boost it. A story with -5 votes gets no boost (multiplier of 1.0). This prevents downvoted stories from being actively punished beyond their natural collaborative ranking. The philosophy is: downvotes are a signal that "this isn't a good recommendation," but they shouldn't make the story *worse* than its collaborative filtering score suggests.

- **`ln(1 + ...)`**: Natural logarithm of 1 + the vote count. This gives diminishing returns: going from 0 to 10 votes gives a bigger boost than going from 100 to 110. The `1 +` ensures we don't take `ln(0)` (which is undefined).

- **`gamma`**: A tuning parameter from the config. Higher values make votes matter more. Typical values are 0.3 to 0.7.

For example, with `gamma = 0.5`:

| Net Votes | Boost Multiplier |
|-----------|-----------------|
| 0         | 1.00            |
| 1         | 1.35            |
| 5         | 1.80            |
| 10        | 1.98            |
| 20        | 2.25            |
| 50        | 2.45            |
| 100       | 2.75            |
| 500       | 3.46            |

The first few votes have the biggest impact. Going from 0 to 5 votes increases the boost by 80%. Going from 100 to 105 votes increases it by less than 2%. This is the "early votes matter most" philosophy—it ensures that new suggestions can quickly rise if the community likes them, without allowing popular suggestions to become unstoppable.

### Voting UI in the Frontend

The voting interface is simple and immediate. Each suggestion card has an upvote button (▲) and a downvote button (▼), with the current net score displayed between them.

The design follows a common pattern seen in Reddit, Hacker News, and Stack Overflow:

```
┌─────────────────────────────────────────────┐
│  ▲                                          │
│  7     "Same author, similar pacing!"       │
│  ▼     — suggested for "Midnight Sun"       │
│                                              │
│  ▲                                          │
│  2     "If you liked the worldbuilding..."  │
│  ▼     — suggested for "Midnight Sun"       │
│                                              │
│  ▲                                          │
│ -1     "Better worldbuilding IMO"           │
│  ▼     — suggested for "Midnight Sun"       │
└─────────────────────────────────────────────┘
```

The vote buttons are large enough to tap on mobile (important—many users browse fanfiction on their phones). The score is prominently displayed between the buttons, using green for positive scores, red for negative, and gray for zero.

The API endpoint:

```rust
pub async fn vote_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<VoteBody>,
) -> Result<Json<Value>, AppError> {
    if body.vote != 1 && body.vote != -1 {
        return Ok(Json(json!({
            "err": -1,
            "msg": "vote must be 1 (upvote) or -1 (downvote)"
        })));
    }

    let new_score = cast_vote(
        &state.db,
        body.suggestion_id,
        "0.0.0.0",
        body.vote as i16,
    ).await?;

    Ok(Json(json!({
        "err": 0,
        "new_score": new_score,
    })))
}
```

The `VoteBody` struct:

```rust
pub struct VoteBody {
    pub suggestion_id: i64,
    pub vote: i32,
}
```

The validation at the top (`vote != 1 && vote != -1`) catches invalid values before they reach the database. This is a defense-in-depth measure—the database also has a `CHECK` constraint, but early rejection saves a round trip and provides a clearer error message.

The response includes `new_score`—the updated net vote count. This is essential for optimistic UI updates. When the frontend sends a vote request, it can immediately update the displayed score without waiting for the server. If the server returns a different score than expected (rare, but possible if someone else voted simultaneously), the frontend corrects to the server's value.

### Loading Suggestions: The votes_handler

The frontend loads suggestions and their current votes when the user views a story's recommendations:

```rust
pub async fn votes_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<VotesQuery>,
) -> Result<Json<Value>, AppError> {
    if params.url_id.is_empty() {
        return Ok(Json(json!({
            "err": -1,
            "msg": "url_id is required"
        })));
    }

    let suggestions = get_community_suggestions(&state.db, &params.url_id).await?;

    Ok(Json(json!({
        "err": 0,
        "url_id": params.url_id,
        "suggestions": suggestions,
    })))
}
```

The `GET /api/v0/recommendations/votes` endpoint returns all suggestions and their net scores for a given story. The frontend uses this to populate the community suggestions section, showing suggestions sorted by votes. This is typically called once when the page loads, not on every vote—the frontend manages vote state locally after the initial load.

### Optimistic Voting: Immediate UI Feedback

When a user clicks the upvote button, the frontend doesn't wait for the server response. It immediately:

1. Increments the displayed score by 1.
2. Highlights the upvote button to show the user's choice.
3. Sends the vote request to the server in the background.

If the server returns an error (network failure, rate limit, database error), the frontend rolls back—decrementing the score and un-highlighting the button. This is **optimistic UI**—assuming success and handling failure as an edge case.

Why optimistic? Because voting is a low-stakes action. If the vote fails 1 in 100 times, it's better to show immediate feedback and occasionally roll back than to make every vote feel sluggish. The user sees instant results, and the rare failure is handled gracefully.

The flow:

```
User clicks ▲
  → Frontend: score += 1, highlight ▲
  → Background: POST /api/v0/recommendations/vote {suggestion_id: 42, vote: 1}
  → Server: stores vote, returns new_score = 7
  → Frontend: score = 7 (matches? great. no adjustment needed.)
  
  OR (failure path)
  
  → Server: returns error
  → Frontend: score -= 1, un-highlight ▲
  → Show subtle error toast: "Vote couldn't be recorded"
```

The key insight is that the frontend doesn't need the server's response to update the UI—it can calculate the new score locally. The server's response is just a confirmation. If the confirmation says something different (rare, but possible if someone else voted simultaneously), the frontend updates to match.

Here's how this looks in practice. The frontend maintains a local state for each suggestion:

```javascript
// Pseudocode for the voting logic
function handleVote(suggestionId, direction) {
    const suggestion = suggestions.find(s => s.id === suggestionId);
    const previousScore = suggestion.net_votes;
    
    // Optimistic update
    suggestion.net_votes += direction;
    suggestion.userVote = direction;
    
    // Send to server
    fetch('/api/v0/recommendations/vote', {
        method: 'POST',
        body: JSON.stringify({
            suggestion_id: suggestionId,
            vote: direction
        })
    })
    .then(res => res.json())
    .then(data => {
        // Reconcile with server state
        suggestion.net_votes = data.new_score;
    })
    .catch(err => {
        // Rollback on error
        suggestion.net_votes = previousScore;
        suggestion.userVote = 0;
        showToast("Vote couldn't be recorded. Please try again.");
    });
}
```

The reconciliation step is important. Even if the optimistic update is correct, the server's response is the source of truth. If another user voted between our optimistic update and the server response, the scores might not match. The frontend always adopts the server's value.

⚠️ **Watch Out**: Optimistic UI requires careful state management. If the user clicks upvote, then immediately clicks downvote before the first request completes, you could end up with race conditions. The frontend needs to cancel or queue the previous request. A simple approach is to track the pending vote and ignore stale responses. Another approach is to debounce the vote requests—only send the most recent vote to the server.

### The netScore Calculation

The net score is the backbone of the voting system. Let's trace how it flows through the system:

1. **Storage**: Each vote is stored as +1 or -1 in `recommendation_votes`.
2. **Aggregation**: `SUM(vote)` across all votes for a suggestion gives the net score.
3. **Display**: The suggestions list shows `net_votes` sorted descending.
4. **Engine integration**: `get_community_votes` aggregates votes for all candidates and feeds them into the boost formula.

The net score serves double duty: it's a social signal (this suggestion is well-liked) and an engine signal (this recommendation gets a boost). The same number drives both the UI and the algorithm.

```rust
pub async fn votes_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<VotesQuery>,
) -> Result<Json<Value>, AppError> {
    if params.url_id.is_empty() {
        return Ok(Json(json!({
            "err": -1,
            "msg": "url_id is required"
        })));
    }

    let suggestions = get_community_suggestions(&state.db, &params.url_id).await?;

    Ok(Json(json!({
        "err": 0,
        "url_id": params.url_id,
        "suggestions": suggestions,
    })))
}
```

The `GET /api/v0/recommendations/votes` endpoint returns all suggestions and their net scores for a given story. The frontend uses this to populate the community suggestions section, showing suggestions sorted by votes.

### The Suggestion Struct in the API

When a user requests suggestions for a story, the API returns JSON like this:

```json
{
    "err": 0,
    "url_id": "abc123",
    "suggestions": [
        {
            "id": 42,
            "suggested_url_id": "def456",
            "comment": "Same author, similar pacing!",
            "net_votes": 7,
            "created": "2024-01-15T10:30:00Z"
        },
        {
            "id": 43,
            "suggested_url_id": "ghi789",
            "comment": null,
            "net_votes": 2,
            "created": "2024-01-16T14:20:00Z"
        },
        {
            "id": 44,
            "suggested_url_id": "jkl012",
            "comment": "Better worldbuilding IMO",
            "net_votes": -1,
            "created": "2024-01-17T09:00:00Z"
        }
    ]
}
```

The suggestions are sorted by `net_votes DESC, created DESC`—most upvoted first, newest first as tiebreaker. This means the best community recommendations appear at the top. A suggestion with -1 votes appears at the bottom—visible but clearly not favored by the community.

### How Voting Feeds the Engine

Let's trace the complete flow from a user voting to that vote affecting recommendations:

1. User upvotes suggestion #42 (linking Story A to Story B).
2. `cast_vote` stores `+1` in `recommendation_votes` for suggestion #42.
3. The net vote count for Story B increases by 1.
4. Next time someone requests recommendations for any story, and Story B is a candidate, `get_community_votes` picks up the new net score.
5. The boost formula applies: `score *= (1.0 + gamma × ln_1p(max(0, net_votes)))`.
6. Story B's final score is higher, so it ranks higher in the results.

This creates a virtuous cycle: good suggestions get upvoted, which boosts their ranking, which makes them more visible, which leads to more upvotes. Meanwhile, bad suggestions get downvoted and sink to the bottom.

The key insight is that votes influence *all* recommendations for a story, not just the specific suggestion. If Story B is upvoted as a recommendation for Story A, and Story B is also a collaborative filtering candidate for Story C, the vote boost applies to both recommendations. The voting system and the collaborative filtering system reinforce each other.

🧪 **Try It Yourself**: Trace through the math. A story has a collaborative score of 0.3. It has 8 upvotes and 2 downvotes (net = 6). With `gamma = 0.5`, the boost is `1.0 + 0.5 × ln(1 + 6) = 1.0 + 0.5 × 1.95 = 1.97`. The final score is `0.3 × 1.97 = 0.59`. That's nearly double—votes can significantly shift a recommendation's ranking.

### Putting It All Together

The recommendation system in FicHub is a blend of automated intelligence and human curation:

- **Collaborative filtering** discovers patterns from real reading behavior—what the crowd's bookmarks tell us about taste.
- **Tag fallback** handles new stories that don't have enough data yet—matching by author and title keywords.
- **Community suggestions** let readers contribute their expertise—their "you'll love this" moments.
- **Voting** surfaces the best suggestions and feeds signals back into the engine.

No single signal dominates. The blended approach means recommendations are robust: even if collaborative filtering is weak (cold start), tag matching and community votes can carry the day. Even if the community hasn't voted on a recommendation yet, collaborative filtering provides a solid baseline. Even if there's no collaborative data at all, community suggestions provide immediate value.

The result is a recommendation engine that feels alive—it learns from the community, surfaces human judgment, and adapts as new stories appear and reader tastes evolve. It's not just an algorithm; it's a conversation between readers, mediated by data.

The beauty of this architecture is its extensibility. The `SiteFetcher` trait makes adding new sites trivial—implement two methods and you're done. The Redis queue makes scaling the worker horizontal—just run more instances. The voting system makes community curation self-sustaining—no moderator intervention needed. And the recommendation engine itself—the heart of it all—is just SQL queries and a few blending formulas. No magic. Just math, data, and a community that cares about good stories.

### The API Surface: Connecting Everything

The recommendation system exposes three API endpoints that tie all these components together:

1. **`GET /api/v0/recommendations`** — The main endpoint. Accepts a `url_id` or URL, optionally a `site_domain` filter, and a count `n`. Returns up to `n` recommendations with scores and metadata. This endpoint calls `get_recommendations` on the engine, which handles cache lookup and live computation.

2. **`POST /api/v0/recommendations/suggest`** — Submit a community suggestion. Accepts a seed `url_id`, a suggested URL, and an optional comment. Resolves the URL, validates both stories exist, and stores the suggestion. Returns the suggestion ID.

3. **`POST /api/v0/recommendations/vote`** — Cast a vote on a suggestion. Accepts a `suggestion_id` and a `vote` value (1 or -1). Returns the new net score.

4. **`GET /api/v0/recommendations/votes`** — List all suggestions and their votes for a given story. Returns suggestions sorted by net votes.

These four endpoints cover the entire recommendation workflow: request recommendations, suggest new ones, vote on suggestions, and view community feedback. The frontend consumes all four, creating a rich, interactive recommendation experience.

The `AppState` struct holds references to all the components:

```rust
pub struct AppState {
    pub db: PgPool,
    pub recommender_engine: RecommendationEngine,
    pub collection_worker: CollectionWorker,
    pub config: Config,
    pub http_client: Client,
    pub scraper_registry: Arc<ScraperRegistry>,
}
```

Each request handler accesses the shared state via Axum's `State` extractor. The `recommender_engine` handles recommendation computation, the `collection_worker` handles background data collection, and the `config` provides tuning parameters. This clean separation of concerns makes each component independently testable and replaceable.

---

## Summary

Part 5 built FicHub's recommendation engine from the ground up—a system that turns passive bookmark data into genuinely useful recommendations.

- **Chapter 21** explained collaborative filtering: the Jaccard coefficient measures similarity between stories based on shared bookmarkers. The co-occurrence table tracks which stories appear together, and the SQL-based Jaccard query efficiently ranks candidates. The engine blends collaborative scores with tag-based fallback for new stories (matching by author and title keywords when bookmark data is sparse), and applies a logarithmic voting boost for community favorites. The precomputed cache ensures fast response times for popular stories, while live computation handles cache misses.

- **Chapter 22** built the collection worker: a background process that scrapes bookmark data from fanfiction sites using per-site `SiteFetcher` implementations. The worker uses Redis queues for work distribution, per-site rate limiters (atomic CAS-based) to stay polite, and SHA-256 hashing to protect user privacy. The co-occurrence update uses a nested loop to increment pairwise counts for each user's bookmark set.

- **Chapter 23** added community suggestions: users can manually recommend stories by pasting a URL and adding a comment. The backend resolves URLs, validates both stories exist in the database, and stores the suggestion with IP-based deduplication. Missing stories are automatically queued for collection, creating a seamless bridge between community input and data gathering.

- **Chapter 24** implemented voting and scoring: a binary upvote/downvote system with optimistic UI feedback (instant score updates, server reconciliation). The logarithmic boost formula gives early votes outsized impact while preventing popular suggestions from becoming unstoppable. Votes feed directly into the recommendation engine's scoring pipeline, creating a virtuous cycle where good suggestions rise and bad ones sink.

The recommendation engine is a closed loop: collect data → compute similarity → surface recommendations → gather feedback → improve rankings. Each component feeds the others, creating a system that gets smarter as the community grows. The architecture is deliberately simple—SQL queries and a few blending formulas—but that simplicity is its strength. It's debuggable, fast, and extensible. New sites can be added by implementing a trait. New signals can be blended by adjusting weights. The community can contribute through suggestions and votes. And the whole thing runs on the same PostgreSQL server that powers the rest of FicHub.

---

## The Last 500 Words

The recommendation engine in FicHub represents a philosophy: that the best recommendations come from the community, not from algorithms alone. Collaborative filtering is powerful—it can discover connections that no human curator would spot—but it's fundamentally passive. It observes. It doesn't understand *why* a story is good, only that certain readers tend to bookmark certain combinations.

Community suggestions add the "why." When a reader says "you'll love this because it has the same slow-burn romance" or "this has the same world-building style," they're contributing semantic understanding that no bookmark-based algorithm can extract. And voting amplifies the signal: good suggestions rise, bad ones sink.

The collection worker is the unglamorous foundation. Without it, the engine would starve. Without data flowing in from AO3, FanFiction.net, SpaceBattles, and the other sites, there's nothing to compute. The worker's polite rate limiting, privacy-preserving hashing, and careful co-occurrence tracking ensure that data collection is sustainable and ethical.

The scoring formula—blending collaborative, tag-based, and community signals with a logarithmic voting boost—is deliberately simple. We could add machine learning models, neural collaborative filtering, or content-based NLP analysis. But simplicity has value: it's debuggable, it's fast, and it works. A reader shouldn't have to wait 2 seconds for recommendations. They should click and see results instantly.

As the system matures, the co-occurrence table grows denser, Jaccard coefficients become more meaningful, and the community's collective wisdom surfaces through votes. The engine doesn't need to be perfect—it just needs to be useful. And every time a reader discovers a new favorite through a recommendation, the system has done its job.

The beauty of this architecture is its extensibility. The `SiteFetcher` trait makes adding new sites trivial. The Redis queue makes scaling the worker horizontal. The voting system makes community curation self-sustaining. And the recommendation engine itself—the heart of it all—is just SQL queries and a few blending formulas. No magic. Just math, data, and a community that cares about good stories.

Looking ahead, the natural next step would be to add content-based features—using the story's summary text, tags, and metadata to compute similarity independent of bookmark data. Techniques like TF-IDF or even simple word embeddings could complement collaborative filtering, especially for brand-new stories that haven't accumulated any bookmarks. But that's a future chapter. For now, the collaborative + tag fallback + community voting blend serves FicHub's readers well. It's not perfect, but it's genuinely useful—and in a system built by and for fanfiction readers, that's exactly what matters.
# Part 6: Server and Deployment

---

# Chapter 25: The Full Axum Router

We've spent five parts of this book building the individual pieces of FicHub — scrapers, exporters, caches, a recommendation engine, a tagging system. Now it's time to stitch them all together. The place where everything connects is the Axum router, and it lives in a single file: `src/server.rs`.

If you open that file, you'll see a `build_router` function that returns a `Router`. That's it. That's the heart of the application. Every HTTP request that comes in gets routed through this tree, matched to a handler, and processed with shared application state.

Let's walk through every piece.

## AppState: The Backpack of Shared Resources

Before we look at routes, we need to understand what `AppState` is. In Axum, you can pass a piece of state to all your handlers through a technique called "shared state." FicHub wraps everything it needs in an `Arc<AppState>` and hands it to every route.

Here's the struct straight from the source:

```rust
pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    pub http_client: reqwest::Client,
    pub scraper_registry: Arc<ScraperRegistry>,
    pub cache_semaphores: CacheSemaphores,
    pub rate_limiter: Box<dyn limiter::RateLimiter>,
    pub recommender_engine: RecommendationEngine,
    pub collection_worker: CollectionWorker,
}
```

Each field deserves a moment:

- **`config`** — The `Config` struct loaded from environment variables. It has everything: database URLs, Redis URLs, cache directories, port numbers, rate limit settings, recommender parameters, tagging thresholds. We looked at this in detail in earlier chapters, and it's all pulled from the environment in `Config::from_env()`. The config contains over 30 configuration options, from the core `DATABASE_URL` and `REDIS_URL` to fine-tuned recommender parameters like `REC_VOTING_BOOST_GAMMA` and `REC_CACHE_TTL_HOURS`. Every one of these can be overridden by setting the corresponding environment variable, making FicHub configurable without any code changes.

- **`db`** — A SQLx connection pool for PostgreSQL. This is the persistent storage layer where fic metadata, export logs, recommendations, tags, and all the other data lives. SQLx manages the pool automatically — it creates connections as needed, reuses them across requests, and handles connection timeouts and reconnection. The pool size is tunable, but the defaults work well for most deployments.

- **`redis`** — A multiplexed async Redis connection. Redis serves double duty here: it's used for the token-bucket rate limiter and for caching recommendation results. The "multiplexed" part means multiple logical streams share a single TCP connection — more efficient than opening a new connection for every request.

- **`http_client`** — A `reqwest::Client` with a custom user agent (`fichub.net/0.1.0`) and a 30-second timeout. This is what the scrapers use to fetch pages from AO3, FanFiction.net, and all the other supported sites. The 30-second timeout is generous enough for slow sites but prevents the server from hanging indefinitely on unresponsive upstreams.

- **`scraper_registry`** — An `Arc<ScraperRegistry>` that holds all the registered scrapers (AO3, FFN, FictionPress, HPFanFicArchive, AdultFanFiction, XenForo forums, etc.). When a URL comes in, the registry finds the right scraper for it by matching the URL pattern. The registry is created once at startup and shared across all requests — no need to recreate it for every request.

- **`cache_semaphores`** — A `HashMap` behind a `Mutex` that holds per-fic export semaphores. These prevent the same fic from being exported ten times concurrently — if two people request the same URL at the same time, only one export runs and the other waits for the result. This is a classic "thundering herd" prevention pattern. Without it, if a popular fic gets ten simultaneous requests, you'd make ten identical requests to AO3, generate ten identical EPUBs, and waste a lot of time and bandwidth.

- **`rate_limiter`** — A trait object for the token-bucket rate limiter backed by Redis. It limits how many requests each IP can make. The rate limiter uses a Redis-backed token bucket algorithm, which is distributed across all instances of the application. It can also load datacenter IP ranges from external sources and apply different rate limits to datacenter IPs versus residential IPs.

- **`recommender_engine`** — The recommendation engine we built in Part 5. It computes recommendations based on collaboratively-filtered favourites. The engine is stateless — it takes a database pool reference and computes recommendations on the fly — but having it in the shared state avoids recreating it per request.

- **`collection_worker`** — A background worker that can fetch and process fics from a collection or author page. When someone submits a collection URL (like an AO3 collection or an author's entire works page), the worker processes it in the background, fetching metadata and generating exports for each fic in the collection.

All of this gets created in the `run()` function, wrapped in `Arc::new()`, and passed to the router. The `Arc` (Atomic Reference Counted) pointer allows multiple handlers to share ownership of the same state without violating Rust's ownership rules. Since `Arc` uses atomic operations for reference counting, it's safe to share across async tasks.

## The run() Function: Bringing It to Life

The `run()` function is the entry point for the server. Here's what it does, step by step:

```rust
pub async fn run(config: Config) {
    // Connect to PostgreSQL
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // Connect to Redis
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");

    // Build HTTP client
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");

    // Initialize scraper registry
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());

    // Initialize rate limiter
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::new(
        redis_conn.clone(),
        config.dynamic_rate_limit,
    )
    .await
    .expect("Failed to initialize rate limiter");

    // Load datacenter IPs if configured
    if !config.ip_tag_sources.is_empty() {
        rate_limiter.load_datacenter_ips(&config.ip_tag_sources).await;
    }

    // Create shared state
    let recommender_engine = RecommendationEngine::new(db_pool.clone());
    let collection_worker = CollectionWorker::new(
        db_pool.clone(),
        redis_conn.clone(),
        http_client.clone(),
        config.clone(),
        scraper_registry.clone(),
    );
    let state = Arc::new(AppState {
        config: config.clone(),
        db: db_pool,
        redis: redis_conn,
        http_client,
        scraper_registry,
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(
            std::collections::HashMap::new()
        )),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine,
        collection_worker,
    });

    // Build router
    let app = build_router(state).await;

    // Bind and serve
    let addr = format!("0.0.0.0:{}", config.app_port);
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
        .await
        .expect("Server error");
}
```

First, it connects to PostgreSQL using SQLx's `init_pool`, which runs any pending migrations and creates a connection pool. The `init_pool` function is smart — it checks the database schema version and applies any SQL migration files that haven't been run yet. This means you can just point FicHub at an empty database and it'll set itself up.

Then it connects to Redis, builds the HTTP client, initializes the scraper registry and rate limiter, loads datacenter IP lists if configured, creates the recommendation engine and collection worker, and wraps everything in `AppState`.

Notice the use of `.clone()` throughout. When building the state, we need to clone the database pool, Redis connection, HTTP client, and scraper registry because they're all shared. Cloning a connection pool doesn't create a new pool — it just creates another handle to the same pool. This is the beauty of Rust's `Arc` and reference-counted types.

The rate limiter initialization is particularly interesting. It creates a new `RedisBucketLimiter` with the Redis connection and the `dynamic_rate_limit` config flag. If dynamic rate limiting is enabled, the limiter adjusts its thresholds based on server load. The `load_datacenter_ips` call fetches IP ranges from external sources (like Cloudflare's published ranges) so the limiter can apply different limits to datacenter IPs versus residential users.

Then it builds the router and binds to a TCP listener:

```rust
let addr = format!("0.0.0.0:{}", config.app_port);
let listener = tokio::net::TcpListener::bind(&addr)
    .await
    .expect("Failed to bind to address");

axum::serve(
    listener,
    app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
)
.await
.expect("Server error");
```

Notice the `into_make_service_with_connect_info::<SocketAddr>()` call. This tells Axum to extract the client's IP address and make it available through the `ConnectInfo` extractor. That's how the `/api/v0/remote` endpoint knows your IP address. Without this call, the `ConnectInfo` extractor would panic at runtime.

The server binds to `0.0.0.0:{port}`, which means it listens on all network interfaces. By default, the port is 3000 (set via the `PORT` environment variable). Binding to `0.0.0.0` instead of `127.0.0.1` means the server is accessible from other machines on the network, not just localhost. This is necessary for Docker (where the container needs to be reachable from outside) and for bare-metal deployment (where other machines need to reach the server).

If the port is already in use, `TcpListener::bind` will fail with an error. The `.expect("Failed to bind to address")` call will panic with a clear message. In production, you'd want to handle this more gracefully — perhaps logging the error and suggesting a different port.

## The Router Tree: Every Route Explained

Now let's look at the actual router. The `build_router` function constructs it by chaining route definitions. Here's the complete tree, explained route by route.

### API Documentation

```rust
.route("/api/", get(routes::api_docs::api_docs_handler))
```

The root `/api/` endpoint returns a JSON document describing all available API endpoints. It's like a built-in Swagger page, except it's just hand-written JSON. If you curl `http://localhost:3000/api/`, you'll get back a list of endpoints with their methods, parameters, and descriptions:

```json
{
    "name": "fichub-rs API",
    "version": "0.1.0",
    "endpoints": {
        "/api/v0/epub": {
            "method": "GET",
            "params": { "q": "URL of the fanfiction" },
            "description": "Fetch metadata and download links"
        },
        "/api/v0/meta": {
            "method": "GET",
            "params": { "q": "URL of the fanfiction" },
            "description": "Fetch metadata only"
        },
        "/api/v0/remote": {
            "method": "GET",
            "description": "Get request source information"
        },
        "/cache/:etype/:url_id": {
            "method": "GET",
            "params": { "h": "MD5 hash for validation" },
            "description": "Download cached export file"
        }
    }
}
```

This is handy for debugging and for anyone who wants to integrate with the API. It doesn't cover every endpoint (the OPDS, recommendation, and tag routes aren't listed), but it covers the core ones that most people use.

### Social and Curator Routes

```rust
.route("/api/bookmarks", post(routes::social::add_bookmark).get(routes::social::list_bookmarks))
.route("/api/bookmarks/{work_id}", delete(routes::social::remove_bookmark))
.route("/api/ratings", post(routes::social::rate_work))
.route("/api/ratings/{work_id}", get(routes::social::get_ratings))
.route("/api/comments", post(routes::social::add_comment).get(routes::social::list_comments))
.route("/api/comments/{work_id}", get(routes::social::list_comments))
.route("/api/works/{id}/comments", post(routes::comments::post_comment).get(routes::comments::get_threaded))
.route("/api/work-proposals", post(routes::work_proposals::create_proposal).get(routes::work_proposals::list_proposals))
.route("/api/work-proposals/{id}", get(routes::work_proposals::get_proposal))
.route("/api/work-proposals/{id}/vote", post(routes::work_proposals::vote_proposal))
```

These routes handle bookmarks, ratings, comments, and curator proposals. The social routes use `work_id` (integer) instead of the old `url_id` (string). The proposal routes let curators propose merging or splitting works, and vote on others' proposals.

### Core Export Endpoint

```rust
.route("/api/v0/epub", get(routes::export::epub_handler))
```

This is the main workhorse endpoint. It takes a query parameter `q` containing a fanfiction URL, looks up metadata, checks blacklists, generates EPUB and HTML files (or serves them from cache), and returns JSON with metadata, download URLs, and hashes.

The handler signature is:

```rust
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError>
```

The `State(state)` extractor pulls the shared `AppState` from the router. The `Query(params)` extractor deserializes the query string into the `ExportQuery` struct. And the return type `Result<Json<Value>, AppError>` means the handler either returns a JSON response or an error.

The flow inside `epub_handler` is:

1. **Parse the `q` parameter** — If it's empty, return an error with code `-1` and the message "no query".

2. **Block automated requests** — If the `automated=true` parameter is present, return an error with code `-10` and the message "automated requests blocked". This prevents bots from hammering the service.

3. **Find the right scraper** — Call `state.scraper_registry.find_scraper(query)` to match the URL to a scraper (AO3, FFN, etc.). If no scraper matches, return error `-5` with "unsupported URL".

4. **Look up metadata** — Call `scraper.lookup(&state.http_client, query)` to fetch the fic's title, author, word count, chapter count, description, and other metadata from the upstream site.

5. **Upsert fic info** — Call `queries::upsert_fic_info` to store or update the fic's metadata in PostgreSQL. This uses an "upsert" (INSERT ... ON CONFLICT DO UPDATE) so it works whether the fic is new or already exists.

5.5. **Auto-merge to canonical work** — Call `works::find_or_create_work` to link the source to a canonical work. If a work with the same title and author exists (within 5% word count tolerance), the source is linked to it. Otherwise, a new work is created. This is how the same story from different sites gets unified.

6. **Auto-populate tags** — Call `scraper.extract_tags` to pull genre/rating/relationship tags from the upstream site, then resolve and store them in the tagging system.

7. **Check blacklists** — Query the database for fic and author blacklists. If the fic is greylisted (reason 6), return metadata but no download links. If it's hard-blacklisted (reason 5, 7, or 8), return error `-7` with "fic is blacklisted".

8. **Check cache** — Look up the export log in the database. If a cached EPUB exists for the current version and content hash, build download URLs from the cached hash and return them immediately.

9. **If cache miss** — Acquire the export semaphore (preventing concurrent duplicate exports), recheck the cache (double-check pattern — another concurrent request might have generated it while we were waiting), fetch chapters, generate the EPUB, move it to the cache directory, record it in the export log, generate the HTML bundle, record that too, and return the URLs.

10. **Log the request** — Insert a record into `request_log` with timing information, the fic metadata, and the export hash.

The handler returns JSON shaped like:

```json
{
    "err": 0,
    "q": "https://archiveofourown.org/works/12345",
    "fixits": [],
    "info": "The Story by Author\n50000 words in 10 chapters\nStatus: complete\nUpdated: 2024-01-15 12:30:00 - 30 days ago\n",
    "work_id": 42,
    "url_id": "abc123",
    "slug": "The_Story-abc123",
    "meta": {
        "id": "abc123",
        "title": "The Story",
        "author": "Author",
        "chapters": 10,
        "words": 50000,
        "description": "<p>A great story</p>",
        "status": "complete",
        "source": "https://archiveofourown.org/works/12345",
        "created": "2023-12-01T00:00:00Z",
        "updated": "2024-01-15T12:30:00Z"
    },
    "hashes": {
        "epub": "a1b2c3d4e5f6",
        "html": "f6e5d4c3b2a1"
    },
    "urls": {
        "epub": "/cache/epub/abc123?h=a1b2c3d4e5f6",
        "html": "/cache/html/abc123?h=f6e5d4c3b2a1"
    },
    "epub_url": "/cache/epub/abc123?h=a1b2c3d4e5f6",
    "html_url": "/cache/html/abc123?h=f6e5d4c3b2a1",
    "mobi_url": null,
    "pdf_url": null,
    "notes": []
}
```

The error code (`err`) tells the caller what happened: `0` means success, negative numbers mean various errors. The `fixits` array is reserved for future use (potential corrections to the fic). The `info` string is a human-readable summary. The `slug` is a URL-safe version of the title combined with the URL ID.

### Metadata-Only Endpoint

```rust
.route("/api/v0/meta", get(routes::meta::meta_handler))
```

This endpoint is like the epub handler but lighter. It takes the same `q` parameter, finds the scraper, looks up metadata, and returns it without generating any files. This is useful when you just want to display fic information without waiting for an export to complete — for example, when the user is browsing and wants to see a preview before deciding to download.

The handler does the same scraper lookup, blacklist check, and slug generation, but skips all the caching and export logic. The response has the same shape as the epub endpoint, but `hashes`, `urls`, and `epub_url` are all empty/null:

```json
{
    "err": 0,
    "q": "https://archiveofourown.org/works/12345",
    "info": "The Story by Author - 50000 words, 10 chapters",
    "url_id": "abc123",
    "slug": "The_Story-abc123",
    "meta": { ... },
    "hashes": {},
    "urls": {},
    "epub_url": null,
    "html_url": null,
    "mobi_url": null,
    "pdf_url": null,
    "notes": []
}
```

The `info` format is slightly different here — it's a single-line summary instead of the multi-line format used in the epub handler.

### Remote Info

```rust
.route("/api/v0/remote", get(remote_handler))
```

A simple diagnostic endpoint that returns your IP address and port:

```rust
async fn remote_handler(
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> Json<serde_json::Value> {
    Json(json!({
        "ip": remote_addr.ip().to_string(),
        "port": remote_addr.port(),
        "is_automated": false,
    }))
}
```

This uses the `ConnectInfo` extractor, which only works because the server was set up with `into_make_service_with_connect_info::<std::net::SocketAddr>()`. If you forget that call in `run()`, this handler will panic.

The `is_automated` field is always `false` in the response — it's a field the client can set in requests to indicate automated access, and this endpoint echoes back the default. It's useful for debugging proxy configurations, verifying that the client IP is being correctly extracted, and testing whether Docker networking is set up properly.

### Cache Download Routes

```rust
.route("/cache/{etype}/{url_id}/{fname}", get(routes::cache_download::download_with_hash))
.route("/cache/{etype}/{url_id}", get(routes::cache_download::download_or_export))
```

These routes serve cached export files directly. There are two variants, and understanding the difference is important.

**`/cache/{etype}/{url_id}/{fname}`** — The direct download route. The `fname` parameter includes the hash embedded in the filename (like `a1b2c3d4e5f6.epub`). The handler extracts the hash, validates it against the file's actual MD5 hash, and serves the file with the correct MIME type and Content-Disposition header.

The validation step is crucial. Without it, someone could request `/cache/epub/abc123/anything.epub` and potentially read arbitrary files. By requiring a valid MD5 hash, the server ensures that only correctly-hashed filenames can access cached files.

The MIME types are determined by the `etype` parameter:

| Format | Suffix | MIME Type |
|--------|--------|-----------|
| epub | `.epub` | `application/epub+zip` |
| html | `.zip` | `application/zip` |
| mobi | `.mobi` | `application/x-mobipocket-ebook` |
| pdf | `.pdf` | `application/pdf` |

**`/cache/{etype}/{url_id}`** — The smart download route. This is what the API returns in its `urls` field. If a hash is provided via query parameter (`?h=a1b2c3d4e5f6`) and the cached file exists and validates, it's served directly. Otherwise, the handler redirects to `/?id={url_id}`, which opens the SvelteKit frontend with that fic pre-loaded — the frontend then triggers the export via the API.

This redirect behavior is smart: if the cache has been evicted or the server was restarted, the user doesn't get a confusing error. Instead, they're gently redirected to the web UI, which will re-export the fic on demand.

### Recommendation Routes

```rust
.route("/api/v0/recommendations",
    get(crate::recommender::routes::recommendations_handler))
.route("/api/v0/recommendations/suggest",
    post(crate::recommender::routes::suggest_handler))
.route("/api/v0/recommendations/vote",
    post(crate::recommender::routes::vote_handler))
.route("/api/v0/recommendations/votes",
    get(crate::recommender::routes::votes_handler))
```

These four routes power the recommendation engine:

- **GET `/api/v0/recommendations`** — Fetch recommendations for a given fic or author. The engine uses collaboratively-filtered favourites to find similar stories. Pass a `url_id` or `author` parameter to get recommendations. The response includes a list of recommended fics with similarity scores.

- **POST `/api/v0/recommendations/suggest`** — Submit a new recommendation (suggest that fic A is similar to fic B). This is an authenticated endpoint that requires a user identifier. Suggestions feed into the collaborative filtering algorithm — if enough users suggest that two fics are similar, the system learns to recommend them together.

- **POST `/api/v0/recommendations/vote`** — Vote on an existing recommendation (agree or disagree). Voting strengthens or weakens the recommendation connection between two fics. The voting system uses a gamma parameter (`REC_VOTING_BOOST_GAMMA`) to control how much votes affect recommendation scores.

- **GET `/api/v0/recommendations/votes`** — Fetch vote counts for recommendations on a given fic. This lets the UI display how many people agree or disagree with each recommendation.

### Tag Routes

```rust
.route("/api/v0/tags/submit", post(crate::tags::routes::submit_tag))
.route("/api/v0/tags/vote", post(crate::tags::routes::vote_tag))
.route("/api/v0/tags/flag", post(crate::tags::routes::flag_tag))
.route("/api/v0/tags", get(crate::tags::routes::get_tags))
```

The tagging system (v3) is a community-driven categorization system:

- **POST `/api/v0/tags/submit`** — Submit a new tag for a fic. Tags go through a resolution process that handles duplicates and synonyms — if someone submits "Romance" and another person submits "romance", they're merged.

- **POST `/api/v0/tags/vote`** — Vote on an existing tag (upvote or downvote). Tags with votes below `TAG_HIDDEN_THRESHOLD` (default -3) are hidden from the public view. This is the community's way of curating tag quality.

- **POST `/api/v0/tags/flag`** — Flag a tag for moderator review. If someone sees a tag that's inappropriate, misleading, or spam, they can flag it for the curators to handle.

- **GET `/api/v0/tags`** — Fetch existing tags for a fic. Returns all tags with their vote counts and types.

All tag endpoints are rate-limited: `TAG_SUBMIT_LIMIT_PER_HOUR` (default 10) for submissions and `TAG_VOTE_LIMIT_PER_HOUR` (default 20) for votes.

### Curator Routes

```rust
.route("/api/v0/curator/alias", post(crate::tags::curator::create_alias))
.route("/api/v0/curator/merge", post(crate::tags::curator::merge_tags))
.route("/api/v0/curator/tags/{id}", delete(crate::tags::curator::delete_tag))
.route("/api/v0/curator/flags", get(crate::tags::curator::list_flags))
.route("/api/v0/curator/flags/{id}/resolve", post(crate::tags::curator::resolve_flag))
```

These routes are behind a curator token (set via `CURATOR_TOKEN` environment variable). They provide moderation tools:

- **POST `/api/v0/curator/alias`** — Create a tag alias. For example, making "fluff" an alias of "Humor/Fluff" ensures that both terms map to the same tag.

- **POST `/api/v0/curator/merge`** — Merge two tags into one. All fics tagged with the source tag get re-tagged with the destination tag, and the source tag is deleted.

- **DELETE `/api/v0/curator/tags/{id}`** — Delete a tag entirely. This removes it from all fics.

- **GET `/api/v0/curator/flags`** — List all flagged tags that need moderator attention. This is the curator's dashboard.

- **POST `/api/v0/curator/flags/{id}/resolve`** — Resolve a flag (mark it as handled). The curator can choose to dismiss the flag (tag is fine) or act on it (delete the tag, merge it, etc.).

The curator token is checked in the handler code, not in middleware. This means if someone sends a request without the token, they get a 403 Forbidden response, not a 401 Unauthorized (which would suggest they need to authenticate differently).

### Search Route

```rust
.route("/api/v0/search", get(crate::search::routes::search_handler))
```

The advanced search endpoint lets users search across the fic database by title, author, tags, word count, status, and other criteria. It returns paginated results and uses the `SEARCH_MAX_PER_PAGE` config option to limit results (default 50).

The search is powered by SQLx queries against PostgreSQL. For simple title searches, it uses `ILIKE` (case-insensitive LIKE). For more complex queries involving multiple criteria, it builds dynamic SQL with appropriate parameter binding to prevent SQL injection.

### OPDS Catalog Routes

```rust
.route("/opds", get(crate::routes::opds::feeds::root_catalog))
.route("/opds/new", get(crate::routes::opds::feeds::recent_feed))
.route("/opds/popular", get(crate::routes::opds::feeds::popular_feed))
.route("/opds/tags", get(crate::routes::opds::tags::tag_types))
.route("/opds/tags/{type_id}", get(crate::routes::opds::tags::tags_by_type))
.route("/opds/tags/{type_id}/{tag_name}", get(crate::routes::opds::tags::fics_by_tag))
.route("/opds/authors", get(crate::routes::opds::authors::author_list))
.route("/opds/recommendations/popular",
    get(crate::routes::opds::recommendations::popular_recommendations))
.route("/opds/recommendations",
    get(crate::routes::opds::recommendations::fic_recommendations))
.route("/opds/search", get(crate::routes::opds::search::search_feed))
.route("/opds/shelves", get(crate::routes::opds::shelves::shelf_list))
.route("/opds/shelf/{shelf_id}", get(crate::routes::opds::shelves::shelf_contents))
```

OPDS (Open Publication Distribution System) is a standard that e-readers like KOReader, Calibre, Thorium Reader, and Moon+ Reader use to browse and download books. FicHub implements a full OPDS catalog, so you can point your e-reader at `http://your-server:3000/opds` and browse fics right from your device — no app installation needed.

The routes cover every aspect of a book catalog:

- **Root catalog** — The top-level navigation feed. When you first point your e-reader at FicHub, this is what it loads. It contains links to all the sub-feeds.

- **New fics** (`/opds/new`) — Recently added stories, sorted by when they were last updated. Great for finding new things to read.

- **Popular fics** (`/opds/popular`) — Stories ranked by popularity. The popularity score considers word count, chapter count, and how many times the fic has been downloaded.

- **Tags** — A three-level hierarchy. `/opds/tags` lists tag types (genre, rating, relationship, etc.). `/opds/tags/{type_id}` lists tags of that type. `/opds/tags/{type_id}/{tag_name}` lists fics with that specific tag.

- **Authors** (`/opds/authors`) — Browse by author. The feed lists all authors who have fics in the system, with links to each author's fics.

- **Recommendations** — Both popular recommendations (`/opds/recommendations/popular`) and per-fic recommendations (`/opds/recommendations?url_id=abc123`). This brings the recommendation engine to your e-reader.

- **Search** (`/opds/search?q=keyword`) — Full-text search via the OPDS protocol. Your e-reader's built-in search will use this endpoint.

- **Shelves** — User-created reading lists, authenticated with `OPDS_SHELF_TOKEN`. `/opds/shelves` lists all shelves, and `/opds/shelf/{shelf_id}` shows the fics on a specific shelf.

This is one of FicHub's killer features — it turns your fanfiction library into a personal OPDS server. You can browse, search, and download fics from any OPDS-compatible reader.

### Legacy Redirect Routes

```rust
.route("/legacy/epub_export", get(redirect_to_root))
.route("/fic/{url_id}", get(redirect_to_root))
.route("/changes", get(redirect_to_root))
.route("/popular/", get(redirect_to_root))
```

These four routes handle old URLs from the Python-based fichub.net. If someone has a bookmark from the old site — `https://fichub.net/fic/abc123` or `https://fichub.net/changes` — they'll get redirected to the root page where the SvelteKit frontend can handle it.

The redirect handler is defined inside `build_router` as a simple async function:

```rust
async fn redirect_to_root() -> Redirect {
    Redirect::to("/")
}
```

It's defined inside the function rather than as a closure because Axum's routing works better with named functions. Closures with async blocks can cause lifetime issues in the router construction.

This is a nice touch for backward compatibility. Old links don't break — they just land on the modern UI, where the frontend can handle the URL.

### The Fallback: Serving the Frontend

After all the API routes, there's a crucial piece:

```rust
.fallback_service(
    ServeDir::new(&frontend_dir)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(frontend_dir.join("index.html"))),
)
```

This is how FicHub serves its SvelteKit frontend. The `fallback_service` is called last, so it only handles requests that didn't match any of the explicit routes above. Let's break it down:

1. **`ServeDir::new(&frontend_dir)`** — Serve static files from the frontend build directory (default: `./frontend/build`). When someone requests `/style.css`, it serves `./frontend/build/style.css`. When someone requests `/app.js`, it serves `./frontend/build/app.js`. The directory path comes from `state.config.frontend_dir`.

2. **`append_index_html_on_directories(true)`** — If someone requests `/` or `/about/`, serve the `index.html` from that directory. This handles the basic SPA routing case where the user navigates to a directory-like path.

3. **`fallback(ServeFile::new(frontend_dir.join("index.html")))`** — If `ServeDir` can't find a matching file, serve `index.html` anyway. This is the SPA (Single Page Application) catch-all. The SvelteKit frontend uses client-side routing, so routes like `/fic/abc123` or `/settings` need to serve `index.html` and let the JavaScript router take over.

### Middleware: Logging and CORS

At the bottom of the router construction:

```rust
.layer(TraceLayer::new_for_http())
.layer(CorsLayer::permissive())
```

Two middleware layers wrap the entire application:

- **`TraceLayer`** — Logs every incoming request and outgoing response with timing information. This is essential for debugging production issues. The trace includes the HTTP method, path, status code, and response time.

- **`CorsLayer::permissive()`** — Allows all CORS requests. This is important for development (where the frontend might be running on port 5173 and the API on port 3000) and for any API consumers. In a production environment, you'd typically tighten this down to only allow requests from your specific domain.

The middleware is applied in reverse order — `CorsLayer` runs first (closest to the application), then `TraceLayer` (closest to the network). So a request flows through: TraceLayer → CorsLayer → Route matching → Handler.

### Injecting State

The final line is:

```rust
.with_state(state)
```

This injects the shared `AppState` into the router. Without this line, every handler that tries to extract `State<Arc<AppState>>` will fail at runtime with a confusing error message. The `with_state` call is what makes the state available to all handlers through Axum's extractor system.

## Connecting to the System: A Data Flow Diagram

To really understand how the router connects to the rest of FicHub, let's trace data as it flows through the system:

```
User's Browser / E-Reader
         │
         ▼
    TCP Connection (tokio::net::TcpListener)
         │
         ▼
    Axum Server (axum::serve)
         │
         ├──▶ TraceLayer (logs request)
         ├──▶ CorsLayer (handles CORS)
         │
         ▼
    Router Tree (build_router)
         │
         ├──▶ /api/v0/epub ──▶ epub_handler ──▶ ScraperRegistry ──▶ AO3/FFN/etc.
         │                              │──▶ Database (upsert fic_info)
         │                              │──▶ Cache (check/generate EPUB)
         │                              │──▶ Response (JSON with URLs)
         │
         ├──▶ /api/v0/meta ──▶ meta_handler ──▶ ScraperRegistry ──▶ Database
         │
         ├──▶ /cache/{etype}/{url_id} ──▶ cache_download ──▶ Disk (serve file)
         │
         ├──▶ /api/v0/recommendations ──▶ RecommendationEngine ──▶ Database + Redis
         │
         ├──▶ /api/v0/tags ──▶ Tag routes ──▶ Database
         │
         ├──▶ /api/v0/search ──▶ Search handler ──▶ Database
         │
         ├──▶ /opds/* ──▶ OPDS feeds ──▶ Database ──▶ XML response
         │
         └──▶ fallback_service ──▶ ServeDir (static files) ──▶ SvelteKit frontend
```

This diagram shows the complete picture. Every request enters through the same door (the TCP listener), passes through the same middleware (logging and CORS), and gets routed to the appropriate handler. Each handler reaches into the shared `AppState` to access databases, caches, scrapers, and other resources.

The beauty of this architecture is that each piece is independent and testable. You can test the scraper registry without the router. You can test the router without the database (using mock state). You can test the cache download handler without the scraper. The router is the integration point, but the individual pieces are loosely coupled.

## Connecting the Dots

The `run()` function is called from `main()`:

```rust
#[tokio::main]
async fn main() {
    // Load .env if present
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(
                    "info,fichub=debug"
                )),
        )
        .init();

    // Load configuration
    let config = config::Config::from_env();

    tracing::info!("Starting fichub-rs server on port {}", config.app_port);

    // Run the server
    server::run(config).await;
}
```

It loads environment variables from `.env` (if present), sets up structured logging with `tracing_subscriber`, loads the configuration, and runs the server. The `RUST_LOG` environment variable controls log verbosity — `info,fichub=debug` is a good default, giving you info-level logs for everything except FicHub's own code, which gets debug-level logging.

The `.ok()` on `dotenvy::dotenv()` means if there's no `.env` file, it silently continues. This is important for Docker deployments where environment variables are passed through Docker Compose instead of a file.

## Request Flow: End to End

Let's trace a complete request to understand how everything connects:

1. User sends `GET /api/v0/epub?q=https://archiveofourown.org/works/12345`
2. Tokio receives the TCP connection
3. Axum parses the HTTP request
4. `TraceLayer` logs the incoming request
5. `CorsLayer` checks for CORS headers
6. The router matches `/api/v0/epub` → `epub_handler`
7. `State(state)` extractor pulls the shared `AppState`
8. `Query(params)` extractor parses `q=https://...`
9. The handler runs through its 10-step flow
10. Returns `Json(value)`
11. `CorsLayer` adds CORS headers to the response
12. `TraceLayer` logs the response with timing
13. Tokio sends the HTTP response back

The whole thing takes milliseconds for cached requests and a few seconds for fresh exports. The async runtime handles thousands of concurrent requests without breaking a sweat.

## 🧪 Try It Yourself

1. Start FicHub locally with `cargo run`
2. Open a browser and navigate to `http://localhost:3000/api/`
3. You should see the API documentation JSON
4. Try `http://localhost:3000/api/v0/remote` — it shows your IP
5. Try `http://localhost:3000/` — you should see the SvelteKit frontend (if the build is in place)
6. Run `curl -v http://localhost:3000/api/v0/epub` to see the empty query error
7. Check the terminal output — you'll see TraceLayer logging each request

## ⚠️ Watch Out

The router construction order matters. In Axum, more specific routes take precedence over less specific ones. The `/api/` routes are registered first, so they always win. If you added `.fallback_service()` before your API routes, the static file server would swallow everything. Always put your fallback last.

Also note that the `with_state(state)` call at the very end injects the shared state into all handlers. If you forget this line, every handler that tries to extract `State<Arc<AppState>>` will fail at runtime with a cryptic error. The error message usually mentions something about "missing state" but it's not always clear what went wrong.

One more subtlety: the `remote_handler` function uses `ConnectInfo<SocketAddr>`, which requires the server to be configured with `into_make_service_with_connect_info`. If you change how the server is started (for example, using a different serve method), this handler will break silently — it'll compile fine but panic at runtime.

---

# Chapter 26: Serving Static Files

We've alluded to it a few times, but now let's really dig into how FicHub serves its SvelteKit frontend. The frontend is a separate application — a SvelteKit SPA that builds into static HTML, CSS, and JavaScript files. FicHub's Rust server doesn't just run an API — it also serves those static files, acting as both the backend API server and the frontend web server.

This is actually a pretty elegant setup. You get a single server, a single port, and a single Docker container that handles everything.

## The Frontend Module

Let's look at `src/frontend/mod.rs`:

```rust
/// Frontend module - serves the SvelteKit built SPA
/// The SvelteKit build output is served as static files by
/// tower-http ServeDir in server.rs

/// Static file serving is handled in server.rs via:
/// `.fallback_service(ServeDir::new(&frontend_dir)
///     .append_index_html_on_directories(true))`
///
/// This module only provides constants and helpers.

/// Default location for the SvelteKit build output
pub const DEFAULT_FRONTEND_DIR: &str = "./frontend/build";
```

This module is intentionally minimal. It defines a single constant — the default path to the SvelteKit build output — and that's it. All the actual file serving happens in `server.rs` through `tower-http`'s `ServeDir` and `ServeFile`.

The `FRONTEND_DIR` environment variable lets you override this path. In Docker, for example, it's set to `/app/frontend`, while locally it defaults to `./frontend/build`. This flexibility means the same binary works in development, Docker, and bare-metal deployments without any code changes.

Why is the module so small? Because `tower-http` already does everything we need. There's no point writing custom file-serving logic when a well-tested, battle-hardened library handles it. The module exists as a central place for the constant and as documentation about how the frontend is served.

## ServeDir: The Static File Engine

The magic happens in the `fallback_service` call:

```rust
ServeDir::new(&frontend_dir)
    .append_index_html_on_directories(true)
    .fallback(ServeFile::new(frontend_dir.join("index.html")))
```

Let's unpack each piece.

### ServeDir

`ServeDir` is a middleware from `tower-http` that serves files from a directory. When a request comes in, it maps the URL path to a file path within the directory and serves it. It handles:

- **MIME type detection** — Based on file extension. `.css` → `text/css`, `.js` → `application/javascript`, `.json` → `application/json`, `.svg` → `image/svg+xml`, etc. It uses the `mime_guess` crate under the hood for reliable detection.

- **404 responses** — If the file doesn't exist, it returns a 404 status code. Without the fallback, this is what happens for all SPA routes.

- **Efficient streaming** — Files are streamed to the client using `tokio::fs::File`, which means the entire file doesn't need to be loaded into memory. For large JavaScript bundles (which can be megabytes), this matters.

- **Conditional requests** — `ServeDir` supports `If-Modified-Since` and `If-None-Match` headers. If the client sends one of these and the file hasn't changed, the server returns 304 Not Modified instead of re-sending the entire file.

- **Content-Length headers** — The file size is determined upfront and sent in the response headers, so the client can show a progress bar during download.

The directory is determined by `state.config.frontend_dir`, which defaults to `./frontend/build`. That's where the SvelteKit build puts its output.

### append_index_html_on_directories

When you set `append_index_html_on_directories(true)`, requesting a directory (like `/` or `/about/`) will serve the `index.html` from that directory. This is important for SvelteKit's routing — when you navigate to `/`, the server needs to serve `frontend/build/index.html`.

Without this option, requesting `/` would return a 404 or a directory listing, neither of which is useful for an SPA. With it, `/` maps to `./frontend/build/index.html`, which is exactly what we want.

This option is particularly important for the root path `/`. When someone visits `http://your-server.com/`, the server receives a request for `/`. Without `append_index_html_on_directories`, ServeDir would look for a file named `` (empty string) and fail. With it, ServeDir recognizes `/` as a directory and serves `index.html`.

### The Fallback: SPA Catch-All

The most important part is the `.fallback(ServeFile::new(frontend_dir.join("index.html")))` call. This is the SPA catch-all.

Consider this scenario: a user navigates directly to `http://your-server.com/fic/abc123`. The server receives a request for `/fic/abc123`. `ServeDir` looks for `frontend/build/fic/abc123` — that file doesn't exist. Without the fallback, the user would get a 404.

With the fallback, `ServeDir` gives up and passes the request to `ServeFile`, which serves `index.html` regardless of the path. The SvelteKit JavaScript then boots up, sees the URL path `/fic/abc123`, and renders the appropriate page. This is the standard SPA routing pattern.

The fallback chain works like this:

1. Request for `/fic/abc123` arrives
2. `ServeDir` tries `./frontend/build/fic/abc123` — not found
3. `ServeDir` tries `./frontend/build/fic/abc123/index.html` — not found
4. `ServeDir` gives up, invokes the fallback
5. `ServeFile` serves `./frontend/build/index.html`
6. SvelteKit's JavaScript boots, reads the URL, renders the fic page

This is invisible to the user — they just see the page they asked for. And it works for every possible path: `/settings`, `/search?q=hello`, `/recommendations/abc123`, anything. They all serve `index.html`, and the JavaScript handles the rest.

## Build Directory Structure

When you build the SvelteKit frontend (`npm run build` or `vite build`), it produces a `build/` directory with a structure like this:

```
frontend/build/
├── _app/
│   ├── immutable/
│   │   ├── assets/
│   │   │   ├── index-Bx3d5j1L.css      (hashed CSS bundle)
│   │   │   └── index-C7H5dK3x.js       (hashed JS bundle)
│   │   └── chunks/
│   │       ├── index-Bk2eP1aR.js       (code-split chunk)
│   │       ├── error-CjH5fG2x.js       (error page chunk)
│   │       └── _layout-Dm4eN8wQ.js     (layout chunk)
│   └── version.json                     (build version info)
├── favicon.png
├── index.html                           (the entry point)
└── manifest.json                        (PWA manifest)
```

Key things to notice:

- **`index.html`** — The entry point. This is what the SPA fallback serves for any unmatched route. It contains `<script>` tags that load the JavaScript bundles, and `<link>` tags that load the CSS.

- **`_app/immutable/assets/`** — Hashed asset filenames. These are the "immutable" assets — once built, they never change. The hash in the filename (like `index-Bx3d5j1L.css`) ensures that if the content changes, the filename changes too. This is the key to cache busting — browsers can safely cache these files forever because the filename will change when the content changes.

- **`_app/immutable/chunks/`** — Code-split chunks that are loaded on demand. SvelteKit automatically splits code so that only the JavaScript needed for the current page is loaded initially. If you navigate to a different page, the corresponding chunk is fetched on demand.

- **`version.json`** — Contains the build timestamp and a hash of the build. The SvelteKit service worker uses this to detect when a new version is available and prompt the user to refresh.

The entire build directory is typically 500KB to 2MB, depending on how many components and pages the frontend has.

## Cache Headers

One thing that SvelteKit handles beautifully is cache headers. The immutable assets in `_app/immutable/` are meant to be cached forever. Their filenames contain content hashes, so when the code changes, the filename changes too. Browsers can safely cache `index-Bx3d5j1L.css` forever because if the CSS changes, it'll be `index-Kq9mN2wQ.css` next time.

The `index.html` file, on the other hand, should never be cached (or cached very briefly). It's the entry point, and it references the hashed asset filenames. If `index.html` is cached, users might load stale JavaScript that references assets that no longer exist.

`tower-http`'s `ServeDir` doesn't set cache headers by default. For FicHub's production deployment, you'd typically put an nginx reverse proxy in front that sets appropriate cache headers:

```nginx
# Immutable assets — cache for a year
location /_app/immutable/ {
    add_header Cache-Control "public, max-age=31536000, immutable";
}

# Everything else — don't cache
location / {
    add_header Cache-Control "no-cache, must-revalidate";
}
```

If you're not using nginx, you can add cache header middleware in the Axum router itself. You'd create a custom middleware layer that inspects the request path and adds appropriate `Cache-Control` headers to the response.

### Adding Cache Headers in Axum

For deployments without nginx, here's how you'd add cache headers directly in the Axum router:

```rust
use tower_http::set_header::SetResponseHeaderLayer;
use axum::http::header::CACHE_CONTROL;

// Before the fallback_service:
.layer(SetResponseHeaderLayer::overriding(
    CACHE_CONTROL,
    HeaderValue::from_static("no-cache"),
))
```

This sets a default `Cache-Control: no-cache` header on all responses. You'd then need a more targeted middleware to set longer cache times for the immutable assets. The `tower-http` ecosystem has the tools for this, but nginx is generally the easier choice.

## How the Rust Server Serves the SvelteKit Frontend

The full flow works like this:

**First visit (no cache):**

1. User visits `http://your-server.com/`
2. Request hits Axum
3. No API route matches (`/api/`, `/api/v0/...`, etc.)
4. `ServeDir` looks for `./frontend/build/` → finds `index.html` → serves it
5. Browser downloads `index.html`, which loads `index-C7H5dK3x.js`
6. Request for `index-C7H5dK3x.js` hits Axum
7. `ServeDir` finds the file in `./frontend/build/_app/immutable/assets/` → serves it
8. SvelteKit JavaScript boots up, takes over client-side routing
9. User sees the FicHub web interface

**Navigating within the app:**

10. User clicks a link to `/fic/abc123`
11. Client-side router intercepts the click (prevents a full page reload)
12. Router renders the fic page component with the `abc123` parameter
13. No request to the server (unless the component fetches data)

**Direct URL access (refresh on a sub-page):**

14. User refreshes the page on `/fic/abc123`
15. Browser requests `http://your-server.com/fic/abc123`
16. No API route matches
17. `ServeDir` can't find `./frontend/build/fic/abc123`
18. The fallback `ServeFile` kicks in → serves `index.html`
19. SvelteKit boots up, sees the URL, renders the fic page

**Loading a hashed asset:**

20. `index.html` contains `<link rel="stylesheet" href="/_app/immutable/assets/index-Bx3d5j1L.css">`
21. Browser requests `/_app/immutable/assets/index-Bx3d5j1L.css`
22. `ServeDir` maps this to `./frontend/build/_app/immutable/assets/index-Bx3d5j1L.css`
23. File exists → served with correct MIME type (`text/css`)
24. Browser caches it forever (because of the content hash in the filename)

This is the beauty of SPA routing with a Rust backend. The server is simple — it just serves files — and the client handles the routing intelligence.

## Production Considerations

In development, you'd typically run the SvelteKit dev server on one port (like 5173) and the Rust server on another (3000), with the SvelteKit dev server proxying API requests to the Rust server. This gives you hot module replacement and fast iteration. The Vite dev server is configured with a proxy:

```javascript
// vite.config.js
export default {
    server: {
        proxy: {
            '/api': 'http://localhost:3000',
            '/cache': 'http://localhost:3000',
            '/opds': 'http://localhost:3000',
        }
    }
};
```

For production, you build the SvelteKit app (`npm run build`) and copy the `build/` output into the path where FicHub expects it (default: `./frontend/build`). Then the Rust server serves everything.

### Building the Frontend

```bash
cd frontend
npm install
npm run build
# Output goes to frontend/build/
```

The build process:
1. Compiles all Svelte components
2. Tree-shakes unused code (removes anything not imported)
3. Code-splits into chunks for lazy loading
4. Hashes all asset filenames
5. Generates `index.html` with correct script/link tags
6. Copies static assets (favicon, manifest, etc.)

### Deploying the Build

If you're running FicHub directly (not in Docker), the build output needs to be in `./frontend/build` relative to where you run the FicHub binary:

```bash
# Build the frontend
cd frontend && npm run build && cd ..

# The build output is already in frontend/build/
# Just make sure FRONTEND_DIR points to the right place
# Default: ./frontend/build
export FRONTEND_DIR=./frontend/build

# Start the server
cargo run --release
```

In Docker, the Dockerfile copies the frontend build into the container (or the frontend is mounted as a volume). The `FRONTEND_DIR` environment variable in `docker-compose.yml` is set to `/app/frontend`.

### Without the Frontend

If you don't have the frontend build in place, the SPA fallback still works — `index.html` won't exist, so every non-API request will return a 404 from `ServeFile`. The API endpoints will still work perfectly, though. You can use FicHub purely as an API server without the web UI.

This is useful for headless deployments where another frontend (a mobile app, a command-line tool, or a different web framework) consumes the API directly. The OPDS catalog also works without the web UI — e-readers only need the OPDS endpoints.

### Performance

Static file serving is incredibly efficient. `tower-http`'s `ServeDir` uses `tokio::fs::File` for async I/O, so file reads don't block the runtime. On a modern SSD, FicHub can serve thousands of static file requests per second. The main bottleneck is network bandwidth, not the file serving itself.

If you're serving a lot of static files, you might want to add gzip compression. The `tower-http` crate includes `CompressionLayer` that can compress responses on the fly. For text-based assets (HTML, CSS, JS), gzip compression typically reduces file sizes by 60-80%, significantly improving load times.

## 🧪 Try It Yourself

1. Build the SvelteKit frontend: `cd frontend && npm install && npm run build`
2. Start the FicHub server: `cargo run`
3. Visit `http://localhost:3000/` — you should see the frontend
4. Open your browser's developer tools, go to the Network tab
5. Navigate to a non-existent page like `http://localhost:3000/nonexistent`
6. You'll see the request for `/nonexistent` return `index.html` (status 200, type document)
7. The SvelteKit router handles the 404 page client-side
8. Check the response headers — you should see `Content-Type: text/html` and the file size
9. Try clearing the browser cache and refreshing — watch the network requests to see the full loading sequence

## ⚠️ Watch Out

The most common deployment mistake is forgetting to build the frontend. If `./frontend/build/index.html` doesn't exist, the SPA fallback returns a 404, and users see a blank page or an error. The API still works, but the web UI is dead. Always verify that `index.html` exists before deploying.

Another gotcha: `ServeDir` serves files from the filesystem at request time. It doesn't cache them in memory. If you're serving thousands of files, the filesystem performance matters. On a fast SSD, this is fine. On a slow spinning disk, you might want to add a caching layer.

A third issue: symlinks. If your `frontend/build` directory contains symbolic links (which can happen if you use a build tool that creates them), `ServeDir` follows them by default. This is usually fine, but it can be a security concern if the symlinks point outside the expected directory.

And one more: the trailing slash. `ServeDir` handles paths with and without trailing slashes differently. `/style.css` works fine, but `/about` vs `/about/` might behave differently depending on the `append_index_html_on_directories` setting. SvelteKit generates links with trailing slashes for directory-like routes, so this usually works out of the box.

---

# Chapter 27: Docker and Docker Compose

If you've made it this far, you have a working FicHub server with PostgreSQL, Redis, scrapers, exporters, a recommendation engine, and a frontend. Running it all locally with `cargo run` works for development, but for production — and especially for deployment on a remote server — Docker is your best friend.

## What Is Docker? A Lunchbox for Your App

Imagine you've made a delicious sandwich. Now you want to take it to work. You put it in a lunchbox so it stays fresh and doesn't make a mess. Docker is the lunchbox for your application.

When you "containerize" an application with Docker, you pack up:
- The application code
- The compiled binary
- All system libraries it needs
- A minimal operating system to run it

The result is a self-contained unit (a "container") that runs the same way on your laptop, on a cloud server, on your friend's machine, or on a Raspberry Pi. No "works on my machine" problems.

Docker images are like recipes — they describe exactly how to build the container. Containers are like instances — they're the running version of the recipe.

Let's clarify some terminology:

- **Dockerfile** — A text file with instructions for building a Docker image. Think of it as a recipe.
- **Image** — The compiled result of a Dockerfile. A read-only snapshot of everything your app needs.
- **Container** — A running instance of an image. Multiple containers can run from the same image.
- **Volume** — A persistent storage area that survives container restarts and removals.
- **Docker Compose** — A tool for defining and running multi-container applications. You describe all your services in a YAML file and start them with one command.

## The Dockerfile: A Recipe for FicHub

FicHub's Dockerfile uses a multi-stage build. This is a clever trick that keeps the final image small. Here it is:

```dockerfile
# syntax=docker/dockerfile:1

# Stage 1: Build the Rust binary
FROM rust:slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

# Stage 2: Create the slim runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/fichub /app/fichub
COPY --from=builder /app/migrations /app/migrations
COPY .env.example /app/.env

EXPOSE 3000

CMD ["/app/fichub"]
```

### The Builder Stage

The first stage (`FROM rust:slim-bookworm AS builder`) is the build environment. It's a complete development environment with the Rust compiler and all necessary tools.

**Base image:** `rust:slim-bookworm` is a Debian Bookworm image with Rust pre-installed. The "slim" variant strips out documentation, man pages, and other non-essential files, keeping the image smaller than the full Rust image. At around 200-300 MB, it's still much larger than the final runtime image — but that's OK because we throw it away after building.

**Build dependencies:** The `apt-get install` line installs three packages needed at compile time:

- `pkg-config` — Helps Cargo find system libraries (like OpenSSL and libpq) during compilation
- `libssl-dev` — OpenSSL development headers (needed by crates that use TLS)
- `libpq-dev` — PostgreSQL client library development headers (needed by SQLx)

The `&& rm -rf /var/lib/apt/lists/*` line cleans up the apt cache to keep the layer smaller.

**Source code:** The `COPY` commands bring in the source files:

```dockerfile
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
```

Notice that `Cargo.toml` and `Cargo.lock` are copied first. This is a Docker best practice — Docker caches layers, and if `Cargo.toml` hasn't changed, Docker can skip the dependency resolution step. Only `src/` and `migrations/` trigger a full rebuild.

**Build command:** `cargo build --release` compiles the binary in release mode. Release mode enables optimizations (`-O2` or `-O3`), which makes the binary much faster but takes longer to compile. In a Docker build, this typically takes 5-10 minutes.

### The Runtime Stage

The second stage (`FROM debian:bookworm-slim`) is the actual runtime environment. It's much smaller:

**Base image:** `debian:bookworm-slim` is a minimal Debian image without Rust, without compilers, without build tools. Just the essentials — a basic shell, glibc, and a package manager. It's typically around 70-80 MB.

**Runtime dependencies:** Only two packages are needed:

- `ca-certificates` — Root certificates for HTTPS connections (needed by `reqwest` when scraping sites over HTTPS)
- `libpq-dev` — The PostgreSQL client library that the compiled binary links against at runtime

**Copy the binary:** The key line is:

```dockerfile
COPY --from=builder /app/target/release/fichub /app/fichub
```

This pulls just the compiled binary from the builder stage. No source code, no build artifacts, no compiler. Just the single executable file.

**Copy migrations:** The SQL migration files are copied too, since FicHub runs migrations on startup:

```dockerfile
COPY --from=builder /app/migrations /app/migrations
```

**Copy default config:** `COPY .env.example /app/.env` provides a default environment configuration. In production, you'd override this with Docker Compose environment variables.

**Expose and run:**

```dockerfile
EXPOSE 3000
CMD ["/app/fichub"]
```

`EXPOSE 3000` is documentation — it tells Docker users that the container listens on port 3000. It doesn't actually publish the port (that's done with `-p 3000:3000` or in Docker Compose). `CMD` specifies the default command to run when the container starts.

### Why Multi-Stage?

The final image only contains the runtime binary and its minimal dependencies. It does NOT contain:
- The Rust compiler (hundreds of MB)
- All the source code
- Build dependencies like `libssl-dev` and `pkg-config`
- Cargo cache, intermediate build artifacts
- Any of the `target/debug/` or `target/release/` intermediate files

This makes the final image much smaller — typically 50-100 MB instead of 1+ GB. Smaller images mean faster pulls, less disk usage, and a smaller attack surface. They also start faster and use less memory.

### Docker Build Cache

Docker caches each layer of a Dockerfile. If a layer hasn't changed since the last build, Docker reuses the cached version. This means subsequent builds are much faster.

For example, if you only change source code (not `Cargo.toml`), Docker will:
1. Reuse the cached `rust:slim-bookworm` base image ✓
2. Reuse the cached `apt-get install` layer ✓
3. Reuse the cached `COPY Cargo.toml Cargo.lock` layer ✓
4. Rebuild `COPY src ./src` (changed) ✗
5. Rebuild `cargo build --release` (because src changed) ✗

This can save significant time during development iterations.

## Docker Compose: Orchestrating the Stack

FicHub needs three services to run: PostgreSQL, Redis, and the application itself. Docker Compose makes it easy to define and run all of them together.

Here's `docker-compose.yml`:

```yaml
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: fichub
      POSTGRES_PASSWORD: fichub
      POSTGRES_USER: fichub
    volumes:
      - pgdata:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U fichub"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5

  calibre:
    build:
      context: .
      dockerfile: docker/calibre.Dockerfile
    volumes:
      - cache:/app/cache
      - tmp:/app/tmp

  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgres://fichub:***@postgres:5432/fichub
      REDIS_URL: redis://redis:6379
      CACHE_DIR: /app/cache
      TMP_DIR: /app/tmp
      CALIBRE_CONTAINER: calibre
      PORT: 3000
      FRONTEND_DIR: /app/frontend
      RUST_LOG: info,fichub=debug
    volumes:
      - cache:/app/cache
      - tmp:/app/tmp
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy

volumes:
  pgdata:
  cache:
  tmp:
```

### PostgreSQL

```yaml
postgres:
  image: postgres:16-alpine
  environment:
    POSTGRES_DB: fichub
    POSTGRES_PASSWORD: fichub
    POSTGRES_USER: fichub
  volumes:
    - pgdata:/var/lib/postgresql/data
```

FicHub uses PostgreSQL 16 (the Alpine variant, which is smaller — about 80 MB instead of 350 MB for the full image). The environment variables configure PostgreSQL:

- `POSTGRES_DB` — Creates a database called `fichub`
- `POSTGRES_USER` — Creates a user called `fichub`
- `POSTGRES_PASSWORD` — Sets the password for that user

The `pgdata` named volume persists the database across container restarts — you won't lose your data when you run `docker compose down` and `docker compose up` again.

The healthcheck ensures that the app service doesn't start until PostgreSQL is actually ready to accept connections:

```yaml
healthcheck:
  test: ["CMD-SHELL", "pg_isready -U fichub"]
  interval: 5s
  timeout: 5s
  retries: 5
```

This runs `pg_isready` every 5 seconds. If it fails 5 times in a row (25 seconds), PostgreSQL is marked as unhealthy. The `depends_on: condition: service_healthy` on the app service means FicHub won't start until PostgreSQL passes its healthcheck.

### Redis

```yaml
redis:
  image: redis:7-alpine
  ports:
    - "6379:6379"
```

Redis 7 (also Alpine) handles rate limiting and recommendation caching. The healthcheck pings Redis to confirm it's alive:

```yaml
healthcheck:
  test: ["CMD", "redis-cli", "ping"]
  interval: 5s
  timeout: 5s
  retries: 5
```

Unlike PostgreSQL, Redis doesn't need a persistent volume — the rate limit data and recommendation cache are ephemeral and can be lost without issues. If Redis restarts, the rate limit counters reset (which is actually fine — it gives everyone a fresh start).

### Calibre

```yaml
calibre:
  build:
    context: .
    dockerfile: docker/calibre.Dockerfile
  volumes:
    - cache:/app/cache
    - tmp:/app/tmp
```

This is the sidecar container that handles MOBI and PDF conversion. When FicHub needs to convert an EPUB to MOBI or PDF, it talks to the Calibre container. The Calibre container shares the `cache` and `tmp` volumes with the main app, so it can read EPUBs from the cache and write converted files back.

FicHub communicates with the Calibre container using Docker's networking — it calls the Calibre container by its service name (`calibre`), which Docker Compose resolves automatically via its built-in DNS server.

The Calibre container is built from a separate Dockerfile (`docker/calibre.Dockerfile`) that installs Calibre and runs a conversion server. Calibre is a large package (hundreds of MB), so isolating it in a separate container keeps the main app image small.

### The Application

```yaml
app:
  build: .
  ports:
    - "3000:3000"
  environment:
    DATABASE_URL: postgres://fichub:***@postgres:5432/fichub
    REDIS_URL: redis://redis:6379
    CACHE_DIR: /app/cache
    TMP_DIR: /app/tmp
    CALIBRE_CONTAINER: calibre
    PORT: 3000
    FRONTEND_DIR: /app/frontend
    RUST_LOG: info,fichub=debug
  depends_on:
    postgres:
      condition: service_healthy
    redis:
      condition: service_healthy
```

The app service:

1. **Builds from the Dockerfile** in the root directory (`.`)
2. **Maps port 3000** to the host (`host:container`)
3. **Sets environment variables** that the Rust app reads via `Config::from_env()`
4. **Shares volumes** for cache and temp files (same volumes as Calibre)
5. **Depends on** PostgreSQL and Redis, waiting for their healthchecks to pass

Notice that `DATABASE_URL` uses the Docker network hostname `postgres` and `REDIS_URL` uses `redis`. Inside Docker Compose, containers can reach each other by service name. Docker Compose creates a virtual network and registers each service's name in its built-in DNS. This is different from local development where you'd use `localhost`.

## Environment Variables in Docker

The environment variables in `docker-compose.yml` correspond directly to the `Config::from_env()` method we looked at in earlier chapters:

| Variable | Value | Purpose |
|----------|-------|---------|
| `DATABASE_URL` | `postgres://fichub:***@postgres:5432/fichub` | PostgreSQL connection string |
| `REDIS_URL` | `redis://redis:6379` | Redis connection string |
| `CACHE_DIR` | `/app/cache` | Where cached exports are stored |
| `TMP_DIR` | `/app/tmp` | Temporary files during export |
| `CALIBRE_CONTAINER` | `calibre` | Docker service name for Calibre |
| `PORT` | `3000` | HTTP server port |
| `FRONTEND_DIR` | `/app/frontend` | Path to SvelteKit build output |
| `RUST_LOG` | `info,fichub=debug` | Log levels |

In production, you'd override these with a `.env` file or Docker secrets. The `docker-compose.yml` shown here uses sensible defaults for getting started.

### Security Note

The `DATABASE_URL` in the compose file contains a password (`***`). In a real deployment, you'd want to:

1. Use a `.env` file for secrets (add it to `.gitignore`)
2. Or use Docker secrets for more secure secret management
3. Or use an environment-specific compose override file

Never commit real passwords to version control.

## Volumes: Persistence Across Restarts

Docker Compose defines three named volumes:

```yaml
volumes:
  pgdata:
  cache:
  tmp:
```

- **`pgdata`** — PostgreSQL's data directory. This is the most critical volume — it contains your database. Without it, you'd lose all fic metadata, export logs, recommendations, and tags every time you restart. Docker stores this in its own storage area (usually `/var/lib/docker/volumes/fichub_pgdata/_data/`).

- **`cache`** — Exported EPUBs, HTML bundles, MOBIs, and PDFs. If this is lost, FicHub can regenerate exports on demand, but it means extra scraping and processing time. For a busy server, regenerating all cached exports could take hours.

- **`tmp`** — Temporary files during export. This can be lost without issues. It's only used as a staging area during EPUB generation.

Named volumes are managed by Docker and persist across container restarts. They're stored in Docker's own storage area and are independent of the container's lifecycle. When you run `docker compose down`, containers are stopped and removed, but volumes persist. Only `docker compose down -v` removes volumes.

## Building and Running

### First-Time Setup

```bash
# Clone the repository
git clone https://github.com/youruser/fichub.git
cd fichub

# Build and start all services
docker compose up -d

# Watch the logs
docker compose logs -f app
```

The first build takes a while — Docker needs to download base images and compile the Rust binary. Subsequent builds are faster because Docker caches layers.

### Running in the Foreground

For development, you might want to see the logs:

```bash
docker compose up
```

This runs all services in the foreground with interleaved log output. You'll see PostgreSQL initializing, Redis starting, Calibre loading, and FicHub compiling and starting. Press Ctrl+C to stop all services.

### Running in the Background

For production:

```bash
docker compose up -d
```

The `-d` flag runs containers in detached mode. They'll keep running even after you close the terminal. You can check their status with `docker compose ps`.

### Checking Status

```bash
docker compose ps
```

This shows which containers are running, their ports, their health status, and their uptime. You should see all four services (postgres, redis, calibre, app) in a "running" state.

### Viewing Logs

```bash
# All services
docker compose logs

# Just the app
docker compose logs app

# Follow mode (like tail -f)
docker compose logs -f app

# Last 100 lines
docker compose logs --tail 100 app

# Since a specific time
docker compose logs --since 10m app
```

### Stopping

```bash
# Stop all containers (preserves volumes)
docker compose down

# Stop AND remove volumes (destroys data!)
docker compose down -v
```

⚠️ **Watch Out:** Never run `docker compose down -v` in production. The `-v` flag removes volumes, which deletes your PostgreSQL database and cached exports. Use `docker compose down` (without `-v`) to preserve data.

### Rebuilding After Code Changes

When you change the Rust code, you need to rebuild the Docker image:

```bash
# Rebuild just the app service
docker compose build app

# Or rebuild everything
docker compose build

# Then restart
docker compose up -d
```

Docker Compose will use cached layers for unchanged parts, so rebuilds are faster than fresh builds.

## The Calibre Sidecar for MOBI/PDF Conversion

FicHub generates EPUB files natively in Rust. But MOBI and PDF formats require Calibre — a powerful ebook management tool with conversion capabilities. Rather than installing Calibre in the main container (which would bloat the image enormously — Calibre is over 500 MB), FicHub uses a "sidecar" pattern:

1. The main app generates an EPUB
2. If MOBI or PDF is requested, it communicates with the Calibre container
3. Calibre reads the EPUB from the shared cache volume
4. Calibre converts it and writes the result back to the cache volume
5. The main app serves the converted file

This keeps the main container lean while still supporting all export formats. The Calibre container is built from a separate Dockerfile (`docker/calibre.Dockerfile`) that installs Calibre and runs a conversion server.

The `CALIBRE_CONTAINER` environment variable tells the main app what Docker service name to use when communicating with Calibre. If you don't need MOBI/PDF conversion, you can leave `CALIBRE_CONTAINER` empty and FicHub will skip the Calibre integration. The epub and html exports still work — only mobi and pdf are affected.

## Docker Networking

Docker Compose creates a default network for all services. Containers on this network can reach each other by service name:

- `postgres:5432` — PostgreSQL from the app
- `redis:6379` — Redis from the app
- `calibre` — Calibre from the app

The `ports` mapping in the compose file publishes ports to the host machine:

- `5432:5432` — PostgreSQL is accessible from the host (useful for debugging)
- `6379:6379` — Redis is accessible from the host
- `3000:3000` — FicHub is accessible from the host

If you don't need external access to PostgreSQL or Redis, remove their port mappings. This is actually more secure — it prevents other processes on the host from connecting to your database.

## Docker Build Optimization

Building Rust in Docker can be slow because of dependency compilation. Here are some optimization techniques:

### Cache Cargo Dependencies

The key insight is that Cargo dependencies change much less frequently than source code. By copying `Cargo.toml` and `Cargo.lock` first, Docker can cache the dependency compilation:

```dockerfile
# Copy manifests first (for dependency caching)
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to compile dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Now copy real source and build
COPY src ./src
RUN cargo build --release
```

This creates a dummy build that compiles all dependencies, then reuses that cache when building with the real source code. For FicHub, which has many dependencies (axum, sqlx, reqwest, serde, etc.), this can save 5+ minutes on subsequent builds.

### Use cargo-chef

For even better caching, use `cargo-chef`:

```dockerfile
FROM rust:slim-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release
```

`cargo-chef` separates dependency compilation from source compilation, giving Docker more opportunities to cache layers. It analyzes your `Cargo.toml` and `Cargo.lock` to create a "recipe" of just the dependency compilation steps, then cooks that recipe in a separate layer. When you change source code, only the final `cargo build` layer needs to rebuild — all dependencies are already compiled and cached.

### Build the Frontend in Docker

If you want to build the SvelteKit frontend inside Docker (to avoid installing Node.js on your dev machine), you can add another build stage:

```dockerfile
# Stage 1: Build the frontend
FROM node:20-alpine AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# Stage 2: Build the Rust binary (existing builder stage)
FROM rust:slim-bookworm AS builder
# ... existing build steps ...

# Stage 3: Runtime
FROM debian:bookworm-slim
# ... existing runtime setup ...
COPY --from=frontend-builder /app/frontend/build /app/frontend
```

This adds a Node.js build stage that compiles the SvelteKit frontend, and the runtime stage copies the build output into the container. The result is a fully self-contained Docker image with both the backend API and the frontend web UI.

### Docker Security Best Practices

For production deployments, consider these security additions to your Dockerfile:

```dockerfile
# Don't run as root
RUN useradd -r -s /bin/false fichub
USER fichub

# Use a non-root base image
# (debian:bookworm-slim already runs as root, but you can add USER)

# Don't install unnecessary packages
# (already handled by using slim images)

# Scan for vulnerabilities
# docker scout cves fichub:latest
```

Running as a non-root user inside the container is a defense-in-depth measure. Even if an attacker breaks out of the container, they'd only have the permissions of the `fichub` user, not root.

## Production Hardening

Whether you're running Docker or bare-metal, there are some production considerations that apply to both:

### TLS/HTTPS

FicHub doesn't handle TLS directly. In production, you'd put an nginx or Caddy reverse proxy in front that terminates TLS:

```nginx
server {
    listen 443 ssl http2;
    server_name fichub.yourdomain.com;

    ssl_certificate /etc/letsencrypt/live/fichub.yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/fichub.yourdomain.com/privkey.pem;

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

The proxy also handles:
- Request logging
- Rate limiting (at the edge, before FicHub even sees the request)
- Gzip compression for API responses
- Static asset caching
- Security headers (HSTS, CSP, etc.)

### Log Rotation

FicHub uses `tracing` for structured logging. In production, logs can grow quickly. Set up log rotation:

```bash
# /etc/logrotate.d/fichub
/home/fichub/logs/*.log {
    daily
    rotate 7
    compress
    delaycompress
    notifempty
    copytruncate
}
```

Or if running in Docker, use the `json-file` log driver with max-size:

```yaml
logging:
  driver: json-file
  options:
    max-size: "10m"
    max-file: "3"
```

### Backup Strategy

For a production FicHub deployment, back up these things:

1. **PostgreSQL database** — `pg_dump fichub > backup.sql`
2. **Cache directory** — `tar czf cache-backup.tar.gz /home/fichub/cache/`
3. **Configuration** — The `.env` file and systemd service file
4. **Migrations** — Already in version control, so just `git clone` on a new machine

A simple backup script:

```bash
#!/bin/bash
BACKUP_DIR="/backup/fichub/$(date +%Y%m%d)"
mkdir -p "$BACKUP_DIR"

# Database
sudo -u fichub pg_dump fichub > "$BACKUP_DIR/fichub.sql"

# Cache (just the manifest, not all files)
ls /home/fichub/cache/ > "$BACKUP_DIR/cache-manifest.txt"

# Config
cp /home/fichub/.env "$BACKUP_DIR/"
cp /etc/systemd/system/fichub.service "$BACKUP_DIR/"

# Cleanup old backups (keep 30 days)
find /backup/fichub/ -maxdepth 1 -mtime +30 -exec rm -rf {} +
```

Run this daily via cron:

```bash
0 2 * * * /home/fichub/backup.sh >> /var/log/fichub-backup.log 2>&1
```

The 2 AM timing ensures backups happen during low-traffic hours.

## 🧪 Try It Yourself

1. Install Docker and Docker Compose if you haven't already
2. Run `docker compose up` in the FicHub directory
3. Wait for all services to start (watch the logs for "Listening on 0.0.0.0:3000")
4. Open `http://localhost:3000/` in your browser
5. Try `docker compose ps` to see all running containers
6. Try `docker compose exec postgres psql -U fichub fichub` to explore the database
7. Try stopping and restarting: `docker compose down && docker compose up -d`
8. Notice that your data persists across restarts (thanks to named volumes)

## ⚠️ Watch Out

The biggest pitfall with Docker is the database. When you run `docker compose down -v`, the PostgreSQL volume is destroyed. All your data — fic metadata, export logs, recommendations, tags — is gone. If you're testing, this is fine. If you're in production, always use named volumes and never use `-v` unless you really mean it.

Another common issue: the first `cargo build --release` inside Docker takes 5-10 minutes. If you're making code changes and rebuilding frequently, consider using `cargo-watch` locally and only building Docker images for deployment. Alternatively, use Docker build caching as described above.

A third issue: the `ports` mapping exposes PostgreSQL to the host machine. In production, remove the PostgreSQL port mapping to prevent unauthorized access. The app can reach PostgreSQL through Docker's internal network, and that's all you need.

---

# Chapter 28: Cross-Compilation and Orange Pi Deploy

Docker is great for deployment, but sometimes you want to run your application directly on a machine without the overhead of containers. Maybe you have a small ARM board like an Orange Pi. Maybe you want the simplest possible deployment: copy a binary, run it, done.

That's where cross-compilation comes in.

## What Is Cross-Compilation?

Normally, when you run `cargo build --release`, the compiler builds a binary for your current machine. If you're on an x86_64 Linux machine, you get an x86_64 Linux binary. That binary won't run on an ARM machine like a Raspberry Pi or Orange Pi.

Cross-compilation means building a binary on one machine that runs on a different machine. You're still sitting at your x86_64 desktop, but you're telling the Rust compiler: "Please make an ARM64 binary instead."

This requires two things:
1. **A target toolchain** — The Rust compiler needs to know how to generate ARM64 machine code
2. **A cross-linker** — The final linking step needs a linker that can create ARM64 executables

Rust's cross-compilation support is excellent. The compiler itself handles most architectures natively. You just need to install the target and provide a linker. This is one of Rust's strengths over languages like C/C++ where cross-compilation can be a nightmare of configuring sysroots, compilers, and toolchains.

## The Orange Pi 5: A Small ARM Server

The Orange Pi 5 is a small single-board computer based on the Rockchip RK3588S processor. It has:
- An ARM64 (AArch64) processor — 4 performance cores + 4 efficiency cores
- 4GB, 8GB, or 16GB of LPDDR4/5 RAM
- Gigabit Ethernet
- eMMC and SD card storage
- USB 3.0, PCIe, HDMI
- Very low power consumption (5-15 watts)

It's a popular choice for self-hosting because it's powerful enough to run services like FicHub but small enough to fit on a shelf and use minimal power. Think of it as a Raspberry Pi's bigger, faster cousin — roughly 3-5x the CPU performance of a Raspberry Pi 4.

We're going to cross-compile FicHub on an x86_64 machine and deploy it to an Orange Pi 5 running an ARM64 Linux distribution (like Armbian or Manjaro ARM).

### Why Not Just Build on the Pi?

You absolutely could install the Rust toolchain on the Pi and build there. But:

- The Pi's CPU is slower than a desktop, so compilation takes 5-10x longer
- Building on the Pi consumes resources that could be serving requests
- Cross-compilation lets you build in a CI/CD pipeline on faster hardware
- You can test the binary on your desktop (via QEMU) before deploying

Cross-compilation is a skill that pays off whenever you deploy to embedded or ARM devices.

## Installing the AArch64 Target

First, you need to tell Rust that you want to target ARM64 Linux. This is a one-time setup step on your development machine:

```bash
rustup target add aarch64-unknown-linux-gnu
```

This downloads the Rust standard library pre-compiled for ARM64 Linux. The standard library includes all the fundamental types and functions that Rust programs need — things like string handling, collections, I/O, networking, etc.

You can verify it's installed:

```bash
rustup target list --installed
# Should show: aarch64-unknown-linux-gnu (and your native target)
```

The target name follows a triple format: `{arch}-{vendor}-{os}-{env}`. For ARM64 Linux, it's `aarch64-unknown-linux-gnu`. The `unknown` means we don't care about the vendor, and `gnu` means we're linking against glibc (as opposed to `musl` for a fully static binary).

### Alternative: musl for Fully Static Binaries

If you want a binary that doesn't depend on glibc at all (which means it'll run on ANY Linux, regardless of the system's glibc version), you can use musl:

```bash
rustup target add aarch64-unknown-linux-musl
```

Musl-based binaries are fully static — they don't link against any system libraries. This is useful for deployment to minimal Linux distributions or containers without glibc. However, musl has some limitations (less mature threading support, different DNS resolution behavior), so glibc is usually the safer choice for a server application.

## The Cross-Compiler

Rust can generate ARM64 machine code, but it needs help with linking. The linker is the program that takes all the compiled object files and combines them into a single executable. For cross-compilation, you need a linker that can create ARM64 executables.

On Arch Linux or similar distributions:

```bash
sudo pacman -S aarch64-linux-gnu-gcc
```

On Ubuntu/Debian:

```bash
sudo apt install gcc-aarch64-linux-gnu
```

On Fedora:

```bash
sudo dnf install gcc-aarch64-linux-gnu
```

On macOS (via Homebrew):

```bash
brew install aarch64-elf-gcc
# Note: macOS cross-compilation to Linux requires more setup
```

This installs a full cross-compilation toolchain including:
- `aarch64-linux-gnu-gcc` — The cross-linker (and C compiler)
- `aarch64-linux-gnu-ld` — The cross-assembler/linker
- `aarch64-linux-gnu-ar` — The cross-archiver
- Standard ARM64 libraries (like `libgcc`)

You can verify the installation:

```bash
aarch64-linux-gnu-gcc --version
# Should show: aarch64-linux-gnu-gcc (GCC) ...
```

## Setting the Linker in .cargo/config.toml

Now you need to tell Cargo to use the cross-linker when building for the `aarch64-unknown-linux-gnu` target. Create (or edit) `.cargo/config.toml` in your project root:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

This configuration tells Cargo: "When I'm building for `aarch64-unknown-linux-gnu`, use `aarch64-linux-gnu-gcc` as the linker."

You can put this in the project's `.cargo/config.toml` (so it's checked into git and shared with the team) or in `~/.cargo/config.toml` (so it applies to all your projects). For FicHub, the project-level config is better because it ensures everyone on the team uses the same cross-compilation setup.

Without this configuration, Cargo would try to use the system's default linker (usually `cc`), which creates x86_64 binaries. The cross-linker is essential for producing ARM64 executables.

## Building the Binary

Now the fun part. One command to build for ARM64:

```bash
cargo build --release --target aarch64-unknown-linux-gnu
```

This:
1. Compiles all Rust code targeting ARM64 architecture
2. Links using the `aarch64-linux-gnu-gcc` cross-linker
3. Produces an executable at `target/aarch64-unknown-linux-gnu/release/fichub`

The build takes a few minutes (cross-compilation can be slightly slower than native compilation because the cross-linker has more work to do). When it's done, you can verify the binary:

```bash
file target/aarch64-unknown-linux-gnu/release/fichub
# Should show: ELF 64-bit LSB executable, ARM aarch64, ...
```

If you see "ARM aarch64" in the output, you've successfully cross-compiled!

You can also check the binary size:

```bash
ls -lh target/aarch64-unknown-linux-gnu/release/fichub
# Typically 15-30 MB for a release build with full optimizations
```

The binary is much larger than the debug build because release mode includes all optimizations and debug symbols. You can strip debug symbols to make it smaller:

```bash
aarch64-linux-gnu-strip target/aarch64-unknown-linux-gnu/release/fichub
ls -lh target/aarch64-unknown-linux-gnu/release/fichub
# Now typically 10-15 MB
```

Stripping removes debug information that's only useful for debugging, not for running the application. It's a safe operation that doesn't affect functionality.

## Shipping the Binary: rsync to the Pi

Now that you have an ARM64 binary, you need to get it to the Orange Pi. The simplest way is `rsync` over SSH:

```bash
# First, make sure SSH is set up on the Pi
ssh-copy-id user@192.168.1.100

# Copy the binary
rsync -avz target/aarch64-unknown-linux-gnu/release/fichub \
    user@192.168.1.100:~/

# Copy the migrations
rsync -avz migrations/ user@192.168.1.100:~/migrations/

# Copy the frontend build (if serving from the Pi)
rsync -avz frontend/build/ user@192.168.1.100:~/frontend/build/

# Copy the .env file
rsync -avz .env user@192.168.1.100:~/
```

`rsync` is smart about transfers — it only sends the parts of files that have changed, so subsequent deploys are fast. The `-a` flag preserves permissions and timestamps, `-v` shows what's being transferred, and `-z` compresses the data during transfer.

You can also create a simple deploy script:

```bash
#!/bin/bash
# deploy.sh — cross-compile and ship to the Pi

set -e

PI_HOST="192.168.1.100"
PI_USER="user"

echo "Building for aarch64..."
cargo build --release --target aarch64-unknown-linux-gnu

echo "Deploying binary..."
rsync -avz --progress \
    target/aarch64-unknown-linux-gnu/release/fichub \
    ${PI_USER}@${PI_HOST}:~/

echo "Deploying migrations..."
rsync -avz --progress \
    migrations/ \
    ${PI_USER}@${PI_HOST}:~/migrations/

echo "Deploying frontend..."
rsync -avz --progress \
    frontend/build/ \
    ${PI_USER}@${PI_HOST}:~/frontend/build/

echo "Deploying .env..."
rsync -avz --progress \
    .env \
    ${PI_USER}@${PI_HOST}:~/

echo "Restarting service..."
ssh ${PI_USER}@${PI_HOST} "sudo systemctl restart fichub"

echo "Done! Verify with: curl http://${PI_HOST}:3000/api/"
```

Save this as `deploy.sh`, make it executable (`chmod +x deploy.sh`), and you have a one-command deployment pipeline.

## Creating the Systemd Service File

On the Orange Pi, you want FicHub to start automatically when the machine boots and restart if it crashes. That's what systemd is for.

Create the service file on the Pi:

```bash
sudo nano /etc/systemd/system/fichub.service
```

Here's the service file:

```ini
[Unit]
Description=FicHub - Fanfiction Download Server
After=network.target postgresql.service redis.service
Wants=postgresql.service redis.service

[Service]
Type=simple
User=fichub
Group=fichub
WorkingDirectory=/home/fichub
ExecStart=/home/fichub/fichub
Restart=always
RestartSec=5

# Environment
EnvironmentFile=/home/fichub/.env

# Security
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/home/fichub/cache /home/fichub/tmp
PrivateTmp=true

# Resource limits
LimitNOFILE=65536
MemoryMax=512M

[Install]
WantedBy=multi-user.target
```

Let's break this down section by section.

### [Unit] Section

```ini
[Unit]
Description=FicHub - Fanfiction Download Server
After=network.target postgresql.service redis.service
Wants=postgresql.service redis.service
```

The `Description` is what shows up in `systemctl status` and journal logs. The `After` line tells systemd to start FicHub after the network is up and after PostgreSQL and Redis are running. The `Wants` line means PostgreSQL and Redis are "wanted" (preferred) but not strictly required — if they're not installed, FicHub still starts (it'll fail to connect to the database, but at least systemd doesn't complain about missing dependencies).

This is important for boot ordering. Without `After=network.target`, FicHub might try to start before the network interface is configured, which would cause the bind to fail.

### [Service] Section

```ini
[Service]
Type=simple
User=fichub
Group=fichub
WorkingDirectory=/home/fichub
ExecStart=/home/fichub/fichub
Restart=always
RestartSec=5
```

- **`Type=simple`** — The process runs in the foreground (which is exactly what Axum does with `axum::serve().await`). systemd tracks the process by its PID.

- **`User=fichub`** and **`Group=fichub`** — Run as a dedicated `fichub` user, not root. This is a security best practice. If FicHub is compromised, the attacker only gets the permissions of the `fichub` user, not the entire system.

- **`WorkingDirectory=/home/fichub`** — Sets the working directory to where the binary and data live. This is important because FicHub's default paths (like `./cache` and `./frontend/build`) are relative to the working directory.

- **`ExecStart=/home/fichub/fichub`** — The full path to the FicHub binary.

- **`Restart=always`** — If the process crashes, systemd restarts it. This is crucial for a server that should be always-on.

- **`RestartSec=5`** — Wait 5 seconds before restarting. This prevents rapid restart loops — if FicHub keeps crashing immediately (say, because the database is down), it won't restart more than once every 5 seconds.

### Environment File

```ini
EnvironmentFile=/home/fichub/.env
```

This loads environment variables from a file, just like `dotenvy::dotenv()` does in the Rust code. The `.env` file on the Pi might look like:

```bash
DATABASE_URL=postgres://fichub:secretpassword@localhost:5432/fichub
REDIS_URL=redis://localhost:6379
CACHE_DIR=/home/fichub/cache
TMP_DIR=/home/fichub/tmp
PORT=3000
FRONTEND_DIR=/home/fichub/frontend/build
RUST_LOG=info,fichub=debug
```

Notice the `DATABASE_URL` uses `localhost` instead of `postgres` — this is a bare-metal deployment, not Docker, so PostgreSQL runs on the same machine.

### Security Hardening

```ini
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/home/fichub/cache /home/fichub/tmp
PrivateTmp=true
```

These lines limit what FicHub can do if it's compromised:

- **`NoNewPrivileges`** — Prevents the process from gaining new privileges (like setuid). This is a defense-in-depth measure.

- **`ProtectSystem=strict`** — Makes the entire filesystem read-only except explicitly listed paths. FicHub can't write to `/etc`, `/var`, or any other system directory.

- **`ProtectHome=read-only`** — Makes `/home` read-only. FicHub can read its own home directory but not write to it directly — except for the explicitly listed paths.

- **`ReadWritePaths`** — Explicitly allows writing to cache and tmp directories. These are the only writable locations outside of tmpfs.

- **`PrivateTmp=true`** — Gives the process its own `/tmp` (a tmpfs mount). This prevents `/tmp`-based attacks — if an attacker creates a file in `/tmp`, FicHub won't see it.

### Resource Limits

```ini
LimitNOFILE=65536
MemoryMax=512M
```

- **`LimitNOFILE`** — Sets the maximum number of open file descriptors to 65536. This is important for handling many concurrent connections — each open socket, database connection, and file handle counts as an open file descriptor. The default (usually 1024) is too low for a server.

- **`MemoryMax`** — Caps memory usage at 512MB. If FicHub exceeds this (due to a memory leak or unexpected load), systemd kills it and restarts it. This prevents a runaway process from crashing the entire Pi by exhausting memory.

### [Install] Section

```ini
[Install]
WantedBy=multi-user.target
```

This means FicHub starts when the system reaches multi-user mode (the normal boot target for servers). You could use `graphical.target` instead if the Pi has a desktop environment, but for a headless server, `multi-user.target` is correct.

## Enabling and Starting the Service

On the Pi, after creating the service file:

```bash
# Create a dedicated user
sudo useradd -r -s /bin/false fichub
sudo mkdir -p /home/fichub/cache /home/fichub/tmp /home/fichub/frontend/build
sudo chown -R fichub:fichub /home/fichub

# Copy files (from the deploy script or manually)
sudo cp /home/user/fichub /home/fichub/
sudo cp /home/user/.env /home/fichub/
sudo cp -r /home/user/migrations /home/fichub/
sudo cp -r /home/user/frontend/build/* /home/fichub/frontend/build/
sudo chown -R fichub:fichub /home/fichub

# Reload systemd (after creating/editing service files)
sudo systemctl daemon-reload

# Enable (start on boot)
sudo systemctl enable fichub

# Start immediately
sudo systemctl start fichub

# Check status
sudo systemctl status fichub
```

The `daemon-reload` command tells systemd to re-read its configuration files. This is necessary after creating or editing any service file. Without it, systemd continues using the old configuration.

The `enable` command creates symlinks in the systemd directories so FicHub starts automatically on boot. It doesn't start the service — it just ensures it will be started in the future.

The `start` command starts FicHub immediately. You should see output like:

```
● fichub.service - FicHub - Fanfiction Download Server
     Loaded: loaded (/etc/systemd/system/fichub.service; enabled; ...)
     Active: active (running) since ...
   Main PID: 12345 (fichub)
      Tasks: 8 (limit: 4589)
     Memory: 45.2M
        CPU: 1.234s
     CGroup: /system.slice/fichub.service
             └─12345 /home/fichub/fichub
```

The `active (running)` status confirms that FicHub started successfully. The PID and memory usage are shown.

## Verifying with curl

After the service is running, verify it works:

```bash
# From the Pi itself
curl http://localhost:3000/api/

# From another machine on the network
curl http://192.168.1.100:3000/api/

# Check the API documentation
curl http://192.168.1.100:3000/api/ | python3 -m json.tool

# Try the remote endpoint
curl http://192.168.1.100:3000/api/v0/remote

# Check the frontend
curl -I http://192.168.1.100:3000/
```

If you get JSON responses back, FicHub is running and accepting requests. The `-I` flag on the frontend request shows just the headers, confirming that `Content-Type: text/html` is returned.

For more detailed verification:

```bash
# Check logs
sudo journalctl -u fichub -f

# Check the service is healthy
sudo systemctl is-active fichub

# Check PostgreSQL connection
sudo -u fichub psql -h localhost -U fichub -d fichub -c "SELECT count(*) FROM fic_info;"

# Test an export
curl "http://localhost:3000/api/v0/epub?q=https://archiveofourown.org/works/12345"

# Check disk usage
du -sh /home/fichub/cache/

# Check memory usage
ps aux | grep fichub

# Check open file descriptors
ls /proc/$(pgrep fichub)/fd | wc -l
```

## The Deployment Checklist

Here's a complete checklist for deploying FicHub to an Orange Pi (or any ARM64 Linux machine):

### One-Time Setup (on the Pi)

- [ ] Install an ARM64 Linux distribution (Armbian, Manjaro ARM, Ubuntu ARM)
- [ ] Update the system: `sudo apt update && sudo apt upgrade`
- [ ] Install PostgreSQL: `sudo apt install postgresql`
- [ ] Install Redis: `sudo apt install redis-server`
- [ ] Create the database: `sudo -u postgres createdb fichub`
- [ ] Create database user: `sudo -u postgres createuser -P fichub`
- [ ] Grant permissions: `sudo -u postgres psql -c "GRANT ALL ON DATABASE fichub TO fichub;"`
- [ ] Create the fichub user: `sudo useradd -r -s /bin/false fichub`
- [ ] Create directories: `sudo mkdir -p /home/fichub/{cache,tmp,frontend/build,migrations}`
- [ ] Set permissions: `sudo chown -R fichub:fichub /home/fichub`
- [ ] Install `libpq` and `ca-certificates`: `sudo apt install libpq-dev ca-certificates`

### Cross-Compile (on your dev machine)

- [ ] Install the aarch64 target: `rustup target add aarch64-unknown-linux-gnu`
- [ ] Install the cross-linker: `sudo pacman -S aarch64-linux-gnu-gcc` (or equivalent)
- [ ] Configure `.cargo/config.toml` with the linker path
- [ ] Build: `cargo build --release --target aarch64-unknown-linux-gnu`
- [ ] Verify: `file target/aarch64-unknown-linux-gnu/release/fichub`
- [ ] Strip debug symbols: `aarch64-linux-gnu-strip target/aarch64-unknown-linux-gnu/release/fichub`

### Deploy (rsync)

- [ ] Copy the binary: `rsync target/aarch64-unknown-linux-gnu/release/fichub user@pi:~/`
- [ ] Copy migrations: `rsync -r migrations/ user@pi:~/migrations/`
- [ ] Copy frontend build: `rsync -r frontend/build/ user@pi:~/frontend/build/`
- [ ] Copy `.env` file: `rsync .env user@pi:~/`
- [ ] Set permissions: `ssh user@pi "sudo chown -R fichub:fichub /home/fichub"`

### Configure (on the Pi)

- [ ] Create systemd service file at `/etc/systemd/system/fichub.service`
- [ ] `sudo systemctl daemon-reload`
- [ ] `sudo systemctl enable fichub`
- [ ] `sudo systemctl start fichub`

### Verify

- [ ] `curl http://pi-ip:3000/api/` returns JSON
- [ ] `curl http://pi-ip:3000/` returns the frontend
- [ ] `sudo systemctl status fichub` shows `active (running)`
- [ ] `sudo journalctl -u fichub` shows no errors
- [ ] Test a fic export end-to-end

## Updating: Rebuild, Rsync, Restart

Once FicHub is deployed, updating is straightforward:

1. Pull latest code: `git pull`
2. Build: `cargo build --release --target aarch64-unknown-linux-gnu`
3. Deploy: `./deploy.sh`
4. The systemd service restarts automatically

Or if you wrote the `deploy.sh` script from earlier, it's literally one command:

```bash
./deploy.sh
```

The script builds, copies files, and restarts the service. From code change to deployed update, it takes about 30 seconds (assuming no dependency changes).

### Zero-Downtime Updates

For zero-downtime updates, you could run two instances of FicHub on different ports and use nginx to switch between them:

```bash
# Build the new binary
cargo build --release --target aarch64-unknown-linux-gnu

# Copy it with a different name
rsync target/aarch64-unknown-linux-gnu/release/fichub user@pi:~/fichub-new

# Start the new instance on a different port
ssh user@pi "PORT=3001 /home/fichub/fichub-new &"

# Switch nginx to the new port
ssh user@pi "sudo sed -i 's/3000/3001/' /etc/nginx/sites-enabled/fichub"
ssh user@pi "sudo nginx -s reload"

# Stop the old instance
ssh user@pi "sudo systemctl stop fichub"

# Move the new binary into place
ssh user@pi "mv /home/fichub/fichub-new /home/fichub/fichub"

# Update the service file to use port 3001 (or revert to 3000)
ssh user@pi "sudo systemctl start fichub"
```

This is more complex but ensures users never see a downtime blip.

### Health Check Script

For monitoring, you can set up a simple health check:

```bash
#!/bin/bash
# healthcheck.sh — check if FicHub is running

RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/api/)

if [ "$RESPONSE" != "200" ]; then
    echo "FicHub is down! Response code: $RESPONSE"
    sudo systemctl restart fichub
    echo "Restarted fichub"
fi
```

Run this from a cron job every 5 minutes:

```bash
crontab -e
*/5 * * * * /home/fichub/healthcheck.sh >> /var/log/fichub-health.log 2>&1
```

## Performance on the Orange Pi 5

The Orange Pi 5 with its RK3588S processor handles FicHub surprisingly well:

- **API responses** are sub-100ms for cached requests
- **First-time exports** take 2-5 seconds depending on fic length
- **Concurrent requests** are handled well thanks to Tokio's async runtime
- **Memory usage** typically stays under 100MB
- **CPU usage** is minimal at idle (a few percent)
- **Disk usage** depends on cached exports (each EPUB is 1-10MB)

For a personal fanfiction server, the Orange Pi 5 is more than enough. You could serve dozens of concurrent users without breaking a sweat. The RK3588S's 8 cores handle Tokio's thread pool easily, and the 8-16GB of RAM provides plenty of headroom.

### Monitoring Resource Usage

Keep an eye on resource usage with these commands:

```bash
# Memory
free -h

# Disk
df -h /home/fichub/cache/

# CPU
top -p $(pgrep fichub)

# Network connections
ss -tlnp | grep 3000

# Database size
sudo -u fichub psql -h localhost -U fichub -d fichub \
    -c "SELECT pg_size_pretty(pg_database_size('fichub'));"
```

## 🧪 Try It Yourself

1. Install the cross-compilation toolchain on your dev machine
2. Build for ARM64: `cargo build --release --target aarch64-unknown-linux-gnu`
3. Verify the binary: `file target/aarch64-unknown-linux-gnu/release/fichub`
4. If you have access to an ARM64 machine, copy the binary and run it
5. Try the deploy script workflow end-to-end
6. Monitor the service with `journalctl -u fichub -f` and watch the logs

## ⚠️ Watch Out

The most common cross-compilation issue is missing system libraries. If you see errors like "cannot find -lpq" or "cannot find -lssl", you need to install the ARM64 versions of those libraries on your cross-compilation machine. On Arch Linux: `sudo pacman -S aarch64-linux-gnu-postgresql-libs aarch64-linux-gnu-openssl`. On Ubuntu: `sudo apt install libpq-dev:arm64`.

Another pitfall: forgetting to install the runtime dependencies on the target machine. The cross-compiled binary is linked against `libpq` and `libssl`, so the Orange Pi needs those libraries installed. Run `sudo apt install libpq-dev ca-certificates` on the Pi before deploying.

A third issue: the `.env` file on the Pi uses `localhost` in the `DATABASE_URL`, not `postgres` as in Docker. The Docker network hostname `postgres` only works inside Docker Compose. On a bare metal deployment, it's `localhost` or the actual IP address of the database server.

A fourth issue: file permissions. If the `fichub` user can't read the binary, the `.env` file, or the frontend build, the service will fail silently. Always check `ls -la /home/fichub/` to verify permissions.

A fifth issue: the `ExecStart` path in the service file must be the full path to the binary. A relative path won't work because systemd changes to the `WorkingDirectory` before starting the process, but `ExecStart` is evaluated before that change takes effect.

---

# Summary

Part 6 brought everything together — from the internal architecture of the Axum router to Docker containers to cross-compiled deployments on ARM hardware.

## Chapter 25 Recap

The Axum router is the central nervous system of FicHub. The `build_router` function in `server.rs` creates a tree of routes that covers everything: API endpoints for export and metadata, cache download routes, recommendation and tagging APIs, curatorial tools, search, OPDS catalog feeds, and legacy redirects. The `AppState` struct bundles all shared resources (database pools, Redis connections, scraper registries, rate limiters) into a single `Arc` that's accessible from every handler. The `run()` function ties it all together — connecting to services, building state, constructing the router, binding to a port, and serving requests.

The key design decisions in the router are worth noting:
- Routes are organized by API version (`/api/v0/`) for future-proofing
- The OPDS routes provide e-reader integration without any additional setup
- Legacy redirects maintain backward compatibility with old bookmarks
- The SPA fallback ensures the SvelteKit frontend works for any URL
- Middleware layers (logging and CORS) are applied to all requests uniformly

## Chapter 26 Recap

FicHub serves its SvelteKit frontend directly from the Rust server using `tower-http`'s `ServeDir` and `ServeFile`. The SPA routing pattern — serve `index.html` for any path that doesn't match a real file — lets the SvelteKit JavaScript router handle client-side navigation. Immutable assets get long cache lifetimes thanks to content-hashed filenames, while `index.html` should never be cached.

The frontend module is intentionally minimal — just a constant for the default build directory. All the heavy lifting is done by `tower-http`, which provides efficient, async file serving with proper MIME type detection and conditional request support.

## Chapter 27 Recap

Docker makes deployment reproducible and portable. The multi-stage Dockerfile compiles Rust in a full build image and copies just the binary into a slim runtime image — reducing the final image from 1+ GB to 50-100 MB. Docker Compose orchestrates PostgreSQL, Redis, Calibre, and the FicHub app, with named volumes for persistence and healthchecks for proper startup ordering.

The Calibre sidecar pattern is a great example of the microservice approach — isolate heavy dependencies in separate containers and communicate through shared volumes. This keeps the main image lean while still supporting all export formats.

## Chapter 28 Recap

Cross-compilation lets you build ARM64 binaries on x86_64 machines with just a target add and a linker configuration. Deploying to an Orange Pi 5 via rsync and systemd gives you a lean, efficient personal server that starts on boot, restarts on crash, and handles security hardening out of the box. The deploy script reduces the update workflow to a single command.

The systemd service file demonstrates production-grade process management: dedicated users, security hardening, resource limits, automatic restarts, and dependency ordering. Combined with the health check script and cron job, you get a self-healing deployment that monitors itself and recovers from failures.

---

In the next part, we'll look at monitoring, testing, and operational concerns — how to keep FicHub running smoothly in production. We'll cover structured logging, Prometheus metrics, integration testing, and operational runbooks for common issues like database migrations, cache eviction, and rate limit tuning.
# Part 7: The SvelteKit Frontend

*Building the browser-side of FicHub — a single-page app that talks to the Rust backend we built in Parts 1–6.*

---

# Chapter 29: SvelteKit Fundamentals and SPA Mode

## From Rust to the Browser

For the last six parts of this book, we lived entirely in Rustland — building API routes, parsing fanfiction URLs, scraping chapter text, constructing EPUBs, and deploying with systemd. We set up PostgreSQL databases, wrote scraping engines for AO3 and FanFiction.net, implemented collaborative filtering for recommendations, and wired everything together with Axum routers and middleware. Our backend runs on port 8004, serves JSON over REST, and generates EPUB files on demand.

But a backend without a frontend is like a library without a reading room: all the books are there, but nobody can access them. We need a user interface — something that lets people paste a URL, click a button, and get an EPUB. Something that shows search results with pretty tag pills and word counts. Something that displays recommendations with community vote badges.

In this part of the book, we build the SvelteKit frontend that gives users a beautiful interface to FicHub. They paste a URL, click a button, and get an EPUB. They search for fics by fandom or word count. They see recommendations and vote on community suggestions. All of this happens in the browser, through a tiny API client that talks to our Rust server.

We chose SvelteKit for three reasons:

1. **Simplicity.** Svelte is famously minimal — no virtual DOM, no JSX, no complex state management libraries. Components are just HTML with some JavaScript sprinkled in. The compiler does the heavy lifting, turning your declarative components into efficient imperative DOM operations.

2. **SPA mode.** We don't need server-side rendering. Our Rust backend already serves the built files. SvelteKit can produce a pure client-side app that handles all routing in the browser — no server round-trips for navigation, no hydration mismatches, no SSR configuration headaches.

3. **Small bundle.** The compiled output is tiny — usually under 100KB of JavaScript, gzipped to under 30KB. That's dramatically smaller than a typical React or Vue bundle. Important when the app is served from the same machine running the Rust backend, and users might be on slow connections.

Let's start with the fundamentals.

## SvelteKit Review: Routes, Layouts, Pages

If you've used Next.js, Nuxt, or any other file-based routing framework, SvelteKit will feel familiar. The `src/routes/` directory is the heart of the app:

```
src/routes/
├── +layout.svelte        # Wraps every page (the app shell)
├── +layout.ts            # Shared config (SPA mode flags)
├── +page.svelte          # The root page (/)
├── [...slug]/
│   └── +page.ts          # Catch-all route (deep link support)
└── search/
    ├── +page.svelte      # The search page (/search)
    ├── +page.ts          # Search page data loader
    └── syntax/
        └── +page.svelte  # Syntax guide (/search/syntax)
```

Each `+page.svelte` file defines what the user sees at that URL. Each `+layout.svelte` wraps everything below it — think of it as a shared frame that persists across navigations. Each `+page.ts` (or `+layout.ts`) file loads data that gets passed down to the component as props.

SvelteKit uses these file conventions:

- **`+page.svelte`** — the UI for a route. Contains `<script>`, markup, and `<style>`.
- **`+page.ts`** (or `+page.js`) — the data loader for a route. Exports a `load` function that returns data passed to the component.
- **`+layout.svelte`** — shared UI that wraps child routes. The `{children}` slot contains the child page.
- **`+layout.ts`** — shared data loader and configuration. This is where we set SSR/prerender flags.
- **`[...slug]/+page.ts`** — a catch-all route that matches any path not handled by other routes.

The `+` prefix in file names is a SvelteKit convention that tells the framework "this is a special route file, not a regular component." Regular components in `src/lib/` don't have the `+` prefix — they're imported by route files.

### How Routing Works

When a user navigates to `/search?q=fluff`, SvelteKit does the following:

1. Matches the URL to `src/routes/search/+page.ts`
2. Runs the `load` function, which extracts the `q` parameter
3. Passes the data to `src/routes/search/+page.svelte` as the `data` prop
4. Renders the search page inside the layout (`+layout.svelte`)

All of this happens client-side in SPA mode. The browser never asks the server for a new HTML page — it just swaps JavaScript modules.

## SPA Mode: ssr=false, prerender=false

Here's the critical configuration that makes our frontend a single-page application. In `src/routes/+layout.ts`:

```typescript
// SPA mode: no SSR, no prerender. The backend serves the static build.
export const ssr = false;
export const prerender = false;
```

That's it. Two lines. But they fundamentally change how SvelteKit works:

**`ssr = false`** means the server never renders HTML. When a browser requests a page, the server sends a bare shell with a `<script>` tag, and the JavaScript takes over. All rendering happens client-side.

Without SSR, the initial HTML sent to the browser is just:

```html
<!DOCTYPE html>
<html>
  <head>
    <link rel="stylesheet" href="/_app/immutable/assets/0.css">
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/_app/immutable/entry/start.js"></script>
  </body>
</html>
```

The browser downloads the JavaScript, which then renders the entire UI. The initial paint is a blank page, but the JavaScript loads fast enough that users rarely notice.

**`prerender = false`** means SvelteKit doesn't try to pre-build any pages at build time. Every route is handled dynamically by the client-side router. This is important because we don't want SvelteKit to try to crawl our pages during build — there's nothing to crawl (the content is dynamic) and the crawl would fail anyway (the backend isn't running during build).

Why would we do this? Because our architecture looks like this:

```
Browser → Rust backend (port 8004) → serves static files
                                       ↑
                                       built by SvelteKit
Browser → Rust backend (port 8004) → handles /api/v0/* requests
```

The Rust backend serves both roles: it serves the static HTML/JS/CSS files AND handles API requests at `/api/v0/*`. The browser downloads the static files, the JavaScript runs, and from then on, all navigation happens client-side — no full page reloads, no server roundtrips for navigation.

```mermaid
graph LR
    A[Browser] -->|请求页面| B[Rust Backend]
    B -->|返回 index.html + JS| A
    A -->|SPA 路由| A
    A -->|API 调用| B
```

The beauty of this setup is that we get a fully static build output (just HTML, CSS, and JS files) that can be served by anything — nginx, a simple file server, or even our Rust backend's own static file handler. No Node.js runtime required in production. We built the backend in Rust for performance, and the frontend in JavaScript for the rich UI — and they communicate over a simple HTTP API.

## adapter-static: Building for Static Hosting

In `svelte.config.js`:

```javascript
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      pages: 'build',
      assets: 'build',
      fallback: 'index.html',
      strict: false,
    }),
  },
};

export default config;
```

The `adapter-static` package is one of several SvelteKit adapters. Each adapter targets a different deployment platform:

- `adapter-static` — static files (our choice)
- `adapter-node` — Node.js server
- `adapter-vercel` — Vercel
- `adapter-netlify` — Netlify

We chose `adapter-static` because we want to serve the frontend from our Rust backend. No serverless functions, no Node runtime — just static files.

The configuration options:

- **`pages: 'build'`** — Output directory for HTML pages. When you run `npm run build`, the built files go here.
- **`assets: 'build'`** — Output directory for static assets (JS, CSS, images). Same as `pages` — everything goes in one directory.
- **`fallback: 'index.html'`** — For SPA routing: any URL that doesn't match a pre-built file gets this HTML. This is what makes client-side routing work — the server always returns the same `index.html`, and the JavaScript router figures out what to display.
- **`strict: false`** — Don't fail if some pages can't be prerendered (since we're not prerendering anything anyway). Without this, the build might warn about prerendering failures.

When you run `npm run build`, SvelteKit produces a `build/` directory containing:

```
build/
├── index.html                 # The SPA shell
├── favicon.png                # App icon
└── _app/
    └── immutable/
        ├── assets/
        │   ├── 0.css          # Global styles
        │   └── ...
        ├── chunks/
        │   ├── 0-abc123.js    # App modules
        │   └── ...
        └── entry/
            ├── start.js       # Entry point
            └── ...
```

The total output is usually under 200KB — remarkably small for a full-featured SPA. The `_app/immutable/` directory structure includes content hashes in file names (that's what `abc123` represents), enabling aggressive browser caching. When you rebuild, the hashes change, so browsers automatically fetch the new versions.

Our Rust backend then serves this directory. In Part 6, we configured the static file handler to serve everything under `build/` and fall back to `index.html` for unknown paths. The `fallback: 'index.html'` configuration in SvelteKit makes this work: when a user navigates directly to `/search?q=fluff`, nginx (or our Rust backend) returns `index.html`, and the client-side JavaScript routes them correctly.

## The +layout.ts File: Disabling SSR

We already looked at the two-line file:

```typescript
// SPA mode: no SSR, no prerender. The backend serves the static build.
export const ssr = false;
export const prerender = false;
```

But let's think about what this actually means in practice. When you define `ssr = false` in a layout, it cascades to all child routes. Every page under this layout will be rendered client-side only.

The cascade works like this: if a child route defines `export const ssr = true`, it overrides the parent layout's `ssr = false` for that specific route. So you could have SSR for specific pages (like a blog post for SEO) while keeping the rest as SPA.

But we don't need that. Our entire app is a tool for fanfiction readers — there's no SEO requirement, no social media previews we need to optimize. Pure SPA is the right call. The simplicity of two lines vs. complex SSR configuration is one of those architecture decisions that pays dividends throughout the project.

### What Happens Without These Lines

If you accidentally remove `ssr = false`, SvelteKit defaults to SSR. This means:

1. The server tries to render each page to HTML
2. The server needs access to browser APIs (`window`, `document`) — which don't exist on the server
3. Components using `$state` and `$effect` runes don't work during SSR
4. API calls to `localhost:8004` would fail during SSR (the backend isn't running during build)

The result: cryptic errors like "window is not defined" or "document is not defined." If you see these errors, check that `ssr = false` is set.

## The +page.svelte File: The Root Page

Here's something that initially confused me when I first saw it:

```svelte
<script lang="ts">
  // The root page is intentionally empty: the tabbed UI lives in +layout.svelte.
  // This page renders nothing so the layout's Download tab is the default view.
</script>
```

That's the entire root page. It's... nothing. Empty. Not even any markup.

The comment explains why: all the real UI lives in the layout. The `+layout.svelte` file defines the top bar, tab navigation, and renders the appropriate tab content (Download, Recommendations, or Suggestions) based on which tab is active. The root page exists solely to satisfy SvelteKit's routing requirements.

Why separate them? Because the layout persists across navigations. When a user clicks from the home page to `/search`, the layout stays mounted — only the `{children}` slot changes. This means the top bar, tabs, and navigation search box don't re-render. State like the active tab and search input value is preserved.

If the layout UI were in `+page.svelte` instead, navigating to `/search` would unmount the layout and mount the search page, losing all the tab state. By putting the shared UI in the layout and leaving the page empty, we get persistent navigation.

The root page is like an empty picture frame — it exists so SvelteKit has something to render at `/`, but the real content comes from the layout.

## The [...slug]/+page.ts Catch-All Route

```typescript
import type { PageLoad } from './$types';

// SPA catch-all so deep links and refresh work under adapter-static fallback.
export const load: PageLoad = async ({ url }) => {
  return { path: url.pathname };
};
```

This is the safety net. Without it, if a user refreshes the browser on a deep link like `/some/deep/path`, the `adapter-static` fallback would serve `index.html`, but SvelteKit wouldn't know how to route it. The catch-all route captures any path that hasn't been matched by a more specific route and passes it to a page component.

The `[...slug]` syntax is SvelteKit's "rest parameter" — it matches any path segments. For example:

- `/anything` matches
- `/a/b/c/d` matches
- `/search` does NOT match (because `search/+page.ts` is more specific)
- `/search/syntax` does NOT match (because `search/syntax/+page.svelte` is more specific)

The catch-all only catches what the other routes miss.

In our case, the `path` data is passed but never really used — because the layout handles all the routing. But it's there as insurance. If we ever need to handle unknown routes gracefully (maybe showing a 404 or redirecting), we can add logic to the catch-all's page component:

```svelte
<!-- Example: if we wanted a 404 page -->
<script lang="ts">
  let { data } = $props();
</script>

<div class="card">
  <h1>Page Not Found</h1>
  <p>The path <code>{data.path}</code> doesn't exist.</p>
  <a href="/">Go Home</a>
</div>
```

## File Structure Overview

Let's map out the entire frontend:

```
frontend/
├── package.json                    # Dependencies and scripts
├── svelte.config.js                # SvelteKit config with adapter-static
├── vite.config.ts                  # Vite dev server with API proxy
├── src/
│   ├── app.css                     # Global styles (CSS variables, utilities)
│   ├── lib/
│   │   ├── api/
│   │   │   ├── types.ts            # TypeScript interfaces for all API responses
│   │   │   ├── client.ts           # API client functions (fetchExport, etc.)
│   │   │   └── search.ts           # Search API types and client
│   │   ├── search/
│   │   │   └── syntax.ts           # AO3-like search syntax parser
│   │   ├── util.ts                 # Formatting helpers (formatWords, stripHtml, etc.)
│   │   └── components/
│   │       ├── DownloadTab.svelte   # Main download feature
│   │       ├── RecommendationsTab.svelte  # Recommendation engine
│   │       └── SuggestionsTab.svelte     # Community suggestions + voting
│   └── routes/
│       ├── +layout.svelte           # App shell: topbar, tabs, search
│       ├── +layout.ts               # SPA mode (ssr=false, prerender=false)
│       ├── +page.svelte             # Root page (intentionally empty)
│       ├── [...slug]+page.ts        # Catch-all for deep links
│       └── search/
│           ├── +page.svelte         # Advanced search page
│           ├── +page.ts             # Search data loader
│           └── syntax/+page.svelte  # Syntax reference guide
```

Notice how clean this is. There are no configuration files for state management (no Redux, no Zustand, no Svelte stores), no routing libraries, no API middleware. SvelteKit handles routing. Svelte's reactivity handles state. The API client is just a few functions with `fetch` calls.

The `lib/` directory is SvelteKit's convention for shared code — it's importable as `$lib/...` from any file in the project. This is where we put:

- `api/` — types and client functions for talking to the backend
- `search/` — the syntax parser
- `util.ts` — small formatting helpers
- `components/` — reusable Svelte components (the three tabs)

The `routes/` directory contains only route-specific files. The components that render at each route are either in `+page.svelte` (for simple pages) or imported from `$lib/components/` (for complex ones like the tabs).

### Why This Structure?

This organization follows the "colocation" principle — related code lives together. The API types and client are in the same directory because they're tightly coupled. The search syntax parser is separate from the search API client because it's a different concern (parsing vs. HTTP). The components live in `lib/` because they're shared across routes (the tabs are used by the layout, not by a specific route).

The `routes/` directory stays lean — it only contains files that define routes. Business logic, API clients, and utility functions live in `lib/`. This separation makes it easy to find things:

- "Where's the download feature?" → `lib/components/DownloadTab.svelte`
- "What does the API return?" → `lib/api/types.ts`
- "How does search parsing work?" → `lib/search/syntax.ts`
- "What CSS variables are available?" → `src/app.css`

Compare this to a monolithic structure where everything lives in `routes/` — you'd have to search through dozens of files to find the API types. The `lib/` directory is a clear signal: "this code is used by multiple routes."

### The `$lib` Import Alias

Every import like `$lib/api/client` maps to `src/lib/api/client`. This is a SvelteKit convention that makes imports clean and consistent:

```typescript
// Without the alias (relative paths — fragile and ugly):
import { fetchExport } from '../../lib/api/client';

// With the alias (clean and absolute):
import { fetchExport } from '$lib/api/client';
```

The `$lib` alias is always available — no configuration needed. There are other aliases too: `$app` for SvelteKit internals, `$env` for environment variables.

## Svelte 5 Runes Recap

Our codebase uses Svelte 5's runes — the new reactive primitives that replaced the old `$:` and `let` magic. If you're coming from Svelte 4, runes might feel different at first. If you're new to Svelte, runes are the only version you'll learn. Either way, they're elegant and powerful.

### `$state` — Reactive State

```typescript
let url = $state('');
let loading = $state(false);
let result = $state<ExportResponse | null>(null);
```

`$state` creates a reactive variable. When you change it, any UI that depends on it automatically updates. It's like `let` but with superpowers — the compiler tracks reads and writes and knows exactly what to re-render.

Behind the scenes, Svelte compiles `$state` into fine-grained reactivity. When `loading` changes, only the parts of the DOM that depend on `loading` are updated — not the entire component. This is why Svelte doesn't need a virtual DOM: it knows the exact DOM nodes that need to change.

You can even use `$state` for objects and arrays, and it automatically makes them deeply reactive:

```typescript
let suggestions = $state<Suggestion[]>([]);

// This triggers a re-render:
suggestions = [...suggestions, newSuggestion];

// So does this (mutating the array):
suggestions.push(newSuggestion);
```

Svelte 5 uses JavaScript Proxies to make arrays and objects deeply reactive. You don't need to learn special mutation methods (`push` instead of `concat`, `splice` instead of `filter`) like in Vue or old Svelte stores.

### `$state` for Primitive vs. Object Values

When `$state` wraps a primitive value (string, number, boolean), Svelte creates a reactive signal. When you read `loading`, you get the current value. When you write `loading = true`, subscribers are notified.

When `$state` wraps an object, it creates a Proxy that intercepts property access. This is why you can mutate objects directly:

```typescript
let user = $state({ name: 'Alice', age: 25 });

// This works and triggers reactivity:
user.name = 'Bob';

// This also works:
user = { ...user, name: 'Bob' };
```

Both patterns trigger reactivity. Use whichever is more natural for your use case.

### `$derived` — Computed Values

```typescript
let downloads = $derived.by(() => {
  if (!result) return [];
  const out: { label: string; type: string; href: string }[] = [];
  const add = (label: string, type: string, href?: string | null) => {
    if (href) out.push({ label, type, href });
  };
  add('EPUB', 'epub', result.epub_url);
  add('HTML', 'html', result.html_url);
  add('MOBI', 'mobi', result.mobi_url);
  add('PDF', 'pdf', result.pdf_url);
  return out;
});
```

`$derived` creates a value that automatically recalculates when its dependencies change. It's like a computed property — you never manually update it, you just read it. When `result` changes, `downloads` automatically recalculates.

There are two forms:

```typescript
// Simple expression:
let fullName = $derived(`${firstName} ${lastName}`);

// Complex computation (with a function body):
let downloads = $derived.by(() => {
  // ... multi-line logic
});
```

The simple form is for single expressions. The `.by()` form is for anything that needs multiple statements, conditionals, or helper functions.

### `$effect` — Side Effects

```typescript
let initialized = $state(false);

$effect(() => {
  if (data.q && !initialized) {
    initialized = true;
    queryInput = data.q;
    doSearch();
  }
});
```

`$effect` runs code whenever its reactive dependencies change. It's the replacement for `$:` reactive statements. The function you pass to `$effect` runs after the DOM updates, and Svelte automatically tracks which `$state` and `$derived` values you read inside it.

The key behaviors:

1. **Automatic dependency tracking** — Svelte records which reactive values you read inside the effect function. When any of those values change, the effect re-runs.
2. **Runs after DOM updates** — Effects run after the component has been rendered, so you can safely access DOM elements.
3. **Cleans up** — When the component is destroyed, Svelte calls the effect's cleanup function (if you return one).

The `initialized` guard is a common pattern — you want to run the effect only once (on mount), but Svelte's `$effect` re-runs whenever dependencies change. By setting `initialized = true` inside the effect, it won't run again because the condition `data.q && !initialized` becomes false.

Another pattern is using effects for URL synchronization:

```typescript
$effect(() => {
  // This runs whenever currentPage changes:
  const qs = new URLSearchParams();
  qs.set('page', String(currentPage));
  goto(`/search?${qs.toString()}`, { replaceState: true });
});
```

### `$props` — Component Inputs

```svelte
<script lang="ts">
  let { children } = $props();
  let { data } = $props();
</script>
```

`$props` replaces the old `export let` syntax for component props. It destructures the props object directly. In our layout, `children` is the special prop that represents the child route's content — Svelte injects it automatically.

The `{ children }` prop is how SvelteKit layouts work. The layout renders `{children}` in its template, and the child page's content appears there:

```svelte
<!-- +layout.svelte -->
<main class="container">
  {#if activeTab === 'download'}
    <DownloadTab />
  {:else if activeTab === 'recs'}
    <RecommendationsTab />
  {:else if activeTab === 'sugg'}
    <SuggestionsTab />
  {/if}
</main>
```

Wait — in our layout, we don't actually use `{children}`. Instead, we conditionally render the tab components directly. This is because the tabs are managed by state (not by URL routing). The `{children}` prop is available but unused in this architecture.

If we wanted the search page to appear inside the layout, we'd use `{children}` like this:

```svelte
<main class="container">
  {#if activeTab === 'download'}
    <DownloadTab />
  {:else if activeTab === 'recs'}
    <RecommendationsTab />
  {:else if activeTab === 'sugg'}
    <SuggestionsTab />
  {:else}
    {children}
  {/if}
</main>
```

### `$derived.by` — Complex Derived Values

```typescript
let sorted = $derived.by(() => {
  return [...recs].sort(
    (a, b) => b.score + b.community_score * 0.1 - (a.score + a.community_score * 0.1),
  );
});
```

When the derived computation is more than a single expression, `$derived.by()` lets you write a function body. The result of that function becomes the derived value. The `[...recs]` spread is important — it creates a copy before sorting, so we don't mutate the original array (which would break Svelte's reactivity tracking).

### Summary of Runes

```mermaid
graph LR
    A["$state — reactive state"] --> B["$derived — computed values"]
    A --> C["$effect — side effects"]
    D["$props — component inputs"] --> A
    B --> C
```

These five runes — `$state`, `$derived`, `$derived.by`, `$effect`, and `$props` — are all we need. No stores, no context, no middleware. Svelte 5's runes are genuinely that powerful. The entire component state model is five keywords.

### A Note on Reactivity

Svelte's reactivity model is fundamentally different from React's. In React, changing state triggers a re-render of the entire component (and potentially child components). In Svelte, changing state triggers targeted DOM updates — only the specific text nodes, attributes, or elements that depend on the changed value are updated.

This means Svelte components are inherently more efficient. There's no reconciliation, no virtual DOM diffing, no shouldComponentUpdate. The compiler knows exactly which DOM nodes depend on which state variables, and it updates only what's necessary.

For our FicHub frontend, this means:

- Typing in the search input only updates the input value — not the entire page
- Clicking a vote button only updates the score display and button styles
- Loading results only updates the results area — the top bar and tabs stay untouched

This fine-grained reactivity is why Svelte feels fast even without optimization.

## The Vite Dev Server

One more configuration piece worth understanding is the Vite config:

```typescript
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  resolve: {
    conditions: ['browser'],
  },
  server: {
    port: 5173,
    host: '0.0.0.0',
    proxy: {
      '/api': {
        target: 'http://localhost:8004',
        changeOrigin: true,
      },
    },
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['src/test-setup.ts'],
    include: ['src/**/*.test.{ts,svelte}'],
  },
});
```

The `proxy` section is clever: during development, when the SvelteKit dev server (running on port 5173) receives a request to `/api/v0/*`, it forwards it to the Rust backend (running on port 8004). This means during development, you just run `cargo run` and `npm run dev` — no need for nginx or CORS configuration.

The `changeOrigin: true` option modifies the `Host` header of the proxied request to match the target, which is important for virtual hosting setups.

The `test` section configures Vitest for unit testing:

- `environment: 'jsdom'` — simulates a browser DOM for testing Svelte components
- `globals: true` — makes `describe`, `it`, `expect` available globally
- `include: ['src/**/*.test.{ts,svelte}']` — test files live next to the code they test

In production, the Rust backend serves both the static files AND handles API requests, so there's no need for a proxy.

### The Proxy in Detail

The Vite proxy works by intercepting HTTP requests at the dev server level. When your JavaScript code calls `fetch('/api/v0/epub?q=...')`, the browser sends the request to `localhost:5173` (the Vite dev server). The proxy sees the `/api` prefix and forwards the request to `localhost:8004` (the Rust backend), then returns the response to the browser.

This is transparent to your JavaScript code — it doesn't know (or care) that the request was proxied. The code just calls `fetch('/api/v0/...')` and gets a response.

The `changeOrigin: true` option rewrites the `Host` header to match the target server. Without it, the Rust backend would see `Host: localhost:5173` instead of `Host: localhost:8004`, which could cause issues with virtual hosting or CORS.

### Package.json Scripts

```json
{
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview",
    "test": "vitest run",
    "test:watch": "vitest"
  }
}
```

- `npm run dev` — starts the development server with hot reload
- `npm run build` — creates the static output in `build/`
- `npm run preview` — serves the built output locally (for testing before deployment)
- `npm run test` — runs all tests once
- `npm run test:watch` — runs tests in watch mode (re-runs on file changes)

The dev dependencies include the testing stack: Vitest, Testing Library for Svelte, jest-dom matchers, and jsdom for DOM simulation. This setup gives us component testing capabilities similar to React Testing Library.

### Dependencies: Minimal by Design

Notice what's NOT in the dependencies: no state management library (Redux, Zustand, Pinia), no UI component library (Material UI, Bootstrap, Tailwind), no form library (Formik, React Hook Form), no animation library (Framer Motion), no HTTP client (Axios).

Svelte's built-in features handle all of these:

- **State management** — `$state`, `$derived`, `$effect`
- **UI components** — hand-written with scoped CSS
- **Forms** — `bind:value` with HTML validation
- **Animations** — CSS transitions and `@keyframes`
- **HTTP** — the `fetch` API (no wrapper needed)

The entire dependency list is just the build tools: SvelteKit, Vite, TypeScript, and the testing stack. This keeps the project simple, the bundle small, and the attack surface minimal.

The total dependency count is around 15 packages (including transitive dependencies). For comparison, a typical React project with state management, routing, UI library, and form handling might have 50-100+ dependencies. Fewer dependencies means fewer potential security vulnerabilities, faster installs, and less maintenance burden.

🧪 **Try It Yourself: Build the Static Output**

Run `npm run build` in the frontend directory and inspect the `build/` folder. You'll see a single `index.html` and a handful of JS/CSS files. The total size? Usually under 100KB of JavaScript — dramatically smaller than a typical React or Vue bundle. Open `build/index.html` in a browser with the Rust backend running — it should work identically to `npm run dev`.

⚠️ **Watch Out: Don't Accidentally Server-Render**

If you remove `export const ssr = false` from `+layout.ts`, the pages will try to server-render, and since we have no SSR setup (no database connections, no API calls during SSR), you'll get hydration mismatches or "window is not defined" errors. Always keep that line unless you have a specific reason to enable SSR for a route.

⚠️ **Watch Out: The Proxy Is Development-Only**

The Vite proxy in `vite.config.ts` only works during `npm run dev`. The built output (from `npm run build`) doesn't include the proxy — it's just static files. In production, nginx or the Rust backend handles the `/api` routing. Don't confuse development behavior with production behavior.

---

# Chapter 30: The API Client and TypeScript Types

## Why an API Client?

Picture this: you're building a Svelte component that needs to download a fanfiction. You could write this directly in the component:

```svelte
<script>
  async function handleDownload() {
    const res = await fetch(`/api/v0/epub?q=${encodeURIComponent(url)}`);
    const data = await res.json();
    // ... use data somehow
  }
</script>
```

But this has problems:

1. **No type safety.** What does `data` look like? What properties does it have? TypeScript can't help you. You'll find out about missing properties at runtime — in the browser console, in front of users.

2. **Repeated error handling.** Every component needs to handle network errors, API errors, and JSON parsing errors the same way. Copy-pasting error handling across five components is a bug factory.

3. **Hardcoded URLs.** If the API path changes from `/api/v0/epub` to `/api/v1/epub`, you'd need to update every component. Miss one, and that feature silently breaks.

4. **No reusability.** If two components need the same API call, you'd duplicate the fetch logic in both.

An API client solves all four problems. It's a centralized module that:

- Defines TypeScript types for every API response
- Handles HTTP requests, errors, and JSON parsing
- Provides named functions for each API endpoint
- Returns fully typed data

Think of it as a translator between the UI and the server. The UI says "I want to download this fic," and the API client says "I'll handle the HTTP details, error handling, and type conversion. Here's your typed response."

## types.ts: The Type Dictionary

Let's walk through `src/lib/api/types.ts` — the file that defines the shape of every API response. This is the contract between the frontend and the Rust backend. Every type here corresponds to a struct or enum in the Rust code.

### ExportUrls

```typescript
/** A single export's download URLs (keys: epub, html, mobi, pdf). */
export interface ExportUrls {
  epub?: string;
  html?: string;
  mobi?: string;
  pdf?: string;
}
```

Each property is optional (`?`) because not every fic is available in every format. The Rust backend only returns URLs for formats it successfully generated. If the EPUB conversion failed but HTML succeeded, you'd get `{ html: "/cache/html/123?h=abc" }` — no `epub` property at all.

### FicMeta

```typescript
/** Fic metadata object returned by /api/v0/epub and /api/v0/meta. */
export interface FicMeta {
  id: string;              // The URL ID (site-specific)
  work_id?: number;        // Canonical work ID (unified works model)
  title: string;
  author: string;
  chapters: number;
  words: number;
  description: string;
  status: string;
  source: string;
  created: string;
  updated: string;
  extra_meta: unknown | null;
  raw_extended_meta: unknown | null;
  author_url: string;
  author_local_id: string;
  source_id: number;
  author_id: number;
}
```

This mirrors the `FicMeta` struct in our Rust code. The TypeScript interface ensures that whenever the API returns metadata, every component knows exactly what fields are available.

Let's look at each field:

- `id` — the URL ID (a hash of the source URL, used as a unique identifier)
- `title`, `author` — basic fic info scraped from the source site
- `chapters`, `words` — numeric stats
- `description` — the fic's summary, may contain HTML
- `status` — "Complete", "In Progress", "Abandoned", etc.
- `source` — the original URL or source identifier
- `created`, `updated` — ISO 8601 timestamps
- `extra_meta`, `raw_extended_meta` — site-specific metadata (e.g., kudos, hits, bookmarks for AO3 fics). These are `unknown | null` because they can be anything depending on the source site.
- `author_url`, `author_local_id` — author identification
- `source_id`, `author_id` — numeric IDs for database relationships

The `unknown | null` types for the metadata fields reflect that these can be anything (or nothing) depending on the source site. AO3 fics might have `{ kudos: 1500, hits: 50000 }`, while FanFiction.net fics might have `{ favorites: 200, alerts: 100 }`.

### ExportResponse

```typescript
/** Response from GET /api/v0/epub?q=<url> and GET /api/v0/meta?q=<url>. */
export interface ExportResponse {
  err: number;
  q?: string;
  msg?: string;
  fixits?: unknown[];
  info?: string;
  url_id?: string;
  slug?: string;
  meta?: FicMeta;
  hashes?: Record<string, string>;
  urls?: ExportUrls;
  epub_url?: string | null;
  html_url?: string | null;
  mobi_url?: string | null;
  pdf_url?: string | null;
  notes?: string[];
}
```

This is the main response type for our primary feature — downloading fics. Let's break down the fields:

- `err` — numeric error code. 0 means success. Non-zero means something went wrong.
- `q` — the original query (URL) echoed back
- `msg` — error message (when `err !== 0`)
- `fixits` — suggestions for fixing broken URLs (when the backend can detect what went wrong)
- `info` — informational message
- `url_id` — the resolved URL ID
- `slug` — URL-safe slug for the fic
- `meta` — the fic metadata (when successful)
- `hashes` — content hashes for cache-busting
- `urls` — download URLs object
- `epub_url`, `html_url`, `mobi_url`, `pdf_url` — convenience URLs for direct download
- `notes` — array of notes (e.g., "This fic has been updated since your last download")

The individual `epub_url`, `html_url` etc. fields at the top level are convenience URLs for direct download. The `urls` object and `hashes` are for the cache route (which we built in Part 5).

### Recommendation Types

```typescript
/** A single recommendation returned by GET /api/v0/recommendations. */
export interface RecResult {
  url_id: string;
  title: string;
  author: string;
  words: number;
  chapters: number;
  status: string;
  site_domain: string;
  summary: string;
  score: number;
  community_score: number;
  download_urls: Record<string, string>;
}
```

Each recommendation includes both an algorithmic `score` (from the collaborative filtering engine we built in Part 4) and a `community_score` (from user votes). The `download_urls` field lets users directly download any recommended fic without leaving the recommendations page.

```typescript
/** Response from GET /api/v0/recommendations. */
export interface RecommendationsResponse {
  err: number;
  url_id: string;
  site_domain?: string | null;
  recommendations: RecResult[];
  generated_at: string;
}
```

The `generated_at` timestamp tells the frontend when the recommendations were computed. This is useful for showing "Recommendations generated 2 hours ago" — letting users know the data might be stale.

### Suggestion and Vote Types

```typescript
/** A community suggestion returned by GET /api/v0/recommendations/votes. */
export interface Suggestion {
  id: number;
  suggested_url_id: string;
  comment: string | null;
  net_votes: number;
  created: string | null;
}
```

Each suggestion is a user-submitted recommendation with a URL and optional comment. The `net_votes` is the current vote count (upvotes minus downvotes).

```typescript
/** Response from POST /api/v0/recommendations/suggest. */
export interface SuggestResponse {
  err: number;
  suggestion_id?: number;
  msg?: string;
}
```

```typescript
/** Response from POST /api/v0/recommendations/vote. */
export interface VoteResponse {
  err: number;
  new_score: number;
}
```

The `VoteResponse` returns the updated score after a vote. This lets the UI update the displayed score without re-fetching all suggestions.

Every type maps directly to what the Rust backend returns. This 1:1 correspondence is intentional — it makes debugging easy. If the TypeScript type says something exists but it's `undefined` in the browser, the backend probably isn't returning it. Check the Rust handler's `serde::Serialize` derive.

## client.ts: The HTTP Layer

Now let's look at the actual API client functions that make requests to the Rust backend.

### The ApiError Class

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
```

The `ApiError` class extends the standard `Error` with HTTP-specific details. This lets us catch errors and still know what happened:

```typescript
try {
  await fetchExport(url);
} catch (e) {
  if (e instanceof ApiError) {
    // We know the status code and response body
    error = `Server returned ${e.status}. Is the backend running?`;
  } else {
    error = 'Network error.';
  }
}
```

Setting `this.name = 'ApiError'` ensures the error name in stack traces shows `ApiError` instead of `Error`, making debugging easier.

### The request() Helper

```typescript
async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, init);
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new ApiError(res.status, text);
  }
  return (await res.json()) as T;
}
```

This is the foundation of every API call. It's beautifully simple:

1. **Prepend the base URL** — `BASE` is `/api/v0`, so `request('/epub?q=...')` fetches `/api/v0/epub?q=...`
2. **Make the fetch request** — pass through any additional options (method, headers, body)
3. **Check for errors** — `res.ok` is true for status 200-299. If not, read the error text and throw an `ApiError`
4. **Parse and return** — `res.json()` parses the JSON body, cast to type `T`

The `<T>` generic parameter is what gives us type safety. When we call `request<ExportResponse>('/epub?q=...')`, TypeScript knows the return type is `ExportResponse`. Every function that uses `request()` inherits this type safety.

The `.catch(() => '')` after `res.text()` handles the edge case where the server returns a non-JSON error response that can't be read as text (e.g., a binary error page).

### The buildQuery() Helper

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
```

It filters out `undefined`, `null`, and empty string values — so we only include parameters that are actually set. This keeps URLs clean.

For example:

```typescript
buildQuery({ q: 'hello', n: 20, url_id: undefined })
// Returns: "?q=hello&n=20"
// Note: url_id is excluded because it's undefined
```

The `encodeURIComponent` ensures special characters in values (like `&`, `=`, `?`) are properly escaped.

### Why Not Use URLSearchParams?

We could use `URLSearchParams` for building query strings (like `buildSearchQuery()` does). But `buildQuery()` in `client.ts` was written earlier and follows a different pattern — it takes a plain object and returns a string. `URLSearchParams` is more modern and handles edge cases better, but both approaches work. The inconsistency between `client.ts` and `search.ts` is a minor technical debt — in a perfect world, we'd use `URLSearchParams` everywhere.

### fetchExport()

```typescript
/** GET /api/v0/epub?q=<url> — export a fic and return download URLs. */
export async function fetchExport(url: string): Promise<ExportResponse> {
  return request<ExportResponse>(`/epub${buildQuery({ q: url })}`);
}
```

One line. The entire download flow is: take a URL, build a query string, make a GET request, return typed data. All the complexity (scraping, EPUB generation, caching) happens on the Rust backend.

The function signature tells you everything: it takes a string (the fic URL), and returns a Promise that resolves to an `ExportResponse`. No ambiguity about what goes in or what comes out.

### Error Handling Philosophy

Notice the error handling pattern throughout the API client: we don't catch errors inside the client functions. The `request()` helper throws `ApiError` for HTTP errors, and individual functions let those errors propagate to the caller.

This is intentional. The API client is a "thin" layer — it handles HTTP mechanics (headers, status codes, JSON parsing) but leaves error handling to the UI components. Each component can decide how to display errors based on context:

- The download tab shows a red card with supported sites
- The recommendations tab shows a simpler error message
- The suggestions tab shows errors inline in the modal

If the API client caught errors internally, it would need to know about UI concerns (what message to show, where to display it). By letting errors propagate, we keep the client focused on its job: making HTTP requests and returning typed data.

The only exception is the `ApiError` class — it wraps HTTP errors with structured data (status code, response body) so callers can make informed decisions about how to handle them.

### fetchMeta()

```typescript
/** GET /api/v0/meta?q=<url> — fetch fic metadata without downloading. */
export async function fetchMeta(url: string): Promise<ExportResponse> {
  return request<ExportResponse>(`/meta${buildQuery({ q: url })}`);
}
```

Same shape as `fetchExport()` but hits the `/meta` endpoint. This is for getting metadata without actually generating download files — useful for previews or quick lookups. Note that it returns `ExportResponse` (not a separate `MetaResponse`) because the backend uses the same response type for both endpoints.

### fetchRecommendations()

```typescript
/** GET /api/v0/recommendations?q=<url>&n=<n> — get recommendations for a fic. */
export async function fetchRecommendations(
  q?: string,
  url_id?: string,
  n = 20,
): Promise<RecommendationsResponse> {
  return request<RecommendationsResponse>(
    `/recommendations${buildQuery({ q, url_id, n })}`,
  );
}
```

This is interesting because it accepts either a URL (`q`) or a pre-resolved `url_id`. The backend can resolve either one. If you already know the `url_id` (from a previous call), you can pass it directly to skip the URL resolution step.

The `n` parameter defaults to 20 — the number of recommendations to return. The backend's collaborative filtering engine computes scores for all known fics and returns the top `n`.

### fetchVotes()

```typescript
/** GET /api/v0/recommendations/votes?url_id=<id> — list community votes. */
export async function fetchVotes(url_id: string): Promise<VotesResponse> {
  return request<VotesResponse>(`/recommendations/votes${buildQuery({ url_id })}`);
}
```

Fetches community suggestions for a given fic (identified by `url_id`). Returns an array of `Suggestion` objects, each with its current vote count.

### submitSuggestion()

```typescript
/** POST /api/v0/recommendations/suggest — submit a recommendation suggestion. */
export async function submitSuggestion(
  url_id: string,
  suggested_url: string,
  comment?: string,
): Promise<SuggestResponse> {
  return request<SuggestResponse>('/recommendations/suggest', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ url_id, suggested_url, comment }),
  });
}
```

This is the first POST request in our client. The rest have been GETs. It sends a JSON body with the suggestion details. The `comment` is optional — users can suggest a fic without explaining why.

The `request()` helper passes through the `init` parameter (method, headers, body) to `fetch()`, making it flexible enough for both GET and POST requests.

### castVote()

```typescript
/** POST /api/v0/recommendations/vote — upvote (1) or downvote (-1) a suggestion. */
export async function castVote(suggestion_id: number, vote: 1 | -1): Promise<VoteResponse> {
  return request<VoteResponse>('/recommendations/vote', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ suggestion_id, vote }),
  });
}
```

The `vote` parameter is typed as `1 | -1` — a union type that prevents bugs like accidentally passing `0` or `2`. TypeScript enforces this at compile time. If you try `castVote(42, 0)`, the compiler will flag an error.

The response includes `new_score` — the updated vote count after the vote is applied. This lets the UI update the displayed score immediately.

### Exporting the API

```typescript
export { ApiError };
```

The `ApiError` class is exported so components can use `instanceof` checks in catch blocks. The individual functions (`fetchExport`, `fetchMeta`, etc.) are exported where they're defined.

### Social API Types

The social features use `work_id` (integer) to identify works:

```typescript
export interface Bookmark {
    work_id: number;
    created_at: string;
}

export interface WorkRating {
    work_id: number;
    rating: number;
    created_at: string;
}

export interface WorkComment {
    id: number;
    work_id: number;
    user_id: number;
    content: string;
    created_at: string;
}
```

The social API client methods use `work_id` instead of `url_id`:

```typescript
export async function addBookmark(workId: number): Promise<Bookmark> {
    const response = await apiFetch('/api/bookmarks', {
        method: 'POST',
        body: JSON.stringify({ work_id: workId }),
    });
    return response.json();
}

export async function removeBookmark(workId: number): Promise<void> {
    await apiFetch(`/api/bookmarks/${workId}`, { method: 'DELETE' });
}

export async function rateWork(workId: number, rating: number): Promise<WorkRating> {
    const response = await apiFetch('/api/ratings', {
        method: 'POST',
        body: JSON.stringify({ work_id: workId, rating }),
    });
    return response.json();
}
```

## search.ts: The Search API

The search module is more complex because it handles multiple filter types and includes constants for dropdown options.

### The Import Structure

Looking at the imports across the codebase, there's a clear pattern:

```typescript
// In components:
import { fetchExport, ApiError } from '$lib/api/client';
import type { ExportResponse } from '$lib/api/types';
import { formatWords, detectSite, stripHtml } from '$lib/util';

// In the search page:
import { search } from '$lib/api/search';
import type { SearchFilters, SearchResult, SearchResponse } from '$lib/api/search';
import { SORT_OPTIONS, COMPLETE_OPTIONS, SOURCE_OPTIONS, defaultFilters } from '$lib/api/search';
import { parseSearchQuery } from '$lib/search/syntax';
```

The pattern is: API functions from `client.ts`, types imported as `type` (type-only imports erased at compile time), constants from the module that defines them, and utility functions from `util.ts`.

Type-only imports (`import type { ... }`) are important for bundle size. They tell the compiler "I only need this for type checking — don't include it in the JavaScript output." Without `type`, the import might cause the module to be included in the bundle even if only the type is used.

### SearchFilters

```typescript
export interface SearchFilters {
  q: string;
  include_tags: string;
  exclude_tags: string;
  include_any_tags: string;
  min_words: number | null;
  max_words: number | null;
  min_chapters: number | null;
  max_chapters: number | null;
  complete: boolean | null;
  source: string;
  date_from: string;
  date_to: string;
  sort: string;
  page: number;
  per_page: number;
}
```

This covers every filter the search API supports. The `null` values mean "no filter applied" — different from an empty string, which might mean something different depending on the field.

### How Filters Map to URL Parameters

Each filter corresponds to a URL parameter that the Rust backend accepts:

```
q              → ?q=fandom:Harry+Potter
include_tags   → ?include_tags=1:Harry+Potter,4:Fluff
exclude_tags   → ?exclude_tags=1:Naruto
min_words      → ?min_words=10000
max_words      → ?max_words=50000
complete       → ?complete=true
source         → ?source=archiveofourown.org
date_from      → ?date_from=2024-01-01T00:00:00Z
sort           → ?sort=updated
page           → ?page=2
```

The `buildSearchQuery()` function handles this mapping — it's the bridge between the typed filter object and the URL string the backend expects.

### The Tag Format

Tags use `typeId:name` format separated by commas, mapping directly to our database:

```
1:Harry Potter  → Fandom (type_id = 1)
4:Fluff         → Freeform (type_id = 4)
1:Harry Potter,4:Fluff  → Multiple tags
```

For example, `source: ''` means "all sites" (no filter), while `source: 'archiveofourown.org'` means "AO3 only." The `null` values for numeric fields (`min_words`, etc.) mean the backend applies no constraint.

### buildSearchQuery()

```typescript
export function buildSearchQuery(filters: SearchFilters): string {
  const params = new URLSearchParams();

  if (filters.q) params.set('q', filters.q);
  if (filters.include_tags) params.set('include_tags', filters.include_tags);
  if (filters.exclude_tags) params.set('exclude_tags', filters.exclude_tags);
  if (filters.include_any_tags) params.set('include_any_tags', filters.include_any_tags);
  if (filters.min_words !== null) params.set('min_words', String(filters.min_words));
  if (filters.max_words !== null) params.set('max_words', String(filters.max_words));
  if (filters.min_chapters !== null) params.set('min_chapters', String(filters.min_chapters));
  if (filters.max_chapters !== null) params.set('max_chapters', String(filters.max_chapters));
  if (filters.complete !== null) params.set('complete', String(filters.complete));
  if (filters.source) params.set('source', filters.source);
  if (filters.date_from) params.set('date_from', filters.date_from);
  if (filters.date_to) params.set('date_to', filters.date_to);
  if (filters.sort) params.set('sort', filters.sort);
  if (filters.page > 1) params.set('page', String(filters.page));
  if (filters.per_page !== 20) params.set('per_page', String(filters.per_page));

  return params.toString();
}
```

Two smart defaults: `page` is only included if it's greater than 1, and `per_page` is only included if it's not the default of 20. This keeps the URL clean for the most common case (first page, default results per page).

The `URLSearchParams` class handles URL encoding automatically, which is why we don't need manual `encodeURIComponent` calls here.

### The search() Function

```typescript
export async function search(filters: SearchFilters): Promise<SearchResponse> {
  const qs = buildSearchQuery(filters);
  const res = await fetch(`/api/v0/search${qs ? '?' + qs : ''}`);
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`Search failed (${res.status}): ${text}`);
  }
  return (await res.json()) as SearchResponse;
}
```

Note that this function doesn't use the `request()` helper from `client.ts`. It uses raw `fetch` directly. This is because the search module was added later and follows slightly different patterns. In a perfect world, we'd unify them — but in a real project, getting things working often takes priority over architectural purity.

The `SearchResponse` type:

### Why the Search Module Is Separate

The search functionality could have been part of `client.ts`, but it's separated for a reason: the search system has its own types, constants, and helper functions that are specific to search. Putting them in `client.ts` would make that file much larger and harder to navigate.

By separating search into its own module, we get:

1. **Focused imports** — `import { search, SearchFilters } from '$lib/api/search'` instead of a long import from a monolithic client
2. **Independent evolution** — the search API might change format (different filters, pagination) without affecting the core client
3. **Testability** — the search module can be tested independently
4. **Discoverability** — searching for "search" in the file system finds the right module immediately

This is the "single responsibility principle" applied to file organization: each module does one thing well.

```typescript
export interface SearchResponse {
  total: number;
  page: number;
  per_page: number;
  results: SearchResult[];
}
```

Each `SearchResult` includes rich data:

```typescript
export interface SearchResult {
  url_id: string;
  title: string;
  author: string;
  source: string;
  words: number;
  chapters: number;
  status: string;
  description: string;
  updated: string | null;
  rank: number | null;
  tags: SearchTag[];
  total_freeform: number;
}
```

The `tags` array contains the fic's tags with their types and relevance scores:

```typescript
export interface SearchTag {
  name: string;
  type: string;
  type_id: number;
  score: number;
}
```

### Sort and Filter Constants

```typescript
export const TAG_TYPES = {
  1: 'Fandom',
  2: 'Character',
  3: 'Relationship',
  4: 'Freeform',
} as const;

export const SORT_OPTIONS = [
  { value: '', label: 'Relevance' },
  { value: 'updated', label: 'Date Updated' },
  { value: 'created', label: 'Date Published' },
  { value: 'words', label: 'Word Count' },
  { value: 'kudos', label: 'Kudos' },
] as const;

export const COMPLETE_OPTIONS = [
  { value: '', label: 'All Works' },
  { value: 'true', label: 'Complete Only' },
  { value: 'false', label: 'In Progress Only' },
] as const;

export const SOURCE_OPTIONS = [
  { value: '', label: 'All Sites' },
  { value: 'archiveofourown.org', label: 'Archive of Our Own' },
  { value: 'fanfiction.net', label: 'FanFiction.net' },
  { value: 'fictionpress.com', label: 'FictionPress' },
  { value: 'forums.spacebattles.com', label: 'SpaceBattles' },
  { value: 'forums.sufficientvelocity.com', label: 'SufficientVelocity' },
] as const;
```

These are used by the dropdown selects on the search page. The `as const` assertion tells TypeScript these arrays are readonly and their types are the exact literal values — not just `string`. This means TypeScript knows that `SORT_OPTIONS[0].value` is `''` (the empty string literal), not just `string`.

The `TAG_TYPES` constant maps numeric type IDs (from the database) to human-readable names. This is used in the search results to show tag types on hover.

### defaultFilters()

```typescript
export function defaultFilters(): SearchFilters {
  return {
    q: '',
    include_tags: '',
    exclude_tags: '',
    include_any_tags: '',
    min_words: null,
    max_words: null,
    min_chapters: null,
    max_chapters: null,
    complete: null,
    source: '',
    date_from: '',
    date_to: '',
    sort: '',
    page: 1,
    per_page: 20,
  };
}
```

A factory function that creates a fresh `SearchFilters` object with all fields at their default (empty/null) values. This is used to reset filters and as the initial state for the search page.

## util.ts: Formatting Helpers

The utility module contains small functions shared across components. Each one is pure (no side effects) and focused on a single formatting task.

### formatWords()

```typescript
export function formatWords(words: number): string {
  return words.toLocaleString('en-US');
}
// formatWords(1234567) → "1,234,567"
// formatWords(42) → "42"
// formatWords(1000000) → "1,000,000"
```

Converts large numbers into readable strings with commas. This is used everywhere — in search results, recommendation cards, and the download tab. "1,234,567 words" is much more readable than "1234567 words."

### detectSite()

```typescript
export function detectSite(url: string): string {
  if (url.includes('archiveofourown.org')) return 'AO3';
  if (url.includes('fanfiction.net')) return 'FanFiction.net';
  if (url.includes('fictionpress.com')) return 'FictionPress';
  if (url.includes('xenforo') || url.includes('spacebattles') || url.includes('sufficientvelocity'))
    return 'Forum';
  return 'Unknown';
}
```

Detects the fanfiction site from a URL. The function checks for known domain patterns and returns a human-readable name. The "Forum" category covers XenForo-based sites like SpaceBattles and SufficientVelocity — they share the same underlying software.

The detection is simple string matching, not URL parsing. It's fast and works for all our supported sites. If a URL doesn't match any known pattern, it returns "Unknown."

### stripHtml()

```typescript
export function stripHtml(html: string): string {
  if (!html) return '';
  return html
    .replace(/<br\s*\/?>/gi, ' ')        // <br> → space
    .replace(/<\/(p|div)>/gi, ' ')        // </p> </div> → space
    .replace(/<[^>]+>/g, '')              // all other tags → nothing
    .replace(/&amp;/g, '&')              // HTML entities
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/\s+/g, ' ')               // collapse whitespace
    .trim();
}
```

Fanfiction summaries come from the source sites with HTML markup — paragraphs, line breaks, italics, links. We need plain text for display in our UI. The regex chain handles:

1. **Line breaks** — `<br>` becomes a space (preserving word separation)
2. **Block elements** — `</p>` and `</div>` become spaces
3. **All other tags** — stripped entirely
4. **HTML entities** — converted to their actual characters
5. **Whitespace** — collapsed to single spaces
6. **Leading/trailing whitespace** — trimmed

This is a pragmatic approach. It doesn't handle every HTML edge case (like nested tags or malformed HTML), but it works well for fanfiction summaries. A more robust solution would use a DOM parser like `DOMParser`, but that would be overkill for this use case.

### Why Not Use a DOM Parser?

We could use `new DOMParser().parseFromString(html, 'text/html').textContent` to strip HTML. This would handle every edge case correctly. But it's slower (creates a DOM tree), requires a browser environment (won't work in Node.js tests), and adds complexity.

For fanfiction summaries, the regex approach works because:
- Summaries are short (usually under 1000 characters)
- They use simple HTML (paragraphs, line breaks, bold, italic)
- They don't contain complex structures (tables, forms, embedded content)

The regex chain handles the 95% case correctly. The remaining 5% (malformed HTML, unusual tags) might produce slightly messy text, but it's still readable.

### relativeTime()

```typescript
export function relativeTime(iso: string): string {
  if (!iso) return '';
  const then = new Date(iso).getTime();
  if (isNaN(then)) return '';
  const diff = Date.now() - then;
  const sec = Math.floor(diff / 1000);
  if (sec < 60) return 'less than a minute ago';
  const min = Math.floor(sec / 60);
  if (min < 60) return `${min} minute${min > 1 ? 's' : ''} ago`;
  const hr = Math.floor(min / 60);
  if (hr < 24) return `${hr} hour${hr > 1 ? 's' : ''} ago`;
  const day = Math.floor(hr / 24);
  return `${day} day${day > 1 ? 's' : ''} ago`;
}
```

Converts ISO timestamps like `"2024-06-15T10:30:00Z"` into friendly strings like "3 days ago." The function handles edge cases:

- Empty string → returns empty string
- Invalid date → returns empty string
- Very recent → "less than a minute ago"
- Singular/plural → "1 minute" vs "2 minutes"

The granularity stops at days — for older fics, "30 days ago" is fine. We don't need "1 month ago" or "1 year ago" for this use case.

### Why Not Use Intl.RelativeTimeFormat?

JavaScript has a built-in `Intl.RelativeTimeFormat` that handles relative time formatting with proper pluralization and localization. We could use it:

```javascript
const rtf = new Intl.RelativeTimeFormat('en', { numeric: 'auto' });
rtf.format(-3, 'day'); // "3 days ago"
```

But we didn't, for two reasons:

1. **Bundle size** — the custom function is 15 lines. `Intl.RelativeTimeFormat` is a large API that might not be tree-shaken.
2. **Control** — our function gives us exact control over the output format. We can stop at days, skip hours, or add custom formatting.

For a fanfiction app serving English-speaking users, the custom function is the right choice.

### cacheUrl()

```typescript
export function buildDownloadUrl(etype: string, urlId: string, hash: string): string {
  return `/cache/${etype}/${urlId}?h=${hash}`;
}
```

Builds URLs for our cache route — the one we set up in Part 5 to serve cached EPUB/HTML files with proper content types and cache headers. The `h` parameter is the content hash, used for cache validation.

### Why Cache URLs?

The cache route (`/cache/epub/{url_id}?h={hash}`) serves two purposes:

1. **Content-type headers** — the Rust backend sets `Content-Type: application/epub+zip` for EPUBs, `text/html` for HTML, etc. Without this, browsers might try to display EPUB files as text.

2. **Cache headers** — the `h` parameter is a content hash. If the hash matches, the backend returns `304 Not Modified` (no body). If the hash is stale, the backend regenerates and returns the new file with a fresh hash.

This means the first download is slow (generation), but subsequent downloads of the same fic are instant (cache hit). The hash ensures users always get the latest version.

## Type Safety in Practice

The beauty of all this typing shows up in the components. When a component calls `fetchExport(url)`, TypeScript knows the response is an `ExportResponse`. When it accesses `result.meta`, it knows that's a `FicMeta` with `title`, `author`, `words`, etc. If you try to access `result.meta.titleSize`, TypeScript will flag an error before you ever run the code.

This isn't just nice to have — it catches entire categories of bugs:

- **Typos in property names** — caught at compile time (`result.titel` instead of `result.title`)
- **Missing null checks** — caught at compile time (`result.meta.title` when `meta` is optional)
- **Wrong argument types** — caught at compile time (`fetchExport(42)` when it expects a string)
- **Wrong return types** — caught at compile time (assigning an `ExportResponse` to a `RecResult` variable)

In a project where the frontend and backend are developed separately (Rust in one language, TypeScript in another), having strong types on both sides is invaluable. If the backend changes its response shape, you update the TypeScript types, and every component that uses the wrong field gets a compile error.

The types also serve as documentation. When you open `types.ts`, you immediately understand what the API returns. No need to check the Rust code, no need to read the Axum handlers, no need to run the server and inspect responses. The types tell the story.

🧪 **Try It Yourself: Explore the Types**

Open `src/lib/api/types.ts` and try modifying a type — say, rename `epub_url` to `epubLink`. Save the file. The TypeScript compiler will immediately flag every component that references `result.epub_url` — showing you exactly where the old name is used. This is the kind of safety net that pays for itself instantly.

Try adding a new field to `FicMeta` — say, `rating: string`. Then go to `DownloadTab.svelte` and add `{m.rating}` to the template. TypeScript will happily accept it because the type already includes the field.

⚠️ **Watch Out: `unknown` vs `any`**

Notice the use of `unknown` instead of `any` in the types:

```typescript
extra_meta: unknown | null;
raw_extended_meta: unknown | null;
fixits?: unknown[];
```

TypeScript's `unknown` is the safe version of `any`. While `any` lets you do anything (access any property, call any method), `unknown` forces you to check the type before using it. This prevents accidental runtime errors — you can't accidentally call `.toString()` on an `unknown` value without first checking that it's actually a string.

Always prefer `unknown` when you genuinely don't know the shape of data. Reserve `any` for truly dynamic situations (like third-party library integrations with poor type definitions).

---

# Chapter 31: Download, Recommendations, Suggestions Tabs

## The Three Tabs of FicHub

The main interface is a tabbed layout. Three tabs, three features:

1. **Download** — paste a URL, get an EPUB
2. **Recommendations** — paste a URL, discover similar fics
3. **Suggestions** — see what the community recommends, vote on suggestions

Each tab is a self-contained Svelte component. They share the same layout (top bar, tab navigation) but operate independently. The layout switches between them based on which tab is active, destroying the previous tab's component and creating the new one.

This tab-based architecture means each component manages its own state, its own API calls, and its own error handling. There's no shared state between tabs (except the active tab index in the layout). This keeps things simple and avoids the complexity of global state management.

### Why Tabs Instead of Routes?

You might wonder why the tabs aren't separate routes (`/download`, `/recommendations`, `/suggestions`). The answer: tabs are faster. When you switch tabs, the previous tab's component is destroyed and the new one is created — but the layout (top bar, navigation) stays mounted. If these were routes, SvelteKit would unmount the layout and mount a new one, which is slower and loses state.

Tabs also preserve state more naturally. If you're on the Recommendations tab and switch to Download, the recommendations data is still in memory when you switch back. With routes, the component would be destroyed and recreated, losing all state.

The tradeoff: tabs don't have their own URLs. You can't bookmark `/recommendations` — you have to click the tab. For FicHub, this is fine — the primary feature (Download) is the default, and the other tabs are secondary.

If we wanted URL-synced tabs later, we could add URL parameters (`/?tab=recs`) without changing the component architecture.

Let's walk through each one in detail.

## DownloadTab.svelte: The Main Feature

This is the bread and butter of FicHub — the component users interact with most. It's the first thing they see (the default tab), and it's what most people come for: paste a URL, get an EPUB.

### The Script Block

```svelte
<script lang="ts">
  import { fetchExport, ApiError } from '$lib/api/client';
  import type { ExportResponse } from '$lib/api/types';
  import { formatWords, detectSite, stripHtml } from '$lib/util';

  let url = $state('');
  let loading = $state(false);
  let error = $state('');
  let result = $state<ExportResponse | null>(null);
```

Four reactive variables define the component's state:

- `url` — the user's input (bound to the text field). Starts empty, updates as the user types.
- `loading` — whether a request is in progress. When true, the button shows a spinner and the input is disabled.
- `error` — any error message to display. Empty string means no error.
- `result` — the API response. `null` until we get a successful response.

The imports bring in:
- `fetchExport` — the API function for downloading fics
- `ApiError` — the error class for catching HTTP errors
- `ExportResponse` — the type for the response
- `formatWords`, `detectSite`, `stripHtml` — formatting helpers

### The Complete Download Flow

Let's trace the entire flow from paste to EPUB:

1. **User pastes** `https://archiveofourown.org/works/12345`
2. **Presses Enter** → triggers `onKeydown` → calls `handleDownload()`
3. **Validation** → `url.trim()` is non-empty, so we proceed
4. **Loading state** → `loading = true`, `error = `, `result = null`
5. **API call** → `fetchExport(url.trim())` sends GET to `/api/v0/epub?q=https://archiveofourown.org/works/12345`
6. **Backend processes** → resolves URL, scrapes metadata, generates EPUB/HTML/MOBI/PDF, caches files
7. **Response arrives** → `{ err: 0, meta: {...}, epub_url: "/cache/epub/abc123?h=def456", ... }`
8. **Error check** → `res.err === 0`, so no error
9. **Store result** → `result = res`
10. **Derived updates** → `downloads` recalculates to `[{ label: "EPUB", href: "/cache/epub/abc123?h=def456" }, ...]`
11. **UI re-renders** → result card shows title, author, word count, download buttons
12. **Loading cleared** → `loading = false`, button re-enabled

The entire flow takes 2-5 seconds depending on the source site and fic length. The first download is slowest (scraping + generation); subsequent downloads of the same fic are instant (served from cache).

### The Download Handler

```typescript
  async function handleDownload() {
    if (!url.trim()) {
      error = 'Please paste a fanfiction URL first.';
      return;
    }
    loading = true;
    error = '';
    result = null;
    try {
      const res = await fetchExport(url.trim());
      if (res.err !== 0) {
        error = res.msg || 'Something went wrong with that URL.';
        return;
      }
      result = res;
    } catch (e) {
      if (e instanceof ApiError) {
        error = `Server returned ${e.status}. Is the backend running?`;
      } else {
        error = e instanceof Error ? e.message : 'Network error.';
      }
    } finally {
      loading = false;
    }
  }
```

This is the core flow:

1. **Validate** — reject empty URLs with a friendly message. The `.trim()` removes accidental whitespace.
2. **Set loading** — this disables the button and shows a spinner (prevents double-clicks).
3. **Clear previous state** — `error = ''` and `result = null` ensure we don't show stale data.
4. **Call the API** — `fetchExport()` sends the URL to the Rust backend.
5. **Check for errors** — the `err` field in the response tells us if something went wrong.
6. **Store result** — if successful, the `result` variable triggers the UI to show metadata and download links.
7. **Catch exceptions** — handle network errors and API errors differently (different user-facing messages).
8. **Clear loading** — in the `finally` block, always re-enable the button.

The error handling is multi-layered:

- **API-level errors** (`res.err !== 0`) — the backend understood the request but couldn't process the fic (unsupported site, broken URL, etc.). The `msg` field contains a user-friendly explanation.
- **HTTP errors** (`ApiError`) — the server returned a 4xx or 5xx status. This usually means the backend isn't running or there's a proxy issue.
- **Network errors** — the request didn't reach the server at all (no internet connection, DNS failure, etc.).
- **Other errors** — any unexpected JavaScript error. The `e instanceof Error` check ensures we can safely extract a message.

Each gets a different, user-friendly message. Users never see raw error objects or stack traces.

### The Enter Key Handler

```typescript
  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') handleDownload();
  }
```

A simple but important UX detail: pressing Enter in the input field triggers the download. Users expect this — it's a universal convention for search/download inputs. Without it, users would have to click the button every time, which is tedious.

The `KeyboardEvent` type is a standard Web API type — Svelte provides it automatically.

### The Derived Downloads List

```typescript
  let downloads = $derived.by(() => {
    if (!result) return [];
    const out: { label: string; type: string; href: string }[] = [];
    const add = (label: string, type: string, href?: string | null) => {
      if (href) out.push({ label, type, href });
    };
    add('EPUB', 'epub', result.epub_url);
    add('HTML', 'html', result.html_url);
    add('MOBI', 'mobi', result.mobi_url);
    add('PDF', 'pdf', result.pdf_url);
    return out;
  });
```

This `$derived.by()` creates a filtered list of available download formats. It only includes formats that the backend actually generated URLs for. If the fic wasn't available as MOBI (say, the conversion failed or the backend doesn't support MOBI for this site), that format simply won't appear in the list.

The `add()` helper function is a nice pattern — it avoids repeating the conditional check for each format:

```typescript
// Without the helper (verbose):
if (result.epub_url) out.push({ label: 'EPUB', type: 'epub', href: result.epub_url });
if (result.html_url) out.push({ label: 'HTML', type: 'html', href: result.html_url });
if (result.mobi_url) out.push({ label: 'MOBI', type: 'mobi', href: result.mobi_url });
if (result.pdf_url) out.push({ label: 'PDF', type: 'pdf', href: result.pdf_url });

// With the helper (clean):
add('EPUB', 'epub', result.epub_url);
add('HTML', 'html', result.html_url);
add('MOBI', 'mobi', result.mobi_url);
add('PDF', 'pdf', result.pdf_url);
```

The `add` function checks `if (href)` — if the URL is `undefined`, `null`, or empty string, the format is skipped. This handles all the edge cases (backend didn't generate EPUB, backend returned `null`, etc.).

### The Template: Search Row

```svelte
<div class="download-tab">
  <div class="search-row">
    <input
      type="url"
      placeholder="Paste a fanfiction URL (AO3, FanFiction.net, forums…)"
      bind:value={url}
      onkeydown={onKeydown}
      disabled={loading}
      aria-label="Fanfiction URL"
    />
    <button class="btn" onclick={handleDownload} disabled={loading}>
      {#if loading}
        <span class="spinner"></span> Working…
      {:else}
        Download
      {/if}
    </button>
  </div>
```

A few things to notice:

- **`type="url"`** — tells the browser this is a URL field (enables URL validation on mobile, shows a URL keyboard on touch devices)
- **`bind:value={url}`** — two-way binding. When the user types, `url` updates. When `url` changes programmatically, the input updates.
- **`disabled={loading}`** — prevents double-clicking while a request is in progress
- **`aria-label`** — accessibility: screen readers will announce this input's purpose
- **Conditional button content** — the `{#if loading}` block shows either a spinner or the "Download" text
- **`onclick` instead of `on:click`** — Svelte 5 syntax (replacing the older `on:click` directive)

The spinner is a pure CSS animation defined in `app.css`:

```css
.spinner {
  display: inline-block;
  width: 1rem;
  height: 1rem;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
```

No SVG, no icon library, no image file. Just a CSS border trick with a rotation animation. The `border-top-color: white` makes one edge of the circle white while the rest is semi-transparent, creating the classic spinning effect.

The `0.7s linear infinite` timing means it completes one full rotation every 0.7 seconds, with constant speed and no easing. This creates a smooth, mechanical-looking spinner.

### Why Pure CSS?

We could use an SVG spinner, a GIF animation, or a library like `lucide-svelte` for icons. But the CSS spinner is:
- **Tiny** — 10 lines of CSS, zero JavaScript
- **Customizable** — change `border-top-color` to match any theme
- **Performant** — GPU-accelerated CSS animation, no JavaScript frame calculation
- **Accessible** — paired with "Working..." text for screen readers

The CSS border trick is one of those web development gems that's been around for over a decade and still works perfectly.

### Error Display

```svelte
  {#if error}
    <div class="card error-card">
      <strong class="error-text">⚠️ {error}</strong>
      <p class="muted">
        Supported sites include Archive of Our Own, FanFiction.net, FictionPress,
        and XenForo forums (SpaceBattles, SufficientVelocity).
      </p>
    </div>
  {/if}
```

When an error occurs, we show a card with a red border and a helpful message. The `error-card` class adds `border-color: var(--color-error)` to make the error visually distinct from regular cards.

The paragraph below reminds users which sites are supported. This is a common UX pattern — when something fails, immediately tell the user what might be wrong and how to fix it.

The `{#if error}` conditional means this section only appears when there's an error. When `error` is an empty string, the entire block is not rendered (not just hidden — completely absent from the DOM).

### Result Display

```svelte
  {#if result && result.meta}
    {@const m = result.meta}
    <div class="card result-card">
      <h2>{m.title}</h2>
      <p class="muted">by {m.author} · <span class="tag">{detectSite(m.source)}</span></p>
      <p class="meta-line">
        {formatWords(m.words)} words · {m.chapters} chapters · status: {m.status}
      </p>
      {#if m.description}
        <p class="desc">{stripHtml(m.description).slice(0, 300)}</p>
      {/if}
```

The `{@const m = result.meta}` directive creates a local constant — a shorthand so we don't have to type `result.meta.title`, `result.meta.author`, etc. repeatedly. It's a Svelte-specific feature that keeps templates clean. The `m` variable is only available inside the `{#if result && result.meta}` block.

The result card shows:
- **Title** — as an `<h2>` heading, prominently displayed
- **Author and detected site** — "by AuthorName · AO3" with a badge-style site tag
- **Word count, chapters, and status** — formatted with `formatWords()` for readability
- **Description** — truncated to 300 characters, HTML-stripped, shown in muted color

### Download Buttons

```svelte
      {#if downloads.length > 0}
        <div class="downloads">
          {#each downloads as d}
            <a class="btn btn-secondary dl-btn" href={d.href} download>{d.label}</a>
          {/each}
        </div>
      {:else}
        <p class="muted">
          {result.notes?.[0] ?? 'This fic is not available for download right now.'}
        </p>
      {/if}
```

The `{#each downloads as d}` loop creates a button for each available format. The `<a>` tag with the `download` attribute tells the browser to download the linked file rather than navigating to it. The `btn-secondary` class gives it a subdued appearance (gray background instead of primary purple).

If no downloads are available, we show the first note from the API response, or a default message. The `?.[0]` is optional chaining — if `result.notes` is null/undefined, it won't throw an error. The `??` is the nullish coalescing operator — it uses the right side only if the left side is null/undefined.

```svelte
      {#if result.notes && result.notes.length > 0 && downloads.length > 0}
        <p class="note muted">{result.notes[0]}</p>
      {/if}
```

If there are notes AND downloads are available, we show the note below the download buttons. This handles cases like "This fic was last updated 3 months ago" — useful context without blocking the download.

### The Bookmarklet

```svelte
  <p class="muted hint">
    Tip: drag this to your bookmarks bar to download any fic from its page:
    <a
      class="bookmarklet"
      draggable="true"
      href="javascript:location.href='http://localhost:8004/api/v0/epub?q='+encodeURIComponent(location.href)"
    >FicHub ↗</a>
  </p>
```

This is a fun touch. The link contains a `javascript:` URL — when dragged to the bookmarks bar, it becomes a bookmarklet. Clicking it on any fanfiction page will redirect to FicHub's API with the current page URL, triggering a download.

The `draggable="true"` attribute enables the drag behavior. The `encodeURIComponent(location.href)` ensures the current page URL is properly encoded for the query parameter.

Note the `http://localhost:8004` hardcoded address — this is meant for local development. In production, you'd update this to point to your actual server URL.

### How the Bookmarklet Works

When the user drags the `FicHub ↗` link to their bookmarks bar, the browser creates a bookmark with the `javascript:` URL. When they click it on any page:

1. The browser executes the JavaScript: `location.href = 'http://localhost:8004/api/v0/epub?q=' + encodeURIComponent(location.href)`
2. This navigates the current tab to the FicHub API endpoint with the current page URL as the query
3. The Rust backend receives the URL, scrapes the fic, generates the EPUB, and returns the download
4. The user gets the EPUB download dialog

It's a one-click download from any fanfiction page. The `draggable="true"` attribute enables the drag-to-bookmarks-bar behavior in modern browsers. We could make this configurable, but for a personal tool, hardcoding is fine.

The `<!-- svelte-ignore a11y_invalid_attribute -->` comment suppresses a Svelte accessibility warning — `javascript:` URLs are technically invalid `href` values, but they're intentional for bookmarklets.

### Component Styles

```svelte
<style>
  .search-row {
    display: flex;
    gap: 0.6rem;
    margin-bottom: 1rem;
  }
  .search-row input {
    flex: 1;
  }
  .error-card {
    border-color: var(--color-error);
  }
  .result-card h2 {
    margin: 0 0 0.3rem;
    font-size: 1.4rem;
  }
  .downloads {
    display: flex;
    flex-wrap: wrap;
    gap: 0.6rem;
    margin-top: 1rem;
  }
  .dl-btn {
    text-decoration: none;
  }
  @media (max-width: 600px) {
    .search-row {
      flex-direction: column;
    }
  }
</style>
```

The `<style>` block in a Svelte component is scoped by default — the CSS only applies to elements in this component. This means `.search-row` in `DownloadTab.svelte` doesn't affect `.search-row` in `RecommendationsTab.svelte`, even though they have the same class name.

Svelte achieves this by adding a unique attribute (like `svelte-abc123`) to each element in the component, and scoping the CSS selectors to match only elements with that attribute. So `.search-row` becomes `.search-row.svelte-abc123` — it only matches elements in that specific component.

This is different from CSS Modules (which hash class names) or Shadow DOM (which isolates entire subtrees). Svelte's approach is lightweight — no runtime overhead, just a build-time transformation. The tradeoff is that global styles (from `app.css`) still apply everywhere, which is usually what you want (base styles, resets, utility classes).

The `flex: 1` on the input makes it expand to fill available space, keeping the button at its natural width. The `@media (max-width: 600px)` query stacks the input and button vertically on mobile.

⚠️ **Watch Out: The `download` Attribute**

The `download` attribute on `<a>` tags only works for same-origin URLs. If the API returns an absolute URL to a different domain (like a CDN), the browser will navigate to it instead of downloading. The FicHub backend returns same-origin URLs (relative paths like `/cache/epub/123?h=abc`), so this works — but if you change the architecture, keep this in mind.

## RecommendationsTab.svelte: Discovering Similar Fics

The recommendations component follows the same structural pattern as the download tab but serves a different purpose. Instead of downloading a single fic, it discovers similar fics based on collaborative filtering.

### State and Handler

```svelte
<script lang="ts">
  import { fetchRecommendations, ApiError } from '$lib/api/client';
  import type { RecommendationsResponse, RecResult } from '$lib/api/types';
  import { formatWords, stripHtml } from '$lib/util';

  let url = $state('');
  let loading = $state(false);
  let error = $state('');
  let recs = $state<RecResult[]>([]);
  let siteDomain = $state('');
```

The `siteDomain` variable stores which site the seed fic came from (e.g., "archiveofourown.org"). It's displayed in the results header to give context: "20 recommendations from archiveofourown.org."

```typescript
  async function handleGetRecs() {
    if (!url.trim()) {
      error = 'Paste the URL of a fic you liked to see similar stories.';
      return;
    }
    loading = true;
    error = '';
    recs = [];
    try {
      const res = await fetchRecommendations(url.trim(), undefined, 20);
      if (res.err !== 0) {
        error = 'Could not load recommendations.';
        return;
      }
      recs = res.recommendations;
      siteDomain = res.site_domain ?? '';
    } catch (e) {
      error = e instanceof ApiError ? 'Backend error loading recommendations.' : 'Network error.';
    } finally {
      loading = false;
    }
  }
```

The `fetchRecommendations()` call passes `undefined` for the `url_id` parameter because we only have the URL. The backend resolves the `url_id` internally (it hashes the URL to get the ID). We request 20 recommendations — the default and maximum.

The error messages are different from the download tab — "Could not load recommendations" instead of "Something went wrong with that URL." Tailored messages help users understand what went wrong.

### Sorting by Combined Score

```typescript
  let sorted = $derived.by(() => {
    return [...recs].sort(
      (a, b) => b.score + b.community_score * 0.1 - (a.score + a.community_score * 0.1),
    );
  });
```

This is a fascinating scoring formula. Each recommendation has two scores:

- `score` — the algorithmic score from collaborative filtering (what other people who read this fic also bookmarked). This is computed by the Rust backend's recommendation engine.
- `community_score` — user votes (upvote/downvote from the suggestions tab). This is the net vote count from the community.

The formula weights the community score at 10% of the algorithmic score: `score + community_score * 0.1`. This means the algorithm drives recommendations, but popular community picks get a boost. A fic with a community score of +10 gets a 1-point bonus — enough to bump it up a few positions but not enough to override the algorithm.

Why 10%? It's a design choice. Too much community weight and the system becomes a popularity contest. Too little and community feedback is ignored. 10% feels right — the algorithm suggests, the community nudges.

### The Scoring Formula in Context

The formula `score + community_score * 0.1` is applied during sorting, not during data fetching. The backend returns both scores separately, and the frontend combines them for display. This means we could change the weighting without touching the backend — just modify the `$derived` formula in the component.

For example, if we wanted to give community votes more weight, we could change `0.1` to `0.5`:

```typescript
let sorted = $derived.by(() => {
  return [...recs].sort(
    (a, b) => b.score + b.community_score * 0.5 - (a.score + a.community_score * 0.5),
  );
});
```

This flexibility is intentional — the frontend owns the presentation logic, the backend owns the data.

The `[...recs]` spread creates a copy before sorting — we never mutate the original array. This is important for reactivity: Svelte needs to see the original array unchanged to know when to re-render. If we sorted `recs` directly, Svelte would see the mutation but not know the sort order changed.

### The Template

```svelte
  <p class="muted intro">
    Paste a fic you enjoyed, and FicHub will suggest similar stories based on what
    other readers bookmarked together.
  </p>
```

A brief explanation of what recommendations are. This is important UX — users might not know what "collaborative filtering" means, but they understand "what other readers bookmarked together."

```svelte
  {#if !loading && recs.length === 0 && !error}
    <div class="card empty">
      <p class="muted">No recommendations yet. Try a fic above!</p>
    </div>
  {/if}
```

The empty state appears when: not loading, no results, and no error. This means the user hasn't searched yet (or the search returned nothing). The "Try a fic above!" text guides them to the input.

### Recommendation Cards

```svelte
  {#if sorted.length > 0}
    <p class="muted count">{sorted.length} recommendations{siteDomain ? ` from ${siteDomain}` : ''}</p>
    <div class="rec-list">
      {#each sorted as r (r.url_id)}
        <div class="card rec-item">
          <h3>
            <a href={r.download_urls?.epub ?? '#'}>{r.title}</a>
          </h3>
          <p class="muted">by {r.author} · <span class="tag">{r.site_domain}</span></p>
          <p class="meta-line">
            {formatWords(r.words)} words · {r.chapters} chapters · {r.status}
          </p>
          {#if r.summary}
            <p class="desc">{stripHtml(r.summary).slice(0, 200)}</p>
          {/if}
          <div class="rec-footer">
            {#if r.community_score !== 0}
              <span class="badge" class:positive={r.community_score > 0}>
                {r.community_score > 0 ? '▲' : '▼'} {Math.abs(r.community_score)} community
              </span>
            {/if}
            {#if r.download_urls?.epub}
              <a class="btn btn-secondary sm" href={r.download_urls.epub} download>EPUB</a>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
```

Each recommendation card shows:

- **Title** — linked to the EPUB download (if available)
- **Author and site** — for context ("by AuthorName · archiveofourown.org")
- **Stats** — word count, chapters, status
- **Summary** — truncated to 200 characters (shorter than the download tab's 300, because we show many recommendations)
- **Community score badge** — shows ▲ or ▼ with the vote count (only if non-zero)
- **Download button** — quick EPUB download

The `(r.url_id)` after `{#each sorted as r}` is the keyed each block — it tells Svelte to use `url_id` as the unique identifier for efficient DOM updates. Without it, Svelte would re-render the entire list when the sort order changes.

Keyed each blocks are important for performance and correctness. When the list changes (new items added, items reordered, items removed), Svelte uses the key to match old DOM nodes with new data. With keys, it can efficiently move, add, or remove individual DOM nodes. Without keys, Svelte falls back to comparing by index — which can cause incorrect animations, lost focus, and unnecessary re-renders.

For the recommendations list, the key is `url_id` (a unique identifier for each fic). For the suggestions list, it's `s.id` (the database primary key). For the search results, it's also `url_id`. These are stable identifiers that don't change when the list is reordered or filtered. With the key, it can efficiently move DOM nodes around.

The community score badge uses conditional classes:

```svelte
<span class="badge" class:positive={r.community_score > 0}>
  {r.community_score > 0 ? '▲' : '▼'} {Math.abs(r.community_score)} community
</span>
```

The `class:positive={condition}` syntax adds the `positive` class only when the condition is true. When the score is positive, the badge is green. When negative, it uses the default color (muted gray with a ▼ indicator).

The `{Math.abs(r.community_score)}` ensures the number is always positive — the ▲ or ▼ symbol already indicates direction.

### Component Styles

The recommendation styles include:

```css
  .rec-item h3 a {
    color: var(--color-text);
  }
```

This overrides the default link color (which is `var(--color-primary-hover)`, a blue-ish purple) to make recommendation titles white. This is a deliberate design choice — the title should look like a heading, not a link, even though it is one.

The `.rec-list` uses `flex-direction: column` with `gap: 0.8rem` to stack cards vertically with consistent spacing.

## SuggestionsTab.svelte: Community Voting

The suggestions tab is the most complex component — it has a modal dialog, optimistic voting, and multi-step data loading. It's where users can propose fics that pair well with a seed story, and vote on other people's suggestions.

### State

```svelte
<script lang="ts">
  import {
    fetchVotes,
    submitSuggestion,
    castVote,
    ApiError,
  } from '$lib/api/client';
  import type { Suggestion } from '$lib/api/types';

  let seedUrl = $state('');
  let loading = $state(false);
  let error = $state('');
  let suggestions = $state<Suggestion[]>([]);
  let loadedUrlId = $state('');

  // Modal state
  let showModal = $state(false);
  let suggestUrl = $state('');
  let suggestComment = $state('');
  let submitting = $state(false);
  let modalError = $state('');

  // Local vote tracking to give instant feedback before server confirms.
  let localVotes = $state<Record<number, number>>({});
```

This component has three groups of state:

1. **Main state** — the URL input, loading/error states, and the suggestions list
2. **Modal state** — the suggest-a-fic form (URL, comment, submission status)
3. **Local vote tracking** — for optimistic voting

The `localVotes` dictionary is particularly interesting. It maps suggestion IDs to net vote changes that haven't been confirmed by the server yet. When a user clicks "upvote," we immediately update the UI, and the server confirmation follows. The key insight is that `localVotes` stores *deltas* (changes), not absolute values.

### Loading Suggestions

```typescript
  async function loadSuggestions() {
    if (!seedUrl.trim()) {
      error = 'Paste the URL of a fic to see its community suggestions.';
      return;
    }
    loading = true;
    error = '';
    try {
      // Resolve url_id via the recommendations endpoint (returns url_id even with 0 recs).
      const { fetchRecommendations } = await import('$lib/api/client');
      const recRes = await fetchRecommendations(seedUrl.trim(), undefined, 1);
      if (recRes.err !== 0 || !recRes.url_id) {
        error = 'Could not resolve that fic.';
        return;
      }
      loadedUrlId = recRes.url_id;
      const votesRes = await fetchVotes(recRes.url_id);
      if (votesRes.err !== 0) {
        error = 'Could not load suggestions.';
        return;
      }
      suggestions = votesRes.suggestions;
    } catch (e) {
      error = e instanceof ApiError ? 'Backend error.' : 'Network error.';
    } finally {
      loading = false;
    }
  }
```

This is a two-step process, and there's a good reason for it:

### Why Two Steps?

The suggestions API (`/api/v0/recommendations/votes`) needs a `url_id`, not a URL. But users paste URLs. So we need to resolve the URL to an ID first.

The recommendations endpoint (`/api/v0/recommendations`) can resolve URLs to IDs — it does this internally as part of computing recommendations. By calling it with `n=1` (minimal request), we get the `url_id` without computing a full recommendation set. This is a clever reuse of existing infrastructure.

An alternative would be a dedicated `/api/v0/resolve?url=...` endpoint, but that would mean maintaining another route. Reusing the recommendations endpoint is simpler and the performance cost is negligible.

1. **Resolve the URL to an ID** — the votes endpoint needs a `url_id`, not a URL. We use `fetchRecommendations()` with `n=1` (minimal request) to get the backend to resolve the URL to an ID. The comment explains why: "returns url_id even with 0 recs." The backend always resolves the URL, regardless of whether it has recommendations.

2. **Fetch the votes** — now that we have the `url_id`, we can fetch the community suggestions.

The dynamic import (`await import('$lib/api/client')`) is interesting — it's a lazy import that loads the client module only when this function is called. In practice, since the client is already loaded (the layout imports it), this is just a style choice. It could have been a static import at the top.

### The Suggest Modal

```typescript
  function openModal() {
    suggestUrl = '';
    suggestComment = '';
    modalError = '';
    showModal = true;
  }

  function closeModal() {
    showModal = false;
  }
```

Opening the modal resets all form fields. This ensures a clean slate every time — no leftover data from a previous submission.

```typescript
  async function handleSuggest() {
    if (!suggestUrl.trim()) {
      modalError = 'Paste the URL of the fic you want to suggest.';
      return;
    }
    submitting = true;
    modalError = '';
    try {
      const res = await submitSuggestion(loadedUrlId, suggestUrl.trim(), suggestComment.trim() || undefined);
      if (res.err !== 0) {
        modalError = res.msg || 'Could not submit suggestion.';
        return;
      }
      closeModal();
      await loadSuggestions();
    } catch (e) {
      modalError = e instanceof ApiError ? 'Backend error.' : 'Network error.';
    } finally {
      submitting = false;
    }
  }
```

The suggest handler follows the same pattern as other forms: validate, submit, handle errors, close on success. After a successful submission, it refreshes the suggestions list to include the new suggestion.

The `suggestComment.trim() || undefined` converts an empty comment to `undefined` (so the API doesn't receive an empty string). This is a common pattern for optional fields.

The modal template uses a backdrop overlay:

```svelte
{#if showModal}
  <div class="modal-backdrop" onclick={closeModal} role="presentation">
    <div class="modal" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true">
      <h3>Suggest a fic</h3>
      <p class="muted">For: <code>{loadedUrlId}</code></p>
      <label>
        Fic URL
        <input type="url" placeholder="https://archiveofourown.org/works/…" bind:value={suggestUrl} />
      </label>
      <label>
        Comment (optional)
        <textarea rows="3" placeholder="Why do you recommend this?" bind:value={suggestComment}></textarea>
      </label>
      {#if modalError}
        <p class="error-text">{modalError}</p>
      {/if}
      <div class="modal-actions">
        <button class="btn btn-secondary" onclick={closeModal}>Cancel</button>
        <button class="btn" onclick={handleSuggest} disabled={submitting}>
          {#if submitting}<span class="spinner"></span> Submitting…{:else}Submit{/if}
        </button>
      </div>
    </div>
  </div>
{/if}
```

The modal follows a classic overlay pattern:

### Modal Accessibility

The modal uses ARIA attributes for screen reader support:
- `role="dialog"` tells screen readers this is a dialog
- `aria-modal="true"` indicates the rest of the page is inert
- `role="presentation"` on the backdrop marks it as non-semantic

The `e.stopPropagation()` on the inner div prevents clicks inside the modal from closing it. Without this, clicking an input field would close the modal (because the click bubbles to the backdrop).

### Modal Pattern

1. **Backdrop** — clicking outside the modal closes it (`onclick={closeModal}`)
2. **Stop propagation** — clicks inside the modal don't bubble to the backdrop (`e.stopPropagation()`)
3. **Form fields** — URL input and optional comment textarea
4. **Error display** — shows validation or API errors
5. **Actions** — Cancel (close) and Submit (with loading state)

The `aria-modal="true"` and `role="dialog"` attributes make the modal accessible to screen readers. The `role="presentation"` on the backdrop indicates it's purely visual (not semantic content).

### The Complete Voting Flow

Let's trace what happens when a user upvotes a suggestion:

1. **User clicks ▲** → calls `handleVote(s.id, 1)`
2. **Save previous** → `prev = localVotes[s.id] ?? 0` (saves current local delta)
3. **Optimistic update** → `localVotes[s.id] += 1` (UI updates immediately)
4. **Score recalculates** → `netScore(s)` returns `s.net_votes + localVotes[s.id]` (higher now)
5. **Sorted list reorders** → `$derived` re-sorts, suggestion may move up
6. **API call** → `castVote(id, 1)` sends POST to `/api/v0/recommendations/vote`
7. **Backend processes** → records vote in database, returns `{ err: 0, new_score: 3 }`
8. **Success** → `loadSuggestions()` refreshes all suggestions from server
9. **Server data replaces local** → `localVotes` deltas are reconciled with true scores
10. **UI stabilizes** → displayed score matches server truth

If step 7 fails (network error, server error), step 9 reverts: `localVotes[id] = prev`. The user sees the score snap back to its original value.

### Optimistic Voting with Rollback

```typescript
  async function handleVote(id: number, vote: 1 | -1) {
    // Optimistic update.
    const prev = localVotes[id] ?? 0;
    localVotes[id] = (localVotes[id] ?? 0) + vote;
    try {
      const res = await castVote(id, vote);
      if (res.err !== 0) {
        localVotes[id] = prev; // rollback
        return;
      }
      // Refresh to get true net score.
      await loadSuggestions();
    } catch {
      localVotes[id] = prev;
    }
  }
```

This is the most sophisticated pattern in the frontend. Here's what's happening step by step:

1. **Save the previous value** — `prev = localVotes[id] ?? 0` saves the current local vote count. The `?? 0` handles the case where this suggestion hasn't been locally voted on yet.

2. **Apply the vote immediately** — `localVotes[id] += vote` updates the UI instantly. The user sees their vote reflected before the server even processes it.

3. **Send the request** — `castVote(id, vote)` tells the backend about the vote.

4. **On failure, rollback** — if the API returns an error or the network fails, restore the previous value. The user sees the vote disappear as if nothing happened.

5. **On success, refresh** — `loadSuggestions()` fetches the true scores from the server. This reconciles the optimistic update with reality.

The result: the app feels snappy even on slow connections. Users click upvote and immediately see the score change. If the server rejects it (e.g., duplicate vote, network error), the UI reverts.

### Net Score Calculation

```typescript
  function netScore(s: Suggestion): number {
    return s.net_votes + (localVotes[s.id] ?? 0);
  }

  let sorted = $derived.by(() => [...suggestions].sort((a, b) => netScore(b) - netScore(a)));
```

The `netScore()` function combines the server's vote count with any local optimistic votes. If a user just upvoted a suggestion but the server hasn't confirmed yet, the local +1 is included in the displayed score.

### Why Track Deltas, Not Absolute Values?

The `localVotes` dictionary stores *deltas* (changes), not absolute values. When a user upvotes, we add `+1` to `localVotes[id]`. When the server confirms, we refresh and the delta is reconciled.

If we stored absolute values instead, we'd need to track the original server score separately and compute the display value as `originalScore + delta`. The delta approach is simpler — `netScore()` just adds the delta to whatever the server reported.

The `sorted` derived value re-sorts whenever suggestions change or local votes change, keeping the list in order of popularity. This is reactive — when `localVotes` changes (from a vote click), `sorted` recalculates, and the UI re-renders.

### Voting UI

```svelte
  <div class="vote-box">
    <button
      class="vote-btn up"
      class:active={netScore(s) > 0}
      onclick={() => handleVote(s.id, 1)}
      aria-label="Upvote"
    >
      ▲
    </button>
    <span class="score" class:positive={netScore(s) > 0} class:negative={netScore(s) < 0}>
      {netScore(s)}
    </span>
    <button
      class="vote-btn down"
      class:active={netScore(s) < 0}
      onclick={() => handleVote(s.id, -1)}
      aria-label="Downvote"
    >
      ▼
    </button>
  </div>
```

The vote buttons use conditional classes for visual feedback:

- When the score is positive, the upvote button gets the `active` class (green border)
- When the score is negative, the downvote button gets the `active` class (red border)
- The score number itself is green when positive, red when negative

The `class:active={netScore(s) > 0}` syntax is Svelte's conditional class binding. It adds the `active` class only when the condition is true. This is equivalent to `class={netScore(s) > 0 ? 'vote-btn up active' : 'vote-btn up'}` but cleaner.

The `aria-label` attributes make the vote buttons accessible — screen readers will announce "Upvote" or "Downvote" when focused.

The component styles for voting:

```css
  .vote-box {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.15rem;
    flex-shrink: 0;
  }
  .vote-btn {
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
    color: var(--color-muted);
    border-radius: var(--radius-sm);
    width: 2rem;
    height: 1.6rem;
    line-height: 1;
  }
  .vote-btn.up.active {
    color: var(--color-success);
    border-color: var(--color-success);
  }
  .vote-btn.down.active {
    color: var(--color-error);
    border-color: var(--color-error);
  }
```

The `flex-shrink: 0` on `.vote-box` prevents the vote section from shrinking when the card is narrow. The buttons are small (2rem × 1.6rem) and use the arrow characters (▲ ▼) instead of icons.

🧪 **Try It Yourself: Add a Comment Counter**

The suggestion form has a textarea for comments but no character limit indicator. Try adding a character counter that shows "45/280" below the textarea. You'd use a `$derived` value:

```typescript
let commentLength = $derived(suggestComment.length);
```

And in the template:

```svelte
<span class="muted">{commentLength}/280</span>
```

⚠️ **Watch Out: Optimistic Updates and Race Conditions**

The optimistic voting pattern has a subtle issue: if a user clicks "upvote" rapidly (say, 5 times in 1 second), the local vote count becomes +5, but the server might only register one vote (depending on how the backend handles duplicates). The `loadSuggestions()` refresh after each vote helps — but there's still a window where the displayed score is wrong. In a high-traffic app, you'd debounce the vote requests or use a queue.

Another edge case: if two browser tabs are open, each tab's `localVotes` is independent. A vote in one tab won't be reflected in the other until the page is refreshed.

⚠️ **Watch Out: Dynamic Imports Are Rarely Necessary**

The dynamic `import('$lib/api/client')` in `loadSuggestions()` is unusual. In most cases, you'd use a static import at the top of the script block. Dynamic imports are useful for code splitting (loading code only when needed), but since the API client is small and used by many components, it's already loaded by the time this function runs. The dynamic import here is a leftover from an earlier implementation, not a pattern you should copy.

---

# Chapter 32: The Search Syntax Parser

## What Is a Syntax Parser?

Every time you type a search query into Google, there's software behind the scenes interpreting your input. Type `site:github.com svelte` and Google knows to search only GitHub. Type `"exact phrase"` and it knows to match the whole thing. Type `svelte -react` and it knows to exclude results containing "react."

FicHub has its own search language, inspired by AO3's tag-based search. Users can type things like:

```
fandom:"Harry Potter" words:10000-50000 complete:true sort:updated
```

And the search parser translates that into structured filter parameters that the backend understands. The parser lives in `src/lib/search/syntax.ts` — about 350 lines of TypeScript that turn human input into machine-usable filters.

This is a classic compiler/compiler-adjacent problem: you have a DSL (domain-specific language) and need to parse it into a structured representation. We're not building a full compiler (no AST, no code generation), but the fundamentals are the same: tokenization, parsing, and evaluation.

## The AO3-Like Syntax

Our syntax supports these key-value pairs, inspired by AO3's advanced search:

| Syntax | Meaning | Backend Field |
|--------|---------|---------------|
| `fandom:Harry Potter` | Include this fandom tag | `include_tags` (type 1) |
| `char:Harry Potter` | Include this character tag | `include_tags` (type 2) |
| `rel:Harry/Ginny` | Include this relationship tag | `include_tags` (type 3) |
| `tag:Fluff` | Include this freeform tag | `include_tags` (type 4) |
| `words:10000-50000` | Word count range | `min_words`, `max_words` |
| `chapters:5-20` | Chapter count range | `min_chapters`, `max_chapters` |
| `complete:true` | Only complete fics | `complete=true` |
| `site:ao3` | Source site shorthand | `source` |
| `sort:updated` | Sort order | `sort` |
| `after:2024-01-01` | Published after | `date_from` |
| `before:2024-12-31` | Published before | `date_to` |

Plus abbreviations: `t` for title, `a` for author, `f` for fandom, `c` for character, `r` for relationship, `w` for words, `ch` for chapters, `s` for site, `comp` for complete, `creator` for author.

And exclusion with `-`: `-fandom:Naruto` excludes the Naruto fandom. `-tag:Angst` excludes the Angst tag.

The known keys are defined in a Set for O(1) lookup:

```typescript
const KNOWN_KEYS = new Set([
  'title', 't',
  'author', 'creator', 'a',
  'fandom', 'f',
  'char', 'character', 'c',
  'rel', 'relationship', 'r',
  'tag', 'freeform',
  'words', 'w',
  'chapters', 'ch',
  'complete', 'comp',
  'site', 's',
  'sort',
  'after',
  'before',
]);
```

Using a `Set` instead of an array means `KNOWN_KEYS.has(key)` is O(1) — constant time regardless of how many keys we add. For a small set like this, it doesn't matter much, but it's the right habit.

## Tokenizing: Splitting the Input

The first step is breaking the raw input string into tokens. But it's not as simple as splitting on spaces — values can contain spaces:

```
fandom:Harry Potter words:10000-50000
```

If we split on spaces, we'd get `["fandom:Harry", "Potter", "words:10000-50000"]` — wrong! "Harry Potter" is a single value for the `fandom` key.

Here's the basic tokenizer:

```typescript
function tokenize(raw: string): string[] {
  const tokens: string[] = [];
  let current = '';
  let inQuote = false;
  let quoteChar = '';

  for (const ch of raw) {
    if (inQuote) {
      if (ch === quoteChar) {
        inQuote = false;
      } else {
        current += ch;
      }
    } else if (ch === '"' || ch === "'") {
      inQuote = true;
      quoteChar = ch;
    } else if (ch === ' ' || ch === '\t') {
      if (current) {
        tokens.push(current);
        current = '';
      }
    } else {
      current += ch;
    }
  }
  if (current) tokens.push(current);

  return tokens;
}
```

This handles three cases:

1. **Regular words** — split on spaces (and tabs)
2. **Quoted strings** — `fandom:"Harry Potter"` stays as one token, quotes stripped
3. **Empty spaces** — multiple spaces between tokens are ignored

The character-by-character iteration is the classic tokenizer approach. It's simple and handles all edge cases:

- `fandom:Harry Potter words:10000-50000` → `["fandom:Harry", "Potter", "words:10000-50000"]`
- `fandom:"Harry Potter" words:10000-50000` → `["fandom:Harry Potter", "words:10000-50000"]`
- `fandom:'Harry Potter' words:10000-50000` → `["fandom:Harry Potter", "words:10000-50000"]`

The basic tokenizer correctly handles quoted strings but doesn't merge multi-word values without quotes. That's what the greedy tokenizer does.

## The Greedy Tokenizer

The basic tokenizer handles quoted strings, but what about unquoted multi-word values? Users shouldn't have to type `fandom:"Harry Potter"` — `fandom:Harry Potter` should work too.

That's where the greedy tokenizer comes in:

```typescript
function greedyTokenize(raw: string): string[] {
  const basic = tokenize(raw);
  const result: string[] = [];
  let i = 0;

  while (i < basic.length) {
    const token = basic[i];

    if (isKeyToken(token)) {
      const colonIdx = token.indexOf(':');
      const afterColon = token.slice(colonIdx + 1);

      if (!afterColon) {
        // Empty value: "fandom:" — take next tokens until next key
        const parts: string[] = [token];
        i++;
        while (i < basic.length && !isKeyToken(basic[i])) {
          parts.push(basic[i]);
          i++;
        }
        result.push(parts.join(' '));
      } else {
        const key = token.slice(0, colonIdx).toLowerCase();
        if (key === 'words' || key === 'w' || key === 'chapters' || key === 'ch' || key === 'sort' || key === 'complete' || key === 'comp' || key === 'after' || key === 'before' || key === 'site' || key === 's') {
          result.push(token);
          i++;
        } else {
          // Tag-like keys: merge subsequent non-key tokens
          const parts: string[] = [token];
          i++;
          while (i < basic.length && !isKeyToken(basic[i])) {
            parts.push(basic[i]);
            i++;
          }
          result.push(parts.join(' '));
        }
      }
    } else {
      result.push(token);
      i++;
    }
  }

  return result;
}
```

This is the clever part. After basic tokenization, the greedy tokenizer looks at each token and decides how to handle it:

### Case 1: Key token with empty value

If the token is `fandom:` (colon at the end, no value), greedily consume the next tokens until hitting another key token. So `fandom: Harry Potter words:10000` becomes `["fandom: Harry Potter", "words:10000"]`.

### Case 2: Key token with a "range" value

If the key is `words`, `chapters`, `sort`, `complete`, `after`, `before`, or `site`, don't merge subsequent tokens. These values are always single tokens — numbers, dates, or single words. `words:10000-50000` should NOT merge with the next token.

### Case 3: Key token with a tag-like value

For tag-like keys (`fandom`, `char`, `rel`, `tag`, `title`, `author`), merge subsequent non-key tokens. So `fandom: Harry Potter Fluff` becomes `["fandom: Harry Potter Fluff"]` — "Harry Potter Fluff" is all part of the fandom value.

Wait — that's wrong! "Harry Potter" is one fandom, not "Harry Potter Fluff". The greedy tokenizer doesn't know about fandom boundaries. It just merges everything until the next key token. This is a known limitation — users need to be specific:

```
fandom:Harry Potter fandom:Fluff
```

Or use quotes:

```
fandom:"Harry Potter" tag:Fluff
```

### Case 4: Non-key token

Bare words (tokens without a colon, or tokens that don't match a known key) are passed through as-is. These become the full-text search query.

### Edge Cases the Greedy Tokenizer Handles

The tokenizer handles several tricky cases:

- **Empty values** — `fandom:` (colon at end) greedily consumes the next tokens until a key is found
- **Quoted values** — `fandom:"Harry Potter"` keeps the space inside quotes as part of the value
- **Unknown keys** — `rating:T` is treated as a bare word (not a key-value pair) because "rating" isn't in KNOWN_KEYS
- **Mixed exclusion and keys** — `-fandom:Naruto tag:Fluff` correctly excludes Naruto and includes Fluff
- **Multiple spaces** — `fandom:  Harry   Potter` collapses to `fandom:Harry Potter`

The tokenizer is intentionally simple. It doesn't handle nested quotes, escape characters, or complex grammar. For a search DSL, this simplicity is a feature — users can learn the syntax quickly.

### The isKeyToken() Helper

```typescript
function isKeyToken(s: string): boolean {
  let t = s;
  if (t.startsWith('-')) t = t.slice(1);
  const colonIdx = t.indexOf(':');
  if (colonIdx < 1) return false;
  const key = t.slice(0, colonIdx).toLowerCase();
  return KNOWN_KEYS.has(key);
}
```

This identifies whether a token starts a key-value pair. It handles:

1. The `-` exclusion prefix (stripped before checking)
2. The colon separator (must be present and not at position 0)
3. Case-insensitive key matching (lowercased before checking)
4. Known keys only (unknown keys like `rating:T` are treated as bare words)

The `colonIdx < 1` check ensures the colon isn't the first character (which would be `:value` — not a valid key-value pair).

## parseToken(): Extracting Key-Value Pairs

```typescript
interface ParsedToken {
  key: string;
  value: string;
  exclude: boolean;
}

function parseToken(raw: string): ParsedToken | null {
  let exclude = false;
  let s = raw;
  if (s.startsWith('-')) {
    exclude = true;
    s = s.slice(1);
  }

  const colonIdx = s.indexOf(':');
  if (colonIdx < 1) return null;

  const key = s.slice(0, colonIdx).toLowerCase();
  const value = s.slice(colonIdx + 1).trim();
  if (!value) return null;

  return { key, value, exclude };
}
```

This function extracts the key, value, and exclusion flag from a token. It returns `null` for bare words (tokens without a colon), which are treated as full-text search terms.

The flow:
1. Check for `-` prefix → set `exclude = true`
2. Find the colon → split into key and value
3. Lowercase the key → case-insensitive matching
4. Trim the value → remove leading/trailing whitespace
5. Return `null` if no value → bare words are not key-value pairs

The `ParsedToken` interface:

```typescript
interface ParsedToken {
  key: string;    // "fandom", "words", "tag", etc.
  value: string;  // "Harry Potter", "10000-50000", "Fluff", etc.
  exclude: boolean; // true if prefixed with "-"
}
```

## Range Parsing

The `parseRange()` function handles numeric ranges for word count, chapter count, and similar fields:

```typescript
function parseRange(value: string): { min: number | null; max: number | null } {
  if (value.startsWith('>')) {
    const n = Number(value.slice(1));
    return { min: isNaN(n) ? null : n, max: null };
  }
  if (value.startsWith('<')) {
    const n = Number(value.slice(1));
    return { min: null, max: isNaN(n) ? null : n };
  }

  const parts = value.split('-');
  if (parts.length === 2) {
    const min = Number(parts[0]);
    const max = Number(parts[1]);
    return {
      min: isNaN(min) ? null : min,
      max: isNaN(max) ? null : max,
    };
  }

  const n = Number(value);
  return { min: isNaN(n) ? null : n, max: null };
}
```

This handles three formats:

- `words:>10000` → `{ min: 10000, max: null }` — minimum only (greater than)
- `words:<50000` → `{ min: null, max: 50000 }` — maximum only (less than)
- `words:10000-50000` → `{ min: 10000, max: 50000 }` — both min and max
- `words:5000` → `{ min: 5000, max: null }` — single number treated as minimum

The `isNaN()` checks handle malformed input gracefully — if someone types `words:abc`, it returns `{ min: null, max: null }` instead of crashing.

## Site Shorthands

```typescript
const SITE_MAP: Record<string, string> = {
  ao3: 'archiveofourown.org',
  ff: 'fanfiction.net',
  ffn: 'fanfiction.net',
  fp: 'fictionpress.com',
  sb: 'forums.spacebattles.com',
  sv: 'forums.sufficientvelocity.com',
};

function resolveSite(value: string): string {
  return SITE_MAP[value.toLowerCase()] ?? value;
}
```

Users can type `site:ao3` instead of `site:archiveofourown.org`. The map includes common abbreviations — `ff` and `ffn` both map to FanFiction.net. The `?? value` fallback means if someone types a full domain, it's passed through unchanged.

## Date Normalization

```typescript
function normalizeDate(value: string, bound: 'start' | 'end'): string {
  if (/^\d{4}-\d{2}-\d{2}$/.test(value)) {
    return bound === 'start' ? `${value}T00:00:00Z` : `${value}T23:59:59Z`;
  }
  if (/^\d{4}-\d{2}$/.test(value)) {
    return bound === 'start' ? `${value}-01T00:00:00Z` : `${value}-28T23:59:59Z`;
  }
  if (/^\d{4}$/.test(value)) {
    return bound === 'start' ? `${value}-01-01T00:00:00Z` : `${value}-12-31T23:59:59Z`;
  }
  return value;
}
```

This normalizes dates to ISO format with time boundaries:

- `after:2024` → `2024-01-01T00:00:00Z` (start of year)
- `before:2024-06` → `2024-06-28T23:59:59Z` (end of month)
- `after:2024-01-15` → `2024-01-15T00:00:00Z` (start of day)
- `before:2024-12-31` → `2024-12-31T23:59:59Z` (end of day)

The `bound` parameter controls whether we use midnight (start) or end-of-day (end). This ensures `after:2024-01-15` includes fics published on January 15th, and `before:2024-12-31` includes fics published on December 31st.

The regex patterns match three date formats:
- `YYYY-MM-DD` — full date
- `YYYY-MM` — year and month
- `YYYY` — year only

### Date Normalization Edge Cases

A few notes on the implementation:

- The `28` in `${value}-28T23:59:59Z` is a simplification. For months with 30 or 31 days, this loses the last few days. For a search filter, this is acceptable — if someone searches `before:2024-06`, they probably don't care about June 29-30.

- The `12-31` in the year-only case ensures the full year is included. Searching `after:2024` means "published sometime in 2024 or later."

- If the value doesn't match any known date format, it's passed through unchanged. The backend will either parse it or reject it — the frontend doesn't validate dates strictly.

These are pragmatic tradeoffs. A more rigorous implementation would validate dates against a calendar (accounting for leap years, month lengths, etc.), but for a search DSL, the simplicity is worth the minor inaccuracies.

## The parseSearchQuery() Function: Full Walkthrough

This is the main function that ties everything together:

```typescript
export function parseSearchQuery(raw: string): SearchFilters {
  const filters = defaultFilters();
  if (!raw.trim()) return filters;

  const tokens = greedyTokenize(raw);
  const bareWords: string[] = [];

  for (const token of tokens) {
    const parsed = parseToken(token);
    if (!parsed) {
      bareWords.push(token);
      continue;
    }

    const { key, value, exclude } = parsed;

    switch (key) {
      case 'title':
      case 't':
        bareWords.push(value);
        break;

      case 'author':
      case 'creator':
      case 'a':
        bareWords.push(value);
        break;

      case 'fandom':
      case 'f':
        if (exclude) {
          filters.exclude_tags = appendTag(filters.exclude_tags, 1, value);
        } else {
          filters.include_tags = appendTag(filters.include_tags, 1, value);
        }
        break;

      case 'char':
      case 'character':
      case 'c':
        if (exclude) {
          filters.exclude_tags = appendTag(filters.exclude_tags, 2, value);
        } else {
          filters.include_tags = appendTag(filters.include_tags, 2, value);
        }
        break;

      case 'rel':
      case 'relationship':
      case 'r':
        if (exclude) {
          filters.exclude_tags = appendTag(filters.exclude_tags, 3, value);
        } else {
          filters.include_tags = appendTag(filters.include_tags, 3, value);
        }
        break;

      case 'tag':
      case 'freeform':
        if (exclude) {
          filters.exclude_tags = appendTag(filters.exclude_tags, 4, value);
        } else {
          filters.include_tags = appendTag(filters.include_tags, 4, value);
        }
        break;

      case 'words':
      case 'w': {
        const range = parseRange(value);
        if (range.min !== null) filters.min_words = range.min;
        if (range.max !== null) filters.max_words = range.max;
        break;
      }

      case 'chapters':
      case 'ch': {
        const range = parseRange(value);
        if (range.min !== null) filters.min_chapters = Math.round(range.min);
        if (range.max !== null) filters.max_chapters = Math.round(range.max);
        break;
      }

      case 'complete':
      case 'comp':
        if (value === 'true' || value === 'yes' || value === '1') {
          filters.complete = true;
        } else if (value === 'false' || value === 'no' || value === '0') {
          filters.complete = false;
        }
        break;

      case 'site':
      case 's':
        filters.source = resolveSite(value);
        break;

      case 'sort':
        filters.sort = value.toLowerCase();
        break;

      case 'after':
        filters.date_from = normalizeDate(value, 'start');
        break;

      case 'before':
        filters.date_to = normalizeDate(value, 'end');
        break;

      default:
        bareWords.push(token);
        break;
    }
  }

  if (bareWords.length > 0) {
    filters.q = bareWords.join(' ');
  }

  return filters;
}
```

The flow:

1. **Start with defaults** — `defaultFilters()` creates an empty filter set
2. **Handle empty input** — return defaults immediately
3. **Tokenize greedily** — split the input into meaningful tokens
4. **Parse each token** — extract key, value, and exclude flag
5. **Route by key** — each key maps to a specific filter field
6. **Collect bare words** — anything that's not a key-value pair becomes the full-text search query
7. **Join bare words** — combine into a single query string

### Tag Appending

```typescript
function appendTag(existing: string, typeId: number, name: string): string {
  const entry = `${typeId}:${name}`;
  return existing ? `${existing},${entry}` : entry;
}
```

This builds a comma-separated string with type prefixes. So `fandom:Harry Potter tag:Fluff` produces:

```
include_tags = "1:Harry Potter,4:Fluff"
```

The format `typeId:name` matches what the backend expects. The backend splits on commas, then on colons, to extract the tag type and name.

### How Bare Words Become the Query

When the parser encounters a token that's not a key-value pair (like "Harry" or "Potter"), it adds it to `bareWords`. At the end, all bare words are joined with spaces:

```
Harry Potter words:>10000
```

Bare words: `["Harry", "Potter"]`
Result: `q = "Harry Potter"`

Title and author values are also added to bare words (not separate filter fields). This is because the backend's full-text search covers title and author fields automatically. So `title:Harry Potter` and just `Harry Potter` produce the same `q` value — the backend searches titles and authors by default.

## Example Walkthroughs

Let's trace through a few examples:

### Example 1: Simple search

Input: `Harry Potter Fluff`

Tokenize: `["Harry", "Potter", "Fluff"]`
Parse tokens: all are bare words (no colon)
Result: `q = "Harry Potter Fluff"`, everything else at defaults

### Example 2: Tag-based search

Input: `fandom:Harry Potter tag:Fluff words:>10000 complete:true`

Greedy tokenize: `["fandom:Harry Potter", "tag:Fluff", "words:>10000", "complete:true"]`
Parse tokens:
- `fandom:Harry Potter` → `include_tags = "1:Harry Potter"`
- `tag:Fluff` → `include_tags = "1:Harry Potter,4:Fluff"`
- `words:>10000` → `min_words = 10000`
- `complete:true` → `complete = true`

Result: all filters set as expected.

### Example 3: Exclusion

Input: `-fandom:Naruto tag:Fluff`

Parse tokens:
- `-fandom:Naruto` → `exclude_tags = "1:Naruto"` (exclude flag is true)
- `tag:Fluff` → `include_tags = "4:Fluff"`

Result: fics tagged with Fluff but NOT in the Naruto fandom.

### Example 4: Site and sort

Input: `site:ao3 sort:updated after:2024-01-01`

Parse tokens:
- `site:ao3` → `source = "archiveofourown.org"` (resolved via SITE_MAP)
- `sort:updated` → `sort = "updated"`
- `after:2024-01-01` → `date_from = "2024-01-01T00:00:00Z"` (normalized)

🧪 **Try It Yourself: Test the Parser**

Open a browser console and paste the `parseSearchQuery` function (you can find it in `src/lib/search/syntax.ts`). Try these inputs:

```javascript
parseSearchQuery('Harry Potter')
// → { q: "Harry Potter", include_tags: "", exclude_tags: "", ... }

parseSearchQuery('fandom:Harry Potter words:>10000 complete:true')
// → { q: "", include_tags: "1:Harry Potter", min_words: 10000, complete: true, ... }

parseSearchQuery('-fandom:Naruto tag:Fluff sort:updated')
// → { exclude_tags: "1:Naruto", include_tags: "4:Fluff", sort: "updated", ... }

parseSearchQuery('site:ffn words:50000-200000')
// → { source: "fanfiction.net", min_words: 50000, max_words: 200000, ... }
```

⚠️ **Watch Out: The `-` Prefix vs Negative Numbers**

The exclusion prefix (`-fandom:...`) looks similar to negative numbers. But the parser handles this correctly because `isKeyToken()` strips the `-` before checking if it's a known key. If you type `words:-1000`, it would be parsed as `exclude_words` if that were a valid key — but it's not (we don't support excluding word ranges), so it falls through to bare words. Be careful with this edge case.

⚠️ **Watch Out: Greedy Merging Can Be Surprising**

The greedy tokenizer merges non-key tokens after tag-like keys. So:

```
fandom:Harry Potter Fluff tag:Angst
```

Gets tokenized as `["fandom:Harry Potter Fluff", "tag:Angst"]`. The parser treats "Harry Potter Fluff" as a single fandom name. If you wanted two separate tags, you'd need:

```
fandom:Harry Potter fandom:Fluff tag:Angst
```

Or use quotes:

```
fandom:"Harry Potter" tag:Fluff tag:Angst
```

This is a design tradeoff. The greedy approach is more user-friendly for simple cases (most fandoms are multi-word) but can be surprising for complex queries.

---

# Chapter 33: Advanced Search Page

## The Full Application Shell

Before we dive into the search page, let's understand the application layout that wraps it. The layout is the frame that holds the entire app.

### +layout.svelte: The App Shell

```svelte
<script lang="ts">
  import '../app.css';
  import DownloadTab from '$lib/components/DownloadTab.svelte';
  import RecommendationsTab from '$lib/components/RecommendationsTab.svelte';
  import SuggestionsTab from '$lib/components/SuggestionsTab.svelte';
  import { goto } from '$app/navigation';

  let { children } = $props();

  type Tab = 'download' | 'recs' | 'sugg';
  let activeTab = $state<Tab>('download');

  const tabs: { id: Tab; label: string; icon: string }[] = [
    { id: 'download', label: 'Download', icon: '⬇' },
    { id: 'recs', label: 'Recommendations', icon: '★' },
    { id: 'sugg', label: 'Suggestions', icon: '💡' },
  ];

  let searchQuery = $state('');

  function onSearchKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && searchQuery.trim()) {
      goto(`/search?q=${encodeURIComponent(searchQuery.trim())}`);
    }
  }
</script>
```

The layout imports `app.css` globally — every page inherits the base styles. It defines a tab system with three tabs and a navigation search box.

The tab type definition uses a TypeScript union type:

```typescript
type Tab = 'download' | 'recs' | 'sugg';
```

This restricts `activeTab` to exactly these three values. If you try `activeTab = 'settings'`, TypeScript will flag an error.

The tabs array includes icons (emoji) for visual appeal:

```typescript
const tabs: { id: Tab; label: string; icon: string }[] = [
  { id: 'download', label: 'Download', icon: '⬇' },
  { id: 'recs', label: 'Recommendations', icon: '★' },
  { id: 'sugg', label: 'Suggestions', icon: '💡' },
];
```

The search box uses SvelteKit's `goto()` function for navigation:

```typescript
function onSearchKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && searchQuery.trim()) {
    goto(`/search?q=${encodeURIComponent(searchQuery.trim())}`);
  }
}
```

This navigates to the search page with the query in the URL. The `goto()` function is SvelteKit's client-side router — it doesn't cause a full page reload, it just updates the URL and swaps the page component. The `encodeURIComponent` ensures special characters in the query are properly encoded for the URL.

### The Top Bar

```svelte
<div class="app">
  <header class="topbar">
    <div class="brand">
      <span class="logo">📚</span>
      <span class="title">FicHub</span>
    </div>
    <nav class="tab-bar" role="tablist">
      {#each tabs as t}
        <button
          class="tab"
          class:active={activeTab === t.id}
          onclick={() => (activeTab = t.id)}
          role="tab"
          aria-selected={activeTab === t.id}
        >
          <span class="tab-icon">{t.icon}</span>
          <span class="tab-label">{t.label}</span>
        </button>
      {/each}
    </nav>
    <div class="search-area">
      <input
        class="nav-search"
        type="search"
        placeholder="Search…"
        bind:value={searchQuery}
        onkeydown={onSearchKeydown}
        aria-label="Search fanfiction"
      />
      <a class="adv-link" href="/search" onclick={() => (activeTab = 'download')}>Advanced</a>
    </div>
  </header>

  <main class="container">
    {#if activeTab === 'download'}
      <DownloadTab />
    {:else if activeTab === 'recs'}
      <RecommendationsTab />
    {:else if activeTab === 'sugg'}
      <SuggestionsTab />
    {/if}
  </main>

  <footer class="footer muted">
    <span>FicHub — download & discover fanfiction.</span>
    <span class="sites">AO3 · FanFiction.net · FictionPress · Forums</span>
  </footer>
</div>
```

The layout is a full-height flex column:

```css
.app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}
```

This ensures the footer sticks to the bottom even when there's little content.

The `position: sticky; top: 0;` on `.topbar` keeps the navigation visible when scrolling. The `z-index: 10` ensures it stays above content. The `flex-wrap: wrap` allows the top bar items to wrap on narrow screens.

The tab bar uses ARIA attributes (`role="tablist"`, `role="tab"`, `aria-selected`) for accessibility — screen readers can properly announce the current tab and allow keyboard navigation between tabs.

The conditional rendering (`{#if activeTab === 'download'}`) means only the active tab's component is mounted. When you switch from Download to Recommendations, the DownloadTab component is destroyed and RecommendationsTab is created. This keeps the DOM light and avoids unnecessary rendering.

Note the "Advanced" link next to the search box:

```svelte
<a class="adv-link" href="/search" onclick={() => (activeTab = 'download')}>Advanced</a>
```

The `onclick` handler resets the active tab to 'download' — this ensures that when the user returns from the search page, they see the Download tab (not a stale tab state).

### Mobile-Responsive Styles

```css
@media (max-width: 700px) {
  .tab-label {
    display: none;
  }
  .search-area {
    width: 100%;
    margin-left: 0;
  }
  .nav-search {
    flex: 1;
  }
}
```

On mobile (screens narrower than 700px):

### Mobile Considerations

The responsive design addresses several mobile-specific concerns:

1. **Touch targets** — buttons are at least 44×44px (Apple's minimum), achieved with padding on `.btn`
2. **Input types** — `type="url"` triggers the URL keyboard on iOS, `type="number"` triggers the numeric keyboard, `type="date"` triggers the native date picker
3. **Scrolling** — the sticky topbar stays visible while scrolling, so users always have access to navigation and search

The mobile experience is a first-class citizen, not an afterthought.

- **Tab labels hidden** — only the emoji icons (⬇ ★ 💡) are shown. This saves horizontal space.
- **Search area takes full width** — the `margin-left: 0` overrides the desktop `margin-left: auto` (which pushed it to the right).
- **Search input expands** — `flex: 1` makes it fill available space.

This is a common mobile pattern — reduce text, keep functionality. Users recognize the emoji icons, so the labels are redundant on small screens.

## The /search Route

### The Search Page Architecture

The search page is the most complex component in the app. It combines:

1. **Syntax parsing** — translating human input into structured filters
2. **Advanced filters** — dropdowns and inputs for fine-grained control
3. **Filter merging** — combining syntax and advanced filters with priority rules
4. **URL synchronization** — keeping the browser URL in sync with the search state
5. **Auto-search** — triggering a search when the page loads with URL parameters
6. **Pagination** — navigating through large result sets
7. **Result rendering** — displaying results with tags, metadata, and download links

Each of these concerns is handled by a separate piece of state or function, keeping the component manageable despite its complexity.

### +page.ts: Data Loading

```typescript
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url }) => {
  return {
    q: url.searchParams.get('q') ?? '',
    tab: url.searchParams.get('tab') ?? 'work',
  };
};
```

This is a SvelteKit page load function. It runs before the component mounts and extracts the `q` and `tab` URL parameters. These are passed to the component as the `data` prop.

The `PageLoad` type comes from SvelteKit's generated types (`./$types`). It ensures the function signature matches what SvelteKit expects.

This is what makes URL-synced search possible — when you navigate to `/search?q=fandom:Harry+Potter`, the page component receives `data.q = "fandom:Harry Potter"`. When you share a search URL, the recipient sees the same results.

The `tab` parameter defaults to `'work'` — the most common search type.

### +page.svelte: The Search Page

Let's walk through the script block:

```svelte
<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { search } from '$lib/api/search';
  import type { SearchFilters, SearchResult, SearchResponse } from '$lib/api/search';
  import {
    SORT_OPTIONS, COMPLETE_OPTIONS, SOURCE_OPTIONS, defaultFilters,
  } from '$lib/api/search';
  import { parseSearchQuery } from '$lib/search/syntax';
  import { formatWords, relativeTime, detectSite, stripHtml } from '$lib/util';

  let { data } = $props();
```

The `{ data }` prop comes from the `+page.ts` load function. It contains `q` and `tab`. Note that `page` is imported from `$app/state` — this is SvelteKit 5's way of accessing the current page information (URL, params, etc.).

### Tab Management

```svelte
  type SearchTab = 'work' | 'people' | 'bookmark' | 'tag';
  let activeTab = $state<SearchTab>((data.tab as SearchTab) || 'work');

  const searchTabs: { id: SearchTab; label: string }[] = [
    { id: 'work', label: 'Work Search' },
    { id: 'people', label: 'People Search' },
    { id: 'bookmark', label: 'Bookmark Search' },
    { id: 'tag', label: 'Tag Search' },
  ];
```

Four search modes, though only Work Search is fully implemented. The others show placeholder forms with "coming soon" messages. This is a common pattern — build the most important feature first, show placeholders for the rest, and iterate.

### Tab State Management

The search page has two levels of tab state:

1. **Main tabs** (in the layout) — Download, Recommendations, Suggestions. Managed by `activeTab` in `+layout.svelte`.
2. **Search tabs** (in the search page) — Work, People, Bookmark, Tag. Managed by `activeTab` in `search/+page.svelte`.

These are separate state variables in separate components. The search page's tabs don't affect the layout's tabs. This separation keeps concerns clean: the layout manages main navigation, the search page manages search-specific navigation.

When the user clicks "Advanced" in the layout, the layout resets to the Download tab. When the search page loads, it reads the `tab` URL parameter and sets its own active tab.

The `data.tab as SearchTab` cast is necessary because `data.tab` is typed as `string | null`, but `activeTab` expects a `SearchTab` value.

### The doSearch() Function

This is the core of the search page:

```svelte
  let queryInput = $state(data.q ?? '');
  let filters = $state<SearchFilters>(defaultFilters());
  let loading = $state(false);
  let error = $state('');
  let results = $state<SearchResult[]>([]);
  let total = $state(0);
  let currentPage = $state(1);
  let searched = $state(false);
```

Each state variable has a specific purpose:

- `queryInput` — the raw search text (may contain syntax like `fandom:Harry Potter`)
- `filters` — the parsed filters (sent to the API)
- `loading` — whether a search is in progress
- `error` — error message to display
- `results` — the search results array
- `total` — total number of results (for pagination)
- `currentPage` — current page number
- `searched` — whether any search has been performed (to distinguish "no results" from "not searched yet")

### State Naming Conventions

Notice the naming patterns: `queryInput` (not `query`) for the raw text before parsing, `filters` for the parsed object sent to the API, `results` for the search results, `searched` for a boolean flag. Svelte 5 runes work best with simple, descriptive names.

### Advanced Filter Fields

```svelte
  // Advanced filter fields (Work Search).
  let filterComplete = $state('');
  let filterSource = $state('');
  let filterMinWords = $state('');
  let filterMaxWords = $state('');
  let filterMinChapters = $state('');
  let filterMaxChapters = $state('');
  let filterDateFrom = $state('');
  let filterDateTo = $state('');
  let filterSort = $state('');
  let filterIncludeTags = $state('');
  let filterExcludeTags = $state('');
```

Each advanced filter is its own state variable. These correspond to the dropdown and input fields in the filter grid. They're strings (not numbers) because they're bound to input elements — the conversion to numbers happens in `doSearch()`.

### The doSearch() Implementation

```typescript
  async function doSearch() {
    loading = true;
    error = '';
    searched = true;

    // Parse syntax from query input.
    const syntaxFilters = parseSearchQuery(queryInput);

    // Merge with advanced filter fields (advanced overrides syntax).
    filters = {
      ...syntaxFilters,
      q: syntaxFilters.q,
      min_words: filterMinWords ? Number(filterMinWords) : syntaxFilters.min_words,
      max_words: filterMaxWords ? Number(filterMaxWords) : syntaxFilters.max_words,
      min_chapters: filterMinChapters ? Number(filterMinChapters) : syntaxFilters.min_chapters,
      max_chapters: filterMaxChapters ? Number(filterMaxChapters) : syntaxFilters.max_chapters,
      complete: filterComplete === 'true'
        ? true
        : filterComplete === 'false'
          ? false
          : syntaxFilters.complete,
      source: filterSource || syntaxFilters.source,
      date_from: filterDateFrom || syntaxFilters.date_from,
      date_to: filterDateTo || syntaxFilters.date_to,
      sort: filterSort || syntaxFilters.sort,
      include_tags: filterIncludeTags || syntaxFilters.include_tags,
      exclude_tags: filterExcludeTags || syntaxFilters.exclude_tags,
      page: currentPage,
      per_page: 20,
    };

    // Update URL without navigation.
    const qs = new URLSearchParams();
    if (queryInput) qs.set('q', queryInput);
    if (activeTab !== 'work') qs.set('tab', activeTab);
    goto(`/search?${qs.toString()}`, { replaceState: true, keepFocus: true });

    try {
      const res: SearchResponse = await search(filters);
      total = res.total;
      results = res.results;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Search failed.';
      results = [];
      total = 0;
    } finally {
      loading = false;
    }
  }
```

This is a great example of merging two input sources:

1. **Syntax parsing** — the `queryInput` text is parsed into filter values by `parseSearchQuery()`
2. **Advanced filter fields** — the dropdown/input values from the UI

The merge logic: "if the advanced field is set, use it; otherwise, use the parsed syntax value."

### Merge Priority Example

Let's trace through a concrete example:

1. User types `words:>10000 tag:Fluff` in the search bar
2. `parseSearchQuery` returns `{ min_words: 10000, include_tags: "4:Fluff", ... }`
3. User sets "Min Words" dropdown to `5000`
4. `doSearch()` merges: `min_words` from dropdown (5000) overrides syntax (10000), but `include_tags` from syntax ("4:Fluff") is preserved because the advanced tag field is empty.

This merge behavior is intuitive: the UI fields and the syntax bar complement each other. Users can use one or both.

For example, if the user types `words:>10000` in the search bar AND sets the "Min Words" dropdown to 5000, the dropdown value (5000) takes precedence. This gives users flexibility — they can use syntax OR the UI, or mix both.

The `Number(filterMinWords)` converts the string input to a number. If the input is empty, it returns `NaN`, which is falsy, so the syntax value is used instead.

The spread operator (`...syntaxFilters`) copies all properties from the syntax-parsed filters, then the advanced fields override specific properties. This is a clean merge pattern.

### URL Sync: Bookmarkable Searches

URL synchronization is a subtle but important feature. When a user searches, the URL updates to reflect their query. This enables:

- **Bookmarking** — users can save the URL and return to the same search
- **Sharing** — sending a URL to someone shows the same results
- **Back button** — the browser's back button returns to the previous search
- **Direct linking** — clicking a link like `/search?q=fandom:Harry+Potter` shows those results immediately

The implementation uses `goto()` with `replaceState: true` — this updates the URL without adding a history entry. Without `replaceState`, every search would add a history entry, making the back button cycle through intermediate searches instead of going back to the previous page.

### URL Sync Code

```typescript
    // Update URL without navigation.
    const qs = new URLSearchParams();
    if (queryInput) qs.set('q', queryInput);
    if (activeTab !== 'work') qs.set('tab', activeTab);
    goto(`/search?${qs.toString()}`, { replaceState: true, keepFocus: true });
```

The URL is updated to reflect the current search state. This enables:

- **Bookmarking** — users can save the URL and return to the same search
- **Sharing** — sending a URL to someone shows the same results
- **Back button** — the browser's back button returns to the previous search

The `replaceState: true` option updates the URL without adding a new entry to the browser's history. This means the back button doesn't take you through every intermediate search — it goes back to whatever page you were on before the search page. Without this option, every keystroke (or at least every search) would add a history entry.

The `keepFocus: true` option keeps the search input focused after the URL update, so users can keep typing.

### The Complete Search Flow

Let's trace the entire flow from user input to displayed results:

1. **User types** `fandom:Harry Potter words:>10000` in the search bar
2. **Presses Enter** → triggers `onKeydown` → calls `currentPage = 1; doSearch()`
3. **`doSearch()` starts** → sets `loading = true`, `error = ''`, `searched = true`
4. **Parse syntax** → `parseSearchQuery("fandom:Harry Potter words:>10000")` returns `{ q: "", include_tags: "1:Harry Potter", min_words: 10000, ... }`
5. **Merge filters** → syntax values + any advanced filter overrides
6. **Update URL** → `goto("/search?q=fandom:Harry+Potter+words:%3E10000", { replaceState: true })`
7. **Call API** → `search(filters)` sends GET to `/api/v0/search?q=&include_tags=1:Harry+Potter&min_words=10000`
8. **Backend processes** → PostgreSQL full-text search + tag filtering + word count filtering
9. **Response arrives** → `{ total: 42, results: [...], page: 1, per_page: 20 }`
10. **State updates** → `total = 42`, `results = [...]`, `loading = false`
11. **UI re-renders** → results header ("42 results"), result cards with tags, pagination ("Page 1 of 3")

The entire flow takes under 100ms for most queries — the Rust backend's search is fast, and Svelte's reactivity is efficient.

### Auto-Search on Mount

```typescript
  let initialized = $state(false);

  $effect(() => {
    if (data.q && !initialized) {
      initialized = true;
      queryInput = data.q;
      doSearch();
    }
  });
```

This `$effect` handles the case where the user navigates to `/search?q=some+query` directly (via bookmark or share link). On the first render, if there's a `q` parameter, it auto-runs the search.

The `initialized` guard prevents the effect from running again when `data.q` changes (which would happen after the first search updates the URL). Without this guard, you'd get infinite loops — the search updates the URL, the URL change triggers the effect, the effect runs the search again.

### Why Not Use onMount?

Svelte provides `onMount()` for running code when a component first renders. We could use it here, but `$effect` is more flexible. The `$effect` approach handles edge cases: if `data.q` changes after mount (via URL update), the effect re-runs. The `initialized` guard gives us control. Effects integrate with Svelte's reactivity system more naturally.

For our case, `onMount` would be simpler but less robust. The `$effect` pattern is the Svelte 5 way of handling "run once on initial data, but also react to changes."

This is a common Svelte 5 pattern for "run once on mount with data." The `$effect` re-runs whenever `data.q` or `initialized` changes, but the `if (!initialized)` check ensures it only executes once.

### Pagination

```svelte
  function nextPage() {
    currentPage++;
    doSearch();
  }

  function prevPage() {
    if (currentPage > 1) {
      currentPage--;
      doSearch();
    }
  }

  let totalPages = $derived(Math.ceil(total / 20));
```

Pagination is straightforward — increment/decrement the page number and re-run the search. The `$derived` value automatically recalculates total pages when the result count changes.

The `nextPage` function doesn't check if there are more pages — it relies on the button being disabled:

```svelte
<button class="btn btn-secondary" onclick={nextPage} disabled={currentPage >= totalPages}>
  Next →
</button>
```

This is cleaner than checking inside the function — the button state and the function logic are in sync.

### Results Display

```svelte
  {#if results.length > 0}
    <div class="results-header">
      <span class="muted">{total.toLocaleString()} results</span>
      <span class="muted">Page {currentPage} of {totalPages}</span>
    </div>

    <div class="results">
      {#each results as r (r.url_id)}
        <div class="card result-card">
          <div class="result-main">
            <h3>
              <a href={r.source} target="_blank" rel="noopener">{r.title}</a>
            </h3>
            <p class="muted">by {r.author} · <span class="tag">{detectSite(r.source)}</span></p>
            <p class="meta-line">
              {formatWords(r.words)} words · {r.chapters} chapters · {r.status}
            </p>
            {#if r.description}
              <p class="desc">{stripHtml(r.description).slice(0, 250)}</p>
            {/if}
            {#if r.tags.length > 0}
              <div class="tags">
                {#each r.tags.slice(0, 12) as t}
                  <span class="tag-pill" title="{t.type}: score {t.score}">
                    {t.name}
                  </span>
                {/each}
                {#if r.total_freeform > 12}
                  <span class="tag-pill muted">+{r.total_freeform - 12} more</span>
                {/if}
              </div>
            {/if}
          </div>
          <div class="result-meta">
            {#if r.rank}
              <span class="rank">#{Math.round(r.rank * 10) / 10}</span>
            {/if}
            {#if r.updated}
              <span class="muted">{relativeTime(r.updated)}</span>
            {/if}
            <a class="btn btn-secondary sm" href="/?q={encodeURIComponent(r.source)}">Download</a>
          </div>
        </div>
      {/each}
    </div>
```

Each search result card shows:

### Result Card Anatomy

The card uses `display: flex; justify-content: space-between` to put main content on the left and metadata (rank, time, download button) on the right. Tag pills are limited to 12 with a "+N more" indicator. The rank display uses `Math.round(r.rank * 10) / 10` to round to one decimal place.

### Filter Field Types

Each filter field uses the appropriate HTML input type: `text` for free-text, `select` for dropdowns, `number` for numeric values (with `min` attributes for validation), and `date` for native date pickers — no JavaScript date picker library needed.

- **Title** — linked to the source site (opens in new tab via `target="_blank"`)
- **Author and site badge** — "by AuthorName · AO3"
- **Stats** — word count, chapters, status
- **Description** — truncated to 250 characters, HTML-stripped
- **Tag pills** — up to 12 tags shown, with a "+N more" indicator
- **Rank** — the search relevance score (if available), formatted to 1 decimal
- **Relative time** — when the fic was last updated ("3 days ago")
- **Download button** — links to the home page with the fic URL pre-filled

The tag pills use a `title` attribute to show the tag type and score on hover:

```svelte
<span class="tag-pill" title="{t.type}: score {t.score}">
  {t.name}
</span>
```

This is progressive enhancement — the tooltip provides extra info for curious users without cluttering the UI.

The "Download" button links to `/?q={encodeURIComponent(r.source)}` — this navigates to the home page with the fic's source URL pre-filled in the download input. The user just needs to click "Download" on the home page. This is a simple but effective cross-tab navigation.

### Why Cross-Tab Navigation?

The search results don't trigger downloads directly. Instead, they link to the Download tab with the URL pre-filled. This design choice has a reason: downloading is an async operation that might take several seconds (scraping, EPUB generation, caching). By navigating to the Download tab, the user sees the loading spinner and can monitor progress. Separating search from download keeps each feature focused.

### Why Cross-Tab Navigation?

The search results don't have a direct "download" button that triggers the API call. Instead, they link to the home page with the URL pre-filled. This design choice has a reason: downloading is an async operation that might take several seconds (scraping, EPUB generation, caching). By navigating to the Download tab, the user sees the loading spinner and can monitor progress.

If we triggered the download directly from the search results, we'd need to show a loading state within the result card — which would be confusing alongside the search UI. Separating search from download keeps each feature focused.

The tradeoff: an extra click. But for a feature that takes 2-5 seconds, the extra click is worth the clarity.

The result card layout uses flexbox to put the main content on the left and metadata on the right:

```css
.result-card {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
}
.result-meta {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 0.3rem;
  flex-shrink: 0;
}
```

The `.result-meta` column is right-aligned and doesn't shrink (`flex-shrink: 0`), keeping the rank, timestamp, and download button stable.

### The Work Search Filter Grid

```svelte
  {#if activeTab === 'work'}
    <div class="filters card">
      <div class="filter-grid">
        <label class="filter-item full">
          Title / Any Field
          <input type="text" bind:value={queryInput} placeholder="Search in title or any field…" />
        </label>

        <label class="filter-item">
          Completion Status
          <select bind:value={filterComplete}>
            {#each COMPLETE_OPTIONS as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </label>

        <label class="filter-item">
          Site
          <select bind:value={filterSource}>
            {#each SOURCE_OPTIONS as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </label>

        <label class="filter-item">
          Sort By
          <select bind:value={filterSort}>
            {#each SORT_OPTIONS as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </label>

        <label class="filter-item">
          Min Words
          <input type="number" bind:value={filterMinWords} placeholder="0" min="0" />
        </label>

        <label class="filter-item">
          Max Words
          <input type="number" bind:value={filterMaxWords} placeholder="∞" min="0" />
        </label>

        <!-- ... more filter fields ... -->

        <label class="filter-item full">
          Include Tags (type_id:name format)
          <input type="text" bind:value={filterIncludeTags} placeholder="1:Harry Potter,4:Fluff" />
        </label>

        <label class="filter-item full">
          Exclude Tags
          <input type="text" bind:value={filterExcludeTags} placeholder="4:Major Character Death" />
        </label>
      </div>
    </div>
  {/if}
```

The filter grid uses CSS Grid with two columns:

```css
.filter-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.8rem;
}
.filter-item.full {
  grid-column: 1 / -1;
}
```

The `grid-column: 1 / -1` makes certain fields span both columns (like the text inputs for tags and title). This creates a clean layout — narrow fields (dropdowns, number inputs) in two columns, wide fields (text inputs) across the full width.

The filter labels use a consistent pattern:

```css
.filter-item {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  font-size: 0.85rem;
  color: var(--color-muted);
}
```

Labels are muted (gray) and smaller than body text — they're secondary to the actual input values.

The dropdowns use the constants from `search.ts`:

```typescript
export const COMPLETE_OPTIONS = [
  { value: '', label: 'All Works' },
  { value: 'true', label: 'Complete Only' },
  { value: 'false', label: 'In Progress Only' },
] as const;
```

The `value` is what gets sent to the API (empty string for "no filter", `'true'` or `'false'` for specific states). The `label` is what the user sees.

### People, Bookmark, and Tag Search Tabs

These are placeholders with "coming soon" messages:

```svelte
  {#if activeTab === 'people'}
    <div class="filters card">
      <div class="filter-grid">
        <label class="filter-item full">
          Author Name
          <input type="text" placeholder="Search by author name…" />
        </label>
        <label class="filter-item full">
          Fandom
          <input type="text" placeholder="Filter by fandom…" />
        </label>
      </div>
      <p class="muted">People search coming soon — use syntax: author:Name</p>
    </div>
  {/if}
```

Each placeholder shows the planned UI and directs users to the syntax bar as a workaround. This is a good pattern — don't hide the feature entirely, show what's coming and provide an alternative.

### Error and Empty States

```svelte
  {#if error}
    <div class="card error-card"><strong class="error-text">⚠️ {error}</strong></div>
  {/if}

  {#if searched && !loading && results.length === 0 && !error}
    <div class="card empty"><p class="muted">No results found. Try different keywords or filters.</p></div>
  {/if}
```

The empty state only shows when:
- `searched` is true (the user has performed at least one search)
- `loading` is false (the search is complete)
- `results.length === 0` (no results found)
- `error` is falsy (no error occurred)

This four-condition check prevents the "No results" message from flashing during loading or appearing before any search is performed.

### The Three States of a Search

The search page has three visual states:

1. **Initial** — no search performed yet. Shows the search bar, filters, and a blank area. The `searched` flag is false.

2. **Loading** — search in progress. Shows the spinner in the search button, disables interaction. The `loading` flag is true.

3. **Results** — search complete. Shows result cards, pagination, total count. Or shows "No results" if the result set is empty. Or shows an error card if something went wrong.

The transitions between states are managed by the `doSearch()` function, which sets `loading`, `searched`, `error`, `results`, and `total` in a coordinated way. The UI reacts to these state changes via Svelte's reactivity.

## The /search/syntax Page

This is a static reference page — no dynamic behavior, just documentation:

```svelte
<script lang="ts">
  // The syntax reference page is purely static content.
  // It explains the search syntax so users can learn advanced queries.
</script>

<div class="syntax-page">
  <h1>Search Syntax Reference</h1>
  <p>FicHub supports an AO3-inspired search syntax. Type these commands in the search bar:</p>

  <h2>Tag Filters</h2>
  <table>
    <tr><td><code>fandom:Name</code></td><td>Filter by fandom</td></tr>
    <tr><td><code>char:Name</code></td><td>Filter by character</td></tr>
    <tr><td><code>rel:Pairing</code></td><td>Filter by relationship</td></tr>
    <tr><td><code>tag:Name</code></td><td>Filter by freeform tag</td></tr>
    <tr><td><code>-fandom:Name</code></td><td>Exclude a fandom</td></tr>
  </table>

  <!-- ... more documentation ... -->
</div>
```

It explains the search syntax with examples, serving as built-in documentation for users who want to learn the advanced search language. This is accessible from the search page via the "Syntax guide" link.

## CSS Theming: app.css

The global stylesheet defines FicHub's visual identity through CSS custom properties:

```css
:root {
  --color-bg: #0f1117;
  --color-surface: #171a23;
  --color-surface-2: #1f2430;
  --color-border: #2a3040;
  --color-text: #e8eaf0;
  --color-muted: #9aa3b2;
  --color-primary: #6366f1;
  --color-primary-hover: #818cf8;
  --color-success: #22c55e;
  --color-error: #ef4444;
  --color-warning: #f59e0b;
  --radius: 10px;
  --radius-sm: 6px;
  --shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  --max-width: 820px;
  --font: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
  --mono: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, monospace;
}
```

This is a dark theme — dark backgrounds (#0f1117) with light text (#e8eaf0). The color palette is carefully designed for readability and visual hierarchy:

- **Surfaces** (bg → surface → surface-2) get progressively lighter, creating depth
- **Text hierarchy** (text → muted) guides the eye to important information first
- **Status colors** (primary, success, error, warning) draw attention to interactive elements and states

The overall effect is a dark theme that's easy on the eyes for long reading sessions — appropriate for a fanfiction app where users spend hours browsing. The font stack uses system fonts (no web font downloads), saving 100-500KB and providing a native feel.

The color palette is:

- **Background** (#0f1117) — very dark blue-gray
- **Surface** (#171a23) — slightly lighter, used for cards
- **Surface 2** (#1f2430) — used for input fields, secondary surfaces
- **Border** (#2a3040) — subtle borders
- **Text** (#e8eaf0) — off-white (easier on eyes than pure white)
- **Muted** (#9aa3b2) — gray for secondary text
- **Primary** (#6366f1) — indigo purple (buttons, active states)
- **Primary Hover** (#818cf8) — lighter purple on hover
- **Success** (#22c55e) — green (positive votes, success states)
- **Error** (#ef4444) — red (error messages, negative votes)
- **Warning** (#f59e0b) — amber (warnings)

The `--max-width: 820px` constrains the content width, keeping text readable on wide screens. The `--font` stack uses system fonts for fast loading and native feel. The `--mono` font stack is used for code-like elements (URLs, suggestion IDs).

### Global Resets and Utilities

```css
* {
  box-sizing: border-box;
}

html, body {
  margin: 0;
  padding: 0;
  background: var(--color-bg);
  color: var(--color-text);
  font-family: var(--font);
  line-height: 1.6;
  min-height: 100vh;
}
```

The universal box-sizing reset ensures consistent sizing across browsers. The body styles set up the dark theme and system font stack. `line-height: 1.6` provides comfortable reading for body text.

```css
a {
  color: var(--color-primary-hover);
  text-decoration: none;
}
a:hover {
  text-decoration: underline;
}
```

Links use the primary hover color (light purple) and underline on hover. This is a minimal link style that doesn't distract from content.

### Component Classes

```css
.card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius);
  padding: 1.25rem;
  box-shadow: var(--shadow);
}
```

The `.card` class creates the elevated surface effect used throughout the UI. The `box-shadow` adds depth, making cards appear to float above the background.

```css
.btn {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  justify-content: center;
  background: var(--color-primary);
  color: white;
  border: none;
  border-radius: var(--radius-sm);
  padding: 0.6rem 1.1rem;
  font-weight: 600;
  font-size: 0.95rem;
  transition: background 0.15s, transform 0.05s;
}
.btn:hover {
  background: var(--color-primary-hover);
}
.btn:active {
  transform: translateY(1px);
}
.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
```

The `.btn` class provides a consistent button style with a subtle press animation (`transform: translateY(1px)` on `:active`). The disabled state reduces opacity and changes the cursor.

```css
.btn-secondary {
  background: var(--color-surface-2);
  color: var(--color-text);
  border: 1px solid var(--color-border);
}
.btn-secondary:hover {
  background: var(--color-border);
}
```

The secondary button variant uses a gray background instead of purple — used for less prominent actions (pagination, cancel, format buttons).

### Input Styling

```css
input, textarea, select {
  font-family: inherit;
  font-size: 1rem;
  background: var(--color-surface-2);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  padding: 0.6rem 0.8rem;
  outline: none;
}
input:focus, textarea:focus, select:focus {
  border-color: var(--color-primary);
}
```

All form elements share consistent styling. The focus state changes the border to purple — a subtle but important visual indicator.

### The Spinner

```css
.spinner {
  display: inline-block;
  width: 1rem;
  height: 1rem;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
```

A pure CSS spinner using the border trick. The `border-top-color: white` makes one edge of the circle white while the rest is semi-transparent, creating the classic spinning effect.

## Responsive Design

The frontend is fully responsive with media queries at two breakpoints:

**700px** (layout.svelte) — Mobile navigation:
- Tab labels hidden (emoji only)
- Search bar takes full width

**600px** (individual components) — Mobile content:
- Search rows stack vertically
- Filter grids become single-column
- Result cards stack vertically

The responsive approach is mobile-first-ish — most styles work at all sizes, and media queries override specific layouts. The search rows use `display: flex` with `gap: 0.6rem`, which naturally wraps on narrow screens.

For example, in DownloadTab:

```css
@media (max-width: 600px) {
  .search-row {
    flex-direction: column;
  }
}
```

On mobile, the input and button stack vertically instead of sitting side by side. This gives the input full width for easier URL pasting.

In the search page:

```css
@media (max-width: 600px) {
  .query-row {
    flex-direction: column;
  }
  .filter-grid {
    grid-template-columns: 1fr;
  }
  .result-card {
    flex-direction: column;
  }
  .result-meta {
    flex-direction: row;
    align-items: center;
  }
}
```

On mobile:
- The query row stacks vertically
- The filter grid becomes single-column
- Result cards stack vertically
- The metadata row (rank, time, download) becomes horizontal

## Building for Production

When you run `npm run build`, here's what happens:

1. Vite compiles all TypeScript and Svelte files
2. SvelteKit generates the static output
3. adapter-static creates the `build/` directory with `index.html` as the fallback

The output is a fully static SPA — just HTML, CSS, and JavaScript files. No server required. To deploy:

1. Run `npm run build`
2. Copy the `build/` directory to your server
3. Configure nginx (or whatever) to serve `index.html` for all non-file routes
4. The API calls go to `/api/v0/*` which the Rust backend handles

The Vite config's proxy (`'/api': { target: 'http://localhost:8004' }`) is only used during development. In production, nginx handles the routing.

The total build output is usually under 200KB — remarkably small for a full-featured SPA. The `_app/immutable/` directory structure includes content hashes in file names, enabling aggressive browser caching. When you rebuild, the hashes change, so browsers automatically fetch the new versions.

### Deployment Checklist

Here's the complete deployment process:

1. **Build the frontend** — `npm run build` in the frontend directory
2. **Copy the build output** — `rsync -avz build/ user@server:/path/to/fichub/static/`
3. **Restart the Rust backend** — `sudo systemctl restart fichub` (it serves the static files)
4. **Verify** — open the app in a browser, check that tabs work, search works, download works

If you're using nginx instead of the Rust backend's static file handler:

```nginx
server {
    listen 80;
    server_name fichub.example.com;

    # Serve static files
    root /path/to/fichub/build;
    index index.html;

    # SPA fallback — all non-file routes get index.html
    location / {
        try_files $uri $uri/ /index.html;
    }

    # API proxy to Rust backend
    location /api/ {
        proxy_pass http://127.0.0.1:8004;
    }
}
```

The `try_files` directive is the nginx equivalent of SvelteKit's `fallback: 'index.html'` — it serves `index.html` for any path that doesn't match a real file, enabling client-side routing.

🧪 **Try It Yourself: Build and Inspect**

Run `npm run build` in the frontend directory. Open `build/index.html` in a browser (with the Rust backend running on port 8004). The app should load and work identically to `npm run dev` — but served entirely from static files.

Check the file sizes: the JavaScript bundle is usually under 100KB (gzipped under 30KB). The CSS is even smaller. This is one of Svelte's superpowers — the compiler eliminates dead code and inlines reactive logic, resulting in tiny bundles.

Try this experiment: open Chrome DevTools, go to the Network tab, and load the page. You'll see the initial HTML is tiny (~1KB), and the JavaScript loads in a single chunk. After the first load, subsequent navigations are instant — no network requests, just JavaScript module swapping.

⚠️ **Watch Out: The Vite Proxy Is Development-Only**

If you're testing locally with `npm run dev`, the proxy in `vite.config.ts` forwards `/api` requests to `localhost:8004`. But if you try to build and serve the output without the Rust backend running, all API calls will fail with 404 errors. The built output is just static files — it needs the backend to handle the API endpoints.

⚠️ **Watch Out: Content Hash Caching**

The build output includes content hashes in file names (like `_app/immutable/assets/0-abc123.css`). This means when you rebuild, old files aren't automatically deleted. If you're deploying by copying files over, you might accumulate stale files. Consider using `rsync --delete` or clearing the build directory before copying.

---

# Summary and Last 500 Words

Part 7 has walked through the entire SvelteKit frontend of FicHub — from the SPA configuration that disables server-side rendering, through the API client that translates TypeScript into HTTP requests, to the three main feature tabs (Download, Recommendations, Suggestions) and the advanced search system with its custom syntax parser.

We covered five chapters of content:

**Chapter 29** established the foundation: SvelteKit's file-based routing, the adapter-static configuration that produces a buildable SPA, the two-line `+layout.ts` that disables SSR, the intentionally empty root page, the catch-all route for deep link support, and the complete file structure overview. We recapped Svelte 5 runes — `$state`, `$derived`, `$effect`, `$props` — showing how five keywords replace entire state management libraries. The Vite dev server's proxy configuration enables seamless development without CORS or nginx setup.

**Chapter 30** built the API contract between frontend and backend. The TypeScript types in `types.ts` mirror every Rust struct — `FicMeta`, `ExportResponse`, `RecResult`, `Suggestion` — giving us compile-time safety across the full stack. The `client.ts` module provided the `request<T>()` generic helper, the `ApiError` class, and named functions for every endpoint: `fetchExport()`, `fetchMeta()`, `fetchRecommendations()`, `fetchVotes()`, `submitSuggestion()`, `castVote()`. The search module contributed `SearchFilters`, `buildSearchQuery()`, and `search()`. Utility functions (`formatWords()`, `detectSite()`, `stripHtml()`, `relativeTime()`) handled presentation concerns.

**Chapter 31** walked through the three main components line by line. DownloadTab showed the complete download flow: input validation, loading states, API calls, error handling, result display with derived download lists, and a clever bookmarklet. RecommendationsTab demonstrated collaborative filtering display with combined scoring. SuggestionsTab was the most sophisticated — two-step data loading (URL → ID → votes), a modal form for submissions, and optimistic voting with rollback that makes the UI feel instant.

**Chapter 32** demystified the search syntax parser. We built a basic tokenizer, then a greedy tokenizer that handles multi-word values, then `parseToken()` for extracting key-value pairs, then `parseSearchQuery()` as the orchestrator. The parser handles tag types, ranges, site shorthands, date normalization, and exclusion syntax — translating human input into structured `SearchFilters`.

**Chapter 33** assembled everything into the search page. The layout provides the app shell with sticky navigation and tab switching. The search page merges syntax-parsed filters with advanced UI fields, syncs the URL for bookmarkability, auto-searches on mount, and displays paginated results with tag pills and relative timestamps. The CSS theme uses custom properties for easy customization, and responsive media queries make the app work on mobile.

Looking back at the codebase, what strikes me is how *small* everything is. The entire frontend — three feature tabs, a search page, a syntax parser, an API client, and global styles — fits in about 2,500 lines of code. There's no state management library, no routing library, no UI component library. Svelte 5's runes handle state, SvelteKit handles routing, and plain CSS handles styling.

This simplicity is intentional. FicHub is a focused tool, not a platform. Every line of code should earn its place. The API client has exactly the functions the UI needs. The search parser handles exactly the syntax the backend supports. The CSS has exactly the variables the components use.

There's a deeper lesson here about full-stack development with Rust and Svelte. When your backend is fast and your frontend is small, the whole system is fast. A SvelteKit SPA loads in milliseconds. API calls to an Axum backend return in under 50ms. The user experience is snappy not because we optimized aggressively, but because we chose tools that are inherently efficient.

The patterns we've established — typed API clients, optimistic updates with rollback, syntax parsing with greedy tokenizers, URL-synced search — are reusable. They apply to any project where a JavaScript frontend talks to a backend API. The specific types change, but the architecture stays the same.

As we move into the remaining parts of this book, we'll expand FicHub's capabilities — adding more site support, refining the recommendation engine, and improving the search index. But the frontend architecture is solid. The three-tab layout, the API client, the search syntax — these are the foundation everything else builds on.

The best code is code you don't have to think about. When you paste a URL and click Download, you're not thinking about the API client, the fetch call, the type validation, or the EPUB generation. You're just getting a fic. That's the goal — and the frontend delivers it.
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

### Work and Proposal Tests

The unified works model and curator proposals add new test coverage:

```rust
#[sqlx::test]
async fn test_find_or_create_work_new(db: PgPool) {
    let meta = make_meta("Test Story", "Author", 10000);
    let result = find_or_create_work(&db, &meta).await.unwrap();
    match result {
        AutoMergeResult::Created { work_id } => assert!(work_id > 0),
        _ => panic!("Expected Created"),
    }
}

#[sqlx::test]
async fn test_find_or_create_work_merge(db: PgPool) {
    // Create a work first
    let meta1 = make_meta("Same Story", "Author", 10000);
    let result1 = find_or_create_work(&db, &meta1).await.unwrap();
    let work_id = match &result1 {
        AutoMergeResult::Created { work_id } => *work_id,
        _ => panic!("Expected Created"),
    };

    // Second request with same title/author should merge
    let meta2 = make_meta("Same Story", "Author", 10200);
    let result2 = find_or_create_work(&db, &meta2).await.unwrap();
    match result2 {
        AutoMergeResult::Merged { work_id: merged_id, .. } => {
            assert_eq!(merged_id, work_id);
        }
        _ => panic!("Expected Merged"),
    }
}

#[sqlx::test]
async fn test_create_proposal_and_vote(db: PgPool) {
    let user_id = create_test_user(&db, "curator").await;
    set_user_role(&db, user_id, "curator").await;
    let proposal_id = create_proposal(&db, user_id, "merge", Some(1), Some(2), None).await.unwrap();
    vote_proposal(&db, proposal_id, user_id, 1).await.unwrap();
    let proposal = get_proposal(&db, proposal_id).await.unwrap();
    assert_eq!(proposal.status, "pending");
}
```

The test count grows with each new feature. Running `cargo test --lib` after these changes should show 12 passing backend tests.

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
│   │   ├── tags.rs             # /api/v0/tags — tag CRUD
│   │   ├── social.rs           # Bookmarks, ratings, comments (work_id)
│   │   ├── work_proposals.rs   # Curator merge/split proposals
│   │   ├── comments.rs         # Threaded comments (recursive CTE)
│   │   └── health.rs           # Health check endpoint
│   ├── works/
│   │   └── mod.rs              # Auto-merge: find_or_create_work()
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
│   │   ├── models.rs           # FicInfo, ExportLog, WorkRow, WorkProposal, etc.
│   │   └── queries.rs          # SQLx query functions + reputation
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
  "work_id": 42,
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
- [ ] **Frontend tests pass:** `npm run test` — all 98 tests green
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

9. **A unified works model** that links the same story across multiple sites. Auto-merge matches stories by title and author with word count tolerance. Curators can propose merging or splitting works, with community voting and reputation rewards.

10. **A comprehensive test suite** — 327 Rust backend tests (unit + integration), 98 frontend JavaScript tests, covering pure functions, API clients, search builders, syntax parsing, component rendering, social features, and curator proposals.

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

Full user accounts with authentication are still a future feature. However, bookmarks and ratings are already implemented using a simplified session-based approach:
- **Bookmarks** — save works to your collection (POST /api/bookmarks with work_id)
- **Ratings** — rate works on a 1-5 scale (POST /api/ratings with work_id)
- **Comments** — threaded discussions on works (POST /api/comments)

Full authentication (JWT or session-based) with user profiles, reading lists, and download history would require a `users` table with password hashing (argon2) and protected API routes.

### A Mobile App

The API is already REST-based, which makes it trivial to build a mobile client. Consider:
- **React Native** — If you're comfortable with JavaScript (you already know React concepts from Svelte)
- **Flutter** — If you want a single codebase for iOS and Android with good performance
- **Tauri Mobile** — If you want to reuse your existing Rust code as the core

A mobile app could add offline reading (cache EPUBs locally), push notifications (when bookmarked fics update), and native sharing.

### Community Features

The tag system is already a step toward community features. Several are now implemented:
- **Bookmarks** — save works to your personal collection (work_id-based)
- **Ratings** — rate works on a 1-5 scale (work_id-based)
- **Comments** — threaded comments on works, with moderation
- **Curator proposals** — propose merging or splitting works, with community voting
- **Reputation system** — earn reputation by contributing to curation

Future community features:
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
