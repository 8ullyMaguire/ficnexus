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
7. The EPUB generator creates an ebook file from the extracted data
8. The file is saved to a disk cache (so the next download is instant)
9. The metadata is stored in PostgreSQL (so it appears in search results)
10. The EPUB file is sent back to the browser for download

This flow is deterministic and auditable. If something goes wrong, logs tell you exactly which step failed. That's the kind of reliability you get when you build things methodically.

## How the Pieces Connect

Let's trace the journey of a single request, from your browser to the database and back:

1. **You paste a URL and click Download.** Your browser sends an HTTP request to the SvelteKit server running on port 5173.

2. **SvelteKit processes the request.** The SvelteKit frontend has a page component with a form. When you submit it, JavaScript sends a fetch request to the backend API at `http://localhost:3000/api/download`.

3. **Axum receives the request.** The Rust backend is running on port 3000. The Axum router sees the URL path `/api/download` and matches it to a handler function.

4. **The handler processes the request.** It extracts the URL from the request body, figures out which scraper to use based on the domain (ao3.com → AO3 scraper, fanfiction.net → FFN scraper, etc.), and calls the appropriate scraper.

5. **The scraper fetches the story.** It sends an HTTP request to the fanfiction site, parses the HTML response, and extracts the story content, chapter structure, and metadata.

6. **The EPUB generator builds the file.** It takes the scraped data and creates an EPUB file — a zip archive with HTML files for each chapter, a table of contents, metadata, and cover image.

7. **The response comes back.** The handler sends the EPUB file back through Axum → your browser → your Downloads folder.

8. **Meanwhile, PostgreSQL stores data.** The story metadata, user bookmarks, and suggestion data all live in PostgreSQL. When you download a story, its metadata gets saved to the database.

9. **Redis handles caching.** Frequently accessed data — like recent downloads, recommendation scores, and rate limits — gets cached in Redis for lightning-fast access.

That's the full loop. Browser → SvelteKit → API → Rust → Scraper → Fanfiction Site → EPUB Generator → PostgreSQL + Redis → Response → Your browser → Download.

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

Think of a database like a filing cabinet. Each drawer is a **table** (like "users" or "stories" or "suggestions"). Inside each drawer, each folder is a **row** — one record (one user, one story, one suggestion). Each folder has labeled sections — those are the **columns** (name, email, word count, etc.).

Here's what FicHub's main tables look like:

**users**
| id | username | ao3_user | created_at |
|----|----------|----------|------------|
| 1 | readingrainbow | ao3_user_42 | 2026-01-15 |
| 2 | fictionfiend | ao3_user_87 | 2026-02-03 |

**stories**
| id | site | site_id | title | author | word_count | tags |
|----|------|---------|-------|--------|------------|------|
| 1 | ao3 | 12345 | The Luminary | starweaver42 | 85000 | [Adventure, Slow Burn] |
| 2 | ffn | 67890 | Second Chances | phoenix_writer | 45000 | [Romance, Angst] |

**bookmarks**
| user_id | story_id | created_at |
|---------|----------|------------|
| 1 | 1 | 2026-03-01 |
| 2 | 1 | 2026-03-05 |

**suggestions**
| id | story_id | user_id | review | votes |
|----|----------|---------|--------|-------|
| 1 | 2 | 1 | "Absolutely beautiful writing" | 42 |

When you download a story, FicHub checks if it's already in the `stories` table. If not, it adds it. When someone bookmarks a story, a row goes into the `bookmarks` table. The recommendation engine queries the `bookmarks` table to find patterns.

## PostgreSQL: Our Filing Cabinet of Choice

There are dozens of databases you could use — MySQL, SQLite, MongoDB, Redis, and many more. For FicHub, we chose **PostgreSQL** (often called "Postgres"). Let's quickly compare the main options so you understand why:

- **SQLite** — A file-based database, great for simple apps and mobile apps. It doesn't require a server process. But it doesn't handle concurrent writes well, and it lacks advanced features like arrays and full-text search. Fine for a todo app, not ideal for FicHub.

- **MySQL** — The most popular database in the world (by number of installations). It's fast, reliable, and well-documented. But it lacks some of PostgreSQL's advanced features, and its default settings prioritize speed over data safety.

- **MongoDB** — A "NoSQL" document database that stores JSON-like documents instead of tables and rows. It's flexible and easy to get started with, but it can lose data if not configured carefully, and complex queries (like our recommendation engine) are harder to express.

- **PostgreSQL** — The "most advanced" open-source database. It supports SQL and advanced features like arrays, full-text search, JSON columns, and custom types. It prioritizes data safety (it won't silently lose your data) and extensibility.

Why PostgreSQL?

**It's battle-tested.** PostgreSQL has been in development since 1996 and powers some of the biggest websites in the world (Instagram, Spotify, Reddit). It's incredibly reliable.

**It handles complex queries.** We need to do things like "find stories that users with similar tastes have bookmarked." PostgreSQL's query language (SQL) makes these kinds of queries straightforward.

**It has rich data types.** PostgreSQL supports arrays, JSON, full-text search, and more. We'll use arrays for tags and full-text search for the search feature.

**It's free and open source.** No licensing fees, no vendor lock-in. You own your data.

Here's what a SQL query looks like:

```sql
-- Find all stories with the "Harry Potter" tag, sorted by word count
SELECT title, author, word_count
FROM stories
WHERE 'Harry Potter' = ANY(tags)
ORDER BY word_count DESC
```

Don't worry if that doesn't make sense yet — we'll learn SQL step by step in later chapters. For now, just know that this query searches the `stories` table for anything tagged "Harry Potter" and returns the results sorted from longest to shortest.

We'll also use **Redis** (or Valkey, the open-source Redis alternative) as a fast, temporary storage for caching. Think of Redis like a whiteboard — quick to write on, quick to read from, but erased when you power off. We use it to cache recommendation scores, rate limiting counters, and frequently accessed data.

## Putting It All Together

Let's zoom out and see how all these pieces fit:

- **HTML** gives structure to our pages
- **CSS** makes them look beautiful
- **JavaScript** makes them interactive
- **SvelteKit** organizes the frontend code into pages and components
- **Axum** organizes the backend code into routes and handlers
- **JSON** is the language between frontend and backend
- **PostgreSQL** stores all our data
- **Redis** caches frequently accessed data for speed

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

💡 **Key Concept:** Redis is NOT a replacement for PostgreSQL. PostgreSQL stores all our permanent data (stories, users, bookmarks). Redis stores temporary, frequently-accessed data (caches, counters, session tokens). Think of PostgreSQL as a filing cabinet in a vault, and Redis as a whiteboard on your desk.

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
