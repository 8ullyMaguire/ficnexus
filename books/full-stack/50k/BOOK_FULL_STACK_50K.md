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

## Schema Design Principles

Looking at all our migrations together, some patterns emerge:

### 1. Use Appropriate Data Types


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
  id: string;
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
