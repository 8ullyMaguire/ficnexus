# Part 1: Welcome to Web Development

---

## Chapter 1: What We're Building

Welcome, friend. Pull up a chair. You're about to learn something amazing.

By the time you finish this book, you'll have built a real, working website from scratch—a website that people actually use. Not a toy. Not a "hello world" page that sits in a drawer. A real application that does something useful and looks great doing it.

The website is called **FicHub**.

### Meet FicHub

Here's the idea: you love reading fanfiction. You've got your favorites bookmarked, your subscriptions queued up, your reading list longer than a CVS receipt. But reading on Archive of Our Own (AO3) or FanFiction.net on your phone or tablet isn't always comfortable. The ads are annoying. The formatting is weird. You want to read your favorites on your Kindle, or on an e-reader, or just in a nice clean ebook format.

That's where FicHub comes in.

You go to FicHub. You paste in the URL of a fanfiction story—maybe it's a 200,000-word Harry Potter epic you've been putting off, or a sweet 5,000-word one-shot that made you cry. You click a button. A few moments later, you download a perfectly formatted ebook file. You load it onto your e-reader. Done.

That's the core idea. Simple, right? But under that simple idea is a whole world of web development, and we're going to explore every corner of it.

### What You'll Learn

This isn't just a book about building FicHub. It's a book about learning the fundamental skills of modern web development. By the end, you'll understand:

- **SvelteKit** — A modern framework for building web applications. Think of it as the scaffolding that holds your website together. We'll use it for the frontend (what users see and interact with) and the backend routes (the glue between the user and the server).

- **TypeScript** — A version of JavaScript that helps you catch mistakes before they become bugs. If JavaScript is writing with a pen, TypeScript is writing with a pen that tells you when you misspell a word.

- **CSS** — The styling language that makes your website look beautiful. Fonts, colors, layouts, animations—CSS is where the art meets the code.

- **APIs** — Application Programming Interfaces. These are the bridges that let different parts of your application talk to each other. The frontend asks the backend a question; the backend answers.

- **Testing** — Writing automated tests that make sure your code actually works. It's like having a robot assistant who double-checks everything you do.

- **Rust** (in later parts) — The programming language that powers FicHub's backend. Fast, safe, and increasingly popular.

- **PostgreSQL** (in later parts) — The database that stores all the information about stories, users, and downloads.

You don't need to know any of these things yet. That's the whole point of this book. We're starting from zero, and we're building up, step by step.

### The Three Tabs

When you open FicHub, you'll see three main tabs at the top of the page. Each one does something different:

**1. Download Tab**

This is the heart of the app. You paste a fanfiction URL into a text box, click "Download," and FicHub does its magic. It goes to the source website (AO3, FanFiction.net, and others), grabs the story content, and packages it into an ebook file. You get a download button and you're on your way.

**2. Recommendations Tab**

Sometimes you don't know what you want to read next. The Recommendations tab suggests stories based on what's popular, what's new, or what matches your interests. It's like that friend who always knows the best books to recommend.

**3. Suggestions Tab**

This is where the community comes in. Other users can suggest stories they think people would enjoy. It's a curated list, built by readers for readers. You can browse, discover, and find your next obsession.

### The Search Feature

But here's the really cool part: FicHub has a powerful search feature that uses syntax inspired by AO3. You might know AO3's search system if you've ever wanted to find, say, "all Harry Potter fics where Draco is the main character, rated Mature, with at least 50,000 words, that are complete."

In FicHub, you can type queries like:

```
hp/dm rating:mature words:50000+ status:complete
```

That's not made up—that's real search syntax that FicHub understands. We'll build this in later chapters, and by the time we do, you'll understand exactly how it works under the hood.

### What the Finished App Looks Like

Let me paint you a picture.

You open your browser and navigate to FicHub. The page loads with a dark background—a deep charcoal gray that's easy on the eyes, perfect for late-night browsing. The logo sits in the top left, sleek and modern.

Below the logo, three tabs stretch across the page: **Download**, **Recommendations**, **Suggestions**. The active tab glows with a subtle highlight—maybe a warm amber or a cool blue, depending on the theme.

On the Download tab, there's a prominent text input field in the center of the page. It's wide, inviting, with a placeholder that says "Paste your story URL here..." Below it, a button that says "Download." Clean. Simple. No clutter.

The Recommendations tab shows a grid of story cards—each one with a title, author, word count, rating, and a short excerpt. You can click any card to see more details or download it directly.

The Suggestions tab is similar but feels more community-driven—stories that other readers have flagged as worth checking out.

The whole interface is responsive. It looks great on a massive desktop monitor, a laptop, a tablet, and a phone. The layout adjusts. The text is readable. The buttons are tappable.

That's what we're building. And by the end of this book, it'll be real.

### How the Pieces Fit Together

Let's zoom out for a moment and see the big picture. When you use FicHub, here's what happens behind the scenes:

1. **You open your browser** and go to FicHub's website. Your browser asks FicHub's server for the page.

2. **SvelteKit serves the page.** SvelteKit is the framework we're using. It generates the HTML, CSS, and JavaScript that your browser needs to display the page. It's like a chef preparing your meal.

3. **You paste a URL and click Download.** Your browser sends a request to FicHub's backend API. "Hey, please download this story."

4. **The API talks to the Rust backend.** The API is like a waiter—it takes your order to the kitchen (the Rust backend) and brings back the result.

5. **The Rust backend does the work.** It fetches the story from AO3 (or wherever), parses the content, and creates an ebook file. This all happens server-side, on a powerful machine somewhere in the cloud.

6. **PostgreSQL stores the data.** Information about stories, downloads, and users gets saved in a database. Think of it as a filing cabinet that never loses anything.

7. **The result comes back to your browser.** You get a download link. You click it. The ebook is on your device.

8. **You read your fanfiction in comfort.** Happy ending.

It's a lot of moving parts, but we're going to understand each one. We'll build them one at a time, and by the end, you'll know how they all fit together.

### Why Build This?

There are a million reasons to learn web development. Maybe you want a new career. Maybe you want to build your own projects. Maybe you just want to understand how the internet works.

Whatever your reason, building something real is the best way to learn. Not tutorials. Not courses. Not watching someone else code. *Building something yourself.*

FicHub is the perfect project because it touches every part of modern web development:
- Frontend (what users see)
- Backend (the logic that makes it work)
- Database (where data lives)
- APIs (how the pieces talk)
- File handling (downloading ebooks)
- Third-party integrations (fetching stories from other sites)

You'll learn skills that transfer to any web development project, not just this one.

### Let's Go

Grab a drink, clear your desk, and take a deep breath. We're about to start building.

The next chapter covers the fundamentals of how the web works. Even if you think you already know this stuff, stick around—you might learn something new. The basics matter more than you think.

See you there.

---

> **Try It Yourself: Before We Start**
>
> Before moving on, do these two things:
>
> 1. Open your web browser. Visit any website you like—maybe a news site, maybe Google, maybe a social media feed. Look at the URL in the address bar. That URL is the website's address, just like your house has an address. You'll learn what all those parts mean soon.
>
> 2. Right-click on the page and select "View Page Source" (the exact wording varies by browser). You'll see a wall of text that looks like gibberish. That's HTML—the language that makes web pages. Don't worry about understanding it yet. Just look at it and know that this is what every website is made of.

---

## Chapter 2: How the Web Works

Before we write a single line of code, let's understand what we're working with. The web is the most complex system humans have ever built, but the core ideas are surprisingly simple. Let's break it down.

### The Internet: A Postal System

Think of the internet like a global postal system.

When you send a letter to a friend, the letter doesn't just teleport to their mailbox. It goes through a whole journey:

1. You write the letter and put it in your mailbox.
2. The mail carrier picks it up.
3. It goes to a local post office.
4. It gets sorted and routed through regional distribution centers.
5. It travels by truck, plane, or train to your friend's area.
6. It arrives at their local post office.
7. Their mail carrier delivers it to their mailbox.
8. Your friend opens it and reads it.

The internet works almost exactly the same way. When you type a URL into your browser (like `https://fichub.com`), here's what happens:

1. Your browser looks up the address of fichub.com (called DNS—like a phone book for the internet).
2. Your browser sends a request to that address.
3. The request hops through routers, cables, and maybe even undersea fiber optic lines.
4. It arrives at the server where fichub.com lives.
5. The server processes your request and sends back a response.
6. The response travels back through the same maze.
7. Your browser receives it and renders the page.

The whole thing takes less than a second. It's magic, really. But it's magic built on layers of simple, logical systems.

### HTML: The Structure

**HTML** stands for **HyperText Markup Language**. That's a fancy name for something simple: it's the structure of a web page.

Imagine you're building a house. Before you paint the walls, hang curtains, or install light switches, you need a frame. The walls, the floors, the roof—those are the structure. Without them, you've got nothing.

HTML is that frame.

When you view the source of any web page, you see HTML tags. A tag looks like this:

```html
<h1>This is a Heading</h1>
<p>This is a paragraph of text.</p>
```

The stuff inside the angle brackets (`< >`) tells the browser what each piece of content *is*. The `<h1>` tag says "this is a big heading." The `<p>` tag says "this is a paragraph." The browser reads these tags and knows how to lay out the page.

Here are the most common HTML elements you'll encounter:

```html
<!-- A heading (big text) -->
<h1>This is the biggest heading</h1>
<h2>This is a slightly smaller one</h2>
<h3>And so on, getting smaller</h3>

<!-- A paragraph -->
<p>This is a block of text. Browsers add space around paragraphs automatically.</p>

<!-- A link -->
<a href="https://example.com">Click here to go somewhere</a>

<!-- A button -->
<button>Click me!</button>

<!-- A text input -->
<input type="text" placeholder="Type something here">

<!-- A container (divides content) -->
<div>
  <p>This paragraph is inside a div.</p>
</div>
```

HTML is the foundation. Everything else—CSS, JavaScript, images, fonts—is built on top of it.

### CSS: The Styling

**CSS** stands for **Cascading Style Sheets**. If HTML is the frame of the house, CSS is the paint, the wallpaper, the hardwood floors, the light fixtures. It's everything that makes the house look like *your* house.

Without CSS, every web page looks like a plain text document—black text on a white background, everything stacked vertically, no colors, no fonts, no layout.

With CSS, you can do anything:

```css
/* Make all headings blue */
h1 {
  color: blue;
}

/* Add a dark background to the page */
body {
  background-color: #1a1a2e;
  color: white;
}

/* Make a button look clickable */
button {
  background-color: #e94560;
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 8px;
  cursor: pointer;
  font-size: 16px;
}

/* Make the button glow when you hover over it */
button:hover {
  background-color: #ff6b81;
  box-shadow: 0 0 10px rgba(233, 69, 96, 0.5);
}
```

CSS selectors are how you target specific elements. You can select by tag name (`h1`), by class (`.highlight`), by ID (`#main-title`), or by many other patterns. We'll dive deep into CSS in Chapter 4.

One of the most powerful things CSS does is **layout**. Using tools like Flexbox and CSS Grid, you can arrange elements in rows, columns, grids, or any configuration you want. The dark, tabbed interface of FicHub? That's all CSS.

### JavaScript: The Interactivity

If HTML is the structure and CSS is the style, **JavaScript** is the behavior. The light switches, the door locks, the thermostat—JavaScript makes things *happen*.

JavaScript is the programming language of the web. Every web browser understands it. When you click a button and a menu appears, that's JavaScript. When you type in a search box and see suggestions pop up, that's JavaScript. When a page loads data without refreshing, that's JavaScript.

Here's a tiny example:

```javascript
// When the button is clicked, show an alert
document.querySelector('#my-button').addEventListener('click', () => {
  alert('Hello from JavaScript!');
});
```

This code finds a button on the page (the one with `id="my-button"`) and says "when someone clicks this, pop up an alert box."

JavaScript is everywhere on the modern web. It handles:
- Form validation (checking if your email address looks right before you submit)
- Animations (smooth transitions between states)
- API calls (fetching data from servers)
- State management (keeping track of what's happening in the app)
- Routing (navigating between pages without full page reloads)

In FicHub, JavaScript (written as TypeScript) handles everything: when you click the Download button, when the tabs switch, when the search results appear, when the progress bar moves.

> **Watch Out: Confusion Between Java and JavaScript**
>
> Java and JavaScript are completely different languages. The name "JavaScript" was a marketing decision in the 1990s, and people have been confused ever since. They share about as much in common as a car and a carpet. If someone tells you "I know Java," that doesn't mean they know JavaScript, and vice versa.

### What Is a Web Server?

Let's go back to our analogies.

A **web server** is like a restaurant kitchen.

When you go to a restaurant, you don't walk into the kitchen and start cooking. You sit at a table. You look at the menu. You tell the waiter what you want. The waiter takes your order to the kitchen. The kitchen prepares your food. The waiter brings it back to your table.

A web server works the same way:

- **You** are the customer.
- **Your browser** is the dining room where you sit.
- **The server** is the kitchen.
- **The API** (which we'll talk about next) is the menu—the list of things you can order.
- **The waiter** is the network connection that carries your request and delivers the response.

When you visit FicHub, your browser sends a request to FicHub's server. The server processes the request—maybe it needs to look up some data in the database, or generate a page, or fetch a story from AO3. Then it sends a response back to your browser: "Here's the HTML, CSS, and JavaScript you need to display this page."

Servers are just computers. They run software that listens for incoming requests and sends back responses. They can be a Raspberry Pi under your desk or a massive machine in a data center. The concept is the same.

### What Is an API?

**API** stands for **Application Programming Interface**. That's a mouthful, but the concept is simple.

Remember the menu at the restaurant? The menu tells you what dishes are available, what they cost, and what ingredients they have. You don't need to know how the chef cooks them. You just need to know what you can order.

An API is exactly that—a menu for software.

When FicHub's frontend (the part you see in your browser) needs data from the backend (the part that runs on the server), it calls the API. The API says, "Here are the things you can ask for and the format you need to ask for them."

For example, FicHub might have an API endpoint like:

```
POST /api/download
```

That's like a menu item. When the frontend sends a request to that endpoint with a story URL, the backend says, "Got it! I'll fetch that story and send you an ebook."

APIs use specific formats. The most common is **REST** (Representational State Transfer), which uses HTTP methods:

- **GET** — "Give me some data" (like reading from a menu)
- **POST** — "Here's some new data, please process it" (like placing an order)
- **PUT** — "Update this existing data" (like changing your order)
- **DELETE** — "Remove this data" (like canceling your order)

We'll work with APIs throughout this book, and by the end, you'll be building your own.

### The Request-Response Cycle

Let's trace through the full journey of a request in FicHub:

**Step 1: You type a URL.**  
You paste a fanfiction URL into FicHub's input field and click "Download."

**Step 2: JavaScript sends a request.**  
FicHub's JavaScript code takes the URL and sends it to the backend API. This happens behind the scenes—you don't see it, but it's happening.

**Step 3: The request travels across the internet.**  
Your request hops from your browser to FicHub's server, just like our postal system analogy.

**Step 4: The server processes the request.**  
The Rust backend receives the request. It validates the URL, connects to AO3 (or wherever the story is), fetches the content, and creates an ebook file.

**Step 5: The server sends a response.**  
The backend sends back a response. If everything worked, it includes a download link. If something went wrong, it includes an error message.

**Step 6: JavaScript displays the result.**  
FicHub's JavaScript receives the response and updates the page. Maybe it shows a "Download Complete!" message and a link to your ebook.

**Step 7: You download the file.**  
You click the link, and the ebook file downloads to your device.

This cycle—request, process, respond—happens millions of times a day across the entire internet. Every time you load a page, click a button, or submit a form, this cycle runs.

### Status Codes: The Three-Digit Language

When a server sends back a response, it includes a **status code**—a three-digit number that tells you what happened. Think of it like a doctor's diagnosis: a simple code that tells a big story.

Here are the ones you need to know:

**200 OK**  
Everything went perfectly. The server found what you asked for and sent it back. This is the "all clear" signal. When you load a page successfully, the status code is 200.

```json
HTTP/1.1 200 OK
Content-Type: application/json

{
  "status": "success",
  "download_url": "/files/story.epub"
}
```

**301 Moved Permanently**  
The page has moved to a new address. Your browser automatically redirects you to the new location. Like a "We've moved!" sign on a storefront.

**404 Not Found**  
The server can't find what you're looking for. The page doesn't exist, or the URL is wrong. We've all seen the "404 Not Found" page—it's the internet's version of "Sorry, that address doesn't exist."

**401 Unauthorized**  
You need to log in to access this resource. The server is saying, "Who are you? Show me your ID."

**403 Forbidden**  
You're logged in, but you don't have permission to access this. The server knows who you are but says, "Nope, not for you."

**500 Internal Server Error**  
Something went wrong on the server's side. The server is having a bad day. This is the most frustrating status code because it means the problem isn't on your end—there's nothing you can do but try again later.

**503 Service Unavailable**  
The server is too busy or under maintenance. It's like a restaurant with a "Closed for Renovation" sign.

```http
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

{
  "error": "Something went wrong while processing your request."
}
```

You don't need to memorize all of these right now. But knowing the basics will help you debug problems later. When something goes wrong, the status code is usually the first clue.

### What Is JSON?

**JSON** stands for **JavaScript Object Notation**. It's the most common format for sending data over the internet.

Think of JSON as a universal language that computers speak. When the server sends data back to your browser, it's usually in JSON format. When your browser sends data to the server, it's usually JSON too.

Here's what JSON looks like:

```json
{
  "title": "The Dragon's Apprentice",
  "author": "SilverQuill42",
  "word_count": 87432,
  "rating": "Mature",
  "is_complete": true,
  "tags": ["fantasy", "romance", "slow burn"],
  "chapters": 24
}
```

It's organized as key-value pairs, wrapped in curly braces. The keys are always strings (in quotes), and the values can be strings, numbers, booleans (true/false), arrays (lists), or other objects.

JSON is readable by humans and parseable by computers. It's like a well-organized spreadsheet that both people and machines understand.

Here's a more complex example—the kind of JSON FicHub's API might send back:

```json
{
  "status": "success",
  "story": {
    "title": "The Dragon's Apprentice",
    "author": "SilverQuill42",
    "summary": "When Harry discovers he has an unexpected connection to dragons...",
    "chapters": [
      {
        "number": 1,
        "title": "The Egg",
        "word_count": 3201
      },
      {
        "number": 2,
        "title": "Hatching",
        "word_count": 4102
      }
    ]
  },
  "download_url": "/api/download/abc123"
}
```

You'll be reading and writing a lot of JSON in this book. Don't worry—it looks complicated at first, but it becomes second nature quickly.

### What Is a Framework?

Let's go back to furniture.

You *could* build a chair from scratch. You'd need to harvest the wood, mill it, shape each piece, drill the holes, carve the joinery, and assemble it. It would take forever, and your first chair probably wouldn't be very comfortable.

Or you could buy a chair from IKEA. It comes with pre-cut pieces, instructions, and a design that's been refined through thousands of iterations. You still assemble it yourself, but the hard engineering is already done.

A **framework** is like an IKEA chair for web development.

SvelteKit is a framework. It provides:
- **File-based routing** — Create a file, and it automatically becomes a page on your website.
- **Server-side rendering** — Generate pages on the server for better performance.
- **API routes** — Build your backend API right alongside your frontend.
- **Build tools** — Package everything up for production.
- **Development server** — A local server that auto-reloads when you change code.

Without SvelteKit, you'd have to build all of this yourself. With SvelteKit, it's ready to go. You just add your own code on top.

There are many frameworks in web development:
- **React** (with Next.js) — Used by Facebook, Netflix, and many others.
- **Vue** (with Nuxt.js) — Popular for its gentle learning curve.
- **Angular** — Used by Google and large enterprise applications.
- **SvelteKit** — Our framework. Modern, fast, and developer-friendly.

We chose SvelteKit because it's excellent for learning. The code is clean, the concepts are clear, and it handles a lot of the complexity that used to trip up beginners.

> **Watch Out: Framework Fatigue**
>
> New JavaScript frameworks pop up every year. It can feel overwhelming. But here's the secret: the concepts are the same across frameworks. If you learn SvelteKit well, you'll understand React, Vue, or any other framework. The syntax changes; the ideas don't. Don't chase the shiny new thing—master the fundamentals.

### Putting It All Together: A Real Example

Let's trace through exactly what happens when someone visits FicHub and downloads a story. This is how every web application works—not just FicHub.

**The Setup:**

Imagine your friend, Alex, has just heard about FicHub from you. They open their phone's browser and type `fichub.com` into the address bar.

**Step 1: DNS Lookup — Finding the Server**

The first thing that happens is DNS (Domain Name System). The browser needs to know where `fichub.com` lives on the internet. It asks a DNS server (like a phone book) for the IP address of `fichub.com`. The DNS server responds: "That's at `192.168.1.100`" (or whatever the actual IP address is).

This is like calling directory assistance: "I need the phone number for FicHub." "Got it—it's 555-0123."

**Step 2: The Request Leaves Alex's Phone**

Alex's browser sends an HTTP request: "GET / HTTP/1.1" — which translates to "Please give me the homepage."

**Step 3: The Request Travels**

The request travels through Wi-Fi to the local router, through the internet service provider, across the internet backbone, and arrives at the server hosting FicHub. This takes milliseconds.

**Step 4: SvelteKit Responds**

FicHub's server receives the request. SvelteKit processes it, generates the HTML for the homepage, bundles the CSS and JavaScript, and sends it all back as a response with status code 200 OK.

**Step 5: Alex Sees the Page**

Alex's browser receives the response, parses the HTML, applies the CSS, executes the JavaScript, and displays the page. Alex sees the beautiful dark interface we described.

**Step 6: Alex Pastes a URL and Clicks Download**

Alex pastes a fanfiction URL into the input field and clicks "Download." FicHub's JavaScript sends a POST request to `/api/download` with the URL. This time, the request includes data.

**Step 7: The Backend Works**

The server receives the download request. The Rust backend kicks in: it validates the URL, connects to AO3, fetches the story content, parses the chapters, and creates an EPUB ebook file. This might take a few seconds for a long story.

**Step 8: The Response Returns**

The backend sends back a response: "Here's your download link: `/files/abc123.epub`."

**Step 9: Alex Downloads the Ebook**

FicHub's JavaScript updates the page to show a "Download Complete!" message with a link. Alex clicks it, and the EPUB file downloads to their phone.

**Step 10: Happy Reader**

Alex opens the EPUB on their e-reader and starts reading. The whole process, from pasting the URL to having the ebook, took about 10 seconds.

This entire flow—the DNS lookup, the request, the routing, the backend processing, the response, the file download—is what web development is. And you're going to build all of it.

### How FicHub Uses All of This

Let's put it all together:

1. **HTML** defines the structure of FicHub—the tabs, the input field, the buttons, the story cards.
2. **CSS** makes it look amazing—the dark theme, the smooth transitions, the responsive layout.
3. **JavaScript (TypeScript)** handles the interactivity—clicking tabs, submitting forms, displaying results.
4. **SvelteKit** is the framework that ties it all together and provides routing, server-side rendering, and API endpoints.
5. **The API** is the interface between the frontend and the backend.
6. **Rust** (the backend language) does the heavy lifting—fetching stories, creating ebooks, managing users.
7. **PostgreSQL** (the database) stores everything—story metadata, user preferences, download history.
8. **JSON** is how the frontend and backend talk to each other.

Every piece has a purpose. Every piece connects to the others. Understanding these connections is what separates someone who *uses* the web from someone who *builds* it.

### You Now Know More Than You Think

Take a breath. You just learned the fundamentals of how the entire web works. You understand the postal system of the internet, the three building blocks of web pages (HTML, CSS, JavaScript), what servers and APIs do, how requests and responses flow, what status codes mean, what JSON looks like, and what frameworks are for.

That's not a small amount of knowledge. That's the foundation that every web developer builds on.

### A Quick Vocabulary Guide

Before we move on, here's a quick reference of terms we've covered. Bookmark this page—you'll want to come back to it:

| Term | What It Is | Analogy |
|------|-----------|---------|
| **HTML** | Structure of a web page | The frame of a house |
| **CSS** | Styling and layout | Paint and furniture |
| **JavaScript** | Interactivity and behavior | Light switches and locks |
| **Web Server** | Computer that serves web pages | Restaurant kitchen |
| **API** | Interface for software communication | Restaurant menu |
| **HTTP** | Protocol for web communication | The postal system |
| **URL** | Address of a web page | Street address |
| **JSON** | Data format for APIs | Universal computer language |
| **Framework** | Pre-built code for common tasks | IKEA furniture |
| **npm** | Package manager for JavaScript | App store |
| **Git** | Version control system | Save points in a game |
| **Frontend** | What users see in the browser | The dining room |
| **Backend** | Server-side logic | The kitchen |
| **Database** | Where data is stored | Filing cabinet |
| **Status Code** | HTTP response indicator | Doctor's diagnosis |
| **DNS** | Translates domain names to IP addresses | Phone book |

You don't need to memorize this table. Just know it exists. When you encounter a term you've forgotten, come back and look it up.

Now let's set up your workshop and start building.

---

> **Try It Yourself: Inspect a Real API**
>
> Here's a fun experiment:
>
> 1. Open your browser and go to [https://httpbin.org/get](https://httpbin.org/get). This is a free testing tool for APIs. You'll see JSON data appear on your screen.
>
> 2. Now open your browser's Developer Tools (press F12 or right-click and select "Inspect").
>
> 3. Click the "Network" tab in the Developer Tools.
>
> 4. Reload the page. You'll see a request appear in the list.
>
> 5. Click on that request. Look at the "Headers" section. You'll see the status code (probably 200 OK), the response headers, and the JSON body.
>
> Congratulations—you just inspected a real API request! This is exactly the kind of thing you'll be doing throughout this book.

---

> **Watch Out: Don't Skip the Fundamentals**
>
> It's tempting to jump straight into building FicHub. Resist that temptation. The fundamentals in this chapter will save you hours of confusion later. When you encounter a 500 error, you'll know what it means. When you need to send data to an API, you'll know the format. When you're debugging CSS, you'll understand the box model.
>
> These basics are your compass. Keep them in mind.

---

## Chapter 3: Setting Up Your Workshop

Time to get our hands dirty. In this chapter, we're setting up your development environment—the tools you'll use every day to write, run, and test your code.

Think of this like setting up a workshop before starting a woodworking project. You need the right tools, organized and ready to go.

### Installing Node.js (The JavaScript Engine)

Before anything else, you need **Node.js**. Node.js is a runtime that lets you run JavaScript outside of a web browser. You'll use it for:
- Running SvelteKit's development server
- Installing packages (little bundles of code that other people wrote)
- Running build scripts
- Managing your project

**On macOS:**

1. Go to [https://nodejs.org](https://nodejs.org)
2. Click the big green button that says "LTS" (Long Term Support)
3. Run the installer
4. Follow the prompts (the defaults are fine)

**On Windows:**

1. Go to [https://nodejs.org](https://nodejs.org)
2. Click the LTS button
3. Run the `.msi` installer
4. Follow the prompts (accept the defaults)

**On Linux (Ubuntu/Debian):**

```bash
# Update your package list
sudo apt update

# Install Node.js and npm
sudo apt install -y nodejs npm
```

Or, for the latest version:

```bash
# Install nvm (Node Version Manager)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash

# Restart your terminal, then install Node.js
nvm install --lts
```

**On Arch Linux (like the system we're building on):**

```bash
# Install Node.js and npm
sudo pacman -S nodejs npm
```

After installation, verify it worked. Open your terminal and type:

```bash
node --version
# Should show something like: v20.11.0

npm --version
# Should show something like: 10.2.4
```

If you see version numbers, you're good to go.

> **Watch Out: Node.js Version**
>
> Make sure you install the LTS (Long Term Support) version. It's the stable, tested version that all tutorials assume you're using. The "Current" version has the newest features but might have bugs. Go with LTS.

### What Is npm?

**npm** stands for **Node Package Manager**. It's like an app store for JavaScript libraries.

When you need a piece of code that someone else already wrote—maybe a date formatting library, or an HTTP client, or a CSS framework—you don't write it yourself. You install it with npm:

```bash
npm install moment
```

This downloads the `moment` library (a popular date library) into your project. You can then use it in your code.

npm also manages your project's **dependencies**—the list of all the packages your project needs. We'll see this in action when we set up SvelteKit.

Here are some npm commands you'll use frequently:

```bash
# Install a package and save it to package.json
npm install axios

# Install a package used only for development
npm install -D @types/node

# Remove a package
npm uninstall axios

# Install all dependencies from package.json
npm install

# Run a script defined in package.json
npm run dev

# Search for packages on npm
npm search svelte-component
```

When you run `npm install`, npm reads `package.json`, finds all the dependencies listed there, and downloads them into a folder called `node_modules/`. This folder can get big—hundreds of megabytes of code. That's why `node_modules/` is always in `.gitignore` (we don't want to upload all those files to Git). Instead, we just commit `package.json` and `package-lock.json` (which records the exact versions). Anyone who clones the project can run `npm install` to recreate the exact same setup.

> **Watch Out: Don't Commit node_modules/**
>
> The `node_modules/` folder can contain thousands of files and hundreds of megabytes of code. Never commit it to Git. Your `.gitignore` file (which SvelteKit creates for you) already excludes it. If you accidentally commit it, you'll bloat your repository and slow everything down. To recover from this, remove it from Git tracking:
>
> ```bash
> git rm -r --cached node_modules
> git commit -m "Remove node_modules from tracking"
> ```

### Installing a Code Editor

You need a **code editor**—a program designed for writing code. Don't use Notepad or TextEdit. A good code editor gives you:
- Syntax highlighting (code is colored so you can read it easily)
- Auto-complete (the editor suggests what you're about to type)
- Error detection (catches mistakes as you type)
- Extensions (extra features you can add)
- Built-in terminal (run commands without switching windows)

**Visual Studio Code (VS Code)** is the most popular code editor, and it's what we'll use in this book. It's free, powerful, and has extensions for everything.

1. Go to [https://code.visualstudio.com](https://code.visualstudio.com)
2. Download and install it
3. Open VS Code

**Recommended Extensions:**

After installing VS Code, open it and click the Extensions icon (the four squares on the left sidebar, or press `Ctrl+Shift+X`). Install these:

- **Svelte** — Syntax highlighting and tools for Svelte code
- **ESLint** — Finds JavaScript/TypeScript errors
- **Prettier** — Formats your code automatically
- **Auto Rename Tag** — When you rename an HTML tag, it renames the closing tag too
- **GitLens** — Shows who last changed each line of code (invaluable for collaboration)

You can install them by searching in the Extensions panel and clicking "Install."

> **Try It Yourself: Customize Your Editor**
>
> Open VS Code settings (`Ctrl+,` on Windows/Linux, `Cmd+,` on Mac). Search for "Font Size" and set it to something comfortable—14px or 16px is good for most people. Also search for "Format On Save" and enable it. This will automatically clean up your code every time you save a file. Small comfort, big impact.

### The Terminal: Your Magic Wand

The **terminal** (also called the command line, console, or shell) is a text-based interface for your computer. Instead of clicking icons and menus, you type commands.

It might seem intimidating at first, but the terminal is your most powerful tool as a developer. Many things are faster and easier in the terminal. Some things are *only* possible in the terminal.

**Opening the terminal:**

- **macOS:** Press `Cmd+Space`, type "Terminal," and press Enter
- **Windows:** Press `Win+R`, type `cmd`, and press Enter (or search for "Command Prompt")
- **Linux:** Press `Ctrl+Alt+T` (on most distributions)

When you open the terminal, you'll see something like:

```
user@computer:~$
```

That `$` is the prompt. It's the terminal saying, "I'm ready for your command." Type a command and press Enter to run it.

### Basic Terminal Commands

Here are the essential commands you'll use every day. Don't try to memorize them all at once—just know they exist, and you'll pick them up naturally.

**`pwd` — Where Am I?**

`pwd` stands for "print working directory." It shows you where you are in your computer's file system.

```bash
pwd
# Output: /home/user
```

This is like asking, "What room am I in?" The answer tells you where you are.

**`ls` — What's Here?**

`ls` lists the files and folders in your current directory.

```bash
ls
# Output: Documents  Downloads  Music  Pictures  code
```

This is like opening a drawer and seeing what's inside.

**`cd` — Go Somewhere**

`cd` stands for "change directory." It moves you to a different folder.

```bash
cd code
# Now you're in the "code" folder

cd ..
# The ".." means "go up one level" — back to the parent folder

cd ~
# The "~" means "go to your home directory" — always takes you home
```

This is like walking from one room to another.

**`mkdir` — Make a Folder**

`mkdir` creates a new folder.

```bash
mkdir my-project
# Creates a folder called "my-project" in your current directory
```

This is like saying, "I need a new shelf in this room."

**`touch` — Create an Empty File**

```bash
touch hello.txt
# Creates an empty file called "hello.txt"
```

**`cat` — Read a File**

```bash
cat hello.txt
# Displays the contents of the file
```

**`rm` — Remove a File**

```bash
rm hello.txt
# Deletes the file (be careful!)
```

> **Watch Out: The Terminal is Unforgiving**
>
> When you delete a file in the terminal with `rm`, it's gone. There's no trash can, no "undo," no "are you sure?" It just disappears. Be careful with `rm`, especially with `rm -rf` (which forces deletion of entire directories). Triple-check your command before pressing Enter.

Let's practice. Try these commands in your terminal right now:

```bash
# See where you are
pwd

# List what's in your home directory
ls

# Go to your Documents folder
cd Documents

# Make a new folder for our project
mkdir fichub-project

# Go into the new folder
cd fichub-project

# See where you are now
pwd
# Should show something like: /home/user/Documents/fichub-project
```

> **Try It Yourself: Terminal Practice**
>
> Spend 5 minutes just practicing these commands. Navigate around your file system. Create a few folders. List their contents. Go up and down the directory tree. Get comfortable with the rhythm: type command, press Enter, see result. This muscle memory will serve you well.

### Installing SvelteKit

Now for the exciting part: creating our SvelteKit project. This is where it all begins.

SvelteKit has a handy command-line tool that sets everything up for you. Open your terminal and run:

```bash
npx sv create fichub
```

Here's what this command does:
- `npx` — Run a command from npm without installing it globally
- `sv create` — The SvelteKit project creation tool
- `fichub` — The name of our project (this becomes the folder name)

You'll see a series of prompts. Here's how to answer them:

```
Which SvelteKit template?
  ❯ Skeleton project
  SvelteKit demo app
  ──────────────────────
```

**Select "Skeleton project."** This gives us a clean starting point without any demo code we'd have to delete.

```
Add type checking with TypeScript?
  ❯ Yes, using TypeScript syntax
  JavaScript
  ──────────────────────────────
```

**Select "Yes, using TypeScript syntax."** TypeScript is our friend. It catches errors before they become bugs.

```
Select additional options:
  ❯ Add ESLint for code linting
  ❯ Add Prettier for code formatting
  ❯ Add Playwright for browser testing
```

**Select all three.** ESLint catches bad code patterns, Prettier formats your code, and Playwright lets you test your app in a real browser.

After a few moments (the tool downloads and installs dependencies), you'll see:

```
Your project is ready!
```

**`cd` into your new project:**

```bash
cd fichub
```

### Understanding the Project Structure

Let's look at what SvelteKit created for us:

```bash
ls -la
```

You'll see something like:

```
├── .gitignore
├── package.json
├── package-lock.json
├── svelte.config.js
├── tsconfig.json
├── vite.config.ts
├── src/
│   ├── app.html
│   ├── app.css
│   ├── lib/
│   │   └── ...
│   └── routes/
│       ├── +layout.svelte
│       ├── +page.svelte
│       └── +page.server.ts
├── static/
└── README.md
```

Don't worry about understanding every file. Let's focus on the important ones:

**`package.json`** — The shopping list for your project. It lists all the packages (dependencies) your project needs. Think of it like a recipe: "You need flour, sugar, eggs, and 2 cups of baking soda."

**`src/`** — The source code folder. This is where all your code lives.

**`src/routes/`** — The pages of your website. Each file here becomes a URL.

- `+page.svelte` — The main page (the homepage at `/`)
- `+layout.svelte` — The layout that wraps all pages (shared header, footer, etc.)
- `+page.server.ts` — Server-side code for the main page (data fetching, API logic)

**`static/`** — Static files like images, fonts, and icons. Anything that doesn't change.

**`svelte.config.js`** — Configuration for SvelteKit. You probably won't touch this much.

**`vite.config.ts`** — Configuration for Vite, the build tool SvelteKit uses. Again, you'll rarely need to change this.

### package.json Explained

Let's look at `package.json` more closely:

```json
{
  "name": "fichub",
  "version": "0.0.1",
  "private": true,
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview",
    "check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
    "check:watch": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json --watch",
    "lint": "prettier --check . && eslint .",
    "format": "prettier --write ."
  },
  "devDependencies": {
    "@sveltejs/adapter-auto": "^3.0.0",
    "@sveltejs/kit": "^2.0.0",
    "svelte": "^5.0.0",
    "typescript": "^5.0.0",
    "vite": "^6.0.0"
  }
}
```

Key sections:

- **`name`** — Your project's name
- **`version`** — The current version (starts at 0.0.1)
- **`scripts`** — Commands you can run. These are the buttons on your toolbox:
  - `npm run dev` — Start the development server
  - `npm run build` — Build the project for production
  - `npm run check` — Run TypeScript type checking
  - `npm run lint` — Check code formatting and style
  - `npm run format` — Auto-fix formatting issues
- **`devDependencies`** — Packages needed for development (not shipped to users)

The `scripts` section is especially important. When you type `npm run dev`, npm looks in the `scripts` section for a command called "dev" and runs it. It's a shortcut system.

### Running the Dev Server

Now let's fire up the development server and see our app in action.

In your terminal (make sure you're in the `fichub` folder):

```bash
npm run dev
```

You'll see something like:

```
  VITE v6.0.0  ready in 300 ms

  ➜  Local:   http://localhost:5173/
  ➜  Network: use --host to expose
  ➜  press h + enter to show help
```

This means your development server is running! It's listening on **port 5173** of your computer.

Open your browser and go to:

```
http://localhost:5173/
```

You should see the SvelteKit welcome page—a dark page with a Svelte logo, some text about getting started, and links to documentation.

**Congratulations!** You just created and ran your first SvelteKit application. Take a moment. That's a real accomplishment.

> **Try It Yourself: Explore Your App**
>
> 1. Look at the page in your browser. Click around on the links.
>
> 2. Go back to your code editor (VS Code). Open `src/routes/+page.svelte`.
>
> 3. Change some text. For example, find the heading and change it to "Welcome to FicHub."
>
> 4. Save the file. Watch the browser—your changes appear instantly! This is called **hot module replacement** or **hot reloading**. The dev server watches your files and updates the browser automatically.
>
> 5. Try changing a color in the CSS. See how it updates in real time.
>
> 6. Try breaking something on purpose—misspell a tag, leave out a closing bracket. See what error message you get. Learning from errors is just as valuable as learning from success.

### The Dev Server in Detail

When you run `npm run dev`, several things happen:

1. **Vite starts up** — Vite is the build tool (the "build system") that SvelteKit uses under the hood. It watches your files for changes.

2. **A local server starts** — Your computer becomes a web server, listening on port 5173. Only you can access it (that's what "localhost" means).

3. **Your code is compiled** — SvelteKit converts your Svelte components, TypeScript code, and routes into plain HTML, CSS, and JavaScript that browsers understand.

4. **Hot reloading begins** — Every time you save a file, Vite detects the change, recompiles only what changed, and pushes the update to your browser. No manual refresh needed.

The dev server is your constant companion during development. It's where you'll spend most of your time: write code, save, see changes, iterate.

> **Watch Out: Port Conflicts**
>
> If you see an error like "Port 5173 is already in use," it means another process is using that port. Either stop the other process or change the port:
>
> ```bash
> npm run dev -- --port 5174
> ```
>
> This starts the server on port 5174 instead.

### Understanding SvelteKit Files

Let's look at the files SvelteKit created more closely. Understanding what each one does will save you confusion later:

**`src/app.html`** — The shell of your HTML page. It's the document that wraps everything. You'll rarely change this, but it's where you'd add global things like `<meta>` tags or link to external stylesheets.

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <link rel="icon" href="%sveltekit.assets%/favicon.png" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    %sveltekit.head%
  </head>
  <body data-sveltekit-prerender="true">
    <div style="display: contents">%sveltekit.body%</div>
  </body>
</html>
```

The `%sveltekit.head%` and `%sveltekit.body%` are placeholders that SvelteKit fills in with your page content. Don't worry too much about this file—SvelteKit manages it for you.

**`src/routes/+page.svelte`** — This is your homepage. Everything in this file shows up when you visit `/`. It's a Svelte component—a self-contained piece of HTML, CSS, and JavaScript.

```svelte
<script>
  // JavaScript goes here
  let count = $state(0);
</script>

<h1>Welcome to FicHub</h1>
<p>Click the button: {count}</p>
<button onclick={() => count++}>Increment</button>

<style>
  /* CSS goes here */
  h1 {
    color: #e94560;
  }
</style>
```

Notice how everything is in one file: the logic (`<script>`), the markup (HTML), and the styles (`<style>`). That's the Svelte philosophy—co-locate everything that belongs together.

**`src/routes/+layout.svelte`** — This wraps every page on your site. Use it for things that appear on every page: a header, a footer, navigation, a global background.

**`src/routes/+page.server.ts`** — Server-side code. This runs on the server, not in the browser. It's where you fetch data from databases, call external APIs, or do anything that shouldn't happen on the client.

```typescript
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async () => {
  // This runs on the server
  const stories = await fetchStories();
  return { stories };
};
```

**`src/lib/`** — Shared code and components. Put reusable pieces here—buttons, cards, utility functions, API clients. Think of it as your project's toolbox.

```
src/lib/
├── components/
│   ├── Card.svelte
│   ├── Button.svelte
│   └── TabBar.svelte
├── api/
│   └── client.ts
└── stores/
    └── state.ts
```

### The File-Based Routing System

One of the coolest things about SvelteKit is how routing works. In most web frameworks, you have to manually define which URLs go to which pages. In SvelteKit, the file system *is* the router.

Here's how it works:

```
src/routes/
├── +page.svelte          →  / (the homepage)
├── about/
│   └── +page.svelte      →  /about
├── stories/
│   ├── +page.svelte      →  /stories
│   └── [id]/
│       └── +page.svelte  →  /stories/123
├── api/
│   └── download/
│       └── +server.ts    →  /api/download (API endpoint)
```

Create a file, and it becomes a page. Delete the file, and the page disappears. It's intuitive and powerful.

The square brackets `[id]` mean a dynamic parameter—the URL can be any value, and that value is available in your code as a parameter. So `/stories/abc`, `/stories/123`, and `/stories/my-favorite-story` all map to the same page, with different `id` values.

We'll use this extensively when building FicHub.

### Stopping the Dev Server

When you're done for the day, you can stop the dev server by pressing `Ctrl+C` in the terminal where it's running. This sends an "interrupt" signal that tells the server to shut down gracefully.

You can always restart it with `npm run dev`.

### Git: Keeping Track of Changes

You may have noticed a `.gitignore` file in your project. This means SvelteKit has already set up Git for you. Git is a version control system—it tracks changes to your code over time.

Think of Git like a time machine for your project. Every commit is a snapshot. If you mess something up, you can travel back to a previous snapshot and try again.

Here's a quick setup:

```bash
# Initialize git (SvelteKit may have already done this)
git init

# Make your first commit
git add .
git commit -m "Initial commit: SvelteKit project created"
```

From now on, after each significant change, you should commit:

```bash
git add .
git commit -m "Description of what you changed"
```

This creates a snapshot of your project. If something goes wrong, you can always go back to a previous snapshot. Think of it like a save point in a video game.

**Some essential Git commands:**

```bash
# See what files have changed
git status

# See the changes you've made
git diff

# Commit your changes
git add .
git commit -m "Your message here"

# See your commit history
git log --oneline

# Go back to a previous commit (don't worry, you can come back)
git checkout <commit-hash>

# Create a branch (a parallel timeline for experimenting)
git checkout -b feature/new-button

# Switch back to the main timeline
git checkout main
```

The `git checkout -b` command is especially useful. It creates a branch—a separate copy of your code where you can experiment without affecting the main version. When your experiment works, you merge it back. When it fails, you just delete the branch and nobody knows it happened.

> **Try It Yourself: Make a Commit**
>
> 1. Change the welcome message in `+page.svelte` (we did this earlier).
> 2. In your terminal: `git add .`
> 3. Then: `git commit -m "Change welcome message to FicHub"`
> 4. Now try `git log --oneline` to see your commit history.
> 5. Make another change—add a new paragraph or change a color.
> 6. Commit again with a different message.
> 7. Run `git log --oneline` to see both commits listed.
>
> You've just learned one of the most important habits in software development: committing often with descriptive messages.

### Your Development Workflow

Now that you have all the tools set up, here's the workflow you'll follow every time you sit down to code:

1. **Open your terminal** and navigate to your project:
   ```bash
   cd ~/Documents/fichub-project
   ```

2. **Start the dev server:**
   ```bash
   npm run dev
   ```

3. **Open VS Code** in a second window or split pane.

4. **Open your browser** to `http://localhost:5173/`.

5. **Write code** in VS Code. Save the file. See the changes in the browser.

6. **Commit often** with descriptive messages:
   ```bash
   git add .
   git commit -m "Add card component with dark theme"
   ```

This three-window setup—terminal, editor, browser—is how professional web developers work. The terminal runs your server, the editor is where you write code, and the browser shows the results. Over time, this workflow becomes second nature.

> **Watch Out: Save Your Work Frequently**
>
> Computers crash. Browsers close. Power goes out. Save your work often—both in your editor (Ctrl+S / Cmd+S) and to Git (git commit). A good rule of thumb: commit every 30 minutes or after every meaningful change. It takes 10 seconds and could save you hours of lost work.

### You're Ready to Build

Your workshop is set up. You have:
- ✅ Node.js and npm installed
- ✅ VS Code configured with extensions
- ✅ Terminal basics under your belt
- ✅ SvelteKit project created
- ✅ Dev server running and hot-reloading
- ✅ Git initialized with your first commits
- ✅ A workflow: terminal + editor + browser

That's everything you need to start coding. In the next chapter, we'll write our first real HTML and CSS—no framework magic, just the fundamentals.

### Quick Reference: Commands You'll Use Daily

Here's a cheat sheet of the most common commands. Print it out, pin it above your desk, or just bookmark this page:

```bash
# Navigate to your project
cd ~/Documents/fichub-project

# Start the dev server
npm run dev

# Install a new package
npm install <package-name>

# Install a dev-only package
npm install -D <package-name>

# Commit your work
git add .
git commit -m "Your message"

# Check your git status
git status

# See recent commits
git log --oneline

# Stop the dev server
# Press Ctrl+C in the terminal
```

You'll develop muscle memory for these commands. Within a week, you'll be typing them without thinking.

> **Watch Out: The "It Works on My Machine" Problem**
>
> As you develop, you might find that something works on your computer but not on someone else's. This happens because of different versions of Node.js, npm, or operating systems. The solution is to commit your `package-lock.json` file (which records exact versions) and to keep your tools up to date. When in doubt, ask your teammate: "What version of Node.js are you running?" (`node --version`).

---

> **Watch Out: Don't Skip the Git Step**
>
> It's tempting to skip Git setup because it doesn't seem important right now. Trust me: the first time your code breaks and you can't figure out why, you'll wish you had committed earlier. Make it a habit now. Every hour, every few meaningful changes: `git add . && git commit -m "description"`. Future you will be grateful.

---

## Chapter 4: Your First Web Page with HTML and CSS

Time to build something real. In this chapter, we're writing actual HTML and CSS—not in SvelteKit's framework, but in the raw, fundamental way the web works. Understanding the basics before adding framework magic will make you a better developer.

We're going to build a **card component**—a small, styled box with a title, some text, and a button. It's simple, but it teaches you everything you need to know about HTML structure and CSS styling.

### HTML Elements: The Building Blocks

Let's review the core HTML elements you'll use constantly. Each one has a specific purpose:

```html
<!-- Headings: h1 is the biggest, h6 is the smallest -->
<h1>Main Title</h1>
<h2>Section Title</h2>
<h3>Subsection Title</h3>

<!-- Paragraphs: blocks of text -->
<p>This is a paragraph of text. Browsers add space between paragraphs automatically.</p>

<!-- Links: clickable text that goes somewhere -->
<a href="https://example.com">Click here to visit Example.com</a>

<!-- Buttons: clickable elements -->
<button>Click me!</button>

<!-- Inputs: text fields for user input -->
<input type="text" placeholder="Type something...">
<input type="email" placeholder="your@email.com">

<!-- Divs: invisible containers that group elements -->
<div>
  <h2>Grouped together</h2>
  <p>These elements are inside a div.</p>
</div>

<!-- Images -->
<img src="photo.jpg" alt="A description of the image">

<!-- Lists -->
<ul>
  <li>First item</li>
  <li>Second item</li>
</ul>

<!-- Spans: inline containers for styling parts of text -->
<p>This word is <span class="highlight">important</span>.</p>
```

Key concepts:
- **Block elements** (h1, p, div) take up a full line width. They stack vertically.
- **Inline elements** (a, span, img) only take up as much width as they need. They sit side by side.
- **Attributes** add extra information to elements. `href` tells a link where to go. `src` tells an image where its file is. `class` gives an element a name for CSS targeting.

### CSS Selectors: Targeting Elements

CSS selectors are how you tell the browser which elements to style. Here are the three most common:

**Element Selector:** Targets all elements of a type.

```css
/* Style all paragraphs */
p {
  color: blue;
}

/* Style all h1 elements */
h1 {
  font-size: 32px;
}
```

**Class Selector:** Targets elements with a specific class name. Classes start with a dot (`.`).

```html
<p class="intro">This paragraph is special.</p>
<p>This paragraph is normal.</p>
```

```css
/* Only style elements with class="intro" */
.intro {
  color: green;
  font-weight: bold;
}
```

**ID Selector:** Targets a single, unique element. IDs start with a hash (`#`).

```html
<h1 id="main-title">Welcome to FicHub</h1>
```

```css
/* Only style the one element with id="main-title" */
#main-title {
  color: #e94560;
  text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.3);
}
```

> **Watch Out: Use Classes More Than IDs**
>
> In practice, you'll use classes much more than IDs. Classes can be reused—many elements can share the same class. IDs are unique—only one element should have a given ID. Most styling belongs on classes. Save IDs for JavaScript targeting or unique, one-off elements.

### The Box Model: Everything Is a Box

This is one of the most important concepts in CSS. Every element on a web page is a rectangular box. Even a round button is a rectangular box with rounded corners. And every box has four layers:

```
┌─────────────────────────────┐
│           margin            │  ← Space outside the border
│  ┌───────────────────────┐  │
│  │        border         │  │  ← The visible edge
│  │  ┌─────────────────┐  │  │
│  │  │     padding     │  │  │  ← Space between content and border
│  │  │  ┌───────────┐  │  │  │
│  │  │  │  content   │  │  │  │  ← The actual text or image
│  │  │  └───────────┘  │  │  │
│  │  └─────────────────┘  │  │
│  └───────────────────────┘  │
└─────────────────────────────┘
```

- **Content** — The text, image, or whatever is inside the element.
- **Padding** — Space between the content and the border. Like the padding inside a picture frame.
- **Border** — A visible line around the element. Can be thin, thick, dashed, solid, or any style.
- **Margin** — Space outside the border. Like the distance between two picture frames on a wall.

Here's how you control these in CSS:

```css
.card {
  /* Padding: 20px on all sides */
  padding: 20px;

  /* Border: 1px solid, dark gray */
  border: 1px solid #333;

  /* Margin: 16px on all sides */
  margin: 16px;

  /* The content width is 300px (padding and border are extra) */
  width: 300px;
}
```

If you set `width: 300px` and `padding: 20px`, the total visible width is actually 340px (300 + 20 left + 20 right). This catches beginners off guard.

> **Try It Yourself: The Box Model in Action**
>
> 1. Create a file called `boxmodel.html` in your project folder.
> 2. Add this code:
>
> ```html
> <!DOCTYPE html>
> <html>
> <head>
>   <style>
>     .box {
>       background-color: #e94560;
>       color: white;
>       padding: 20px;
>       margin: 10px;
>       border: 3px solid #ff6b81;
>       width: 200px;
>     }
>   </style>
> </head>
> <body>
>   <div class="box">Hello, box model!</div>
>   <div class="box">Another box!</div>
> </body>
> </html>
> ```
>
> 3. Open it in your browser (double-click the file or drag it into your browser).
> 4. Now change the padding to 40px. See how the box grows?
> 5. Change the margin to 30px. See how the space between boxes increases?
>
> This is the box model in action. Every element you ever style will follow these rules.

### Flexbox: Arranging Things in a Row

**Flexbox** is CSS's superpower for layout. It lets you arrange elements in a row, column, or any configuration you want—without weird hacks.

Imagine you have three boxes and you want them in a row, evenly spaced. With Flexbox, it's one property:

```css
.container {
  display: flex;
  justify-content: space-between;
}
```

That's it. Three boxes, in a row, with equal space between them.

Let's try a more complete example:

```html
<!DOCTYPE html>
<html>
<head>
  <style>
    .row {
      display: flex;
      gap: 16px;           /* Space between items */
      justify-content: center;  /* Center items horizontally */
      align-items: center;      /* Center items vertically */
    }

    .card {
      background-color: #16213e;
      color: white;
      padding: 24px;
      border-radius: 8px;
      width: 200px;
      text-align: center;
    }
  </style>
</head>
<body>
  <div class="row">
    <div class="card">Card 1</div>
    <div class="card">Card 2</div>
    <div class="card">Card 3</div>
  </div>
</body>
</html>
```

The key Flexbox properties:

```css
.container {
  display: flex;              /* Turn on Flexbox */

  /* Horizontal alignment */
  justify-content: flex-start;    /* Items to the left */
  justify-content: flex-end;      /* Items to the right */
  justify-content: center;        /* Items in the middle */
  justify-content: space-between; /* Equal space between items */
  justify-content: space-around;  /* Space around each item */

  /* Vertical alignment (for rows) */
  align-items: stretch;      /* Fill the container's height */
  align-items: flex-start;   /* Align to the top */
  align-items: flex-end;     /* Align to the bottom */
  align-items: center;       /* Align to the middle */

  /* Wrap to new lines if needed */
  flex-wrap: wrap;

  /* Gap between items */
  gap: 16px;
}
```

For a single item inside a flex container:

```css
.item {
  flex: 1;              /* Grow to fill available space */
  flex-grow: 1;         /* How much to grow */
  flex-shrink: 0;       /* How much to shrink */
  flex-basis: 200px;    /* Starting width before growing/shrinking */
}
```

Flexbox is the single most useful CSS concept you'll learn. It solves 90% of layout problems. We'll use it extensively in FicHub.

### CSS Variables: Reusable Colors and Values

**CSS variables** (also called custom properties) let you define a value once and reuse it everywhere. This is incredibly useful for maintaining a consistent theme.

```css
:root {
  /* Define your theme colors */
  --color-primary: #e94560;
  --color-secondary: #0f3460;
  --color-bg: #1a1a2e;
  --color-text: #eaeaea;
  --color-border: #333333;

  /* Define spacing */
  --space-sm: 8px;
  --space-md: 16px;
  --space-lg: 24px;
  --space-xl: 32px;

  /* Define font sizes */
  --font-sm: 14px;
  --font-md: 16px;
  --font-lg: 20px;
  --font-xl: 28px;

  /* Define border radius */
  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-lg: 12px;
}

/* Use the variables throughout your CSS */
body {
  background-color: var(--color-bg);
  color: var(--color-text);
  font-size: var(--font-md);
}

h1 {
  color: var(--color-primary);
  font-size: var(--font-xl);
}

.card {
  border: 1px solid var(--color-border);
  padding: var(--space-lg);
  border-radius: var(--radius-md);
  margin-bottom: var(--space-md);
}

button {
  background-color: var(--color-primary);
  color: white;
  padding: var(--space-sm) var(--space-md);
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
}
```

When you want to change the theme, you change the variables in one place (`:root`), and everything updates automatically. It's like having a master light switch that controls every light in the house.

> **Try It Yourself: Theme Switching**
>
> 1. Create a file called `variables.html`.
> 2. Use the CSS variables from above.
> 3. Now change `--color-primary` from red to blue.
> 4. Reload the page. Notice how every element using `--color-primary` changed at once.
> 5. Try changing `--color-bg` to a light color. You'll need to change `--color-text` to a dark color too (otherwise you'll have dark text on a dark background).
>
> This is the power of CSS variables. One change, everywhere updates.

### Building a Real Component: The FicHub Card

Let's put it all together and build a real component—a card that looks like something you'd see on FicHub. This card will have a title, author name, a short description, tags, and a download button.

Create a file called `card.html` and add this code:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>FicHub Card Component</title>
  <style>
    /* CSS Variables: the FicHub theme */
    :root {
      --color-primary: #e94560;
      --color-primary-hover: #ff6b81;
      --color-secondary: #0f3460;
      --color-bg: #1a1a2e;
      --color-card-bg: #16213e;
      --color-text: #eaeaea;
      --color-text-muted: #8899aa;
      --color-border: #333333;
      --color-tag-bg: #0f3460;

      --space-xs: 4px;
      --space-sm: 8px;
      --space-md: 16px;
      --space-lg: 24px;
      --space-xl: 32px;

      --font-sm: 14px;
      --font-md: 16px;
      --font-lg: 20px;
      --font-xl: 28px;

      --radius-sm: 4px;
      --radius-md: 8px;
      --radius-lg: 12px;
    }

    /* Reset: remove default browser styles */
    * {
      margin: 0;
      padding: 0;
      box-sizing: border-box;
    }

    body {
      background-color: var(--color-bg);
      color: var(--color-text);
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      padding: var(--space-xl);
    }

    /* The card component */
    .card {
      background-color: var(--color-card-bg);
      border: 1px solid var(--color-border);
      border-radius: var(--radius-lg);
      padding: var(--space-lg);
      max-width: 400px;
      transition: transform 0.2s ease, box-shadow 0.2s ease;
    }

    .card:hover {
      transform: translateY(-2px);
      box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
    }

    .card-title {
      font-size: var(--font-lg);
      font-weight: 600;
      margin-bottom: var(--space-xs);
    }

    .card-author {
      font-size: var(--font-sm);
      color: var(--color-text-muted);
      margin-bottom: var(--space-md);
    }

    .card-summary {
      font-size: var(--font-md);
      line-height: 1.6;
      margin-bottom: var(--space-md);
    }

    .card-tags {
      display: flex;
      flex-wrap: wrap;
      gap: var(--space-sm);
      margin-bottom: var(--space-lg);
    }

    .tag {
      background-color: var(--color-tag-bg);
      color: var(--color-text-muted);
      font-size: var(--font-sm);
      padding: var(--space-xs) var(--space-sm);
      border-radius: var(--radius-sm);
    }

    .card-meta {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: var(--space-md);
      font-size: var(--font-sm);
      color: var(--color-text-muted);
    }

    .card-rating {
      background-color: var(--color-primary);
      color: white;
      padding: var(--space-xs) var(--space-sm);
      border-radius: var(--radius-sm);
      font-weight: 600;
    }

    .download-btn {
      width: 100%;
      padding: var(--space-md);
      background-color: var(--color-primary);
      color: white;
      border: none;
      border-radius: var(--radius-md);
      font-size: var(--font-md);
      font-weight: 600;
      cursor: pointer;
      transition: background-color 0.2s ease;
    }

    .download-btn:hover {
      background-color: var(--color-primary-hover);
    }
  </style>
</head>
<body>
  <div class="card">
    <div class="card-title">The Dragon's Apprentice</div>
    <div class="card-author">by SilverQuill42</div>

    <div class="card-meta">
      <span>87,432 words</span>
      <span class="card-rating">Mature</span>
    </div>

    <div class="card-summary">
      When Harry discovers he has an unexpected connection to dragons, his
      seventh year at Hogwarts takes a turn nobody expected. With Draco
      Malfoy as his reluctant ally, he must navigate prophecy, politics,
      and the fire in his veins.
    </div>

    <div class="card-tags">
      <span class="tag">fantasy</span>
      <span class="tag">romance</span>
      <span class="tag">slow burn</span>
      <span class="tag">dragons</span>
    </div>

    <button class="download-btn">Download EPUB</button>
  </div>
</body>
</html>
```

Open this file in your browser. You should see a beautiful, dark-themed card with:
- A title and author name
- Word count and rating badge
- A summary paragraph
- Tags in styled pills
- A download button

That's real, production-quality HTML and CSS. Everything we've learned—the box model, flexbox, CSS variables, selectors—comes together in this one component.

### CSS Grid: Two-Dimensional Layouts

While Flexbox handles one-dimensional layouts (rows OR columns), **CSS Grid** handles two-dimensional layouts (rows AND columns). Grid is perfect for page layouts and card grids.

```css
.page-layout {
  display: grid;
  grid-template-columns: 250px 1fr 250px;  /* Three columns */
  grid-template-rows: auto 1fr auto;        /* Three rows */
  gap: 16px;
  min-height: 100vh;
}

.page-header {
  grid-column: 1 / -1;  /* Spans all columns */
}

.page-sidebar-left {
  grid-column: 1;
}

.page-main {
  grid-column: 2;
}

.page-sidebar-right {
  grid-column: 3;
}

.page-footer {
  grid-column: 1 / -1;  /* Spans all columns */
}
```

This creates a classic three-column layout: sidebar, main content, sidebar. The header and footer span the full width. It's the same pattern used by most websites.

For card grids (like what we built in FicHub), Grid is perfect:

```css
.card-grid {
  display: grid;
  /* Create as many columns as will fit, each at least 300px */
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 24px;
}
```

That single line of `grid-template-columns` creates a responsive grid that:
- Has at least 300px per column
- Fills available space evenly
- Wraps to fewer columns on smaller screens
- Requires no media queries for basic responsiveness

### Responsive Design: Making It Work on Phones

**Responsive design** means your website looks good on every screen size—desktop, tablet, and phone. It's not about making a separate mobile version. It's about making one version that adapts.

The key tool for this is **media queries**. A media query says, "When the screen is this wide, apply these styles."

```css
/* Default styles (for large screens) */
.card {
  max-width: 400px;
  padding: var(--space-lg);
}

/* When the screen is smaller than 600px wide */
@media (max-width: 600px) {
  .card {
    max-width: 100%;  /* Fill the whole width */
    padding: var(--space-md);
  }

  .card-title {
    font-size: var(--font-lg);
  }

  .card-summary {
    font-size: var(--font-sm);
  }
}
```

Think of media queries like automatic doors. When you walk up (the screen gets smaller), they open (new styles activate). When you walk away (the screen gets bigger), they close (default styles return).

Here's a more complete responsive example. Add this to the `<style>` section of your `card.html`:

```css
/* Multiple cards in a grid on larger screens */
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: var(--space-lg);
  max-width: 1200px;
  margin: 0 auto;
}

/* On medium screens (tablets), 2 columns */
@media (max-width: 900px) {
  .card-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

/* On small screens (phones), 1 column */
@media (max-width: 600px) {
  .card-grid {
    grid-template-columns: 1fr;
  }
}
```

Now let's update the HTML to use this grid:

```html
<body>
  <div class="card-grid">
    <div class="card">
      <div class="card-title">The Dragon's Apprentice</div>
      <div class="card-author">by SilverQuill42</div>
      <div class="card-meta">
        <span>87,432 words</span>
        <span class="card-rating">Mature</span>
      </div>
      <div class="card-summary">
        When Harry discovers he has an unexpected connection to dragons...
      </div>
      <div class="card-tags">
        <span class="tag">fantasy</span>
        <span class="tag">romance</span>
        <span class="tag">slow burn</span>
      </div>
      <button class="download-btn">Download EPUB</button>
    </div>

    <div class="card">
      <div class="card-title">Midnight at the Ministry</div>
      <div class="card-author">by NightWriter99</div>
      <div class="card-meta">
        <span>52,100 words</span>
        <span class="card-rating">Teen</span>
      </div>
      <div class="card-summary">
        Hermione stumbles upon a secret department in the Ministry that
        shouldn't exist. What she finds there changes everything she
        knows about magic.
      </div>
      <div class="card-tags">
        <span class="tag">mystery</span>
        <span class="tag">adventure</span>
        <span class="tag">HERMIONE WINS</span>
      </div>
      <button class="download-btn">Download EPUB</button>
    </div>

    <div class="card">
      <div class="card-title">Through the Wardrobe (Again)</div>
      <div class="card-author">by NarniaNerd</div>
      <div class="card-meta">
        <span>34,800 words</span>
        <span class="card-rating">General</span>
      </div>
      <div class="card-summary">
        The Pevensie children are all grown up, but the wardrobe still
        works. A new adventure in Narnia awaits the next generation.
      </div>
      <div class="card-tags">
        <span class="tag">crossover</span>
        <span class="tag">adventure</span>
        <span class="tag">nostalgia</span>
      </div>
      <button class="download-btn">Download EPUB</button>
    </div>
  </div>
</body>
```

Now you have a three-column grid of cards on desktop, two columns on tablet, and one column on phone. Resize your browser window to see the responsive layout in action.

> **Try It Yourself: Test on Different Screen Sizes**
>
> 1. Open your `card.html` file in the browser.
> 2. Make the browser window really wide. You should see three cards in a row.
> 3. Slowly narrow the window. At around 900px, the cards should rearrange into two columns.
> 4. Narrow it further. Below 600px, you should see one card per row—like a phone screen.
> 5. In Chrome, press `F12` to open Developer Tools, then click the "Toggle Device Toolbar" icon (looks like a phone and tablet). You can now see exactly how your page looks on different phone models.
>
> This is how professional web developers test responsive design. The browser's built-in tools simulate different screen sizes.

### CSS Properties You'll Use Constantly

Before we wrap up, here are the CSS properties you'll use most often. Bookmark this list—you'll come back to it:

```css
/* Typography */
font-size: 16px;           /* How big the text is */
font-weight: bold;         /* How thick the text is */
line-height: 1.6;          /* Space between lines of text */
text-align: center;        /* Horizontal alignment */
text-decoration: none;     /* Remove underlines from links */

/* Colors */
color: #ffffff;            /* Text color */
background-color: #1a1a2e; /* Background color */
opacity: 0.8;             /* Transparency (1 = solid, 0 = invisible) */

/* Spacing */
padding: 16px;             /* Space inside the border */
margin: 16px;              /* Space outside the border */
gap: 16px;                 /* Space between flex/grid items */

/* Layout */
display: flex;             /* Flexbox layout */
display: grid;             /* Grid layout */
display: none;             /* Hide the element */
position: fixed;           /* Fix to the viewport */
position: absolute;        /* Fix relative to parent */

/* Sizing */
width: 100px;              /* Fixed width */
max-width: 400px;          /* Maximum width */
height: 100px;             /* Fixed height */

/* Appearance */
border: 1px solid #333;    /* Border */
border-radius: 8px;        /* Rounded corners */
box-shadow: 0 4px 8px rgba(0,0,0,0.2); /* Shadow */

/* Animation */
transition: all 0.3s ease; /* Smooth transitions */
transform: translateY(-2px); /* Move, rotate, or scale */
```

### Understanding CSS Specificity

When two CSS rules target the same element with different values, which one wins? CSS has a specificity system—like a hierarchy of importance.

The order of specificity (from lowest to highest):

1. **Element selectors** (`p`, `div`, `h1`) — specificity: 1
2. **Class selectors** (`.card`, `.btn`) — specificity: 10
3. **ID selectors** (`#main-title`) — specificity: 100
4. **Inline styles** (`style="..."`) — specificity: 1000
5. **`!important`** — overrides everything (avoid this)

```css
/* Specificity: 1 (element) */
p {
  color: blue;
}

/* Specificity: 10 (class) - WINS */
.special-text {
  color: red;
}

/* Specificity: 100 (id) - WINS over both */
#unique-text {
  color: green;
}
```

> **Watch Out: Don't Use !important**
>
> The `!important` declaration overrides all specificity rules. It's tempting when you can't get a style to apply, but it creates a maintenance nightmare. If every rule uses `!important`, nothing is predictable. Fix specificity properly instead: add a more specific selector.

> **Try It Yourself: Specificity Quiz**
>
> What color will the paragraph be? Don't run it—think first, then check:
>
> ```html
> <p id="main" class="text intro">What color am I?</p>
> ```
>
> ```css
> p { color: red; }
> .text { color: blue; }
> #main { color: green; }
> .intro { color: yellow; }
> ```
>
> The answer is **green**. The `#main` selector has the highest specificity (100), so it wins. The class selectors tie at 10, but since there are two of them, the last one (`.intro`) would win over `.text` if there were no ID. But the ID beats them all.

### The Browser Developer Tools

Every modern browser has built-in developer tools that are invaluable for web development. Let's learn how to use them.

**Opening Developer Tools:**

- Press `F12` (Windows/Linux) or `Cmd+Option+I` (macOS)
- Or right-click anywhere on the page and select "Inspect" or "Inspect Element"

**The Elements Tab:**

This is the most important tab. It shows you the HTML structure of the page—the actual DOM (Document Object Model). You can:

1. **Click on elements** to see their CSS styles
2. **Edit HTML** directly in the browser (changes are temporary)
3. **Toggle classes** on and off to see their effect
4. **Hover over elements** in the HTML to highlight them on the page

Try it now: open any website, open Developer Tools, go to the Elements tab, and click around. You'll see the HTML on the left and the CSS on the right. Hover over different elements and watch them highlight on the page.

**The Console:**

The console is where you can run JavaScript code directly. Type any JavaScript expression and press Enter:

```javascript
> 2 + 2
4

> document.title
"Google"

> document.querySelector('h1').textContent
"Welcome to FicHub"
```

You'll use the console constantly for debugging—printing variables, testing expressions, and checking for errors.

**The Network Tab:**

This tab shows every request your browser makes—every image, script, stylesheet, and API call. It's essential for debugging API issues:

1. Open the Network tab
2. Reload the page
3. You'll see a waterfall of requests
4. Click on any request to see details: headers, response, timing

If your API call isn't working, the Network tab will show you exactly what's being sent and what's coming back.

**The Styles Panel:**

When you inspect an element, the Styles panel shows you every CSS rule that applies to it, in order of specificity. Rules that are crossed out have been overridden by more specific rules. This is incredibly useful for debugging CSS.

> **Try It Yourself: Debug with DevTools**
>
> 1. Open the card component we built earlier (`card.html`).
> 2. Right-click on the card title and select "Inspect."
> 3. In the Elements tab, look at the CSS rules on the right.
> 4. Find the `.card-title` rule. Change the color directly in the Styles panel.
> 5. Find the `padding` value and change it. Watch the card grow.
> 6. Add a new property: type `border: 2px solid red;` in the styles for `.card`. See the red border appear.
>
> These changes are temporary—they reset when you refresh the page. But they're perfect for experimenting with styles before committing them to your code.

### Practice: Build a Complete Card Component

Let's practice everything by building a complete, polished card component. This time, we'll add more details and make it look production-ready.

Create `practice-card.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>FicHub Card - Practice</title>
  <style>
    :root {
      --color-primary: #e94560;
      --color-primary-hover: #ff6b81;
      --color-bg: #0d1117;
      --color-card-bg: #161b22;
      --color-card-border: #30363d;
      --color-text: #c9d1d9;
      --color-text-bright: #f0f6fc;
      --color-text-muted: #8b949e;
      --color-success: #3fb950;
      --color-warning: #d29922;
      --color-info: #58a6ff;

      --space-xs: 4px;
      --space-sm: 8px;
      --space-md: 12px;
      --space-lg: 16px;
      --space-xl: 24px;
      --space-xxl: 32px;

      --font-sm: 12px;
      --font-md: 14px;
      --font-lg: 18px;
      --font-xl: 24px;

      --radius-sm: 6px;
      --radius-md: 8px;
      --radius-lg: 12px;
    }

    * {
      margin: 0;
      padding: 0;
      box-sizing: border-box;
    }

    body {
      background-color: var(--color-bg);
      color: var(--color-text);
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      line-height: 1.5;
      padding: var(--space-xxl);
    }

    .page-title {
      text-align: center;
      color: var(--color-text-bright);
      font-size: var(--font-xl);
      margin-bottom: var(--space-xxl);
    }

    .page-subtitle {
      text-align: center;
      color: var(--color-text-muted);
      font-size: var(--font-md);
      margin-bottom: var(--space-xxl);
    }

    /* Card Grid */
    .card-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
      gap: var(--space-lg);
      max-width: 1100px;
      margin: 0 auto;
    }

    /* Card */
    .card {
      background-color: var(--color-card-bg);
      border: 1px solid var(--color-card-border);
      border-radius: var(--radius-lg);
      overflow: hidden;
      transition: transform 0.2s ease, box-shadow 0.2s ease, border-color 0.2s ease;
    }

    .card:hover {
      transform: translateY(-3px);
      box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
      border-color: var(--color-primary);
    }

    /* Card Header */
    .card-header {
      padding: var(--space-lg) var(--space-lg) 0;
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
    }

    .card-title {
      font-size: var(--font-lg);
      font-weight: 600;
      color: var(--color-text-bright);
      line-height: 1.3;
    }

    .card-rating {
      font-size: var(--font-sm);
      font-weight: 600;
      padding: var(--space-xs) var(--space-sm);
      border-radius: var(--radius-sm);
      white-space: nowrap;
    }

    .rating-mature {
      background-color: rgba(233, 69, 96, 0.2);
      color: var(--color-primary);
      border: 1px solid var(--color-primary);
    }

    .rating-teen {
      background-color: rgba(210, 153, 34, 0.2);
      color: var(--color-warning);
      border: 1px solid var(--color-warning);
    }

    .rating-general {
      background-color: rgba(63, 185, 80, 0.2);
      color: var(--color-success);
      border: 1px solid var(--color-success);
    }

    /* Card Body */
    .card-body {
      padding: var(--space-md) var(--space-lg);
    }

    .card-author {
      font-size: var(--font-sm);
      color: var(--color-text-muted);
      margin-bottom: var(--space-sm);
    }

    .card-author span {
      color: var(--color-info);
    }

    .card-meta {
      display: flex;
      gap: var(--space-lg);
      font-size: var(--font-sm);
      color: var(--color-text-muted);
      margin-bottom: var(--space-md);
    }

    .card-meta-item {
      display: flex;
      align-items: center;
      gap: var(--space-xs);
    }

    .card-summary {
      font-size: var(--font-md);
      color: var(--color-text);
      line-height: 1.6;
      display: -webkit-box;
      -webkit-line-clamp: 3;
      -webkit-box-orient: vertical;
      overflow: hidden;
    }

    /* Tags */
    .card-tags {
      padding: 0 var(--space-lg);
      display: flex;
      flex-wrap: wrap;
      gap: var(--space-sm);
      margin-bottom: var(--space-md);
    }

    .tag {
      font-size: var(--font-sm);
      padding: var(--space-xs) var(--space-sm);
      background-color: rgba(88, 166, 255, 0.1);
      color: var(--color-info);
      border-radius: var(--radius-sm);
      border: 1px solid rgba(88, 166, 255, 0.2);
    }

    /* Card Footer */
    .card-footer {
      padding: var(--space-md) var(--space-lg) var(--space-lg);
      border-top: 1px solid var(--color-card-border);
      display: flex;
      gap: var(--space-sm);
    }

    .btn {
      flex: 1;
      padding: var(--space-md);
      border: none;
      border-radius: var(--radius-md);
      font-size: var(--font-md);
      font-weight: 600;
      cursor: pointer;
      transition: background-color 0.2s ease, transform 0.1s ease;
    }

    .btn:active {
      transform: scale(0.98);
    }

    .btn-primary {
      background-color: var(--color-primary);
      color: white;
    }

    .btn-primary:hover {
      background-color: var(--color-primary-hover);
    }

    .btn-secondary {
      background-color: transparent;
      color: var(--color-text-muted);
      border: 1px solid var(--color-card-border);
    }

    .btn-secondary:hover {
      background-color: rgba(255, 255, 255, 0.05);
      color: var(--color-text);
    }

    /* Responsive */
    @media (max-width: 768px) {
      body {
        padding: var(--space-lg);
      }

      .card-grid {
        grid-template-columns: 1fr;
      }

      .page-title {
        font-size: var(--font-lg);
      }
    }

    @media (max-width: 480px) {
      .card-header {
        flex-direction: column;
        gap: var(--space-sm);
      }

      .card-rating {
        align-self: flex-start;
      }

      .card-footer {
        flex-direction: column;
      }
    }
  </style>
</head>
<body>
  <h1 class="page-title">📖 FicHub — Your Fanfiction Library</h1>
  <p class="page-subtitle">Download your favorite stories as ebooks</p>

  <div class="card-grid">
    <!-- Card 1 -->
    <div class="card">
      <div class="card-header">
        <div class="card-title">The Dragon's Apprentice</div>
        <span class="card-rating rating-mature">Mature</span>
      </div>
      <div class="card-body">
        <div class="card-author">by <span>SilverQuill42</span></div>
        <div class="card-meta">
          <span class="card-meta-item">📄 87,432 words</span>
          <span class="card-meta-item">📖 24 chapters</span>
          <span class="card-meta-item">✅ Complete</span>
        </div>
        <div class="card-summary">
          When Harry discovers he has an unexpected connection to dragons, his
          seventh year at Hogwarts takes a turn nobody expected. With Draco
          Malfoy as his reluctant ally, he must navigate prophecy, politics,
          and the fire in his veins.
        </div>
      </div>
      <div class="card-tags">
        <span class="tag">fantasy</span>
        <span class="tag">romance</span>
        <span class="tag">slow burn</span>
        <span class="tag">dragons</span>
      </div>
      <div class="card-footer">
        <button class="btn btn-primary">📥 Download EPUB</button>
        <button class="btn btn-secondary">↗ View on AO3</button>
      </div>
    </div>

    <!-- Card 2 -->
    <div class="card">
      <div class="card-header">
        <div class="card-title">Midnight at the Ministry</div>
        <span class="card-rating rating-teen">Teen</span>
      </div>
      <div class="card-body">
        <div class="card-author">by <span>NightWriter99</span></div>
        <div class="card-meta">
          <span class="card-meta-item">📄 52,100 words</span>
          <span class="card-meta-item">📖 18 chapters</span>
          <span class="card-meta-item">✅ Complete</span>
        </div>
        <div class="card-summary">
          Hermione stumbles upon a secret department in the Ministry that
          shouldn't exist. What she finds there changes everything she
          knows about magic—and about the war she thought was over.
        </div>
      </div>
      <div class="card-tags">
        <span class="tag">mystery</span>
        <span class="tag">adventure</span>
        <span class="tag">Hermione-centric</span>
      </div>
      <div class="card-footer">
        <button class="btn btn-primary">📥 Download EPUB</button>
        <button class="btn btn-secondary">↗ View on AO3</button>
      </div>
    </div>

    <!-- Card 3 -->
    <div class="card">
      <div class="card-header">
        <div class="card-title">The Weasley Family Road Trip</div>
        <span class="card-rating rating-general">General</span>
      </div>
      <div class="card-body">
        <div class="card-author">by <span>MuggleMagic</span></div>
        <div class="card-meta">
          <span class="card-meta-item">📄 15,200 words</span>
          <span class="card-meta-item">📖 8 chapters</span>
          <span class="card-meta-item">✅ Complete</span>
        </div>
        <div class="card-summary">
          The Weasleys decide to take a Muggle road trip across America.
          Arthur is thrilled. Molly is stressed. The twins have packed
          fireworks. Ron is already homesick. A complete fluff adventure.
        </div>
      </div>
      <div class="card-tags">
        <span class="tag">humor</span>
        <span class="tag">fluff</span>
        <span class="tag">family</span>
        <span class="tag">road trip</span>
      </div>
      <div class="card-footer">
        <button class="btn btn-primary">📥 Download EPUB</button>
        <button class="btn btn-secondary">↗ View on AO3</button>
      </div>
    </div>

    <!-- Card 4 -->
    <div class="card">
      <div class="card-header">
        <div class="card-title">Severus Snape and the Art of Self-Care</div>
        <span class="card-rating rating-teen">Teen</span>
      </div>
      <div class="card-body">
        <div class="card-author">by <span>PotionsMaster99</span></div>
        <div class="card-meta">
          <span class="card-meta-item">📄 8,400 words</span>
          <span class="card-meta-item">📖 1 chapter</span>
          <span class="card-meta-item">✅ Complete</span>
        </div>
        <div class="card-summary">
          After the war, Severus Snape doesn't die. He just... leaves.
          This is the story of a bitter man learning, slowly and painfully,
          that he deserves a life beyond survival.
        </div>
      </div>
      <div class="card-tags">
        <span class="tag">hurt/comfort</span>
        <span class="tag">character study</span>
        <span class="tag">one-shot</span>
      </div>
      <div class="card-footer">
        <button class="btn btn-primary">📥 Download EPUB</button>
        <button class="btn btn-secondary">↗ View on AO3</button>
      </div>
    </div>

    <!-- Card 5 -->
    <div class="card">
      <div class="card-header">
        <div class="card-title">Marauders: Year Four</div>
        <span class="card-rating rating-mature">Mature</span>
      </div>
      <div class="card-body">
        <div class="card-author">by <span>MoonyWormtailPadfootProngs</span></div>
        <div class="card-meta">
          <span class="card-meta-item">📄 124,800 words</span>
          <span class="card-meta-item">📖 32 chapters</span>
          <span class="card-meta-item">🔄 In Progress</span>
        </div>
        <div class="card-summary">
          The Marauders' fourth year brings new enemies, new alliances,
          and new understanding of the dark magic growing at the edges of
          the wizarding world. Sirius Black begins to question his family.
        </div>
      </div>
      <div class="card-tags">
        <span class="tag">prequel</span>
        <span class="tag">marauders era</span>
        <span class="tag">slow burn</span>
        <span class="tag">canon divergence</span>
      </div>
      <div class="card-footer">
        <button class="btn btn-primary">📥 Download EPUB</button>
        <button class="btn btn-secondary">↗ View on AO3</button>
      </div>
    </div>

    <!-- Card 6 -->
    <div class="card">
      <div class="card-header">
        <div class="card-title">The Hogwarts Exchange Program</div>
        <span class="card-rating rating-general">General</span>
      </div>
      <div class="card-body">
        <div class="card-author">by <span>IlvermornyAlumni</span></div>
        <div class="card-meta">
          <span class="card-meta-item">📄 28,900 words</span>
          <span class="card-meta-item">📖 12 chapters</span>
          <span class="card-meta-item">✅ Complete</span>
        </div>
        <div class="card-summary">
          When three students from Ilvermorny spend a semester at Hogwarts,
          cultural clashes and magical misunderstandings ensue. But they
          also discover that magic is bigger than any one school.
        </div>
      </div>
      <div class="card-tags">
        <span class="tag">crossover</span>
        <span class="tag">humor</span>
        <span class="tag">friendship</span>
        <span class="tag">Ilvermorny</span>
      </div>
      <div class="card-footer">
        <button class="btn btn-primary">📥 Download EPUB</button>
        <button class="btn btn-secondary">↗ View on AO3</button>
      </div>
    </div>
  </div>
</body>
</html>
```

Open this file in your browser. You should see a beautiful grid of six fanfiction cards with:
- Different color-coded rating badges (Mature = red, Teen = orange, General = green)
- Word count, chapter count, and completion status
- Tag pills with a blue tint
- Hover effects (cards lift up and glow)
- Responsive layout (grid on desktop, single column on phone)

This is a real, production-quality component. The same patterns we used here—the same CSS variables, the same box model, the same Flexbox and Grid layouts—are the exact same patterns used by professional web developers at companies like GitHub, Netflix, and Stripe.

### What You've Learned

In this chapter, you learned:

✅ **HTML elements** — div, p, h1, a, button, input, span, img, and more  
✅ **CSS selectors** — element, class, and ID selectors  
✅ **The box model** — margin, border, padding, content  
✅ **Flexbox** — arranging elements in rows and columns  
✅ **CSS variables** — reusable colors and values  
✅ **Media queries** — making your design responsive  
✅ **Grid layout** — creating card grids that adapt to screen size  
✅ **Hover effects** — making interactive, alive-feeling components  
✅ **Specificity** — understanding which CSS rules win  

These are the fundamental skills of frontend web development. Everything else builds on them.

---

> **Try It Yourself: Build Something New**
>
> Before moving on to the next part of the book, try building something on your own. Here are some ideas:
>
> 1. **A profile card** — Create a card with a circle avatar image, a name, a bio, and social media links. Use Flexbox to arrange the links in a row.
>
> 2. **A pricing card** — Build a pricing card with a title ("Pro Plan"), a price ("$9.99/mo"), a list of features, and a "Sign Up" button. Use the card pattern we learned.
>
> 3. **A dark mode toggle** — Create a page with a button that switches between dark and light themes by changing CSS variables. (Hint: You'll need a tiny bit of JavaScript for this.)
>
> 4. **A responsive navbar** — Build a navigation bar with a logo on the left and links on the right. On mobile, make the links stack vertically.
>
> Don't aim for perfection. Aim for practice. The more you build, the faster you improve.

---

> **Watch Out: Copying Code vs. Writing Code**
>
> When you're learning, it's fine to type code from examples—don't copy-paste it. Typing it yourself forces your brain to process every character. You'll make typos. You'll fix typos. You'll understand the code better because of it.
>
> After typing an example, try modifying it. Change the colors. Add an element. Remove something. Break it on purpose and fix it. This is how learning happens—not in the smooth path, but in the struggle and recovery.

### Where We Go From Here

You've done something remarkable. In four chapters, you've gone from knowing nothing (or very little) to understanding how the web works, having a development environment set up, and building real, styled components.

You understand:
- How the internet delivers pages from servers to browsers
- How HTML, CSS, and JavaScript work together
- What SvelteKit does and how to run it
- How to write clean, styled HTML with CSS
- How to make your designs responsive
- How to organize your code with CSS variables
- How to debug with browser developer tools
- How to commit and track your work with Git

The foundation is solid. Now it's time to go deeper.

In Part 2, we'll start building FicHub for real. We'll set up SvelteKit routes, build the three-tab interface, create the download form, and connect everything to a backend API. You'll see how the pieces we've learned—the HTML structure, the CSS styling, the SvelteKit framework—come together in a real application.

Ready? Let's go.

---

> **Summary: Part 1 Checklist**
>
> Before moving to Part 2, make sure you can answer "yes" to all of these:
>
> - [ ] I can explain what HTML, CSS, and JavaScript each do
> - [ ] I can explain what a web server and an API are
> - [ ] I can open a terminal and use `cd`, `ls`, `mkdir`, and `pwd`
> - [ ] I have Node.js and npm installed (`node --version` works)
> - [ ] I have VS Code installed with the Svelte extension
> - [ ] I have created a SvelteKit project (`npx sv create`)
> - [ ] I can run the dev server (`npm run dev`) and see it in the browser
> - [ ] I can write basic HTML with headings, paragraphs, links, and buttons
> - [ ] I can style elements with CSS using class selectors
> - [ ] I understand the box model (margin, border, padding, content)
> - [ ] I can use Flexbox to arrange elements in a row
> - [ ] I can create CSS variables for theme colors
> - [ ] I can use media queries to make a layout responsive
> - [ ] I've committed my work with `git add . && git commit -m "..."`
>
> If you're missing any of these, go back to the relevant chapter and practice. The foundation matters.
# Part 2: SvelteKit Fundamentals

*Welcome back, fellow builder! In Part 1, we got our feet wet with HTML, CSS, and JavaScript — the three ingredients that make up every web page. Now it's time to level up. We're going to meet SvelteKit, a modern framework that makes building web apps feel like snapping LEGO bricks together. Let's dive in!*

---

## Chapter 5: What is SvelteKit?

### Svelte vs React vs Vue: The Three Friends

Imagine you're at a playground. There are three friends who all love building things — but they each have their own style. They're all at the same park, playing the same game, but they've each brought their own favorite tools.

**React** is the oldest and most popular kid on the block. It was created by Facebook (now Meta) in 2013, and it's been the go-to choice for years. React uses something called JSX, which mixes HTML-like code directly into your JavaScript. It has a huge community, tons of libraries, and powers sites like Instagram, Netflix, and Airbnb. React is like the kid with the biggest toolbox — there's a tool for everything, but sometimes you need a special tool just to find the right tool.

Here's what a React component looks like:

```jsx
// React component (JSX syntax)
function Greeting({ name }) {
  const [count, setCount] = useState(0);
  
  return (
    <div>
      <h1>Hello, {name}!</h1>
      <p>Count: {count}</p>
      <button onClick={() => setCount(count + 1)}>Click me</button>
    </div>
  );
}
```

Notice the JSX — it looks like HTML mixed into JavaScript. You need to use `className` instead of `class`, `htmlFor` instead of `for`, and wrap everything in a single parent element. It works, but it takes some getting used to.

**Vue** is the friendly neighbor. Created by Evan You in 2014, Vue is known for being easy to learn. It uses a template syntax that looks a lot like plain HTML, so beginners often find it approachable. Vue powers sites like Alibaba, GitLab, and Nintendo. Vue is like the kid who shares their toys and explains the rules before you start playing.

Here's what a Vue component looks like:

```vue
<!-- Vue component -->
<template>
  <div>
    <h1>Hello, {{ name }}!</h1>
    <p>Count: {{ count }}</p>
    <button @click="count++">Click me</button>
  </div>
</template>

<script setup>
import { ref } from 'vue';

const name = ref('World');
const count = ref(0);
</script>
```

Vue separates the template (HTML), script (JavaScript), and style (CSS) into different sections. It's clean and organized, but you have to learn Vue-specific directives like `@click` and `{{ }}`.

**Svelte** is the newcomer with the brilliant idea. Created by Rich Harris in 2016, Svelte takes a completely different approach. Instead of doing work in the browser (like React and Vue), Svelte does its work at *build time* — it compiles your code into tiny, efficient vanilla JavaScript before it ever reaches the user. Svelte powers parts of the New York Times, Apple Music, and Spotify. Svelte is like the kid who does all their homework before class so they can just relax and enjoy the lesson.

Here's what a Svelte component looks like:

```svelte
<!-- Svelte component -->
<script>
  let count = $state(0);
</script>

<h1>Hello, World!</h1>
<p>Count: {count}</p>
<button onclick={() => count++}>Click me</button>
```

Look how clean that is! The HTML is just HTML. The JavaScript is just JavaScript. There's no special template syntax to learn, no JSX quirks, no special imports for basic reactivity. It reads almost like plain HTML with a little bit of magic sprinkled on top.

Here's a quick comparison:

| Feature | React | Vue | Svelte |
|---------|-------|-----|--------|
| Created | 2013 | 2014 | 2016 |
| Syntax | JSX | Templates | HTML-like |
| Virtual DOM | Yes | Yes | No |
| Build step | Needed | Optional | Required |
| Learning curve | Medium | Easy | Easy |
| Bundle size | Medium | Small | Tiny |
| Popularity | Highest | High | Growing fast |
| Component syntax | `onClick` | `@click` | `onclick` |
| State management | `useState` | `ref()` | `$state` |
| Styling | CSS-in-JS / Modules | Scoped `<style>` | Scoped `<style>` |

### How Svelte is Different: The Compiler Approach

This is where things get really interesting. Let's talk about what makes Svelte special, because this is the core idea that everything else is built on.

**React and Vue** both use something called a **Virtual DOM**. Think of it like this: when you want to change a page, React and Vue first make a copy of the entire page in memory (the "virtual" copy), figure out what changed, and then carefully update only the parts that are different. It's like making a photocopy of your homework, crossing out what's wrong, and then copying the changes onto the real homework. It works, but it takes extra steps.

The Virtual DOM process looks like this:

```
Step 1: State changes (someone clicked a button)
Step 2: Framework creates a new virtual tree (a copy of what the page SHOULD look like)
Step 3: Framework compares the new tree to the old tree (diffing)
Step 4: Framework figures out the minimum changes needed
Step 5: Framework applies those changes to the real DOM
```

That's five steps for every single change. When your app has hundreds of components updating frequently, those steps add up.

**Svelte** skips all of that. Instead of doing work in the browser, Svelte works at *build time*. When you write Svelte code, the Svelte compiler transforms it into plain JavaScript that directly updates the DOM. There's no virtual DOM, no copying, no diffing. It's like having a personal scribe who writes your homework directly on the paper — no drafts, no copies, just the final result.

The Svelte process looks like this:

```
Step 1: You write Svelte code
Step 2: The Svelte compiler transforms it into vanilla JavaScript (at build time)
Step 3: The browser runs the vanilla JavaScript
Step 4: JavaScript directly updates the DOM — no intermediary
```

That's fewer steps at runtime, which means faster performance.

Here's the key insight:

```
React/Vue approach:
  Your code → Runtime framework (in the browser) → DOM updates
  
Svelte approach:
  Your code → Compiler (at build time) → Optimized vanilla JS → DOM updates
```

This means:
- **Smaller bundle sizes** — you don't ship a big framework to every user
- **Faster performance** — no virtual DOM overhead
- **Less JavaScript** — the browser does less work

Think of it this way: React ships the entire kitchen (stove, fridge, counters, utensils) to every user so they can cook their meal. Svelte pre-cooks the meal and just ships the plate.

Let's look at a real example. Here's how you'd toggle a class in each framework:

**React:**
```jsx
function Toggle() {
  const [isOn, setIsOn] = useState(false);
  
  return (
    <div className={`toggle ${isOn ? 'on' : 'off'}`}>
      <button onClick={() => setIsOn(!isOn)}>
        {isOn ? 'ON' : 'OFF'}
      </button>
    </div>
  );
}
```

**Svelte:**
```svelte
<script>
  let isOn = $state(false);
</script>

<div class="toggle" class:on={isOn} class:off={!isOn}>
  <button onclick={() => isOn = !isOn}>
    {isOn ? 'ON' : 'OFF'}
  </button>
</div>
```

Svelte's version is shorter, more readable, and produces less JavaScript. The `class:on={isOn}` syntax is Svelte's way of conditionally adding a class — it's like saying "add the 'on' class when `isOn` is true." No need for string concatenation or ternary operators.

### SvelteKit: Svelte + Routing + Build Tools

Now, Svelte on its own is like a really good set of building blocks. But building blocks alone don't make a house — you also need a blueprint, some power tools, and a way to organize everything.

That's where **SvelteKit** comes in.

SvelteKit is the **meta-framework** for Svelte. A meta-framework is a framework built on top of another framework. It gives you:

1. **File-based routing** — your file structure determines your URLs
2. **Server-side rendering** — pages can be rendered on the server for speed
3. **Build tools** — Vite under the hood for blazing-fast development
4. **Code splitting** — only load the JavaScript you need for each page
5. **Static site generation** — pre-render pages for maximum speed
6. **TypeScript support** — optional but wonderful
7. **API routes** — build backend endpoints right in your app
8. **Form handling** — progressive enhancement for forms
9. **Deployment adapters** — deploy to Vercel, Netlify, Cloudflare, or anywhere

Here's an analogy: If Svelte is a hammer, SvelteKit is the entire toolbox with a workbench, clamps, and a measuring tape. You *can* build things with just the hammer, but having the full setup makes everything easier.

Other frameworks have similar meta-frameworks:
- **React** has Next.js
- **Vue** has Nuxt.js
- **Svelte** has SvelteKit

Here's how the layers stack up:

```
┌─────────────────────────────────────────┐
│            Your Application             │
├─────────────────────────────────────────┤
│              SvelteKit                   │
│  (routing, SSR, build tools, adapters)  │
├─────────────────────────────────────────┤
│               Svelte                     │
│     (components, reactivity, compiler)  │
├─────────────────────────────────────────┤
│  HTML + CSS + JavaScript (the basics!)  │
└─────────────────────────────────────────┘
```

### Why We Chose SvelteKit: Fast, Small, Simple

There are lots of great frameworks out there, so why did we pick SvelteKit for this book? Three reasons:

**1. Fast** — SvelteKit apps load incredibly quickly. Because Svelte compiles away its runtime, the JavaScript your users download is small and efficient. Pages load fast, interactions feel snappy, and your users don't sit there staring at loading spinners. In benchmarks, SvelteKit consistently ranks among the fastest frameworks for page load time and time to interactive.

**2. Small** — A typical Svelte component is much smaller than the equivalent React or Vue component. Less code means less to download, less to parse, and fewer bugs. It's like packing for a trip — the less you bring, the less you have to carry. A typical SvelteKit app might ship 50KB of JavaScript, while the equivalent React app might be 200KB or more.

**3. Simple** — Svelte's syntax is intentionally close to plain HTML, CSS, and JavaScript. If you learned those three in Part 1, you're already most of the way to learning Svelte. There's less "magic" to memorize, fewer concepts to wrap your head around, and the code reads almost like English.

Rich Harris, Svelte's creator, once said that his goal was to make a framework that "disappears" — one that gets out of your way and lets you focus on building, not fighting with the framework. That philosophy runs through every part of SvelteKit.

Here's a real comparison. Let's count the lines of code for a simple counter component:

```
React (with hooks):     ~12 lines
Vue (Composition API):  ~10 lines  
Svelte:                 ~8 lines
Plain HTML + JS:        ~15 lines (no reactivity)
```

Svelte wins on conciseness without sacrificing readability.

### What is a Single Page Application (SPA)?

Let's talk about how modern web apps work.

In the old days of the web (we're talking early 2000s), every time you clicked a link, the browser would:
1. Send a request to the server
2. Wait for the server to build a whole new HTML page
3. Download that entire page
4. Display it

This meant the entire page would flash white and reload every time you navigated. It was like flipping through a photo album where every photo required you to walk to a different room, find the photo, bring it back, and put it in the album.

A **Single Page Application (SPA)** works differently. Instead of loading a whole new page every time, the app:
1. Loads ONE HTML page with all the JavaScript it needs
2. When you click a link, JavaScript intercepts the click
3. JavaScript updates the page content without reloading
4. The browser URL changes, but the page doesn't flash or refresh

It's like having a magic photo album where all the photos are already on the page — you just slide them into view without ever leaving your chair.

The benefits of SPAs are huge:
- **Faster navigation** — no full page reloads
- **Smoother transitions** — you can animate between pages
- **Better user experience** — feels like a native app, not a website
- **Less server work** — the server just sends data, not whole pages

But SPAs also have downsides:
- **Slower initial load** — all that JavaScript has to download first
- **SEO challenges** — search engines sometimes struggle with SPAs
- **No "back" button initially** — you have to handle this yourself

SvelteKit actually solves most of these downsides. It can do server-side rendering for the first load (fast initial paint!) and then take over as an SPA for navigation. It's the best of both worlds.

### Client-Side vs Server-Side Rendering

There are three main ways a page can be rendered. Let's understand each one:

**1. Client-Side Rendering (CSR)** — The server sends a mostly empty HTML shell with a bunch of JavaScript. The JavaScript runs in the browser and builds the entire page. This is what traditional SPAs do.

```
Browser requests page → Server sends empty shell + JS → 
Browser downloads JS → Browser runs JS → Page appears
```

The downside? The user sees a blank page for a moment while JavaScript loads and runs. If the user has a slow connection, they're staring at nothing. It's like being invited to a party but the host is still setting up when you arrive.

**2. Server-Side Rendering (SSR)** — The server builds the page and sends fully rendered HTML to the browser. The user sees the content immediately, and then JavaScript "hydrates" the page to make it interactive.

```
Browser requests page → Server builds complete HTML → 
Browser shows page immediately → JS loads and hydrates → Page becomes interactive
```

The advantage? The user sees content right away — no blank screens, no waiting for JavaScript to download and run. It's like arriving at a party and everything is already set up and ready to go.

**3. Static Site Generation (SSG)** — The server pre-builds all pages at build time. The pre-built HTML files are served directly from a CDN. This is the fastest possible option.

```
Build time: Server generates all pages → HTML files saved to disk
Request time: CDN serves pre-built HTML → Instant load!
```

The advantage? Blazing fast performance because there's no server-side computation needed at request time. The HTML is already built and waiting.

**SvelteKit does all three.** By default, pages are server-rendered for the initial load (fast first paint!) and then take over on the client for interactivity. You can choose SSR, CSR, or static generation on a per-page basis. That's incredibly powerful — you get the best of all worlds.

Here's a comparison:

| Approach | First Paint Speed | Subsequent Navigation | SEO | Use Case |
|----------|------------------|----------------------|-----|----------|
| CSR | Slow | Fast | Poor | Private dashboards |
| SSR | Fast | Fast | Good | Public websites |
| SSG | Fastest | Fastest | Best | Blogs, docs, marketing |
| SvelteKit (default) | Fast | Fast | Good | Everything! |

> **🔍 Fun Fact:** The term "hydration" comes from the idea that the server sends "dry" HTML and the JavaScript "hydrates" it with interactivity, like adding water to a dried sponge to make it squishy and alive again!

### The SvelteKit File Structure: +page.svelte, +layout.svelte, +page.ts

One of the coolest things about SvelteKit is how it organizes your project. It uses **file-based routing**, which means the structure of your folders and files directly maps to the URLs in your app.

Here's the basic idea:

```
src/
  routes/
    +page.svelte          → matches "/" (your home page)
    about/
      +page.svelte        → matches "/about"
    blog/
      +page.svelte        → matches "/blog"
      [slug]/
        +page.svelte      → matches "/blog/my-post"
    +layout.svelte        → wraps ALL pages
```

Notice the special files:
- **`+page.svelte`** — This is the component for a specific page. Every URL in your app needs one. Think of it as a single "screen" in your app.
- **`+layout.svelte`** — This wraps around multiple pages. Think of it like a picture frame that holds all your pages. The frame (layout) stays the same; only the picture (page) changes.
- **`+page.ts`** (or `+page.js`) — This is a "load" function that fetches data for a page before it renders. We'll dive deep into this in Chapter 9.
- **`+layout.ts`** — Like `+page.ts` but for layout-level data. If all pages in a section need the same data, you load it once in the layout.
- **`+page.server.ts`** — Server-only load function. Runs on the server, never sent to the browser. Good for secret API keys or database queries.
- **`+layout.server.ts`** — Server-only layout data.

The `+` prefix tells SvelteKit "this is a special file." Without the `+`, it's just a regular file that SvelteKit won't treat as a route. It's like the `+` is a magic prefix that says "I'm part of the routing system!"

Here's a visual map:

```
Your SvelteKit app
├── src/
│   ├── app.html          ← The HTML shell (every page lives inside this)
│   ├── app.css            ← Global styles
│   ├── lib/               ← Shared code, components, utilities
│   │   ├── components/    ← Reusable UI components
│   │   ├── stores/        ← Shared state (if using stores)
│   │   └── utils/         ← Helper functions
│   └── routes/            ← All your pages and layouts
│       ├── +layout.svelte     ← The wrapper for every page
│       ├── +layout.ts         ← Data for the root layout
│       ├── +page.svelte       ← Home page (/)
│       ├── +page.ts           ← Data for the home page
│       ├── about/
│       │   ├── +page.svelte   ← About page (/about)
│       │   └── +page.ts       ← Data for the about page
│       ├── blog/
│       │   ├── +layout.svelte ← Blog section wrapper
│       │   ├── +layout.ts     ← Data for all blog pages
│       │   ├── +page.svelte   ← Blog index (/blog)
│       │   ├── +page.ts       ← Blog post list data
│       │   └── [slug]/
│       │       ├── +page.svelte ← Individual blog post
│       │       └── +page.ts     ← Single post data
│       └── contact/
│           └── +page.svelte   ← Contact page (/contact)
├── static/                ← Images, fonts, files served as-is
│   ├── favicon.png
│   └── images/
├── svelte.config.js       ← SvelteKit configuration
├── vite.config.ts         ← Vite build configuration
├── tsconfig.json          ← TypeScript configuration
└── package.json           ← Dependencies and scripts
```

Don't worry if this feels like a lot — we'll build this structure step by step in the next chapter. For now, just remember: **files are routes**.

The magic of file-based routing is that it's **obvious**. You don't need to open a separate router configuration file to understand how URLs map to pages. Just look at the folder structure! If you see `blog/[slug]/+page.svelte`, you know there's a dynamic blog route. It's self-documenting.

> **🧪 Try It Yourself:** Before we move on, think about a website you use every day (like YouTube or Instagram). Can you figure out what its routes might be? YouTube has `/watch`, `/channel`, `/playlist`, and `/results` routes. Instagram has `/`, `/explore`, `/accounts`, and `/direct`. Once you start thinking about URLs as file paths, the whole web starts to make more sense!

### Wrapping Up Chapter 5

Let's recap what we learned:

- **Svelte** is a compiler-based framework that builds efficient vanilla JavaScript at compile time
- **SvelteKit** is the full-stack framework built on Svelte, giving you routing, rendering, and build tools
- **SPAs** load one page and update content without full reloads
- **SSR** renders pages on the server for faster first paint
- **SSG** pre-builds pages for maximum speed
- **CSR** renders everything in the browser (good for private apps)
- **File-based routing** means your folder structure determines your URLs
- Special files like `+page.svelte`, `+layout.svelte`, and `+page.ts` are the building blocks of a SvelteKit app

In the next chapter, we'll create our very first SvelteKit project from scratch. We'll run the commands, explore the files, and see the Svelte compiler in action. Let's go!

---

## Chapter 6: Creating Your First SvelteKit Project

### Running npx sv create

Alright, it's time to get our hands dirty! Let's create a brand new SvelteKit project. This is the moment where theory meets practice — everything we talked about in Chapter 5 becomes real.

Open your terminal (that's the command-line tool on your computer — on Mac it's called Terminal, on Windows it's PowerShell or Command Prompt, on Linux it's whatever terminal emulator you prefer).

Navigate to the folder where you want to create your project:

```bash
# Navigate to your projects folder (or wherever you like)
cd ~/projects

# Create a new SvelteKit project
npx sv create my-first-sveltekit-app
```

The `npx` command is a tool that comes with Node.js. It downloads and runs a package temporarily, without installing it globally. Think of it like renting a tool from a hardware store instead of buying it — you use it once and then return it.

When you run this command, you'll see a friendly interactive prompt. Let's walk through each option:

```
┌──────────────────────────────────────────┐
│  Welcome to the SvelteKit CLI!           │
│                                          │
│  ✔ Project name: my-first-sveltekit-app  │
│  ✔ Type of project: Skeleton project     │
│  ✔ Add TypeScript: Yes                   │
│  ✔ Select additional options:            │
│    ✔ Add ESLint for code linting          │
│    ✔ Add Prettier for code formatting     │
│    ✔ Add Playwright for browser testing   │
└──────────────────────────────────────────┘
```

Let's talk about each choice in detail:

**Project name** — This is the name of your project's folder. You can call it anything you like (no spaces though!). Good names describe what the project is: `todo-app`, `portfolio`, `my-blog`, `weather-dashboard`. Avoid names like `test123` or `stuff` — you'll thank yourself later when you have twenty projects and need to find the right one.

**Type of project** — The "Skeleton project" is a clean, minimal starting point. It gives you the bare essentials to start building. There are other options for specific templates (like `SvelteKit demo app` which comes with example code), but the skeleton is perfect for learning because you start with a blank canvas.

**TypeScript** — We recommend saying **Yes** here. TypeScript adds type checking to JavaScript, which helps catch errors before they happen. It's like having a spell-checker that catches mistakes while you type, rather than after you've already saved the file. Don't worry if you're not familiar with it — we'll use it gently throughout this book.

**ESLint** — Say **Yes**. ESLint is like a spell-checker for your code. It catches common mistakes and helps you write better code. It might feel annoying at first ("Why is it complaining about my semicolons?"), but it'll save you from headaches later. Think of it as a patient teacher who gently corrects your grammar.

**Prettier** — Say **Yes**. Prettier automatically formats your code so it always looks neat and consistent. No more arguments about tabs vs spaces! It's like having a personal stylist for your code — everything always looks its best.

**Playwright** — You can say Yes or No here. Playwright is for automated browser testing, which is more advanced. It lets you write scripts that click buttons and check pages automatically. We'll skip it for now — we'll come back to testing in a later chapter.

> **🧪 Try It Yourself:** Go ahead and run the command! Choose the options we suggested. If you get stuck on a prompt, look for the arrow keys on your keyboard to navigate between options, and press Enter to select. Don't be afraid to experiment — you can always create a new project if something goes wrong!

### Prerequisites: What You Need Installed

Before creating your project, make sure you have Node.js installed. Node.js is the JavaScript runtime that powers SvelteKit's development server and build tools.

Check if you have it:

```bash
node --version
# Should show something like: v20.11.0

npm --version
# Should show something like: 10.2.4
```

If you don't have Node.js, download it from [nodejs.org](https://nodejs.org). Choose the LTS (Long Term Support) version — it's the most stable.

You'll also need a code editor. We recommend **VS Code** (Visual Studio Code) — it's free, powerful, and has great Svelte support. Install the **Svelte extension** for syntax highlighting and helpful suggestions:

1. Open VS Code
2. Press `Ctrl+Shift+X` (or `Cmd+Shift+X` on Mac) to open Extensions
3. Search for "Svelte"
4. Install the "Svelte for VS Code" extension

### The Project Structure Explained

Once the project is created, let's look at what SvelteKit generated for us:

```bash
# Navigate into your new project
cd my-first-sveltekit-app

# Look at the files
ls -la
```

You should see something like this:

```
my-first-sveltekit-app/
├── src/
│   ├── routes/
│   │   ├── +page.svelte
│   │   └── +layout.svelte
│   ├── app.html
│   └── app.css
├── static/
│   └── favicon.png
├── svelte.config.js
├── vite.config.ts
├── tsconfig.json
├── package.json
└── README.md
```

That's it! That's the entire skeleton. Not bad, right? Let's explore each file.

> **⚠️ Watch Out:** Don't delete files just because you don't recognize them! Each file has a purpose. If you're curious about a file, read the comments at the top — they often explain what the file does.

Let's also check what's in `package.json` — this file lists all the tools and libraries your project uses:

```json
{
  "name": "my-first-sveltekit-app",
  "version": "0.0.1",
  "private": true,
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview",
    "check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
    "check:watch": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json --watch",
    "lint": "prettier --check . && eslint .",
    "format": "prettier --write ."
  },
  "devDependencies": {
    "@sveltejs/adapter-auto": "...",
    "@sveltejs/kit": "...",
    "@sveltejs/vite-plugin-svelte": "...",
    "svelte": "...",
    "svelte-check": "...",
    "typescript": "...",
    "vite": "..."
  }
}
```

Don't worry about every line — just notice that `svelte`, `@sveltejs/kit`, and `vite` are in there. These are the core tools that make your app work.

### src/app.html: The Shell

Let's start with the most fundamental file — `src/app.html`:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <link rel="icon" href="%sveltekit.assets%/favicon.png" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    %sveltekit.head%
  </head>
  <body data-sveltekit-preload-data="hover">
    <div style="display: contents">%sveltekit.body%</div>
  </body>
</html>
```

This is the **HTML shell** — the container that every page in your app gets loaded into. Think of it like the cover of a binder or the frame of a picture. Every page you create gets slipped inside this cover.

Let's break down each part:

**`<!doctype html>`** — This tells the browser "this is an HTML5 document." It's always the first line.

**`<html lang="en">`** — The root HTML element. The `lang="en"` attribute tells screen readers and search engines that this page is in English. If you're building for a different language, you'd change this to `lang="es"` for Spanish, `lang="fr"` for French, etc.

**`<meta charset="utf-8" />`** — This tells the browser to use UTF-8 character encoding, which supports all languages and special characters. Without this, characters like é, ñ, or 日 might display incorrectly.

**`<link rel="icon" href="%sveltekit.assets%/favicon.png" />`** — This is the little icon that appears in the browser tab. The `%sveltekit.assets%` is a placeholder that points to your static assets folder.

**`<meta name="viewport" content="width=device-width, initial-scale=1" />`** — This makes your app responsive on mobile devices. Without this line, your site would look tiny on phones because the browser would try to show the desktop version scaled down.

**`%sveltekit.head%`** — This is a placeholder where SvelteKit injects any `<head>` content your pages need (title tags, meta tags, CSS, etc.). When you use `<svelte:head>` in a component, the content gets injected here.

**`<div style="display: contents">%sveltekit.body%</div>`** — This is where your actual page content gets rendered. It's like the blank pages inside the binder. The `display: contents` style makes the div "invisible" to CSS layout — it doesn't add any extra styling, it just holds the content.

You rarely need to edit this file. It just works. But knowing it exists helps you understand where everything lives. It's the foundation that everything else builds on.

### src/routes/+page.svelte: Your First Page

Now let's look at the star of the show — `src/routes/+page.svelte`:

```svelte
<script>
  let count = $state(0);

  function increment() {
    count++;
  }
</script>

<h1>Welcome to SvelteKit</h1>
<p>Visit <a href="https://svelte.dev/docs/kit">svelte.dev/docs/kit</a> to read the documentation</p>

<button onclick={increment}>
  Clicked {count} {count === 1 ? 'time' : 'times'}
</button>

<button onclick={() => (count += 5)}>
  Add 5
</button>
```

Let's break this down piece by piece:

**The `<script>` block** — This is where your JavaScript lives. In Svelte, you write JavaScript right inside the component. No imports needed for basic functionality. The `$state(0)` creates a reactive variable called `count` with an initial value of `0`. We'll learn all about runes like `$state` in the next chapter.

**The HTML** — Below the script, you write regular HTML! The `<h1>` and `<p>` tags are just normal HTML. The `{count}` in the button text is Svelte's way of showing JavaScript values in your HTML — this is called **interpolation**. The curly braces `{}` say "insert the value of this JavaScript expression here."

**The `onclick`** — This is an event handler. When the button is clicked, it runs the `increment` function. Notice how clean it looks compared to React's `onClick={increment}` or Vue's `@click="increment"`. Svelte's syntax is closer to plain HTML.

**The ternary expression** — `{count === 1 ? 'time' : 'times'}` is a ternary operator. It says "if count equals 1, use 'time'; otherwise, use 'times'." This gives us proper grammar: "Clicked 1 time" vs "Clicked 2 times."

**The arrow function** — `onclick={() => (count += 5)}` is an arrow function. It's a shorthand way of writing a function. When the button is clicked, it adds 5 to the count. We could also write it as:

```svelte
<button onclick={addFive}>Add 5</button>

<!-- In the script: -->
<script>
  function addFive() {
    count += 5;
  }
</script>
```

> **🧪 Try It Yourself:** Save this file and go to your browser at `http://localhost:5173`. Click the button and watch the count go up! Then try changing the initial value from `0` to `10`. What happens when you reload the page? (Hint: it resets to 10 — that's because `state` is client-side only!)

### src/routes/+layout.svelte: The Wrapper

Open up `src/routes/+layout.svelte`:

```svelte
<script>
  import '../app.css';
</script>

<slot />
```

This is simpler than you might expect. Let's understand each part:

**`import '../app.css'`** — This imports the global CSS file. This is where you put styles that apply to your entire app (like font families, color schemes, and reset styles). The `../` means "go up one folder" — from `routes/` to `src/`.

**`<slot />`** — This is a Svelte placeholder. It says "put whatever page content goes here right in the middle." Every page that matches the current route gets rendered where the `<slot />` is. Think of it as a window in a wall — the wall stays the same, but you can see different things through the window.

In Svelte 5, if you need to pass content from parent to child, you use `{@render children()}` instead of `<slot />`. The `<slot />` syntax still works but is considered legacy. Here's the modern version:

```svelte
<script>
  import '../app.css';
  let { children } = $props();
</script>

{@render children()}
```

Think of the layout as a **picture frame**. The frame (layout) stays the same no matter which picture (page) you display inside it. If you have a header, footer, or sidebar that should appear on every page, you put it in the layout:

```svelte
<script>
  import '../app.css';
  let { children } = $props();
</script>

<nav>
  <a href="/">Home</a>
  <a href="/about">About</a>
  <a href="/blog">Blog</a>
</nav>

<main>
  {@render children()}
</main>

<footer>
  <p>© 2026 My Awesome App</p>
</footer>

<style>
  nav {
    background: #1e293b;
    padding: 1rem;
    display: flex;
    gap: 1rem;
  }
  
  nav a {
    color: white;
    text-decoration: none;
  }
  
  main {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
  }
  
  footer {
    background: #f1f5f9;
    text-align: center;
    padding: 1rem;
    margin-top: 2rem;
  }
</style>
```

Now every page in your app will have the navigation bar at the top and the footer at the bottom, with the page content appearing in the middle. You only write the nav and footer once — no duplication!

> **💡 Pro Tip:** The `<style>` block at the bottom of a Svelte component is **scoped** — the styles only apply to that component. They won't leak out and affect other components. This means you can safely use class names like `.title` or `.container` in multiple components without conflicts!

### Running npm run dev and Seeing Your App

Time to start the development server and see your app in action!

```bash
# In your project folder, run:
npm run dev
```

You should see output like this:

```
  VITE v5.x.x  ready in 312 ms

  ➜  Local:   http://localhost:5173/
  ➜  Network: use --host to expose
  ➜  press h + enter to show help
```

Open your browser and go to `http://localhost:5173`. You should see your app!

The development server has some cool features:

- **Hot Module Replacement (HMR)** — When you change a file, the browser updates instantly without a full page reload. It's like having a magical clipboard that updates your work the second you save it. You change the text in your `<h1>`, hit save, and — boom — the browser updates. No refresh needed!

- **Error overlay** — If there's a mistake in your code, a friendly error screen appears right in the browser. It tells you exactly what's wrong and on which line. No more staring at a blank screen wondering what went wrong.

- **Terminal errors** — Any errors also show up in your terminal with helpful line numbers. This is useful when the browser error overlay isn't enough.

Try this: go back to `src/routes/+page.svelte` and change the `<h1>` text to something else. Save the file. Watch the browser update *instantly*. No refresh needed! That's HMR in action.

> **⚠️ Watch Out:** The development server runs as long as your terminal is open. To stop it, press `Ctrl + C` in the terminal. Closing the browser tab doesn't stop the server — it keeps running in the background. If you accidentally close your terminal, the server stops.

### The Svelte Compiler: What Happens Behind the Scenes

Let's peek under the hood. When you run `npm run dev`, something magical happens to your code.

Remember how we said Svelte is a **compiler**? Let's see it in action.

Take this simple Svelte component:

```svelte
<script>
  let count = $state(0);

  function increment() {
    count++;
  }
</script>

<h1>Hello, Svelte!</h1>
<button onclick={increment}>
  Count: {count}
</button>
```

When you save this file, the Svelte compiler transforms it into plain JavaScript that runs in the browser. The compiled output might look something like this (simplified for understanding):

```javascript
// Compiled JavaScript (simplified)
function mount(component, target) {
  const h1 = document.createElement('h1');
  h1.textContent = 'Hello, Svelte!';

  const button = document.createElement('button');
  let count = 0;

  function update() {
    button.textContent = `Count: ${count}`;
  }

  update();

  button.addEventListener('click', () => {
    count++;
    update();
  });

  target.appendChild(h1);
  target.appendChild(button);
}
```

Notice what happened:
1. **No framework runtime** — it's just vanilla DOM APIs. No `createElement`, no virtual DOM, no diffing algorithm. Just direct DOM manipulation.
2. **The compiler knows exactly what needs to change** — it saw that `count` is used in the button text, so it generates a direct update to `button.textContent`. No unnecessary work.
3. **The update function is minimal** — it only touches the elements that use `count`. If `count` only appears in the button, only the button gets updated.

This is why Svelte apps are so fast and small. The compiler does the heavy lifting so your users don't have to.

Compare this to React, which would ship a runtime library (~40KB) plus the component code, plus virtual DOM diffing logic. Svelte ships just the component code. That's like the difference between shipping a whole restaurant (stove, fridge, tables, chairs) versus shipping just the meal.

If you're curious, you can actually see the compiled output! The Svelte REPL at `svelte.dev/repl` lets you write Svelte code and see the compiled JavaScript side by side. It's a great way to understand what's happening behind the scenes.

> **🔍 Fun Fact:** The Svelte compiler is written in TypeScript and is about 60,000 lines of code. But it generates output that's typically 40% less JavaScript than the equivalent React app. That's like a chef who writes a 60-page cookbook but teaches you to cook meals that take up half the shelf space!

### Build Commands

Let's learn about the different commands you can run in your SvelteKit project:

```bash
# Development mode (what we just ran)
npm run dev

# Build for production
npm run build

# Preview the production build
npm run preview

# Run the linter
npm run lint

# Format code with Prettier
npm run format

# Type-check with svelte-check
npm run check
```

**`npm run dev`** starts the development server. This is what you'll use 90% of the time while building. It has hot reload, error overlays, and helpful warnings.

**`npm run build`** is particularly important. It creates a production-ready version of your app in a `build` folder. This is what you'd upload to a web server or deploy to a hosting platform like Vercel, Netlify, or Cloudflare Pages. The build process:
1. Compiles all your Svelte components
2. Bundles your JavaScript (combines files, removes unused code)
3. Optimizes your CSS
4. Generates static HTML (if using SSR/SSG)
5. Copies static assets

**`npm run preview`** lets you test the production build locally before deploying. It's like doing a dress rehearsal before the big show. Always preview before deploying!

**`npm run lint`** checks your code for style issues and potential bugs. Run this before committing code to catch problems early.

**`npm run format`** automatically reformats your code to follow consistent style rules. Run this when your code looks messy.

**`npm run check`** runs TypeScript type checking. It catches type errors that might not show up during development. This is like having a proofreader check your document before publishing.

> **🧪 Try It Yourself:** Run `npm run build` and look at the `build` folder. Notice how small the JavaScript files are? That's the Svelte compiler at work — shipping minimal code to your users! Then run `npm run preview` to see what the production build looks like. Can you tell the difference from the dev version?

### Wrapping Up Chapter 6

You did it! You created your first SvelteKit project. Let's review what we accomplished:

- Created a new SvelteKit project with `npx sv create`
- Explored the project structure and understand what each file does
- Learned about `app.html` (the shell), `+page.svelte` (the page), and `+layout.svelte` (the wrapper)
- Started the development server with `npm run dev`
- Saw the Svelte compiler transform our code into efficient JavaScript
- Learned about the different build commands

In the next chapter, we'll dive deep into **Svelte 5 Runes** — the new reactivity system that makes Svelte components feel magical. We'll learn about `$state`, `$derived`, `$effect`, and more. Get ready for some serious fun!

---

## Chapter 7: Svelte 5 Runes — The New Reactivity

### What are Runes? (Magic Symbols for Reactivity)

In the previous chapters, we mentioned that Svelte is a compiler. Now it's time to understand one of the most powerful features of Svelte 5: **runes**.

The word "rune" comes from Old Norse — it means a mysterious or magical symbol. In ancient times, runes were letters in alphabets used for writing, magic, and divination. In Svelte 5, runes are special expressions that tell the compiler "hey, make this reactive!" They look like dollar signs followed by a word:

```javascript
let count = $state(0);              // reactive variable
let doubled = $derived(count * 2);  // computed value
$effect(() => { /* ... */ });       // side effect
```

Think of runes like **special instructions** written on a treasure map. When you write `$state`, you're telling the Svelte compiler "this variable needs to trigger updates when it changes." Without runes, Svelte wouldn't know which variables are important.

Before Svelte 5, reactivity was based on assignments — `count = 5` would automatically be reactive. Runes make this more explicit and powerful. It's like the difference between assuming someone wants to hear your news versus clearly saying "hey, listen, I have something to tell you!"

Here's a quick overview of all the runes we'll cover:

| Rune | Purpose | Example |
|------|---------|---------|
| `$state` | Create reactive state | `let count = $state(0)` |
| `$derived` | Compute values from state | `let doubled = $derived(count * 2)` |
| `$derived.by` | Complex computed values | `let stats = $derived.by(() => {...})` |
| `$effect` | Run code when state changes | `$effect(() => { document.title = count })` |
| `$props` | Receive data from parent | `let { name } = $props()` |
| `$bindable` | Enable two-way binding | `let { value = $bindable(0) } = $props()` |

Each rune has a specific job. Together, they give you everything you need to build interactive, reactive applications.

### $state: Making Variables Reactive

The most fundamental rune is `$state`. It makes a variable reactive, which means whenever the variable changes, the parts of your page that use it will automatically update.

```svelte
<script>
  let count = $state(0);
  let name = $state('Svelte');
  let items = $state(['apple', 'banana', 'cherry']);
  let user = $state({ name: 'Alice', age: 25 });
</script>

<h1>Hello, {name}!</h1>
<p>Count: {count}</p>
<ul>
  {#each items as item}
    <li>{item}</li>
  {/each}
</ul>
<p>User: {user.name}, Age: {user.age}</p>

<button onclick={() => count++}>Increment</button>
<button onclick={() => name = 'World'}>Change Name</button>
<button onclick={() => items = [...items, 'date']}>Add Fruit</button>
<button onclick={() => user.age++}>Birthday</button>
```

When you click the "Increment" button, `count` changes, and the `<p>` tag updates instantly. When you click "Change Name", the `<h1>` updates. No extra code needed — the `$state` rune handles everything.

Let's break down what `$state` does:

```javascript
// WITHOUT runes (old Svelte 4 style)
let count = 0;  // This was automatically reactive in Svelte 4

// WITH runes (Svelte 5)
let count = $state(0);  // Explicitly reactive
```

The key difference? With runes, you **choose** which variables are reactive. This gives you more control and makes your code clearer. You're being explicit about your intent, which is always a good thing in programming.

> **⚠️ Watch Out:** `$state` creates a **copy** of objects and arrays for reactivity. This means if you pass an object to a child component, changes to the original object won't automatically reflect in the child. We'll see how to handle this with `$state.snapshot()` later.

### Reactive Arrays

Arrays with `$state` get special treatment. Svelte tracks which elements you access, so it only updates the DOM when *those specific elements* change. This is incredibly efficient.

```svelte
<script>
  let todos = $state([
    { id: 1, text: 'Learn Svelte', done: false },
    { id: 2, text: 'Build an app', done: false },
    { id: 3, text: 'Deploy to production', done: false }
  ]);

  function toggleTodo(id) {
    const todo = todos.find(t => t.id === id);
    if (todo) {
      todo.done = !todo.done;
    }
  }
</script>

{#each todos as todo}
  <div class:done={todo.done}>
    <input
      type="checkbox"
      checked={todo.done}
      onchange={() => toggleTodo(todo.id)}
    />
    {todo.text}
  </div>
{/each}
```

Notice how we can mutate the array directly — `todos.find(t => t.id === id)` and `todo.done = !todo.done`. In React, you'd need to create copies of everything:

```javascript
// React: You'd need to do this
setTodos(todos.map(todo => 
  todo.id === id ? { ...todo, done: !todo.done } : todo
));
```

Svelte handles the reactivity for you. It's like having a personal assistant who automatically updates your to-do list whenever you check something off.

**Adding items:**
```javascript
todos = [...todos, { id: 4, text: 'Write docs', done: false }];
```

**Removing items:**
```javascript
todos = todos.filter(t => t.id !== id);
```

**Reordering items:**
```javascript
todos = [todos[2], todos[0], todos[1]]; // Reorder
```

> **💡 Key Insight:** With Svelte 5's fine-grained reactivity, if you only read `todos[0].text` in your template, only that specific element's text will update when you change it. The other elements won't be touched. This is much more efficient than React's approach of re-rendering the entire component.

### $derived: Computed Values That Update Automatically

Sometimes you need a value that depends on another value. For example, a "full name" that combines a first name and last name. Or a shopping cart total that changes when items are added or removed. That's where `$derived` comes in.

```svelte
<script>
  let firstName = $state('Ada');
  let lastName = $state('Lovelace');
  
  let fullName = $derived(`${firstName} ${lastName}`);
</script>

<h1>Hello, {fullName}!</h1>
<input bind:value={firstName} placeholder="First name" />
<input bind:value={lastName} placeholder="Last name" />
```

Whenever `firstName` or `lastName` changes, `fullName` automatically updates. You don't need to write any code to make this happen — the `$derived` rune watches the variables it depends on and recomputes when they change.

Think of `$derived` like a **recipe**. If you change an ingredient (first name), the recipe (full name) automatically produces a different result. You don't need to manually re-run the recipe — it just knows.

Here's another example — a shopping cart total:

```svelte
<script>
  let items = $state([
    { name: 'Apples', price: 3.50, quantity: 2 },
    { name: 'Bread', price: 2.00, quantity: 1 },
    { name: 'Milk', price: 4.25, quantity: 1 }
  ]);

  let total = $derived(
    items.reduce((sum, item) => sum + item.price * item.quantity, 0)
  );

  let itemCount = $derived(
    items.reduce((sum, item) => sum + item.quantity, 0)
  );
  
  let averagePrice = $derived(
    itemCount > 0 ? total / itemCount : 0
  );
</script>

<h2>Shopping Cart</h2>
{#each items as item}
  <div>
    {item.name} × {item.quantity} = ${(item.price * item.quantity).toFixed(2)}
  </div>
{/each}
<hr />
<p>Total: {itemCount} items — ${total.toFixed(2)}</p>
<p>Average price: ${averagePrice.toFixed(2)}</p>
```

Change an item's quantity, and the total updates instantly. That's the power of `$derived`.

**More `$derived` examples:**

```svelte
<script>
  let radius = $state(5);
  
  // Circle calculations
  let area = $derived(Math.PI * radius * radius);
  let circumference = $derived(2 * Math.PI * radius);
  let diameter = $derived(radius * 2);
  
  // Formatted for display
  let areaFormatted = $derived(area.toFixed(2));
</script>

<p>Radius: {radius}</p>
<p>Area: {areaFormatted} square units</p>
<p>Circumference: {circumference.toFixed(2)} units</p>
<p>Diameter: {diameter} units</p>

<input type="range" bind:value={radius} min="1" max="20" />
```

> **⚠️ Watch Out:** `$derived` should only **read** state, never **change** it. If you try to modify a state variable inside `$derived`, you'll get an error. Derived values are like mirrors — they reflect what's there, they don't change it. If you need to change state based on other state, use `$effect` instead.

### $derived.by: Complex Computations

Sometimes you need to do more complex calculations in your derived value — multiple steps, temporary variables, or logic that's too complex for a single expression. That's when you use `$derived.by`:

```svelte
<script>
  let todos = $state([
    { text: 'Learn Svelte', done: true },
    { text: 'Build an app', done: true },
    { text: 'Deploy to production', done: false },
    { text: 'Write documentation', done: false },
    { text: 'Celebrate!', done: false }
  ]);

  let stats = $derived.by(() => {
    const total = todos.length;
    const completed = todos.filter(t => t.done).length;
    const remaining = total - completed;
    const percentage = total > 0 ? Math.round((completed / total) * 100) : 0;
    
    // Find the next incomplete todo
    const nextTodo = todos.find(t => !t.done);
    
    // Group todos by completion status
    const doneTodos = todos.filter(t => t.done);
    const pendingTodos = todos.filter(t => !t.done);
    
    return { 
      total, 
      completed, 
      remaining, 
      percentage,
      nextTodo,
      doneTodos,
      pendingTodos
    };
  });
</script>

<div class="stats">
  <h3>Progress: {stats.percentage}%</h3>
  <div class="progress-bar">
    <div class="fill" style="width: {stats.percentage}%"></div>
  </div>
  
  <div class="stat-grid">
    <div class="stat">
      <span class="stat-value">{stats.total}</span>
      <span class="stat-label">Total</span>
    </div>
    <div class="stat">
      <span class="stat-value">{stats.completed}</span>
      <span class="stat-label">Done</span>
    </div>
    <div class="stat">
      <span class="stat-value">{stats.remaining}</span>
      <span class="stat-label">Remaining</span>
    </div>
  </div>
  
  {#if stats.nextTodo}
    <p class="next">Next up: {stats.nextTodo.text}</p>
  {:else}
    <p class="congrats">🎉 All done!</p>
  {/if}
</div>
```

Notice that `$derived.by` takes a **function** (wrapped in curly braces `{}`) instead of a single expression. Inside that function, you can do as much computation as you need and return the final result.

**`$derived`** is for simple one-line calculations:
```javascript
let doubled = $derived(count * 2);
let fullName = $derived(`${first} ${last}`);
let isEven = $derived(count % 2 === 0);
```

**`$derived.by`** is for complex multi-step calculations:
```javascript
let stats = $derived.by(() => {
  // Multiple lines of logic here
  // Temporary variables are fine
  // Just return the final result
  return result;
});
```

> **💡 Rule of thumb:** If your derived value fits in one line, use `$derived`. If it needs multiple lines or temporary variables, use `$derived.by`. Think of `$derived` as a calculator and `$derived.by` as a spreadsheet.

### $effect: Side Effects (DOM Manipulation, API Calls)

So far, we've learned about state (`$state`) and computed values (`$derived`). But sometimes you need to *do* something when state changes — not just compute a value, but actually interact with the outside world. That's where `$effect` comes in.

```svelte
<script>
  let count = $state(0);

  $effect(() => {
    console.log(`Count changed to: ${count}`);
    document.title = `Count: ${count}`;
  });
</script>

<button onclick={() => count++}>Count: {count}</button>
```

Every time `count` changes, the effect runs. It logs to the console *and* updates the page title. The effect "reacts" to whatever state it reads.

Here's a more practical example — fetching data from an API:

```svelte
<script>
  let userId = $state(1);
  let user = $state(null);
  let loading = $state(false);
  let error = $state(null);

  $effect(() => {
    // This runs whenever userId changes
    const currentId = userId; // Capture the current value
    
    loading = true;
    error = null;
    
    fetch(`https://jsonplaceholder.typicode.com/users/${currentId}`)
      .then(res => {
        if (!res.ok) throw new Error('User not found');
        return res.json();
      })
      .then(data => {
        user = data;
        loading = false;
      })
      .catch(err => {
        error = err.message;
        loading = false;
      });
  });
</script>

<div class="user-selector">
  <button onclick={() => userId = 1}>User 1</button>
  <button onclick={() => userId = 2}>User 2</button>
  <button onclick={() => userId = 3}>User 3</button>
</div>

{#if loading}
  <p>Loading...</p>
{:else if error}
  <p class="error">Error: {error}</p>
{:else if user}
  <div class="user-card">
    <h2>{user.name}</h2>
    <p>Email: {user.email}</p>
    <p>Phone: {user.phone}</p>
    <p>Company: {user.company.name}</p>
  </div>
{/if}
```

Click each button and watch the user data change. The `$effect` re-runs whenever `userId` changes, fetching the new user's information.

> **⚠️ Watch Out:** Be careful with `$effect` and async code. The effect function itself should be synchronous, but you can call async functions inside it. Also, if you change state inside an effect, make sure you're not creating an infinite loop!

**Cleanup in effects:** If your effect sets up something that needs to be cleaned up (like a timer or a WebSocket connection), return a cleanup function:

```svelte
<script>
  let count = $state(0);
  let intervalId;

  $effect(() => {
    intervalId = setInterval(() => {
      count++;
    }, 1000);

    // This runs when the effect is re-run or the component is destroyed
    return () => {
      clearInterval(intervalId);
    };
  });
</script>

<p>Auto-incrementing: {count}</p>
```

This sets up an interval that increments `count` every second. The cleanup function clears the interval when the component is destroyed. It's like a responsible guest who always cleans up before leaving!

**Another example — document title sync:**

```svelte
<script>
  let pageTitle = $state('My App');
  let notificationCount = $state(0);

  $effect(() => {
    // This updates the browser tab title
    const title = notificationCount > 0 
      ? `(${notificationCount}) ${pageTitle}` 
      : pageTitle;
    document.title = title;
  });
</script>

<input bind:value={pageTitle} placeholder="Page title" />
<button onclick={() => notificationCount++}>
  Notify ({notificationCount})
</button>
```

### $props: Receiving Data from Parent Components

Components don't live in isolation — they receive data from their parents. The `$props` rune is how you declare what data a component accepts.

```svelte
<!-- Button.svelte -->
<script>
  let { text, variant = 'primary', size = 'medium', disabled = false, onclick } = $props();
</script>

<button class="btn btn-{variant} btn-{size}" {disabled} {onclick}>
  {text}
</button>
```

Now you can use this button in any parent component:

```svelte
<script>
  import Button from './Button.svelte';

  function handleClick() {
    alert('Button clicked!');
  }
  
  function handleSave() {
    alert('Saved!');
  }
  
  function handleDelete() {
    if (confirm('Are you sure?')) {
      alert('Deleted!');
    }
  }
</script>

<Button text="Save" variant="primary" onclick={handleSave} />
<Button text="Cancel" variant="secondary" onclick={handleClick} />
<Button text="Delete" variant="danger" onclick={handleDelete} />
<Button text="Small Button" variant="primary" size="small" />
<Button text="Disabled" variant="primary" disabled />
```

Let's break down the syntax:

```javascript
let { text, variant = 'primary', size = 'medium', disabled = false, onclick } = $props();
```

This is **destructuring** — it pulls specific properties out of the props object. It's like opening a gift box and taking out just the things you need.

- `text` — a required prop (no default value). If you don't pass it, it'll be `undefined`.
- `variant = 'primary'` — an optional prop with a default value of `'primary'`. If you don't pass it, it defaults to `'primary'`.
- `size = 'medium'` — an optional prop with a default.
- `disabled = false` — an optional boolean prop, defaults to `false`.
- `onclick` — an event handler passed from the parent.

### Default Props

Default props give your components sensible defaults so the parent doesn't always have to provide every value. This makes your components more flexible and easier to use.

```svelte
<!-- Card.svelte -->
<script>
  let {
    title = 'Untitled Card',
    content = '',
    image = null,
    variant = 'default',  // 'default', 'outlined', 'elevated'
    actions = []
  } = $props();
</script>

<div class="card card-{variant}">
  {#if image}
    <img src={image} alt={title} class="card-image" />
  {/if}
  <div class="card-body">
    <h3>{title}</h3>
    <p>{content}</p>
    {#if actions.length > 0}
      <div class="card-actions">
        {#each actions as action}
          <button onclick={action.handler}>{action.label}</button>
        {/each}
      </div>
    {/if}
  </div>
</div>
```

Now a parent can create cards with different levels of detail:

```svelte
<!-- Minimal card — uses all defaults -->
<Card />

<!-- Card with just a title -->
<Card title="My Card" />

<!-- Card with title and content -->
<Card title="Product" content="This is an amazing product" />

<!-- Full card with image and actions -->
<Card
  title="Product"
  content="This is an amazing product"
  image="/product.jpg"
  variant="elevated"
  actions={[
    { label: 'Buy', handler: () => buyProduct() },
    { label: 'Details', handler: () => showDetails() }
  ]}
/>
```

The default props mean the component always works, even when you don't provide everything. This is the "sensible defaults" philosophy — your component should work out of the box with sensible behavior.

### $bindable: Two-Way Binding

Sometimes a parent needs to pass a value to a child, and the child needs to send it back. This is called **two-way binding**. The `$bindable` rune makes this easy.

```svelte
<!-- Slider.svelte -->
<script>
  let { value = $bindable(0), min = 0, max = 100, label = 'Value' } = $props();
</script>

<div class="slider">
  <label>{label}: {value}</label>
  <input
    type="range"
    {min}
    {max}
    bind:value
  />
</div>
```

Now the parent can use it with `bind:value`:

```svelte
<script>
  import Slider from './Slider.svelte';
  let volume = $state(50);
  let brightness = $state(75);
  let contrast = $state(60);
</script>

<h2>Audio: {volume}%</h2>
<Slider bind:value={volume} min={0} max={100} label="Volume" />

<h2>Display</h2>
<Slider bind:value={brightness} min={0} max={100} label="Brightness" />
<Slider bind:value={contrast} min={0} max={100} label="Contrast" />
```

When you move the slider, `volume` in the parent updates. When the parent changes `volume` programmatically, the slider updates. It's a two-way street!

> **💡 Think of it like a walkie-talkie:** The parent says "set volume to 70" (passing the value down), and the kid says "actually, I moved it to 85" (sending the value back up). Both sides stay in sync.

**Another example — a color picker:**

```svelte
<!-- ColorInput.svelte -->
<script>
  let { value = $bindable('#3b82f6'), label = 'Color' } = $props();
</script>

<div class="color-input">
  <label>{label}</label>
  <input type="color" bind:value />
  <span>{value}</span>
</div>
```

```svelte
<!-- Parent -->
<script>
  import ColorInput from './ColorInput.svelte';
  let primaryColor = $state('#3b82f6');
  let secondaryColor = $state('#6b7280');
</script>

<ColorInput bind:value={primaryColor} label="Primary" />
<ColorInput bind:value={secondaryColor} label="Secondary" />

<div class="preview" style="background: {primaryColor}; color: {secondaryColor}">
  <p>This text uses your colors!</p>
</div>
```

### Why Runes are Better Than Svelte 4 Stores

If you've seen older Svelte tutorials, you might have encountered **stores** — the previous way to share state between components. Runes are a major improvement. Here's why:

**1. No more special syntax**
```javascript
// Old way (Svelte 4)
import { writable } from 'svelte/store';
let count = writable(0);
$: doubled = $count * 2;  // Special $ prefix

// New way (Svelte 5)
let count = $state(0);
let doubled = $derived(count * 2);  // Just regular code
```

**2. Better TypeScript support**
Runes are TypeScript-native. The old store system was tricky to type correctly. With runes, you get full type inference for free. Your editor can autocomplete, catch errors, and help you refactor.

**3. No more $ prefix**
In Svelte 4 stores, you had to use `$count` to access the value. This was confusing — where did the `$` come from? Is it a variable? A store? A rune? With runes, you just use `count`. Simple!

**4. Fine-grained reactivity**
Runes track which specific properties of objects and arrays you use. If you only read `user.name`, only the DOM elements using `user.name` will update when `user` changes. The old system wasn't this precise.

**5. Composability**
Runes work in any `.svelte` or `.svelte.js` file. Stores were global and could get tangled. Runes are local and scoped, making your code easier to understand and maintain.

```javascript
// Old way: stores lived in separate files
// stores/count.js
import { writable } from 'svelte/store';
export const count = writable(0);

// stores/todos.js
import { writable } from 'svelte/store';
export const todos = writable([]);

// Component had to import and use $ prefix
<script>
  import { count } from './stores/count';
  import { todos } from './stores/todos';
  // Then use $count and $todos in template
</script>

// New way: runes live right in your component
let count = $state(0);
let todos = $state([]);
```

> **🔍 Fun Fact:** The runes system was inspired by signals — a reactivity pattern used in frameworks like Solid.js and Angular. Rich Harris studied these approaches and designed Svelte 5's runes to be even simpler and more powerful. He wanted the best ideas from the JavaScript ecosystem, but with Svelte's signature simplicity.

### Common Pitfalls: Don't Update State Inside $derived

This is one of the most important rules in Svelte 5: **never modify state inside `$derived` or `$derived.by`**.

```svelte
<script>
  let count = $state(0);
  
  // ❌ WRONG: This will cause an error!
  let doubled = $derived.by(() => {
    count = count * 2;  // Don't do this!
    return count;
  });
  
  // ✅ CORRECT: Read only, compute and return
  let doubled = $derived(count * 2);
</script>
```

Why? Because `$derived` is meant to be a **pure computation** — it reads state and returns a value. If you change state inside it, you'd create a loop: state changes → derived runs → state changes again → derived runs again → infinite loop!

Think of `$derived` like a calculator. A calculator takes in numbers and gives you a result. You wouldn't expect a calculator to also change the numbers on the paper while computing — that would be chaos!

**Other common pitfalls:**

1. **Forgetting that `$state` copies objects:**
```svelte
<script>
  let user = $state({ name: 'Alice', settings: { theme: 'dark' } });
  
  // This works:
  user.name = 'Bob';  // ✅ Svelte tracks this
  
  // This ALSO works (deeply nested):
  user.settings.theme = 'light';  // ✅ Svelte tracks nested properties
  
  // But be careful with arrays:
  let items = $state([1, 2, 3]);
  // items.push(4)  // ❌ Won't trigger reactivity!
  items = [...items, 4];  // ✅ This will trigger reactivity
</script>
```

2. **Not capturing values in effects:**
```svelte
<script>
  let userId = $state(1);
  
  // ❌ May cause issues (userId might change during fetch)
  $effect(() => {
    fetch(`/api/users/${userId}`).then(/* ... */);
  });
  
  // ✅ Better: capture the value
  $effect(() => {
    const id = userId;  // Capture current value
    fetch(`/api/users/${id}`).then(/* ... */);
  });
</script>
```

3. **Using $derived for side effects:**
```svelte
<script>
  let count = $state(0);
  
  // ❌ WRONG: Don't use $derived for side effects
  let result = $derived(() => {
    console.log('count changed');  // This is a side effect!
    return count * 2;
  });
  
  // ✅ CORRECT: Use $effect for side effects
  $effect(() => {
    console.log('count changed to:', count);
  });
  
  // ✅ CORRECT: Use $derived for computed values
  let result = $derived(count * 2);
</script>
```

### Practice: Build a Counter, a Todo List, a Form

Let's put everything together! Here are three exercises for you.

#### Exercise 1: A Fancy Counter

```svelte
<!-- Counter.svelte -->
<script>
  let count = $state(0);
  let step = $state(1);
  let history = $state([]);
  
  let doubled = $derived(count * 2);
  let tripled = $derived(count * 3);
  let isPositive = $derived(count > 0);
  let isNegative = $derived(count < 0);
  let isZero = $derived(count === 0);
  
  let historyCount = $derived(history.length);
  
  function increment() {
    history = [...history, { action: 'increment', value: count, newValue: count + step }];
    count += step;
  }
  
  function decrement() {
    history = [...history, { action: 'decrement', value: count, newValue: count - step }];
    count -= step;
  }
  
  function reset() {
    history = [...history, { action: 'reset', value: count, newValue: 0 }];
    count = 0;
  }
  
  function clearHistory() {
    history = [];
  }
  
  $effect(() => {
    document.title = `Count: ${count}`;
  });
</script>

<div class="counter">
  <h1>Counter</h1>
  <p class="count" class:positive={isPositive} class:negative={isNegative} class:zero={isZero}>
    {count}
  </p>
  <p>Doubled: {doubled} | Tripled: {tripled}</p>
  
  <div class="controls">
    <label>
      Step:
      <input type="number" bind:value={step} min="1" max="10" />
    </label>
  </div>
  
  <div class="buttons">
    <button onclick={decrement}>-{step}</button>
    <button onclick={reset}>Reset</button>
    <button onclick={increment}>+{step}</button>
  </div>
  
  {#if history.length > 0}
    <div class="history">
      <h3>History ({historyCount} entries)</h3>
      {#each history.slice(-5).reverse() as entry}
        <p>
          {entry.action}: {entry.value} → {entry.newValue}
        </p>
      {/each}
      <button onclick={clearHistory}>Clear History</button>
    </div>
  {/if}
</div>
```

#### Exercise 2: A Todo List

```svelte
<!-- TodoList.svelte -->
<script>
  let todos = $state([
    { id: 1, text: 'Learn $state', done: true },
    { id: 2, text: 'Learn $derived', done: false },
    { id: 3, text: 'Learn $effect', done: false }
  ]);
  
  let newTodo = $state('');
  let filter = $state('all'); // 'all', 'done', 'pending'
  
  let filteredTodos = $derived.by(() => {
    switch (filter) {
      case 'done':
        return todos.filter(t => t.done);
      case 'pending':
        return todos.filter(t => !t.done);
      default:
        return todos;
    }
  });
  
  let stats = $derived.by(() => {
    const total = todos.length;
    const done = todos.filter(t => t.done).length;
    return { total, done, pending: total - done };
  });
  
  let nextId = $derived(Math.max(...todos.map(t => t.id), 0) + 1);
  let canAdd = $derived(newTodo.trim().length > 0);
  
  function addTodo() {
    if (canAdd) {
      todos = [...todos, { id: nextId, text: newTodo.trim(), done: false }];
      newTodo = '';
    }
  }
  
  function toggleTodo(id) {
    const todo = todos.find(t => t.id === id);
    if (todo) {
      todo.done = !todo.done;
    }
  }
  
  function deleteTodo(id) {
    todos = todos.filter(t => t.id !== id);
  }
  
  function handleKeydown(event) {
    if (event.key === 'Enter') {
      addTodo();
    }
  }
  
  $effect(() => {
    console.log(`Todos: ${stats.done}/${stats.total} completed`);
  });
</script>

<div class="todo-list">
  <h1>Todo List</h1>
  <p>{stats.done} of {stats.total} completed</p>
  
  <div class="add-form">
    <input
      type="text"
      bind:value={newTodo}
      placeholder="What needs to be done?"
      onkeydown={handleKeydown}
    />
    <button onclick={addTodo} disabled={!canAdd}>Add</button>
  </div>
  
  <div class="filters">
    <button class:active={filter === 'all'} onclick={() => filter = 'all'}>
      All ({stats.total})
    </button>
    <button class:active={filter === 'done'} onclick={() => filter = 'done'}>
      Done ({stats.done})
    </button>
    <button class:active={filter === 'pending'} onclick={() => filter = 'pending'}>
      Pending ({stats.pending})
    </button>
  </div>
  
  <ul>
    {#each filteredTodos as todo (todo.id)}
      <li class:done={todo.done}>
        <input
          type="checkbox"
          checked={todo.done}
          onchange={() => toggleTodo(todo.id)}
        />
        <span>{todo.text}</span>
        <button class="delete" onclick={() => deleteTodo(todo.id)}>×</button>
      </li>
    {/each}
  </ul>
  
  {#if filteredTodos.length === 0}
    <p class="empty">
      {#if filter === 'all'}
        No todos yet! Add one above.
      {:else if filter === 'done'}
        No completed todos.
      {:else}
        No pending todos — all done! 🎉
      {/if}
    </p>
  {/if}
</div>
```

#### Exercise 3: A Reactive Form

```svelte
<!-- ReactiveForm.svelte -->
<script>
  let form = $state({
    username: '',
    email: '',
    password: '',
    confirmPassword: '',
    age: ''
  });
  
  let errors = $derived.by(() => {
    const errs = {};
    
    if (form.username.length < 3 && form.username.length > 0) {
      errs.username = 'Username must be at least 3 characters';
    }
    
    if (form.username.length > 20) {
      errs.username = 'Username must be less than 20 characters';
    }
    
    if (form.email && !form.email.includes('@')) {
      errs.email = 'Please enter a valid email';
    }
    
    if (form.password.length > 0 && form.password.length < 8) {
      errs.password = 'Password must be at least 8 characters';
    }
    
    if (form.password && !/[A-Z]/.test(form.password)) {
      errs.password = 'Password must contain an uppercase letter';
    }
    
    if (form.password && !/[0-9]/.test(form.password)) {
      errs.password = 'Password must contain a number';
    }
    
    if (form.password !== form.confirmPassword && form.confirmPassword) {
      errs.confirmPassword = 'Passwords do not match';
    }
    
    if (form.age && (parseInt(form.age) < 13 || parseInt(form.age) > 120)) {
      errs.age = 'Age must be between 13 and 120';
    }
    
    return errs;
  });
  
  let errorCount = $derived(Object.keys(errors).length);
  let isValid = $derived(errorCount === 0);
  let hasAllFields = $derived(
    form.username && form.email && form.password && form.confirmPassword
  );
  
  // Password strength indicator
  let passwordStrength = $derived.by(() => {
    let strength = 0;
    if (form.password.length >= 8) strength++;
    if (form.password.length >= 12) strength++;
    if (/[A-Z]/.test(form.password)) strength++;
    if (/[0-9]/.test(form.password)) strength++;
    if (/[^A-Za-z0-9]/.test(form.password)) strength++;
    return strength;
  });
  
  function handleSubmit() {
    if (isValid && hasAllFields) {
      alert(`Welcome, ${form.username}! Your account has been created.`);
      // Reset form
      form.username = '';
      form.email = '';
      form.password = '';
      form.confirmPassword = '';
      form.age = '';
    }
  }
</script>

<div class="form-container">
  <h1>Create Account</h1>
  
  {#if errorCount > 0}
    <p class="error-count">{errorCount} error{errorCount === 1 ? '' : 's'} found</p>
  {/if}
  
  <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
    <div class="field">
      <label for="username">Username</label>
      <input
        id="username"
        type="text"
        bind:value={form.username}
        class:error={errors.username}
        placeholder="Choose a username"
      />
      {#if errors.username}
        <span class="error-text">{errors.username}</span>
      {/if}
    </div>
    
    <div class="field">
      <label for="email">Email</label>
      <input
        id="email"
        type="email"
        bind:value={form.email}
        class:error={errors.email}
        placeholder="you@example.com"
      />
      {#if errors.email}
        <span class="error-text">{errors.email}</span>
      {/if}
    </div>
    
    <div class="field">
      <label for="password">Password</label>
      <input
        id="password"
        type="password"
        bind:value={form.password}
        class:error={errors.password}
        placeholder="At least 8 characters"
      />
      {#if form.password}
        <div class="strength">
          <div class="strength-bar">
            <div
              class="strength-fill"
              class:weak={passwordStrength <= 1}
              class:medium={passwordStrength >= 2 && passwordStrength <= 3}
              class:strong={passwordStrength >= 4}
              style="width: {passwordStrength * 20}%"
            ></div>
          </div>
          <span>
            {#if passwordStrength <= 1}Weak
            {:else if passwordStrength <= 3}Medium
            {:else}Strong
            {/if}
          </span>
        </div>
      {/if}
      {#if errors.password}
        <span class="error-text">{errors.password}</span>
      {/if}
    </div>
    
    <div class="field">
      <label for="confirm">Confirm Password</label>
      <input
        id="confirm"
        type="password"
        bind:value={form.confirmPassword}
        class:error={errors.confirmPassword}
        placeholder="Type your password again"
      />
      {#if errors.confirmPassword}
        <span class="error-text">{errors.confirmPassword}</span>
      {/if}
    </div>
    
    <div class="field">
      <label for="age">Age (optional)</label>
      <input
        id="age"
        type="number"
        bind:value={form.age}
        class:error={errors.age}
        placeholder="Your age"
      />
      {#if errors.age}
        <span class="error-text">{errors.age}</span>
      {/if}
    </div>
    
    <button type="submit" disabled={!isValid || !hasAllFields}>
      Create Account
    </button>
  </form>
</div>
```

> **🧪 Try It Yourself:** Copy these three examples into separate Svelte files and run your dev server. Try modifying them — add a dark mode toggle to the counter, or add categories to the todo list, or add a "show password" toggle to the form. The best way to learn is to experiment!

### Wrapping Up Chapter 7

Wow, that was a lot! Let's recap the runes:

- **`$state`** — Makes a variable reactive. When it changes, the DOM updates. Use it for any data that needs to change over time.
- **`$derived`** — A computed value that automatically updates when its dependencies change. Use it for values that depend on other state.
- **`$derived.by`** — Like `$derived` but for complex computations that need a function. Use it when a single expression isn't enough.
- **`$effect`** — Runs code whenever its dependencies change. Used for side effects like DOM manipulation, API calls, or timers.
- **`$props`** — Declares what data a component accepts from its parent. Use it in every component that needs data from outside.
- **`$bindable`** — Enables two-way binding between parent and child components. Use it when both parent and child need to update the same value.

The key rules to remember:
1. `$derived` should never modify state — only read and compute
2. `$effect` should clean up after itself — return cleanup functions for timers and listeners
3. Runes make reactivity explicit and powerful — you choose what's reactive
4. Default props make components more reusable — always provide sensible defaults
5. `$bindable` is for two-way data flow — parent and child stay in sync

In the next chapter, we'll explore **components and props** in more depth — building reusable pieces that you can snap together like LEGO bricks. Let's keep building!

---

## Chapter 8: Components and Props

### What is a Component? (A LEGO Brick)

Imagine you're building a LEGO castle. You don't carve every single brick from scratch — you use pre-made pieces that snap together. A window piece, a door piece, a wall piece, a turret piece. Each piece does one thing well, and you combine them to make something amazing.

A **component** in Svelte is exactly like a LEGO brick. It's a self-contained piece of your app that has its own:
- **Markup** (HTML) — what it looks like
- **Script** (JavaScript) — what it does
- **Style** (CSS) — how it's styled

You create a component once, and then you can use it anywhere in your app — just like a LEGO brick can be used in any part of your castle.

Here's the beautiful thing: components can also contain other components. Your Button component might be inside a Card component, which might be inside a Modal component, which might be on a Page component. It's components all the way down! Like Russian nesting dolls, but for web development.

Why components are awesome:
1. **Reusability** — Write once, use everywhere. That button? Use it on the home page, the settings page, the dashboard page. Every time.
2. **Consistency** — Every button looks and behaves the same. No accidental differences.
3. **Maintainability** — Change the button once, and every instance updates. No hunting through files.
4. **Testability** — Test the button in isolation. Does it click? Does it look right? Yes? Then it works everywhere.
5. **Organization** — Your code is broken into small, understandable pieces. Each file has one job.

### Creating a Component: Button.svelte

Let's create our first real component. In your SvelteKit project, create a new folder called `src/lib/components` and add a file called `Button.svelte`:

```svelte
<!-- src/lib/components/Button.svelte -->
<script>
  let {
    text = 'Click me',
    variant = 'primary',
    size = 'medium',
    disabled = false,
    loading = false,
    onclick
  } = $props();
</script>

<button
  class="btn btn-{variant} btn-{size}"
  {disabled}
  {onclick}
>
  {#if loading}
    <span class="spinner"></span>
  {/if}
  {text}
</button>

<style>
  .btn {
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-weight: 600;
    transition: all 0.2s ease;
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  
  .btn:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }
  
  .btn:active:not(:disabled) {
    transform: translateY(0);
  }
  
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }
  
  /* Variants */
  .btn-primary {
    background: #3b82f6;
    color: white;
  }
  
  .btn-primary:hover:not(:disabled) {
    background: #2563eb;
  }
  
  .btn-secondary {
    background: #6b7280;
    color: white;
  }
  
  .btn-secondary:hover:not(:disabled) {
    background: #4b5563;
  }
  
  .btn-danger {
    background: #ef4444;
    color: white;
  }
  
  .btn-danger:hover:not(:disabled) {
    background: #dc2626;
  }
  
  .btn-success {
    background: #22c55e;
    color: white;
  }
  
  .btn-success:hover:not(:disabled) {
    background: #16a34a;
  }
  
  .btn-outline {
    background: transparent;
    color: #3b82f6;
    border: 2px solid #3b82f6;
  }
  
  .btn-outline:hover:not(:disabled) {
    background: #3b82f6;
    color: white;
  }
  
  /* Sizes */
  .btn-small {
    padding: 6px 12px;
    font-size: 0.875rem;
  }
  
  .btn-medium {
    padding: 10px 20px;
    font-size: 1rem;
  }
  
  .btn-large {
    padding: 14px 28px;
    font-size: 1.125rem;
  }
  
  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
```

Now you can use this Button anywhere in your app:

```svelte
<!-- src/routes/+page.svelte -->
<script>
  import Button from '$lib/components/Button.svelte';
  
  let loading = $state(false);
  
  async function handleSave() {
    loading = true;
    await new Promise(resolve => setTimeout(resolve, 2000)); // Simulate API call
    loading = false;
    alert('Saved!');
  }
</script>

<h1>Button Gallery</h1>

<div class="button-grid">
  <Button text="Save" variant="primary" />
  <Button text="Cancel" variant="secondary" />
  <Button text="Delete" variant="danger" />
  <Button text="Success!" variant="success" />
  <Button text="Outline" variant="outline" />
  
  <h2>Sizes</h2>
  <Button text="Small" variant="primary" size="small" />
  <Button text="Medium" variant="primary" size="medium" />
  <Button text="Large" variant="primary" size="large" />
  
  <h2>States</h2>
  <Button text="Disabled" variant="primary" disabled />
  <Button text={loading ? 'Saving...' : 'Save'} variant="primary" loading={loading} onclick={handleSave} />
</div>
```

See how clean that is? Every button in your app uses the same component, ensuring consistent styling. If you want to change the border radius of all buttons, you edit one file — `Button.svelte` — and every button updates. That's the power of components!

> **💡 The `$lib` alias:** In SvelteKit, `$lib` is a shortcut that points to your `src/lib` folder. So `$lib/components/Button.svelte` is the same as `../lib/components/Button.svelte`. It's cleaner and easier to read, and it works from anywhere in your project.

### Passing Data with Props: let { name } = $props()

We touched on props in the last chapter, but let's go deeper. Props are the way parent components pass data down to child components. They're the component's "configuration" — you tell the component what to do, and it does it.

```svelte
<!-- Greeting.svelte -->
<script>
  let { name, emoji = '👋', excited = false } = $props();
  
  let greeting = $derived(
    excited ? `${emoji} HELLO, ${name.toUpperCase()}!!! ${emoji}` : `${emoji} Hello, ${name}!`
  );
</script>

<h2>{greeting}</h2>
```

Using it:

```svelte
<script>
  import Greeting from './Greeting.svelte';
</script>

<Greeting name="Alice" />
<Greeting name="Bob" emoji="🎉" />
<Greeting name="Charlie" emoji="🚀" excited />
```

Props flow in **one direction** — from parent to child. Think of it like giving someone a gift. You (the parent) give a gift (data) to your friend (the child). Your friend uses the gift, but they can't reach into your pocket and change what you gave them.

This one-way flow makes your app predictable and easier to debug. If something goes wrong with a component, you know exactly where the data came from. No magic, no surprises.

**Multiple props:** You can pass as many props as you need:

```svelte
<!-- UserProfile.svelte -->
<script>
  let { name, email, avatar, bio = 'No bio yet', joinedDate, role = 'user' } = $props();
</script>

<div class="profile">
  <img src={avatar} alt={name} class="avatar" />
  <div class="info">
    <h2>{name}</h2>
    <span class="role role-{role}">{role}</span>
    <p class="email">{email}</p>
    <p class="bio">{bio}</p>
    <p class="joined">Joined: {joinedDate}</p>
  </div>
</div>
```

**Passing different types:** Props can be strings, numbers, booleans, arrays, objects, or functions:

```svelte
<script>
  import UserProfile from './UserProfile.svelte';
  
  let user = $state({
    name: 'Ada Lovelace',
    email: 'ada@example.com',
    avatar: '/avatars/ada.jpg',
    bio: 'First programmer ever',
    joinedDate: '1843',
    role: 'admin'
  });
</script>

<!-- Passing different types -->
<UserProfile
  name={user.name}           <!-- string -->
  email={user.email}          <!-- string -->
  avatar={user.avatar}        <!-- string -->
  bio={user.bio}              <!-- string -->
  joinedDate={user.joinedDate} <!-- string -->
  role={user.role}            <!-- string -->
/>
```

### Children Props: {@render children()}

Sometimes you want a component that wraps around other content — like a Card that contains any content you put inside it. In Svelte 5, you use the `children` prop for this:

```svelte
<!-- Card.svelte -->
<script>
  let { title, subtitle, children, actions } = $props();
</script>

<div class="card">
  {#if title}
    <div class="card-header">
      <h3>{title}</h3>
      {#if subtitle}
        <p class="subtitle">{subtitle}</p>
      {/if}
    </div>
  {/if}
  <div class="card-body">
    {@render children()}
  </div>
  {#if actions}
    <div class="card-footer">
      {@render actions()}
    </div>
  {/if}
</div>

<style>
  .card {
    background: white;
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
    overflow: hidden;
  }
  
  .card-header {
    padding: 20px 20px 0;
  }
  
  .card-header h3 {
    margin: 0;
  }
  
  .subtitle {
    color: #666;
    margin: 4px 0 0;
  }
  
  .card-body {
    padding: 20px;
  }
  
  .card-footer {
    padding: 12px 20px;
    border-top: 1px solid #eee;
    background: #f9f9f9;
  }
</style>
```

Using it:

```svelte
<script>
  import Card from './Card.svelte';
</script>

<!-- Simple card -->
<Card title="My Card">
  <p>This content goes inside the card body!</p>
  <p>It can be anything — HTML, other components, whatever you need.</p>
</Card>

<!-- Card with subtitle -->
<Card title="Welcome Back" subtitle="Here's what's new">
  <p>You have 3 new notifications.</p>
</Card>

<!-- Card with actions slot -->
<Card title="Confirm Action">
  <p>Are you sure you want to delete this item?</p>
  
  {#snippet actions()}
    <button class="cancel">Cancel</button>
    <button class="danger">Delete</button>
  {/snippet}
</Card>
```

The `{@render children()}` is where the content between the component tags gets rendered. Think of `children` as a **placeholder** — it says "put whatever the parent gives me right here."

> **⚠️ Watch Out:** `children` is a special prop name. Don't use it for regular data — it's specifically for the content that goes between a component's opening and closing tags. Using it for anything else will cause confusion.

### Event Handling: onclick, onchange

Components need to communicate with their parents. One way to do this is through **event callbacks** — functions the parent passes to the child.

```svelte
<!-- IconButton.svelte -->
<script>
  let { icon, label, onclick } = $props();
</script>

<button class="icon-btn" {onclick} title={label}>
  <span class="icon">{icon}</span>
  <span class="label">{label}</span>
</button>

<style>
  .icon-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    border: 2px solid #e2e8f0;
    border-radius: 8px;
    background: white;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .icon-btn:hover {
    border-color: #3b82f6;
    color: #3b82f6;
  }
</style>
```

Parent handles the event:

```svelte
<script>
  import IconButton from './IconButton.svelte';
  
  let log = $state([]);
  
  function handleSave() {
    log = [...log, { time: new Date().toLocaleTimeString(), action: 'Saved' }];
  }
  
  function handleDelete() {
    if (confirm('Delete this?')) {
      log = [...log, { time: new Date().toLocaleTimeString(), action: 'Deleted' }];
    }
  }
  
  function handleShare() {
    log = [...log, { time: new Date().toLocaleTimeString(), action: 'Shared' }];
  }
</script>

<div class="toolbar">
  <IconButton icon="💾" label="Save" onclick={handleSave} />
  <IconButton icon="🗑️" label="Delete" onclick={handleDelete} />
  <IconButton icon="🔗" label="Share" onclick={handleShare} />
</div>

{#if log.length > 0}
  <h3>Activity Log</h3>
  {#each log as entry}
    <p>{entry.time} — {entry.action}</p>
  {/each}
{/if}
```

You can also pass event data back from the child:

```svelte
<!-- ColorPicker.svelte -->
<script>
  let { colors = ['#ff0000', '#00ff00', '#0000ff'], onchange } = $props();
  let selected = $state(colors[0]);
  
  function selectColor(color) {
    selected = color;
    onchange?.(color);  // Call parent's handler with the color
  }
</script>

<div class="color-picker">
  {#each colors as color}
    <button
      class="color-swatch"
      class:selected={selected === color}
      style="background: {color}"
      onclick={() => selectColor(color)}
      title={color}
    ></button>
  {/each}
</div>

<style>
  .color-picker {
    display: flex;
    gap: 8px;
  }
  
  .color-swatch {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    border: 3px solid transparent;
    cursor: pointer;
    transition: transform 0.2s;
  }
  
  .color-swatch:hover {
    transform: scale(1.1);
  }
  
  .color-swatch.selected {
    border-color: #1e293b;
    box-shadow: 0 0 0 2px white, 0 0 0 4px #1e293b;
  }
</style>
```

Parent handles it:

```svelte
<script>
  import ColorPicker from './ColorPicker.svelte';
  let selectedColor = $state('#ff0000');
</script>

<div class="preview" style="background: {selectedColor}">
  <p>Selected: {selectedColor}</p>
</div>

<ColorPicker
  colors={['#ff0000', '#00ff00', '#0000ff', '#ff00ff', '#00ffff', '#ffff00']}
  onchange={(color) => selectedColor = color}
/>
```

> **💡 The `?.` operator:** Notice `onchange?.(color)` in the ColorPicker. The `?.` means "if onchange exists, call it; otherwise, do nothing." This makes the prop optional — the component works fine even if no event handler is provided. It's defensive programming at its simplest.

### Conditional Rendering: {#if}...{:else}...{/if}

Sometimes you want to show different things based on a condition. Svelte gives you `{#if}` blocks for this:

```svelte
<script>
  let isLoggedIn = $state(false);
  let user = $state({ name: 'Alice', role: 'admin' });
  let notifications = $state(3);
</script>

{#if isLoggedIn}
  <div class="welcome">
    <h1>Welcome back, {user.name}! 👋</h1>
    <p>You are logged in as {user.role}.</p>
    {#if notifications > 0}
      <p class="badge">You have {notifications} new notification{notifications === 1 ? '' : 's'}!</p>
    {/if}
    <button onclick={() => isLoggedIn = false}>Log Out</button>
  </div>
{:else}
  <div class="login">
    <h1>Welcome, stranger! 🤔</h1>
    <p>Please log in to continue.</p>
    <button onclick={() => isLoggedIn = true}>Log In</button>
  </div>
{/if}
```

This is like a bouncer at a club — if you're on the list (logged in), you get the VIP treatment. If not, you see the "please log in" message.

**Chaining conditions:** You can chain multiple conditions:

```svelte
<script>
  let temperature = $state(72);
</script>

{#if temperature > 90}
  <p>🔥 It's blazing hot! Stay hydrated!</p>
{:else if temperature > 70}
  <p>☀️ Nice weather! Perfect for a walk.</p>
{:else if temperature > 50}
  <p>🧥 A bit chilly. Bring a jacket!</p>
{:else if temperature > 32}
  <p>❄️ Cold! Bundle up.</p>
{:else}
  <p>🥶 Below freezing! Stay warm inside!</p>
{/if}

<input type="range" bind:value={temperature} min="0" max="110" />
<p>Temperature: {temperature}°F</p>
```

**Nested conditions:** You can nest `{#if}` blocks inside each other:

```svelte
<script>
  let user = $state({ isLoggedIn: true, role: 'admin' });
</script>

{#if user.isLoggedIn}
  <h1>Welcome!</h1>
  {#if user.role === 'admin'}
    <p>You have admin access. 🔒</p>
    <a href="/admin">Go to Admin Panel</a>
  {:else if user.role === 'moderator'}
    <p>You have moderator access. 🛡️</p>
    <a href="/mod">Go to Mod Panel</a>
  {:else}
    <p>You're a regular user. 👤</p>
    <a href="/dashboard">Go to Dashboard</a>
  {/if}
{:else}
  <p>Please log in. 🔑</p>
{/if}
```

### List Rendering: {#each items as item (item.id)}

Most apps show lists of things — users, products, messages, etc. The `{#each}` block lets you render a list from an array:

```svelte
<script>
  let fruits = $state([
    { id: 1, name: '🍎 Apple', color: 'red', emoji: '🍎' },
    { id: 2, name: '🍌 Banana', color: 'yellow', emoji: '🍌' },
    { id: 3, name: '🍇 Grape', color: 'purple', emoji: '🍇' },
    { id: 4, name: '🍊 Orange', color: 'orange', emoji: '🍊' },
    { id: 5, name: '🍓 Strawberry', color: 'red', emoji: '🍓' }
  ]);
  
  let searchTerm = $state('');
  
  let filteredFruits = $derived(
    fruits.filter(f => f.name.toLowerCase().includes(searchTerm.toLowerCase()))
  );
</script>

<input bind:value={searchTerm} placeholder="Search fruits..." />

<ul>
  {#each filteredFruits as fruit (fruit.id)}
    <li style="color: {fruit.color}">
      {fruit.emoji} {fruit.name}
    </li>
  {/each}
</ul>

<p>Total: {filteredFruits.length} of {fruits.length} fruits</p>
```

The `(fruit.id)` after `as fruit` is a **key**. It tells Svelte how to identify each item uniquely. Keys help Svelte update the list efficiently when items are added, removed, or reordered. Think of keys like name tags — they help Svelte keep track of who's who.

**Accessing the index:** You can also get the index (position) of each item:

```svelte
{#each fruits as fruit, index (fruit.id)}
  <p>{index + 1}. {fruit.name}</p>
{/each}
```

**Empty lists:** What if the array is empty? You can use the `{:else}` block:

```svelte
<script>
  let todos = $state([]);
  let newTodo = $state('');
  
  function addTodo() {
    if (newTodo.trim()) {
      todos = [...todos, { id: Date.now(), text: newTodo.trim() }];
      newTodo = '';
    }
  }
</script>

<input bind:value={newTodo} placeholder="Add a todo" onkeydown={(e) => e.key === 'Enter' && addTodo()} />
<button onclick={addTodo}>Add</button>

{#each todos as todo (todo.id)}
  <div class="todo">
    {todo.text}
  </div>
{:else}
  <p class="empty">No todos yet! Add one above. 📝</p>
{/each}
```

> **⚠️ Watch Out:** Always provide a key in `{#each}` blocks when items can be added, removed, or reordered. Without keys, Svelte might update the wrong elements, leading to subtle bugs. Use a unique identifier (like `id` or `slug`) as the key.

### Snippets: Reusable Markup Patterns

Sometimes you want to reuse a piece of markup in multiple places within a component. That's what **snippets** are for in Svelte 5.

A snippet is a reusable chunk of markup defined with the `{#snippet}` syntax:

```svelte
<script>
  let items = $state([
    { name: 'Alice', role: 'Developer', avatar: '👩‍💻' },
    { name: 'Bob', role: 'Designer', avatar: '👨‍🎨' },
    { name: 'Charlie', role: 'Manager', avatar: '👨‍💼' }
  ]);
</script>

{#snippet userCard(user)}
  <div class="user-card">
    <span class="avatar">{user.avatar}</span>
    <h3>{user.name}</h3>
    <p>{user.role}</p>
  </div>
{/snippet}

<!-- Use the snippet multiple times -->
<div class="users">
  {#each items as user}
    {@render userCard(user)}
  {/each}
</div>
```

Snippets are like **stencils** — you design the shape once, and then use it to trace the same shape over and over. If you want to change the shape, you edit the stencil, and every traced shape updates.

**More complex snippets with parameters:**

```svelte
<script>
  let alerts = $state([
    { type: 'success', message: 'Profile saved!' },
    { type: 'error', message: 'Something went wrong.' },
    { type: 'warning', message: 'Disk space low.' },
    { type: 'info', message: 'New update available.' }
  ]);
  
  let stats = $state({ users: 150, posts: 423, comments: 1892 });
</script>

{#snippet alert(type, message)}
  <div class="alert alert-{type}">
    {#if type === 'success'}
      <span class="icon">✅</span>
    {:else if type === 'error'}
      <span class="icon">❌</span>
    {:else if type === 'warning'}
      <span class="icon">⚠️</span>
    {:else}
      <span class="icon">ℹ️</span>
    {/if}
    <span class="message">{message}</span>
  </div>
{/snippet}

{#snippet statCard(label, value)}
  <div class="stat-card">
    <p class="stat-value">{value.toLocaleString()}</p>
    <p class="stat-label">{label}</p>
  </div>
{/snippet}

<!-- Render all alerts -->
{#each alerts as a}
  {@render alert(a.type, a.message)}
{/each}

<!-- Render stats -->
<div class="stats">
  {@render statCard('Users', stats.users)}
  {@render statCard('Posts', stats.posts)}
  {@render statCard('Comments', stats.comments)}
</div>
```

> **💡 Snippets vs Components:** Use snippets when you want to reuse markup *within* the same component. Use components when you want to reuse markup *across* different components. Snippets are private to the component they're defined in — like helper functions that only exist inside one file.

### Practice: Build a Card Component, a Modal Component

Let's build two practical components that you'll use in almost every app.

#### Exercise 1: A Flexible Card Component

```svelte
<!-- src/lib/components/Card.svelte -->
<script>
  let {
    title,
    subtitle,
    image,
    imageAlt,
    children,
    footer,
    variant = 'default', // 'default', 'bordered', 'elevated'
    hoverable = false,
    padding = true
  } = $props();
</script>

<div
  class="card card-{variant}"
  class:hoverable
  class:no-padding={!padding}
>
  {#if image}
    <div class="card-image">
      <img src={image} alt={imageAlt || title} />
    </div>
  {/if}
  
  <div class="card-content">
    {#if title}
      <h3 class="card-title">{title}</h3>
    {/if}
    {#if subtitle}
      <p class="card-subtitle">{subtitle}</p>
    {/if}
    
    {#if children}
      <div class="card-body">
        {@render children()}
      </div>
    {/if}
  </div>
  
  {#if footer}
    <div class="card-footer">
      {@render footer()}
    </div>
  {/if}
</div>

<style>
  .card {
    background: white;
    border-radius: 12px;
    overflow: hidden;
    transition: all 0.3s ease;
  }
  
  .card-default {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  }
  
  .card-bordered {
    border: 2px solid #e5e7eb;
  }
  
  .card-elevated {
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
  }
  
  .card.hoverable:hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  }
  
  .card-image img {
    width: 100%;
    height: 200px;
    object-fit: cover;
  }
  
  .card-content {
    padding: 20px;
  }
  
  .card.no-padding .card-content {
    padding: 0;
  }
  
  .card-title {
    margin: 0 0 8px 0;
    font-size: 1.25rem;
  }
  
  .card-subtitle {
    margin: 0;
    color: #666;
    font-size: 0.9rem;
  }
  
  .card-body {
    margin-top: 12px;
  }
  
  .card-footer {
    padding: 12px 20px;
    border-top: 1px solid #eee;
    background: #f9f9f9;
  }
</style>
```

#### Exercise 2: A Reusable Modal Component

```svelte
<!-- src/lib/components/Modal.svelte -->
<script>
  let {
    open = $bindable(false),
    title = 'Modal',
    size = 'medium', // 'small', 'medium', 'large'
    closable = true,
    children,
    onclose
  } = $props();
  
  function close() {
    if (closable) {
      open = false;
      onclose?.();
    }
  }
  
  function handleKeydown(e) {
    if (e.key === 'Escape' && closable) {
      close();
    }
  }
  
  function handleBackdropClick(e) {
    if (e.target === e.currentTarget) {
      close();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div
    class="modal-backdrop"
    onclick={handleBackdropClick}
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
  >
    <div class="modal modal-{size}">
      <div class="modal-header">
        <h2 id="modal-title">{title}</h2>
        {#if closable}
          <button class="close-btn" onclick={close} aria-label="Close">×</button>
        {/if}
      </div>
      
      <div class="modal-body">
        {@render children()}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    animation: fadeIn 0.2s ease;
  }
  
  .modal {
    background: white;
    border-radius: 16px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.2);
    max-height: 90vh;
    overflow-y: auto;
    animation: slideUp 0.3s ease;
  }
  
  .modal-small { width: 400px; }
  .modal-medium { width: 600px; }
  .modal-large { width: 800px; }
  
  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px 24px;
    border-bottom: 1px solid #eee;
  }
  
  .modal-header h2 {
    margin: 0;
    font-size: 1.25rem;
  }
  
  .close-btn {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    color: #666;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .close-btn:hover {
    background: #f0f0f0;
  }
  
  .modal-body {
    padding: 24px;
  }
  
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
  
  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
```

Using the Modal with the Card component together:

```svelte
<script>
  import Modal from '$lib/components/Modal.svelte';
  import Card from '$lib/components/Card.svelte';
  import Button from '$lib/components/Button.svelte';
  
  let showModal = $state(false);
  let showConfirm = $state(false);
  let selectedPost = $state(null);
  
  let posts = $state([
    { id: 1, title: 'Getting Started', content: 'Learn the basics of SvelteKit.' },
    { id: 2, title: 'Advanced Patterns', content: 'Deep dive into component patterns.' },
    { id: 3, title: 'Deployment', content: 'Deploy your app to production.' }
  ]);
</script>

<h1>My Blog</h1>

<div class="post-grid">
  {#each posts as post}
    <Card title={post.title} hoverable>
      <p>{post.content}</p>
      
      {#snippet footer()}
        <button onclick={() => { selectedPost = post; showModal = true; }}>
          Read More
        </button>
        <button class="delete" onclick={() => { selectedPost = post; showConfirm = true; }}>
          Delete
        </button>
      {/snippet}
    </Card>
  {/each}
</div>

<!-- Post Detail Modal -->
<Modal bind:open={showModal} title={selectedPost?.title || 'Post'} size="medium">
  {#if selectedPost}
    <p>{selectedPost.content}</p>
    <p>This is the full content of the post. In a real app, you'd fetch more data here.</p>
  {/if}
</Modal>

<!-- Delete Confirmation Modal -->
<Modal bind:open={showConfirm} title="Delete Post" size="small" closable={false}>
  <p>Are you sure you want to delete "{selectedPost?.title}"?</p>
  <p>This action <strong>cannot be undone</strong>.</p>
  
  <div class="confirm-actions">
    <Button text="Cancel" onclick={() => showConfirm = false} />
    <Button
      text="Yes, Delete"
      variant="danger"
      onclick={() => {
        posts = posts.filter(p => p.id !== selectedPost.id);
        showConfirm = false;
        selectedPost = null;
      }}
    />
  </div>
</Modal>
```

> **🧪 Try It Yourself:** Build both the Card and Modal components in your project. Create a page that uses them. Try adding your own variants to the Card (like `flat` or `glass`), and try adding a confirmation dialog to the Modal. Experiment with different props and see what happens! Then try building your own components — maybe a Toast notification, a Tab component, or an Accordion.

### Wrapping Up Chapter 8

You've learned the building blocks of Svelte applications:

- **Components** are reusable pieces (like LEGO bricks). Create once, use everywhere.
- **Props** pass data from parent to child using `$props()`. Data flows one way.
- **Default props** make components work out of the box without requiring every value.
- **Children** (`{@render children()}`) lets components wrap arbitrary content from their parents.
- **Event callbacks** let children communicate back to parents. Pass functions as props.
- **Conditional rendering** (`{#if}`) shows different content based on conditions. Chain with `{:else if}` and `{:else}`.
- **List rendering** (`{#each}`) renders arrays of data. Always provide keys!
- **Snippets** reuse markup within a component. They're like private helper templates.

These patterns are the foundation of every Svelte app. Master them, and you can build anything!

In the next chapter, we'll explore **routing and pages** — how SvelteKit maps files to URLs and how to build multi-page applications. Let's keep going!

---

## Chapter 9: Routing and Pages

### File-Based Routing: The File IS the Route

Remember in Chapter 5 when we introduced file-based routing? Now it's time to really understand it. In SvelteKit, the path of a file determines the URL of a page. There's no router configuration file, no route table, no special syntax. The file structure IS the routing.

Here's the rule: **every folder with a `+page.svelte` file becomes a route.**

```
src/routes/
  +page.svelte              → /
  about/
    +page.svelte             → /about
  blog/
    +page.svelte             → /blog
    my-first-post/
      +page.svelte           → /blog/my-first-post
  contact/
    +page.svelte             → /contact
```

It's like organizing your books on a shelf. Each folder is a section, and each `+page.svelte` is a book. The path from the shelf root to the book determines its address. No need to write down addresses in a separate book — the shelf itself tells you where everything is.

Let's create a simple multi-page app to see this in action.

First, set up your project structure:

```bash
# In your SvelteKit project
cd src/routes

# Create folders
mkdir about blog contact

# Create the files (we'll fill them in next)
touch about/+page.svelte
touch blog/+page.svelte
touch contact/+page.svelte
```

Now let's add content to each page. Start with the home page:

```svelte
<!-- src/routes/+page.svelte -->
<script>
  import Card from '$lib/components/Card.svelte';
</script>

<svelte:head>
  <title>Home - My SvelteKit App</title>
  <meta name="description" content="Welcome to our awesome SvelteKit application!" />
</svelte:head>

<div class="hero">
  <h1>Welcome Home! 🏠</h1>
  <p>This is the home page of my awesome SvelteKit application.</p>
</div>

<div class="links">
  <Card title="📖 About Us" hoverable>
    <p>Learn about our team and mission.</p>
    {#snippet footer()}
      <a href="/about">Go to About →</a>
    {/snippet}
  </Card>
  
  <Card title="✍️ Blog" hoverable>
    <p>Read our latest articles and tutorials.</p>
    {#snippet footer()}
      <a href="/blog">Go to Blog →</a>
    {/snippet}
  </Card>
  
  <Card title="📬 Contact" hoverable>
    <p>Get in touch with us.</p>
    {#snippet footer()}
      <a href="/contact">Go to Contact →</a>
    {/snippet}
  </Card>
</div>

<style>
  .hero {
    text-align: center;
    padding: 3rem 0;
  }
  
  .links {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 1.5rem;
    max-width: 1000px;
    margin: 0 auto;
  }
  
  a {
    color: #3b82f6;
    text-decoration: none;
    font-weight: 600;
  }
  
  a:hover {
    text-decoration: underline;
  }
</style>
```

The About page:

```svelte
<!-- src/routes/about/+page.svelte -->
<svelte:head>
  <title>About - My SvelteKit App</title>
</svelte:head>

<h1>About Us</h1>
<p>We're a team of passionate developers building cool things with SvelteKit.</p>

<h2>Our Mission</h2>
<p>To learn web development while having fun and building real projects.</p>

<h2>Our Team</h2>
<ul>
  <li><strong>Alice</strong> — Frontend Developer 👩‍💻</li>
  <li><strong>Bob</strong> — Designer 👨‍🎨</li>
  <li><strong>Charlie</strong> — Backend Developer 👨‍💻</li>
</ul>

<h2>Tech Stack</h2>
<p>We're using SvelteKit because it's fast, simple, and fun!</p>
```

The Blog page:

```svelte
<!-- src/routes/blog/+page.svelte -->
<script>
  let { data } = $props();
</script>

<svelte:head>
  <title>Blog - My SvelteKit App</title>
</svelte:head>

<h1>Our Blog</h1>

{#each data.posts as post}
  <article class="post">
    <h2>
      <a href="/blog/{post.slug}">{post.title}</a>
    </h2>
    <time>{post.date}</time>
    <p>{post.excerpt}</p>
    <a href="/blog/{post.slug}" class="read-more">Read more →</a>
  </article>
{:else}
  <p>No posts yet!</p>
{/each}
```

The Contact page:

```svelte
<!-- src/routes/contact/+page.svelte -->
<script>
  let name = $state('');
  let email = $state('');
  let message = $state('');
  let submitted = $state(false);
  let submittedData = $state(null);
  
  function handleSubmit() {
    submittedData = { name, email, message };
    submitted = true;
  }
  
  function reset() {
    name = '';
    email = '';
    message = '';
    submitted = false;
    submittedData = null;
  }
</script>

<svelte:head>
  <title>Contact - My SvelteKit App</title>
</svelte:head>

<h1>Contact Us</h1>

{#if submitted}
  <div class="success">
    <p>Thanks, {submittedData.name}! We'll get back to you soon. 🎉</p>
    <p>We'll email you at {submittedData.email}.</p>
    <button onclick={reset}>Send another message</button>
  </div>
{:else}
  <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
    <div class="field">
      <label for="name">Name</label>
      <input id="name" bind:value={name} required placeholder="Your name" />
    </div>
    <div class="field">
      <label for="email">Email</label>
      <input id="email" type="email" bind:value={email} required placeholder="you@example.com" />
    </div>
    <div class="field">
      <label for="message">Message</label>
      <textarea id="message" bind:value={message} rows="5" required placeholder="How can we help?"></textarea>
    </div>
    <button type="submit">Send Message</button>
  </form>
{/if}
```

Finally, let's update the layout to include navigation:

```svelte
<!-- src/routes/+layout.svelte -->
<script>
  import { page } from '$app/stores';
  import '../app.css';
  let { children } = $props();
</script>

<nav class="main-nav">
  <div class="nav-inner">
    <a href="/" class="logo">🚀 MyApp</a>
    <div class="nav-links">
      <a href="/" class:active={$page.url.pathname === '/'}>Home</a>
      <a href="/about" class:active={$page.url.pathname === '/about'}>About</a>
      <a href="/blog" class:active={$page.url.pathname.startsWith('/blog')}>Blog</a>
      <a href="/contact" class:active={$page.url.pathname === '/contact'}>Contact</a>
    </div>
  </div>
</nav>

<main>
  {@render children()}
</main>

<footer>
  <p>Built with SvelteKit ❤️ • © 2026</p>
</footer>

<style>
  .main-nav {
    background: #1e293b;
    padding: 1rem 0;
    position: sticky;
    top: 0;
    z-index: 100;
  }
  
  .nav-inner {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 2rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .logo {
    color: white;
    font-size: 1.5rem;
    font-weight: bold;
    text-decoration: none;
  }
  
  .nav-links {
    display: flex;
    gap: 1.5rem;
  }
  
  .nav-links a {
    color: #94a3b8;
    text-decoration: none;
    font-weight: 500;
    transition: color 0.2s;
  }
  
  .nav-links a:hover,
  .nav-links a.active {
    color: white;
  }
  
  footer {
    background: #f1f5f9;
    text-align: center;
    padding: 2rem;
    margin-top: 4rem;
    color: #64748b;
  }
</style>
```

> **🧪 Try It Yourself:** Run `npm run dev` and navigate between your pages. Notice how fast the transitions feel? That's SvelteKit at work — it preloads pages when you hover over links! Try adding more pages — maybe a `/pricing` page or a `/faq` page. Just create a new folder with a `+page.svelte` and you're done!

### Dynamic Routes: [slug], [...rest]

Static routes are great, but most real apps have pages that change based on data. A blog needs individual post pages, a store needs product pages, a social app needs user profiles. These are called **dynamic routes**.

In SvelteKit, you create dynamic routes by wrapping a folder or file name in square brackets:

```
src/routes/
  blog/
    [slug]/
      +page.svelte       → matches /blog/anything-here
```

The `[slug]` part is a **parameter** — it captures whatever value appears in that position of the URL. It's like a blank space in a mad-libs game:

```
/blog/[slug]
/blog/hello-world     → slug = "hello-world"
/blog/svelte-5        → slug = "svelte-5"
/blog/my-first-post   → slug = "my-first-post"
```

Let's create the dynamic blog post page:

```bash
# Create the dynamic route
mkdir -p "src/routes/blog/[slug]"
```

```svelte
<!-- src/routes/blog/[slug]/+page.svelte -->
<script>
  let { data } = $props();
</script>

<svelte:head>
  <title>{data.title} - My SvelteKit Blog</title>
  <meta name="description" content={data.excerpt} />
</svelte:head>

<article>
  <header>
    <h1>{data.title}</h1>
    <time>{data.date}</time>
  </header>
  
  <div class="content">
    {@html data.content}
  </div>
  
  <nav class="post-nav">
    <a href="/blog">← Back to Blog</a>
  </nav>
</article>
```

Now we need a **load function** to fetch the blog post data. Create a `+page.ts` file alongside the `+page.svelte`:

```typescript
// src/routes/blog/[slug]/+page.ts
import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';

export const load: PageLoad = ({ params }) => {
  // In a real app, you'd fetch from a database or API
  const posts: Record<string, { title: string; date: string; excerpt: string; content: string }> = {
    'hello-sveltekit': {
      title: 'Hello, SvelteKit!',
      date: '2026-01-15',
      excerpt: 'Our first blog post using SvelteKit.',
      content: `
        <p>Welcome to our first blog post! We're excited to start this journey with SvelteKit.</p>
        <p>In this series, we'll learn how to build amazing web applications from scratch.</p>
        <h2>Why SvelteKit?</h2>
        <p>SvelteKit is fast, simple, and fun. It lets us focus on building, not fighting with the framework.</p>
        <h2>What We'll Build</h2>
        <p>We're building a blog — and you're reading it right now! Every page you see is a SvelteKit component.</p>
      `
    },
    'understanding-runes': {
      title: 'Understanding Svelte 5 Runes',
      date: '2026-02-01',
      excerpt: 'A deep dive into the new reactivity system.',
      content: `
        <p>Svelte 5 introduced runes — a powerful new way to handle reactivity.</p>
        <p>Instead of implicit reactivity, runes make it explicit with special symbols.</p>
        <h2>The Main Runes</h2>
        <ul>
          <li><code>$state</code> — reactive state</li>
          <li><code>$derived</code> — computed values</li>
          <li><code>$effect</code> — side effects</li>
          <li><code>$props</code> — receiving data</li>
        </ul>
        <h2>Why Runes?</h2>
        <p>Runes are explicit, TypeScript-friendly, and composable. They make reactivity clearer and more powerful.</p>
      `
    },
    'building-components': {
      title: 'Building Reusable Components',
      date: '2026-02-20',
      excerpt: 'How to create components that work everywhere.',
      content: `
        <p>Components are the building blocks of Svelte applications.</p>
        <p>Good components are reusable, flexible, and easy to understand.</p>
        <h2>Component Design Principles</h2>
        <p>Keep components small and focused. One job, one component.</p>
        <h2>Props</h2>
        <p>Use props to make components configurable. Always provide sensible defaults.</p>
      `
    }
  };
  
  const post = posts[params.slug];
  
  if (!post) {
    throw error(404, {
      message: `Post "${params.slug}" not found`
    });
  }
  
  return {
    title: post.title,
    date: post.date,
    excerpt: post.excerpt,
    content: post.content
  };
};
```

**How it works:**

1. User visits `/blog/hello-sveltekit`
2. SvelteKit matches the `[slug]` route with `slug = "hello-sveltekit"`
3. The `+page.ts` load function runs with `params.slug = "hello-sveltekit"`
4. The function returns the data for that post
5. The `+page.svelte` receives the data and renders the page

If the slug doesn't match any post, we throw a 404 error. SvelteKit has a nice error page that shows when this happens.

> **💡 The `$types` import:** `import type { PageLoad } from './$types'` gives you TypeScript types for the load function. It knows what `params` looks like based on your route structure. If your route is `[slug]`, then `params.slug` is a string. This is one of SvelteKit's superpowers — full type safety for your routes! No more guessing what `params` contains.

### Page Data: +page.ts Load Functions

The `+page.ts` (or `+page.js`) file is where you fetch data for a page. This is one of the most important concepts in SvelteKit.

```typescript
// src/routes/about/+page.ts
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
  // You can do async work here!
  const response = await fetch('https://api.example.com/team');
  const team = await response.json();
  
  return {
    team
  };
};
```

The `load` function runs:
- **On the server** during SSR (for the initial page load)
- **On the client** during navigation (when you click links)

The data it returns becomes available as the `data` prop in your `+page.svelte`:

```svelte
<script>
  let { data } = $props();
</script>

<h1>Our Team</h1>
{#each data.team as member}
  <div class="member">
    <h3>{member.name}</h3>
    <p>{member.role}</p>
  </div>
{/each}
```

**Multiple load functions:** Each layout level can have its own load function. Data cascades down from layouts to pages:

```
src/routes/
  +layout.ts              → Layout-level data (available to ALL pages)
  +page.ts                → Home page data
  blog/
    +layout.ts            → Blog section data (available to all /blog/* pages)
    +page.ts              → Blog index data
    [slug]/
      +page.ts            → Individual post data
```

A page receives data from its own load function AND all parent layouts:

```svelte
<!-- src/routes/blog/[slug]/+page.svelte -->
<script>
  // data includes:
  // - Root layout data (user info, site settings)
  // - Blog layout data (categories)
  // - Page data (this specific post)
  let { data } = $props();
</script>

<h1>{data.title}</h1>
<!-- data.title comes from the page's +page.ts -->
<!-- data.categories comes from the blog layout's +layout.ts -->
<!-- data.user comes from the root layout's +layout.ts -->
```

### Layout Data: +layout.svelte and +layout.ts

Layouts wrap around pages and provide shared data and UI. A `+layout.ts` file fetches data that's shared across all pages in that route section.

```typescript
// src/routes/blog/+layout.ts
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async () => {
  // This data is available to all pages under /blog
  const categories = ['Svelte', 'SvelteKit', 'JavaScript', 'CSS', 'TypeScript'];
  
  // You could also fetch from an API:
  // const response = await fetch('https://api.example.com/categories');
  // const categories = await response.json();
  
  return {
    categories
  };
};
```

The layout component receives both its own data AND its parent's data:

```svelte
<!-- src/routes/blog/+layout.svelte -->
<script>
  let { data, children } = $props();
</script>

<div class="blog-layout">
  <aside class="sidebar">
    <h2>Categories</h2>
    <ul>
      {#each data.categories as category}
        <li>
          <a href="/blog?category={category}">{category}</a>
        </li>
      {/each}
    </ul>
  </aside>
  
  <div class="blog-content">
    {@render children()}
  </div>
</div>

<style>
  .blog-layout {
    display: grid;
    grid-template-columns: 200px 1fr;
    gap: 2rem;
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
  }
  
  .sidebar h2 {
    font-size: 1.1rem;
    margin-bottom: 0.5rem;
  }
  
  .sidebar ul {
    list-style: none;
  }
  
  .sidebar li {
    padding: 0.5rem 0;
  }
  
  .sidebar a {
    color: #64748b;
    text-decoration: none;
  }
  
  .sidebar a:hover {
    color: #3b82f6;
  }
</style>
```

> **⚠️ Watch Out:** Layout load functions only run once for their section. If you navigate from `/blog/post-1` to `/blog/post-2`, the blog layout's load function doesn't re-run. Only the page's load function runs again. This is a performance optimization — if the categories don't change, why fetch them again? If you need the layout data to refresh, you can use `invalidate()`.

### Navigation: goto() from $app/navigation

SvelteKit provides several ways to navigate between pages. Let's explore each one.

**1. HTML links (most common and recommended):**
```svelte
<a href="/about">About Us</a>
<a href="/blog/my-post">Read this post</a>
<a href="https://example.com">External link</a>
```

HTML links are the simplest and most reliable way to navigate. They work without JavaScript, they're accessible to screen readers, and SvelteKit automatically uses client-side navigation for internal links.

**2. Programmatic navigation with `goto()`:**

Sometimes you need to navigate in response to an event — like redirecting after a form submission or navigating after login. That's what `goto()` is for.

```svelte
<script>
  import { goto } from '$app/navigation';
  
  let username = $state('');
  let password = $state('');
  let error = $state('');
  
  async function handleLogin() {
    try {
      const response = await fetch('/api/login', {
        method: 'POST',
        body: JSON.stringify({ username, password })
      });
      
      if (response.ok) {
        // Navigate to dashboard after successful login
        goto('/dashboard');
      } else {
        error = 'Invalid credentials';
      }
    } catch (err) {
      error = 'Connection failed';
    }
  }
  
  function handleSearch(query) {
    // Navigate to search page with query parameter
    goto(`/search?q=${encodeURIComponent(query)}`);
  }
</script>

<form onsubmit={(e) => { e.preventDefault(); handleLogin(); }}>
  <input bind:value={username} placeholder="Username" />
  <input bind:value={password} type="password" placeholder="Password" />
  {#if error}
    <p class="error">{error}</p>
  {/if}
  <button type="submit">Log In</button>
</form>
```

**3. `goto()` options:**

```javascript
// Replace the current history entry (no back button)
goto('/dashboard', { replaceState: true });

// Invalidate all load functions (refetch data)
goto('/dashboard', { invalidateAll: true });

// Prevent scrolling to top
goto('/dashboard', { keepFocus: true });

// Combine options
goto('/dashboard', { replaceState: true, invalidateAll: true });
```

**4. Preloading on hover:**

```svelte
<!-- Preload when user hovers over the link -->
<a href="/about" data-sveltekit-preload-data="hover">About</a>

<!-- Preload when link enters viewport (for mobile) -->
<a href="/about" data-sveltekit-preload-data="viewport">About</a>

<!-- Disable preloading for this link -->
<a href="/about" data-sveltekit-preload-data="off">About</a>
```

This tells SvelteKit to start loading the page data when the user hovers over the link. By the time they click, everything is ready — instant navigation!

> **🔍 Fun Fact:** SvelteKit's `<a>` tag is smart — it knows when to use client-side navigation (for internal links) and when to do a full page load (for external links, downloads, or links with `target="_blank"`). You don't have to think about it — it just works!

### The Page Store: $page.url, $page.params

Sometimes you need to know information about the current page — the URL, the parameters, or any query strings. SvelteKit gives you the `$page` store for this.

```svelte
<script>
  import { page } from '$app/stores';
</script>

<h1>Current Page Info</h1>
<p>URL: {$page.url.pathname}</p>
<p>Full URL: {$page.url.href}</p>

{#if $page.params.slug}
  <p>Post slug: {$page.params.slug}</p>
{/if}

{#if $page.url.searchParams.has('q')}
  <p>Search query: {$page.url.searchParams.get('q')}</p>
{/if}
```

**Common uses:**

**Active navigation link:**
```svelte
<script>
  import { page } from '$app/stores';
</script>

<nav>
  <a href="/" class:active={$page.url.pathname === '/'}>Home</a>
  <a href="/about" class:active={$page.url.pathname === '/about'}>About</a>
  <a href="/blog" class:active={$page.url.pathname.startsWith('/blog')}>Blog</a>
</nav>

<style>
  .active {
    color: #3b82f6;
    font-weight: bold;
    border-bottom: 2px solid #3b82f6;
  }
  
  nav a {
    padding: 0.5rem 0;
    text-decoration: none;
    color: #64748b;
  }
</style>
```

**Breadcrumbs from URL:**
```svelte
<script>
  import { page } from '$app/stores';
  
  let crumbs = $derived(
    $page.url.pathname
      .split('/')
      .filter(Boolean)
      .map((segment, index, arr) => ({
        label: segment.charAt(0).toUpperCase() + segment.slice(1).replace(/-/g, ' '),
        href: '/' + arr.slice(0, index + 1).join('/')
      }))
  );
</script>

<nav class="breadcrumbs">
  <a href="/">Home</a>
  {#each crumbs as crumb, i}
    <span class="separator">/</span>
    {#if i < crumbs.length - 1}
      <a href={crumb.href}>{crumb.label}</a>
    {:else}
      <span class="current">{crumb.label}</span>
    {/if}
  {/each}
</nav>
```

**Debug info (great for development):**
```svelte
<script>
  import { page } from '$app/stores';
</script>

{#if import.meta.env.DEV}
  <div class="debug-panel">
    <p><strong>Route:</strong> {$page.url.pathname}</p>
    <p><strong>Params:</strong> {JSON.stringify($page.params)}</p>
    <p><strong>Status:</strong> {$page.status}</p>
    <p><strong>Query:</strong> {$page.url.search}</p>
  </div>
{/if}
```

> **⚠️ Watch Out:** `$page` is a store (notice the `$` prefix). In Svelte 5, stores with the `$` prefix auto-subscribe — the component re-renders when the store value changes. Don't confuse this with runes like `$state` — the `$` here means "subscribe to this store," not "make this reactive." They're different concepts that happen to use the same symbol!

### The Catch-All Route: [...slug]/+page.ts

Sometimes you need a route that catches **any** URL path, no matter how many segments it has. That's what the **rest parameter** (`[...slug]`) is for.

```
src/routes/
  docs/
    [...slug]/
      +page.svelte          → matches /docs/anything
                                /docs/anything/here
                                /docs/anything/here/even/this
```

This is perfect for documentation sites, wiki pages, or any app with deeply nested content.

```bash
mkdir -p "src/routes/docs/[...slug]"
```

```typescript
// src/routes/docs/[...slug]/+page.ts
import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';

export const load: PageLoad = ({ params }) => {
  // params.slug is a string of the full path after /docs/
  // /docs/getting-started/installation → "getting-started/installation"
  // /docs/intro                        → "intro"
  // /docs                              → "" (empty string)
  
  const path = params.slug ? params.slug.split('/') : ['index'];
  
  // Simulate a docs structure
  const pages: Record<string, { title: string; content: string }> = {
    'getting-started/installation': {
      title: 'Installation',
      content: '<p>Install SvelteKit with: npm create svelte@latest</p>'
    },
    'getting-started/quickstart': {
      title: 'Quick Start',
      content: '<p>Create your first SvelteKit app in 5 minutes!</p>'
    },
    'guides/routing': {
      title: 'Routing Guide',
      content: '<p>Learn about file-based routing in SvelteKit.</p>'
    }
  };
  
  const key = path.join('/');
  const page = pages[key];
  
  if (!page) {
    throw error(404, {
      message: `Documentation page "${key}" not found`
    });
  }
  
  return {
    path,
    title: page.title,
    content: page.content
  };
};
```

```svelte
<!-- src/routes/docs/[...slug]/+page.svelte -->
<script>
  let { data } = $props();
</script>

<svelte:head>
  <title>Docs: {data.title}</title>
</svelte:head>

<nav class="doc-breadcrumb">
  <a href="/docs">Documentation</a>
  {#each data.path as segment, i}
    <span>/</span>
    <a href="/docs/{data.path.slice(0, i + 1).join('/')}">
      {segment}
    </a>
  {/each}
</nav>

<h1>{data.title}</h1>
<p>Documentation for: {data.path.join(' → ')}</p>

<div class="content">
  {@html data.content}
</div>
```

> **💡 When to use rest parameters:** Use `[...slug]` when you don't know how deep the URL nesting will go. Use regular `[slug]` when you know there's exactly one dynamic segment. A documentation site might use `[...slug]` (any depth), while a blog uses `[slug]` (just the post title). A store might use `[...slug]` for categories → subcategories → products.

### SPA Mode: ssr=false, prerender=false

By default, SvelteKit renders pages on the server (SSR). But sometimes you want a pure client-side Single Page Application. Maybe you're building a dashboard that requires login, or an app that only works offline.

You can disable SSR and prerendering in `+page.ts` or `+layout.ts`:

```typescript
// src/routes/dashboard/+layout.ts
export const ssr = false;
export const prerender = false;
```

Or for specific pages:

```typescript
// src/routes/dashboard/settings/+page.ts
export const ssr = false;
export const prerender = false;
```

**What this does:**

- **`ssr = false`** — Pages are rendered entirely on the client. The server sends an empty shell, and JavaScript builds the page. Good for pages that need browser APIs.
- **`prerender = false`** — Pages aren't pre-rendered at build time. They're rendered on-demand. Good for pages with dynamic content.

**When to use SPA mode:**
- Dashboards that require authentication
- Apps that heavily use browser APIs (localStorage, Web Workers, etc.)
- Apps behind a login wall where SEO doesn't matter
- Development/admin interfaces
- Apps that fetch data from the client only

```svelte
<!-- src/routes/dashboard/+page.svelte -->
<script>
  import { onMount } from 'svelte';
  
  let data = $state(null);
  let loading = $state(true);
  let error = $state(null);
  
  onMount(async () => {
    // This only runs on the client!
    try {
      const response = await fetch('/api/dashboard');
      if (!response.ok) throw new Error('Failed to load dashboard');
      data = await response.json();
    } catch (err) {
      error = err.message;
    } finally {
      loading = false;
    }
  });
</script>

{#if loading}
  <div class="loading">
    <div class="spinner"></div>
    <p>Loading dashboard...</p>
  </div>
{:else if error}
  <div class="error">
    <p>Something went wrong: {error}</p>
    <button onclick={() => window.location.reload()}>Try Again</button>
  </div>
{:else}
  <h1>Dashboard</h1>
  <p>Welcome back, {data.user.name}! 👋</p>
  
  <div class="stats">
    <div class="stat-card">
      <span class="stat-value">{data.stats.posts}</span>
      <span class="stat-label">Posts</span>
    </div>
    <div class="stat-card">
      <span class="stat-value">{data.stats.comments}</span>
      <span class="stat-label">Comments</span>
    </div>
    <div class="stat-card">
      <span class="stat-value">{data.stats.likes}</span>
      <span class="stat-label">Likes</span>
    </div>
  </div>
{/if}
```

> **⚠️ Watch Out:** In SPA mode, the initial page load is slower because JavaScript has to download and run before anything appears. Use SSR for public-facing pages where first-paint speed matters. Use SPA mode for authenticated or private pages. You can mix both in the same app — SSR for the public site, SPA for the admin dashboard.

### adapter-static: Building for Static Hosting

SvelteKit can build your app as a completely static site — just HTML, CSS, and JavaScript files. This is perfect for hosting on GitHub Pages, Netlify, Cloudflare Pages, or any static file server.

First, install the static adapter:

```bash
npm install -D @sveltejs/adapter-static
```

Then update your SvelteKit config:

```javascript
// svelte.config.js
import adapter from '@sveltejs/adapter-static';

export default {
  kit: {
    adapter: adapter({
      pages: 'build',
      assets: 'build',
      fallback: '404.html',  // SPA fallback for client-side routing
      precompress: false,
      strict: true
    })
  }
};
```

Now you need to tell SvelteKit which pages to pre-render. Create a root layout with prerendering enabled:

```typescript
// src/routes/+layout.ts
export const prerender = true;
```

Or prerender specific pages:

```typescript
// src/routes/about/+page.ts
export const prerender = true;

// src/routes/blog/[slug]/+page.ts
export const prerender = true;

// Generate all blog posts at build time
export async function entries() {
  // Return all possible slug values
  return [
    { slug: 'hello-sveltekit' },
    { slug: 'understanding-runes' },
    { slug: 'building-components' }
  ];
}
```

The `entries` function tells SvelteKit exactly which dynamic routes to pre-render. Without it, SvelteKit doesn't know which blog posts exist at build time. You can also generate entries from an API:

```typescript
export async function entries() {
  const response = await fetch('https://api.example.com/posts');
  const posts = await response.json();
  return posts.map(post => ({ slug: post.slug }));
}
```

**Build and deploy:**

```bash
# Build the static site
npm run build

# Preview it locally
npm run preview

# The 'build' folder contains your static site
ls build/
# → index.html, about/index.html, blog/index.html, _app/...
```

The `build` folder contains everything you need. Upload it to any static hosting service!

**Deploying to GitHub Pages:**

```yaml
# .github/workflows/deploy.yml
name: Deploy to GitHub Pages
on:
  push:
    branches: [main]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - run: npm ci
      - run: npm run build
      - uses: actions/upload-pages-artifact@v3
        with:
          path: build
  deploy:
    needs: build
    runs-on: ubuntu-latest
    permissions:
      pages: write
      id-token: write
    environment:
      name: github-pages
    steps:
      - uses: actions/deploy-pages@v4
```

> **🔍 Fun Fact:** Static sites are incredibly fast because there's no server to wait for. The HTML files are already built — your browser just downloads and displays them. It's like the difference between ordering food at a restaurant (SSR — wait for the kitchen) vs. picking up pre-made food from a shelf (static — grab and go). Static sites can load in under 100 milliseconds!

### Practice: Create a Multi-Page App with Navigation

Let's put everything together and build a complete multi-page app!

First, set up the project:

```bash
npx sv create my-sveltekit-app
cd my-sveltekit-app
npm install
npm install -D @sveltejs/adapter-static  # For static deployment
```

Here's the complete project structure we'll create:

```
src/
  lib/
    components/
      Nav.svelte
      Footer.svelte
      Card.svelte
  routes/
    +layout.svelte
    +layout.ts
    +page.svelte
    +page.ts
    about/
      +page.svelte
    blog/
      +layout.svelte
      +layout.ts
      +page.svelte
      +page.ts
      [slug]/
        +page.svelte
        +page.ts
    contact/
      +page.svelte
```

**1. Create the navigation component:**

```svelte
<!-- src/lib/components/Nav.svelte -->
<script>
  import { page } from '$app/stores';
  
  let menuOpen = $state(false);
</script>

<nav>
  <div class="nav-container">
    <a href="/" class="logo">🚀 MyApp</a>
    
    <button class="menu-toggle" onclick={() => menuOpen = !menuOpen}>
      {menuOpen ? '✕' : '☰'}
    </button>
    
    <div class="nav-links" class:open={menuOpen}>
      <a href="/" class:active={$page.url.pathname === '/'}>Home</a>
      <a href="/about" class:active={$page.url.pathname === '/about'}>About</a>
      <a href="/blog" class:active={$page.url.pathname.startsWith('/blog')}>Blog</a>
      <a href="/contact" class:active={$page.url.pathname === '/contact'}>Contact</a>
    </div>
  </div>
</nav>

<style>
  nav {
    background: #1e293b;
    padding: 1rem 0;
    position: sticky;
    top: 0;
    z-index: 100;
  }
  
  .nav-container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 2rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .logo {
    color: white;
    font-size: 1.5rem;
    font-weight: bold;
    text-decoration: none;
  }
  
  .menu-toggle {
    display: none;
    background: none;
    border: none;
    color: white;
    font-size: 1.5rem;
    cursor: pointer;
  }
  
  .nav-links {
    display: flex;
    gap: 1.5rem;
  }
  
  .nav-links a {
    color: #94a3b8;
    text-decoration: none;
    font-weight: 500;
    transition: color 0.2s;
  }
  
  .nav-links a:hover,
  .nav-links a.active {
    color: white;
  }
  
  @media (max-width: 768px) {
    .menu-toggle {
      display: block;
    }
    
    .nav-links {
      display: none;
      position: absolute;
      top: 100%;
      left: 0;
      right: 0;
      background: #1e293b;
      flex-direction: column;
      padding: 1rem 2rem;
    }
    
    .nav-links.open {
      display: flex;
    }
  }
</style>
```

**2. Create the Card component:**

```svelte
<!-- src/lib/components/Card.svelte -->
<script>
  let { title, description, href } = $props();
</script>

<div class="card">
  <h3>{title}</h3>
  <p>{description}</p>
  {#if href}
    <a href={href}>Read more →</a>
  {/if}
</div>

<style>
  .card {
    background: white;
    border-radius: 12px;
    padding: 24px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
    transition: transform 0.2s;
  }
  
  .card:hover {
    transform: translateY(-4px);
  }
  
  h3 {
    margin-top: 0;
  }
  
  a {
    color: #3b82f6;
    text-decoration: none;
    font-weight: 600;
  }
  
  a:hover {
    text-decoration: underline;
  }
</style>
```

**3. Create the footer component:**

```svelte
<!-- src/lib/components/Footer.svelte -->
<footer>
  <p>Built with ❤️ using SvelteKit • © 2026</p>
</footer>

<style>
  footer {
    background: #f1f5f9;
    text-align: center;
    padding: 2rem;
    margin-top: 4rem;
    color: #64748b;
  }
</style>
```

**4. Set up the layout and global styles:**

```svelte
<!-- src/routes/+layout.svelte -->
<script>
  import Nav from '$lib/components/Nav.svelte';
  import Footer from '$lib/components/Footer.svelte';
  import '../app.css';
  let { children } = $props();
</script>

<Nav />

<main>
  {@render children()}
</main>

<Footer />

<style>
  main {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
    min-height: calc(100vh - 200px);
  }
</style>
```

```css
/* src/app.css */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  line-height: 1.6;
  color: #1e293b;
  background: #f8fafc;
}

a {
  color: #3b82f6;
}

h1 {
  font-size: 2.5rem;
  margin-bottom: 1rem;
}

h2 {
  font-size: 1.75rem;
  margin-bottom: 0.75rem;
}
```

**5. Create all the pages:**

```svelte
<!-- src/routes/+page.svelte -->
<script>
  import Card from '$lib/components/Card.svelte';
</script>

<svelte:head>
  <title>Home - MyApp</title>
</svelte:head>

<div class="hero">
  <h1>Welcome to MyApp! 🎉</h1>
  <p class="subtitle">A SvelteKit demonstration project</p>
</div>

<div class="grid">
  <Card
    title="📖 About Us"
    description="Learn about our mission and team."
    href="/about"
  />
  <Card
    title="✍️ Blog"
    description="Read our latest articles and tutorials."
    href="/blog"
  />
  <Card
    title="📬 Contact"
    description="Get in touch with us."
    href="/contact"
  />
</div>

<style>
  .hero {
    text-align: center;
    padding: 3rem 0;
  }
  
  .subtitle {
    font-size: 1.25rem;
    color: #64748b;
    margin-bottom: 2rem;
  }
  
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1.5rem;
  }
</style>
```

```svelte
<!-- src/routes/about/+page.svelte -->
<svelte:head>
  <title>About - MyApp</title>
</svelte:head>

<h1>About Us</h1>
<p>We're learning SvelteKit together, one chapter at a time! 📚</p>

<h2>Our Mission</h2>
<p>To make web development accessible, fun, and productive for everyone.</p>

<h2>Tech Stack</h2>
<ul>
  <li><strong>SvelteKit</strong> — Framework</li>
  <li><strong>Svelte 5</strong> — Reactivity with Runes</li>
  <li><strong>Vite</strong> — Blazing-fast build tool</li>
  <li><strong>TypeScript</strong> — Type safety</li>
  <li><strong>adapter-static</strong> — Static site generation</li>
</ul>

<h2>Why This Project?</h2>
<p>We wanted to learn modern web development by building something real. 
   SvelteKit makes it easy to focus on the fun parts — creating and shipping!</p>
```

```svelte
<!-- src/routes/blog/+layout.svelte -->
<script>
  let { children, data } = $props();
</script>

<div class="blog-layout">
  <aside>
    <h3>Categories</h3>
    <ul>
      {#each data.categories as cat}
        <li>{cat}</li>
      {/each}
    </ul>
  </aside>
  
  <div class="blog-main">
    {@render children()}
  </div>
</div>

<style>
  .blog-layout {
    display: grid;
    grid-template-columns: 200px 1fr;
    gap: 2rem;
  }
  
  aside h3 {
    margin-bottom: 0.5rem;
  }
  
  aside ul {
    list-style: none;
  }
  
  aside li {
    padding: 0.5rem 0;
    color: #64748b;
  }
  
  aside a {
    text-decoration: none;
    color: inherit;
  }
  
  aside a:hover {
    color: #3b82f6;
  }
  
  @media (max-width: 768px) {
    .blog-layout {
      grid-template-columns: 1fr;
    }
  }
</style>
```

```typescript
// src/routes/blog/+layout.ts
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async () => {
  return {
    categories: ['Svelte', 'SvelteKit', 'JavaScript', 'CSS', 'TypeScript']
  };
};
```

```svelte
<!-- src/routes/blog/+page.svelte -->
<script>
  let { data } = $props();
</script>

<svelte:head>
  <title>Blog - MyApp</title>
</svelte:head>

<h1>Blog</h1>

{#each data.posts as post}
  <article>
    <h2><a href="/blog/{post.slug}">{post.title}</a></h2>
    <time>{post.date}</time>
    <p>{post.excerpt}</p>
    <a href="/blog/{post.slug}">Read more →</a>
  </article>
{/each}
```

```typescript
// src/routes/blog/+page.ts
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
  return {
    posts: [
      {
        slug: 'getting-started-sveltekit',
        title: 'Getting Started with SvelteKit',
        date: '2026-01-15',
        excerpt: 'Your first steps into the world of SvelteKit.'
      },
      {
        slug: 'mastering-runes',
        title: 'Mastering Svelte 5 Runes',
        date: '2026-02-01',
        excerpt: 'A deep dive into reactive programming with runes.'
      },
      {
        slug: 'component-patterns',
        title: 'Advanced Component Patterns',
        date: '2026-02-20',
        excerpt: 'Building reusable, flexible components.'
      }
    ]
  };
};
```

```svelte
<!-- src/routes/contact/+page.svelte -->
<script>
  let form = $state({ name: '', email: '', message: '' });
  let submitted = $state(false);
  
  function handleSubmit() {
    submitted = true;
  }
</script>

<svelte:head>
  <title>Contact - MyApp</title>
</svelte:head>

<h1>Contact Us</h1>

{#if submitted}
  <div class="success">
    <p>Thanks, {form.name}! We'll get back to you soon. 🎉</p>
    <p>We'll email you at {form.email}.</p>
    <button onclick={() => { submitted = false; form = { name: '', email: '', message: '' }; }}>
      Send another message
    </button>
  </div>
{:else}
  <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
    <div class="field">
      <label for="name">Name</label>
      <input id="name" bind:value={form.name} required placeholder="Your name" />
    </div>
    <div class="field">
      <label for="email">Email</label>
      <input id="email" type="email" bind:value={form.email} required placeholder="you@example.com" />
    </div>
    <div class="field">
      <label for="msg">Message</label>
      <textarea id="msg" bind:value={form.message} rows="5" required placeholder="How can we help?"></textarea>
    </div>
    <button type="submit">Send Message</button>
  </form>
{/if}

<style>
  .field {
    margin-bottom: 1rem;
  }
  
  label {
    display: block;
    margin-bottom: 0.25rem;
    font-weight: 600;
  }
  
  input, textarea {
    width: 100%;
    padding: 0.75rem;
    border: 2px solid #e2e8f0;
    border-radius: 8px;
    font-size: 1rem;
  }
  
  input:focus, textarea:focus {
    outline: none;
    border-color: #3b82f6;
  }
  
  button {
    background: #3b82f6;
    color: white;
    border: none;
    padding: 12px 24px;
    border-radius: 8px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
  }
  
  button:hover {
    background: #2563eb;
  }
  
  .success {
    background: #ecfdf5;
    border: 2px solid #22c55e;
    border-radius: 12px;
    padding: 2rem;
    text-align: center;
  }
  
  .success button {
    margin-top: 1rem;
    background: #22c55e;
  }
</style>
```

Now run your app:

```bash
npm run dev
```

Navigate through all the pages and notice:
- ✅ The navigation highlights the current page
- ✅ Pages load quickly with smooth transitions
- ✅ The layout (nav + footer) stays consistent
- ✅ The responsive mobile menu works
- ✅ The contact form works with reactive state
- ✅ Preloading makes navigation feel instant

> **🧪 Try It Yourself:** Extend this app! Add a search page with `/search?q=...`, a user profile page with `/users/[id]`, or an error page. Try building with `npm run build` and deploying to a static host. Experiment with nested layouts, dynamic routes, and load functions. The best way to learn is to build!

### Wrapping Up Chapter 9

Congratulations! You've learned the core of SvelteKit routing:

- **File-based routing** maps files to URLs automatically. The folder structure IS the routing.
- **Dynamic routes** (`[slug]`) capture URL parameters. Use them for pages that change based on data.
- **Rest parameters** (`[...slug]`) catch any number of path segments. Use them for deeply nested content.
- **Load functions** (`+page.ts`, `+layout.ts`) fetch data for pages. They run on the server for SSR and on the client for navigation.
- **Layouts** wrap pages with shared UI and data. Data cascades down from layouts to pages.
- **`goto()`** enables programmatic navigation. Use it after form submissions or logins.
- **`$page`** gives you access to current URL info. Use it for active navigation links and breadcrumbs.
- **SPA mode** disables server rendering for private pages. Good for dashboards and admin panels.
- **Static adapter** builds your app as static HTML/CSS/JS. Deploy anywhere!

You now have all the fundamentals to build real SvelteKit applications! In the next part of this book, we'll dive deeper into advanced topics like data fetching strategies, form handling with progressive enhancement, API routes, and deploying your apps to production. Keep building, and remember — the best way to learn is to ship real projects!

---

*End of Part 2: SvelteKit Fundamentals*
# Part 3: TypeScript for Safety

*Writing code that catches mistakes before your users do.*

---

## Chapter 10: Why TypeScript Matters

### JavaScript's Superpower Is Also Its Weakness

JavaScript is incredibly flexible. You can write code like this, and it runs just fine:

```javascript
let thing = "hello";
thing = 42;
thing = { name: "Alice", age: 30 };
thing = undefined;
thing = null;
thing = [1, 2, "three"];
```

That variable `thing` starts as a string, becomes a number, then an object, then undefined, then null, then an array. JavaScript doesn't care. It lets you do *anything* with *anything*. And that... is both JavaScript's greatest superpower and its most dangerous weakness.

Think about it this way. Imagine you're writing a recipe, and every ingredient can turn into a completely different ingredient at any moment. "Add two cups of flour — which might be sugar — or maybe water — or actually, it's now a rubber duck." That would be a terrible recipe! Your cake would explode.

But that's essentially what happens in plain JavaScript when things go wrong. You write a function that expects a number, but somewhere in your app, a string sneaks in:

```javascript
function calculateTotal(price, quantity) {
  return price * quantity;
}

// This works great
calculateTotal(10, 3);  // 30

// But what happens here?
calculateTotal("10", 3);  // "10101010101010" — a string repeated 3 times!
```

JavaScript doesn't throw an error. It just silently does something weird. Your user clicks "Add to Cart" and instead of $30, they see `$10101010101010`. That's the kind of bug that can sit in your code for months and ruin someone's day.

And it gets worse. What if you try to access a property that doesn't exist?

```javascript
function getGreeting(user) {
  return "Hello, " + user.name.toUpperCase();
}

// This works
getGreeting({ name: "Alice" });  // "Hello, ALICE"

// But this crashes your app
getGreeting({ firstName: "Alice" });  // TypeError: Cannot read properties of undefined
```

The error message doesn't tell you *which* function had the problem or *why* `user.name` is undefined. You just see a crash in the console. In a small project, you might find it quickly. In a big project with hundreds of files? Good luck.

These kinds of bugs are called **type errors**, and they make up a surprisingly large percentage of all JavaScript bugs. Studies from companies like Google and Microsoft have found that **type errors cause anywhere from 15% to 40% of production bugs**. That's a lot of preventable crashes.

### TypeScript's Solution: Declare What Things Are

TypeScript solves this problem with one simple idea: **you tell the computer what type of thing each variable holds**. Not the value — just the *type*.

Instead of writing:

```javascript
let name = "Alice";
```

You write:

```typescript
let name: string = "Alice";
```

That little `: string` after the variable name is called a **type annotation**. It tells TypeScript, "This variable will always hold a string. Never a number, never an object, never anything else. Always a string."

Now, if you accidentally try to put a number in there:

```typescript
let name: string = "Alice";
name = 42;  // ❌ Error: Type 'number' is not assignable to type 'string'
```

TypeScript catches the mistake immediately. Not at runtime when your user is staring at a broken page — but right now, while you're still writing the code. In your editor. With a clear, red squiggly line and a helpful error message.

This is the magic of TypeScript. It's like having a really smart proofreader who reads your code and catches mistakes *before* you run it.

> **Think of it like this:** JavaScript is like writing in pencil — you can erase and change anything, which is great for creativity but easy to make mistakes. TypeScript is like writing in pen with a friend looking over your shoulder, saying "Hey, you said this was a string but you just wrote a number."

### The TypeScript Compiler: Your New Best Friend

So how does TypeScript actually catch these errors? Through something called the **TypeScript compiler** (often called "tsc" for short).

Here's the cool part: **TypeScript isn't actually a new language.** It's JavaScript with a type system added on top. When you're done writing your TypeScript code, the compiler strips away all the type information and turns it back into regular JavaScript that browsers can run.

```
TypeScript Code  →  TypeScript Compiler (tsc)  →  Regular JavaScript
```

It's like writing notes on your manuscript in the margins — type annotations, interfaces, all that good stuff — and then the compiler creates a clean copy without the notes, ready for the world to see.

You don't run TypeScript in the browser directly. You write TypeScript, compile it to JavaScript, and the browser runs the JavaScript. But during that compilation step, the compiler checks *everything* for type errors. If there are any, it stops and tells you exactly what's wrong and where.

```bash
# Install TypeScript
npm install -g typescript

# Compile a file
tsc myfile.ts

# Or just check for errors without producing output
tsc --noEmit myfile.ts
```

> **Try It Yourself:**
>
> 1. Install TypeScript: `npm install -g typescript`
> 2. Create a file called `test.ts`
> 3. Write this code:
>    ```typescript
>    let age: number = "twenty-five";
>    ```
> 4. Run `tsc test.ts`
> 5. See the error message! TypeScript tells you exactly what's wrong.

When TypeScript catches an error at compile time, it means you **never deploy that bug to production**. It's like having a safety net before the tightrope, not after.

### How TypeScript Works with SvelteKit

Here's the great news: **SvelteKit already uses TypeScript.** In fact, when you create a new SvelteKit project, it asks you whether you want TypeScript. Almost everyone says yes, and for good reason.

If you've been following along from the earlier parts of this book, you've been writing TypeScript without even thinking about it. SvelteKit defaults to TypeScript, and every `.svelte` file is already set up for it. You just add `lang="ts"` to your script tag and you're off to the races.

```svelte
<!-- This is already TypeScript-ready! -->
<script lang="ts">
  let greeting: string = "Hello, world!";
</script>

<h1>{greeting}</h1>
```

That's it. No configuration, no setup wizard, no special build steps. TypeScript just works.

SvelteKit and TypeScript are a match made in heaven. Here's why:

1. **SvelteKit generates type definitions for you.** When you create routes, pages, and layouts, SvelteKit automatically generates TypeScript types that describe the data flowing through your app. You don't have to figure out the types yourself — SvelteKit hands them to you on a silver platter.

2. **Your editor becomes incredibly helpful.** With TypeScript, your code editor (like VS Code) can show you exactly what props a component expects, what data a page loader returns, and what parameters a function takes — all with autocomplete and hover documentation.

3. **Errors happen early.** Instead of discovering that your page template references a data field that doesn't exist when a user visits that page, TypeScript tells you immediately. Right there in your editor. Red squiggly line. Clear message. Problem solved before it becomes a problem.

4. **Refactoring is safe.** Want to rename a data field? Change the structure of an API response? TypeScript will tell you every single place in your code that needs to be updated. No more hunting through dozens of files to find every reference.

When you create a new SvelteKit project with TypeScript support, you get a `tsconfig.json` file in your project root. This file tells TypeScript how to behave — what features to enable, how strict to be, and which files to check.

```json
{
  "extends": "./.svelte-kit/tsconfig.json",
  "compilerOptions": {
    "allowJs": true,
    "checkJs": true,
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "skipLibCheck": true,
    "sourceMap": true,
    "strict": true,
    "moduleResolution": "bundler"
  }
}
```

That `"strict": true` line is important. It enables the strictest type checking settings, which means TypeScript will catch the most possible errors. It might feel a bit intense at first, but it's like learning to play an instrument with a metronome — strict at first, but it builds good habits.

### Type Annotations: Telling TypeScript What Things Are

A **type annotation** is a little piece of code you add to tell TypeScript the type of a variable, function parameter, or return value. It uses a colon followed by the type name.

Here are the basics:

```typescript
// Strings
let title: string = "My Fanfiction";
let author: string = "Alice";

// Numbers
let wordCount: number = 45000;
let rating: number = 4.8;

// Booleans
let isComplete: boolean = true;
let isExplicit: boolean = false;

// Arrays of strings
let tags: string[] = ["romance", "hurt/comfort", "slow burn"];
let chapters: number[] = [1, 2, 3, 4, 5];

// Objects with specific shapes
let story: { title: string; author: string; wordCount: number } = {
  title: "My Fanfiction",
  author: "Alice",
  wordCount: 45000
};
```

Notice how each variable gets a label after the colon: `string`, `number`, `boolean`. These are TypeScript's **primitive types** — the basic building blocks.

You can also annotate function parameters and return values:

```typescript
// This function takes two numbers and returns a number
function calculateTotal(price: number, quantity: number): number {
  return price * quantity;
}

// This function takes a string and returns a string
function shout(text: string): string {
  return text.toUpperCase() + "!!!";
}

// This function takes a boolean and returns nothing (void)
function logStatus(isActive: boolean): void {
  console.log("Active:", isActive);
}
```

The `: number` after the parameters means "this function returns a number." The `: void` means "this function doesn't return anything useful."

### Type Inference: When TypeScript Figures It Out for You

Here's something wonderful: **you don't always need type annotations.** TypeScript is smart enough to figure out the type on its own in many cases.

```typescript
// TypeScript knows this is a string because you assigned a string
let name = "Alice";  // TypeScript infers: string

// TypeScript knows this is a number
let count = 42;  // TypeScript infers: number

// TypeScript knows this is a boolean
let isActive = true;  // TypeScript infers: boolean

// TypeScript knows this is an array of strings
let tags = ["romance", "angst"];  // TypeScript infers: string[]
```

This is called **type inference**, and it's one of TypeScript's best features. You write clean, natural-looking code, and TypeScript silently figures out the types behind the scenes. It's like having a mind reader on your development team.

This means you can often write TypeScript code that looks almost identical to JavaScript:

```typescript
// This is valid TypeScript! No type annotations needed.
function greet(name) {
  return `Hello, ${name}!`;
}

// TypeScript infers:
// - name parameter is a string (because it's used as a string)
// - return type is a string
```

> **Try It Yourself:**
>
> Create a TypeScript file with this code:
> ```typescript
> let x = 10;
> let y = "hello";
> let z = true;
> let arr = [1, 2, 3];
>
> // Now try to assign wrong types:
> x = "oops";
> y = 123;
> z = "nope";
> arr = ["a", "b"];
> ```
> Run `tsc --noEmit test.ts` and see how TypeScript catches every single error — even though you never wrote a single type annotation!

Here's another way to think about inference: TypeScript looks at what you *do* with a variable to figure out its type. If you assign a string, it's a string. If you use string methods on it (like `.toUpperCase()`), it confirms it's a string. If you try to use a number method on a string variable, TypeScript catches it:

```typescript
let name = "Alice";  // TypeScript infers: string

name.toUpperCase();  // ✅ string methods work
name.toFixed(2);     // ❌ Error: 'toFixed' does not exist on type 'string'
// TypeScript knows this is a string, and strings don't have toFixed!
```

This is incredibly helpful because it means your editor can offer accurate autocomplete. When you type `name.`, your editor knows to show string methods like `toUpperCase()`, `trim()`, `slice()` — not number methods or array methods.

When should you add explicit annotations? A few guidelines:

- **Function parameters** — always annotate these, because TypeScript can't always infer them
- **Function return types** — optional, but helpful for complex functions
- **Variables that could be ambiguous** — if the initial value doesn't make the type clear
- **Complex objects** — when you want to document the expected shape

### The `any` Type: TypeScript's Escape Hatch (and Why to Avoid It)

TypeScript has a special type called `any`. It means "this can be literally anything":

```typescript
let mystery: any = "hello";
mystery = 42;
mystery = { whatever: true };
mystery = [1, 2, 3];
mystery = undefined;
```

When you use `any`, TypeScript essentially turns off type checking for that variable. It's like telling TypeScript, "I don't know what this is, just leave me alone." And TypeScript says, "Okay, no checking for you."

The `any` type is what we call an **escape hatch**. It's there for when you genuinely don't know the type, or when you're working with code that doesn't have types yet. And sometimes, you genuinely need it.

But here's the thing: **`any` defeats the entire purpose of TypeScript.** If you use `any` everywhere, you've basically turned TypeScript back into JavaScript with extra steps. You lose all the safety benefits — the error catching, the autocomplete, the documentation.

```typescript
// 😬 Don't do this
function processData(data: any): any {
  return data.something.that.might.not.exist;
}

// ✅ Do this instead
interface DataShape {
  something: {
    that: {
      exists: boolean;
    };
  };
}

function processData(data: DataShape): boolean {
  return data.something.that.exists;
}
```

Think of `any` like a "jail free card" in a board game. Sometimes you need it, but if you use it every turn, you're not really playing the game.

**Rules of thumb for `any`:**

1. **Avoid it when you can.** Use a more specific type instead.
2. **If you use it, add a comment explaining why.** Future you (or your teammates) will thank you.
3. **Never use it in public APIs or shared code.** Other people depend on those types.
4. **If you're migrating JavaScript to TypeScript**, `any` can help you make incremental progress — convert files one at a time, starting with the most critical ones.

> **Watch Out:** Some TypeScript configurations include a `noImplicitAny` setting (which is enabled in strict mode). This means TypeScript will *warn* you if you accidentally use `any`. It's like TypeScript saying, "Hey, are you sure about that?" If you see a warning about implicit `any`, take it as a sign to figure out the real type.

### Why TypeScript Is Worth the Effort

I know what you might be thinking: "This sounds like a lot of extra work. I just want to write code and make it work."

And you're right — TypeScript does add a few extra characters here and there. But the payoff is enormous:

1. **Fewer bugs in production.** TypeScript catches entire categories of errors before they ever reach your users. No more "undefined is not a function" errors at 3 AM.

2. **Better developer experience.** Autocomplete, hover documentation, and inline type hints make you feel like you have superpowers. When you type `story.`, your editor shows you every property with its type and description. It's like having an API reference built right into your editor.

3. **Easier refactoring.** Changing code becomes safe because TypeScript tells you exactly what breaks. Want to rename a field in your database? TypeScript will show you every single place in your codebase that references that field.

4. **Self-documenting code.** Types act as documentation. When you see a function signature like `function exportStory(story: Story, format: ExportFormat): Promise<Blob>`, you know exactly what it does without reading the implementation. No more guessing what a function expects or returns.

5. **Team collaboration.** Types are like a contract between parts of your codebase. When someone changes a type, everyone else sees the impact immediately. No more "I didn't know you changed that!" moments.

6. **Onboarding new developers.** When a new team member joins, they can understand the codebase faster because types describe the data flow. They don't need to read every function to understand what data goes where.

7. **Safer refactoring of third-party code.** When you update a library, TypeScript tells you if any of the types changed. No more surprise breakages after an `npm install`.

In the chapters ahead, we'll dive deep into TypeScript's type system and learn how to use it effectively in SvelteKit. By the end, you'll wonder how you ever wrote code without it.

> **Key Takeaways:**
> - JavaScript allows any type of value in any variable, which leads to subtle bugs
> - TypeScript adds a type system on top of JavaScript, catching errors at compile time
> - The TypeScript compiler (`tsc`) checks your code and strips types before running
> - SvelteKit works beautifully with TypeScript, generating types for routes and data
> - **Type annotations** (`: string`, `: number`) tell TypeScript what things are
> - **Type inference** lets TypeScript figure out types automatically
> - The `any` type disables type checking — use it sparingly!

---

## Chapter 11: Types, Interfaces, and Generics

### The Building Blocks of TypeScript's Type System

Now that you understand *why* TypeScript matters, let's learn *how* to use its type system. Think of this chapter as learning the alphabet before writing sentences. We'll start with the simplest types and build up to powerful patterns that make your code safer and more expressive.

Don't worry if some of this feels like a lot at first. TypeScript's type system is deep, but you don't need to master everything at once. Start with the basics, use them in your projects, and the advanced features will start making sense naturally.

### Primitive Types: The Simple Stuff

TypeScript has six **primitive types** — the simplest types that represent individual values:

```typescript
// String: text data
let title: string = "The Art of Falling";
let author: string = 'Eleanor Vance';
let summary: string = `A story about ${author}`;  // Template literals work too!

// Number: integers and decimals
let wordCount: number = 45200;
let rating: number = 4.8;
let chapterNumber: number = 12;
let negativeScore: number = -3;  // Yep, negatives work

// Boolean: true or false
let isComplete: boolean = true;
let hasExplicitContent: boolean = false;

// Null: intentionally empty
let middleName: string | null = null;

// Undefined: not yet assigned
let sequelTitle: string | undefined = undefined;

// BigInt: really, really big numbers (rare, but good to know)
let population: bigint = 9007199254740991n;

// Symbol: unique identifiers (even rarer)
let id: symbol = Symbol("id");
```

A few things to notice:

- **Strings** use single quotes, double quotes, or backticks (template literals). They all work the same way.
- **Numbers** include integers, decimals, negatives, and even special values like `Infinity` and `NaN`.
- **Booleans** are just `true` or `false`. No truthy/falsy tricks here — TypeScript is strict about this.
- **`null`** means "this is intentionally empty." You set it to null on purpose.
- **`undefined`** means "this hasn't been assigned a value yet." It's the default.

> **Try It Yourself:**
>
> Create a file `primitives.ts` and try to assign wrong types:
> ```typescript
> let count: number = "five";
> let name: string = 42;
> let active: boolean = "yes";
> ```
> Run `tsc --noEmit primitives.ts` and read the error messages. Notice how TypeScript tells you *exactly* what's expected and what was found.

### Arrays: Lists of Things

Arrays are one of the most common data structures, and TypeScript lets you specify what type of items they hold:

```typescript
// Method 1: Type[]
let tags: string[] = ["romance", "angst", "hurt/comfort"];
let scores: number[] = [4.5, 4.8, 4.2, 5.0];
let flags: boolean[] = [true, false, true, true];

// Method 2: Array<Type>
let titles: Array<string> = ["Chapter 1", "Chapter 2", "Chapter 3"];
let pages: Array<number> = [10, 20, 15, 25, 18];

// Both methods are equivalent — use whichever you prefer!
// Most people prefer the first (string[]) because it's shorter.
```

Now here's a cool feature: **TypeScript can infer the type of array elements**:

```typescript
// TypeScript infers: string[]
let tags = ["romance", "angst"];

// TypeScript infers: (string | number)[]
let mixed = ["hello", 42, "world"];

// TypeScript infers: (string | number | boolean)[]
let reallyMixed = ["hello", 42, true];
```

When you mix types in an array, TypeScript creates a **union type** automatically. The array `mixed` can hold strings OR numbers, but nothing else.

What if you want to push a wrong type into an array?

```typescript
let numbers: number[] = [1, 2, 3];
numbers.push("four");  // ❌ Error: Argument of type 'string' is not assignable to parameter of type 'number'

numbers.push(4);  // ✅ This works
```

TypeScript catches it. Your array stays pure.

### Objects: Structured Data

Objects are how we represent structured data in JavaScript, and TypeScript lets you describe their shape precisely:

```typescript
// Inline object type
let story: { title: string; author: string; wordCount: number } = {
  title: "The Art of Falling",
  author: "Eleanor Vance",
  wordCount: 45200
};

// Accessing properties works as expected
console.log(story.title);    // "The Access of Falling"
console.log(story.author);   // "Eleanor Vance"
console.log(story.wordCount); // 45200

// But trying to access a non-existent property fails
console.log(story.rating);   // ❌ Error: Property 'rating' does not exist on type...

// And trying to set a wrong type fails
story.wordCount = "lots";    // ❌ Error: Type 'string' is not assignable to type 'number'
```

Inline object types work great for small objects, but they get verbose quickly. That's where **interfaces** come in.

### Interfaces: Defining Object Shapes

An **interface** is a named definition of an object's shape. Instead of writing out the full type every time, you give it a name and reuse it:

```typescript
// Define an interface
interface Story {
  title: string;
  author: string;
  wordCount: number;
  isComplete: boolean;
}

// Use it as a type
let myStory: Story = {
  title: "The Art of Falling",
  author: "Eleanor Vance",
  wordCount: 45200,
  isComplete: true
};

// Another story can use the same interface
let anotherStory: Story = {
  title: "Whispers in the Dark",
  author: "James Wright",
  wordCount: 32000,
  isComplete: false
};
```

Interfaces are incredibly powerful because they:

1. **Document your data.** When you see `Story`, you immediately know what properties it has.
2. **Prevent mistakes.** Try to forget a property or add a wrong type — TypeScript catches it.
3. **Enable autocomplete.** Your editor knows exactly what properties are available.
4. **Are reusable.** Define once, use everywhere.

Here's what happens when you break the rules:

```typescript
let brokenStory: Story = {
  title: "My Story"
  // ❌ Error: Missing properties: 'author', 'wordCount', 'isComplete'
};

let wrongTypeStory: Story = {
  title: 12345,           // ❌ Error: 'number' is not assignable to 'string'
  author: "Alice",
  wordCount: "lots",       // ❌ Error: 'string' is not assignable to 'number'
  isComplete: "yes"        // ❌ Error: 'string' is not assignable to 'boolean'
};
```

TypeScript has your back. Every property must be the right type, and every required property must be present.

> **Watch Out:** Interfaces only check the *shape* of objects, not their *identity*. This means a plain object with the right properties satisfies any interface — it doesn't need to have been "created from" that interface. This is by design and is actually a useful feature of TypeScript's structural type system.

### Optional Properties: Not Everything Is Required

Sometimes, not every property needs to be present. Maybe a story doesn't have a sequel yet, or maybe the author's bio is optional. You can mark properties as **optional** with a question mark:

```typescript
interface Story {
  title: string;        // Required
  author: string;       // Required
  wordCount: number;    // Required
  sequel: string;       // ❌ This forces every story to have a sequel
  rating?: number;      // ✅ This makes rating optional
  summary?: string;     // ✅ Optional too
}

let story: Story = {
  title: "The Art of Falling",
  author: "Eleanor Vance",
  wordCount: 45200
  // No rating or summary needed!
};

let ratedStory: Story = {
  title: "Whispers in the Dark",
  author: "James Wright",
  wordCount: 32000,
  rating: 4.8,
  summary: "A haunting tale of loss and redemption."
};
```

When you access an optional property, TypeScript knows it might not exist:

```typescript
function getRating(story: Story): string {
  // story.rating is type number | undefined
  if (story.rating !== undefined) {
    return `Rating: ${story.rating}/5`;
  }
  return "Not yet rated";
}
```

> **Try It Yourself:**
>
> Create an interface for a fan character:
> ```typescript
> interface FanCharacter {
>   name: string;
>   age: number;
>   species: string;
>   backstory?: string;
>   specialAbility?: string;
> }
> ```
> Create two characters — one minimal, one with all fields. Try creating one with wrong types. See what happens!

### Union Types: This OR That

Sometimes a variable can be one of several types. **Union types** let you express this:

```typescript
// This can be a string OR a number
let id: string | number;
id = "abc-123";   // ✅ string is fine
id = 42;          // ✅ number is fine
id = true;        // ❌ boolean is not allowed

// This can be a string OR null
let nickname: string | null = null;
nickname = "Alice";  // ✅
nickname = null;     // ✅

// This can be a number OR an array of numbers
let score: number | number[] = 10;
score = [10, 20, 30];  // ✅
```

Union types are super common in real-world code. Think about API responses — sometimes a field has a value, sometimes it's null:

```typescript
interface User {
  name: string;
  email: string;
  bio: string | null;  // User might not have set a bio
}
```

Or when a function accepts different types of input:

```typescript
function formatId(id: string | number): string {
  return `ID-${id}`;
}

formatId("abc-123");  // "ID-abc-123"
formatId(42);          // "ID-42"
```

### Literal Types: Specific Values

What if you don't just want to restrict a type to `string`, but to *specific strings*? That's where **literal types** come in:

```typescript
// This can be ANY string
let status: string;

// This can ONLY be "active" or "inactive"
let accountStatus: "active" | "inactive";
accountStatus = "active";     // ✅
accountStatus = "inactive";   // ✅
accountStatus = "deleted";    // ❌ Error: not assignable

// This can ONLY be true or false (yes, boolean literals exist!)
let isReady: true | false;  // Same as boolean, but more explicit

// Numbers can be literal too
let statusCode: 200 | 404 | 500;
statusCode = 200;  // ✅
statusCode = 404;  // ✅
statusCode = 500;  // ✅
statusCode = 301;  // ❌ Error: not assignable
```

Literal types are incredibly useful when you have a fixed set of valid values. They catch typos and invalid states at compile time.

### Type Aliases: Naming Your Types

You've seen `interface` for naming object shapes. **Type aliases** (using the `type` keyword) are more general — they can name any type:

```typescript
// Type alias for a union
type Status = "active" | "inactive" | "pending";

// Type alias for a primitive
type UserID = string;

// Type alias for a complex type
type StoryID = string | number;

// Type alias for an object shape (works like interface!)
type StorySummary = {
  title: string;
  author: string;
  wordCount: number;
};

// Using them
let currentStatus: Status = "active";
let userId: UserID = "user-123";
let storyId: StoryID = 42;
```

**When to use `interface` vs `type`?** Here's a simple rule:

- Use **`interface`** for object shapes (especially ones that might be extended or implemented by classes).
- Use **`type`** for unions, intersections, and anything that's not just an object shape.

In practice, both work for most cases, so don't stress too much about which to choose. Pick one and be consistent in your project.

> **Try It Yourself:**
>
> Create a type alias for different fanfiction genres:
> ```typescript
> type Genre =
>   | "romance"
>   | "hurt/comfort"
>   | "angst"
>   | "fluff"
>   | "adventure"
>   | "mystery"
>   | "humor"
>   | "drama";
> ```
> Now create a function that accepts a Genre and returns a description:
> ```typescript
> function describeGenre(genre: Genre): string {
>   // Your code here
> }
> ```

### Generics: Types That Take Parameters

Here's where things get *really* cool. **Generics** are like functions, but for types. They let you write reusable code that works with *any* type while still being type-safe.

Think about it this way. Without generics, you'd have to write separate functions for each type:

```typescript
function getFirstString(items: string[]): string | undefined {
  return items[0];
}

function getFirstNumber(items: number[]): number | undefined {
  return items[0];
}

function getFirstBoolean(items: boolean[]): boolean | undefined {
  return items[0];
}
```

These all do the exact same thing — return the first element of an array. The only difference is the type. That's a lot of code duplication!

Generics solve this by letting you write ONE function that works for ALL types:

```typescript
// The <T> is a type parameter — a placeholder for any type
function getFirst<T>(items: T[]): T | undefined {
  return items[0];
}

// TypeScript infers the type from what you pass in
getFirst(["a", "b", "c"]);  // T is string, returns string | undefined
getFirst([1, 2, 3]);          // T is number, returns number | undefined
getFirst([true, false]);      // T is boolean, returns boolean | undefined
```

You've already been using generics without knowing it! Remember `string[]` and `Array<string>`? That `Array<string>` is a generic type. `Array` is a generic type that takes a type parameter (`string` in this case).

Let's build a generic from scratch to understand how it works:

```typescript
// Without generics: this only works for strings
function firstOfString(items: string[]): string | undefined {
  return items[0];
}

// Without generics: this only works for numbers
function firstOfNumber(items: number[]): number | undefined {
  return items[0];
}

// With generics: this works for ANY array type!
function first<T>(items: T[]): T | undefined {
  return items[0];
}

// TypeScript infers the type from what you pass in
first(["a", "b", "c"]);  // T is string, returns string | undefined
first([1, 2, 3]);          // T is number, returns number | undefined
first([true, false]);      // T is boolean, returns boolean | undefined
```

The `<T>` in `function first<T>` is the **type parameter**. You can name it anything (`T`, `U`, `Item`, `Whatever`), but `T` is the convention. When you call the function, TypeScript replaces `T` with the actual type.

You can even have multiple type parameters:

```typescript
function pair<A, B>(first: A, second: B): [A, B] {
  return [first, second];
}

pair("hello", 42);       // [string, number]
pair(true, "world");     // [boolean, string]
pair(1, 2);              // [number, number]
```

The return type `[A, B]` is a **tuple** — an array with exactly two elements of specific types.

### Built-in Generic Types

TypeScript has several built-in generic types that you'll use all the time:

```typescript
// Promise<T>: A promise that resolves to type T
async function fetchStory(id: string): Promise<Story> {
  const response = await fetch(`/api/stories/${id}`);
  return response.json();
}

// Record<K, V>: An object with keys of type K and values of type V
let ratings: Record<string, number> = {
  "story-1": 4.5,
  "story-2": 4.8,
  "story-3": 4.2
};

// Map<K, V>: A typed Map
let storyMap = new Map<string, Story>();

// Set<T>: A typed Set
let uniqueTags = new Set<string>(["romance", "angst", "fluff"]);

// ReadonlyArray<T>: An array that can't be modified
let frozenTags: ReadonlyArray<string> = ["romance", "angst"];
frozenTags.push("fluff");  // ❌ Error: Property 'push' does not exist
```

> **Watch Out:** `Promise<T>` is one of the most common generics you'll use in SvelteKit. Every `load` function returns a Promise, and every `fetch` call returns a Promise. Always type the `T` so TypeScript knows what the promise resolves to!

### Utility Types: Transforming Types

TypeScript comes with built-in **utility types** that let you transform existing types. These are incredibly useful for real-world code. Think of them as power tools for your type system — they let you reshape types without defining everything from scratch.

```typescript
interface Story {
  title: string;
  author: string;
  wordCount: number;
  summary: string;
  isComplete: boolean;
}

// Partial<T>: All properties become optional
type PartialStory = Partial<Story>;
// Equivalent to:
// { title?: string; author?: string; wordCount?: number; summary?: string; isComplete?: boolean }

let draftStory: PartialStory = {
  title: "My New Story"
  // Everything else is optional — no errors!
};

// Required<T>: All properties become required
type RequiredStory = Required<PartialStory>;
// Back to all required — useful for ensuring completeness

// Pick<T, K>: Only certain properties
type StoryBasic = Pick<Story, "title" | "author">;
// Equivalent to: { title: string; author: string }

let basicInfo: StoryBasic = {
  title: "My Story",
  author: "Alice"
};

// Omit<T, K>: All properties EXCEPT certain ones
type StoryWithoutWordCount = Omit<Story, "wordCount">;
// Equivalent to: { title: string; author: string; summary: string; isComplete: boolean }

// Readonly<T>: All properties become readonly
type FrozenStory = Readonly<Story>;
let immutableStory: FrozenStory = {
  title: "Frozen",
  author: "Alice",
  wordCount: 100,
  summary: "Can't change me!",
  isComplete: true
};
immutableStory.title = "New Title";  // ❌ Error: Cannot assign to 'title'

// Record<K, V>: A convenient way to make objects
type StoryMap = Record<string, Story>;
// Equivalent to: { [key: string]: Story }

// Exclude<T, U>: Remove types from a union
type ActiveStatus = Exclude<Status, "abandoned">;
// "in-progress" | "complete" | "on-hiatus" (no "abandoned")

// Extract<T, U>: Keep only types from a union
type CompletedStatus = Extract<Status, "complete" | "abandoned">;
// "complete" | "abandoned"

// NonNullable<T>: Remove null and undefined
type RequiredString = NonNullable<string | null | undefined>;
// string

// ReturnType<T>: Get the return type of a function
function getStory() {
  return { title: "My Story", wordCount: 1000 };
}
type StoryReturn = ReturnType<typeof getStory>;
// { title: string; wordCount: number }

// Parameters<T>: Get the parameter types of a function
function createStory(title: string, wordCount: number) { /* ... */ }
type CreateParams = Parameters<typeof createStory>;
// [string, number]
```

These utility types are like power tools. Once you start using them, you'll wonder how you ever lived without them. They let you create exactly the type you need from existing types, rather than defining everything from scratch. The most common ones you'll use are `Partial<T>` (for optional updates), `Pick<T, K>` (for selecting specific fields), and `Omit<T, K>` (for removing fields).

### Practice: Type a Fanfiction Metadata Object

Let's put it all together! We'll create a comprehensive type system for fanfiction metadata. This exercise combines everything you've learned: interfaces, optional properties, union types, generics, and utility types.

Here's a real-world example of how these types work together:

```typescript
// Define the possible ratings
type Rating = "G" | "T" | "M" | "MA";

// Define the possible statuses
type Status = "in-progress" | "complete" | "abandoned" | "on-hiatus";

// Define the possible languages
type Language = "English" | "Spanish" | "French" | "German" | "Japanese" | "Chinese";

// Define the genre type
type Genre =
  | "romance"
  | "hurt/comfort"
  | "angst"
  | "fluff"
  | "adventure"
  | "mystery"
  | "humor"
  | "drama"
  | "sci-fi"
  | "fantasy";

// Define the character interface
interface Character {
  name: string;
  fandom: string;
  role: "protagonist" | "antagonist" | "supporting" | "minor";
}

// Define the main story metadata interface
interface StoryMetadata {
  id: string;
  title: string;
  author: string;
  summary: string;
  rating: Rating;
  status: Status;
  language: Language;
  genres: Genre[];
  characters: Character[];
  wordCount: number;
  chapterCount: number;
  publishDate: string;
  updateDate: string | null;
  kudos: number;
  comments: number;
  bookmarks: number;
  isExplicit: boolean;
  warnings?: string[];           // Optional: not all stories have warnings
  series?: {                     // Optional: not all stories are in a series
    name: string;
    position: number;
  };
  relatedWorks?: string[];       // Optional: related work IDs
}
```

Now let's create helper functions that work with this type:

```typescript
// Create a helper function using our types
function formatStoryInfo(story: StoryMetadata): string {
  const genreList = story.genres.join(", ");
  const updateInfo = story.updateDate
    ? `Last updated: ${story.updateDate}`
    : "Never updated";

  return `
    📖 ${story.title}
    ✍️  by ${story.author}
    ⭐ Rating: ${story.rating} | ${story.status}
    📊 ${story.wordCount.toLocaleString()} words | ${story.chapterCount} chapters
    🏷️  ${genreList}
    💜 ${story.kudos} kudos | 💬 ${story.comments} comments | 🔖 ${story.bookmarks} bookmarks
    📅 Published: ${story.publishDate}
    🔄 ${updateInfo}
  `;
}

// Create a function that creates a draft (all optional!)
function createDraft(overrides: Partial<StoryMetadata> = {}): StoryMetadata {
  return {
    id: crypto.randomUUID(),
    title: "Untitled",
    author: "Anonymous",
    summary: "",
    rating: "G",
    status: "in-progress",
    language: "English",
    genres: [],
    characters: [],
    wordCount: 0,
    chapterCount: 0,
    publishDate: new Date().toISOString(),
    updateDate: null,
    kudos: 0,
    comments: 0,
    bookmarks: 0,
    isExplicit: false,
    ...overrides
  };
}

// Create a function to filter stories by genre
function filterByGenre(stories: StoryMetadata[], genre: Genre): StoryMetadata[] {
  return stories.filter(story => story.genres.includes(genre));
}

// Create a function to get story statistics
function getStats(stories: StoryMetadata[]): {
  totalWordCount: number;
  averageRating: number;
  completedCount: number;
  inProgressCount: number;
} {
  const totalWordCount = stories.reduce((sum, s) => sum + s.wordCount, 0);
  const completedCount = stories.filter(s => s.status === "complete").length;
  const inProgressCount = stories.filter(s => s.status === "in-progress").length;

  return {
    totalWordCount,
    averageRating: stories.length > 0
      ? stories.reduce((sum, s) => sum + s.kudos, 0) / stories.length
      : 0,
    completedCount,
    inProgressCount
  };
}

// Usage
const draft = createDraft({
  title: "My New Story",
  author: "Alice",
  genres: ["romance", "fluff"]
});

const fullStory: StoryMetadata = {
  id: "abc-123",
  title: "The Art of Falling",
  author: "Eleanor Vance",
  summary: "A story about finding love in unexpected places.",
  rating: "T",
  status: "complete",
  language: "English",
  genres: ["romance", "hurt/comfort"],
  characters: [
    { name: "Eleanor", fandom: "Original", role: "protagonist" },
    { name: "James", fandom: "Original", role: "protagonist" }
  ],
  wordCount: 45200,
  chapterCount: 12,
  publishDate: "2025-01-15",
  updateDate: "2025-03-20",
  kudos: 1247,
  comments: 89,
  bookmarks: 342,
  isExplicit: false,
  warnings: ["Major Character Death"]
};

// These work because the types are correct:
const info = formatStoryInfo(fullStory);
const romanceStories = filterByGenre([fullStory, draft], "romance");
const stats = getStats([fullStory]);
```

Notice how every property has a type, optional properties use `?`, and we use unions for fixed sets of values. If you try to set `rating: "X"`, TypeScript catches it. If you forget `title`, TypeScript catches that too.

> **Try It Yourself:**
>
> Extend the metadata system above by adding:
> 1. A `Fandom` interface with name and source type (anime, book, movie, etc.)
> 2. An `Pairing` interface with two character names and a relationship type
> 3. Make `genres` require at least one genre (hint: you can use tuple types for this!)
>
> See if TypeScript catches your mistakes when you try to create invalid data.

> **Key Takeaways:**
> - **Primitive types:** `string`, `number`, `boolean`, `null`, `undefined`
> - **Arrays:** `string[]` or `Array<string>` — both work the same way
> - **Interfaces** define the shape of objects with named properties
> - **Optional properties** use `?` to mark non-required fields
> - **Union types** (`string | number`) allow multiple types
> - **Literal types** (`"active" | "inactive"`) restrict to specific values
> - **Type aliases** (`type Status = ...`) name any type
> - **Generics** (`<T>`) create reusable, type-safe code
> - **Utility types** (`Partial<T>`, `Pick<T, K>`, `Omit<T, K>`) transform existing types

---

## Chapter 12: TypeScript in Svelte Components

### Where Types Meet Components

You've learned the basics of TypeScript's type system. Now let's see how it works *inside* Svelte components. This is where the rubber meets the road — where types go from theoretical to practical, where they actually make your day-to-day coding easier and safer.

Svelte 5's runes (`$props()`, `$state()`, `$derived()`) work beautifully with TypeScript. Let's learn exactly how.

### Typing Props: What Your Component Expects

Props are how components receive data from their parents. With TypeScript, you can declare exactly what props a component accepts:

```svelte
<script lang="ts">
  let { message, count, isVisible }: {
    message: string;
    count: number;
    isVisible: boolean;
  } = $props();
</script>

{#if isVisible}
  <p>{message}: {count}</p>
{/if}
```

Let's break that down:

1. `lang="ts"` in the `<script>` tag tells Svelte this is TypeScript
2. `$props()` returns an object with all the props
3. We destructure it with types: `{ message, count, isVisible }: { ... }`
4. The type annotation after the colon says "this object must have a `message` (string), `count` (number), and `isVisible` (boolean)"

Now, if someone tries to use your component wrong:

```svelte
<!-- ✅ Correct usage -->
<MyComponent message="Hello" count={42} isVisible={true} />

<!-- ❌ Wrong types -->
<MyComponent message={42} count="hello" isVisible="yes" />

<!-- ❌ Missing props -->
<MyComponent message="Hello" />
```

TypeScript catches all of these mistakes. Your editor shows red squiggly lines, and the build fails with helpful error messages.

For optional props, use the `?` modifier:

```svelte
<script lang="ts">
  let { title, subtitle, author }: {
    title: string;
    subtitle?: string;  // Optional!
    author?: string;     // Optional!
  } = $props();
</script>

<h1>{title}</h1>
{#if subtitle}
  <h2>{subtitle}</h2>
{/if}
{#if author}
  <p>by {author}</p>
{/if}
```

> **Watch Out:** When you destructure props with optional types, remember that optional props might be `undefined`. You can't access `.length` or other methods on `undefined` without checking first. Use `{#if subtitle}` blocks in templates, or optional chaining (`subtitle?.length`) in script blocks.

Here's a more realistic component with many prop types:

```svelte
<script lang="ts">
  interface Props {
    title: string;
    author: string;
    wordCount: number;
    tags: string[];
    rating: "G" | "T" | "M" | "MA";
    isComplete: boolean;
    coverUrl?: string;
    onFavorite?: (storyId: string) => void;
  }

  let {
    title,
    author,
    wordCount,
    tags,
    rating,
    isComplete,
    coverUrl,
    onFavorite
  }: Props = $props();

  let isFavorited = $state(false);

  function handleFavorite() {
    isFavorited = !isFavorited;
    onFavorite?.(title);  // Optional chaining — only call if it exists
  }
</script>

<article class="story-card">
  {#if coverUrl}
    <img src={coverUrl} alt={title} class="cover" />
  {/if}

  <h2>{title}</h2>
  <p class="author">by {author}</p>

  <div class="meta">
    <span class="rating">{rating}</span>
    <span class="words">{wordCount.toLocaleString()} words</span>
    <span class="status" class:complete={isComplete}>
      {isComplete ? "Complete" : "In Progress"}
    </span>
  </div>

  <div class="tags">
    {#each tags as tag}
      <span class="tag">{tag}</span>
    {/each}
  </div>

  {#if onFavorite}
    <button onclick={handleFavorite}>
      {isFavorited ? "❤️ Favorited" : "🤍 Favorite"}
    </button>
  {/if}
</article>
```

### A Better Way: Interfaces for Props

Writing the type inline every time gets repetitive. A cleaner approach is to define an interface for your props:

```svelte
<script lang="ts">
  import type { StoryMetadata } from '$lib/types';

  interface Props {
    story: StoryMetadata;
    showSummary?: boolean;
    onRead?: (storyId: string) => void;
  }

  let { story, showSummary = false, onRead }: Props = $props();
</script>

<div class="story-card">
  <h2>{story.title}</h2>
  <p>by {story.author}</p>
  {#if showSummary}
    <p>{story.summary}</p>
  {/if}
  <span>{story.wordCount.toLocaleString()} words</span>
  {#if onRead}
    <button onclick={() => onRead(story.id)}>Read</button>
  {/if}
</div>
```

This is much cleaner because:

1. The interface has a descriptive name (`Props`)
2. It can be reused if other components accept similar props
3. The component signature is easy to read
4. Default values (`showSummary = false`) work alongside the types

> **Try It Yourself:**
>
> Create a `StoryCard.svelte` component with these props:
> ```typescript
> interface Props {
>   title: string;
>   author: string;
>   wordCount: number;
>   tags: string[];
>   isComplete?: boolean;
>   rating?: "G" | "T" | "M" | "MA";
>   onClick?: () => void;
> }
> ```
> Use the props in the template with conditional rendering for optional fields.

### Typing State: `$state` with Generics

The `$state` rune creates reactive state. With TypeScript, you can specify the exact type:

```svelte
<script lang="ts">
  // TypeScript infers these from the initial values
  let count = $state(0);              // number
  let name = $state("Alice");         // string
  let isActive = $state(true);        // boolean

  // Explicit types when needed
  let items = $state<string[]>([]);   // string array
  let selectedId = $state<string | null>(null);  // string or null

  // Complex state
  let user = $state<{
    name: string;
    email: string;
    preferences: {
      theme: "light" | "dark";
      fontSize: number;
    };
  } | null>(null);
</script>

<button onclick={() => count++}>{count}</button>
```

When the state type is complex, you can use an interface:

```svelte
<script lang="ts">
  interface UserPreferences {
    theme: "light" | "dark";
    fontSize: number;
    notifications: boolean;
  }

  let preferences = $state<UserPreferences>({
    theme: "light",
    fontSize: 16,
    notifications: true
  });

  function toggleTheme() {
    preferences.theme = preferences.theme === "light" ? "dark" : "light";
  }
</script>

<button onclick={toggleTheme}>
  Current theme: {preferences.theme}
</button>
```

### Typing Derived Values: `$derived`

The `$derived` rune computes values from other state. TypeScript infers the return type automatically:

```svelte
<script lang="ts">
  let count = $state(0);
  let items = $state<string[]>([]);

  // TypeScript infers: number
  let doubled = $derived(count * 2);

  // TypeScript infers: string
  let greeting = $derived(`Count is ${count}`);

  // TypeScript infers: boolean
  let isPositive = $derived(count > 0);

  // TypeScript infers: number
  let itemCount = $derived(items.length);

  // TypeScript infers: string
  let summary = $derived(
    items.length === 0
      ? "No items"
      : items.length === 1
        ? "1 item"
        : `${items.length} items`
  );
</script>

<p>Count: {count} (doubled: {doubled})</p>
<p>{greeting}</p>
<p>Items: {summary}</p>
```

For complex derived values, you can add explicit types:

```svelte
<script lang="ts">
  import type { StoryMetadata } from '$lib/types';

  let stories = $state<StoryMetadata[]>([]);
  let filterGenre = $state<string>("all");

  interface FilteredResult {
    stories: StoryMetadata[];
    totalCount: number;
    filteredCount: number;
  }

  let filtered = $derived<FilteredResult>((() => {
    const filtered = filterGenre === "all"
      ? stories
      : stories.filter(s => s.genres.includes(filterGenre as any));

    return {
      stories: filtered,
      totalCount: stories.length,
      filteredCount: filtered.length
    };
  })());
</script>
```

### Typing Events: Event Handlers

Event handlers in Svelte are regular functions, and you can type the event parameter:

```svelte
<script lang="ts">
  let inputValue = $state("");

  // Type the event parameter for DOM events
  function handleSubmit(e: Event) {
    e.preventDefault();
    const form = e.target as HTMLFormElement;
    const data = new FormData(form);
    console.log(data.get("search"));
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      console.log("Enter pressed!");
    }
    if (e.key === "Escape") {
      inputValue = "";
    }
  }

  function handleClick(e: MouseEvent) {
    console.log(`Clicked at ${e.clientX}, ${e.clientY}`);
  }

  function handleChange(e: Event) {
    const target = e.target as HTMLInputElement;
    inputValue = target.value;
  }
</script>

<input
  type="text"
  value={inputValue}
  onkeydown={handleKeydown}
  onchange={handleChange}
/>

<form onsubmit={handleSubmit}>
  <input type="search" name="search" />
  <button type="submit">Search</button>
</form>

<div onclick={handleClick}>Click me</div>
```

The most common event types are:

- `MouseEvent` — for click, mouseenter, mouseleave, etc.
- `KeyboardEvent` — for keydown, keyup, keypress
- `Event` — for generic events (change, submit)
- `FocusEvent` — for focus, blur
- `InputEvent` — for input events on text fields

> **Watch Out:** When you need to access `e.target`, you often need to cast it with `as HTMLInputElement` or similar. This is because `Event.target` is typed as `EventTarget | null`, which is very generic. Casting tells TypeScript "trust me, this is an input element."

### The `$types` Module: SvelteKit's Gift

One of SvelteKit's most powerful features is the `$types` module. When you create routes and pages, SvelteKit automatically generates TypeScript types for the data flowing through your app.

Here's how it works in a page component:

```svelte
<script lang="ts">
  import type { PageData } from './$types';

  let { data }: { data: PageData } = $props();

  // TypeScript knows exactly what properties 'data' has!
  // Based on your +page.server.ts load function
</script>

<h1>{data.title}</h1>
<p>{data.author}</p>
```

And in your server-side load function:

```typescript
// src/routes/stories/[id]/+page.server.ts
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params }) => {
  const story = await db.stories.findUnique({
    where: { id: params.id }
  });

  if (!story) {
    throw error(404, 'Story not found');
  }

  return {
    title: story.title,
    author: story.author,
    wordCount: story.wordCount,
    summary: story.summary
  };
};
```

The magic here is that SvelteKit reads your `load` function's return type and generates the `PageData` type automatically. If you add a new field to the return value, TypeScript knows about it in your component. If you remove a field, TypeScript tells you everywhere it was used.

You can also import types for layouts, actions, and more:

```typescript
// For layout components
import type { LayoutData } from './$types';

// For form actions
import type { Actions } from './$types';

// For page props (Svelte 5 style)
import type { PageProps } from './$types';

// For the error page
import type { ErrorProps } from './$types';
```

In Svelte 5, you can use `PageProps` for a cleaner component signature:

```svelte
<script lang="ts">
  import type { PageProps } from './$types';

  let { data, params }: PageProps = $props();

  // data comes from the load function
  // params comes from the route parameters
</script>

<h1>{data.title}</h1>
<p>ID: {params.id}</p>
```

This is a huge win. You get full type safety across your entire SvelteKit app — from the database query in the load function, through the data in the component, all the way to the HTML in the template.

### Typing Fetch Responses

When you fetch data from an API (either on the client or server), TypeScript needs to know what the response looks like:

```svelte
<script lang="ts">
  import type { StoryMetadata } from '$lib/types';

  let stories = $state<StoryMetadata[]>([]);
  let error = $state<string | null>(null);
  let loading = $state(false);

  async function searchStories(query: string): Promise<void> {
    loading = true;
    error = null;

    try {
      const response = await fetch(`/api/search?q=${encodeURIComponent(query)}`);

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      // Type the JSON response
      const data: { stories: StoryMetadata[] } = await response.json();
      stories = data.stories;
    } catch (e) {
      error = e instanceof Error ? e.message : "An unknown error occurred";
    } finally {
      loading = false;
    }
  }
</script>

{#if loading}
  <p>Searching...</p>
{:else if error}
  <p class="error">{error}</p>
{:else}
  {#each stories as story}
    <div>{story.title} by {story.author}</div>
  {/each}
{/if}
```

Notice how we type `data` directly in the assignment: `const data: { stories: StoryMetadata[] } = await response.json()`. This tells TypeScript exactly what shape the response has.

For more complex API responses, define interfaces:

```typescript
interface SearchResponse {
  stories: StoryMetadata[];
  totalCount: number;
  page: number;
  pageSize: number;
}

interface ApiError {
  error: string;
  code: number;
}

async function searchStories(query: string): Promise<SearchResponse> {
  const response = await fetch(`/api/search?q=${query}`);
  const data = await response.json();
  return data as SearchResponse;
}
```

### Common Type Errors and How to Fix Them

Let's go through the type errors you'll see most often in Svelte components, and how to fix them. These are the errors that trip up everyone at first, so don't feel bad if you hit them — they're part of the learning process!

**Error 1: Property does not exist**

```typescript
// ❌ Error: Property 'rating' does not exist on type 'StoryMetadata'
function getRating(story: StoryMetadata) {
  return story.rating;
}
```

**Fix:** Add the property to your interface, or check if it's an optional property you forgot to handle.

```typescript
// Add rating to your interface
interface StoryMetadata {
  // ... other properties
  rating?: number;  // Make it optional if it might not exist
}

// Or handle the optional case
function getRating(story: StoryMetadata): string {
  if (story.rating !== undefined) {
    return story.rating.toFixed(1);
  }
  return "N/A";
}
```

**Error 2: Type 'string' is not assignable to type 'number'**

```typescript
// ❌ Error in component
let count = $state("0");  // TypeScript infers: string

// Later...
count = count + 1;  // Error! String + number = string concatenation
```

**Fix:** Use the correct type:

```typescript
let count = $state(0);  // TypeScript infers: number
count = count + 1;  // ✅ Works!
```

**Error 3: Object is possibly undefined**

```typescript
// ❌ Error: 'user' is possibly 'undefined'
let user = $state<{ name: string } | null>(null);
let userName = user.name;  // Error! user might be null
```

**Fix:** Check for null first:

```typescript
let user = $state<{ name: string } | null>(null);

// Option 1: Check before using
if (user) {
  let userName = user.name;  // ✅ Safe!
}

// Option 2: Use optional chaining
let userName = user?.name;  // Returns undefined if user is null

// Option 3: Use nullish coalescing for a default
let userName = user?.name ?? "Anonymous";
```

**Error 4: No overload matches this call**

```typescript
// ❌ Error when passing wrong types to a function
function greet(name: string, age: number): void { ... }
greet("Alice");  // Error: Expected 2 arguments, got 1
greet("Alice", "thirty");  // Error: 'string' not assignable to 'number'
```

**Fix:** Match the function signature:

```typescript
greet("Alice", 30);  // ✅ Correct!
```

**Error 5: Cannot find module**

```typescript
// ❌ Error: Cannot find module '$lib/types'
import type { StoryMetadata } from '$lib/types';
```

**Fix:** Make sure the file exists and is properly typed:

```typescript
// src/lib/types.ts
export interface StoryMetadata {
  id: string;
  title: string;
  author: string;
  // ... etc
}
```

**Error 6: Type 'X' is not assignable to type 'Y' (in array methods)**

```typescript
// ❌ Error when filtering
let stories: StoryMetadata[] = [];
let titles = stories.map(s => s.title);  // Error if 'title' doesn't exist on type
```

**Fix:** Make sure the property exists on your type:

```typescript
interface StoryMetadata {
  title: string;  // ← Must be defined here
  // ...
}

let titles = stories.map(s => s.title);  // ✅ Works!
```

**Error 7: Implicit any (in callbacks)**

```typescript
// ❌ Error: Parameter 'e' implicitly has an 'any' type
element.addEventListener("click", (e) => {
  console.log(e.target);
});
```

**Fix:** Type the event parameter:

```typescript
element.addEventListener("click", (e: MouseEvent) => {
  console.log(e.target);  // ✅ TypeScript knows it's a MouseEvent
});
```

> **Watch Out:** The "implicit any" error is one of the most common for beginners. It happens when TypeScript can't figure out a type and defaults to `any`. Always check that your function parameters are typed — especially in event handlers and array methods like `.map()`, `.filter()`, and `.reduce()`.

### The `satisfies` Operator: The Best of Both Worlds

The `satisfies` operator is a newer TypeScript feature that's incredibly useful in SvelteKit. It checks that a value matches a type *without widening* the type to include all possible shapes.

```typescript
interface Route {
  path: string;
  label: string;
  icon?: string;
}

// Without satisfies: TypeScript widens the type
const routes1 = [
  { path: "/", label: "Home", icon: "🏠" },
  { path: "/stories", label: "Stories" },  // No icon — TypeScript doesn't catch this
  { path: "/about", label: "About", icon: "ℹ️" }
];
// routes1 is typed as { path: string; label: string; icon?: string }[]
// The missing icon on the second item is silently allowed

// With satisfies: TypeScript validates the shape
const routes2 = [
  { path: "/", label: "Home", icon: "🏠" },
  { path: "/stories", label: "Stories" },  // ❌ Error if icon is required!
  { path: "/about", label: "About", icon: "ℹ️" }
] satisfies Route[];
// TypeScript checks each item against Route
// BUT keeps the literal types (not widened)
```

The key difference is:

- **Without `satisfies`:** TypeScript infers a general type that might lose specific information
- **With `satisfies`:** TypeScript validates the type AND keeps the specific literal information

This is especially useful for config objects:

```typescript
type Theme = "light" | "dark" | "system";

interface AppConfig {
  theme: Theme;
  apiUrl: string;
  debug: boolean;
}

// satisfies validates the config and keeps literal types
const config = {
  theme: "dark",      // Typed as "dark", not Theme
  apiUrl: "https://api.example.com",
  debug: false        // Typed as false, not boolean
} satisfies AppConfig;

// Because theme is "dark" (literal type), this works:
if (config.theme === "dark") {
  // TypeScript knows this is always true! No error.
}

// And this would error:
if (config.theme === "light") {
  // TypeScript knows this is impossible! ✅ Caught at compile time.
}
```

In SvelteKit, `satisfies` is great for typing route configurations, API response shapes, and configuration objects where you want both validation and precise types.

> **Try It Yourself:**
>
> Create a Svelte component that:
> 1. Accepts a `story` prop with the `StoryMetadata` interface from Chapter 11
> 2. Has local state for whether the summary is expanded
> 3. Uses `$derived` to compute a truncated summary (first 100 characters)
> 4. Handles a click event to toggle the summary
> 5. Uses the `$types` module pattern for props
>
> Make sure everything is properly typed — no `any` allowed!

> **Key Takeaways:**
> - Use `lang="ts"` in `<script>` to enable TypeScript in Svelte components
> - Type props with an interface and `$props()`
> - `$state` and `$derived` infer types from initial values, or accept explicit types
> - Type event handlers with `MouseEvent`, `KeyboardEvent`, etc.
> - SvelteKit's `$types` module generates types from your load functions
> - Always type fetch responses for API calls
> - The `satisfies` operator validates types while preserving literal types

### Quick Reference: TypeScript Patterns for Svelte Components

Here's a cheat sheet you can bookmark for when you're writing Svelte components with TypeScript:

```svelte
<script lang="ts">
  // === PROPS ===
  // Basic props
  let { title, count }: { title: string; count: number } = $props();

  // Optional props
  let { subtitle }: { subtitle?: string } = $props();

  // Default values with types
  let { size = "medium" }: { size?: "small" | "medium" | "large" } = $props();

  // Callback props
  let { onSelect }: { onSelect: (id: string) => void } = $props();

  // === STATE ===
  let name = $state("Alice");           // string (inferred)
  let count = $state(0);                // number (inferred)
  let items = $state<string[]>([]);     // string[] (explicit)
  let selected = $state<string | null>(null);  // string | null

  // === DERIVED ===
  let doubled = $derived(count * 2);            // number (inferred)
  let greeting = $derived(`Hello, ${name}`);   // string (inferred)
  let firstItem = $derived(items[0] ?? null);  // string | null

  // === EVENTS ===
  function handleClick(e: MouseEvent) {
    console.log(e.clientX, e.clientY);
  }

  function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    const form = e.target as HTMLFormElement;
    const data = new FormData(form);
  }

  // === ASYNC DATA ===
  let data = $state<StoryMetadata | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function fetchData() {
    loading = true;
    error = null;
    try {
      const response = await fetch("/api/stories");
      data = await response.json();
    } catch (e) {
      error = e instanceof Error ? e.message : "Unknown error";
    } finally {
      loading = false;
    }
  }
</script>

<!-- In the template, TypeScript checks everything! -->
<h1>{title}</h1>
{#if subtitle}
  <p>{subtitle}</p>
{/if}
<p>Count: {count} (doubled: {doubled})</p>
{#if data}
  <p>{data.title}</p>
{:else if loading}
  <p>Loading...</p>
{:else if error}
  <p class="error">{error}</p>
{/if}
```

This pattern covers 90% of what you'll need when writing typed Svelte components. Keep it handy until the patterns become second nature!

---

## Chapter 13: Type-Safe API Communication

### The Frontier of Your Application

Your SvelteKit app doesn't live in isolation. It talks to APIs — fetching data from backends, sending user actions to servers, receiving updates in real-time. This communication is where a surprising number of bugs hide.

Why? Because the data that comes back from an API is just... JSON. And JSON, by itself, has no type information. It's a pile of strings, numbers, objects, and arrays. Without TypeScript, you're guessing what the data looks like. With TypeScript, you *know*.

In this chapter, we'll learn how to design TypeScript types from API responses, create type-safe fetch functions, and handle the messy reality of real-world APIs.

### Designing Types from API Responses

The first step to type-safe API communication is understanding what your API actually returns. This might sound obvious, but you'd be surprised how many developers guess at the shape of API responses and get burned.

Here's the process:

1. **Make the API call.** Use a tool like `curl`, Postman, or your browser's network tab.
2. **Look at the JSON.** Understand every field, every nesting level, every possible value.
3. **Write TypeScript types that match.** Create interfaces for the response shape.
4. **Use those types everywhere.** Never work with raw JSON again.

Let's say you're building a fic tracking app, and you need to fetch story data from an API. First, let's look at what the API returns:

```json
{
  "id": "abc-123",
  "title": "The Art of Falling",
  "author": {
    "name": "Eleanor Vance",
    "id": "user-456",
    "avatar": "https://example.com/avatar.jpg"
  },
  "chapters": [
    {
      "id": "ch-1",
      "title": "Chapter 1: Beginnings",
      "wordCount": 3500,
      "publishedAt": "2025-01-15T10:30:00Z"
    },
    {
      "id": "ch-2",
      "title": "Chapter 2: Complications",
      "wordCount": 4200,
      "publishedAt": "2025-01-22T14:00:00Z"
    }
  ],
  "totalWordCount": 45200,
  "rating": "T",
  "status": "in-progress",
  "tags": ["romance", "hurt/comfort", "slow burn"],
  "publishDate": "2025-01-15T10:30:00Z",
  "updateDate": "2025-03-20T08:15:00Z",
  "stats": {
    "kudos": 1247,
    "comments": 89,
    "bookmarks": 342,
    "hits": 15600
  },
  "summary": "A story about finding love in unexpected places...",
  "isComplete": false
}
```

Now, let's create TypeScript interfaces that match this exactly:

```typescript
// The author object
interface Author {
  name: string;
  id: string;
  avatar: string;
}

// A single chapter
interface Chapter {
  id: string;
  title: string;
  wordCount: number;
  publishedAt: string;  // ISO date string
}

// Story statistics
interface StoryStats {
  kudos: number;
  comments: number;
  bookmarks: number;
  hits: number;
}

// The full story response
interface StoryResponse {
  id: string;
  title: string;
  author: Author;
  chapters: Chapter[];
  totalWordCount: number;
  rating: "G" | "T" | "M" | "MA";
  status: "in-progress" | "complete" | "abandoned";
  tags: string[];
  publishDate: string;
  updateDate: string | null;  // null if never updated
  stats: StoryStats;
  summary: string;
  isComplete: boolean;
}
```

Notice several important choices:

- **`updateDate: string | null`** — This field might be null (if the story was never updated). We use a union type to express this.
- **`rating: "G" | "T" | "M" | "MA"`** — We use literal types for the fixed set of ratings.
- **`chapters: Chapter[]`** — We create a separate interface for chapters, keeping things organized.
- **`stats: StoryStats`** — Nested objects get their own interfaces.

> **Watch Out:** Don't use `any` for JSON responses! If you type your response as `any`, you lose all the safety benefits. Always take the time to create proper interfaces. Your future self (and your users) will thank you.

### Reading the Backend: What Does the JSON Look Like?

If you're working with a backend you control (like a Rust API or a SvelteKit server route), you might already know the exact shape of the response. But if you're consuming an external API, you need to inspect the actual data.

Here are some practical techniques:

**1. Use the browser's network tab:**

Open your browser's developer tools, go to the Network tab, make the API call, and inspect the response. Most browsers let you copy the response as a JavaScript object.

**2. Use curl:**

```bash
curl -s https://api.example.com/stories/abc-123 | python3 -m json.tool
```

**3. Use TypeScript to explore:**

```typescript
// Start with a loose type
async function fetchStory(id: string): Promise<unknown> {
  const response = await fetch(`/api/stories/${id}`);
  return response.json();
}

// Then narrow it down
const story = await fetchStory("abc-123");

// TypeScript will tell you what you CAN do with 'unknown':
console.log(story.id);  // ❌ Error: 'unknown' has no properties
```

When the response is typed as `unknown`, TypeScript forces you to narrow it down before using it. This is safe — you can't accidentally use a field that doesn't exist.

**4. Use a schema validation library:**

For APIs you don't control, consider using libraries like Zod:

```typescript
import { z } from 'zod';

// Define the schema
const StorySchema = z.object({
  id: z.string(),
  title: z.string(),
  author: z.object({
    name: z.string(),
    id: z.string(),
    avatar: z.string()
  }),
  totalWordCount: z.number(),
  rating: z.enum(["G", "T", "M", "MA"]),
  status: z.enum(["in-progress", "complete", "abandoned"]),
  tags: z.array(z.string()),
  summary: z.string(),
  isComplete: z.boolean()
});

// Get a TypeScript type from the schema
type StoryResponse = z.infer<typeof StorySchema>;

// Validate and parse the response
async function fetchStory(id: string): Promise<StoryResponse> {
  const response = await fetch(`/api/stories/${id}`);
  const json = await response.json();
  return StorySchema.parse(json);  // Throws if validation fails
}
```

Zod validates the data at runtime AND provides TypeScript types. It's the best of both worlds.

### Creating TypeScript Interfaces from JSON

Let's practice turning real JSON responses into TypeScript types. Here's a systematic approach:

**Step 1: Identify the top-level keys**

```json
{
  "stories": [...],
  "totalCount": 42,
  "page": 1,
  "pageSize": 20,
  "hasMore": true
}
```

**Step 2: Determine the type of each key**

- `stories` → array of objects
- `totalCount` → number
- `page` → number
- `pageSize` → number
- `hasMore` → boolean

**Step 3: Create the interface for the top level**

```typescript
interface SearchResults {
  stories: StoryMetadata[];  // We'll define this next
  totalCount: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
}
```

**Step 4: Drill into nested objects and create their interfaces**

```json
{
  "stories": [
    {
      "id": "abc-123",
      "title": "The Art of Falling",
      "author": "Eleanor Vance",
      "wordCount": 45200,
      "rating": "T",
      "status": "complete",
      "tags": ["romance", "hurt/comfort"],
      "summary": "A story about...",
      "coverUrl": "https://example.com/cover.jpg",
      "isComplete": true,
      "publishDate": "2025-01-15"
    }
  ]
}
```

**Step 5: Create the nested interface**

```typescript
interface StoryMetadata {
  id: string;
  title: string;
  author: string;
  wordCount: number;
  rating: "G" | "T" | "M" | "MA";
  status: "in-progress" | "complete" | "abandoned" | "on-hiatus";
  tags: string[];
  summary: string;
  coverUrl: string | null;  // Might not have a cover
  isComplete: boolean;
  publishDate: string;
}
```

**Step 6: Handle edge cases**

Look for fields that might be `null`, `undefined`, or have varying types:

```typescript
// coverUrl might be a string OR null (no cover image)
coverUrl: string | null;

// updateDate might not exist at all
updateDate?: string;

// errors might be an array or a single string
error: string | string[];
```

> **Try It Yourself:**
>
> Take this JSON response and create TypeScript interfaces for it:
> ```json
> {
>   "status": "success",
>   "data": {
>     "user": {
>       "id": "user-789",
>       "username": "ficreader42",
>       "joinDate": "2024-06-15",
>       "subscription": {
>         "plan": "premium",
>         "expiresAt": "2026-06-15"
>       }
>     },
>     "readingList": [
>       {
>         "storyId": "story-111",
>         "title": "My Favorite Story",
>         "progress": 0.75,
>         "lastRead": "2025-03-20"
>       }
>     ]
>   }
> }
> ```
> Create interfaces for every level of nesting. Handle nullable fields properly.

### The FicHub API Types

Let's look at real types for the FicHub API — the fic tracking system you're building. These types represent the actual data structures your app will work with. Here are some key type definitions:

```typescript
// Export response: what you get when you export a story
interface ExportResponse {
  storyId: string;
  title: string;
  author: string;
  exportedAt: string;
  format: "epub" | "pdf" | "html" | "txt";
  downloadUrl: string;
  fileSize: number;  // bytes
  expiresAt: string;  // when the download link expires
  error?: string;     // Optional: error message if export partially failed
}

// Recommendation result
interface RecResult {
  storyId: string;
  title: string;
  author: string;
  matchScore: number;  // 0-100, how well it matches your preferences
  tags: string[];
  summary: string;
  wordCount: number;
  rating: "G" | "T" | "M" | "MA";
  reason: string;  // Why this was recommended
}

// Suggestion: a quick suggestion for what to read
interface Suggestion {
  id: string;
  title: string;
  author: string;
  genre: string;
  matchPercentage: number;
  available: boolean;  // Is this story still available?
}

// API error response
interface ApiError {
  error: string;
  code: number;
  details?: string;
  timestamp: string;
}

// Paginated response (generic!)
interface PaginatedResponse<T> {
  data: T[];
  totalCount: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
  nextPage: number | null;
}

// Usage:
// PaginatedResponse<RecResult> → paginated recommendations
// PaginatedResponse<StoryMetadata> → paginated story list
```

Notice how we use generics for the paginated response: `PaginatedResponse<T>`. This lets us reuse the same pagination structure for any type of data. The `T` gets replaced with the actual data type when you use it.

Why is this powerful? Because pagination logic is always the same — you need page numbers, total counts, "has more" flags. But the *data* changes depending on what you're paginating. With generics, you write the pagination logic once and use it for stories, recommendations, search results, and anything else:

```typescript
// All three use the same PaginatedResponse structure
type StoryPage = PaginatedResponse<StoryMetadata>;
type RecPage = PaginatedResponse<RecResult>;
type SearchPage = PaginatedResponse<SearchStory>;

// The load function for a paginated stories page
async function loadStories(page: number): Promise<StoryPage> {
  const response = await fetch(`/api/stories?page=${page}`);
  return response.json();
}

// TypeScript knows exactly what 'result.data' contains
const result = await loadStories(1);
console.log(result.data[0].title);  // ✅ StoryMetadata has 'title'
console.log(result.totalCount);      // ✅ number
console.log(result.hasMore);         // ✅ boolean
```

### Typing the Fetch Function

Now let's write type-safe fetch functions that use these types:

```typescript
// src/lib/api.ts

import type {
  ExportResponse,
  RecResult,
  Suggestion,
  PaginatedResponse
} from './types';

const API_BASE = '/api';

// Generic fetch helper with error handling
async function apiFetch<T>(endpoint: string): Promise<T> {
  const response = await fetch(`${API_BASE}${endpoint}`);

  if (!response.ok) {
    const error: ApiError = await response.json();
    throw new Error(error.error || `API error: ${response.status}`);
  }

  return response.json() as Promise<T>;
}

// Type-safe API functions
export async function getExport(storyId: string): Promise<ExportResponse> {
  return apiFetch<ExportResponse>(`/stories/${storyId}/export`);
}

export async function getRecommendations(
  page: number = 1
): Promise<PaginatedResponse<RecResult>> {
  return apiFetch<PaginatedResponse<RecResult>>(
    `/recommendations?page=${page}&pageSize=20`
  );
}

export async function getSuggestions(): Promise<Suggestion[]> {
  return apiFetch<Suggestion[]>('/suggestions');
}

export async function searchStories(
  query: string,
  page: number = 1
): Promise<PaginatedResponse<StoryMetadata>> {
  return apiFetch<PaginatedResponse<StoryMetadata>>(
    `/search?q=${encodeURIComponent(query)}&page=${page}`
  );
}
```

Here's what's powerful about this:

1. **Every function has a return type.** You know exactly what each function returns.
2. **The generic `apiFetch<T>`** handles the common pattern of fetch → check response → parse JSON.
3. **Error handling is typed.** We parse the error response as `ApiError` and get structured error information.
4. **Usage is clean and safe:**

```typescript
// In a component
<script lang="ts">
  import { getRecommendations } from '$lib/api';
  import type { RecResult } from '$lib/types';

  let recommendations = $state<RecResult[]>([]);
  let error = $state<string | null>(null);

  async function loadRecommendations() {
    try {
      const result = await getRecommendations();
      recommendations = result.data;  // TypeScript knows this is RecResult[]
      console.log(result.totalCount); // TypeScript knows this is number
      console.log(result.hasMore);    // TypeScript knows this is boolean
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load recommendations";
    }
  }
</script>
```

### Handling Optional Fields: `string | null`

Real-world APIs are messy. Fields that are sometimes present and sometimes not. Values that can be a string or null. Responses that might have an error instead of data.

Let's handle these scenarios properly:

```typescript
interface StoryDetail {
  id: string;
  title: string;
  author: string;
  summary: string;
  coverImage: string | null;        // null if no cover
  completionDate: string | null;    // null if not complete
  prequelId: string | null;         // null if no prequel
  sequelId: string | null;          // null if no sequel
  notes: string | undefined;        // undefined if not set
}

// Working with nullable fields safely
function displayStory(story: StoryDetail): string {
  let output = `${story.title} by ${story.author}\n`;

  // Check for null before using nullable fields
  if (story.coverImage !== null) {
    output += `Cover: ${story.coverImage}\n`;
  } else {
    output += "No cover image\n`;
  }

  // Use optional chaining for clean null checks
  const completionInfo = story.completionDate
    ? `Completed on ${story.completionDate}`
    : "Not yet completed";
  output += `${completionInfo}\n`;

  // Use nullish coalescing for defaults
  const displaySummary = story.summary || "No summary available";
  output += `${displaySummary}\n`;

  return output;
}
```

Key patterns for handling nullable fields:

```typescript
// Pattern 1: Explicit null check
if (value !== null) {
  // TypeScript knows value is NOT null here
  doSomething(value);
}

// Pattern 2: Optional chaining (?.)
const name = user?.profile?.name;  // Returns undefined if any part is null/undefined

// Pattern 3: Nullish coalescing (??)
const displayName = name ?? "Anonymous";  // Uses default if null or undefined

// Pattern 4: Truthy check (careful! empty strings and 0 are falsy)
if (value) {
  // Works for strings/numbers, but watch out for 0 and ""
}

// Pattern 5: Type narrowing with typeof
function processValue(value: string | number): string {
  if (typeof value === "string") {
    return value.toUpperCase();  // TypeScript knows it's a string
  } else {
    return value.toFixed(2);     // TypeScript knows it's a number
  }
}
```

> **Watch Out:** Don't confuse `null` and `undefined`! In TypeScript with strict mode:
> - `null` is an intentional absence of a value (you set it on purpose)
> - `undefined` is the absence of a value (it was never set)
>
> Use `??` (nullish coalescing) when you want to check for *either* null or undefined.
> Use `||` (logical OR) when you want to check for *any* falsy value (including empty string, 0, false).

### Type Guards: Checking if a Field Exists

**Type guards** are expressions that TypeScript uses to narrow down a type. You've already seen some (`typeof`, `!== null`), but let's look at more advanced patterns.

The key insight is that TypeScript can't always know the type of a value at compile time. Sometimes you need to check at runtime — is this value a string or a number? Does this object have a `data` property or an `error` property? Type guards tell TypeScript what you've figured out at runtime.

```typescript
// typeof guard: checking the basic type
function formatValue(value: string | number | boolean): string {
  if (typeof value === "string") {
    return value.toUpperCase();  // TypeScript knows it's a string
  }
  if (typeof value === "number") {
    return value.toFixed(2);     // TypeScript knows it's a number
  }
  return value ? "Yes" : "No";  // TypeScript knows it's a boolean
}

// instanceof guard: checking class instances
function formatDate(date: Date | string): string {
  if (date instanceof Date) {
    return date.toLocaleDateString();  // TypeScript knows it's a Date
  }
  return date;  // TypeScript knows it's a string
}

// "in" guard: checking if a property exists
interface Cat {
  meow(): void;
  purr(): void;
}

interface Dog {
  bark(): void;
  fetch(): void;
}

function makeSound(animal: Cat | Dog) {
  if ("meow" in animal) {
    animal.meow();   // TypeScript knows it's a Cat
  } else {
    animal.bark();   // TypeScript knows it's a Dog
  }
}
```

**Discriminated unions** are the most powerful type guard pattern for API communication. The idea: when a value can be in different states, use a common field (the **discriminant**) to tell them apart.

```typescript
// The discriminant is the 'status' field — it tells you which variant you have
type ApiResponse =
  | { status: "loading" }
  | { status: "success"; data: StoryMetadata[] }
  | { status: "error"; error: string };

// Type guard function
function handleResponse(response: ApiResponse) {
  switch (response.status) {
    case "loading":
      console.log("Loading...");
      break;
    case "success":
      // TypeScript knows response.data exists here!
      console.log(`Got ${response.data.length} stories`);
      response.data.forEach(story => {
        console.log(story.title);
      });
      break;
    case "error":
      // TypeScript knows response.error exists here!
      console.error(response.error);
      break;
  }
}
```

Type guard functions let you create reusable checks:

```typescript
// A type guard function
function isSuccessResponse(
  response: ApiResponse
): response is { status: "success"; data: StoryMetadata[] } {
  return response.status === "success";
}

// Usage
const response: ApiResponse = await fetchStories();

if (isSuccessResponse(response)) {
  // TypeScript knows response has .data
  console.log(response.data);
} else {
  // TypeScript knows response does NOT have .data
  console.log(response.status);
}
```

The `response is ...` return type is the magic. It tells TypeScript, "If this function returns true, then the parameter has this specific type."

### Discriminated Unions: The Power Pattern

You've seen a glimpse of **discriminated unions** above. This is one of TypeScript's most powerful patterns, especially for API communication.

The idea: when a value can be in different states, use a common field (the **discriminant**) to tell them apart.

```typescript
// Without discriminated unions (messy!)
type BadResponse = {
  data?: StoryMetadata[];
  error?: string;
  code?: number;
};

// You have to check multiple optional fields
function handleBadResponse(response: BadResponse) {
  if (response.data) {
    // But what if both data AND error exist? Ambiguous!
    // What if neither exists? We're stuck.
  }
}

// With discriminated unions (clean!)
type GoodResponse =
  | { status: "success"; data: StoryMetadata[] }
  | { status: "error"; error: string; code: number }
  | { status: "loading" };

// TypeScript knows exactly which variant you're in
function handleGoodResponse(response: GoodResponse) {
  switch (response.status) {
    case "success":
      // response.data is StoryMetadata[] — guaranteed!
      console.log(response.data.length);
      break;
    case "error":
      // response.error and response.code exist — guaranteed!
      console.log(`Error ${response.code}: ${response.error}`);
      break;
    case "loading":
      // No extra fields — just the status
      console.log("Still loading...");
      break;
  }
}
```

Here's a more realistic example for API responses:

```typescript
// API response variants
type ExportResult =
  | {
      success: true;
      data: {
        downloadUrl: string;
        fileSize: number;
        format: string;
      };
    }
  | {
      success: false;
      error: {
        code: string;
        message: string;
        retryable: boolean;
      };
    };

async function exportStory(storyId: string): Promise<ExportResult> {
  const response = await fetch(`/api/stories/${storyId}/export`);
  const json = await response.json();

  if (response.ok) {
    return {
      success: true,
      data: {
        downloadUrl: json.downloadUrl,
        fileSize: json.fileSize,
        format: json.format
      }
    };
  } else {
    return {
      success: false,
      error: {
        code: json.errorCode || "UNKNOWN",
        message: json.message || "Export failed",
        retryable: response.status >= 500
      }
    };
  }
}

// Usage: clean, safe, exhaustive
async function handleExport(storyId: string) {
  const result = await exportStory(storyId);

  if (result.success) {
    // TypeScript knows this is the success variant
    console.log(`Download: ${result.data.downloadUrl}`);
    console.log(`Size: ${result.data.fileSize} bytes`);
    startDownload(result.data.downloadUrl);
  } else {
    // TypeScript knows this is the error variant
    console.error(`Error ${result.error.code}: ${result.error.message}`);
    if (result.error.retryable) {
      showToast("Export failed. Please try again.");
    } else {
      showToast("Export failed. This error cannot be retried.");
    }
  }
}
```

Discriminated unions eliminate entire categories of bugs:

- You can't accidentally access `data` on an error response
- You can't accidentally access `error` on a success response
- You're forced to handle every possible state
- TypeScript's exhaustiveness checking ensures you don't miss cases

Here's another practical example — a form submission state machine:

```typescript
// A form can be in several states
type FormState =
  | { status: "idle" }
  | { status: "validating"; fields: string[] }
  | { status: "submitting"; progress: number }
  | { status: "success"; message: string; redirectUrl?: string }
  | { status: "error"; error: string; retryable: boolean };

// A component that handles all form states
function renderFormState(state: FormState): string {
  switch (state.status) {
    case "idle":
      return "Ready to submit";
    case "validating":
      return `Validating: ${state.fields.join(", ")}`;
    case "submitting":
      return `Submitting... ${state.progress}%`;
    case "success":
      return `✅ ${state.message}`;
    case "error":
      return `❌ ${state.error}${state.retryable ? " (retry available)" : ""}`;
    default:
      // TypeScript's exhaustiveness check!
      // If you add a new state and forget to handle it, this catches it
      const _exhaustive: never = state;
      return _exhaustive;
  }
}
```

That `never` trick is powerful: if you add a new variant to `FormState` and forget to handle it in the switch, TypeScript will error on the `default` case because `state` is no longer `never` — it still has unhandled variants. This forces you to handle every possible state.

Here's a complete real-world example combining discriminated unions with async operations:

```typescript
// Loading state for any async operation
type AsyncState<T> =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "success"; data: T; loadedAt: Date }
  | { status: "error"; error: string; canRetry: boolean };

// A typed store for managing async data
class AsyncStore<T> {
  private state: AsyncState<T> = { status: "idle" };

  getState(): AsyncState<T> {
    return this.state;
  }

  async load(fetcher: () => Promise<T>): Promise<void> {
    this.state = { status: "loading" };

    try {
      const data = await fetcher();
      this.state = {
        status: "success",
        data,
        loadedAt: new Date()
      };
    } catch (e) {
      this.state = {
        status: "error",
        error: e instanceof Error ? e.message : "Unknown error",
        canRetry: true
      };
    }
  }

  // Helper to render the current state
  render(
    onIdle: () => string,
    onLoading: () => string,
    onSuccess: (data: T) => string,
    onError: (error: string, canRetry: boolean) => string
  ): string {
    switch (this.state.status) {
      case "idle":
        return onIdle();
      case "loading":
        return onLoading();
      case "success":
        return onSuccess(this.state.data);
      case "error":
        return onError(this.state.error, this.state.canRetry);
    }
  }
}

// Usage with stories
const storiesStore = new AsyncStore<StoryMetadata[]>();

await storiesStore.load(async () => {
  const response = await fetch("/api/stories");
  return response.json();
});

// TypeScript knows exactly what each state contains
const result = storiesStore.render(
  () => "No stories loaded",
  () => "Loading stories...",
  (stories) => `Found ${stories.length} stories`,
  (error, canRetry) => canRetry
    ? `Error: ${error}. Click to retry.`
    : `Error: ${error}`
);
```

> **Try It Yourself:**
>
> Create a discriminated union for a search API response:
> ```typescript
> type SearchState =
>   | { status: "idle" }
>   | { status: "searching"; query: string }
>   | { status: "results"; results: StoryMetadata[]; totalCount: number }
>   | { status: "error"; message: string; canRetry: boolean };
> ```
> Write a function that takes a `SearchState` and returns a user-friendly message for each case.
> Make sure TypeScript forces you to handle all four states.

### Practice: Type a Search API Response

Let's put everything together with a complete, real-world example. We'll type a search API from start to finish:

```typescript
// src/lib/types/search.ts

// Individual search result
interface SearchStory {
  id: string;
  title: string;
  author: {
    id: string;
    name: string;
  };
  summary: string;
  wordCount: number;
  rating: "G" | "T" | "M" | "MA";
  status: "in-progress" | "complete" | "abandoned";
  tags: string[];
  publishedAt: string;
  updatedAt: string | null;
  coverImage: string | null;
}

// Search facets (filter counts)
interface SearchFacets {
  ratings: Array<{ value: string; count: number }>;
  statuses: Array<{ value: string; count: number }>;
  tags: Array<{ value: string; count: number }>;
}

// Successful search response
interface SearchSuccess {
  ok: true;
  stories: SearchStory[];
  totalCount: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
  facets: SearchFacets;
  query: string;
  took: number;  // milliseconds
}

// Error search response
interface SearchError {
  ok: false;
  error: string;
  code: number;
  retryable: boolean;
}

// Combined search response (discriminated union!)
type SearchResponse = SearchSuccess | SearchError;

// Search parameters
interface SearchParams {
  query: string;
  page?: number;
  pageSize?: number;
  rating?: "G" | "T" | "M" | "MA";
  status?: "in-progress" | "complete" | "abandoned";
  tags?: string[];
  sortBy?: "relevance" | "date" | "kudos" | "wordCount";
  sortOrder?: "asc" | "desc";
}

// Search function
async function searchStories(params: SearchParams): Promise<SearchResponse> {
  const searchParams = new URLSearchParams();
  searchParams.set("q", params.query);

  if (params.page) searchParams.set("page", params.page.toString());
  if (params.pageSize) searchParams.set("pageSize", params.pageSize.toString());
  if (params.rating) searchParams.set("rating", params.rating);
  if (params.status) searchParams.set("status", params.status);
  if (params.tags) searchParams.set("tags", params.tags.join(","));
  if (params.sortBy) searchParams.set("sortBy", params.sortBy);
  if (params.sortOrder) searchParams.set("sortOrder", params.sortOrder);

  try {
    const response = await fetch(`/api/search?${searchParams.toString()}`);

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      return {
        ok: false,
        error: errorData.message || `Search failed: ${response.status}`,
        code: response.status,
        retryable: response.status >= 500
      };
    }

    const data = await response.json();

    return {
      ok: true,
      stories: data.stories,
      totalCount: data.totalCount,
      page: data.page,
      pageSize: data.pageSize,
      hasMore: data.hasMore,
      facets: data.facets,
      query: params.query,
      took: data.took
    };
  } catch (e) {
    return {
      ok: false,
      error: e instanceof Error ? e.message : "Network error",
      code: 0,
      retryable: true
    };
  }
}

// Usage in a Svelte component:
// src/routes/search/+page.svelte

<script lang="ts">
  import { searchStories } from '$lib/types/search';
  import type { SearchStory, SearchParams } from '$lib/types/search';

  let query = $state("");
  let results = $state<SearchStory[]>([]);
  let totalCount = $state(0);
  let error = $state<string | null>(null);
  let loading = $state(false);
  let facets = $state<{
    ratings: Array<{ value: string; count: number }>;
    statuses: Array<{ value: string; count: number }>;
    tags: Array<{ value: string; count: number }>;
  } | null>(null);

  async function handleSearch() {
    if (!query.trim()) return;

    loading = true;
    error = null;

    const searchResult = await searchStories({
      query: query.trim(),
      pageSize: 20,
      sortBy: "relevance"
    });

    if (searchResult.ok) {
      results = searchResult.stories;
      totalCount = searchResult.totalCount;
      facets = searchResult.facets;
      console.log(`Search took ${searchResult.took}ms`);
    } else {
      error = searchResult.error;
      if (searchResult.retryable) {
        error += " (You can try again)";
      }
    }

    loading = false;
  }
</script>

<form onsubmit={(e) => { e.preventDefault(); handleSearch(); }}>
  <input
    type="search"
    bind:value={query}
    placeholder="Search stories..."
  />
  <button type="submit" disabled={loading}>
    {loading ? "Searching..." : "Search"}
  </button>
</form>

{#if error}
  <p class="error">{error}</p>
{:else if loading}
  <p>Searching for "{query}"...</p>
{:else}
  <p>{totalCount} results found</p>

  {#each results as story (story.id)}
    <div class="search-result">
      <h3>{story.title}</h3>
      <p>by {story.author.name}</p>
      <p>{story.wordCount.toLocaleString()} words • {story.rating}</p>
      <p>{story.summary}</p>
      <div class="tags">
        {#each story.tags.slice(0, 5) as tag}
          <span class="tag">{tag}</span>
        {/each}
      </div>
    </div>
  {/each}
{/if}
```

This is a complete, type-safe search implementation. Let's review what we've achieved:

1. **Every piece of data has a type.** From the search parameters to the response to the individual story objects.

2. **Discriminated unions handle success/error.** The `SearchResponse` type forces you to handle both cases.

3. **The component uses proper state types.** Every `$state` has a type, and the derived computations are type-safe.

4. **No `any` anywhere.** Every value flows through properly typed code.

5. **Errors are handled gracefully.** The component shows user-friendly messages based on the error type.

> **Watch Out:** When working with real APIs, always handle the network error case (try/catch around fetch) AND the HTTP error case (checking response.ok). Many developers handle one but not the other, leading to uncaught errors in production.

### Advanced Pattern: Type-Safe API Client

For larger projects, you might want a centralized API client that's fully type-safe:

```typescript
// src/lib/api-client.ts

type Method = "GET" | "POST" | "PUT" | "DELETE";

interface RequestOptions {
  method?: Method;
  body?: unknown;
  headers?: Record<string, string>;
}

class ApiClient {
  private baseUrl: string;
  private defaultHeaders: Record<string, string>;

  constructor(baseUrl: string, apiKey?: string) {
    this.baseUrl = baseUrl;
    this.defaultHeaders = apiKey
      ? { Authorization: `Bearer ${apiKey}` }
      : {};
  }

  async request<T>(
    endpoint: string,
    options: RequestOptions = {}
  ): Promise<T> {
    const { method = "GET", body, headers = {} } = options;

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      method,
      headers: {
        "Content-Type": "application/json",
        ...this.defaultHeaders,
        ...headers
      },
      body: body ? JSON.stringify(body) : undefined
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new ApiRequestError(
        error.message || `Request failed: ${response.status}`,
        response.status,
        error
      );
    }

    return response.json() as Promise<T>;
  }

  // Type-safe methods for specific endpoints
  async getStories(page = 1): Promise<PaginatedResponse<StoryMetadata>> {
    return this.request(`/stories?page=${page}`);
  }

  async getStory(id: string): Promise<StoryMetadata> {
    return this.request(`/stories/${id}`);
  }

  async createStory(story: CreateStoryPayload): Promise<StoryMetadata> {
    return this.request("/stories", {
      method: "POST",
      body: story
    });
  }

  async updateStory(
    id: string,
    updates: Partial<StoryMetadata>
  ): Promise<StoryMetadata> {
    return this.request(`/stories/${id}`, {
      method: "PUT",
      body: updates
    });
  }

  async deleteStory(id: string): Promise<void> {
    await this.request(`/stories/${id}`, { method: "DELETE" });
  }
}

// Custom error class with type information
class ApiRequestError extends Error {
  constructor(
    message: string,
    public status: number,
    public body: unknown
  ) {
    super(message);
    this.name = "ApiRequestError";
  }
}

// Usage
const api = new ApiClient("https://api.fichub.example.com", "your-api-key");

// All return types are fully typed!
const stories = await api.getStories(1);
const story = await api.getStory("abc-123");
const newStory = await api.createStory({
  title: "My Story",
  author: "Alice"
});
```

> **Try It Yourself:**
>
> Extend the `ApiClient` class above by adding:
> 1. A `searchStories(query: string)` method that returns `PaginatedResponse<StoryMetadata>`
> 2. An `exportStory(id: string, format: string)` method that returns `ExportResponse`
> 3. A `getRecommendations()` method that returns `PaginatedResponse<RecResult>`
>
> Make sure every method has proper type annotations and handles errors correctly.

### Summary: The Type-Safe API Pattern

Here's the complete pattern for type-safe API communication that you'll use throughout your FicHub project and beyond:

1. **Define interfaces** for every API response shape — don't guess at what the JSON looks like
2. **Use discriminated unions** for responses that can be success or error
3. **Create typed fetch functions** that return `Promise<T>`
4. **Handle nullable fields** with `string | null` and proper null checks
5. **Use generic types** for reusable patterns like pagination
6. **Validate at the boundary** with Zod or similar libraries for external APIs
7. **Never use `any`** for API data

This might feel like a lot of upfront work, but the payoff is enormous. You'll catch API-related bugs before they reach production, your editor will guide you through every API call, and refactoring becomes safe and easy.

Let me leave you with one more pattern that's incredibly useful: **type-safe event emitters**. If your app has events (like "story updated" or "export complete"), you can make them type-safe too:

```typescript
// Define all possible events and their data
interface FicHubEvents {
  "story:updated": { storyId: string; changes: string[] };
  "story:deleted": { storyId: string };
  "export:complete": { storyId: string; downloadUrl: string };
  "export:error": { storyId: string; error: string };
  "recommendation:new": { storyId: string; matchScore: number };
}

// A type-safe event emitter
class TypedEventEmitter {
  private listeners: {
    [K in keyof FicHubEvents]?: Array<(data: FicHubEvents[K]) => void>;
  } = {};

  on<K extends keyof FicHubEvents>(
    event: K,
    listener: (data: FicHubEvents[K]) => void
  ): void {
    if (!this.listeners[event]) {
      this.listeners[event] = [];
    }
    this.listeners[event]!.push(listener);
  }

  emit<K extends keyof FicHubEvents>(
    event: K,
    data: FicHubEvents[K]
  ): void {
    const eventListeners = this.listeners[event];
    if (eventListeners) {
      eventListeners.forEach(listener => listener(data));
    }
  }
}

// Usage — TypeScript knows exactly what data each event provides!
const emitter = new TypedEventEmitter();

emitter.on("story:updated", (data) => {
  // TypeScript knows: data.storyId is string, data.changes is string[]
  console.log(`Story ${data.storyId} updated: ${data.changes.join(", ")}`);
});

emitter.emit("story:updated", {
  storyId: "abc-123",
  changes: ["title", "summary"]  // ✅ Correct!
});

// ❌ This would error — wrong event name
emitter.emit("story:changed", { storyId: "abc-123" });

// ❌ This would error — missing required field
emitter.emit("story:updated", { storyId: "abc-123" });
```

> **Key Takeaways:**
> - **Start with the JSON** — inspect actual API responses before writing types
> - **Create interfaces** for every level of nesting in the response
> - **Use discriminated unions** (`{ ok: true; data: ... } | { ok: false; error: ... }`) for success/error patterns
> - **Handle nullable fields** with `string | null` and proper null checks
> - **Type your fetch functions** with `Promise<T>` return types
> - **Use generic types** (`PaginatedResponse<T>`) for reusable patterns
> - **Type guards** narrow types at runtime, making code safe and expressive
> - **No `any`** in API code — ever!

---

## Part 3 Summary

In this part of the book, you've gone from JavaScript's loose type system to TypeScript's powerful safety features:

- **Chapter 10:** You learned *why* TypeScript matters — catching type errors before they reach users, making code self-documenting, and enabling better tooling.

- **Chapter 11:** You mastered the building blocks — primitives, arrays, objects, interfaces, union types, generics, and utility types. These are the tools you'll use every day.

- **Chapter 12:** You applied TypeScript to Svelte components — typing props with `$props()`, state with `$state()`, derived values with `$derived()`, and events. You learned about SvelteKit's `$types` module and the `satisfies` operator.

- **Chapter 13:** You learned type-safe API communication — designing types from JSON, creating typed fetch functions, using discriminated unions for success/error handling, and building a complete search implementation.

TypeScript might feel like extra work at first, but it quickly becomes second nature. And the safety it provides — catching errors before your users do — is worth every type annotation.

In the next part, we'll build on this foundation to create the actual fic tracking interface, combining your knowledge of Svelte, SvelteKit, and TypeScript into a real, working application.

*Remember: every type annotation is a promise you make to your future self. Keep those promises, and your code will thank you.*
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
# Part 5: Building the UI Components

---

# Chapter 19: Global Styles and CSS Variables

## Why CSS Variables? (Paint Colors You Can Change Everywhere)

Imagine you're painting a house. You pick a beautiful blue for the front door. Then you want to paint the shutters the same blue. And the mailbox. And the window trim. So you keep going back to the hardware store, dipping your brush into the same can of paint, carrying it from room to room.

Now imagine instead you had a magic paint can. You fill it once with your chosen blue, give it a name — "Front Door Blue" — and every time you want to use that blue anywhere in your house, you just say "Front Door Blue." Want to change the whole house to green? Just pour green into the magic can once, and every surface that referenced "Front Door Blue" instantly changes.

That's CSS variables in a nutshell.

In the old days of CSS, if you wanted a specific shade of purple for your buttons, you'd write:

```css
.btn {
  background: #6366f1;
}
.btn:hover {
  background: #818cf8;
}
.nav-tab.active {
  background: #6366f1;
}
.header {
  border-bottom: 2px solid #6366f1;
}
```

That's four places with the same color. If you wanted to change your theme from purple to orange, you'd have to hunt down every single `#6366f1` and replace it. Miss one? You've got a mismatched button sticking out like a sore thumb.

CSS custom properties (also called CSS variables) solve this beautifully:

```css
:root {
  --color-primary: #6366f1;
  --color-primary-hover: #818cf8;
}

.btn {
  background: var(--color-primary);
}
.btn:hover {
  background: var(--color-primary-hover);
}
.nav-tab.active {
  background: var(--color-primary);
}
.header {
  border-bottom: 2px solid var(--color-primary);
}
```

Now every color references a single name. Want to switch to orange? Change two lines in `:root`, and the entire app updates. Every button, every tab, every accent — all of them change at once.

The variable names start with `--` (double dash). That's the CSS convention for custom properties. When you reference them with `var(--name)`, the browser looks up the value wherever it was defined. If the variable isn't defined, the browser uses a fallback or ignores the property entirely.

You can even provide fallback values inside `var()`:

```css
.btn {
  background: var(--color-primary, #6366f1);
}
```

If `--color-primary` somehow isn't defined, the button falls back to the hard-coded purple. It's like having a backup plan built into your styles.

This is exactly how FicHub's theme system works, and it's what makes the whole UI feel cohesive and consistent.

## The app.css File: Defining the Theme

In FicHub, all the global styles live in one file: `src/app.css`. It's the first stylesheet imported in the layout:

```svelte
<!-- src/routes/+layout.svelte -->
<script lang="ts">
  import '../app.css';
  // ... component imports
</script>
```

By importing it in the root layout, the styles are available everywhere in the application. Every page, every component, every widget inherits these styles.

The file has three main sections:
1. **The `:root` variable definitions** (the theme)
2. **Global reset styles** (starting fresh across browsers)
3. **Utility classes** (reusable building blocks)

Let's look at it from top to bottom.

## The Root Variables: The Complete Theme

The file starts with a `:root` block — that's the special CSS selector that applies to the entire document (every HTML element). This is where all our CSS variables live:

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

That's the whole theme. Sixteen variables that define every visual aspect of the application. Let's break them down into groups.

## Color Variables: The Palette

FicHub uses a **dark theme** by default. The background is almost black (`#0f1117`), surfaces are very dark blue-gray, and text is light. This is the opposite of most websites you visit, and it's intentional — FicHub is a reading app, and dark themes are easier on the eyes, especially at night.

Here's the color hierarchy, from background to foreground:

| Variable | Value | Purpose |
|---|---|---|
| `--color-bg` | `#0f1117` | Page background — the deepest dark |
| `--color-surface` | `#171a23` | Card backgrounds, slightly lighter |
| `--color-surface-2` | `#1f2430` | Input backgrounds, hover states, a bit lighter still |
| `--color-border` | `#2a3040` | Lines, dividers, input borders |
| `--color-text` | `#e8eaf0` | Primary text — almost white but not quite |
| `--color-muted` | `#9aa3b2` | Secondary text, descriptions, hints |

The "accent" colors sit on top:

| Variable | Value | Purpose |
|---|---|---|
| `--color-primary` | `#6366f1` | Buttons, active tabs, focus rings — a vibrant indigo |
| `--color-primary-hover` | `#818cf8` | Lighter indigo for hover states |
| `--color-success` | `#22c55e` | Positive scores, good states |
| `--color-error` | `#ef4444` | Error messages, negative scores, warnings |
| `--color-warning` | `#f59e0b` | Caution states, not used much yet |

Notice the progression: `bg` is darkest, then surfaces get lighter as they "float up," and `border` is the lightest of the "structure" colors. Text floats above everything. The accent colors (primary, success, error) are saturated and vivid — they pop against the dark backgrounds.

Why these specific values? Each hex code represents a color:
- `#0f1117` is RGB(15, 17, 23) — almost pure black with a slight blue tint
- `#6366f1` is RGB(99, 102, 241) — a bright indigo/purple
- `#22c55e` is RGB(34, 197, 94) — a bright green
- `#ef4444` is RGB(239, 68, 68) — a bright red

The dark backgrounds use very low RGB values (under 50 for each channel), while the accent colors use high values (over 100). This creates strong contrast — the accents really stand out.

> **Try It Yourself:** Open your browser's developer tools (F12), find the "Elements" or "Inspector" tab, select any element, and look at the "Computed" styles panel. You can modify CSS variables live and watch the entire page change. Try changing `--color-primary` to `#f59e0b` (amber/orange) and see what happens! The buttons, active tabs, and focus rings all change instantly.

> **Another Experiment:** Change `--color-bg` to `#ffffff` (white) and `--color-text` to `#000000` (black). The page suddenly becomes a light theme — without changing a single component file. That's the power of CSS variables.

## Border and Radius: Making Things Round

```css
--radius: 10px;
--radius-sm: 6px;
```

Two radius values keep things simple:

- `--radius` (10px) is for cards, modals, and larger containers — those need a generous curve at the corners
- `--radius-sm` (6px) is for buttons, inputs, tags, and smaller interactive elements — just a subtle rounding

The difference between 10px and 6px might seem small, but it matters. Cards are large surfaces that benefit from softer corners. Buttons and inputs need sharper corners to look crisp and clickable.

And the border color:

```css
--color-border: #2a3040;
```

This single variable controls every border in the app. Cards, inputs, buttons, tags — they all use this same slightly-lighter-than-surface color. If you ever want to make borders more visible or change their style, you change one line.

The border pattern in FicHub is always `1px solid var(--color-border)` — a thin, solid line. No dashed borders, no double borders, no thick borders. Consistency is key.

## Shadows: Depth Without Darkness

```css
--shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
```

A shadow gives elements the feeling of floating above the background. Let's break down each value:

- `0` — horizontal offset: the shadow doesn't shift left or right
- `4px` — vertical offset: the shadow falls 4 pixels downward, as if light comes from above
- `16px` — blur radius: a soft, diffuse shadow (higher = more blur)
- `rgba(0, 0, 0, 0.3)` — the shadow color: black at 30% opacity

This shadow is used by `.card`, the modal dialog, and the navigation bar. It creates a subtle layered effect that makes the dark theme feel dimensional instead of flat.

Without shadows, a dark theme can feel like everything is on the same plane — just colored rectangles sitting on a black background. The shadow creates the illusion that cards float above the background, and modals float above cards.

## Typography: Two Font Stacks

```css
--font: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
--mono: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, monospace;
```

Two font variables cover everything:

- **`--font`**: The "system font stack." This uses whatever the operating system's native font is — San Francisco on macOS, Segoe UI on Windows, Roboto on Android. The result: the app looks native on every platform without downloading any custom fonts.

- **`--mono`**: A monospace font stack for code-like content. In FicHub, this is used for bookmarklet code, URL displays in the suggestions tab, and the syntax parser page.

The system font stack is clever because of how CSS fallbacks work. The browser tries each font in order and uses the first one it finds:
1. `-apple-system` — San Francisco on macOS/iOS (not a real font name, but a browser hint)
2. `BlinkMacSystemFont` — San Francisco on newer macOS
3. `'Segoe UI'` — Windows
4. `Roboto` — Android/ChromeOS
5. `Helvetica` — Older macOS
6. `Arial` — Windows fallback
7. `sans-serif` — The browser's default sans-serif font (last resort)

> **Why system fonts?** Custom fonts need to be downloaded, which adds loading time and uses bandwidth. System fonts are already on the user's device, so they appear instantly. They also feel "right" because they match the rest of the OS. When someone opens FicHub on their Mac, the text looks like every other Mac app. On Windows, it looks like a native Windows app.

## The Max-Width Container

```css
--max-width: 820px;
```

This limits how wide the content can get. On a 2560px ultrawide monitor, you don't want text stretching across the entire screen — it becomes unreadable. Your eyes have to scan too far from left to right, and you lose your place on long lines.

820 pixels is roughly the width of a book page, perfect for reading. Research on optimal line length suggests 50-75 characters per line for comfortable reading, and 820px with the default font hits that range.

The `.container` class uses it:

```css
.container {
  max-width: var(--max-width);
  margin: 0 auto;
  padding: 1rem;
}
```

- `max-width: var(--max-width)` — limits the width to 820px
- `margin: 0 auto` — centers the container horizontally (0 top/bottom, auto left/right)
- `padding: 1rem` — adds 16px of breathing room on all sides

On a narrow screen (like a phone at 375px), the container stretches to fill the width — `max-width` only limits the maximum, not the minimum. On a wide screen, it stays at 820px and centers itself.

## Reset Styles: Starting Fresh

Before our custom styles, FicHub does a minimal CSS reset. Every browser has its own default styles for HTML elements — different margins on `<p>`, different sizes for `<h1>`, different padding on `<button>`. A reset normalizes these differences.

```css
* {
  box-sizing: border-box;
}
```

The universal `box-sizing: border-box` rule is arguably the most important CSS declaration in modern web development. Without it, when you set `width: 200px` and add `padding: 20px`, the element becomes 240px wide (the padding is added OUTSIDE the width). With `border-box`, the padding is included INSIDE the width — the element stays 200px.

This makes layout much more predictable. When you say "this element is 200 pixels wide," it's actually 200 pixels wide, regardless of padding or borders.

```css
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

- `margin: 0; padding: 0` — removes the browser's default margins on body
- `background` and `color` — use our theme variables
- `font-family` — uses our system font stack
- `line-height: 1.6` — adds comfortable spacing between lines (1.6 × the font size)
- `min-height: 100vh` — makes the body at least as tall as the viewport (100% of viewport height)

Links and buttons get their own defaults:

```css
a {
  color: var(--color-primary-hover);
  text-decoration: none;
}
a:hover {
  text-decoration: underline;
}

button {
  font-family: inherit;
  cursor: pointer;
}
```

Links use the lighter indigo (`--color-primary-hover`) instead of the darker primary. They have no underline by default, but gain one on hover — a subtle way to indicate interactivity.

Buttons inherit the font family from their parent (instead of the browser default) and show a pointer cursor on hover.

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

All form inputs share the same base styling. They use `--color-surface-2` for background (slightly lighter than the card surface), `--color-border` for the border, and `--color-primary` when focused. The `outline: none` removes the browser's default focus ring (which is usually ugly and inconsistent across browsers), and the focus border color change serves as a custom focus indicator.

> **Watch Out:** Removing the default outline with `outline: none` and replacing it with a border color change is visually nice but can be an accessibility issue. Screen readers and keyboard-only users rely on focus indicators. The border color change works, but it's less visible than the default outline. A more accessible approach would be to add a subtle box-shadow on focus: `input:focus { box-shadow: 0 0 0 2px var(--color-primary); }`.

## Utility Classes: The Reusable Building Blocks

Below the reset, FicHub defines a handful of utility classes. These are tiny, reusable style patterns that components can apply with a single class name. Think of them as the building blocks of the UI.

### The Card

```css
.card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius);
  padding: 1.25rem;
  box-shadow: var(--shadow);
}
```

This is the most-used class in the app. Every result, every error message, every recommendation uses `.card`. It gives you: a slightly-lighter-than-background color, a subtle border, rounded corners, padding, and a shadow. One class, instant visual consistency.

A card is the fundamental unit of content in FicHub. When you see a result in the Download tab, that's a `.card`. When you see a recommendation, that's a `.card`. When an error appears, that's a `.card` with a red border.

The consistency matters: users learn to recognize cards as "containers of related information." No matter which tab they're on, they know that a card contains something meaningful.

### The Button

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

The button has four states:

1. **Normal** — purple background (`--color-primary`), white text, no outline
2. **Hover** — lighter purple (`--color-primary-hover`), with a smooth 0.15-second transition
3. **Active (pressed)** — shifts down 1 pixel (`translateY(1px)`) for a tactile "press" effect, with a quick 0.05-second transition
4. **Disabled** — 50% opacity, no-pointer cursor

The `display: inline-flex` with `align-items: center` and `gap: 0.4rem` means that if a button contains an icon and text (like the spinner and "Working…"), they're horizontally centered with a small gap between them.

There's also a secondary button variant:

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

This is for less-important actions — download links in the results, the "Suggest a fic" button, cancel buttons in modals. It's visually quieter than the primary button: it uses the surface-2 background instead of the vibrant primary purple, and it has a border to give it structure.

### The Tag

```css
.tag {
  display: inline-block;
  background: var(--color-surface-2);
  border: 1px solid var(--color-border);
  border-radius: 999px;
  padding: 0.15rem 0.7rem;
  font-size: 0.8rem;
  color: var(--color-muted);
}
```

Tags use `border-radius: 999px` — that's the "pill" trick. Any value above 50% on a rectangle makes the corners fully rounded, creating a capsule shape. You don't need to calculate the exact radius — just use a huge number and the browser does the rest.

Tags display site names ("AO3", "FanFiction.net") and other metadata. They're small, pill-shaped badges that provide context without dominating the layout.

### Text Helpers

```css
.muted {
  color: var(--color-muted);
}

.error-text {
  color: var(--color-error);
}
.success-text {
  color: var(--color-success);
}
```

Three text color helpers. `.muted` dims text for secondary information — author names, descriptions, hints. `.error-text` makes text red for errors. `.error-text` is the most commonly used of the three, appearing in every error state across all tabs.

### Text Helper Deep Dive

The `.muted` class is deceptively important. Without it, every piece of secondary text would need `color: var(--color-muted)` inline. With it, you just add `class="muted"` and the text dims appropriately.

Look at how it's used in the Download tab:

```svelte
<p class="muted">by {m.author} · <span class="tag">{detectSite(m.source)}</span></p>
<p class="desc">{stripHtml(m.description).slice(0, 300)}</p>
<p class="muted hint">Tip: drag this to your bookmarks bar...</p>
```

Three different uses of `.muted`, three different contexts, one consistent visual treatment. The author line is muted because it's less important than the title. The description is muted (via `.desc` which also sets muted color) because it's supplementary. The hint is muted because it's a nice-to-know, not critical information.

## The Spinner Animation: Loading Indicator

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

This is one of the most elegant little CSS tricks. Here's how it works, step by step:

1. The spinner is a 1rem × 1rem circle (`border-radius: 50%`)
2. It has a 2px border that's mostly transparent white (`rgba(255, 255, 255, 0.3)`)
3. But the top portion of the border (`border-top-color`) is solid white
4. The `@keyframes spin` animation rotates the entire element from its current position to 360° (a full revolution)
5. At 0.7 seconds per revolution with `linear` timing, it spins at a steady, pleasant pace
6. `infinite` means it never stops spinning

The result: a classic loading spinner — a circle with one bright arc that chases itself around. You see it everywhere in the Download tab ("Working…"), Recommendations tab ("Searching…"), and Suggestions tab ("Loading…").

The key insight is that only the top border is opaque. As the element rotates, that opaque arc appears to move around the circle. The other three borders are semi-transparent, creating a "fading trail" effect.

> **Watch Out:** The spinner color is hard-coded to white (`rgba(255, 255, 255, ...)`) because it always appears inside a purple `.btn` where white text is already set. If you put a spinner outside a button — say, on a card background — you'd need to change the colors to match. A more flexible approach would be to use CSS variables for the spinner colors, but since it's only used inside buttons, hard-coding works fine.

## Responsive Design: Adapting to Screen Sizes

At the bottom of `app.css`, there's no global media query, but every component that needs responsive behavior includes its own. Here's the pattern used in DownloadTab, RecommendationsTab, and SuggestionsTab:

```css
@media (max-width: 600px) {
  .search-row {
    flex-direction: column;
  }
}
```

On screens narrower than 600 pixels (roughly mobile phones), the search row switches from a horizontal layout (input + button side by side) to a vertical stack (input on top, button below). This gives the input full width on small screens, making it easier to type long URLs.

The layout itself has its own responsive breakpoint:

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

On mobile, three things happen:
1. The tab labels ("Download", "Recommendations", "Suggestions") hide, leaving just the icons (⬇, ★, 💡)
2. The search area stretches full width
3. The search input flexes to fill available space

This maximizes the content area on small screens while keeping the navigation functional. Users can still tap the icons to switch tabs, and the search input has plenty of room.

> **Try It Yourself:** Resize your browser window while watching the FicHub app. Around 700px, the tab labels disappear. Around 600px, the search rows stack vertically. This is responsive design in action — the same code adapts to any screen size.

> **Why 600px and 700px?** These aren't arbitrary numbers. 600px is approximately the width of a large phone in portrait mode. 700px gives a little extra room for the navigation to collapse before the content area gets too narrow. Different apps use different breakpoints — there's no "correct" number, just what works for your layout.

## Dark Mode: The Default Theme

Here's something interesting: FicHub doesn't have a light mode. The dark theme IS the theme. There's no `prefers-color-scheme` media query, no theme toggle button, no JavaScript to switch themes.

Why? Because FicHub is a fanfiction reading app, and readers often use it at night. Dark themes are standard for reading apps, code editors, and media players. The colors are carefully chosen to be comfortable for extended reading.

The specific shades matter. The background isn't pure black (`#000000`) — it's a very dark blue-gray (`#0f1117`). Pure black with white text creates too much contrast, which causes eye strain during long reading sessions. The slight color in the background reduces this strain.

If you ever wanted to add a light mode, you'd create a second set of variables:

```css
[data-theme="light"] {
  --color-bg: #ffffff;
  --color-surface: #f5f5f5;
  --color-surface-2: #e8e8e8;
  --color-border: #d0d0d0;
  --color-text: #1a1a1a;
  --color-muted: #666666;
  --color-primary: #6366f1;
  --color-primary-hover: #818cf8;
  --shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
  /* etc. */
}
```

Then you'd toggle the `data-theme` attribute on the root element, and everything would update automatically because all colors reference CSS variables. That's the beauty of the variable system.

## How CSS Variables Cascade

One subtle but powerful feature of CSS custom properties is that they cascade — meaning child elements inherit variables from their parents. If you define `--color-primary` on `:root`, every element in the document can use it.

But you can also override variables on specific elements:

```css
.card {
  --color-primary: #ef4444;  /* Red buttons inside cards */
}
```

Now every `.btn` inside a `.card` would be red instead of purple. This is useful for creating context-specific themes without modifying individual elements.

FicHub doesn't use this feature much (it keeps things simple), but it's good to know it exists. For example, you could create an "admin mode" by setting different variables on a wrapper element:

```css
.admin-panel {
  --color-primary: #f59e0b;
  --color-error: #dc2626;
}
```

Every component inside `.admin-panel` automatically uses the new colors.

## Putting It All Together

Let's trace one element — the download button — through the entire CSS system:

1. **The button gets the `.btn` class** in DownloadTab.svelte
2. **`.btn` uses `--color-primary`** (`#6366f1`) for its background
3. **On hover, it switches to** `--color-primary-hover` (`#818cf8`)
4. **The border radius comes from** `--radius-sm` (`6px`)
5. **When loading, it shows a `.spinner`** that animates with `@keyframes spin`
6. **When disabled, it fades** with `opacity: 0.5`
7. **On mobile (under 600px)**, it stacks below the input
8. **Inside the button**, the spinner uses white text color (inherited from `color: white` on `.btn`)

Six different aspects of styling, all controlled by variables defined in one place. That's the power of CSS custom properties.
## CSS Transitions: Making Changes Smooth

Beyond variables, FicHub uses CSS transitions to make state changes feel smooth rather than jarring. Look at the button's transition property:

```css
.btn {
  transition: background 0.15s, transform 0.05s;
}
```

This tells the browser: when the `background` property changes (like on hover), animate the change over 0.15 seconds. When `transform` changes (on click), animate over 0.05 seconds.

Without transitions, hovering over a button would instantly snap from purple to lighter purple. With transitions, the color fades smoothly over 150 milliseconds. It's a subtle difference that makes the UI feel polished.

The timing matters:

- **0.15s for background** — fast enough to feel responsive, slow enough to be noticeable
- **0.05s for transform** — nearly instant, so the "press" effect feels tactile

If you made the background transition 0.5 seconds, it would feel sluggish — the color change would be too slow. If you made it 0.01 seconds, it would be imperceptible — essentially the same as no transition.

The `:active` state uses `transform: translateY(1px)` — the button shifts down by 1 pixel. Combined with the 0.05s transition, this creates a satisfying "press" effect that mimics a physical button being pushed down.

> **Try It Yourself:** Open the browser dev tools, select a `.btn` element, and add `transition: none;` to its styles. Now hover and click the button — the changes are instant and jarring. Remove the `transition: none;` line and the smoothness returns. This comparison makes the impact of transitions crystal clear.

## The Box Model and Why border-box Matters

Remember the universal `box-sizing: border-box` from the reset styles? Let's see why it matters with a concrete example.

Consider this CSS:

```css
.card {
  width: 300px;
  padding: 1.25rem;
  border: 1px solid var(--color-border);
}
```

**Without `border-box`:** The card would be 300px + 2 × 20px padding + 2 × 1px border = 342px wide. The padding and border are added OUTSIDE the width. This is the default browser behavior.

**With `border-box`:** The card is exactly 300px wide. The padding and border are included INSIDE the width. The content area shrinks to accommodate them.

`border-box` makes layout predictable. When you say "this element is 300 pixels wide," it's actually 300 pixels wide. Without it, you'd need to do mental arithmetic for every element: "I want 300px content, so I need to set width to 300 - 40 - 2 = 258px."

This is why virtually every modern CSS reset starts with `* { box-sizing: border-box; }`. It eliminates an entire category of layout bugs.


## Practice: Customize the Theme Colors

Here's a fun exercise to test your understanding:

1. Open `src/app.css` and change `--color-primary` to `#f59e0b` (amber/gold)
2. Change `--color-primary-hover` to `#fbbf24` (lighter amber)
3. Change `--color-error` to `#ec4899` (pink instead of red)
4. Change `--radius` to `16px` (more rounded cards)
5. Change `--shadow` to `0 8px 32px rgba(0, 0, 0, 0.5)` (deeper shadows)
6. Change `--max-width` to `960px` (wider content area)

Save the file and refresh the browser. The entire app should now have an amber theme with pink error messages, rounder cards, deeper shadows, and a wider layout — all from changing six lines.

This is why CSS variables matter. You've just given FicHub a complete visual redesign without touching a single component file. Every button, every card, every input updated automatically.
## Understanding CSS Specificity and Variable Scopes

Before we move on, there's one more important CSS concept at play: specificity. When multiple CSS rules try to style the same element, the browser decides which one wins based on specificity.

Here's the hierarchy, from lowest to highest:

1. **Element selectors** (`h2`, `p`, `button`) — lowest specificity
2. **Class selectors** (`.card`, `.btn`, `.muted`) — medium specificity
3. **ID selectors** (`#header`, `#main`) — high specificity
4. **Inline styles** (`style="..."`) — highest specificity (avoid these!)

FicHub mostly uses class selectors, which gives a good balance. The utility classes in `app.css` have low specificity, so they're easy to override in component `<style>` blocks when needed.

CSS variables follow their own rules. A variable defined on `:root` can be overridden by any descendant element:

```css
/* Global: all buttons are purple */
:root { --color-primary: #6366f1; }

/* Override: buttons inside error cards are red */
.error-card .btn { --color-primary: #ef4444; }
```

This is called "variable scoping" and it's incredibly powerful. You can create context-specific themes without changing a single class name.

## How the Layout Component Ties Everything Together
## CSS Units: rem, px, and Why They Matter

FicHub uses two main CSS units: `rem` and `px`. Understanding the difference helps you read and modify the styles.

**`px` (pixels)** are absolute units. `10px` is always 10 pixels, regardless of the parent element's font size. FicHub uses pixels for:

- Border widths (`border: 1px solid ...`)
- Shadow offsets (`0 4px 16px`)
- Border radius (`border-radius: 10px`)
- Media query breakpoints (`max-width: 600px`)

**`rem` (root em)** are relative to the root font size (usually 16px). `1rem` = 16px, `0.5rem` = 8px, `1.5rem` = 24px. FicHub uses rem for:

- Padding (`padding: 1.25rem`)
- Margins (`margin: 0.4rem 0`)
- Gap spacing (`gap: 0.6rem`)
- Font sizes (`font-size: 0.95rem`)

**Why this distinction matters:** Pixels are precise but don't scale. Rems scale with the user's font size preference — if someone sets their browser to 20px base font, all `rem` values scale up proportionally, making the layout accessible for users with visual impairments. FicHub uses rem for spacing and sizing (which should scale) but pixels for borders and shadows (which should stay consistent).

> **Try It Yourself:** In the browser dev tools, change the root font size on the `<html>` element: `html { font-size: 20px; }`. Watch how the padding and spacing in the app all scale up, while the borders and shadows stay the same. This is the rem scaling effect in action.

The `+layout.svelte` file is the glue that connects the global styles to the components:
The `+layout.svelte` file is the glue that connects the global styles to the components:

```svelte
<script lang="ts">
  import '../app.css';
  import DownloadTab from '$lib/components/DownloadTab.svelte';
  import RecommendationsTab from '$lib/components/RecommendationsTab.svelte';
  import SuggestionsTab from '$lib/components/SuggestionsTab.svelte';
  // ...
</script>
```

By importing `app.css` in the layout, every component inherits the global styles. The layout also defines the top navigation bar, the tab switching, and the footer. Each component renders inside `<main class="container">`, which applies the max-width constraint.

The layout's CSS handles the sticky top bar:

```css
.topbar {
  position: sticky;
  top: 0;
  z-index: 10;
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
}
```

`position: sticky` keeps the top bar visible when scrolling. `z-index: 10` ensures it stays above page content. The background matches the card surface color, and a bottom border separates it from the content below.

The tab buttons use the `.tab` class with conditional active styling:

```css
.tab.active {
  color: white;
  background: var(--color-primary);
}
```

When a tab is active, it gets the primary purple background with white text. Inactive tabs are transparent with muted text. This creates a clear visual distinction between the selected and unselected tabs.

On mobile (under 700px), the tab labels hide and only icons remain:

```css
@media (max-width: 700px) {
  .tab-label {
    display: none;
  }
}
```

This is a common responsive pattern called "icon-only navigation" — it saves horizontal space on small screens while keeping the navigation functional through recognizable icons.


---

# Chapter 20: Utility Functions

## What Are Utility Functions? (Tools in a Toolbox)

Picture a woodworker's toolbox. Inside, you've got a measuring tape, a level, a pencil, some sandpaper, a screwdriver. Each tool does one thing well. The measuring tape doesn't hammer nails. The level doesn't saw wood. But every project uses several of them.

Utility functions are the tools in your code toolbox. They're small, focused functions that do one specific job: format a number, detect a website, clean up some text. They don't belong to any particular feature — they're shared helpers that any component might need.

Why "utility"? Because they're useful for many things. A "format words" function isn't specific to the Download tab — it's useful anywhere you display a word count. A "strip HTML" function isn't specific to recommendations — it's useful anywhere you display summaries.

In FicHub, all utility functions live in one file: `src/lib/util.ts`. The file starts with a simple comment:

```typescript
// Small formatting helpers shared across components.
```

That tells you everything: these are helpers, they're small, and they're shared. Let's look at every single one.

## formatWords(): Making Numbers Readable

```typescript
/** Format a word count into a human string (1234567 -> "1,234,567"). */
export function formatWords(words: number): string {
  return words.toLocaleString('en-US');
}
```

This is the simplest utility in the file. It takes a number like `1234567` and returns the string `"1,234,567"` with commas. Without this function, the app would show word counts like "1234567" which is hard to read at a glance.

JavaScript's built-in `toLocaleString()` method handles all the heavy lifting. The `'en-US'` argument tells it to use American English formatting (commas as thousand separators, periods as decimal points). Different locales use different separators — in Germany, it would be `"1.234.567"`, and in India, it would be `"12,34,567"`.

The function signature is precise:
- It takes a `number` (not a string — it expects raw numeric data)
- It returns a `string` (formatted for display)

This type precision matters in TypeScript. If someone accidentally passes a string, the compiler catches the error before the code even runs.

This function is used in both the Download tab and the Recommendations tab:

```svelte
{formatWords(m.words)} words · {m.chapters} chapters
```

The output: "1,234,567 words · 42 chapters" — clear and readable.

> **Try It Yourself:** What happens with edge cases? Try `formatWords(0)` → `"0"`. Try `formatWords(999)` → `"999"` (no comma needed). Try `formatWords(1000000)` → `"1,000,000"`. The function handles all these correctly because `toLocaleString()` is battle-tested.

## formatDate(): ISO Strings to Friendly Dates

```typescript
/** Format an ISO timestamp into a short date (2024-01-15). */
export function formatDate(iso: string): string {
  if (!iso) return '';
  const d = new Date(iso);
  if (isNaN(d.getTime())) return '';
  return d.toISOString().slice(0, 10);
}
```

The backend returns dates as ISO 8601 strings like `"2024-01-15T08:30:00Z"`. Nobody wants to read that. `formatDate()` converts it to `"2024-01-15"` — just the date part, no time.

Here's the step-by-step:

1. **Guard clause:** If `iso` is empty (or falsy), return an empty string. Never crash on missing data. This is defensive programming — always expect the unexpected.

2. **Parse the string:** `new Date(iso)` creates a JavaScript Date object from the ISO string. JavaScript is surprisingly good at parsing ISO 8601 dates.

3. **Validate:** `isNaN(d.getTime())` checks if the date was valid. If someone passes `"not a date"`, the Date object becomes invalid, and `getTime()` returns `NaN`. We return empty rather than showing garbage. Without this check, `d.toISOString()` would throw an error.

4. **Format:** `d.toISOString()` converts back to ISO format (`"2024-01-15T08:30:00.000Z"`), then `.slice(0, 10)` grabs just the first 10 characters: `"2024-01-15"`.

> **Watch Out:** The `slice(0, 10)` trick works because ISO dates always start with `YYYY-MM-DD` in the first 10 characters. This is a deliberate property of the ISO 8601 format. If you're working with a different date format (like `"01/15/2024"`), this approach wouldn't work.

**Why not use `toLocaleDateString()`?** You could, but it produces locale-dependent output. On an American machine, it might give you "1/15/2024". On a European machine, "15/1/2024". The `slice(0, 10)` approach always gives the same `YYYY-MM-DD` format, which is consistent and sortable.

## relativeTime(): How Long Ago?

```typescript
/** Relative time from an ISO timestamp (e.g. "3 days ago"). */
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

This function converts timestamps like `"2024-01-12T08:00:00Z"` into human-friendly text like `"3 days ago"`.

The logic cascades through time units:

1. Calculate the difference in milliseconds: `Date.now() - then`
2. Convert to seconds, minutes, hours, and days
3. Return the most appropriate unit:
   - Under 60 seconds → "less than a minute ago"
   - Under 60 minutes → "X minutes ago"
   - Under 24 hours → "X hours ago"
   - 24+ hours → "X days ago"

Notice the pluralization trick: `${min > 1 ? 's' : ''}`. If the number is 1, it shows "1 minute ago" (no s). If it's 2+, it shows "2 minutes ago" (with s). This is called conditional pluralization and it's a common pattern in user-facing text.

`Math.floor()` rounds down to the nearest whole number. If 2.7 hours have passed, we say "2 hours ago" — not "2.7 hours ago" — because rounding down is more natural in casual speech.

This function doesn't handle weeks, months, or years — it just shows "X days ago" even if it's been 300 days. That's fine for FicHub because fic update times are usually within days, not months. If you needed month-level precision, you'd add more branches:

```typescript
const month = Math.floor(day / 30);
if (month < 12) return `${month} month${month > 1 ? 's' : ''} ago`;
const year = Math.floor(day / 365);
return `${year} year${year > 1 ? 's' : ''} ago`;
```

## detectSite(): Identifying the Source

```typescript
/** Detect the fanfiction site from a URL. */
export function detectSite(url: string): string {
  if (url.includes('archiveofourown.org')) return 'AO3';
  if (url.includes('fanfiction.net')) return 'FanFiction.net';
  if (url.includes('fictionpress.com')) return 'FictionPress';
  if (url.includes('xenforo') || url.includes('spacebattles') || url.includes('sufficientvelocity'))
    return 'Forum';
  return 'Unknown';
}
```

Simple pattern matching: look at the URL and return a friendly name. This is used in the Download tab to show which site a fic comes from:

```svelte
<span class="tag">{detectSite(m.source)}</span>
```

The output: a little pill-shaped tag saying "AO3" or "FanFiction.net" next to the author name.

The function uses `includes()` rather than exact matching. It checks if the URL *contains* the domain, not if it *equals* it. This handles variations like:
- `https://archiveofourown.org/works/12345` ✓ (contains 'archiveofourown.org')
- `http://www.fanfiction.net/s/12345/1/` ✓ (contains 'fanfiction.net')

Note the "Forum" category — SpaceBattles and SufficientVelocity both run on XenForo forum software, so they're grouped together. The `xenforo` check catches any XenForo-based forum, not just these two specific ones. If a URL doesn't match any known pattern, it returns "Unknown" rather than crashing.

> **Try It Yourself:** What URL would return "Unknown"? Try `detectSite("https://www.wattpad.com/stories/harrypotter")` — it would return "Unknown" because Wattpad isn't in the detection list. You could add Wattpad support by adding another condition.

## stripHtml(): Cleaning Up Summaries

```typescript
/** Strip HTML tags from a string (for summaries). */
export function stripHtml(html: string): string {
  if (!html) return '';
  return html
    .replace(/<br\s*\/?>(?=)/gi, ' ')
    .replace(/<\/(p|div)>/gi, ' ')
    .replace(/<[^>]+>/g, '')
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/\s+/g, ' ')
    .trim();
}
```

This is the most complex utility in the file. Fanfiction summaries come wrapped in HTML from the source sites. AO3 summaries might look like:

```html
<p>Harry Potter didn't know what to expect from Hogwarts. <br/>
What he got was <em>so much more</em> than he bargained for.</p>
```

If we just displayed that raw text, users would see angle brackets and tags mixed in with the story. `stripHtml()` cleans it up step by step:

1. **`/<br\s*\/?>/gi` → `' '`**: Replace `<br>`, `<br/>`, and `<br />` tags with spaces. Line breaks in HTML become spaces in plain text. The `gi` flags mean "global" (replace all occurrences) and "case-insensitive" (match `<BR>` too).

2. **`/<\/(p|div)>/gi` → `' '`**: Replace closing `</p>` and `</div>` tags with spaces. Paragraph endings become spaces. We don't replace opening tags here because they're handled by the next step.

3. **`/<[^>]+>/g` → `''`**: Remove ALL remaining HTML tags. The regex `<[^>]+>` matches any text between angle brackets (anything that starts with `<` and ends with `>`). This catches `<em>`, `<strong>`, `<a>`, `<span>`, and any other tags.

4. **HTML entity decoding**: The next five replacements convert HTML entities back to their character equivalents:
   - `&amp;` → `&`
   - `&lt;` → `<`
   - `&gt;` → `>`
   - `&quot;` → `"`
   - `&#39;` → `'`

5. **`/\s+/g` → `' '`**: Collapse multiple spaces into one. After removing tags, you might end up with several consecutive spaces. This normalizes them to a single space.

6. **`.trim()`**: Remove leading and trailing whitespace.

The result for our example: "Harry Potter didn't know what to expect from Hogwarts. What he got was so much more than he bargained for."

Clean, readable, no angle brackets.

> **Watch Out:** This is a simplified HTML stripper — it doesn't handle every edge case in HTML. For example, it wouldn't handle `<style>` blocks, `<script>` tags, or nested quotes inside attributes. For a production content management system, you'd want a proper HTML parser like `DOMParser` or a library like `cheerio`. But for cleaning up fic summaries, which follow a consistent format from the scraper, this simple approach works well.

## cacheUrl(): Building Download Links

```typescript
/** Build a download URL that goes through the backend cache route. */
export function cacheUrl(etype: string, urlId: string, hash: string): string {
  return `/cache/${etype}/${urlId}?h=${hash}`;
}
```

This builds URLs like `/cache/epub/abc123?h=def456`. The backend caches generated files (EPUBs, HTML, etc.), and this function constructs the URL to retrieve them from cache.

Three parameters:
- `etype`: the file type (`"epub"`, `"html"`, `"mobi"`, `"pdf"`)
- `urlId`: the unique identifier for the fic
- `hash`: a cache-busting hash that ensures fresh content

The URL structure follows a REST-like pattern: `/cache/{type}/{id}?h={hash}`. This is a common web pattern — the path identifies the resource, and the query parameter controls behavior (in this case, cache validation).

The template literal syntax (`/cache/${etype}/${urlId}?h=${hash}`) inserts the variable values directly into the string. It's cleaner than string concatenation (`'/cache/' + etype + '/' + urlId + '?h=' + hash`).

## Why Keep Utils Separate

You might wonder: why not just inline `formatWords()` inside the Download tab where it's used? A few compelling reasons:

1. **Reusability.** `formatWords()` is used in both DownloadTab and RecommendationsTab. Without a shared utility, you'd copy the function into both files — code duplication. If the formatting needs to change, you'd have to update both copies.

2. **Testability.** It's easy to write tests for standalone functions. You can call `formatWords(1234567)` and assert it returns `"1,234,567"` without setting up any UI or component state. No rendering, no DOM, no mocking — just a function call and an assertion.

3. **Discoverability.** When a new developer joins the project, they know exactly where to find formatting helpers: `src/lib/util.ts`. They don't have to dig through component files to find a function that formats dates.

4. **Maintainability.** If the formatting needs to change (say, adding a locale option), you change it in one place. Every component that uses the function automatically gets the update.

5. **Clarity of intent.** A function in `util.ts` says "I'm a helper, use me anywhere." A function inside a component says "I'm specific to this component." The location communicates the scope.

The `src/lib/` directory in SvelteKit is specifically designed for this kind of shared code. It's not a page, not a component, not a route — it's a library of shared utilities, types, and API clients. The `$lib` alias in imports (`import { formatWords } from '$lib/util'`) makes it easy to reference from anywhere.

## Practice: Add a New Utility Function

Here's an exercise to sharpen your skills. Add a function called `truncateWords()` that takes a string and a maximum word count, and returns the string with only that many words (plus "…" if truncated).

Think about it first:
- What should happen with an empty string?
- What if the text has fewer words than the max?
- How do you split a string into words?
- How do you join them back?

Here's a solution:

```typescript
export function truncateWords(text: string, maxWords: number): string {
  const words = text.split(/\s+/);
  if (words.length <= maxWords) return text;
  return words.slice(0, maxWords).join(' ') + '…';
}
```

Test it:
- `truncateWords("The quick brown fox jumps", 3)` → `"The quick brown…"`
- `truncateWords("Hello", 10)` → `"Hello"` (no truncation needed)
- `truncateWords("", 5)` → `""` (empty input)
- `truncateWords("a b c d e", 0)` → `"…"` (zero words allowed)

Try it yourself before looking at the solution!
## How Utilities Compose Together

The real power of utilities comes when you combine them. Look at how the Download tab uses three utilities together:

```svelte
<p class="muted">by {m.author} · <span class="tag">{detectSite(m.source)}</span></p>
<p class="meta-line">
  {formatWords(m.words)} words · {m.chapters} chapters · status: {m.status}
</p>
{#if m.description}
  <p class="desc">{stripHtml(m.description).slice(0, 300)}</p>
{/if}
```

Three different utilities, each doing its own job:

- `detectSite(m.source)` → turns a URL into "AO3"
- `formatWords(m.words)` → turns `1234567` into `"1,234,567"`
- `stripHtml(m.description)` → turns `<p>Hello</p>` into `"Hello"`

None of these functions know about each other. They don't import each other or share state. They're completely independent tools that happen to be used in the same component. This independence is what makes utilities so valuable — you can mix and match them freely.

The `.slice(0, 300)` after `stripHtml()` is a separate concern: truncation. The utility cleans the HTML, then the template truncates the result. Each step handles one transformation.

## Error Handling Patterns in Utilities

Every utility function in FicHub follows the same defensive pattern: check for empty or invalid input at the top, return early with a safe default. This is called "guard clauses."

```typescript
// formatWords doesn't guard because Number is always a number
// (TypeScript enforces this at compile time)

// formatDate guards against empty strings and invalid dates
if (!iso) return '';
const d = new Date(iso);
if (isNaN(d.getTime())) return '';

// relativeTime also guards
if (!iso) return '';
const then = new Date(iso).getTime();
if (isNaN(then)) return '';

// stripHtml guards against empty/null input
if (!html) return '';
```

The pattern is consistent:
1. Check if the input is empty → return a safe default (empty string)
2. Try to parse the input → check if parsing succeeded
3. Return the formatted result

This pattern prevents crashes. Without the guard clauses, passing an empty string to `new Date('')` would create an Invalid Date, and subsequent operations would produce `NaN` or throw errors. The guards catch these cases early.

> **Watch Out:** Guard clauses make functions safe but they can also hide bugs. If `formatDate(undefined)` silently returns an empty string, you might not realize you're passing undefined where you shouldn't be. TypeScript helps here — it would warn you if you try to pass `undefined` to a parameter typed as `string`. But if you use `any` types, the guards are your last line of defense.


---

# Chapter 21: The Download Tab

## What the Download Tab Does (The Main Feature)

The Download tab is FicHub's front door. It's the first thing users see when they open the app, and it does one thing brilliantly: let you paste a fanfiction URL and get back downloadable files.

Think of it like a vending machine. You put in a URL (the money), press the button, and out comes an EPUB, HTML file, MOBI, or PDF — whatever format you want to read on your device.

The whole interaction takes three steps:

1. **Paste a URL** into the input field
2. **Click "Download"** (or press Enter)
3. **Get results** with links to download files

The component is 181 lines long, including HTML, TypeScript, and CSS. It's a complete, self-contained feature packed into one file.

## Component Structure Overview

Every Svelte component has three sections:

1. **`<script>`** — JavaScript/TypeScript logic
2. **Template** — HTML with Svelte syntax
3. **`<style>`** — Scoped CSS

The Download tab follows this structure exactly. Let's look at each section.

### Imports

```typescript
import { fetchExport, ApiError } from '$lib/api/client';
import type { ExportResponse } from '$lib/api/types';
import { formatWords, detectSite, stripHtml } from '$lib/util';
```

Three import sources:
- `$lib/api/client` — the API function (`fetchExport`) and error class (`ApiError`)
- `$lib/api/types` — TypeScript type definitions (`ExportResponse`)
- `$lib/util` — the utility functions we learned about in Chapter 20

Notice `export` and `ApiError` are value imports (they'll be used in code), while `ExportResponse` is a type-only import (`import type`). TypeScript erases type imports at compile time — they don't appear in the final JavaScript. This is a performance optimization.

### State Variables

```typescript
let url = $state('');
let loading = $state(false);
let error = $state('');
let result = $state<ExportResponse | null>(null);
```

Four pieces of state:

- **`url`** — the text the user types in the input field. Starts empty.
- **`loading`** — whether a request is in progress. Starts false.
- **`error`** — an error message to display. Starts empty (no error).
- **`result`** — the API response. Starts null (no result yet).

The `$state()` function is Svelte 5's way of declaring reactive state. When any of these values change, the UI automatically updates. No manual DOM manipulation needed.

`ExportResponse | null` means the variable can hold either an `ExportResponse` object or `null`. The `null` case represents "no result yet." TypeScript enforces that we check for null before accessing properties.

## The Search Row: Input + Button

Every tab in FicHub starts with the same pattern: a search row. This is a horizontal bar with an input field and a button side by side.

Here's the HTML for the Download tab's search row:

```svelte
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

Let's break down each attribute:

- **`type="url"`**: Tells the browser this is a URL field. On mobile phones, this changes the keyboard to include `.com` and `/` keys. It also enables built-in URL validation — the browser won't submit the form if the text isn't a valid URL format.

- **`placeholder`**: The gray text that shows when the input is empty. It's a hint telling users what to type. The ellipsis (…) in the placeholder suggests there are more options than listed.

- **`bind:value={url}`**: This is Svelte's two-way binding. When the user types, `url` updates. When `url` changes in code, the input updates. They're always in sync. This eliminates the need for manual event handlers like `oninput`.

- **`onkeydown={onKeydown}`**: Handles keyboard events. We'll look at this next.

- **`disabled={loading}`**: When the request is in progress, the input becomes uneditable. You can't paste a new URL while waiting for results — that would be confusing and could trigger duplicate requests.

- **`aria-label`**: An accessibility attribute that screen readers read aloud. It says "Fanfiction URL" even though the visible text is a placeholder that disappears when typing. Without this, screen reader users would hear the input's content but not know what kind of input it is.

## Handling Enter Key: onkeydown

```typescript
function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') handleDownload();
}
```

This is a small but important piece of UX. When a user pastes a URL, their natural instinct is to press Enter — not reach for the mouse to click the button. Without this handler, pressing Enter would do nothing (or in some browsers, submit a form and reload the page).

The handler checks if the pressed key is "Enter" and calls `handleDownload()` just like clicking the button would. This pattern is identical in all three tabs.

The `KeyboardEvent` type tells TypeScript what properties are available: `key`, `code`, `shiftKey`, `ctrlKey`, etc. The `.key` property gives the logical key name — "Enter" for the Enter key, "a" for the A key, etc.

> **Try It Yourself:** Type a URL into the Download tab input and press Enter instead of clicking the button. It works! Now try it in the Recommendations and Suggestions tabs — same behavior everywhere. This consistency is intentional: every tab uses the exact same pattern.

> **Watch Out:** What if you wanted to use Shift+Enter for a different action (like inserting a newline)? The current handler doesn't check modifier keys. If you wanted that, you'd add: `if (e.key === 'Enter' && !e.shiftKey) handleDownload();`.

## Loading State: Spinner + Disabled Button

When the user clicks Download, two things happen immediately:

```typescript
loading = true;
error = '';
result = null;
```

1. `loading` becomes `true` — this disables the input and button, and shows the spinner
2. `error` is cleared — any previous error message disappears
3. `result` is cleared — the old results disappear while loading new ones

The button text changes dynamically:

```svelte
{#if loading}
  <span class="spinner"></span> Working…
{:else}
  Download
{/if}
```

This is Svelte's conditional rendering. The `{#if ... :else}` block shows one of two options:

- While loading: a `<span class="spinner"></span>` (the spinning animation) followed by "Working…"
- Otherwise: just "Download"

The `class:spinner` on the span triggers the CSS animation we saw in Chapter 19. The spinner is an inline element (because `.spinner` has `display: inline-block`), so it sits next to the text rather than on its own line.

Both the input and button get `disabled={loading}`. This prevents:
- Typing new URLs while waiting
- Clicking the button multiple times (which would send duplicate requests)

> **Watch Out:** Notice that `result = null` is set when loading starts. This means old results disappear immediately when you click Download again. Some apps keep showing old results until new ones arrive (so the screen isn't blank during loading), but FicHub clears them for clarity — you always know whether you're looking at current or stale data.

## The fetchExport() Call

The heart of the Download tab — the function that actually does the work:

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

Let's trace the flow:

1. **Validate input:** If the URL is empty (or just whitespace), show an error and return early. `url.trim()` removes leading/trailing spaces — a URL like `"  https://ao3.org/works/123  "` becomes `"https://ao3.org/works/123"`.

2. **Set loading state:** Disable UI, clear old data. These three assignments happen synchronously — the UI updates before the API call starts.

3. **Call the API:** `fetchExport(url.trim())` sends a GET request to `/api/v0/epub?q=<url>`. The `await` keyword pauses the function until the response arrives. The backend fetches the fic, generates the export files, and returns download URLs.

4. **Check for errors:** The backend returns `{ err: 0 }` for success and `{ err: 1, msg: "..." }` for errors. If `err !== 0`, we show the error message. The `||` operator provides a fallback message if `res.msg` is undefined.

5. **Handle exceptions:** Network errors, server crashes, timeouts — they all throw JavaScript errors. We catch them and show user-friendly messages. `ApiError` is a custom error class from the API client that includes the HTTP status code. `instanceof ApiError` checks if the error is an API error (which has a status code) or a network error (which doesn't).

6. **Finally:** `loading = false` runs whether the request succeeded or failed. The `finally` block ensures the spinner always stops. Without it, a network error would leave the UI stuck in a loading state.

The `try/catch/finally` pattern is fundamental to async JavaScript. It ensures:
- Success is handled (set `result`)
- Errors are caught (set `error`)
- Cleanup always runs (set `loading = false`)

## Displaying the Result: Title, Author, Word Count, Status

When the API returns successfully, the result appears:

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

Key details:

- **`{#if result && result.meta}`**: We check both that `result` exists AND that it has a `meta` property. Sometimes the API returns a result without metadata (e.g., when a fic can't be found). The `&&` means both conditions must be true.

- **`{@const m = result.meta}`**: This creates a shorthand variable `m` for `result.meta`. Instead of writing `result.meta.title` everywhere, we can write `m.title`. The `@const` directive in Svelte means "this won't change" — it's calculated once and stays fixed as long as the `{#if}` block is visible. It's more efficient than a regular variable.

- **`{m.title}`**: The fic's title in a big `<h2>` heading. It's the most prominent text on the card.

- **`{m.author}`**: The author's name, shown in muted (dimmer) text.

- **`{detectSite(m.source)}`**: The `detectSite()` utility from Chapter 20 wraps the source domain in a friendly tag: "AO3", "FanFiction.net", etc. The result is wrapped in `<span class="tag">` to display as a pill-shaped badge.

- **`{formatWords(m.words)} words`**: The word count, formatted with commas. "1,234,567 words" instead of "1234567 words."

- **`{stripHtml(m.description).slice(0, 300)`**: The description, cleaned of HTML and truncated to 300 characters. Even long summaries won't overwhelm the card. The `.slice(0, 300)` ensures the description never exceeds 300 characters.

## Download Links: EPUB, HTML, MOBI, PDF

Below the metadata, the download buttons appear:

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

The `downloads` array is computed with `$derived`:

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

This is a smart pattern. The `add()` helper only adds a download link if the URL exists (not null, not undefined, not empty). If the backend didn't generate a MOBI file (maybe the fic is too complex for that format), `result.mobi_url` is null, and the MOBI button simply doesn't appear. The user only sees formats that are actually available.

The `download` attribute on the `<a>` tag tells the browser to download the file instead of navigating to it. When you click "EPUB", the browser downloads the .epub file directly to your downloads folder.

If no downloads are available, we show a helpful message — either the first note from the backend (explaining why the fic can't be downloaded) or a generic fallback: "This fic is not available for download right now."

The `?.` operator in `result.notes?.[0]` safely accesses the first element even if `notes` is undefined. The `??` operator provides the fallback text if the result is null or undefined.

## The Error Card: Showing Errors Nicely

When something goes wrong:

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

The error card uses the standard `.card` class but adds `.error-card` which overrides the border color to red:

```css
.error-card {
  border-color: var(--color-error);
}
```

Inside, the error message appears in red (`.error-text`), followed by a muted hint listing the supported sites. This is helpful UX: when someone pastes a URL from a site FicHub doesn't support, they immediately know why it failed AND which sites ARE supported.

The `⚠️` emoji prefix draws attention to the error. The `strong` tag makes the error text bold, further emphasizing it.

## The Bookmarklet: Drag to Bookmarks Bar

At the bottom of the Download tab, there's a small helper:

```svelte
<p class="muted hint">
  Tip: drag this to your bookmarks bar to download any fic from its page:
  <!-- svelte-ignore a11y_invalid_attribute -->
  <a
    class="bookmarklet"
    draggable="true"
    href="javascript:location.href='http://localhost:8004/api/v0/epub?q='+encodeURIComponent(location.href)"
    >FicHub ↗</a
  >
</p>
```

This is a bookmarklet — a bookmark that contains JavaScript instead of a URL. When dragged to the bookmarks bar and clicked on any fic page, it redirects the browser to FicHub's API with the current page's URL.

The `draggable="true"` attribute makes it draggable. The `javascript:` prefix in the `href` makes it executable code. The `encodeURIComponent()` safely encodes the current page's URL for use as a query parameter.

The `<!-- svelte-ignore a11y_invalid_attribute -->` comment tells Svelte's linter to ignore the accessibility warning about `javascript:` URLs — they're intentionally non-standard.

> **Watch Out:** The bookmarklet points to `localhost:8004`, which only works on your local development machine. In production, you'd change this to the actual server address. Users would also need to update their bookmarklets when the server URL changes.

## CSS Styling: Component-Level Styles

Each Svelte component has its own `<style>` block that's scoped — the styles only apply within that component. This prevents CSS conflicts between components. When the Download tab sets `.search-row { display: flex }`, it doesn't affect the Recommendations tab's `.search-row`.

```css
.search-row {
  display: flex;
  gap: 0.6rem;
  margin-bottom: 1rem;
}
.search-row input {
  flex: 1;
}
```

`display: flex` puts the input and button side by side. `flex: 1` makes the input take up all available space (the button stays its natural width). `gap: 0.6rem` adds spacing between them. `margin-bottom: 1rem` creates space between the search row and the content below.

The result card styles:

```css
.result-card h2 {
  margin: 0 0 0.3rem;
  font-size: 1.4rem;
}
.meta-line {
  color: var(--color-text);
  margin: 0.4rem 0;
}
.desc {
  color: var(--color-muted);
  font-size: 0.92rem;
}
.downloads {
  display: flex;
  flex-wrap: wrap;
  gap: 0.6rem;
  margin-top: 1rem;
}
```

The download buttons use `flex-wrap: wrap` so they wrap to a new line on narrow screens instead of overflowing. On a wide screen, they might show as: `[EPUB] [HTML] [MOBI] [PDF]`. On a narrow screen: `[EPUB] [HTML]` on one line, `[MOBI] [PDF]` on the next.

## The Full DownloadTab.svelte Walkthrough

Let's see the entire component as one cohesive unit, with every piece explained:

**The script section** contains:
- Imports (3 lines)
- State variables (4 lines)
- The download handler (23 lines)
- The keydown handler (3 lines)
- The derived downloads array (10 lines)

**The template section** contains:
- The search row (input + button)
- The error card (conditionally shown)
- The result card (conditionally shown)
- The bookmarklet hint

**The style section** contains:
- Layout rules (search row, downloads)
- Typography rules (headings, descriptions)
- The responsive breakpoint

The architecture follows a clear pattern:
1. **Imports** — what API functions and utilities we need
2. **State** — the reactive variables that track UI state
3. **Logic** — the functions that handle user actions
4. **Template** — the HTML structure with conditional rendering
5. **Styles** — scoped CSS for component-specific layout

Every tab in FicHub follows this same structure. Once you understand the Download tab, the Recommendations and Suggestions tabs will feel familiar.
## The User Experience Flow in Detail

Let's trace through the complete user experience from start to finish, paying attention to every micro-interaction:

**Step 1: The user arrives.** The Download tab is the default view (set in the layout). The input is empty, showing the placeholder text "Paste a fanfiction URL (AO3, FanFiction.net, forums…)." The button says "Download" and is fully opaque (not disabled).

**Step 2: The user pastes a URL.** The input text appears. The `bind:value` two-way binding updates the `url` state variable. The button remains clickable.

**Step 3: The user presses Enter.** The `onkeydown` handler detects the Enter key and calls `handleDownload()`. The function validates the URL (it's not empty), sets `loading = true`, and the UI immediately updates:
- The input becomes disabled (grayed out)
- The button text changes to "Working…" with a spinning animation
- The button becomes disabled

**Step 4: The API call is in progress.** The `fetchExport()` function sends a GET request to `/api/v0/epub?q=<url>`. The browser shows a network request in the dev tools. The UI is locked — no typing, no clicking.

**Step 5a: Success.** The API returns with `err: 0` and metadata. `result` is set to the response. The `{#if result && result.meta}` block renders the result card. The user sees the title, author, word count, and download links. The spinner stops, the button re-enables.

**Step 5b: Error.** The API returns with `err: 1` or throws an exception. `error` is set to a message. The `{#if error}` block renders the error card with the red border. The user sees the error message and the list of supported sites.

**Step 6: The user downloads a file.** They click one of the download buttons (EPUB, HTML, etc.). The browser starts downloading the file. The button uses the `download` attribute, so the file is saved directly instead of navigating to the URL.

This flow is designed to be foolproof:
- Validation prevents empty submissions
- Loading state prevents double-clicks
- Error messages explain what went wrong
- Success state shows clear download links

> **Try It Yourself:** Try pasting an invalid URL (like "not a url") and clicking Download. The backend will return an error, and the error card will appear. Now try pasting a valid AO3 URL — you should see the result card with download links. The transition between these states is what makes the UI feel responsive and alive.

## Accessibility Features

The Download tab includes several accessibility features that make it usable for people with disabilities:

1. **`aria-label="Fanfiction URL"`** on the input — screen readers announce this label when the user focuses the input
2. **`disabled={loading}`** on both input and button — prevents interaction during loading, which screen readers announce as "disabled"
3. **Semantic HTML** — `<h2>` for the title, `<p>` for paragraphs, `<a>` for links — screen readers use these to navigate
4. **Color contrast** — the dark theme's text color (`#e8eaf0`) against the background (`#0f1117`) has a contrast ratio of about 14:1, far exceeding the 4.5:1 minimum for accessibility
5. **Error messages** — errors appear in the DOM (not just in tooltips), so screen readers can announce them

These features don't add much code but make the app usable for a much wider audience.
## The API Client Pattern

Behind every tab's `handleDownload()`, `handleGetRecs()`, or `loadSuggestions()` function is the same API client pattern from `src/lib/api/client.ts`. Let's understand how it works.

The API client uses a generic `request()` function:

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

The `<T>` is a TypeScript generic — it means "this function can return any type, and the caller specifies which type." When `fetchExport()` calls `request<ExportResponse>(...)`, it tells TypeScript to expect an `ExportResponse` back.

The `if (!res.ok)` check handles HTTP errors (404, 500, etc.). It reads the error body as text and throws an `ApiError` with the status code and body. The `.catch(() => '')` ensures that even if reading the body fails, we still throw an error (with an empty body).

The `buildQuery()` helper constructs URL query parameters:

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

This filters out undefined, null, and empty values, then encodes the remaining key-value pairs into a query string. `encodeURIComponent()` ensures special characters in values (like `&` or `=` in URLs) don't break the query string.

The `ApiError` class extends `Error` with extra properties:

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

By extending `Error`, `ApiError` can be caught by standard `catch` blocks. The `instanceof ApiError` check in the component's error handler distinguishes API errors (which have status codes) from network errors (which don't).

This separation of concerns — API client handles networking, components handle UI — keeps the codebase clean and testable.

> **Watch Out:** The `ApiError` class is exported from the API client module and imported by components. This creates a dependency: if you change the `ApiError` class, all components that catch it are affected. In a larger app, you might define error types in a shared types file to reduce coupling.


## Performance Considerations

The Download tab is efficient:

- **No unnecessary re-renders:** `$state` variables only trigger updates when they change. Setting `error = ''` doesn't re-render if it was already empty.
- **Derived state is lazy:** `$derived.by()` only recalculates when its dependencies change. The `downloads` array is recomputed only when `result` changes.
- **Scoped CSS:** The `<style>` block only applies to this component. The browser doesn't need to check `.search-row` against every element on the page — just elements inside this component.
- **No network requests on mount:** The tab doesn't fetch anything when it first renders. Data is only fetched when the user clicks Download.

These optimizations are subtle but add up. The app feels fast because it avoids unnecessary work.
## Debugging the Download Tab
## The Bookmarklet in Detail

The bookmarklet at the bottom of the Download tab deserves a closer look. It's a clever hack that lets users download fics directly from any fic page, without copying the URL.

The bookmarklet code:

```javascript
javascript:location.href='http://localhost:8004/api/v0/epub?q='+encodeURIComponent(location.href)
```

Let's break it down:

1. `javascript:` — This tells the browser "this is executable code, not a URL." When you click a bookmarklet, the browser executes the JavaScript in the bookmark's href.

2. `location.href='...'` — This changes the current page's URL to the new value. The browser navigates to the new URL.

3. `'http://localhost:8004/api/v0/epub?q='` — This is the API endpoint for downloading fics. It's hard-coded to `localhost:8004` (the default backend address).

4. `+encodeURIComponent(location.href)` — This takes the CURRENT page's URL (the fic page you're viewing) and safely encodes it for use as a query parameter. `encodeURIComponent()` converts special characters like `&`, `=`, and `?` into URL-safe encodings.

So when you're on a fic page like `https://archiveofourown.org/works/12345`, clicking the bookmarklet navigates to:

```
http://localhost:8004/api/v0/epub?q=https%3A%2F%2Farchiveofourown.org%2Fworks%2F12345
```

The backend receives this request, fetches the fic, generates the EPUB, and returns a download response.

> **Watch Out:** The bookmarklet uses `localhost:8004`, which only works on your development machine. In production, you'd replace this with the actual server address. Users would also need to update their bookmarklets if the server URL changes. A more robust approach would be to use a relative URL, but that only works if the bookmarklet is served from the same domain as the API.

The `draggable="true"` attribute on the link makes it draggable to the bookmarks bar. Most browsers support dragging links to create bookmarks. The user drags the "FicHub ↗" link to their bookmarks bar, and it becomes a bookmarklet they can click on any page.

> **Try It Yourself:** Open the FicHub app, find the "FicHub ↗" bookmarklet at the bottom of the Download tab, and drag it to your bookmarks bar. Then navigate to any AO3 fic page and click the bookmarklet. You should be redirected to FicHub with the fic URL pre-filled. (This only works if the backend is running on localhost:8004.)


When something goes wrong with the Download tab, here's how to troubleshoot:

**Problem: Button does nothing when clicked**

Check the browser console for JavaScript errors. The most common cause is a missing import or a typo in the function name. Open dev tools (F12), go to the Console tab, and look for red error messages.

**Problem: "Network error" appears immediately**

The backend server isn't running. FicHub's frontend expects the API at `/api/v0/*`, which is proxied to the Rust backend. Start the backend with `cargo run` in the project root.

**Problem: "Server returned 404"**

The API endpoint doesn't exist. Check the backend code to see what routes are defined. The export endpoint is `GET /api/v0/epub?q=<url>`.

**Problem: "Server returned 500"**

The backend crashed while processing the request. Check the backend logs for error messages. Common causes: invalid URL format, network timeout when fetching the fic, or disk space issues when generating the EPUB.

**Problem: Result appears but download buttons don't work**

The download URLs are relative paths (`/cache/epub/abc123?h=...`). If you're running the frontend on a different port than the backend, the URLs won't resolve. Make sure the frontend's Vite dev server is configured to proxy API requests to the backend.

**Problem: Spinner never stops**

The API call is hanging. Open the Network tab in dev tools and look for pending requests. If the request is stuck, the backend might be processing a very large fic or experiencing a network timeout. Check the backend logs.

> **Try It Yourself:** Open the browser dev tools, go to the Network tab, and paste a URL into the Download tab. Watch the network request appear. Click on it to see the request URL, response headers, and response body. This is the most useful debugging technique for API-related issues.

## The Complete Error Handling Chain

Let's trace the full error handling flow from button click to error display:

1. **User clicks Download** → `handleDownload()` is called
2. **Empty URL** → `error = 'Please paste a fanfiction URL first.'` → returns early
3. **API call starts** → `loading = true` → UI shows spinner
4. **Network error** → `catch (e)` block catches it
5. **Check error type** → `e instanceof ApiError`? If yes, show status code. If no, show "Network error."
6. **`finally` block** → `loading = false` → spinner stops, button re-enables
7. **Error display** → `{#if error}` renders the error card with the message

The chain has three layers of error protection:

- **Layer 1: Input validation** (before the API call)
- **Layer 2: API error response** (the backend returns `err: 1`)
- **Layer 3: Network/exception errors** (fetch fails, server crashes)

Each layer handles a different kind of failure. Together, they ensure the user always sees a helpful message instead of a blank screen or a frozen UI.

The `try/catch/finally` pattern is the foundation:

```typescript
try {
  // Try the risky operation
  const res = await fetchExport(url.trim());
  // Handle success
  result = res;
} catch (e) {
  // Handle any error that occurred
  error = 'Something went wrong';
} finally {
  // Always clean up, regardless of success or failure
  loading = false;
}
```

The `finally` block is crucial. Without it, a thrown error would skip the `loading = false` assignment, leaving the UI stuck in a loading state forever. The `finally` block guarantees cleanup happens.

> **Watch Out:** The error handling catches ALL errors, including programming mistakes (like accessing a property that doesn't exist). In development, you might want to log these to the console: `console.error('Download error:', e)`. In production, you'd hide the technical details from users but log them for debugging.



## Practice: Add a New Download Format

The backend might add support for a new format, like "TXT". Here's how you'd add it:

1. In the `downloads` computed property, add one line:

```typescript
add('TXT', 'txt', result.txt_url);
```

2. That's it! The template already loops over `downloads` and renders a button for each one. The new format automatically appears.

Try adding it and checking that a "TXT" button appears in the results. (It won't work unless the backend actually generates TXT files, but the button will show up.)

---

# Chapter 22: The Recommendations Tab

## What Recommendations Are (Like a Friend Saying "You'll Love This")

We've all been there: you finish a great book and think, "What should I read next?" The best recommendations come from friends who know your taste — not algorithms that just match keywords.

FicHub's Recommendations tab works like that friend. You paste the URL of a fic you loved, and the system finds similar stories based on what other readers have bookmarked together. It's collaborative filtering — the same technology Netflix and Spotify use, but for fanfiction.

The core idea: if thousands of readers bookmarked both Fic A and Fic B, those two fics are probably similar. The more people who bookmarked both, the stronger the connection. It's like saying, "People who liked this also liked that."

This approach works because bookmarking behavior reveals taste. If you bookmark a slow-burn romance fic, the system finds other fics that readers with similar taste also bookmarked. It doesn't analyze the content of the fics — it analyzes the behavior of the readers.

## The Search Row: Paste a Fic URL

Same pattern as the Download tab:

```svelte
<div class="search-row">
  <input
    type="url"
    placeholder="URL of a fic you liked…"
    bind:value={url}
    onkeydown={onKeydown}
    disabled={loading}
    aria-label="Fic URL for recommendations"
  />
  <button class="btn" onclick={handleGetRecs} disabled={loading}>
    {#if loading}<span class="spinner"></span> Searching…{:else}Recommend{/if}
  </button>
</div>
```

The button says "Recommend" instead of "Download", and the loading text says "Searching…" instead of "Working…" — small differences that give each tab its own personality.

The placeholder text guides the user: "URL of a fic you liked…" — it tells you not just what to type, but why. You're not downloading this fic; you're using it as a seed to find more.

There's also a helpful intro paragraph above the search row:

```svelte
<p class="muted intro">
  Paste a fic you enjoyed, and FicHub will suggest similar stories based on what
  other readers bookmarked together.
</p>
```

This is important UX. New users might not understand what "recommendations" means in this context. The intro explains the mechanism (other readers' bookmarks) in plain language. It answers the question "How does this work?" before the user even asks it.

## fetchRecommendations() Call

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

Key difference from the Download tab: `fetchRecommendations()` takes three parameters:

1. `url.trim()` — the fic URL (the seed)
2. `undefined` — the `url_id` parameter (we don't have it yet; the API will resolve it from the URL)
3. `20` — the maximum number of recommendations to return

The response includes both the recommendations and the `site_domain` — the domain of the original fic (like "archiveofourown.org"). This is shown later in the count line: "20 recommendations from archiveofourown.org".

The error handling is slightly different from the Download tab. The recommendations endpoint doesn't return per-request error messages (`res.msg`), so we use generic messages: "Could not load recommendations." This is a simpler error experience.

## Sorting by Combined Score

Each recommendation comes with two scores:

```typescript
let sorted = $derived.by(() => {
  return [...recs].sort(
    (a, b) => b.score + b.community_score * 0.1 - (a.score + a.community_score * 0.1),
  );
});
```

- **`score`**: The algorithmic similarity score. Higher means more similar to the seed fic. This comes from the collaborative filtering algorithm.
- **`community_score`**: The net community votes. Readers can upvote or downvote recommendations.

The combined formula is `score + community_score * 0.1`. The community score is multiplied by 0.1 (divided by 10) so it doesn't overpower the algorithmic score. This gives community feedback some weight without letting a few enthusiastic voters hijack the results.

Why this weighting? Because the algorithmic score is based on thousands of bookmarks — it's a massive data signal. A community vote is a single person's opinion. By multiplying the community score by 0.1, we're saying "the algorithm's opinion counts 10× more than a single vote." But multiple community votes can still shift the ranking.

The `[...recs]` spread creates a copy of the array before sorting. This is important — Svelte's reactivity can behave unexpectedly if you sort the original array in place. Sorting modifies the array, which triggers re-renders, which could cause infinite loops. The copy avoids this.

## Displaying Recommendation Cards

Each recommendation renders as a card:

```svelte
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
```

Let's look at the details:

- **`(r.url_id)`**: This is Svelte's keyed each block. It tells Svelte to use `url_id` as the unique identifier for each item. When the list updates, Svelte can efficiently reorder existing DOM elements instead of recreating them. Without a key, Svelte would rebuild the entire list on any change.

- **`<a href={r.download_urls?.epub ?? '#'}>`**: The title is a link to the EPUB download. The `?.` operator safely accesses `download_urls.epub` even if `download_urls` is undefined. The `?? '#'` fallback means if there's no EPUB URL, the link goes to `#` (does nothing when clicked).

- **`stripHtml(r.summary).slice(0, 200)`**: Summaries are truncated even shorter here (200 chars vs 300 in the Download tab) because there are many cards on screen — keeping summaries short prevents the page from feeling overwhelming.

## The Community Score Badge

```svelte
{#if r.community_score !== 0}
  <span class="badge" class:positive={r.community_score > 0}>
    {r.community_score > 0 ? '▲' : '▼'} {Math.abs(r.community_score)} community
  </span>
{/if}
```

This only shows if there are any community votes (non-zero score). The triangle characters are visual shorthand:
- `▲` (up triangle) for positive scores — in green
- `▼` (down triangle) for negative scores — in the default muted color

`Math.abs()` removes the sign from the number so it always shows a positive count: "▲ 5 community" not "▲ +5 community". The triangle already communicates direction; showing the sign would be redundant.

The `.positive` class changes the color to green:

```css
.badge.positive {
  color: var(--color-success);
}
```

## EPUB Download Button

Each recommendation card has a quick-download button:

```svelte
{#if r.download_urls?.epub}
  <a class="btn btn-secondary sm" href={r.download_urls.epub} download>EPUB</a>
{/if}
```

This only shows if an EPUB URL is available. The `.sm` class makes the button smaller than the main Download button:

```css
.btn.sm {
  padding: 0.35rem 0.7rem;
  font-size: 0.82rem;
}
```

You can download directly from the recommendations list — no need to go back to the Download tab for each fic. This saves clicks and makes the discovery-to-reading flow seamless.

## Empty State: "No Recommendations Yet"

```svelte
{#if !loading && recs.length === 0 && !error}
  <div class="card empty">
    <p class="muted">No recommendations yet. Try a fic above!</p>
  </div>
{/if}
```

This appears when:
- We're not loading (the request has completed)
- There are no recommendations to show
- There's no error message

It's a friendly nudge: "The app is ready, just give it a URL!" Without this, the screen would be blank after loading, and users might think something's broken.

The three-part condition is important. If you only checked `recs.length === 0`, the empty message would flash briefly during loading (before the API responds). The `!loading` check prevents that. The `!error` check prevents showing "No recommendations" when there was actually an error.

> **Watch Out:** The condition `!loading && recs.length === 0 && !error` means this message only shows when the initial state is reached — no search has been performed. After a search that returns zero results, `recs.length === 0` is true, but so might be `loading` (briefly). The three-part check ensures clean state transitions.

## The Count Line

```svelte
{#if sorted.length > 0}
  <p class="muted count">{sorted.length} recommendations{siteDomain ? ` from ${siteDomain}` : ''}</p>
```

A small but informative line: "20 recommendations from archiveofourown.org". It tells the user how many results they got and where the seed fic came from.

The `siteDomain ? ... : ''` is a ternary operator: if `siteDomain` exists, append " from archiveofourown.org"; otherwise, show nothing. This handles cases where the site domain isn't available.

## The Full RecommendationsTab.svelte Walkthrough

Here's the complete component:

The script section mirrors the Download tab's structure:
- Same state variables (`url`, `loading`, `error`) plus tab-specific ones (`recs`, `siteDomain`)
- Same `handleGetRecs()` pattern with try/catch/finally
- Same `onKeydown()` handler
- A `$derived` property for sorting

The template follows the same flow:
1. Intro text
2. Search row
3. Error card (if error)
4. Empty state (if no results)
5. Results list (if results exist)

The CSS is also similar:
- Same `.search-row` layout
- Same `.error-card` styling
- Tab-specific styles for `.rec-item`, `.rec-footer`, `.badge`

The consistency is intentional. If you understand one tab, you understand them all. The differences are in the data: what API endpoint is called, what response is expected, what fields are displayed.
## How Recommendations Differ from the Download Tab
## Understanding the API Response Types

Before diving into the component, let's understand the data structures. The TypeScript types in `src/lib/api/types.ts` define the shape of every API response:

```typescript
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

This interface defines what a single recommendation looks like. Every property has a type:

- `url_id: string` — the unique identifier (like "ao3/12345")
- `title: string` — the fic's title
- `author: string` — the author's name
- `words: number` — word count as a raw number (not formatted)
- `chapters: number` — chapter count
- `status: string` — "Complete", "In Progress", etc.
- `site_domain: string` — "archiveofourown.org", "fanfiction.net", etc.
- `summary: string` — the fic's summary (may contain HTML)
- `score: number` — algorithmic similarity score
- `community_score: number` — net community votes
- `download_urls: Record<string, string>` — map of format → URL

The `download_urls` property is a dictionary: `{ "epub": "/cache/epub/abc123?h=...", "html": "/cache/html/abc123?h=..." }`. The component accesses it with `r.download_urls?.epub` — the `?.` safely handles cases where `download_urls` is undefined.

The response wrapper adds metadata:

```typescript
export interface RecommendationsResponse {
  err: number;
  url_id: string;
  site_domain?: string | null;
  recommendations: RecResult[];
  generated_at: string;
}
```

- `err: number` — 0 for success, non-zero for errors
- `url_id: string` — the resolved ID of the seed fic
- `site_domain` — the domain of the seed fic (optional)
- `recommendations: RecResult[]` — the array of recommendations
- `generated_at: string` — when the recommendations were generated

The `err` field is a convention from the Rust backend. Instead of using HTTP status codes for application errors, the backend returns 200 OK with `err: 1` and a `msg` field explaining the error. This allows the frontend to display user-friendly error messages while the HTTP layer remains clean.

> **Try It Yourself:** Open the browser dev tools, go to the Network tab, and make a recommendation request. Click on the response and look at the JSON structure. Can you map each field to the TypeScript interface? Notice how `recommendations` is an array, `err` is 0, and each recommendation has the fields defined in `RecResult`.


While the structure is similar, the Recommendations tab has unique characteristics worth understanding:

**Different API behavior.** The Download tab calls `fetchExport()` which returns a single result for one fic. The Recommendations tab calls `fetchRecommendations()` which returns an *array* of results — multiple fics that are similar to the seed.

**Different data flow.** In the Download tab, the user's URL becomes the result. In the Recommendations tab, the user's URL is just a seed — the results are *other* fics. The seed fic itself doesn't appear in the results.

**Sorting matters.** The Download tab shows one result — no sorting needed. The Recommendations tab shows many results, and the order matters. The combined score formula (`score + community_score * 0.1`) ensures the most relevant fics appear first.

**Links go to downloads.** In the Download tab, the title isn't a link (there's only one fic). In the Recommendations tab, each title links to the EPUB download, making it easy to grab a fic directly from the list.

**Summary truncation.** The Download tab shows up to 300 characters of summary. The Recommendations tab shows only 200. This is because there are many cards on screen, and shorter summaries prevent the page from feeling overwhelming.

These differences show how the same architectural pattern adapts to different use cases. The pattern provides structure; the specifics provide meaning.

## Understanding the Score Formula

The combined score formula deserves a closer look:

```typescript
b.score + b.community_score * 0.1
```

Let's say Fic A has `score: 8.5` and `community_score: 3`. Its combined score is `8.5 + 0.3 = 8.8`.

Fic B has `score: 8.2` and `community_score: 10`. Its combined score is `8.2 + 1.0 = 9.2`.

Despite Fic A having a higher algorithmic score, Fic B ranks higher because of strong community support. The 0.1 multiplier means a community score of 10 adds 1.0 to the combined score — enough to overcome a 1.0 deficit in the algorithmic score.

This balance is tunable. If you wanted community votes to matter more, increase the multiplier to 0.2 or 0.5. If you wanted the algorithm to dominate, decrease it to 0.05. The current value (0.1) is a reasonable starting point that can be adjusted based on user feedback.

> **Try It Yourself:** Look at the recommendations for a popular fic. Notice how the community score badges cluster around 0-5 for most fics, but a few outliers have scores of 10 or more. Those outliers are the community's favorites — fics that many readers explicitly endorsed. They often rank higher than the algorithm would suggest, which is the intended behavior.
## How the Recommendations Algorithm Works

While the full algorithm lives in the Rust backend, understanding the basic concept helps you appreciate the recommendations tab.

The algorithm uses **collaborative filtering** — the same technique Netflix and Spotify use. Here's the simplified version:

1. **Collect bookmarks.** Every user who bookmarks fics creates a data point: "I liked Fic A enough to bookmark it."

2. **Find overlaps.** If User 1 bookmarked Fic A and Fic B, and User 2 bookmarked Fic A and Fic C, then Fic B and Fic C are indirectly connected through their shared relationship with Fic A.

3. **Score connections.** The more users who share bookmarks between two fics, the stronger the connection. If 1000 users bookmarked both Fic A and Fic B, that's a very strong signal.

4. **Rank results.** For a given seed fic, rank all other fics by their connection strength. The top results become the recommendations.

The `score` field in each recommendation represents this connection strength. A score of 8.5 means "this fic is strongly connected to the seed fic based on user bookmarking behavior."

The `community_score` adds a human layer on top. While the algorithm finds fics that are *similar* to the seed, community votes indicate fics that are *good companions* to the seed. These aren't always the same thing — a fic might be algorithmically similar but poorly written, or algorithmically distant but a perfect thematic match.

This two-layer approach (algorithm + community) produces better recommendations than either approach alone.

> **Try It Yourself:** Compare the algorithmic scores and community scores for the top 10 recommendations. Do they correlate? Are there fics with high algorithmic scores but low community scores (or vice versa)? These mismatches reveal the difference between "similar" and "good."



## Practice: Add a "Sort By" Dropdown

Here's a fun modification. Add a dropdown that lets users choose between sorting by score, word count, or recency:

1. Add a new state variable:

```typescript
let sortBy = $state<'score' | 'words' | 'updated'>('score');
```

2. Update the derived sort:

```typescript
let sorted = $derived.by(() => {
  const copy = [...recs];
  if (sortBy === 'words') {
    return copy.sort((a, b) => b.words - a.words);
  }
  if (sortBy === 'updated') {
    return copy.sort((a, b) =>
      (b.updated ?? '').localeCompare(a.updated ?? '')
    );
  }
  // Default: combined score
  return copy.sort(
    (a, b) => b.score + b.community_score * 0.1 - (a.score + a.community_score * 0.1),
  );
});
```

3. Add the dropdown in the template:

```svelte
<select bind:value={sortBy}>
  <option value="score">Best Match</option>
  <option value="words">Longest First</option>
  <option value="updated">Most Recent</option>
</select>
```

The sort changes instantly because `$derived` automatically recalculates whenever `sortBy` or `recs` changes. Svelte handles the DOM updates — no manual re-rendering needed.

---

# Chapter 23: The Suggestions Tab

## What Community Suggestions Are (Readers Propose Similar Fics)

Recommendations are automatic — the algorithm finds similar fics. But what if you, as a reader, know a perfect companion fic that the algorithm missed? That's what suggestions are for.

The Suggestions tab is like a community bulletin board. Readers can propose fics that pair well with a story, add comments explaining why, and other readers can vote on whether the suggestion is good. It's collaborative curation — humans adding their knowledge on top of the algorithm.

The workflow:

1. **Load suggestions** for a fic by pasting its URL
2. **View the list** of proposed companion fics, sorted by net votes
3. **Vote** on suggestions you agree or disagree with
4. **Add your own suggestion** via a modal form

This creates a virtuous cycle: the algorithm provides a base layer of recommendations, readers add their personal picks, and voting surfaces the best community suggestions. Over time, the quality of recommendations improves as more people contribute.

## Loading Suggestions: fetchVotes()

```typescript
let seedUrl = $state('');
let loading = $state(false);
let error = $state('');
let suggestions = $state<Suggestion[]>([]);
let loadedUrlId = $state('');
```

State variables for this tab:
- `seedUrl`: the URL the user pasted
- `loading`: whether we're currently fetching
- `error`: error message to display
- `suggestions`: the array of community suggestions
- `loadedUrlId`: the resolved ID of the seed fic (needed for voting and suggesting)

The loading function:

```typescript
async function loadSuggestions() {
  if (!seedUrl.trim()) {
    error = 'Paste the URL of a fic to see its community suggestions.';
    return;
  }
  loading = true;
  error = '';
  try {
    // Resolve url_id via the recommendations endpoint
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

Here's something interesting: this function makes **two** API calls.

1. **First, it calls `fetchRecommendations()`** with just 1 result. Why? Because the votes API needs a `url_id` (a unique fic identifier), but the user only has a URL. The recommendations endpoint can resolve a URL to a `url_id`. It doesn't need to actually find recommendations — we pass `n=1` just to get the ID.

2. **Then it calls `fetchVotes()`** with the resolved `url_id`. This returns the list of community suggestions for that fic.

This two-step dance is a common pattern in web applications: the user provides a human-readable identifier (URL), the system converts it to an internal identifier (url_id), and then uses that for subsequent queries. It's like looking up someone's phone number by name, then calling the number.

## Resolving url_id via fetchRecommendations()

The dynamic import is worth noting:

```typescript
const { fetchRecommendations } = await import('$lib/api/client');
```

Instead of importing `fetchRecommendations` at the top of the file (with the other imports), it's imported inside the function. This is called a **dynamic import** and it's done for code splitting — the import only loads when the function is called, not when the component first mounts.

In practice, for this small app, it doesn't make a huge difference. But it's a good habit for larger apps where you want to minimize what loads initially. The browser only downloads the code for `fetchRecommendations` when the user actually loads suggestions, not when they open the Suggestions tab.

## The Suggestion Header

Once the url_id is resolved, the tab shows a header with a count and a "Suggest a fic" button:

```svelte
{#if loadedUrlId && !loading}
  <div class="sugg-header">
    <span class="muted">{suggestions.length} community suggestions</span>
    <button class="btn btn-secondary sm" onclick={openModal}>+ Suggest a fic</button>
  </div>
```

The `{#if loadedUrlId && !loading}` guard ensures this only appears after the url_id has been resolved and loading is complete. Before that, we don't know which fic we're looking at, so showing the header would be premature.

The "+ Suggest a fic" button uses the secondary style (quieter than the primary button) and the `.sm` size class. The "+" prefix suggests addition — you're adding a new suggestion.

## The Suggestion List

Suggestions display in a list:

```svelte
{#if suggestions.length === 0}
  <div class="card empty"><p class="muted">No suggestions yet. Be the first!</p></div>
{:else}
  <div class="sugg-list">
    {#each sorted as s (s.id)}
      <div class="card sugg-item">
        <div class="sugg-main">
          <a href={`/api/v0/meta?q=${s.suggested_url_id}`} class="sugg-link">
            {s.suggested_url_id}
          </a>
          {#if s.comment}
            <p class="muted comment">{s.comment}</p>
          {/if}
        </div>
        <div class="vote-box">
          <button class="vote-btn up" class:active={netScore(s) > 0}
            onclick={() => handleVote(s.id, 1)} aria-label="Upvote">▲</button>
          <span class="score" class:positive={netScore(s) > 0} class:negative={netScore(s) < 0}>
            {netScore(s)}
          </span>
          <button class="vote-btn down" class:active={netScore(s) < 0}
            onclick={() => handleVote(s.id, -1)} aria-label="Downvote">▼</button>
        </div>
      </div>
    {/each}
  </div>
{/if}
```

Each suggestion shows:
- The `suggested_url_id` as a clickable link (users can click to see the fic's metadata)
- An optional comment from the suggester
- An upvote/downvote control with the net score

The layout uses flexbox with `justify-content: space-between` — the suggestion content is on the left, the vote box is on the right. This keeps the voting controls aligned regardless of how long the suggestion text is.

> **Try It Yourself:** Click "+ Suggest a fic" to open the modal, type a URL, add a comment, and submit. The list refreshes and your suggestion appears. Now upvote it — the score changes instantly (optimistic update), then refreshes when the server confirms.

## The Suggest Modal: Overlay Form

When users want to add their own suggestion, they click the "+ Suggest a fic" button. This opens a modal dialog:

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

The modal is an overlay that sits on top of the main content. Let's examine each piece.

### Modal State

```typescript
let showModal = $state(false);
let suggestUrl = $state('');
let suggestComment = $state('');
let submitting = $state(false);
let modalError = $state('');
```

Five state variables just for the modal:
- `showModal` controls visibility
- `suggestUrl` and `suggestComment` are the form fields
- `submitting` is a loading state (prevents double-submission)
- `modalError` shows validation or server errors

### Opening and Closing

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

`openModal()` resets all form fields before showing the modal. This ensures you never see leftover text from a previous submission. It's a common pattern: always reset state when opening a modal.

### The Backdrop Click

```svelte
<div class="modal-backdrop" onclick={closeModal} role="presentation">
  <div class="modal" onclick={(e) => e.stopPropagation()}>
```

Clicking the dark backdrop behind the modal closes it. But clicking inside the modal itself does NOT close it — that's what `e.stopPropagation()` does. It prevents the click event from bubbling up to the backdrop.

This is a standard modal pattern: click outside = close, click inside = do nothing. It feels natural because it matches how physical dialogs work — you can interact with the content, and clicking away dismisses it.

The `role="dialog"` and `aria-modal="true"` attributes are accessibility features. They tell screen readers that this is a modal dialog and that focus should be trapped inside it.

### Form Validation

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

The validation is simple: the URL is required, the comment is optional. If the URL is empty, an error appears inside the modal.

After successful submission, the modal closes and the suggestion list reloads to show the new entry.

The `suggestComment.trim() || undefined` is clever: if the comment is empty after trimming whitespace, it passes `undefined` instead of an empty string. The backend treats these differently — `undefined` means "no comment was provided," while `""` would be "the user explicitly submitted an empty comment."

### Closing After Submission

Notice the flow after successful submission:

```typescript
closeModal();           // Close the modal immediately
await loadSuggestions(); // Then reload the list
```

The modal closes before the list reloads. This gives the user instant feedback — the modal disappears right away, and then the list updates. If we awaited `loadSuggestions()` first, the modal would stay open while the list reloads in the background, which feels sluggish.

## Voting: Upvote and Downvote

```typescript
async function handleVote(id: number, vote: 1 | -1) {
  const prev = localVotes[id] ?? 0;
  localVotes[id] = (localVotes[id] ?? 0) + vote;
  try {
    const res = await castVote(id, vote);
    if (res.err !== 0) {
      localVotes[id] = prev;
      return;
    }
    await loadSuggestions();
  } catch {
    localVotes[id] = prev;
  }
}
```

This function introduces a sophisticated pattern: **optimistic voting**.

### Optimistic Voting: Update UI Immediately, Rollback on Error

Here's the problem: if you click upvote and wait for the server to respond before updating the UI, there's a noticeable delay. The user clicks, nothing happens for a second, then the number changes. It feels sluggish.

Optimistic voting solves this by updating the UI *immediately*, before the server confirms:

1. **Save the current value:** `const prev = localVotes[id] ?? 0`
2. **Update the UI immediately:** `localVotes[id] += vote`
3. **Send the request to the server:** `await castVote(id, vote)`
4. **If it fails, roll back:** `localVotes[id] = prev`

The user sees instant feedback. If the server confirms, great — the UI is already correct. If the server rejects the vote, the UI snaps back to the previous value.

The `?? 0` is the nullish coalescing operator. If `localVotes[id]` is undefined (the user hasn't voted on this suggestion before), it defaults to 0.

### localVotes State: Tracking Unconfirmed Votes

```typescript
let localVotes = $state<Record<number, number>>({});
```

This is a dictionary (Record) mapping suggestion IDs to local vote adjustments. When you upvote suggestion #42, `localVotes[42]` becomes `1`. If you then downvote it, it becomes `0` (back to neutral) or `-1` (net downvote).

The `Record<number, number>` type means "an object where keys are numbers (suggestion IDs) and values are numbers (vote adjustments)."

### netScore Calculation: Server Score + Local Optimism

```typescript
function netScore(s: Suggestion): number {
  return s.net_votes + (localVotes[s.id] ?? 0);
}
```

The displayed score combines:
- `s.net_votes`: the server's confirmed vote count
- `localVotes[s.id]`: any unconfirmed local votes

This means the displayed number reflects what the user has done locally, even before the server confirms. It's a blend of "what the server says" and "what I just did."

```typescript
let sorted = $derived.by(() => [...suggestions].sort((a, b) => netScore(b) - netScore(a)));
```

The sort uses `netScore()` so the list reorders instantly when you vote, without waiting for a server round-trip. If you upvote the 5th suggestion, it immediately moves up in the list.

> **Watch Out:** Optimistic updates add complexity. If you vote and then immediately navigate away, the server might not have received the vote yet. For a fanfiction app, this is acceptable — a missed vote isn't catastrophic. For a banking app, you'd want pessimistic updates (wait for confirmation). The choice depends on the consequences of the operation.

## The Full SuggestionsTab.svelte Walkthrough

This is the most complex of the three tabs, with:
- **Two API calls** (fetchRecommendations + fetchVotes) instead of one
- **Modal state** (5 variables)
- **Optimistic voting** (localVotes + netScore)
- **Dynamic import** (fetchRecommendations)

But the pattern is the same: state → logic → derived state → template → styles.

The script section defines:
- Loading state (`seedUrl`, `loading`, `error`, `suggestions`, `loadedUrlId`)
- Modal state (`showModal`, `suggestUrl`, `suggestComment`, `submitting`, `modalError`)
- Voting state (`localVotes`)
- Functions: `loadSuggestions()`, `openModal()`, `closeModal()`, `handleSuggest()`, `handleVote()`
- Derived: `netScore()`, `sorted`

The template follows the same flow as the other tabs, with the addition of the modal overlay.

The CSS adds modal-specific styles on top of the shared patterns.

## CSS: .vote-box, .vote-btn, .modal-backdrop, .modal

### Vote Controls

The vote box is a vertical stack: upvote button on top, score in the middle, downvote on the bottom.

```css
.vote-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.15rem;
  flex-shrink: 0;
}
```

`flex-shrink: 0` prevents the vote box from being squeezed by long text. Without it, if the suggestion text is very long, the vote box would compress and the buttons would become too small to click.

The vote buttons are small, compact controls:

```css
.vote-btn {
  background: var(--color-surface-2);
  border: 1px solid var(--color-border);
  color: var(--color-muted);
  border-radius: var(--radius-sm);
  width: 2rem;
  height: 1.6rem;
  line-height: 1;
}
```

Fixed width (2rem) and height (1.6rem) ensure the buttons are always the same size, regardless of the arrow character's width. The `line-height: 1` vertically centers the arrow characters.

The active states color the buttons:

```css
.vote-btn.up.active {
  color: var(--color-success);
  border-color: var(--color-success);
}
.vote-btn.down.active {
  color: var(--color-error);
  border-color: var(--color-error);
}
```

When the net score is positive, the upvote button turns green. When negative, the downvote button turns red. The `active` class is applied via `class:active={netScore(s) > 0}` — Svelte's conditional class syntax.

The score display changes color too:

```css
.score.positive {
  color: var(--color-success);
}
.score.negative {
  color: var(--color-error);
}
```

### Modal Styles

The modal system has two layers:

```css
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 50;
  padding: 1rem;
}
```

The backdrop covers the entire viewport with a semi-transparent black overlay. `position: fixed` and `inset: 0` (which is shorthand for `top: 0; right: 0; bottom: 0; left: 0`) ensure it covers everything, even if the page is scrolled. `z-index: 50` puts it above all other content.

```css
.modal {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius);
  padding: 1.5rem;
  width: 100%;
  max-width: 480px;
  box-shadow: var(--shadow);
}
```

The modal itself is a card-like container centered in the viewport. `max-width: 480px` prevents it from becoming too wide on large screens. `width: 100%` ensures it fills the available space on small screens (with the `padding: 1rem` from the backdrop providing margins).

The form labels use `display: block` so each label-input pair stacks vertically:

```css
.modal label {
  display: block;
  margin: 0.8rem 0;
  font-size: 0.9rem;
  color: var(--color-muted);
}
.modal label input,
.modal label textarea {
  width: 100%;
  margin-top: 0.3rem;
}
```

The input and textarea stretch full width (`width: 100%`), with a small gap below the label text (`margin-top: 0.3rem`).

The action buttons are right-aligned:

```css
.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.6rem;
  margin-top: 1rem;
}
```

`justify-content: flex-end` pushes the buttons to the right. Cancel is secondary (quieter), Submit is primary (purple).
## Why Optimistic Updates Matter
## Understanding the Suggestion Data Types

Let's look at the TypeScript types that power the Suggestions tab:

```typescript
export interface Suggestion {
  id: number;
  suggested_url_id: string;
  comment: string | null;
  net_votes: number;
  created: string | null;
}
```

Each suggestion has:

- `id: number` — a numeric identifier (used for voting)
- `suggested_url_id: string` — the fic being suggested (like "ao3/12345")
- `comment: string | null` — an optional comment from the suggester
- `net_votes: number` — the current vote count (upvotes minus downvotes)
- `created: string | null` — when the suggestion was submitted

The `comment: string | null` type is important. It means the comment can be a string OR null. In the template, we check `{#if s.comment}` before displaying it — this handles both cases (null and empty string).

The voting API uses separate request and response types:

```typescript
export interface SuggestResponse {
  err: number;
  suggestion_id?: number;
  msg?: string;
}

export interface VoteResponse {
  err: number;
  new_score: number;
}
```

`SuggestResponse` returns the new suggestion's ID (if successful) or an error message. `VoteResponse` returns the new score after voting.

The `VotesResponse` wraps the suggestions array:

```typescript
export interface VotesResponse {
  err: number;
  url_id: string;
  suggestions: Suggestion[];
}
```

This follows the same `err` pattern as other responses. The `url_id` confirms which fic the suggestions are for.

> **Watch Out:** The `Suggestion.id` is a number, while `Suggestion.suggested_url_id` is a string. The ID is an internal database identifier; the URL ID is a human-readable identifier like "ao3/12345". When voting, you use the numeric ID. When displaying the suggestion, you show the URL ID.


The Suggestions tab introduces optimistic updates — a pattern that significantly improves perceived performance. Let's understand why this matters by comparing the two approaches.

**Without optimistic updates (pessimistic):**
1. User clicks ▲ upvote
2. UI shows a loading spinner next to the score
3. Network request fires (200-500ms)
4. Server responds with confirmation
5. UI updates with the confirmed score

Total perceived wait: 200-500ms of "something is happening" followed by the result. The user stares at a spinner.

**With optimistic updates:**
1. User clicks ▲ upvote
2. UI immediately shows score + 1 (instant!)
3. Network request fires in the background
4. Server responds with confirmation
5. If successful, UI refreshes to match server state
6. If failed, UI rolls back to previous value

Total perceived wait: 0ms. The response is instant. The network request is invisible.

The difference is dramatic. Optimistic updates make the app feel like it's running locally, even though it's making network requests. The user never waits.

The trade-off is complexity. You need to:
- Track the local state separately from the server state
- Handle rollback when the server rejects the change
- Reconcile local and server states after confirmation

For voting in a fanfiction app, this trade-off is absolutely worth it. A missed vote isn't catastrophic, but the instant feedback makes the app feel polished and responsive.

## State Management Across the Component

The Suggestions tab has the most complex state management of the three tabs. Let's map it out:

```
seedUrl ──────────┐
loading ──────────┤
error ────────────┤  loadSuggestions()  ──→  suggestions[]
loadedUrlId ──────┘                            │
                                               ▼
                                    sorted = suggestions.sort(netScore)
                                        │
                                        ▼
                                   {#each sorted}
                                        │
                                        ▼
                                    netScore(s)
                                        │
                                   ┌────┴────┐
                                   ▼         ▼
                              s.net_votes  localVotes[s.id]
```

The data flows in one direction: from user input → API calls → state updates → derived sorts → template rendering. This unidirectional flow makes the component predictable and debuggable.

The `localVotes` dictionary is the most interesting piece. It's a parallel state that tracks unconfirmed changes. When the API confirms a vote, the list reloads and `localVotes` becomes irrelevant for that suggestion (because the server state now matches).

> **Watch Out:** If a user votes on multiple suggestions rapidly, several `localVotes` entries accumulate. When `loadSuggestions()` runs after each vote, it refreshes all suggestions, which resets the `net_votes` values. The `localVotes` for already-confirmed votes still have their values, but they're now adding to the new server values. This is correct behavior — the local optimism is additive, not absolute.
## Comparing the Three Tabs

Now that we've covered all three tabs, let's compare them side by side:

| Feature | Download | Recommendations | Suggestions |
|---------|----------|----------------|-------------|
| **Input** | Any fic URL | Any fic URL | Any fic URL |
| **API call(s)** | 1 (`fetchExport`) | 1 (`fetchRecommendations`) | 2 (`fetchRecommendations` + `fetchVotes`) |
| **Output** | Single fic + download links | Array of similar fics | Array of community suggestions |
| **User actions** | Download files | Browse + download | Vote + suggest + browse |
| **State complexity** | Low (4 variables) | Medium (5 variables) | High (12 variables) |
| **Modal** | No | No | Yes |
| **Optimistic updates** | No | No | Yes |
| **Keyed each block** | No | Yes (`r.url_id`) | Yes (`s.id`) |

The Download tab is the simplest — one API call, one result, one action. The Recommendations tab adds multiple results and sorting. The Suggestions tab adds community interaction (voting, suggesting) with the most complex state management.

This progression is intentional. Users encounter the simplest feature first (Download), then discover more complex features as they explore. The complexity increases gradually, matching the user's growing familiarity with the app.

All three tabs share:
- The same search row pattern (input + button)
- The same error handling pattern (try/catch/finally)
- The same loading state pattern (spinner + disabled controls)
- The same CSS patterns (.card, .btn, .error-card, .search-row)

## The Svelte 5 Reactivity Model

All three tabs use Svelte 5's runes for reactivity. Let's understand the key runes:

- **`$state(value)`** — declares reactive state. When the value changes, the UI updates.
- **`$derived(expression)`** — declares a computed value that automatically updates when its dependencies change.
- **`$derived.by(() => ...)`** — same as `$derived` but for complex expressions that need a function body.
- **`$props()`** — declares component props (not used in these tabs since they don't receive props).

The reactivity is automatic. When you write `loading = true`, Svelte knows that any template expression using `loading` needs to re-render. No manual subscriptions, no event emitters, no `setState()` calls.

This is different from React, where you'd write `setLoading(true)` and React would re-render the component. In Svelte, you just assign to the variable, and the framework handles the rest.

The `$derived` rune creates a dependency graph. If `sorted` depends on `recs`, and `recs` changes, `sorted` automatically recalcul. If `recs` doesn't change, `sorted` keeps its cached value. This is called "memoization" and it prevents unnecessary recomputation.

> **Try It Yourself:** Add a `console.log` inside a `$derived.by()` block and watch it log only when dependencies change. Change an unrelated state variable — the derived value shouldn't recompute. This demonstrates Svelte's fine-grained reactivity.
## Common Mistakes and How to Avoid Them

After studying these components, here are the most common mistakes developers make and how to avoid them:

**Mistake 1: Forgetting to clear old state when loading new data.**

```typescript
// ❌ Wrong: old results linger while new ones load
async function handleDownload() {
  loading = true;
  const res = await fetchExport(url);
  result = res;
  loading = false;
}

// ✅ Right: clear old state immediately
async function handleDownload() {
  loading = true;
  error = '';
  result = null;  // Clear old results
  try {
    const res = await fetchExport(url);
    result = res;
  } catch (e) {
    error = '...';
  } finally {
    loading = false;
  }
}
```

**Mistake 2: Not handling the `finally` block.**

```typescript
// ❌ Wrong: if an error occurs, loading stays true forever
try {
  loading = true;
  const res = await fetchExport(url);
  result = res;
  loading = false;  // Only runs on success!
} catch (e) {
  error = '...';
  // loading is still true!
}

// ✅ Right: finally always runs
try {
  loading = true;
  const res = await fetchExport(url);
  result = res;
} catch (e) {
  error = '...';
} finally {
  loading = false;  // Always runs, even on error
}
```

**Mistake 3: Mutating arrays in place instead of creating copies.**

```typescript
// ❌ Wrong: mutating the original array can confuse Svelte's reactivity
let sorted = $derived.by(() => {
  return recs.sort((a, b) => b.score - a.score);  // Mutates recs!
});

// ✅ Right: create a copy first
let sorted = $derived.by(() => {
  return [...recs].sort((a, b) => b.score - a.score);  // Safe copy
});
```

**Mistake 4: Accessing properties without null checks.**

```typescript
// ❌ Wrong: crashes if download_urls is undefined
<a href={r.download_urls.epub}>Download</a>

// ✅ Right: safe access with optional chaining
<a href={r.download_urls?.epub ?? '#'}>Download</a>
```

These patterns appear throughout FicHub's codebase. Understanding them helps you write code that's robust, maintainable, and bug-free.




## Practice: Add a Comment Preview

Here's a fun exercise. Add a live preview of the comment below the textarea so users can see how their comment will look before submitting.

1. Find the textarea in the modal:

```svelte
<textarea rows="3" placeholder="Why do you recommend this?" bind:value={suggestComment}></textarea>
```

2. Add a preview right below it:

```svelte
{#if suggestComment}
  <div class="comment-preview muted">
    Preview: {suggestComment}
  </div>
{/if}
```

3. Add the CSS:

```css
.comment-preview {
  margin-top: 0.4rem;
  padding: 0.5rem;
  background: var(--color-bg);
  border-radius: var(--radius-sm);
  font-size: 0.85rem;
  border: 1px dashed var(--color-border);
}
```

Now as the user types their comment, they see a live preview below the textarea. The dashed border and slightly different background make it clear this is a preview, not another input field. The `{#if suggestComment}` guard hides the preview when the textarea is empty.

---

# Chapter 24: The Search Syntax Parser

## What Is a Syntax Parser? (Translating Human Language to Computer Commands)

Imagine you walk into a coffee shop and say, "Large oat milk latte, extra hot, no whip." The barista hears your words, understands each request, and translates them into actions: pick a large cup, use oat milk, make the milk extra hot, skip the whipped cream.

A syntax parser does the same thing for search queries. Users type things like:

```
fandom:Harry Potter words:10000-50000 -tag:Angst complete:true
```

The parser takes that string and translates it into structured data that the backend can understand:

```json
{
  "include_tags": "1:Harry Potter",
  "exclude_tags": "4:Angst",
  "min_words": 10000,
  "max_words": 50000,
  "complete": true
}
```

This is a massive usability feature. Instead of filling out a complex form with dropdowns and checkboxes, users can type naturally — like they're talking to the search engine. Power users can type fast queries with short aliases: `f:HP w:10k+ comp:y s:ao3`.

The parser lives in `src/lib/search/syntax.ts`. It's 347 lines of TypeScript that convert human-readable search syntax into structured filter objects. Let's look at every piece.

## The AO3-Like Syntax

FicHub's search syntax is inspired by Archive of Our Own (AO3), the largest fanfiction archive. AO3 users are already familiar with syntax like `fandom:Harry Potter` and `words:10000-50000`, so FicHub uses the same conventions.

The full syntax reference:

| Syntax | Meaning | Filter |
|---|---|---|
| `title:Harry Potter` | Search title | → `q` |
| `author:JKR` | Search author | → `q` |
| `fandom:Harry Potter` | Fandom tag | → `include_tags` type 1 |
| `char:Harry Potter` | Character tag | → `include_tags` type 2 |
| `rel:Harry/Hermione` | Relationship tag | → `include_tags` type 3 |
| `tag:Fluff` | Freeform tag | → `include_tags` type 4 |
| `-tag:Angst` | Exclude tag | → `exclude_tags` |
| `words:10000-50000` | Word count range | → `min_words`, `max_words` |
| `words:>10000` | Minimum words | → `min_words` |
| `words:<5000` | Maximum words | → `max_words` |
| `chapters:5-20` | Chapter range | → `min_chapters`, `max_chapters` |
| `complete:true` | Only complete fics | → `complete` |
| `site:ao3` | Filter by site | → `source` |
| `sort:updated` | Sort order | → `sort` |
| `after:2024-01-01` | Published after | → `date_from` |
| `before:2024-12-31` | Published before | → `date_to` |

Short aliases are also supported: `t` for title, `a` for author, `f` for fandom, `c` for character, `r` for relationship, `w` for words, `ch` for chapters, `s` for site. This lets power users type `f:HP w:10k+` instead of `fandom:Harry Potter words:>10000`.

Bare words (anything without a `key:` prefix) become the full-text search query. Mixed queries work too: `best story fandom:Harry Potter` searches for "best story" AND requires the Harry Potter fandom.

## Tokenizing: Splitting the Input into Tokens

The first step in parsing is tokenization — splitting the input string into individual pieces. For example:

```
fandom:Harry Potter words:10000-50000 -tag:Angst
```

Should become: `["fandom:Harry Potter", "words:10000-50000", "-tag:Angst"]`

The basic tokenizer:

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

This walks through each character:

1. If we're inside a quote, add characters to the current token until we hit the closing quote
2. If we hit a quote character (and we're not in a quote), start a quoted section
3. If we hit a space or tab, push the current token and start a new one
4. Otherwise, add the character to the current token

Quotes allow multi-word values without spaces breaking them: `fandom:"Harry Potter"` → `["fandom:Harry Potter"]`

The tokenizer handles both double quotes (`"`) and single quotes (`'`). It also handles mixed quotes: `fandom:"Harry Potter" tag:'Fluff'` works correctly.

> **Watch Out:** The tokenizer is simple and doesn't handle escaped quotes (like `fandom:"Harry \"Potter\""`). For FicHub's use case, this is fine — fic titles and tags don't contain quotes. But in a general-purpose parser, you'd need escape character handling.

## The Greedy Tokenizer: Handling Multi-Word Values

The basic tokenizer is too simple for our needs. Consider:

```
fandom:Harry Potter tag:Fluff
```

The basic tokenizer splits on spaces, giving: `["fandom:Harry", "Potato", "tag:Fluff"]` — wrong! "Harry Potter" should be one token.

Wait, where did "Potato" come from? That was a typo in my explanation — the actual output would be `["fandom:Harry", "Potter", "tag:Fluff"]`. "Harry" and "Potter" are separate tokens because there's a space between them. Either way, it's wrong — we want them together.

The greedy tokenizer fixes this by merging tokens after a key: prefix until the next recognized key:

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
        // Empty value after colon — grab following tokens
        const parts: string[] = [token];
        i++;
        while (i < basic.length && !isKeyToken(basic[i])) {
          parts.push(basic[i]);
          i++;
        }
        result.push(parts.join(' '));
      } else {
        const key = token.slice(0, colonIdx).toLowerCase();
        if (key === 'words' || key === 'w' || key === 'chapters' || key === 'ch' || 
            key === 'sort' || key === 'complete' || key === 'comp' || 
            key === 'after' || key === 'before' || key === 'site' || key === 's') {
          // Range/single-value keys — don't merge
          result.push(token);
          i++;
        } else {
          // Tag-like keys — merge following non-key tokens
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

This is the most complex part of the parser. Here's the logic:

1. Start with basic tokens from `tokenize()`
2. Walk through them one by one
3. If a token starts with a known key (like `fandom:`), look at the following tokens
4. For "range" keys (words, chapters, complete, site, sort, dates), don't merge — the value is always a single token like `words:10000-50000`
5. For "tag" keys (fandom, char, rel, tag, title, author), merge subsequent tokens until the next key — because `fandom:Harry Potter tag:Fluff` needs "Harry Potter" to be part of the fandom value
6. If the colon value is empty (`fandom:` with nothing after it), grab ALL following non-key tokens

The `isKeyToken()` helper:

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

It strips a leading `-` (for exclusions like `-tag:Angst`), finds the colon, extracts the key before it, and checks if it's in the `KNOWN_KEYS` set. If the token has no colon (`colonIdx < 1`), it's not a key token.

## Known Keys

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

A `Set` is a data structure that provides fast membership checking — `KNOWN_KEYS.has('fandom')` is O(1) (instant), while checking an array would be O(n) (slower for large lists). For a list of 20 items, the difference is negligible, but Sets are the idiomatic choice for this pattern.

The aliases provide flexibility:
- `t` for title (saves keystrokes)
- `a` for author (also saves keystrokes)
- `creator` for author (some users think of it as "creator" not "author")
- `freeform` for tag (AO3 terminology)
- `character` for char (full word vs abbreviation)

## parseToken(): Extracting Key:Value Pairs

```typescript
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

This parses a single token into a structured object:

- **`-tag:Angst`** → `{ key: "tag", value: "Angst", exclude: true }`
- **`fandom:Harry Potter`** → `{ key: "fandom", value: "Harry Potter", exclude: false }`
- **`hello`** → `null` (no colon, not a key token)

The function returns `null` for tokens that aren't key:value pairs. The caller handles these as bare words (free-text search terms).

The `ParsedToken` interface defines the structure:

```typescript
interface ParsedToken {
  key: string;
  value: string;
  exclude: boolean;
}
```

Three fields: the key (lowercase), the value (trimmed), and whether it's an exclusion.

## Exclusion with -: -tag:Angst

The leading `-` means "exclude this." The parser handles it by:

1. Stripping the `-` in `parseToken()` and setting `exclude: true`
2. In the main switch statement, routing to `exclude_tags` instead of `include_tags`

```typescript
case 'tag':
case 'freeform':
  if (exclude) {
    filters.exclude_tags = appendTag(filters.exclude_tags, 4, value);
  } else {
    filters.include_tags = appendTag(filters.include_tags, 4, value);
  }
  break;
```

The `appendTag()` helper builds a comma-separated string:

```typescript
function appendTag(existing: string, typeId: number, name: string): string {
  const entry = `${typeId}:${name}`;
  return existing ? `${existing},${entry}` : entry;
}
```

Two tags become `"1:Harry Potter,4:Fluff"`. The `typeId` prefix tells the backend what kind of tag it is: 1=Fandom, 2=Character, 3=Relationship, 4=Freeform.

The `existing ? ... : entry` pattern: if there's already an existing tag string, append with a comma. If not, start fresh with just the entry. This avoids a leading comma.

## Range Parsing: words:>10000, chapters:5-20

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

Three formats are supported:

1. **Greater than:** `>10000` → `{ min: 10000, max: null }`
2. **Less than:** `<5000` → `{ min: null, max: 5000 }`
3. **Range:** `10000-50000` → `{ min: 10000, max: 50000 }`
4. **Exact:** `10000` → `{ min: 10000, max: null }` (just a minimum)

The `isNaN()` checks ensure that invalid numbers (like `words:abc`) don't crash the parser — they just get ignored. A `null` min or max means "no limit" in that direction.

> **Watch Out:** The range parser uses `split('-')` which could be confused by negative numbers. But word counts and chapter counts are always positive, so this isn't an issue in practice. If you needed to support negative ranges (which doesn't make sense for word counts), you'd need a more sophisticated parser.

## Site Shorthands: ao3, ffn, sv, sb

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

Users type `site:ao3`, the parser converts it to `archiveofourown.org`. Both `ff` and `ffn` map to FanFiction.net because people use different abbreviations.

If the value doesn't match any shorthand, it's used as-is — so `site:archiveofourown.org` works too, just not as convenient. The `?? value` fallback passes through unrecognized values.

The `Record<string, string>` type means "an object where both keys and values are strings."

## Date Normalization: after:2024-01-01

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

This normalizes dates in three formats:

1. **Full date:** `2024-01-15` → `"2024-01-15T00:00:00Z"` (for `after`) or `"2024-01-15T23:59:59Z"` (for `before`)
2. **Year-month:** `2024-01` → `"2024-01-01T00:00:00Z"` or `"2024-01-28T23:59:59Z"`
3. **Year only:** `2024` → `"2024-01-01T00:00:00Z"` or `"2024-12-31T23:59:59Z"`

The `bound` parameter controls whether we use the start or end of the period. "After 2024-01" means "starting from January 1st." "Before 2024-01" means "up to January 28th."

The regex patterns validate the date format:
- `^\d{4}-\d{2}-\d{2}$` matches "YYYY-MM-DD"
- `^\d{4}-\d{2}$` matches "YYYY-MM"
- `^\d{4}$` matches "YYYY"

If the value doesn't match any pattern, it's returned as-is — the backend might handle it differently.

> **Watch Out:** The year-month normalization uses `28` for the end day (`2024-01-28`), not 31. This is a conservative choice — it works for all months (February has 28 or 29 days, all others have at least 28). For a more precise implementation, you'd calculate the actual last day of each month.

## The parseSearchQuery() Function: Full Walkthrough

Here's the main function that ties everything together:

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

The function starts by creating default filters — an empty search with no restrictions. If the input is empty, it returns these defaults immediately.

Then it tokenizes the input and processes each token:

1. **`parseToken()`** tries to extract a key:value pair
2. If it returns `null`, the token is a bare word → added to `bareWords[]`
3. If it returns a parsed token, the `switch` statement routes it to the appropriate filter

The `switch` statement handles all known keys with their aliases. Title and author values go into `bareWords` (they're full-text searchable). Tags go into `include_tags` or `exclude_tags`. Ranges go into `min_words`/`max_words`. And so on.

After processing all tokens, any bare words are joined with spaces and set as `filters.q`.

Let's trace through a complex query:

```
fandom:Harry Potter tag:Fluff -tag:Angst words:10000-50000 complete:true sort:updated
```

**Step 1: Greedy tokenize** → `["fandom:Harry Potter", "tag:Fluff", "-tag:Angst", "words:10000-50000", "complete:true", "sort:updated"]`

**Step 2: Parse each token:**

| Token | Key | Value | Exclude | Action |
|---|---|---|---|---|
| `fandom:Harry Potter` | fandom | Harry Potter | false | `include_tags += "1:Harry Potter"` |
| `tag:Fluff` | tag | Fluff | false | `include_tags += "4:Fluff"` |
| `-tag:Angst` | tag | Angst | true | `exclude_tags += "4:Angst"` |
| `words:10000-50000` | words | 10000-50000 | false | `min_words=10000, max_words=50000` |
| `complete:true` | complete | true | false | `complete=true` |
| `sort:updated` | sort | updated | false | `sort="updated"` |

**Step 3: Build result:**

```json
{
  "q": "",
  "include_tags": "1:Harry Potter,4:Fluff",
  "exclude_tags": "4:Angst",
  "min_words": 10000,
  "max_words": 50000,
  "complete": true,
  "sort": "updated"
}
```

That's the whole parser in action. From a human-readable string to structured search filters in about 300 lines of TypeScript.

## Bare Words and Mixed Queries

What about queries that mix syntax with plain text?

```
best story fandom:Harry Potter
```

The parser sees "best" and "story" — neither starts with a known key. They become bare words. `fandom:Harry Potter` becomes an include tag. The result:

```json
{
  "q": "best story",
  "include_tags": "1:Harry Potter"
}
```

Bare words and structured filters work together seamlessly. You can type a free-text search alongside specific filters.

Another example: `title:The Chosen One author:JKR words:>50000`

- `title:The Chosen One` → bare word "The Chosen One" (added to `q`)
- `author:JKR` → bare word "JKR" (added to `q`)
- `words:>50000` → `min_words: 50000`

Result: `q: "The Chosen One JKR"`, `min_words: 50000`

The title and author are searched as free text, while the word count filter restricts the results.

## Practice: Test the Parser with Different Inputs

The project includes a test file (`src/lib/search/syntax.test.ts`) with over 20 test cases. Here are some interesting ones to explore:

**Simple search:**
```typescript
parseSearchQuery('hello world')
// → { q: "hello world", include_tags: "" }
```

**Exclusion:**
```typescript
parseSearchQuery('-tag:Major Character Death')
// → { exclude_tags: "4:Major Character Death" }
```

**Complex query:**
```typescript
parseSearchQuery('fandom:Harry Potter tag:Fluff -tag:Angst words:10000-50000 complete:true sort:updated')
// → {
//   include_tags: "1:Harry Potter,4:Fluff",
//   exclude_tags: "4:Angst",
//   min_words: 10000,
//   max_words: 50000,
//   complete: true,
//   sort: "updated"
// }
```

**Site shorthand:**
```typescript
parseSearchQuery('site:ffn')
// → { source: "fanfiction.net" }
```

**Date filtering:**
```typescript
parseSearchQuery('after:2024-01-01 before:2024-12-31')
// → { date_from: "2024-01-01T00:00:00Z", date_to: "2024-12-31T23:59:59Z" }
```

> **Try It Yourself:** Open `src/lib/search/syntax.test.ts` and add a new test case. Try parsing a query with all features combined: bare words, multiple tags (included and excluded), a word range, complete status, a site filter, and date range. What does the output look like? Then run the tests with `npm test` to verify your understanding.

> **Another Exercise:** What happens with edge cases? Try `parseSearchQuery('')` (empty string), `parseSearchQuery('fandom:')` (empty value), `parseSearchQuery('words:abc')` (invalid number). In each case, the parser handles it gracefully — empty results, null values, or ignored filters.

## Why This Parser Matters

The syntax parser is the bridge between human intent and machine execution. Without it, users would need to fill out a complex form with:
- A text box for search terms
- Separate inputs for min/max word counts
- Checkboxes for each tag type
- Dropdowns for site selection
- Date pickers for date ranges

That's at least 10 form fields. With the parser, users type one line and get everything. It's faster to type `f:HP w:10k+ comp:y s:ao3` than to click through ten form fields.

And because the syntax is inspired by AO3, experienced fanfiction readers already know how to use it. They've been typing these kinds of queries for years on AO3's search page. FicHub just brought that power to the frontend.

The parser also enables URL-based sharing. You can construct a search URL like `/search?q=fandom:Harry%20Potter%20words:>10000` and share it with friends. They paste it into their browser and see the same search results. Try doing that with a form-based search interface!

## The Architecture of Parsing

Looking at the parser's architecture, it follows a classic pattern:

1. **Tokenize** — split raw input into meaningful chunks
2. **Parse tokens** — extract structure (key, value, exclude) from each chunk
3. **Apply rules** — map parsed tokens to filter properties
4. **Assemble output** — combine all filters into the final object

This pattern (tokenize → parse → transform → assemble) is used in compilers, interpreters, and query parsers everywhere. Once you understand it here, you'll recognize it in SQL parsers, Markdown processors, and template engines.

The key insight is the separation of concerns: the tokenizer doesn't know about filter types, the parser doesn't know about the output format, and the main function orchestrates everything. Each piece does one thing well.
## Testing the Parser

The parser has a comprehensive test suite in `src/lib/search/syntax.test.ts` with over 20 test cases. Testing parsers is particularly important because:

1. **Edge cases are everywhere.** Empty strings, invalid dates, unknown keys, malformed ranges — a parser needs to handle all of them gracefully.

2. **Regression risk is high.** A small change to the tokenizer could break complex queries. Tests catch these regressions.

3. **The behavior is deterministic.** Given the same input, the parser always produces the same output. This makes testing straightforward — no mocking, no randomness, no async.

Here's how the test file is structured:

```typescript
import { describe, it, expect } from 'vitest';
import { parseSearchQuery } from './syntax';

describe('parseSearchQuery', () => {
  it('parses bare words into q', () => {
    const f = parseSearchQuery('hello world');
    expect(f.q).toBe('hello world');
  });
  // ... more tests
});
```

Each test is a small, focused assertion. The `describe` block groups related tests. The `it` block names the specific behavior being tested. The `expect` assertion verifies the output.

Some notable test cases:

```typescript
// Multiple tags combine with commas
it('parses fandom:', () => {
  const f = parseSearchQuery('fandom:Harry Potter');
  expect(f.include_tags).toBe('1:Harry Potter');
});

// Exclusion uses exclude_tags
it('parses exclusion with -', () => {
  const f = parseSearchQuery('-tag:Major Character Death');
  expect(f.exclude_tags).toBe('4:Major Character Death');
});

// Complex queries with multiple features
it('handles complex queries', () => {
  const f = parseSearchQuery(
    'fandom:Harry Potter tag:Fluff -tag:Angst words:10000-50000 complete:true sort:updated',
  );
  expect(f.include_tags).toBe('1:Harry Potter,4:Fluff');
  expect(f.exclude_tags).toBe('4:Angst');
  expect(f.min_words).toBe(10000);
  expect(f.max_words).toBe(50000);
  expect(f.complete).toBe(true);
  expect(f.sort).toBe('updated');
});

// Empty string returns defaults
it('handles empty string', () => {
  const f = parseSearchQuery('');
  expect(f.q).toBe('');
  expect(f.include_tags).toBe('');
});
```

> **Try It Yourself:** Run the tests with `npm test` (or `vitest` in the frontend directory). All 20+ tests should pass. Then try adding a test for a query you've invented — something not covered by existing tests. Does the parser handle it correctly?

## How the Parser Connects to the Search Page

The parser isn't used by the Download, Recommendations, or Suggestions tabs. It's used by the Search page — a separate route at `/search`. When a user types a query into the search bar and presses Enter, the query is passed through `parseSearchQuery()` before being sent to the backend:

```typescript
const filters = parseSearchQuery(rawQuery);
const results = await search(filters);
```

The `search()` function (from `src/lib/api/search.ts`) takes the parsed filters and builds a query string for the API:

```typescript
export function buildSearchQuery(filters: SearchFilters): string {
  const params = new URLSearchParams();
  if (filters.q) params.set('q', filters.q);
  if (filters.include_tags) params.set('include_tags', filters.include_tags);
  // ... more parameters
  return params.toString();
}
```

The chain is: **User types query → `parseSearchQuery()` → `SearchFilters` object → `buildSearchQuery()` → URL query string → `fetch('/api/v0/search?...')`**

Each step transforms the data into a more structured form. The parser turns human text into structured filters. The builder turns filters into a URL. The fetch sends the URL to the server.

This layered architecture means each piece can be tested independently. You can test the parser without any API calls. You can test the builder without any parsing. And you can test the full flow end-to-end.
## Common Parsing Patterns You'll See Again

The FicHub parser uses several patterns that appear in parsers everywhere. Understanding them here will help you read other parsers in the future.

**Pattern 1: Tokenize → Parse → Transform**

Almost every parser follows this three-step pipeline:

1. **Tokenize**: Split raw text into meaningful chunks (tokens)
2. **Parse**: Extract structure from each token (key, value, flags)
3. **Transform**: Map parsed tokens into the output format (filter objects)

SQL parsers do this: `SELECT * FROM users WHERE age > 18` → tokens `["SELECT", "*", "FROM", "users", "WHERE", "age", ">", "18"]` → AST nodes → query plan.

Markdown parsers do this: `# Hello **world**` → tokens `["#", "Hello", "**", "world", "**"]` → elements → HTML.

Template engines do this: `Hello {{name}}!` → tokens `["Hello ", "{{", "name", "}}", "!"]` → template nodes → rendered output.

Once you see this pattern, you recognize it everywhere.

**Pattern 2: Guard Clauses for Invalid Input**

The parser never crashes on bad input. Every function checks for edge cases:

```typescript
// If token has no colon, it's not a key:value pair
const colonIdx = s.indexOf(':');
if (colonIdx < 1) return null;

// If value is empty, it's not useful
if (!value) return null;

// If range contains non-numeric parts, ignore them
const n = Number(value.slice(1));
return { min: isNaN(n) ? null : n, max: null };
```

The pattern is: check → return early if invalid → process if valid. This makes the parser robust against any input.

**Pattern 3: Comma-Separated Accumulation**

Tags accumulate in a comma-separated string:

```typescript
function appendTag(existing: string, typeId: number, name: string): string {
  const entry = `${typeId}:${name}`;
  return existing ? `${existing},${entry}` : entry;
}
```

First tag: `"1:Harry Potter"`. Second tag: `"1:Harry Potter,4:Fluff"`. Third tag: `"1:Harry Potter,4:Fluff,2:Draco Malfoy"`.

This pattern is common in APIs that accept multiple values in a single string parameter. The alternative (an array) would require JSON encoding, which is harder to type in a URL.

**Pattern 4: Alias Mapping**

The same concept has multiple names:

```typescript
case 'title':
case 't':
  bareWords.push(value);
  break;
```

`title` and `t` both map to the same behavior. This is called aliasing and it's common in CLIs, query languages, and configuration files. It reduces friction for users who prefer shorter or longer forms.

## Extending the Parser

If you wanted to add new search syntax (like `rating:explicit` or `kudos:>100`), you'd follow these steps:

1. **Add the key to `KNOWN_KEYS`:**

```typescript
const KNOWN_KEYS = new Set([
  // ... existing keys
  'rating', 'kudos',
]);
```

2. **Add a case to the switch statement:**

```typescript
case 'rating':
  filters.rating = value.toLowerCase();
  break;

case 'kudos': {
  const range = parseRange(value);
  if (range.min !== null) filters.min_kudos = range.min;
  if (range.max !== null) filters.max_kudos = range.max;
  break;
}
```

3. **Add the field to `SearchFilters`:**

```typescript
export interface SearchFilters {
  // ... existing fields
  rating: string;
  min_kudos: number | null;
  max_kudos: number | null;
}
```

4. **Add a test case:**

```typescript
it('parses rating:explicit', () => {
  const f = parseSearchQuery('rating:explicit');
  expect(f.rating).toBe('explicit');
});
```

That's the full cycle: define the key, handle the case, declare the type, write the test. The parser's architecture makes extensions straightforward.

> **Try It Yourself:** Try adding a `sort:updated` parser case that maps to `filters.sort = 'updated'`. Then test it with `parseSearchQuery('sort:updated')`. Does it work? What about `sort:kudos`?



---

# Summary

Part 5 covered the entire frontend of FicHub, from global styles to individual components:
# Key Takeaways from Part 5
## How All the Pieces Connect

Let's zoom out and see how the pieces we've covered fit together in the full application:

**The Layout (`+layout.svelte`)** is the container. It imports `app.css` (global styles), renders the top navigation bar, and switches between the three tab components based on user selection. It also includes a search bar that navigates to the search page.

**The Download Tab** is the primary feature. Users paste a URL, the backend fetches the fic and generates export files, and the tab displays download links. It uses `fetchExport()` from the API client, `formatWords()` and `detectSite()` from utilities, and follows the standard state → logic → derived → template pattern.

**The Recommendations Tab** is the discovery feature. Users paste a fic they liked, and the backend returns similar fics based on collaborative filtering. It uses `fetchRecommendations()`, sorts by a combined algorithmic and community score, and displays results in a scrollable list.

**The Suggestions Tab** is the community feature. Users browse community suggestions for a fic, vote on them, and add their own. It uses the most complex state management with optimistic voting, modal forms, and dual API calls.

**The Search Page** (not covered in detail in Part 5, but referenced) uses the syntax parser to convert user queries into structured filters, then sends them to the search API. The parser lives in `src/lib/search/syntax.ts`, and the search API client lives in `src/lib/api/search.ts`.

**The API Client** (`src/lib/api/client.ts`) handles all HTTP communication. It provides typed functions for each endpoint (`fetchExport`, `fetchRecommendations`, `fetchVotes`, etc.) and handles error translation.

**The Utility Functions** (`src/lib/util.ts`) provide shared helpers for formatting and text processing. They're used across multiple components.

**The Type Definitions** (`src/lib/api/types.ts`) define the shape of every API response. They ensure type safety across the entire frontend.

Together, these pieces form a complete frontend application. The data flows from user input → API calls → state updates → DOM rendering. The CSS provides visual consistency. The TypeScript types ensure correctness.

This architecture is clean, maintainable, and scalable. Adding a new feature means adding a new component that follows the same patterns. Changing the theme means updating CSS variables. Modifying the API means updating TypeScript interfaces and the corresponding functions.

> **Try It Yourself:** Draw a diagram of the data flow in FicHub. Start with user input (pasting a URL), trace it through the component logic, API calls, state updates, and finally to the rendered DOM. This exercise will solidify your understanding of how all the pieces connect.


Before we summarize, let's distill the most important lessons from these six chapters:

**1. CSS variables are the foundation of theming.** Define your colors, fonts, and spacing as variables, and every element references them. Changing the theme means changing the variables, not hunting through dozens of files.

**2. Utility functions should be small, focused, and shared.** A function that does one thing well (format a number, strip HTML, detect a site) is more valuable than a function that does many things. Put them in a shared file and import them wherever needed.

**3. Every component follows the same pattern.** State → Logic → Derived State → Template → Styles. Once you learn this pattern, you can read any Svelte component.

**4. Error handling has three layers.** Input validation (before the API call), API error responses (the backend says something went wrong), and network exceptions (the connection failed). Handle all three.

**5. Loading states prevent confusion.** Always show a spinner and disable controls during API calls. Users should never wonder "is it working?"

**6. Optimistic updates improve perceived performance.** For low-risk operations like voting, update the UI immediately and reconcile with the server afterward. Users hate waiting.

**7. Parsers follow a standard pattern.** Tokenize → Parse → Transform → Assemble. Once you understand this pipeline, you can read any parser.

**8. Type safety catches bugs early.** TypeScript interfaces define the shape of every data structure. When the backend changes a field name, the compiler tells you immediately instead of discovering it at runtime.

**9. Responsive design is not optional.** Every component should work on mobile and desktop. Use flexbox for layout, `flex-wrap` for wrapping, and `@media` queries for breakpoints.

**10. Consistency builds trust.** When every tab uses the same patterns (search row, loading state, error card, card layout), users learn the app faster. Consistency reduces cognitive load.

These principles apply far beyond FicHub. They're the foundation of modern frontend development.

- **Chapter 19** showed how CSS variables create a cohesive, easily-customizable theme with just 16 variables controlling every visual aspect

- **Chapter 19** showed how CSS variables create a cohesive, easily-customizable theme with just 16 variables controlling every visual aspect
- **Chapter 20** demonstrated utility functions as reusable tools for formatting numbers, dates, text, and URLs
- **Chapter 21** walked through the Download tab — FicHub's core feature with its search row, API call, result display, and error handling
- **Chapter 22** explored the Recommendations tab and its collaborative filtering approach with community scores
- **Chapter 23** covered the Suggestions tab with its community voting, optimistic updates, and modal form
- **Chapter 24** dissected the search syntax parser that translates human queries into structured data

Every component follows the same architectural pattern: state variables → API calls → derived state → conditional template rendering. Once you understand this pattern, you can read, modify, and extend any part of the UI.

The three tabs share more than structure — they share philosophy. Each one is self-contained (all logic, template, and styles in one file), responsive (works on mobile and desktop), accessible (aria labels, keyboard navigation), and error-resilient (try/catch/finally with user-friendly messages).

In Part 6, we'll move to the backend and see how the Rust server handles all these API requests, generates EPUB files, and serves the search index.
## Glossary of Frontend Terms

Here's a quick reference for the key terms used throughout Part 5:

- **CSS Custom Property** — A variable defined with `--name` and referenced with `var(--name)`. Also called a CSS variable.

- **Component** — A self-contained piece of UI with its own logic, template, and styles. In Svelte, it's a `.svelte` file.

- **Reactive State** — Variables declared with `$state()` that automatically trigger UI updates when changed.

- **Derived State** — Computed values declared with `$derived()` that automatically recalculate when their dependencies change.

- **Two-way Binding** — `bind:value={variable}` in Svelte keeps an input field and a variable in sync automatically.

- **Scoped CSS** — Styles inside a `<style>` block in a Svelte component that only apply to that component's elements.

- **Guard Clause** — An early return at the top of a function that handles invalid input before processing.

- **Optimistic Update** — Updating the UI immediately before the server confirms the change, with rollback on failure.

- **API Client** — A module that provides typed functions for making HTTP requests to the backend.

- **Tokenize** — The process of splitting raw text into meaningful chunks (tokens) for parsing.

- **Collaborative Filtering** — A recommendation technique that finds similar items based on user behavior patterns.

- **Flexbox** — A CSS layout model that distributes space along a single axis. Used extensively in FicHub's layout.

- **Responsive Design** — Designing layouts that adapt to different screen sizes using media queries and flexible units.

- **Accessibility (a11y)** — Designing interfaces that are usable by people with disabilities, including screen reader users and keyboard-only users.

These terms form the vocabulary of modern frontend development. Mastering them will help you read documentation, communicate with other developers, and build better applications.

# Part 6: Advanced Search and Navigation

*Where we build the app shell, the search page, the filter system, and teach users a query language that makes them feel like power users.*

---

# Chapter 25: The Layout and Navigation

## The App Shell

Every great house has a foundation. Every great app has a layout.

In SvelteKit, the layout is defined by a special file: `+layout.svelte`. This file wraps *every* page in your application. It's the frame around the picture, the scaffolding around the building, the shell around the pearl. Whatever page the user navigates to, the layout stays constant.

Think about the apps you use every day. Gmail always has that sidebar with your inbox, sent, drafts. Twitter always has the top navigation bar. YouTube always has the search bar at the top. These persistent elements are layouts — they provide context and navigation no matter where you are in the app.

FicHub follows the same pattern. The layout provides:

1. **A sticky topbar** — always visible at the top of the screen, no matter how far you scroll. It contains the brand, tabs, and search field.
2. **Tab-based content switching** — the main area of the page renders different components depending on which tab is active: Download, Recommendations, or Suggestions.
3. **A footer** — the quiet little line at the bottom listing supported sites.

This separation is important. The layout is the **container** — it knows *where* things go. The pages and components are the **content** — they know *what* goes there. This separation makes the codebase easier to understand, easier to modify, and less likely to have bugs.

Let's look at the actual code. This is the entire `+layout.svelte` file:

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

That's the entire script section. Notice how clean it is — just imports, a type definition, state variables, and one function. It's about 25 lines of actual logic for the entire app shell. Let's break down each piece.

### Imports

```svelte
import '../app.css';
```

The first line imports the global stylesheet. `app.css` defines CSS custom properties like `--color-primary`, `--color-surface`, `--color-border`, and `--radius-sm`. Every component in the app can use these variables, so the whole app shares a consistent look. If you want to change the primary color from blue to purple, you change it in one place and the entire app updates.

The next three imports bring in the three tab components. Each is a self-contained Svelte file in `$lib/components/`. The layout doesn't need to know what's inside them — it just renders the right one based on the active tab. This is a key principle: **the layout is a container, not a content creator**. It knows *which* component to show, but not *what* that component does.

```svelte
import { goto } from '$app/navigation';
```

`goto` is SvelteKit's client-side navigation function. When we call `goto('/search?q=hello')`, the browser doesn't do a full page reload — SvelteKit just swaps the page component and updates the URL. It's fast, smooth, and it keeps the app feeling like a single-page app even though it's technically using URLs.

Behind the scenes, `goto` calls `history.pushState` to update the URL, then triggers SvelteKit's internal routing to unmount the current page and mount the new one. No network request, no white flash, no lost scroll position. It's the magic that makes SvelteKit SPAs feel so snappy.

### Props and Children

```svelte
let { children } = $props();
```

In Svelte 5, `$props()` replaces the old `export let` pattern. The layout receives `children` as a prop — this is the content of whichever page is currently active. If you're on the home page, `children` is the `+page.svelte` from the root route. If you're on `/search`, `children` is the search page. The layout decides *where* to put the children (in the `<main>` element), but the children decide *what* gets rendered there.

Think of it like a theater stage. The layout is the stage itself — it has lights, curtains, and a backdrop. The `children` prop is the play being performed. Different plays (pages) can use the same stage (layout), but the stage doesn't care which play is running.

> **Try It Yourself:** Think of `children` as a slot. The layout says "I have a space for content right here," and each page fills that space with its own markup. This is the fundamental layout pattern — the frame stays the same, the picture changes.

### The Tab Type

```svelte
type Tab = 'download' | 'recs' | 'sugg';
let activeTab = $state<Tab>('download');
```

We define a union type `Tab` with three possible values. The `activeTab` variable holds which tab is currently selected. By default, it's `'download'` — the first thing users see is the download form.

Using `$state<Tab>` means Svelte 5's reactivity system tracks this variable. When `activeTab` changes, Svelte automatically re-renders any `{#if}` blocks that depend on it. The `<Tab>` type parameter is just TypeScript — it tells the compiler that `activeTab` can only be one of three strings. If you accidentally wrote `activeTab = 'banana'`, TypeScript would catch the error before the app even runs.

### The Tabs Array

```svelte
const tabs: { id: Tab; label: string; icon: string }[] = [
  { id: 'download', label: 'Download', icon: '⬇' },
  { id: 'recs', label: 'Recommendations', icon: '★' },
  { id: 'sugg', label: 'Suggestions', icon: '💡' },
];
```

Instead of writing three separate buttons, we define the tabs as data. Each tab has an `id` (matching the `Tab` type), a `label` (the text shown to the user), and an `icon` (the emoji). This makes it trivial to add a new tab later — just add an entry to the array. No need to copy-paste button markup, remember to add the right click handler, or worry about matching class names.

This is the "data-driven UI" pattern: instead of writing UI markup for each item, you describe the items as data and let a loop generate the UI. It's DRY (Don't Repeat Yourself), maintainable, and it scales beautifully.

If we wanted to add a fourth tab — say, "History" — we'd just add `{ id: 'history', label: 'History', icon: '📜' }` to the array and add a new `{:else if}` branch in the template. Two changes, no copy-paste.

### The Search Function

```svelte
let searchQuery = $state('');

function onSearchKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && searchQuery.trim()) {
    goto(`/search?q=${encodeURIComponent(searchQuery.trim())}`);
  }
}
```

The navbar has a search field. When the user types a query and presses Enter, we navigate to `/search?q=...` using SvelteKit's `goto`. The `encodeURIComponent` makes sure special characters in the query don't break the URL — for example, a query like `Harry & Hermione` becomes `Harry%20%26%20Hermione`.

Notice we use `trim()` to remove leading/trailing whitespace, and we check `e.key === 'Enter'` to only trigger on Enter — not on every keystroke. We also check `searchQuery.trim()` to avoid navigating with an empty search. These small guards prevent confusing behavior.

> **Watch Out:** The `searchQuery` is local state — it's not shared with the search page. When the user navigates to `/search`, the search page reads its own `queryInput` from URL params. This means the navbar search field clears after navigation, which is fine — the user can always type a new query there. If you wanted the search field to persist across navigation, you'd need to lift state to a store or use SvelteKit's `page` state.

## The Template: Building the Topbar

Now let's look at the HTML structure:

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
      <a class="adv-link" href="/search" onclick={() => (activeTab = 'download')}>
        Advanced
      </a>
    </div>
  </header>
```

Let's walk through this piece by piece.

### The Brand

```svelte
<div class="brand">
  <span class="logo">📚</span>
  <span class="title">FicHub</span>
</div>
```

The leftmost element in the topbar. The 📚 emoji serves as the logo, and "FicHub" is the title. Using an emoji instead of an image file means we don't need to worry about image loading, alt text, vector formats, or retina displays. It's universally rendered, always crisp, and it gives the app a friendly, approachable feel.

### The Tab Bar

```svelte
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
```

The `{#each tabs as t}` loop renders one button per tab. Each button has several important attributes:

- **`class="tab"`** — the base CSS class for all tab buttons.
- **`class:active={activeTab === t.id}`** — Svelte's conditional class syntax. When `activeTab` matches this tab's ID, the `active` class is added. This is the mechanism that highlights the currently selected tab.
- **`onclick={() => (activeTab = t.id)}`** — clicking the button sets the active tab. The arrow function creates a closure that captures the current tab's `id`.
- **`role="tab"` and `aria-selected`** — accessibility attributes so screen readers understand these are tabs and which one is selected. This is not just nice to have — it's essential for users who rely on assistive technology.

Inside each button, we render both the icon and the label. The icon is always visible. The label gets hidden on mobile via CSS (we'll see that later), showing just the icons on small screens.

> **Watch Out:** The `class:active` syntax is Svelte-specific. It's syntactic sugar for `class:active={expression}` — when the expression is truthy, the class is added. You could also write `class={activeTab === t.id ? 'tab active' : 'tab'}`, but the Svelte way is cleaner and less error-prone. It also composes well — you can have multiple `class:name` directives on the same element.

### The Search Area

```svelte
<div class="search-area">
  <input
    class="nav-search"
    type="search"
    placeholder="Search…"
    bind:value={searchQuery}
    onkeydown={onSearchKeydown}
    aria-label="Search fanfiction"
  />
  <a class="adv-link" href="/search" onclick={() => (activeTab = 'download')}>
    Advanced
  </a>
</div>
```

The search area sits on the right side of the topbar (thanks to `margin-left: auto` in the CSS). It has a text input and a small "Advanced" link.

The `bind:value={searchQuery}` creates two-way binding — when the user types, `searchQuery` updates. When `searchQuery` changes programmatically, the input updates. This is one of Svelte's signature features: two-way binding without the boilerplate of event handlers and manual state updates.

The `type="search"` attribute gives us semantic HTML and a native clear button in some browsers. The `placeholder="Search…"` shows hint text that disappears when the user starts typing.

The "Advanced" link navigates to `/search`. The `onclick` handler also sets `activeTab = 'download'` so that when the user returns from the search page, the download tab is active. This prevents a confusing state where the user returns to a tab they didn't expect.

### The Main Content

```svelte
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

This is where the magic happens. The `{#if}/{:else if}` chain renders exactly one component based on `activeTab`. When the user clicks a different tab, `activeTab` changes, Svelte removes the old component and mounts the new one.

One important detail: each tab component is completely independent. `DownloadTab` doesn't know about `RecommendationsTab`, and neither knows about `SuggestionsTab`. They share the layout's topbar and footer, but they don't share any state with each other. This isolation makes each component easier to understand, test, and modify.

Another detail: the components are conditionally rendered, not hidden with CSS. When `activeTab` is `'download'`, the `RecommendationsTab` and `SuggestionsTab` components don't exist in the DOM at all. They're not just hidden — they're unmounted. This means they don't consume memory or run any JavaScript. When the user switches tabs, the old component is destroyed and the new one is created from scratch.

This is different from using `display: none` to hide tabs. With conditional rendering, switching tabs resets the component's internal state. If the user was scrolling through recommendations and switches to downloads, the recommendations scroll position is lost. For FicHub, this is fine — each tab is a fresh start. But if you needed to preserve scroll position across tab switches, you'd want to use CSS visibility instead.

Note that the root `+page.svelte` is intentionally empty — it contains only a comment explaining that the tabbed UI lives in the layout. This means the root page doesn't compete with the layout; the layout handles everything. The root page is just a placeholder that SvelteKit requires for the `/` route.

An alternative approach would be to use SvelteKit's file-based routing with separate pages for each tab (e.g., `/download`, `/recs`, `/sugg`). But for FicHub, tab-based UI within a single layout is simpler and faster — no page transitions, no URL changes, no loading states. The tabs just swap content instantly.

### The Footer

```svelte
  <footer class="footer muted">
    <span>FicHub — download & discover fanfiction.</span>
    <span class="sites">AO3 · FanFiction.net · FictionPress · Forums</span>
  </footer>
</div>
```

The footer is simple: a tagline on the left and a list of supported sites on the right. The `muted` class uses the `--color-muted` CSS variable for a lighter text color. The footer uses `justify-content: space-between` to push the two spans to opposite ends.

The footer is wrapped inside the `.app` container, which is a flex column. This is important for the layout's height behavior — the `.app` container stretches to at least the full viewport height, and the `main` element grows to fill the remaining space. This means the footer always sticks to the bottom, even when there's little content. No awkward gaps, no footer floating in the middle of the page.

## The CSS: Making It Beautiful

Now let's look at the styles that make this layout look polished:

```css
.app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}
```

The app container is a flex column that takes up at least the full viewport height. `min-height: 100vh` ensures it's at least as tall as the viewport. `flex-direction: column` stacks the topbar, main content, and footer vertically. Together with `flex: 1` on the `main` element (which we'll see in a moment), this creates a sticky footer layout — the footer always stays at the bottom.

### The Topbar

```css
.topbar {
  position: sticky;
  top: 0;
  z-index: 10;
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
  padding: 0.6rem 1rem;
  display: flex;
  align-items: center;
  gap: 1.2rem;
  flex-wrap: wrap;
}
```

`position: sticky` is the key. The topbar sticks to the top of the viewport when you scroll. `top: 0` means it sticks at the very top. `z-index: 10` ensures it stays above other content — without this, search results or cards might scroll over the topbar.

`background: var(--color-surface)` gives it a solid background so content scrolls *behind* it, not *through* it. `border-bottom` provides a subtle separator between the topbar and the content below.

`display: flex` with `align-items: center` creates a horizontal row with everything vertically centered. `gap: 1.2rem` adds consistent spacing between the brand, tabs, and search area. `flex-wrap: wrap` lets the topbar wrap onto multiple lines on very narrow screens — the tabs might wrap below the brand, and the search area might wrap below the tabs.

### Tab Styling

```css
.tab-bar {
  display: flex;
  gap: 0.3rem;
}
.tab {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  background: transparent;
  border: 1px solid transparent;
  color: var(--color-muted);
  padding: 0.45rem 0.9rem;
  border-radius: var(--radius-sm);
  font-weight: 600;
  font-size: 0.92rem;
  transition: all 0.15s;
}
.tab:hover {
  color: var(--color-text);
  background: var(--color-surface-2);
}
.tab.active {
  color: white;
  background: var(--color-primary);
}
```

The tab bar is a flex row with small gaps between tabs. Each inactive tab is transparent with muted text — it blends into the topbar. On hover, the tab gets a subtle background color and darker text, giving visual feedback.

The active tab gets a bold primary-color background with white text. This high contrast makes the selected tab immediately obvious. The `transition: all 0.15s` makes the hover and active transitions smooth — no jarring snap.

The `border: 1px solid transparent` on inactive tabs is a subtle trick: it ensures the tab doesn't "jump" when it gains a border on active state. The border is always there; it's just transparent when inactive. This maintains consistent sizing across all tabs.

### Search Area Positioning

```css
.search-area {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.nav-search {
  width: 180px;
  padding: 0.4rem 0.7rem;
  font-size: 0.88rem;
  border-radius: var(--radius-sm);
}
.adv-link {
  color: var(--color-muted);
  font-size: 0.82rem;
  white-space: nowrap;
}
.adv-link:hover {
  color: var(--color-text);
  text-decoration: none;
}
```

`margin-left: auto` is a classic CSS flexbox trick — in a flex row, applying `margin-left: auto` to the last item pushes it to the far right. This is how we get the search area aligned to the right without any absolute positioning or floats. It's elegant and responsive — the search area naturally stays on the right regardless of how wide the screen is.

The search input is 180 pixels wide — wide enough for typical queries but not so wide it dominates the topbar. The "Advanced" link is small and muted, sitting quietly next to the search field. It only highlights on hover, keeping the visual noise low.

### Responsive Design

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

On screens narrower than 700 pixels, we make two key changes:

1. **Hide tab labels** — only the emoji icons remain visible. This saves horizontal space on mobile. The icons are universally recognizable — ⬇ for download, ★ for recommendations, 💡 for suggestions — so the labels aren't essential.

2. **Full-width search area** — the search field takes the entire remaining width. `flex: 1` on the input makes it grow to fill the container. `margin-left: 0` overrides the desktop `margin-left: auto` so the search area doesn't try to push to the right of a non-existent space.

This means on a phone, the topbar shows: `📚 FicHub ⬇ ★ 💡 [Search field stretching to the right edge]`. Clean, compact, and functional.

> **Try It Yourself:** Open FicHub in your browser and shrink the window below 700px wide. Watch the tab labels disappear and the search field expand. Resize it back — the labels reappear. This is responsive design in action, and it's just 8 lines of CSS.

### The Footer CSS

```css
.footer {
  border-top: 1px solid var(--color-border);
  padding: 1rem;
  display: flex;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 0.5rem;
  font-size: 0.82rem;
}
```

The footer uses `justify-content: space-between` to push the tagline to the left and the sites list to the right. On very narrow screens, `flex-wrap: wrap` allows them to stack vertically. The small font size keeps the footer unobtrusive — it's there if you need it, but it doesn't compete with the main content.

### The Main Element

```css
main {
  flex: 1;
}
```

This single line is crucial. `flex: 1` tells the main element to grow and fill all available vertical space. Combined with the `.app` container's `min-height: 100vh`, this pushes the footer to the bottom of the viewport (or the bottom of the content, whichever is lower).

Without `flex: 1`, the main element would only be as tall as its content. If the content is short (like an empty download form), the footer would float up to the middle of the page. With `flex: 1`, the footer always stays at the bottom.

## Accessibility

A good layout isn't just about looks — it's about usability for everyone. The layout includes several accessibility features:

- **`role="tablist"`** on the tab bar tells screen readers "this is a group of tabs."
- **`role="tab"`** on each button tells screen readers "this is a tab."
- **`aria-selected`** tells screen readers which tab is currently active.
- **`aria-label="Search fanfiction"`** on the search input gives screen readers a description of what the input is for.
- **Semantic HTML** — `<header>`, `<nav>`, `<main>`, `<footer>` — gives the page structure that screen readers can navigate.

These attributes don't change the visual appearance at all. They're invisible to sighted users but essential for users who rely on screen readers. Good accessibility is good engineering.

### Keyboard Navigation

The layout also supports keyboard navigation. Users can:

1. **Tab through the topbar** — pressing Tab moves focus from the brand to the first tab, then to each subsequent tab, then to the search input.
2. **Activate tabs with Enter or Space** — once a tab has focus, pressing Enter or Space selects it.
3. **Search with Enter** — typing in the search field and pressing Enter navigates to the search page.

The `<button>` elements for tabs are natively focusable and activatable with keyboard. The `<a>` element for the Advanced link is also natively focusable. This means the entire topbar is usable without a mouse.

### Focus Management

SvelteKit handles focus management during navigation. When the user clicks a tab, focus stays on the clicked button. When the user navigates to the search page, focus moves to the search input. This prevents the confusing "focus is lost" state where the user has to click somewhere to regain keyboard control.

## Responsive Design Deep Dive

Let's take a closer look at how the responsive design works. The layout uses two breakpoints:

1. **700px** — the topbar breakpoint. Below this, tab labels hide and the search area goes full-width.
2. **600px** — the search page breakpoint. Below this, the filter grid collapses to one column and result cards stack vertically.

These breakpoints are chosen based on common device widths:

- **Phones** (320px–428px) — both breakpoints apply. Topbar shows icons only, search page shows stacked layouts.
- **Tablets** (768px–1024px) — neither breakpoint applies. Full layout with labels and two-column grids.
- **Desktops** (1200px+) — neither breakpoint applies. Full layout with generous spacing.

The breakpoints aren't arbitrary — they're based on where the layout starts to feel cramped. At 700px, the three tabs plus the search field plus the brand don't fit in one row without wrapping. At 600px, the two-column filter grid becomes too narrow for comfortable input sizing.

## The Full Layout: How It All Fits Together

Here's the complete flow when a user interacts with FicHub:

1. **The user loads the app.** SvelteKit renders `+layout.svelte`. The topbar appears with brand, tabs, and search field. The `{#if}` chain renders `DownloadTab` by default because `activeTab` starts as `'download'`.

2. **The user clicks "Recommendations."** The click handler sets `activeTab = 'recs'`. Svelte removes `DownloadTab` from the DOM and mounts `RecommendationsTab`. The URL doesn't change — this is a client-side state transition, not a navigation. The transition is instant.

3. **The user types in the navbar search.** They type `Harry Potter` and press Enter. The `onSearchKeydown` handler fires, encoding the query and calling `goto('/search?q=Harry%20Potter')`. SvelteKit's router kicks in — it unmounts the current page content and mounts the search page.

4. **The search page renders inside `<main>`.** It's the "children" of the layout. The layout's topbar and footer remain visible. The search page reads `q=Harry Potter` from the URL and auto-executes the search.

5. **The user clicks "Advanced" in the navbar.** This navigates to `/search` (no query). The search page renders with its full filter UI, ready for the user to build a complex search.

6. **The user clicks a result's "Download" button.** This navigates to `/?q=...` with the fic's URL. The layout renders the `DownloadTab` (since we set `activeTab = 'download'` in the Advanced link's onclick).

The layout is the constant. The tabs and search page are the variables. This separation of concerns is what makes SvelteKit layouts so powerful — you write the shell once, and every page benefits from it.

---

# Chapter 26: The Advanced Search Page

## The Search Route

When a user clicks "Advanced" in the navbar (or searches from the navbar and gets redirected), they land on the `/search` route. This route has two files:

- **`+page.ts`** — the page loader that reads URL parameters.
- **`+page.svelte`** — the page component with the search UI.

Together, they create a self-contained search experience. The loader handles data fetching (reading URL params), and the component handles presentation (rendering the UI and handling interactions).

### The Page Loader

```typescript
import type { PageLoad } from './$types';

// Read URL search params and pass them to the component.
export const load: PageLoad = async ({ url }) => {
  return {
    q: url.searchParams.get('q') ?? '',
    tab: url.searchParams.get('tab') ?? 'work',
  };
};
```

This is beautifully simple. The loader reads two URL parameters: `q` (the search query) and `tab` (which search tab to show). If they're not provided, it defaults to an empty string and `'work'` respectively.

The `load` function runs before the page component renders. It passes its return value as the `data` prop to `+page.svelte`. This means the component always knows what URL parameters the user arrived with.

The `PageLoad` type comes from SvelteKit's generated `$types` module. It ensures the return type matches what the component expects. If you change the return shape, TypeScript will tell you if the component needs updating.

> **Why use a loader instead of reading `window.location` directly?** Because SvelteKit's loader runs both on the server (during SSR) and on the client (during navigation). If you used `window.location`, it would break during server-side rendering. The loader pattern is the portable, SvelteKit-approved way to access URL data. Even though FicHub runs in SPA mode (`export const ssr = false`), using the loader keeps the code consistent and future-proof.

### SPA Mode

FicHub runs in SPA (Single Page Application) mode. The root `+layout.ts` file sets:

```typescript
// SPA mode: no SSR, no prerender. The backend serves the static build.
export const ssr = false;
export const prerender = false;
```

This means SvelteKit doesn't do server-side rendering — the Rust backend serves the static build, and the browser handles all routing. This simplifies deployment (no Node.js server needed) and means the `load` function always runs in the browser.

## The Page Component Structure

Now let's look at the search page itself. It's a substantial component — about 540 lines — so we'll walk through it in sections.

### Script Section: Imports and Types

```svelte
<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { search } from '$lib/api/search';
  import type { SearchFilters, SearchResult, SearchResponse } from '$lib/api/search';
  import {
    SORT_OPTIONS,
    COMPLETE_OPTIONS,
    SOURCE_OPTIONS,
    defaultFilters,
  } from '$lib/api/search';
  import { parseSearchQuery } from '$lib/search/syntax';
  import {
    formatWords,
    relativeTime,
    detectSite,
    stripHtml,
  } from '$lib/util';

  let { data } = $props();
```

The imports bring in everything we need:

- **`page`** from SvelteKit's app state — gives us access to the current page URL and params.
- **`goto`** for client-side navigation.
- **`search`** — our API client function that hits the backend at `/api/v0/search`.
- **Types** — `SearchFilters`, `SearchResult`, `SearchResponse` for TypeScript safety. These types mirror the backend's response format.
- **Constants** — `SORT_OPTIONS`, `COMPLETE_OPTIONS`, `SOURCE_OPTIONS` are arrays of `{value, label}` objects for dropdown menus. They're defined once in the search API module and shared across components.
- **`defaultFilters`** — returns a fresh `SearchFilters` object with all defaults (empty strings, null values, page 1).
- **`parseSearchQuery`** — the syntax parser that converts query strings like `fandom:Harry Potter words:>10000` into structured `SearchFilters`.
- **Utility functions** — `formatWords` adds commas to numbers, `relativeTime` shows "3 days ago," `detectSite` identifies the fanfiction platform, and `stripHtml` removes HTML tags from summaries.

The `{ data }` prop comes from the page loader. It contains `q` and `tab` from the URL.

### Search Tab State

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

The search page has its own tab system — separate from the layout's tabs. This is a common pattern: the layout handles app-level navigation (Download/Recs/Sugg), while individual pages handle their own sub-navigation.

The `activeTab` is initialized from the URL parameter (`data.tab`), so if the user arrives with `?tab=tag`, the tag tab is pre-selected. If no tab is specified, it defaults to `'work'`.

The `searchTabs` array follows the same data-driven pattern as the layout's tabs: an array of objects that the template loops over. This consistency makes the codebase predictable — every tab system in the app works the same way.

### Query and Filter State

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

These variables hold the entire search state:

- **`queryInput`** — the raw text the user types in the search box. This might contain syntax like `fandom:Harry Potter words:>10000`. It's initialized from the URL's `q` parameter.
- **`filters`** — the parsed `SearchFilters` object sent to the API. This is what the backend actually receives. Updated whenever the user searches.
- **`loading`** — true while a search request is in flight. Used to show a spinner and disable the Search button.
- **`error`** — error message if the search fails. Empty string when there's no error.
- **`results`** — the array of search results from the API. Each result has a title, author, tags, word count, and more.
- **`total`** — total number of matching results (for pagination). The API returns this even though we only fetch 20 results per page.
- **`currentPage`** — which page of results we're on. Starts at 1.
- **`searched`** — whether the user has performed at least one search. This prevents showing "No results found" before the user has even searched.

### Advanced Filter Fields

```svelte
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

These are the individual filter fields shown in the Work Search tab. Each one maps to a specific filter in the `SearchFilters` object. When the user selects "Complete Only" from the dropdown, `filterComplete` becomes `'true'`. When the user types "5000" in Min Words, `filterMinWords` becomes `'5000'`.

All filters start as empty strings, meaning "not set." In `doSearch()`, empty filter fields defer to the syntax-parsed values. This lets users combine the query syntax with the form UI seamlessly.

### Auto-Search on Mount

```svelte
  let initialized = $state(false);

  $effect(() => {
    if (data.q && !initialized) {
      initialized = true;
      queryInput = data.q;
      doSearch();
    }
  });
```

This is a clever pattern. If the user arrives with a query in the URL (e.g., from the navbar search), we automatically execute the search on mount. The `initialized` guard ensures we only auto-search once — not every time the effect re-runs.

The `$effect` is Svelte 5's reactivity primitive. It runs whenever its dependencies change. Since it reads `data.q`, it runs when the component mounts (or when `data` changes during navigation). Without the `initialized` guard, the effect would run again every time `data` changed, potentially causing an infinite search loop.

> **Try It Yourself:** Open a new tab and go to `http://localhost:5173/search?q=Harry+Potter`. The search executes automatically — you don't need to click the Search button. That's the auto-search on mount doing its job. The `initialized` flag prevents it from running again if you navigate away and come back.

## The doSearch Function

This is the heart of the search page. When called, it:

1. Sets loading state and clears errors.
2. Parses the query syntax into filters.
3. Merges with advanced filter fields.
4. Updates the URL to reflect the current search.
5. Calls the search API.
6. Updates the result state with the response.

Let's look at it piece by piece:

```svelte
  async function doSearch() {
    loading = true;
    error = '';
    searched = true;
```

The function starts by setting `loading` to true (which shows a spinner), clearing any previous error, and marking that a search has been performed.

```typescript
    // Parse syntax from query input.
    const syntaxFilters = parseSearchQuery(queryInput);
```

This calls our syntax parser (from `$lib/search/syntax`). It converts a query like `fandom:Harry Potter words:>10000 complete:true` into a `SearchFilters` object with `include_tags: '1:Harry Potter'`, `min_words: 10000`, and `complete: true`. We covered the syntax parser in detail in earlier chapters — this is where it gets used.

```typescript
    // Merge with advanced filter fields (advanced overrides syntax).
    filters = {
      ...syntaxFilters,
      q: filterComplete === '' ? syntaxFilters.q : syntaxFilters.q,
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
```

This is the merge step. The spread operator `...syntaxFilters` copies all the parsed syntax values as the base. Then each advanced filter field overrides its corresponding value — *but only if the user filled it in*.

For example, if the user typed `words:>10000` in the search bar AND typed "5000" in the Min Words field, the form field wins (5000). If the Min Words field is empty, the syntax value wins (10000).

The pattern is consistent: `filterField ? Number(filterField) : syntaxFilters.field` for numbers, and `filterField || syntaxFilters.field` for strings. The `||` operator returns the left side if it's truthy (non-empty string), otherwise the right side.

This means the user can use both the query syntax AND the filter dropdowns simultaneously. The form fields always win when set. The syntax provides the "power user" path, and the form provides the "visual" path.

> **Watch Out:** The `q` line is interesting: `q: filterComplete === '' ? syntaxFilters.q : syntaxFilters.q`. This looks like a no-op — it's the same either way! That's intentional: the `q` field (the free-text search term) always comes from the syntax parser. The advanced form doesn't have its own text search field for the free-text query; it relies on the query input for that. The form fields only control filters like completion status, word count, and tags. The free-text query always flows through the syntax parser.

### URL Sync

```typescript
    // Update URL without navigation.
    const qs = new URLSearchParams();
    if (queryInput) qs.set('q', queryInput);
    if (activeTab !== 'work') qs.set('tab', activeTab);
    goto(`/search?${qs.toString()}`, { replaceState: true, keepFocus: true });
```

After parsing the query, we update the browser URL to reflect the current search. This is important for two reasons:

1. **Bookmarkability** — the user can bookmark the search URL and come back to it later.
2. **Shareability** — the user can copy the URL and share it with someone else.

The `replaceState: true` option means we use `history.replaceState` instead of `history.pushState` — so the URL updates without creating a new entry in the browser history. The user can still hit Back to leave the search page entirely, but pressing Back within a search doesn't create a chain of identical URLs.

`keepFocus: true` ensures the search input stays focused after the URL update, so the user doesn't lose their typing position. This is a small UX detail that makes a big difference — without it, the user would have to click back into the input after every search.

### The API Call

```typescript
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
```

The `search()` function (from `$lib/api/search`) builds a query string from the filters and fetches `/api/v0/search`. The backend parses the filters, queries the database (using Tantivy for full-text search and PostgreSQL for structured queries), and returns the results.

The `try/catch` handles network errors, server errors, and unexpected responses. The `finally` block always sets `loading = false`, even if the request fails. This ensures the spinner always stops, even on error.

The `instanceof Error` check ensures we get a meaningful error message. If the error is a standard `Error` object, we use its `.message`. Otherwise, we fall back to a generic "Search failed" message.

### Pagination Helpers

```svelte
  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      currentPage = 1;
      doSearch();
    }
  }

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

When the user presses Enter in the search box, we reset to page 1 and search — pressing Enter always starts a new search from page 1. Next/Previous buttons increment/decrement the page number and re-search. The `prevPage` function has a guard to prevent going below page 1.

`totalPages` is a derived value — Svelte 5's `$derived` automatically recomputes it whenever `total` changes. It divides the total result count by 20 (the results per page) and rounds up with `Math.ceil`. So if there are 45 results, `totalPages` is 3 (45/20 = 2.25, rounded up to 3).

## The Template: Query Row and Tabs

```svelte
<div class="search-page">
  <h1>Advanced Search</h1>

  <div class="query-row">
    <input
      type="search"
      class="query-input"
      placeholder='Try: fandom:"Harry Potter" tag:Fluff complete:true sort:updated'
      bind:value={queryInput}
      onkeydown={onKeydown}
      aria-label="Search query"
    />
    <button class="btn" onclick={() => { currentPage = 1; doSearch(); }} disabled={loading}>
      {#if loading}<span class="spinner"></span> Searching…{:else}Search{/if}
    </button>
    <a class="syntax-link muted" href="/search/syntax" target="_blank">Syntax guide</a>
  </div>
```

The query row has three elements side by side:

1. **The search input** — a wide text field with a carefully chosen placeholder. The placeholder `fandom:"Harry Potter" tag:Fluff complete:true sort:updated` demonstrates four different syntax features in one query: a tag filter, a freeform filter, a boolean filter, and a sort option. It's an advertisement for the syntax system.

2. **The Search button** — shows a spinner while loading, and is disabled during search to prevent double-clicks. The `{#if}` block switches between "Searching..." with a spinner and a plain "Search" label.

3. **The Syntax guide link** — opens the syntax guide page in a new tab. We'll cover that page in Chapter 28. The `target="_blank"` means the user doesn't lose their current search.

### The Search Tabs

```svelte
  <div class="search-tabs">
    {#each searchTabs as t}
      <button
        class="stab"
        class:active={activeTab === t.id}
        onclick={() => { activeTab = t.id; }}
      >
        {t.label}
      </button>
    {/each}
  </div>
```

Similar to the layout's tab bar, but these are the search-specific tabs. The CSS class is `stab` (short for "search tab") to avoid conflicts with the layout's `.tab` class. Each tab switches the filter panel below it.

## Understanding Search Results

Before we look at how results are displayed, let's understand what a search result looks like. The `SearchResult` type defines the data structure for each fic:

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

Each field tells a story:

- **`url_id`** — the unique identifier for this fic. Used as a React/Svelte key for efficient rendering.
- **`title`** — the fic's title, exactly as stored in the database.
- **`author`** — the author's name.
- **`source`** — the full URL to the original fic on its host site.
- **`words`** — the word count as a number (not formatted — formatting happens in the template with `formatWords`).
- **`chapters`** — the chapter count.
- **`status`** — "Complete" or "In Progress."
- **`description`** — the fic's summary, which may contain HTML tags.
- **`updated`** — ISO timestamp of the last update, or `null` if unknown.
- **`rank`** — the relevance score from Tantivy, or `null` if not applicable.
- **`tags`** — an array of `SearchTag` objects attached to this fic.
- **`total_freeform`** — the total number of freeform tags (we only display the first 12).

Each tag in the `tags` array has its own structure:

```typescript
export interface SearchTag {
  name: string;
  type: string;
  type_id: number;
  score: number;
}
```

The `type` is a human-readable name like "Fandom" or "Freeform." The `type_id` is the numeric ID (1–4). The `score` is how relevant this tag is to the search query — higher means more relevant.

## Results Display

### The Results Header

```svelte
  {#if results.length > 0}
    <div class="results-header">
      <span class="muted">{total.toLocaleString()} results</span>
      <span class="muted">Page {currentPage} of {totalPages}</span>
    </div>
```

A simple header showing the total count and current page. `toLocaleString()` adds commas to large numbers (e.g., 12,345). The muted class keeps it subtle — it's informational, not the main attraction.

### Result Cards

```svelte
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
```

Each result card shows:

- **Title** — linked to the original fic URL, opening in a new tab. The `rel="noopener"` prevents the new tab from having access to the original page's `window` object (a security measure).
- **Author and site** — the author name and a detected site badge (AO3, FF.net, etc.). The `detectSite` utility inspects the URL to identify the platform.
- **Metadata** — word count (formatted with commas), chapter count, and completion status. This gives users a quick overview without clicking into the fic.
- **Description** — the first 250 characters of the summary, with HTML stripped. The `stripHtml` function removes `<p>`, `<br>`, and other tags, converting HTML entities like `&amp;` to their plain-text equivalents.

The `{#each results as r (r.url_id)}` uses the `url_id` as a key. This tells Svelte how to identify each result card, which helps with efficient DOM updates when results change.

### Tag Pills

```svelte
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
```

We show up to 12 tags per result. If there are more, we display a "+N more" indicator. Each tag pill shows the tag name and has a tooltip (via `title` attribute) showing the tag type and relevance score.

The `total_freeform` field tells us how many freeform tags the fic has in total — not just the 12 we're showing. This lets us display an accurate count of hidden tags.

> **Why limit to 12 tags?** Some fics have dozens of freeform tags. Showing all of them would make each result card take up the entire screen. The 12-tag limit keeps results scannable while still showing the most relevant tags. Users who need more detail can click through to the original fic.

### The Right Sidebar

```svelte
          <div class="result-meta">
            {#if r.rank}
              <span class="rank">#{Math.round(r.rank * 10) / 10}</span>
            {/if}
            {#if r.updated}
              <span class="muted">{relativeTime(r.updated)}</span>
            {/if}
            <a class="btn btn-secondary sm" href="/?q={encodeURIComponent(r.source)}">
              Download
            </a>
          </div>
```

On the right side of each result card, we show:

- **Search rank** — how well the fic matched the query, rounded to one decimal place. A lower rank means a better match. The `Math.round(r.rank * 10) / 10` formula rounds to one decimal — for example, 3.76 becomes 3.8.
- **Relative update time** — "3 days ago," "2 weeks ago," etc. The `relativeTime` function calculates this from the ISO timestamp.
- **Download button** — a small secondary button that navigates to the home page with the fic's URL pre-filled. The user can then choose their export format and download.

### Pagination Controls

```svelte
    <div class="pagination">
      <button class="btn btn-secondary" onclick={prevPage} disabled={currentPage <= 1}>
        ← Previous
      </button>
      <span class="muted">Page {currentPage} of {totalPages}</span>
      <button class="btn btn-secondary" onclick={nextPage} disabled={currentPage >= totalPages}>
        Next →
      </button>
    </div>
  {/if}
```

Simple Previous/Next buttons with the current page indicator. The buttons are disabled when there are no more pages to go to — Previous is disabled on page 1, and Next is disabled on the last page.

> **Try It Yourself:** Search for `fandom:Harry Potter complete:true` and look at the results. Click "Next" to page 2. Notice the URL updates with the new page number. Now hit your browser's Back button — you go back to page 1. Hit Back again — you leave the search page entirely. That's the `replaceState` option working correctly: it creates one history entry for the search, not one per page.

---

# Chapter 27: Search Filters and Results

## The Work Search Filter Grid

Below the search tabs, the Work Search tab shows a grid of filter controls. This is where users who prefer point-and-click over query syntax can build their searches visually.

### The Complete Filter Grid

```svelte
  {#if activeTab === 'work'}
    <div class="filters card">
      <div class="filter-grid">
        <label class="filter-item full">
          Title / Any Field
          <input type="text" bind:value={queryInput}
            placeholder="Search in title or any field…" />
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

        <label class="filter-item">
          Min Chapters
          <input type="number" bind:value={filterMinChapters} placeholder="0" min="1" />
        </label>

        <label class="filter-item">
          Max Chapters
          <input type="number" bind:value={filterMaxChapters} placeholder="∞" min="1" />
        </label>

        <label class="filter-item">
          Date After
          <input type="date" bind:value={filterDateFrom} />
        </label>

        <label class="filter-item">
          Date Before
          <input type="date" bind:value={filterDateTo} />
        </label>

        <label class="filter-item full">
          Include Tags (type_id:name format)
          <input type="text" bind:value={filterIncludeTags}
            placeholder="1:Harry Potter,4:Fluff" />
        </label>

        <label class="filter-item full">
          Exclude Tags
          <input type="text" bind:value={filterExcludeTags}
            placeholder="4:Major Character Death" />
        </label>
      </div>
    </div>
  {/if}
```

Let's break down each filter and explain what it does and how it connects to the backend.

### Completion Status

```svelte
<label class="filter-item">
  Completion Status
  <select bind:value={filterComplete}>
    {#each COMPLETE_OPTIONS as opt}
      <option value={opt.value}>{opt.label}</option>
    {/each}
  </select>
</label>
```

The `COMPLETE_OPTIONS` array (from `$lib/api/search`) defines three choices:

| Value | Label |
|-------|-------|
| `''` | All Works |
| `'true'` | Complete Only |
| `'false'` | In Progress Only |

When the user selects "Complete Only," `filterComplete` becomes `'true'`. In `doSearch()`, this gets converted to the boolean `true` in the `SearchFilters` object. The backend then adds `WHERE completed = true` to its SQL query.

The dropdown uses `bind:value` for two-way binding — when the user selects an option, the state updates immediately. No click handlers needed. This is one of Svelte's most loved features — form elements just work with minimal boilerplate.

### Site Filter

```svelte
<label class="filter-item">
  Site
  <select bind:value={filterSource}>
    {#each SOURCE_OPTIONS as opt}
      <option value={opt.value}>{opt.label}</option>
    {/each}
  </select>
</label>
```

The `SOURCE_OPTIONS` array maps site abbreviations to their full domain names:

| Value | Label |
|-------|-------|
| `''` | All Sites |
| `'archiveofourown.org'` | Archive of Our Own |
| `'fanfiction.net'` | FanFiction.net |
| `'fictionpress.com'` | FictionPress |
| `'forums.spacebattles.com'` | SpaceBattles |
| `'forums.sufficientvelocity.com'` | SufficientVelocity |

The backend queries work with the full domain names, so the filter value is the domain, not the abbreviation. This keeps the API clean — it doesn't need to know about abbreviations or nicknames. The frontend handles the translation from human-friendly labels to machine-friendly domain names.

### Sort Options

```svelte
<label class="filter-item">
  Sort By
  <select bind:value={filterSort}>
    {#each SORT_OPTIONS as opt}
      <option value={opt.value}>{opt.label}</option>
    {/each}
  </select>
</label>
```

The `SORT_OPTIONS` array:

| Value | Label |
|-------|-------|
| `''` | Relevance |
| `'updated'` | Date Updated |
| `'created'` | Date Published |
| `'words'` | Word Count |
| `'kudos'` | Kudos Count |

An empty string means "sort by relevance" — the default. The backend uses Tantivy's relevance scoring when no explicit sort is specified. Relevance is based on how well the fic matches the search terms — fics with more matching terms and higher term frequency rank higher.

### Word Count Range

```svelte
<label class="filter-item">
  Min Words
  <input type="number" bind:value={filterMinWords} placeholder="0" min="0" />
</label>

<label class="filter-item">
  Max Words
  <input type="number" bind:value={filterMaxWords} placeholder="∞" min="0" />
</label>
```

Two number inputs for the word count range. The placeholder "∞" suggests "no limit" when the field is empty. In `doSearch()`, empty fields are treated as `null` (no constraint).

The `type="number"` attribute gives us a numeric input with increment/decrement buttons in some browsers. The `min="0"` attribute prevents negative numbers. You can leave either field blank — an empty Min Words means "no minimum," and an empty Max Words means "no maximum."

> **Try It Yourself:** Leave Min Words empty and set Max Words to 1000. Search for `fandom:Harry Potter`. You'll see only short fics — one-shots and flash fiction. Now set Min Words to 50000 and clear Max Words. You'll see only epic-length fics. The word count filters are great for finding fics that match your reading time.

### Chapter Range

```svelte
<label class="filter-item">
  Min Chapters
  <input type="number" bind:value={filterMinChapters} placeholder="0" min="1" />
</label>

<label class="filter-item">
  Max Chapters
  <input type="number" bind:value={filterMaxChapters} placeholder="∞" min="1" />
</label>
```

Same pattern as word count, but for chapters. Note `min="1"` on the inputs — you can't have zero chapters (a fic with no chapters isn't a fic). The chapter filter is useful for finding multi-chapter epics or single-chapter one-shots.

### Date Range

```svelte
<label class="filter-item">
  Date After
  <input type="date" bind:value={filterDateFrom} />
</label>

<label class="filter-item">
  Date Before
  <input type="date" bind:value={filterDateTo} />
</label>
```

The `<input type="date">` gives us a native date picker. The browser shows a calendar widget when the user clicks the field. The value is stored as a `YYYY-MM-DD` string.

The `date_from` and `date_to` fields in `SearchFilters` accept ISO timestamps. In `doSearch()`, we pass them directly — the backend handles the date parsing. If you need more precise control (e.g., "after January 2024"), you can use the query syntax: `after:2024-01`.

### Include/Exclude Tags

```svelte
<label class="filter-item full">
  Include Tags (type_id:name format)
  <input type="text" bind:value={filterIncludeTags}
    placeholder="1:Harry Potter,4:Fluff" />
</label>

<label class="filter-item full">
  Exclude Tags
  <input type="text" bind:value={filterExcludeTags}
    placeholder="4:Major Character Death" />
</label>
```

These text fields use the `type_id:name` format. The type IDs are:

| ID | Type |
|----|------|
| 1 | Fandom |
| 2 | Character |
| 3 | Relationship |
| 4 | Freeform |

So `1:Harry Potter` means "include the Harry Potter fandom tag," and `4:Fluff` means "include the Fluff freeform tag." Multiple tags are comma-separated: `1:Harry Potter,4:Fluff` means "fics in the Harry Potter fandom tagged Fluff."

> **Watch Out:** The tag format is technical. Regular users will probably use the query syntax (`fandom:Harry Potter tag:Fluff`) instead. These fields are more useful for developers testing specific filter combinations, or for users who want to paste in a complex tag filter. The syntax guide (Chapter 28) teaches the friendlier syntax.

### The Grid Layout

```css
.filter-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.8rem;
}
.filter-item {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  font-size: 0.85rem;
  color: var(--color-muted);
}
.filter-item.full {
  grid-column: 1 / -1;
}
.filter-item select,
.filter-item input {
  font-size: 0.88rem;
}
```

The filter grid is a two-column CSS Grid. Each `filter-item` is a label with a child input/select, stacked vertically with `flex-direction: column`. The label text is muted, and the input/select below it is slightly larger.

Items with the `full` class span both columns (`grid-column: 1 / -1`). This is used for the title field (which needs width) and the tag fields (which can be long strings). This pattern — full-width items mixed with half-width items — is common in form layouts.

### The SearchFilters Type

Behind every filter is the `SearchFilters` type, defined in `$lib/api/search.ts`:

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

Every filter in the UI maps to a field in this type:

- **`q`** — the free-text search query. Comes from the query input or the syntax parser.
- **`include_tags`** — comma-separated `type_id:name` pairs for tags to include.
- **`exclude_tags`** — comma-separated `type_id:name` pairs for tags to exclude.
- **`min_words` / `max_words`** — word count range. `null` means no constraint.
- **`min_chapters` / `max_chapters`** — chapter count range. `null` means no constraint.
- **`complete`** — `true` for completed only, `false` for in-progress only, `null` for all.
- **`source`** — the site domain. Empty string means all sites.
- **`date_from` / `date_to`** — ISO timestamp strings. Empty means no constraint.
- **`sort`** — sort order. Empty string means relevance.
- **`page` / `per_page`** — pagination. Default is page 1, 20 results per page.

The `defaultFilters()` function returns a fresh instance with all defaults:

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

This is the starting point for every search. The syntax parser and form fields fill in the values, and the `search()` function sends them to the backend.

### The buildSearchQuery Function

The `buildSearchQuery` function converts a `SearchFilters` object into a URL query string:

```typescript
export function buildSearchQuery(filters: SearchFilters): string {
  const params = new URLSearchParams();
  if (filters.q) params.set('q', filters.q);
  if (filters.include_tags) params.set('include_tags', filters.include_tags);
  if (filters.exclude_tags) params.set('exclude_tags', filters.exclude_tags);
  if (filters.min_words !== null) params.set('min_words', String(filters.min_words));
  // ... etc for all fields
  if (filters.page > 1) params.set('page', String(filters.page));
  if (filters.per_page !== 20) params.set('per_page', String(filters.per_page));
  return params.toString();
}
```

Notice that it only includes non-default values. If `min_words` is `null`, it's not added to the query string. If `page` is 1 (the default), it's not included. This keeps the URL clean — no unnecessary parameters.

The `search()` function then uses this query string to fetch `/api/v0/search`:

```typescript
export async function search(filters: SearchFilters): Promise<SearchResponse> {
  const qs = buildSearchQuery(filters);
  const res = await fetch(`/api/v0/search${qs ? '?' + qs : ''}`);
  if (!res.ok) throw new Error(`Search failed (${res.status})`);
  return (await res.json()) as SearchResponse;
}
```

The `SearchResponse` type contains the results, total count, and pagination info:

```typescript
export interface SearchResponse {
  total: number;
  page: number;
  per_page: number;
  results: SearchResult[];
}
```

On mobile, the grid collapses to one column:

```css
@media (max-width: 600px) {
  .filter-grid {
    grid-template-columns: 1fr;
  }
}
```

## The Placeholder Tabs

FicHub's search supports four types of searches, but only Work Search has a fully functional filter UI. The other three tabs show placeholder forms with a "coming soon" message.

### People Search

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

The People Search tab shows two input fields (author name and fandom) plus a message saying it's coming soon. These fields aren't wired up yet — they're placeholders for a future feature.

The hint "use syntax: author:Name" tells users they can achieve the same result today by using the query syntax. The backend already supports author searches — only the form UI is missing.

### Bookmark Search

```svelte
  {#if activeTab === 'bookmark'}
    <div class="filters card">
      <div class="filter-grid">
        <label class="filter-item full">
          Bookmarked Item
          <input type="text" placeholder="Title or author of bookmarked fic…" />
        </label>
        <label class="filter-item">
          Word Count
          <input type="text" placeholder="e.g. >10000" />
        </label>
        <label class="filter-item">
          Has Rec
          <select>
            <option value="">Any</option>
            <option value="true">Rec only</option>
          </select>
        </label>
      </div>
      <p class="muted">Bookmark search coming soon — use syntax: words:>10000</p>
    </div>
  {/if}
```

Bookmark Search lets you search through your bookmarked fics. Like People Search, it's a placeholder. The form shows fields for the bookmarked item name, word count range, and whether it's a recommendation.

### Tag Search

```svelte
  {#if activeTab === 'tag'}
    <div class="filters card">
      <div class="filter-grid">
        <label class="filter-item full">
          Tag Name
          <input type="text" placeholder="Search tags…" />
        </label>
        <label class="filter-item">
          Tag Type
          <select>
            <option value="">Any</option>
            <option value="1">Fandom</option>
            <option value="2">Character</option>
            <option value="3">Relationship</option>
            <option value="4">Freeform</option>
          </select>
        </label>
      </div>
      <p class="muted">Tag search coming soon — use syntax: fandom:Harry Potter</p>
    </div>
  {/if}
```

Tag Search lets you search the tag database directly — find tags by name, filter by type (fandom, character, relationship, freeform). Again, a placeholder with a syntax hint.

> **Why placeholders?** Building the full UI for every search type takes time. The backend supports all these searches through the query syntax (`author:Name`, `words:>10000`, `fandom:Harry Potter`). The placeholders show users what's coming while the syntax gives them a way to use these features now. It's a pragmatic approach — ship what works, plan what's next. The placeholders also serve as a design sketch — they show what the form will look like when it's built. And they give users something to click on, which is better than a blank page with "Coming Soon."

## Error and Empty States

```svelte
  {#if error}
    <div class="card error-card"><strong class="error-text">⚠️ {error}</strong></div>
  {/if}

  {#if searched && !loading && results.length === 0 && !error}
    <div class="card empty">
      <p class="muted">No results found. Try different keywords or filters.</p>
    </div>
  {/if}
```

Two important states:

1. **Error** — shown when the API call fails. The error message is displayed in a red-bordered card with a warning emoji. The `error-card` class adds a red border using `var(--color-error)`.

2. **Empty** — shown only after a search completes with zero results. The `searched && !loading` guard ensures we don't show "No results" before the first search or while loading. The `!error` guard ensures we don't show both the error and empty state simultaneously.

The empty state message is helpful: "Try different keywords or filters" — it gives the user a concrete next step instead of just saying "nothing found." Good error and empty states are a mark of polished UX.

## The Results CSS

```css
.results {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}
.result-card {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
}
.result-card h3 {
  margin: 0 0 0.3rem;
  font-size: 1.1rem;
}
.result-card h3 a {
  color: var(--color-text);
}
.meta-line {
  margin: 0.3rem 0;
  font-size: 0.9rem;
}
.desc {
  color: var(--color-muted);
  font-size: 0.85rem;
}
.tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem;
  margin-top: 0.5rem;
}
.tag-pill {
  background: var(--color-surface-2);
  border: 1px solid var(--color-border);
  border-radius: 999px;
  padding: 0.1rem 0.5rem;
  font-size: 0.75rem;
  color: var(--color-muted);
}
.result-meta {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 0.3rem;
  flex-shrink: 0;
}
.rank {
  font-weight: 700;
  font-size: 0.9rem;
  color: var(--color-primary);
}
```

The results list is a flex column with cards stacked vertically. Each card is a flex row with the main content on the left and metadata (rank, time, download button) on the right.

The tag pills use `border-radius: 999px` — a huge border radius that makes any rectangle into a pill shape. This is a common CSS trick for rounded badges. The pills have a subtle background and border, making them look like clickable chips.

The `flex-shrink: 0` on `.result-meta` prevents the right column from shrinking when the left column's content is wide. Without this, a long title could squeeze the download button into invisibility. This is a common flex layout gotcha — always set `flex-shrink: 0` on fixed-width elements.

The `.rank` class uses the primary color to make the rank number stand out. This is the search engine's relevance score — it tells users how well the fic matches their query. A rank of `#1.2` means it's a very strong match; `#8.5` means it's a weaker match.

### Mobile Responsive

```css
@media (max-width: 600px) {
  .query-row {
    flex-direction: column;
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

On mobile, the result card stacks vertically (title on top, metadata below), and the query row stacks too (search input above the button). The result metadata switches to a horizontal layout so the rank, time, and download button sit in a row instead of stacking.

---

# Chapter 28: The Syntax Guide

## Why a Syntax Guide?

The query syntax is FicHub's power feature. It lets you express complex search queries in a single line — queries that would take many clicks with dropdown menus alone. But here's the thing: **nobody knows a syntax they haven't been taught**.

Think about it: if you've never seen `fandom:Harry Potter words:>10000`, you'd have no idea that was a valid query. You'd use the dropdown menus, which are fine for simple searches but tedious for complex ones. The syntax guide bridges the gap between "I know what I want" and "I know how to ask for it."

The syntax guide exists to:

1. **Teach users the keywords** — what `fandom:`, `words:`, `complete:` mean.
2. **Show the short aliases** — `f:`, `w:`, `s:` for power users.
3. **Demonstrate combinations** — how to chain multiple filters in one query.
4. **Provide examples** — real queries that users can copy and modify.
5. **Reduce support requests** — instead of asking "how do I search for completed fics?", users can check the guide.

The guide lives at `/search/syntax` and is linked from the search page's "Syntax guide" link. It opens in a new tab, so users don't lose their search.

## The Guide Page Structure

The syntax guide is a static page — no dynamic data, no API calls. It's pure HTML and CSS. This makes it fast to load and easy to maintain. No JavaScript means no loading states, no error states, no interactivity that could break.

```svelte
<svelte:head>
  <title>Search Syntax Guide — FicHub</title>
</svelte:head>

<div class="syntax-page">
  <h1>Search Syntax Guide</h1>
  <p class="subtitle">
    FicHub's search bar supports a powerful query syntax.
    Mix and match these keywords to find exactly what you're looking for.
  </p>
```

The `<svelte:head>` block sets the page title — this appears in the browser tab and search engine results. The subtitle sets the tone: this is a friendly guide, not a dry reference manual.

## Full-Text Search

The first section covers the basics — searching by words:

```svelte
  <section>
    <h2>Full-Text Search</h2>
    <p>Type any words to search across titles, summaries, and tags.</p>

    <table class="syntax-table">
      <tr>
        <td class="code">Harry Potter</td>
        <td>Finds fics mentioning "Harry Potter" in any field</td>
      </tr>
      <tr>
        <td class="code">title:Dragon</td>
        <td>Searches only in the title</td>
      </tr>
      <tr>
        <td class="code">author:cleo</td>
        <td>Searches only in the author name</td>
      </tr>
      <tr>
        <td class="code">author:cleo fandom:Harry Potter</td>
        <td>Author search combined with fandom filter</td>
      </tr>
    </table>
  </section>
```

The table format is consistent throughout the guide: code on the left, description on the right. This makes it easy to scan — you can visually separate "what to type" from "what it does."

### How It Works Under the Hood

When you type `Harry Potter` (bare words with no prefix), the syntax parser collects those words into the `q` field of `SearchFilters`. The backend's Tantivy search engine then searches across the title, summary, and tag fields for those words. This is a full-text search — it finds fics where those words appear anywhere.

When you type `title:Dragon`, the parser extracts "Dragon" and adds it to the `q` field. The `title:` prefix tells the backend to search only in the title field. This is more precise — it won't match fics that mention "Dragon" in the summary but don't have it in the title.

The `author:` prefix works the same way — it narrows the search to the author field. You can combine these with other keywords: `author:cleo fandom:Harry Potter` finds fics by "cleo" in the Harry Potter fandom.

> **Try It Yourself:** Type `author:cleo fandom:Harry Potter` in the search bar and press Enter. The parser splits this into two parts: the author name "cleo" and the fandom tag "Harry Potter." The backend searches for fics where the author matches AND the fandom tag matches. Try removing the `author:` prefix and see how the results change — now "cleo" is searched as a full-text term, which might match fics that mention "cleo" in the summary.

## Tag Filters

The next section covers tag-specific searches:

```svelte
  <section>
    <h2>Tag Filters</h2>
    <p>Filter by specific tag types. These are the building blocks of precise searches.</p>

    <table class="syntax-table">
      <tr>
        <td class="code">fandom:Harry Potter</td>
        <td>Only fics in the Harry Potter fandom</td>
      </tr>
      <tr>
        <td class="code">char:Draco Malfoy</td>
        <td>Fics featuring Draco Malfoy</td>
      </tr>
      <tr>
        <td class="code">rel:Harry/Draco</td>
        <td>Fics with the Harry/Draco relationship</td>
      </tr>
      <tr>
        <td class="code">tag:Fluff</td>
        <td>Fics tagged with "Fluff" (freeform tag)</td>
      </tr>
      <tr>
        <td class="code">-fandom:Naruto</td>
        <td>Exclude the Naruto fandom</td>
      </tr>
      <tr>
        <td class="code">-tag:Angst</td>
        <td>Exclude fics tagged "Angst"</td>
      </tr>
    </table>
  </section>
```

The key insight here is the **exclusion operator**: prefix any tag filter with a minus sign (`-`) to exclude it. `-fandom:Naruto` means "not in the Naruto fandom." You can combine inclusions and exclusions freely: `fandom:Harry Potter -tag:Angst` finds Harry Potter fics without the Angst tag.

Under the hood, the parser calls `appendTag(filters.exclude_tags, 1, 'Naruto')` for `-fandom:Naruto`, which produces the string `1:Naruto` in the `exclude_tags` field. The backend then excludes fics with that tag.

The `appendTag` function is simple:

```typescript
function appendTag(existing: string, typeId: number, name: string): string {
  const entry = `${typeId}:${name}`;
  return existing ? `${existing},${entry}` : entry;
}
```

If there are no existing tags, it returns `1:Naruto`. If there's already a tag, it appends with a comma: `1:Harry Potter,1:Naruto`.

### Multi-Word Tag Values

Tags with spaces work automatically:

```svelte
  <table class="syntax-table">
    <tr>
      <td class="code">fandom:Harry Potter</td>
      <td>The parser greedy-eats everything after the colon until the next keyword</td>
    </tr>
    <tr>
      <td class="code">fandom:"Harry Potter"</td>
      <td>Quoted strings also work for clarity</td>
    </tr>
  </table>
```

The parser's "greedy tokenize" mode is the secret sauce. When it sees `fandom:Harry Potter tag:Fluff`, it knows that "Harry Potter" belongs to the `fandom:` key because "tag:" is a recognized key that starts a new token. Without this greedy behavior, "Harry" would be assigned to fandom and "Potter" would be treated as bare words.

The `KNOWN_KEYS` set defines what tokens the parser recognizes:

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

> **Watch Out:** If you use a keyword that isn't recognized (like `something:fancy`), the parser treats the whole thing as bare words for full-text search. Only recognized keys trigger the special parsing behavior. This means typos in keywords default to full-text search, which is usually what you want — a typo won't break your search, it'll just search for the literal text.

## Numeric Filters

For filtering by word count and chapter count:

```svelte
  <section>
    <h2>Numeric Filters</h2>
    <p>Filter by word count or chapter count using ranges.</p>

    <table class="syntax-table">
      <tr>
        <td class="code">words:10000-50000</td>
        <td>Between 10,000 and 50,000 words</td>
      </tr>
      <tr>
        <td class="code">words:>1000</td>
        <td>More than 1,000 words</td>
      </tr>
      <tr>
        <td class="code">words:<500</td>
        <td>Fewer than 500 words</td>
      </tr>
      <tr>
        <td class="code">chapters:>10</td>
        <td>More than 10 chapters</td>
      </tr>
      <tr>
        <td class="code">chapters:3-20</td>
        <td>Between 3 and 20 chapters</td>
      </tr>
    </table>
  </section>
```

The numeric parser supports three formats:

1. **Range**: `words:10000-50000` → sets both `min_words` and `max_words`. The dash separates the two values.
2. **Greater than**: `words:>1000` → sets `min_words`. The `>` symbol is intuitive — "more than."
3. **Less than**: `words:<500` → sets `max_words`. The `<` symbol means "fewer than."

The parser handles edge cases gracefully. If you type `words:abc`, it returns `{min: null, max: null}` — no constraint. If you type `words:5000-`, it only sets the minimum. These partial ranges are useful for "at least X words" or "at most Y chapters" queries.

The `parseRange` function in the syntax parser handles all three formats:

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

> **Try It Yourself:** Try `fandom:Harry Potter words:>50000 complete:true` — this finds long, completed Harry Potter fics. Or try `tag:Fluff words:<1000` for short fluff one-shots. The numeric filters are great for narrowing down fic length to match your available reading time.

## Status and Site

Filter by completion status and source site:

```svelte
  <section>
    <h2>Status & Site</h2>

    <table class="syntax-table">
      <tr>
        <td class="code">complete:true</td>
        <td>Only completed fics</td>
      </tr>
      <tr>
        <td class="code">complete:false</td>
        <td>Only works in progress</td>
      </tr>
      <tr>
        <td class="code">site:ao3</td>
        <td>Only from Archive of Our Own</td>
      </tr>
      <tr>
        <td class="code">site:ffn</td>
        <td>Only from FanFiction.net</td>
      </tr>
      <tr>
        <td class="code">site:sb</td>
        <td>Only from SpaceBattles</td>
      </tr>
      <tr>
        <td class="code">site:sv</td>
        <td>Only from SufficientVelocity</td>
      </tr>
    </table>
  </section>
```

The site shortcuts are mapped in the `SITE_MAP` object in the syntax parser:

```typescript
const SITE_MAP: Record<string, string> = {
  ao3: 'archiveofourown.org',
  ff: 'fanfiction.net',
  ffn: 'fanfiction.net',
  fp: 'fictionpress.com',
  sb: 'forums.spacebattles.com',
  sv: 'forums.sufficientvelocity.com',
};
```

So `site:ao3` becomes `source=archiveofourown.org` in the filters. The backend then queries only fics from that source. Note that both `ff` and `ffn` map to FanFiction.net — two shortcuts for the same site. This flexibility means users don't have to remember the "right" abbreviation.

### The complete Keyword

The `complete` keyword accepts several truthy/falsy values:

```typescript
if (value === 'true' || value === 'yes' || value === '1') {
  filters.complete = true;
} else if (value === 'false' || value === 'no' || value === '0') {
  filters.complete = false;
}
```

So `complete:yes`, `complete:1`, and `complete:true` all work the same way. This flexibility means users don't have to remember the exact spelling — you can type whatever feels natural. The parser is forgiving, which is important for a syntax that users are learning by trial and error.

> **Try It Yourself:** Try `site:ao3 complete:true sort:kudos` — this finds the most-kudoed completed fics on AO3. Or try `site:ffn words:>100000` for epic-length fics on FanFiction.net. The site filter is great for users who prefer one platform over another.

## Sorting and Dates

Control sort order and date ranges:

```svelte
  <section>
    <h2>Sorting & Dates</h2>

    <table class="syntax-table">
      <tr>
        <td class="code">sort:updated</td>
        <td>Sort by date last updated</td>
      </tr>
      <tr>
        <td class="code">sort:created</td>
        <td>Sort by date published</td>
      </tr>
      <tr>
        <td class="code">sort:words</td>
        <td>Sort by word count</td>
      </tr>
      <tr>
        <td class="code">sort:kudos</td>
        <td>Sort by kudos count</td>
      </tr>
      <tr>
        <td class="code">after:2024-01-01</td>
        <td>Only fics updated after Jan 1, 2024</td>
      </tr>
      <tr>
        <td class="code">before:2023-12-31</td>
        <td>Only fics updated before Dec 31, 2023</td>
      </tr>
    </table>
  </section>
```

The date parser is flexible about formats:

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

You can use `YYYY-MM-DD`, `YYYY-MM`, or just `YYYY`. The parser automatically expands partial dates to full timestamps. For `before:`, it uses the end of the period (23:59:59), and for `after:`, it uses the start (00:00:00).

This means:
- `after:2024` → "after January 1, 2024 at midnight"
- `before:2023-12` → "before December 28, 2023 at 11:59 PM"
- `after:2024-06-15` → "after June 15, 2024 at midnight"

The flexibility is intentional — users think in different levels of precision. Some want "fics from this year," others want "fics from this specific week." The parser accommodates all of them.

> **Try It Yourself:** Try `fandom:Harry Potter after:2023-01-01 before:2024-01-01 sort:updated` — this finds Harry Potter fics updated during 2023, sorted by most recently updated. Great for finding recent activity in an old fandom. Or try `after:2024` with no other filters to see everything updated this year.

## Examples

The guide includes complex multi-filter examples that show how to combine multiple keywords:

```svelte
  <section>
    <h2>Examples</h2>
    <p>Here are some real-world search queries that combine multiple filters.</p>

    <div class="example">
      <code>fandom:Harry Potter tag:Fluff words:>10000 complete:true</code>
      <p>Long, completed Harry Potter fluff fics</p>
    </div>

    <div class="example">
      <code>-fandom:Naruto -tag:Angst words:5000-20000 site:ao3</code>
      <p>Mid-length AO3 fics, not Naruto, not angst</p>
    </div>

    <div class="example">
      <code>author:cleo fandom:Harry Potter sort:kudos</code>
      <p>All fics by author "cleo" in Harry Potter, sorted by kudos</p>
    </div>

    <div class="example">
      <code>rel:Harry/Draco tag:Slow Burn chapters:>20 after:2022-01-01</code>
      <p>Long Harry/Draco slow burns updated since 2022</p>
    </div>
  </section>
```

Each example shows a complete query and explains what it finds. These are real queries that users would actually type — not toy examples. They demonstrate:

1. **Combining tags with numeric filters** — `tag:Fluff words:>10000`
2. **Using exclusions** — `-fandom:Naruto -tag:Angst`
3. **Sorting by popularity** — `sort:kudos`
4. **Using short aliases** — `rel:`, `ch:`
5. **Combining date filters with other criteria** — `after:2022-01-01`

The examples are carefully chosen to cover different use cases: finding completed fics, filtering by site, sorting by popularity, and combining multiple criteria.

## Short Keys

For the impatient (or the frequent user), the guide lists all the shorthand aliases:

```svelte
  <section>
    <h2>Short Keys</h2>
    <p>Every keyword has a short alias. Use whichever you prefer.</p>

    <table class="syntax-table">
      <tr>
        <td class="code">t:</td>
        <td>Alias for <code>title:</code></td>
      </tr>
      <tr>
        <td class="code">a:</td>
        <td>Alias for <code>author:</code> (also <code>creator:</code>)</td>
      </tr>
      <tr>
        <td class="code">f:</td>
        <td>Alias for <code>fandom:</code></td>
      </tr>
      <tr>
        <td class="code">c:</td>
        <td>Alias for <code>char:</code> (also <code>character:</code>)</td>
      </tr>
      <tr>
        <td class="code">r:</td>
        <td>Alias for <code>rel:</code> (also <code>relationship:</code>)</td>
      </tr>
      <tr>
        <td class="code">w:</td>
        <td>Alias for <code>words:</code></td>
      </tr>
      <tr>
        <td class="code">ch:</td>
        <td>Alias for <code>chapters:</code></td>
      </tr>
      <tr>
        <td class="code">s:</td>
        <td>Alias for <code>site:</code></td>
      </tr>
    </table>
  </section>
```

Both `title:` and `t:` are recognized. Both `fandom:` and `f:` work. This means power users can type ultra-short queries like:

```
f:HP w:>10k ch:true s:ao3
```

Which the parser handles just as well as the verbose version.

The short keys are implemented in the `KNOWN_KEYS` set in the syntax parser. Both the long and short forms are members of the set, so the parser treats them identically. You don't need to remember which aliases exist — if you type a single letter followed by a colon, the parser checks if it's a known key.

> **Try It Yourself:** Try typing the same search using both long and short forms. For example: `fandom:Harry Potter words:>10000` vs `f:Harry Potter w:>10000`. Both should return the same results. Pick whichever feels more natural to you. Most users start with the long forms and gradually switch to short keys as they get comfortable.

## Common Mistakes

Learning a new syntax takes practice. Here are the most common mistakes users make, and how to avoid them:

### 1. Forgetting the Colon

**Wrong:** `fandom Harry Potter`
**Right:** `fandom:Harry Potter`

Without the colon, `fandom` is treated as a bare word — the parser searches for the text "fandom" in titles and summaries. The colon is what tells the parser "this is a keyword, not a search term."

### 2. Using the Wrong Keyword

**Wrong:** `relationship:Harry/Draco`
**Right:** `rel:Harry/Draco` (or `relationship:Harry/Draco`)

Both `rel:` and `relationship:` work — they're aliases. But `relationship:` is the long form. If you type `ship:Harry/Draco`, the parser doesn't recognize `ship` as a keyword, so it treats the whole thing as a bare word search.

### 3. Case Sensitivity in Values

**Wrong:** `fandom:harry potter` (if the tag is stored as "Harry Potter")
**Right:** `fandom:Harry Potter`

Keywords are case-insensitive (`fandom:` works the same as `FANDOM:`), but tag values are case-sensitive. The database stores tags with their original capitalization. If you're not sure of the exact spelling, try the dropdown filters first to see what tags exist.

### 4. Confusing `words:` with `chapters:`

**Wrong:** `chapters:>50000` (trying to filter by word count)
**Right:** `words:>50000`

`chapters:` filters by chapter count, not word count. A 50,000-word fic might have 10 chapters, not 50,000. Use `words:` for word count and `chapters:` for chapter count.

### 5. Missing Quotes for Multi-Word Values

**Wrong:** `fandom:Harry Potter tag:Fluff` (this actually works due to greedy parsing!)
**Right:** `fandom:"Harry Potter" tag:Fluff` (also works, and is clearer)

The greedy parser handles unquoted multi-word values correctly, so this isn't really a mistake — but quotes make your intent clearer. Use them when a tag name might be confused with a keyword.

### 6. Overloading the Query

**Wrong:** `fandom:Harry Potter char:Harry rel:Harry/Ginny tag:Fluff words:>10000 complete:true site:ao3 sort:kudos after:2023-01-01`
**Right:** Start simple, add filters gradually.

Complex queries are powerful, but they can return zero results if the combination is too restrictive. Start with one or two filters, see what comes back, and add more if needed. The search engine is fast — you can iterate quickly.

### 7. Excluding Everything

**Wrong:** `-tag:Angst -tag:Major Character Death -tag:Character Death -tag:Slow Burn -tag:Hurt/Comfort`
**Right:** Exclude only what you truly don't want.

Every exclusion narrows the results. If you exclude too many tags, you might filter out fics you'd actually enjoy. Be selective with exclusions — use them for hard "no" preferences, not soft "meh" ones.

## Linking from the Search Bar

The syntax guide is linked from the search page's query row:

```svelte
<a class="syntax-link muted" href="/search/syntax" target="_blank">Syntax guide</a>
```

The `target="_blank"` opens it in a new tab. This is intentional — the user shouldn't lose their current search when they check the syntax. The guide is a reference, not a destination.

In the future, we could add inline tooltips to the search input that show syntax hints as the user types. For example, typing `fandom:` could show a small popup: "Enter a fandom name. Example: fandom:Harry Potter." But for now, the static guide page is the simplest and most reliable approach. It loads instantly, works offline, and can be updated without touching the search component.

## Utility Functions

The search results display uses several utility functions from `$lib/util.ts`. Let's look at the ones that make the results readable:

### formatWords

```typescript
export function formatWords(words: number): string {
  return words.toLocaleString('en-US');
}
```

Converts a number like `1234567` into `"1,234,567"`. This makes large word counts scannable — instead of counting digits, you instantly see "1.2 million." The `'en-US'` locale ensures commas are used as thousand separators.

### relativeTime

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

Converts an ISO timestamp into a human-readable relative time: "3 days ago," "2 hours ago," "less than a minute ago." This is much more intuitive than showing "2024-01-15T14:30:00Z" — users care about recency, not exact dates.

### detectSite

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

Inspects the fic's URL to determine which platform it's from. The result is displayed as a small badge next to the author name. This helps users quickly identify which site a fic is from without clicking the link.

### stripHtml

```typescript
export function stripHtml(html: string): string {
  if (!html) return '';
  return html
    .replace(/<br\s*\/?>/gi, ' ')
    .replace(/<\/(p|div)>/gi, ' ')
    .replace(/<[^>]+>/g, '')
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/\s+/g, ' ')
    .trim();
}
```

Fic summaries often contain HTML tags (`<p>`, `<br>`, `<b>`, etc.). This function strips them all, converting `<br>` to spaces and decoding HTML entities. The result is clean plain text that displays correctly in the result card. We then `slice(0, 250)` to show only the first 250 characters — enough to give a taste of the summary without overwhelming the card.

## What You Learned

In this part, you learned how to:

1. **Build a SvelteKit layout** with sticky navigation, tab switching, and responsive design.
2. **Create a search page** that reads URL parameters, parses query syntax, and displays paginated results.
3. **Implement filter controls** with dropdown menus, number inputs, date pickers, and tag fields.
4. **Write a query syntax** that converts human-readable keywords into structured search filters.
5. **Design result cards** with title, author, tags, metadata, and action buttons.
6. **Handle responsive layouts** that adapt from desktop to mobile with CSS Grid and Flexbox.
7. **Write a syntax guide** that teaches users a powerful query language.
8. **Use utility functions** to format numbers, dates, and HTML content.

These skills transfer to any web application with search functionality. The patterns you learned — data-driven UI, URL-synced state, syntax parsing, and responsive design — are universal.

## Practice: 5 Different Search Queries

Here are five queries to try, ranging from simple to complex. Type them in the search bar and press Enter to see the results. Each query demonstrates different features of the syntax.

1. **Simple keyword search:**
   ```
   time travel
   ```
   Finds fics mentioning "time travel" in any field. This is the simplest possible query — just bare words, no syntax required. The search engine looks for these words in titles, summaries, and tags.

2. **Fandom + completion:**
   ```
   fandom:Lord of the Rings complete:true
   ```
   Completed Lord of the Rings fics. The fandom filter ensures you only see LotR content, and the completion filter eliminates works in progress. Great for when you want a finished story.

3. **Author + word count:**
   ```
   author:snowqueens ibigdragon words:50000-200000
   ```
   Long fics by a specific author. The word count range filters for epic-length works — perfect for when you want a long reading session and don't want to run out of story.

4. **Exclusion + sorting:**
   ```
   -fandom:My Hero Academia -tag:Major Character Death sort:kudos
   ```
   Top-kudos fics excluding MHA and major character death. The exclusions remove content you don't want, and `sort:kudos` brings the most popular fics to the top.

5. **The kitchen sink:**
   ```
   fandom:Harry Potter rel:Harry/Hermione words:>20000 complete:true -tag:Angst -tag:Infidelity after:2022-01-01 sort:updated
   ```
   Long, completed Harry/Hermione fics from 2022 onwards, no angst or infidelity, sorted by most recently updated. This query combines eight different filters in a single line — try building the same search with dropdown menus!

## Bonus Challenges

Ready for more? Try these advanced challenges:

1. **Find fics shorter than 1000 words in a specific fandom, sorted by kudos:**
   ```
   fandom:Naruto words:<1000 sort:kudos
   ```
   This finds the most popular short fics — perfect for a quick read during a break.

2. **Find recently updated fics from a specific author on a specific site:**
   ```
   author:snowqueens ibigdragon site:ao3 after:2024-01-01
   ```
   This finds fics by a specific author updated this year on AO3. Great for following your favorite writers.

3. **Find completed fics with a specific relationship, excluding certain tags:**
   ```
   rel:Drarry complete:true -tag:Angst -tag:Major Character Death
   ```
   This finds happy Drarry fics — completed, no angst, no character deaths. Perfect for when you want a feel-good read.

4. **Find long fics in a fandom you don't usually read, sorted by word count:**
   ```
   fandom:My Hero Academia words:>100000 sort:words
   ```
   This finds the longest MHA fics. Sometimes you want to dive deep into a new fandom with an epic story.

5. **Find fics updated this week in any fandom, sorted by recent updates:**
   ```
   after:2024-01-01 sort:updated
   ```
   Without any fandom filter, this shows recently updated fics across all fandoms. Great for discovering new content.

These challenges help you practice combining different filters. The more you experiment, the more intuitive the syntax becomes.

# Quick Reference

Here's a condensed reference card for everything covered in this part. Pin it to your wall, tape it to your monitor, or just keep it in your back pocket.

## Layout Files

| File | Purpose |
|------|---------|
| `src/routes/+layout.svelte` | App shell: topbar, tabs, footer |
| `src/routes/+layout.ts` | SPA mode config (ssr=false) |
| `src/routes/+page.svelte` | Root page (intentionally empty) |
| `src/routes/+page.ts` | Root page loader (if any) |

## Search Files

| File | Purpose |
|------|---------|
| `src/routes/search/+page.svelte` | Search page UI (540 lines) |
| `src/routes/search/+page.ts` | Page loader (reads URL params) |
| `src/routes/search/syntax/+page.svelte` | Syntax guide (static) |
| `src/lib/api/search.ts` | Search API client + types |
| `src/lib/search/syntax.ts` | Query syntax parser |
| `src/lib/util.ts` | Formatting utilities |

## Syntax Quick Reference

| Keyword | Short | Example | Effect |
|---------|-------|---------|--------|
| `title:` | `t:` | `t:Dragon` | Search title only |
| `author:` | `a:` | `a:cleo` | Search author only |
| `fandom:` | `f:` | `f:HP` | Include fandom tag |
| `char:` | `c:` | `c:Harry` | Include character tag |
| `rel:` | `r:` | `r:Harry/Hermione` | Include relationship tag |
| `tag:` | — | `tag:Fluff` | Include freeform tag |
| `-fandom:` | `-f:` | `-f:Naruto` | Exclude fandom tag |
| `-tag:` | — | `-tag:Angst` | Exclude freeform tag |
| `words:` | `w:` | `w:>10000` | Word count range |
| `chapters:` | `ch:` | `ch:3-20` | Chapter count range |
| `complete:` | — | `comp:true` | Completion status |
| `site:` | `s:` | `s:ao3` | Source site |
| `sort:` | — | `sort:kudos` | Sort order |
| `after:` | — | `after:2024-01-01` | Date range start |
| `before:` | — | `before:2023-12-31` | Date range end |

## Key CSS Classes

| Class | Element | Purpose |
|-------|---------|---------|
| `.topbar` | `<header>` | Sticky navigation bar |
| `.tab-bar` | `<nav>` | Tab button container |
| `.tab` | `<button>` | Individual tab button |
| `.tab.active` | `<button>` | Currently selected tab |
| `.search-area` | `<div>` | Right-side search field |
| `.nav-search` | `<input>` | Navbar search input |
| `.filter-grid` | `<div>` | Two-column filter layout |
| `.filter-item` | `<label>` | Individual filter control |
| `.result-card` | `<div>` | Search result card |
| `.tag-pill` | `<span>` | Tag badge in results |
| `.pagination` | `<div>` | Page navigation controls |

## Search API

The search API endpoint is `GET /api/v0/search`. It accepts query parameters matching the `SearchFilters` fields and returns a `SearchResponse` JSON object.

**Example request:**
```
GET /api/v0/search?q=Harry+Potter&fandom=1:Harry+Potter&words_min=10000&complete=true&page=1
```

**Example response:**
```json
{
  "total": 42,
  "page": 1,
  "per_page": 20,
  "results": [
    {
      "url_id": "ao3-12345678",
      "title": "Harry Potter and the Methods of Rationality",
      "author": "Less Wrong",
      "source": "https://archiveofourown.org/works/10000000",
      "words": 661000,
      "chapters": 122,
      "status": "Complete",
      "description": "A rationalist reimagining of Harry Potter...",
      "updated": "2024-01-15T14:30:00Z",
      "rank": 1.2,
      "tags": [
        { "name": "Harry Potter - J.K. Rowling", "type": "Fandom", "type_id": 1, "score": 0.95 }
      ],
      "total_freeform": 15
    }
  ]
}
```

The `rank` field is the Tantivy relevance score. Lower values mean better matches. A rank of `1.0` means the fic matched very strongly; `5.0` means a weaker match.

## Common Patterns

Throughout this part, we used several patterns that appear in many web applications:

1. **Data-driven UI** — instead of writing separate markup for each tab, we define tabs as data and loop over them. This pattern appears in navigation bars, dropdown menus, and form builders.

2. **URL-synced state** — we update the URL when the search changes, so searches can be bookmarked and shared. This pattern is essential for any page with filterable content.

3. **Syntax parsing** — we convert a human-readable query string into structured data. This pattern appears in SQL query builders, cron job schedulers, and configuration languages.

4. **Responsive design** — we use CSS media queries to adapt layouts for different screen sizes. This pattern is mandatory for any public-facing web application.

5. **Error handling** — we wrap API calls in try/catch and display user-friendly error messages. This pattern prevents confusing blank screens when things go wrong.

# Summary

In this part, we built the navigation and search systems that make FicHub feel like a real application:

- **The layout** (`+layout.svelte`) provides the app shell — a sticky topbar with brand, tabs, and search field, plus a footer. Tabs switch between Download, Recommendations, and Suggestions. The search field navigates to the advanced search page. The layout is defined once and wraps every page, ensuring consistent navigation and appearance. The flex layout with `min-height: 100vh` and `flex: 1` on main creates a sticky footer that stays at the bottom.

- **The search page** (`/search`) is a full-featured search interface with query syntax parsing, advanced filter controls, paginated results, and four search tabs. The `doSearch()` function parses syntax, merges with form filters, syncs the URL, and calls the API. The page loader reads URL parameters so searches can be bookmarked and shared. The auto-search on mount pattern ensures users arriving from the navbar search see results immediately.

- **The filter system** provides dropdown menus for completion status, site, sort order, word count range, chapter range, date range, and tag inclusion/exclusion. The Work Search tab has a two-column grid of these filters. People Search, Bookmark Search, and Tag Search are placeholders for future development, with syntax hints guiding users to the query syntax. The merge logic in `doSearch()` ensures form fields override syntax values when both are set.

- **The syntax guide** (`/search/syntax`) teaches users a powerful query language with keywords like `fandom:`, `words:`, `complete:`, and `sort:`. It supports short aliases (`f:`, `w:`, `s:`), exclusion with `-`, and flexible date formats. The guide turns curious users into power users, and the short keys make complex searches fast to type. The consistent table format makes it easy to scan and reference.

Together, these components create a search experience that's both accessible (point-and-click filters) and powerful (query syntax). Users can start simple and gradually learn the syntax as they need more precision. The layout provides a consistent shell, the search page provides the tools, and the syntax guide provides the knowledge. That's a complete search experience.

The patterns we covered in this part — data-driven UI, URL-synced state, syntax parsing, responsive design, and error handling — are not specific to FicHub. They appear in every well-built web application. Whether you're building an e-commerce site with product filters, a documentation site with search, or a dashboard with data tables, these patterns will serve you well.

In the next part, we'll explore how FicHub handles the actual download process — taking a fic URL, fetching its content, and converting it to an ebook format. But that's a story for another chapter.
# Part 7: Testing Your Code

---

## Chapter 29: Why Test?

### Testing Is Like Checking Your Homework

Remember when you were in school, and after finishing a math worksheet, you'd flip to the back of the book to check your answers? That's essentially what testing is for your code — you write a little program that checks whether your *real* program does what it's supposed to do.

But here's the thing: computers are way more thorough than you were at checking homework. A computer will check *every single time* you change something. It never gets tired, it never skips an answer, and it never says "eh, that's close enough." It either passes or it fails.

That's incredibly powerful.

Imagine you spent all day building a beautiful search feature. It works perfectly! You test it manually, click around, everything looks great. You ship it. Users are happy. Then a week later, you need to tweak how the URL gets built. You make a small change, ship it, and suddenly... searches are broken. Nobody noticed until users started complaining.

With tests, you could have run a simple command — `npm run test` — and your tests would have screamed "HEY! The search is broken!" before you ever shipped the update.

Tests are your safety net. They catch mistakes before your users do. They let you make changes with confidence. They turn "I hope this works" into "I know this works."

And the best part? Once you write a test, it keeps working forever. Every time you make a change, every time you add a feature, every time you fix a bug — you run the tests, and if they all pass, you know you haven't broken anything. It's like having a tireless assistant who checks your work every single time.

### Types of Tests

There are three main types of tests, and they work together like a team:

**Unit Tests** are like checking individual ingredients before you cook. You test one small piece of code — a function, a component, a utility — in isolation. "Does `formatWords(1234567)` return `'1,234,567'`?" That's a unit test. They're fast, focused, and easy to write.

**Integration Tests** are like checking that all the ingredients work together in the recipe. "When the user types a URL into the download form and clicks Download, does the API get called with the right parameters?" That's an integration test. They verify that different pieces of your code work together correctly.

**End-to-End (E2E) Tests** are like having someone eat the whole meal and report back. They test the entire application flow, from button clicks to API calls to database writes, as a real user would experience it. "Can a user go to the website, search for a story, download it as an EPUB, and read it?" That's an E2E test. These are the most realistic but also the slowest and most complex to write.

### Unit Tests: Testing One Function in Isolation

A unit test is the simplest and fastest kind of test. You call a function with specific inputs, and you check that the outputs match what you expect.

```typescript
import { formatWords } from './util';

it('adds thousands separators', () => {
  expect(formatWords(1234567)).toBe('1,234,567');
  expect(formatWords(50000)).toBe('50,000');
  expect(formatWords(0)).toBe('0');
});
```

That's it! You're saying: "Hey `formatWords`, when I give you 1,234,567, you'd better return the string `'1,234,567'`." And if it doesn't, the test fails, and you know exactly where the bug is.

Unit tests are fast (they run in milliseconds), focused (they test one thing), and easy to write. They're the bread and butter of testing. Most of the tests in our project are unit tests — they test individual functions and components without any external dependencies.

The key word is "isolation." A true unit test doesn't talk to the network, doesn't read from the filesystem, and doesn't depend on any other service. It just tests one function with known inputs and expected outputs. This makes them incredibly reliable — they either pass or fail, and there's no ambiguity about why.

Unit tests are also the fastest tests to write and run. You can have hundreds of them and they'll all run in under a second. This means you get instant feedback — run the tests, see the results, move on. No waiting, no setup, no configuration.

The best part? Unit tests are great for learning. When you're unsure how a function works, write a test for it. The test becomes a tiny experiment: "I think this function does X. Let me write a test and find out." If the test passes, you understand the function. If it fails, you've learned something new.

### Integration Tests: Testing How Pieces Work Together

Integration tests check that multiple pieces of your code cooperate correctly. Maybe a component calls a function which calls an API. An integration test verifies that the whole chain works.

```typescript
it('renders download links on success', async () => {
  // Mock the API
  mockFetch.mockResolvedValue({
    ok: true,
    json: async () => ({ err: 0, url_id: 'abc123', /* ... */ }),
  });

  render(DownloadTab);
  const input = screen.getByLabelText('Fanfiction URL');
  await fireEvent.input(input, { target: { value: 'https://ao3.org/works/1' } });
  await fireEvent.click(screen.getByText('Download'));

  await waitFor(() => expect(screen.getByText('My Story')).toBeTruthy());
});
```

Here, we're testing that the DownloadTab component correctly calls the API, processes the response, and renders the results. It's testing several pieces working together — the component, the API client, the data flow, and the rendering.

Integration tests are more powerful than unit tests because they catch bugs that only happen when pieces interact. A function might work perfectly in isolation but break when it's connected to the rest of the system. Integration tests catch those kinds of bugs.

The trade-off is that integration tests are slower and more complex. They might need mocked APIs, setup data, and more verbose test code. That's why you have fewer of them — but you definitely need some.

### Test-Driven Development (TDD): Write Test First, Then Code

Here's a mind-bending idea: what if you wrote the test *before* you wrote the code?

It goes like this:

1. **Red** — Write a test for the feature you want. It fails because the feature doesn't exist yet.
2. **Green** — Write the simplest code that makes the test pass.
3. **Refactor** — Clean up the code while keeping the test green.

This is called Test-Driven Development, or TDD. It sounds backwards, but it works amazingly well. Here's why:

- You think about *what* the code should do before you think about *how* to do it.
- You never write code that isn't tested.
- Every feature has at least one test from the start.
- You end up with clean, focused code because you only write what's needed to pass the test.

Here's a TDD example for a new function:

```typescript
// Step 1: Write the test first
it('capitalizes the first letter', () => {
  expect(capitalize('hello')).toBe('Hello');
});

// Step 2: Run the test — it fails! (Red)
// Step 3: Write the simplest code to make it pass (Green)
function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

// Step 4: Run the test — it passes!
// Step 5: Refactor if needed, then write another test
```

You don't have to use TDD to benefit from tests, but it's a great habit to build. Even if you write tests after the code, you're still getting most of the benefits.

### The Testing Pyramid

Think of your tests as a pyramid:

```
        /\
       /  \        Few E2E tests (slow, comprehensive)
      /    \
     /------\      More integration tests (medium speed)
    /        \
   /----------\    Many unit tests (fast, focused)
  /____________\
```

At the bottom, you have *lots* of unit tests. They're fast, cheap, and test small things. In the middle, you have integration tests — fewer of them, but they test bigger things. At the top, you have a few end-to-end tests that test the whole app.

Why this shape? Because:

- Unit tests are **fast** and **cheap** — you can have hundreds of them, and they run in under a second. Each one tests one small thing, so if one fails, you know exactly what broke.
- Integration tests are **slower** and **more complex** — they need more setup, take longer to run, and are harder to debug. You need them, but you don't need as many.
- E2E tests are **slowest** and **most fragile** — they depend on the entire system working together. If the server is down, the test fails even if your code is perfect. Keep them to a minimum.

The golden rule: aim for about 70% unit tests, 20% integration tests, and 10% E2E tests. Not a strict rule, but a good guideline.

### Benefits of Testing

**Catch bugs early.** A test that fails tells you about a bug *before* your users find it. Bugs found early are cheap to fix. Bugs found in production are expensive — they require emergency hotfixes, user apologies, and sleepless nights.

**Refactor safely.** Want to reorganize your code? Rename a function? Change how something works internally? With tests, you can make changes and run `npm run test` to verify nothing broke. Without tests, you're just hoping for the best. Tests give you the courage to improve your code.

**Document behavior.** Tests are living documentation. A new developer can read your tests and understand what your code is supposed to do. "Oh, so `detectSite` returns 'AO3' for archiveofourown.org URLs — got it!" Unlike comments, tests can never go out of date, because if they're wrong, they fail.

**Sleep better at night.** Seriously. Knowing that a computer is checking your code every time you make a change is a wonderful feeling. It's like having a spell-checker, but for your logic.

**Make code changes faster.** Without tests, every change requires careful manual testing. With tests, you make the change and run `npm run test`. If it passes, you're done. This means you ship features faster and fix bugs faster.

### What to Test

**Functions that do things.** Pure functions like `formatWords`, `detectSite`, `stripHtml` — these are easy to test and super valuable. You give them an input, check the output, done. They're the low-hanging fruit of testing — quick to write, high value.

**API calls.** Your API client is the bridge between your UI and the server. If it breaks, everything breaks. Test every function: success cases, error cases, edge cases. What happens when the server returns a 500? What happens when the response JSON is malformed? What happens with an empty response?

**Components that handle user interaction.** If a button does something when clicked, test that it does the right thing. If a form validates input, test the validation. If a tab switches content, test the switch. These are the parts of your app that users interact with directly — they need to work.

**Edge cases.** What happens when the input is empty? What happens when the API returns an error? What happens with really long strings? What happens with special characters like emojis or HTML tags? Edge cases are where most bugs hide, because developers often forget to test them.

**Business logic.** Any code that implements rules or calculations should be tested. "If the user has a premium account, show unlimited results. Otherwise, show 20." Test both paths. "If the search query contains special characters, encode them properly." Test the encoding.

### What Not to Test

**Implementation details.** Don't test *how* a function works internally. Test *what* it does. If you refactor the internals, your tests should still pass. For example, don't test that a function uses a `for` loop — test that it returns the right result. The implementation might change from a `for` loop to `Array.map`, but the test shouldn't care.

**Third-party libraries.** Don't test that `Array.map` works correctly. That's someone else's job. Don't test that Svelte renders HTML. That's already tested by the Svelte team. Don't test that `fetch()` makes network requests. That's the browser's job. Trust the library, test your code.

**Trivial getters/setters.** If a function just returns a property, you don't need a test for it. `function getName() { return this.name; }` doesn't need a test. The time you'd spend writing the test is better spent testing something useful.

**The framework itself.** Don't write tests to check that SvelteKit routing works. That's already tested by the SvelteKit team. Your tests should focus on *your* code, not the libraries you use. Framework tests are their responsibility, not yours.

**Generated code.** If you're using a tool that generates code (like schema bindings or API clients from OpenAPI specs), you probably don't need to test the generated code. Test the code that *uses* the generated code instead. The generated code is someone else's responsibility.

### Try It Yourself

Before moving on, think about the `client.ts` file you built in earlier chapters. What functions does it have? Which ones would you test? What edge cases can you think of?

Write down your answers. When we get to Chapter 31, we'll test them all — and you can compare your guesses with what we actually test!

Here are some questions to get you thinking:

1. What happens when `fetchExport` gets a network error?
2. What happens when the API returns unexpected JSON?
3. What happens if you pass an empty string to `buildSearchQuery`?
4. What happens if you vote with a value that isn't 1 or -1?

The best testers are the ones who can imagine ways their code might break. Start building that imagination now.

---

## Chapter 30: Setting Up Vitest

### What Is Vitest?

Vitest (pronounced "vy-test," like "vitamin test") is a test runner designed specifically for Vite projects. Since our project uses Vite for building, Vitest is the perfect choice. It's fast, it's modern, and it integrates seamlessly with our existing setup.

Think of Vitest as the conductor of an orchestra. The musicians are your individual tests. Vitest tells them when to play, keeps track of who passed and who failed, and gives you a nice report at the end.

Why Vitest instead of Jest (the other popular test runner)? Because Vitest is designed for Vite projects. It uses the same configuration, the same transforms, and the same module resolution. With Jest, you'd need separate configuration for how TypeScript gets compiled, how Svelte components get transformed, and how module aliases work. Vitest just... works. It reads your `vite.config.ts` and uses it automatically.

### Installing the Tools

Let's install everything we need. Open your terminal and run:

```bash
npm install -D vitest @testing-library/svelte @testing-library/jest-dom jsdom
```

The `-D` flag means "development dependency" — these packages are only needed for testing, not for the running application.

Here's what each package does:

- **vitest** — The test runner. It discovers your test files, runs them, and reports results. It's the engine that makes everything work.

- **@testing-library/svelte** — Helpers for testing Svelte components. Gives you `render()` to mount components, `fireEvent` to simulate user interactions, `screen` to find elements, and `waitFor` to wait for async updates.

- **@testing-library/jest-dom** — Extra assertion matchers. Things like `toBeInTheDocument()`, `toHaveTextContent()`, and `toBeVisible()`. These make your test assertions read like English: "expect the button to be in the document."

- **jsdom** — A fake browser environment. It provides `document`, `window`, DOM elements, and all the browser APIs your components need — without actually opening a browser. It's like a headless browser just for testing.

After installing, your `package.json` devDependencies should include these four packages:

```json
{
  "devDependencies": {
    "vitest": "^2.1.0",
    "@testing-library/svelte": "^5.2.0",
    "@testing-library/jest-dom": "^6.5.0",
    "jsdom": "^25.0.0"
  }
}
```

### The test-setup.ts File

Create a new file at `src/test-setup.ts` with a single line:

```typescript
import '@testing-library/jest-dom/vitest';
```

That's it! One line. But this line is doing something important: it imports the custom matchers from `jest-dom` and registers them with Vitest. This means you can use matchers like `toBeInTheDocument()`, `toHaveClass()`, and `toHaveAttribute()` in your tests without importing them every time.

Think of it as loading a plugin. Without this line, Vitest doesn't know about the extra matchers. With this line, they're available everywhere — in every test file, in every describe block, in every it block.

If you forget this file, you'll get errors like `toBeInTheDocument is not a function` when you try to use jest-dom matchers. The fix is always the same: make sure this file exists and is configured in `vite.config.ts`. It's one of the most common setup mistakes, and now you know how to avoid it.

The `@testing-library/jest-dom/vitest` import path is specifically for Vitest. There are also paths for Jest and other test runners. Make sure you're using the right one for your setup.

### Configuring vite.config.ts

Open your `vite.config.ts` and add a `test` section. Here's the complete file with the test configuration:

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

Let's break down that `test` section line by line:

**`environment: 'jsdom'`** — Run tests in a fake browser environment. This gives us `document`, `window`, DOM elements, and all the browser APIs your components need. Without this, your component tests would crash because there's no DOM to render into. jsdom is not a real browser — it's a JavaScript implementation of browser APIs. It's fast and lightweight, perfect for testing.

**`globals: true`** — Make `describe`, `it`, `expect`, `vi`, `beforeEach`, and other test functions available globally. You don't need to import them in every test file! This saves boilerplate and makes your tests cleaner. If you prefer explicit imports, you can set this to `false` and import from `vitest` in each file. Either way works — it's a matter of preference.

**`setupFiles: ['src/test-setup.ts']`** — Run our setup file before all tests. This is where the `jest-dom` matchers get loaded. Vitest will execute this file once, before running any tests. Think of it as a one-time initialization step.

**`include: ['src/**/*.test.{ts,svelte}']`** — Look for test files matching this pattern. The `**` means "any directory," so this matches any `.test.ts` or `.test.svelte` file inside `src/` or any subdirectory. Files outside `src/` (like in `node_modules`) are ignored. This pattern is flexible enough to find tests anywhere in your source tree.

### Running Tests

Add these scripts to your `package.json`:

```json
{
  "scripts": {
    "test": "vitest run",
    "test:watch": "vitest"
  }
}
```

Now you have two commands:

**`npm run test`** — Runs all tests once and shows the results. Use this before committing code or deploying. It exits when done and tells you exactly how many tests passed and failed. A quick way to verify everything is working.

**`npm run test:watch`** — Keeps Vitest running and automatically re-runs tests whenever you save a file. Use this while developing. It never exits — you stop it with Ctrl+C. This is incredibly convenient because you get instant feedback as you write code.

When you run `npm run test`, you'll see something like:

```
 ✓ src/lib/util.test.ts (9 tests) 12ms
 ✓ src/lib/api/client.test.ts (6 tests) 23ms
 ✓ src/lib/api/search.test.ts (14 tests) 8ms
 ✓ src/lib/components/DownloadTab.test.ts (2 tests) 156ms

 Test Files  4 passed (4)
      Tests  31 passed (31)
   Start at  14:32:07
   Duration  1.2s
```

Green means passing. Red means failing. It's that simple.

When a test fails, you'll see something like:

```
 ✗ src/lib/util.test.ts > formatWords > adds thousands separators
   Expected: "1,234,567"
   Received: "1234567"
```

The error message tells you exactly which test failed, what was expected, and what was actually returned. This makes debugging much easier — you know exactly where to look.

### Test File Naming

By convention, test files live next to the files they test, with `.test.ts` at the end:

```
src/lib/util.ts                ← the code
src/lib/util.test.ts           ← its tests

src/lib/api/client.ts          ← the code
src/lib/api/client.test.ts     ← its tests

src/lib/api/search.ts          ← the code
src/lib/api/search.test.ts     ← its tests
```

This makes it easy to find the tests for any piece of code. You don't have to go hunting through a separate `__tests__` folder. If you see `client.ts`, you know the tests are right next door in `client.test.ts`.

Some teams prefer a separate `tests/` directory at the project root. That's fine too, but for small to medium projects, co-locating tests with source code is more convenient. You always know where the tests are — right next to the code they test.

The naming convention is important because Vitest uses the `include` pattern in `vite.config.ts` to find test files. If your test files don't match the pattern, they won't be discovered. The default pattern is `src/**/*.test.{ts,svelte}`, which means any file ending in `.test.ts` or `.test.svelte` inside the `src` directory.

### Describe Blocks: Organizing Tests

The `describe` function groups related tests together:

```typescript
describe('formatWords', () => {
  it('adds thousands separators', () => {
    expect(formatWords(1234567)).toBe('1,234,567');
  });

  it('handles zero', () => {
    expect(formatWords(0)).toBe('0');
  });

  it('handles negative numbers', () => {
    expect(formatWords(-5000)).toBe('-5,000');
  });
});

describe('detectSite', () => {
  it('detects AO3', () => {
    expect(detectSite('https://archiveofourown.org/works/1')).toBe('AO3');
  });

  it('detects FanFiction.net', () => {
    expect(detectSite('https://www.fanfiction.net/s/1')).toBe('FanFiction.net');
  });
});
```

Think of `describe` as a heading. "Here are all the tests for `formatWords`." "Here are all the tests for `detectSite`." It keeps things organized, especially when you have dozens of tests in one file.

You can also nest `describe` blocks for even more organization:

```typescript
describe('API Client', () => {
  describe('fetchExport', () => {
    it('returns data on success', () => { /* ... */ });
    it('throws on error', () => { /* ... */ });
  });

  describe('submitSuggestion', () => {
    it('sends correct body', () => { /* ... */ });
  });
});
```

This creates a hierarchy in the test output:

```
API Client
  fetchExport
    ✓ returns data on success
    ✓ throws on error
  submitSuggestion
    ✓ sends correct body
```

### It Blocks: Individual Test Cases

The `it` function (also called `test` — they're completely interchangeable) defines a single test case:

```typescript
it('adds thousands separators', () => {
  expect(formatWords(1234567)).toBe('1,234,567');
});
```

The string you pass is the test name. Make it descriptive! When a test fails, you'll see this name in the output, so you want to know exactly what was being tested without reading the code.

**Good test names describe the expected behavior:**
- `"adds thousands separators"` — clear and specific
- `"returns empty string for empty input"` — describes the edge case
- `"throws ApiError on HTTP failure"` — describes the error case
- `"omits page when page is 1"` — describes the conditional logic

**Bad test names leave you guessing:**
- `"test 1"` — which test? what does it test?
- `"it works"` — what works? how?
- `"stuff"` — definitely not helpful
- `"testing the function"` — testing what about the function?

A good rule of thumb: if the test name is the only thing you read, can you understand what the test does? If yes, it's a good name. If you need to read the test code to understand what's being tested, the name needs work.

Another tip: use the pattern "should [expected behavior]" or "[action] [expected result]". For example:
- `"should return formatted number with commas"`
- `"should throw error when input is empty"`
- `"omits page parameter when value is 1"`

These patterns make your test output readable as a specification:

### Expect Assertions

The `expect` function is where the magic happens. It takes a value and checks it against an expected result. Here are the most common matchers:

```typescript
// Equality
expect(result).toBe(expected);              // strict equality (===)
expect(result).toEqual(expected);           // deep equality (for objects/arrays)
expect(result).not.toBe(expected);          // negated

// Types and values
expect(result).toBeNull();                  // is null
expect(result).toBeUndefined();             // is undefined
expect(result).toBeTruthy();                // truthy value
expect(result).toBeFalsy();                 // falsy value
expect(result).toBeInstanceOf(Error);       // is instance of class

// Strings and arrays
expect(result).toContain('hello');          // string contains substring
expect(result).toContain('item');           // array contains element
expect(result).toHaveLength(3);             // array/string length

// Numbers
expect(result).toBeGreaterThan(5);          // greater than
expect(result).toBeLessThan(10);            // less than
expect(result).toBeGreaterThanOrEqual(5);   // greater than or equal

// Functions
expect(fn).toThrow('error message');        // function throws
expect(fn).not.toThrow();                   // function doesn't throw

// Promises
expect(promise).resolves.toBe(value);       // promise resolves
expect(promise).rejects.toBeInstanceOf(Error); // promise rejects
```

### Mocking: vi.fn() and vi.mock()

Sometimes your code calls external things — APIs, the filesystem, random number generators. You don't want your tests to depend on real APIs. That would make them slow, flaky, and dependent on network connectivity.

Mocking replaces real things with fakes that you control:

```typescript
const mockFetch = vi.fn();
globalThis.fetch = mockFetch;

mockFetch.mockResolvedValue({
  ok: true,
  json: async () => ({ err: 0, data: 'test' }),
});

// Now when your code calls fetch(), it gets this fake response
const result = await myFunction();
```

Here's what's happening step by step:

1. `vi.fn()` creates a fake function (a mock). It records every time it's called and what it's called with.
2. We replace the real `fetch` with our mock. Now when any code calls `fetch()`, it calls our mock instead.
3. We tell the mock what to return when called. `mockResolvedValue` means "when called, return a Promise that resolves with this value."
4. Your code calls `fetch()` and gets the fake response. It doesn't know the difference!

After each test, you should reset your mocks:

```typescript
beforeEach(() => {
  mockFetch.mockReset();
});
```

This ensures that one test's mock setup doesn't affect another test. Without this, a test that sets up a mock to return success could leak into the next test, causing confusing failures.

You can also inspect what was passed to the mock:

```typescript
// Check how many times it was called
expect(mockFetch).toHaveBeenCalledTimes(1);

// Check what arguments it was called with
expect(mockFetch).toHaveBeenCalledWith(
  'https://api.example.com/data',
  expect.objectContaining({ method: 'GET' })
);

// Access the raw call data
const [url, options] = mockFetch.mock.calls[0];
expect(url).toBe('https://api.example.com/data');
expect(options.method).toBe('GET');
```

### The globals Config

Remember that `globals: true` in `vite.config.ts`? That means you don't need to import `describe`, `it`, `expect`, or `vi` in your test files. They're just... there. Available everywhere, automatically.

Some projects prefer to import them explicitly:

```typescript
import { describe, it, expect, vi } from 'vitest';
```

Both approaches work. The explicit import makes dependencies clearer — you can see at a glance what test utilities a file uses. The globals approach saves boilerplate and makes tests slightly shorter.

Our project uses the globals approach for the API client tests (no imports needed) and explicit imports for the search tests (importing from `vitest`). Both are valid, and you can choose whichever style you prefer. Just be consistent within your project.

### Watch Out: Common Setup Mistakes

**Forgetting `environment: 'jsdom'`** — Without this, your tests won't have access to DOM APIs. Components that render HTML will crash with errors like `document is not defined`.

**Missing `setupFiles`** — If you skip this, `toBeInTheDocument()` and other jest-dom matchers won't work. You'll get cryptic errors like `toBeInTheDocument is not a function`.

**Wrong `include` pattern** — If your test files aren't being found, check this pattern. The file must match `src/**/*.test.{ts,svelte}`. If your tests are in a different directory, update the pattern.

**Not installing `jsdom`** — Vitest doesn't include jsdom by default. You must install it separately. If you see `Environment "jsdom" not found`, you forgot to install it.

**Forgetting to add `test` scripts** — Make sure `package.json` has `"test": "vitest run"` and `"test:watch": "vitest"` in the scripts section.

### Try It Yourself

Before moving on, try this:

1. Create a file `src/lib/example.test.ts`
2. Write a simple test:

```typescript
describe('math', () => {
  it('adds two numbers', () => {
    expect(2 + 2).toBe(4);
  });

  it('multiplies two numbers', () => {
    expect(3 * 4).toBe(12);
  });

  it('handles string concatenation', () => {
    expect('hello' + ' ' + 'world').toBe('hello world');
  });
});
```

3. Run `npm run test`
4. You should see your tests pass!

This might seem silly, but it confirms that your test setup is working correctly. If these tests fail, something is wrong with your configuration, and you should fix it before writing real tests.

Now try making one fail:

```typescript
it('intentionally fails', () => {
  expect(2 + 2).toBe(5);  // This will fail!
});
```

Run the tests again. You should see a red failure message. This is what failures look like. Get familiar with them — you'll see them often, and that's okay! Failures are how you find bugs.

---

## Chapter 31: Testing the API Client

### Why Test the API Client?

The API client is the bridge between your UI and the server. Every time a user searches for a story, downloads an EPUB, suggests a recommendation, or casts a vote, it's the API client that makes it happen.

If the API client breaks — wrong URL, wrong parameters, wrong headers — everything breaks. Users can't search, can't download, can't do anything. And the worst part? The errors might be subtle. A search might return no results instead of crashing. A download might silently fail.

That's why testing the API client is so important. It's the single point of failure for all your data flow. If the API client is solid, the rest of your application has a much better chance of working correctly.

The API client is also a great place to start testing because it's relatively simple — it's mostly functions that call `fetch()` and process the response. There's no DOM, no component rendering, no user interactions. Just functions, inputs, and outputs. This makes it the perfect place to practice your testing skills.

### The Big Idea: Mocking fetch

Our API client uses `fetch()` to talk to the server. In tests, we don't want to make real network requests — they're slow, they depend on the server being running, and they might return different results each time. So we replace `fetch` with a mock.

Think of it like a movie set. In a movie, when you see someone eating dinner, they're not really eating — they're pretending. The food is fake, the kitchen is fake, everything is a set. That's what mocking does for your tests. Instead of a real network request (the "real kitchen"), you have a fake fetch (the "movie set") that looks and acts like the real thing, but gives you complete control.

Here's the pattern used in `client.test.ts`:

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
});
```

Let's break this down line by line:

**`const mockFetch = vi.fn();`** — Creates a fake function that records every time it's called. You can set it up to return specific values, check what arguments it received, and count how many times it was called.

**`globalThis.fetch = mockFetch as unknown as typeof fetch;`** — Replaces the real `fetch` with our fake. The `as unknown as typeof fetch` is TypeScript type casting — we're telling TypeScript "trust me, this mock function is compatible with the real `fetch` type." It's a necessary bit of type gymnastics.

**`beforeEach(() => mockFetch.mockReset());`** — Clears all recorded calls and return values before each test. This is crucial! Without it, one test's mock setup could leak into the next test, causing confusing failures.

Now whenever any code calls `fetch()`, it calls our mock instead. We control what the mock returns, so we can simulate success, failure, slow responses, or any other scenario.

### Dynamic Imports: Importing After Mock Setup

Here's a subtlety that trips up almost everyone. When Vitest loads a test file, it immediately executes all the imports at the top. But we need to replace `fetch` *before* the API client module loads — because the client module uses `fetch`.

If we import the client at the top of the file:

```typescript
// BAD — this happens BEFORE our mock is set up
import { fetchExport } from './client';
```

Then `client.ts` gets loaded before we replace `fetch`, and our mock has no effect.

The solution? Dynamic imports:

```typescript
async function importClient() {
  return await import('./client');
}

describe('fetchExport', () => {
  it('returns parsed ExportResponse on success', async () => {
    const { fetchExport } = await importClient();  // Imported AFTER mock setup
    // ... test code
  });
});
```

By importing the client *inside* the test function (after the mock is set up), we guarantee that the client sees our mock `fetch` instead of the real one.

This pattern is important to remember. Whenever you mock a global like `fetch`, use dynamic imports for the module that uses it.

### Testing fetchExport(): Success and Error Cases

Let's test the most important function — the one that downloads fanfiction as EPUBs.

**Success case:**

```typescript
describe('fetchExport', () => {
  it('returns parsed ExportResponse on success', async () => {
    const { fetchExport } = await importClient();

    // Set up the mock to return a successful response
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        err: 0,
        url_id: 'x1',
        meta: {
          title: 'Test',
          author: 'A',
          words: 100,
          chapters: 1,
          status: 'complete',
        },
        epub_url: '/cache/epub/x1?h=abc',
      }),
    });

    const res = await fetchExport('https://ao3.org/works/1');

    expect(res.err).toBe(0);
    expect(res.url_id).toBe('x1');
    expect(res.epub_url).toBe('/cache/epub/x1?h=abc');
  });
```

This test says: "When the server responds with success and valid JSON, `fetchExport` should return the parsed data correctly and completely." We mock `fetch` to return a success response, call `fetchExport`, and verify the result.

Notice the mock response structure. It matches what the real API would return: `{ ok: true, json: async () => ({...}) }`. The `json` property is a function (not an object) because real `fetch` responses return `response.json()` as a Promise.

**Error case:**

```typescript
  it('throws ApiError on HTTP failure', async () => {
    const { fetchExport, ApiError } = await importClient();

    // Mock a server error
    mockFetch.mockResolvedValue({
      ok: false,
      status: 500,
      text: async () => 'boom',
    });

    await expect(fetchExport('u')).rejects.toBeInstanceOf(ApiError);
  });
});
```

This test says: "When the server returns a 500 error, `fetchExport` should throw an `ApiError` with the appropriate error information." We mock a failed response and verify that the right error type is thrown.

The `rejects` matcher is important here — it checks that a Promise rejects (throws) rather than resolving. Without `rejects`, you'd be checking the resolved value, which doesn't exist when the Promise rejects.

### Testing buildSearchQuery(): All Parameter Types

The search query builder is pure logic — no network calls, no side effects. These are the easiest and most satisfying tests to write because they're so fast and reliable.

```typescript
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
    expect(buildSearchQuery(f)).toContain('source=archiveofourown.org');
  });

  it('includes date_from in ISO format', () => {
    const f = defaultFilters();
    f.date_from = '2024-01-01T00:00:00Z';
    expect(buildSearchQuery(f)).toContain('date_from=2024-01-01T00%3A00%3A00Z');
  });

  it('includes include_tags', () => {
    const f = defaultFilters();
    f.include_tags = '1:Harry Potter,4:Fluff';
    expect(buildSearchQuery(f)).toContain('include_tags=1%3AHarry+Potter%2C4%3AFluff');
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
```

That's 10 tests for one function! Each one checks a different parameter or edge case. Together, they give you confidence that the query builder handles everything correctly.

Notice the pattern in each test:
1. Create a default filter object
2. Change exactly one thing
3. Build the query string
4. Check that the right parameter appears (or doesn't appear)

This is a great testing pattern: test one thing at a time. If a test fails, you know exactly which parameter is broken.

### Testing defaultFilters(): Correct Defaults

Before testing the query builder, we need to make sure the default filters are set up correctly:

```typescript
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
```

This is a sanity check. If the defaults are wrong, everything else built on top of them will be wrong too. For example, if `defaultFilters` returns `page: 0` instead of `page: 1`, the query builder tests would pass (they'd see `page=0` in the query), but the actual search would fail because page 0 doesn't exist.

### Testing submitSuggestion(): POST Request Body

The suggestion endpoint sends a POST request with a specific JSON body. We need to verify that the body contains the right data:

```typescript
describe('submitSuggestion', () => {
  it('POSTs json body with url_id and suggested_url', async () => {
    const { submitSuggestion } = await importClient();

    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, suggestion_id: 42 }),
    });

    const res = await submitSuggestion('seed1', 'https://ao3.org/works/9', 'great');

    expect(res.err).toBe(0);
    expect(res.suggestion_id).toBe(42);

    // Check what was sent to the server
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toContain('/recommendations/suggest');
    expect(init.method).toBe('POST');

    const body = JSON.parse(init.body);
    expect(body.url_id).toBe('seed1');
    expect(body.suggested_url).toBe('https://ao3.org/works/9');
    expect(body.comment).toBe('great');
  });
});
```

Here's the cool part: `mockFetch.mock.calls` is an array of every call made to the mock. `mockFetch.mock.calls[0]` is the first call. It's an array of `[url, init]`, matching the `fetch(url, init)` signature.

We can inspect:
- The URL — does it point to the right endpoint?
- The method — is it POST (not GET)?
- The body — does it contain the right data?

This is incredibly useful. You're not just testing that the function returns the right thing — you're testing that it *sends* the right thing to the server. This catches bugs like wrong endpoint URLs, missing parameters, or incorrect HTTP methods.

### Testing castVote(): Vote Values

Voting is simple but important — a bug here could mean users' votes aren't recorded:

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

    const body = JSON.parse(mockFetch.mock.calls[0][1].body);
    expect(body.suggestion_id).toBe(7);
    expect(body.vote).toBe(1);
  });
});
```

Again, we're checking both the response (the new score) and the request (the suggestion ID and vote value). This ensures the function works end-to-end.

### The Client Test Walkthrough: All 6 Tests

Let's put it all together. Here's the complete `client.test.ts` file, with all six tests. I'll walk through each section so you understand exactly what's happening:

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest';
```

First, we import everything we need from Vitest. The `vi` object is Vitest's utility for creating mocks. The `beforeEach` function lets us run code before every test.

```typescript
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;
```

We create a mock function and replace the global `fetch`. Now every `fetch()` call in our code goes through this mock.

```typescript
beforeEach(() => {
  mockFetch.mockReset();
});
```

Before every test, we reset the mock. This is like wiping the slate clean — no leftover data from previous tests.

```typescript
async function importClient() {
  return await import('./client');
}
```

This helper function dynamically imports the API client. We do this inside tests (not at the top of the file) so that the mock is set up before the client module loads.

Now the tests:

```typescript
describe('buildQuery', () => {
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

This test verifies that when you search by URL (not by ID), the query includes the `q` parameter but not `url_id`. We mock `fetch` to return success, call `fetchRecommendations`, and then inspect the URL that was passed to `fetch`.

```typescript
  it('includes url_id when provided', async () => {
    const { fetchRecommendations } = await importClient();
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, recommendations: [] }),
    });
    await fetchRecommendations(undefined, 'abc123', 5);
    const called = mockFetch.mock.calls[0][0] as string;
    expect(called).toContain('url_id=abc123');
    expect(called).toContain('n=5');
    expect(called).not.toContain('q=');
  });
});
```

And this test verifies the opposite: when you search by ID (not by URL), the query includes `url_id` but not `q`. Together, these two tests cover both search modes.

The remaining tests follow the same pattern — mock, call, inspect. Each one verifies a specific function behaves correctly. After all six tests, you have confidence that every API client function works as expected. If any function breaks — wrong URL, wrong parameters, wrong response handling — the tests will catch it immediately.

The tests are also incredibly fast. All six run in about 23 milliseconds. That's instant feedback — you make a change, run the tests, and know immediately if anything broke.

### The Search Test Walkthrough: All 14 Tests

Now let's look at `search.test.ts`. This file tests the search query builder and the filter defaults — 14 tests in total. Unlike the API client tests, these tests don't need any mocking because `buildSearchQuery` and `defaultFilters` are pure functions with no external dependencies.

```typescript
import { describe, it, expect } from 'vitest';
import {
  buildSearchQuery,
  defaultFilters,
  SORT_OPTIONS,
  COMPLETE_OPTIONS,
  SOURCE_OPTIONS,
} from './search';
```

Notice the explicit imports from `vitest` here, unlike the API client tests which use globals. Both approaches work — this file just chose the explicit style. We also import the constants `SORT_OPTIONS`, `COMPLETE_OPTIONS`, and `SOURCE_OPTIONS` so we can verify they exist and have the expected values.

**The defaultFilters test:**

```typescript
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
```

This single test checks six properties of the default filter object. It's a sanity check — if the defaults are wrong, every search built on top of them will be wrong too. For example, if `defaultFilters` returns `page: 0` instead of `page: 1`, the search would request a page that doesn't exist on the server.

**The buildSearchQuery tests:**

These 10 tests follow a consistent pattern. Each one:
1. Creates a default filter
2. Changes exactly one property
3. Builds the query string
4. Checks that the right parameter appears

```typescript
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
```

The first test verifies that an empty filter produces an empty query string. This is important because the API might reject requests with unnecessary parameters.

The second test verifies that when you set a search term, it appears in the URL-encoded query string. Notice that spaces become `+` — that's standard URL encoding.

```typescript
  it('includes min_words when set', () => {
    const f = defaultFilters();
    f.min_words = 10000;
    expect(buildSearchQuery(f)).toContain('min_words=10000');
  });
```

This verifies that word count filters are included. If someone sets `min_words: 10000`, the query should include `min_words=10000`.

```typescript
  it('includes complete=true', () => {
    const f = defaultFilters();
    f.complete = true;
    expect(buildSearchQuery(f)).toContain('complete=true');
  });
```

This verifies that the "complete only" filter works. When a user only wants finished stories, this parameter tells the server to filter accordingly.

```typescript
  it('includes sort', () => {
    const f = defaultFilters();
    f.sort = 'updated';
    expect(buildSearchQuery(f)).toContain('sort=updated');
  });

  it('includes source', () => {
    const f = defaultFilters();
    f.source = 'archiveofourown.org';
    expect(buildSearchQuery(f)).toContain('source=archiveofourown.org');
  });

  it('includes date_from in ISO format', () => {
    const f = defaultFilters();
    f.date_from = '2024-01-01T00:00:00Z';
    expect(buildSearchQuery(f)).toContain(
      'date_from=2024-01-01T00%3A00%3A00Z'
    );
  });

  it('includes include_tags', () => {
    const f = defaultFilters();
    f.include_tags = '1:Harry Potter,4:Fluff';
    expect(buildSearchQuery(f)).toContain(
      'include_tags=1%3AHarry+Potter%2C4%3AFluff'
    );
  });
```

Each of these tests follows the same pattern: change one property, verify it appears in the query. The date_from test is interesting — the colons (`:`) get encoded as `%3A` and commas as `%2C`. That's correct URL encoding behavior.

```typescript
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
```

The last two tests check conditional behavior: page 1 is omitted (because it's the default), but page 3 is included. This is a common pattern — default values are often omitted from query strings to keep URLs clean.

**The constants tests:**

```typescript
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

These three tests verify that the option arrays exist and have the expected number of entries. They seem trivial, but they serve as safety nets. If someone accidentally removes an option from `COMPLETE_OPTIONS`, the test catches it immediately. It's the kind of test that saves you from a subtle bug that might not be caught by manual testing. Automated tests don't forget to check things the way humans do.

### Watch Out: Mock Leaks

The most common mistake in API client tests is forgetting to reset mocks between tests. If one test sets up `mockFetch` to return a success response, and the next test doesn't reset it, the second test will see the first test's mock data. This causes confusing failures that seem unrelated to the actual bug.

The `beforeEach` block prevents this:

```typescript
beforeEach(() => {
  mockFetch.mockReset();
});
```

Every test starts with a clean slate. Always, always, always include this when you're mocking.

### Watch Out: Async Testing

API client functions are async — they return Promises. You need to use `await` when calling them:

```typescript
// WRONG — this tests nothing!
it('works', () => {
  fetchExport('url');  // Promise is created but never awaited
});

// RIGHT
it('works', async () => {
  const res = await fetchExport('url');  // Awaited properly
  expect(res.err).toBe(0);
});
```

If you forget `async` and `await`, your test will always pass — because it never actually checks the result. The Promise just gets created and thrown away. This is a sneaky bug because the test appears to pass, but it's not testing anything!

Always double-check: is the function async? Does it return a Promise? If yes, you need `async` on the test function and `await` on the call.

### Try It Yourself

Try adding a test for `fetchRecommendations` with a different scenario. What happens when the API returns an error? Write a test that:

1. Mocks `fetch` to return `{ ok: false, status: 404, text: async () => 'not found' }`
2. Calls `fetchRecommendations`
3. Verifies it throws an `ApiError`

Here's a template to get you started:

```typescript
it('throws ApiError on 404', async () => {
  const { fetchRecommendations, ApiError } = await importClient();
  mockFetch.mockResolvedValue({
    ok: false,
    status: 404,
    text: async () => 'not found',
  });
  await expect(
    fetchRecommendations('https://ao3.org/works/999')
  ).rejects.toBeInstanceOf(ApiError);
});
```

Add this to the `buildQuery` describe block and run the tests. Does it pass? If not, figure out why and fix it. This is a great exercise because it combines everything we've learned: mocking, async, error handling, and assertions.

Also try testing `fetchRecommendations` with a successful response. Mock it to return some recommendations and verify that the function returns them correctly. Practice makes perfect!

---

## Chapter 32: Testing Components

### What Makes Component Testing Different

Unit testing a function is straightforward: call it, check the result. But a component is a living, breathing thing that renders HTML, responds to events, and updates over time. Testing components is more like testing a mini-application.

Think of it this way: testing a function is like testing a recipe — follow the steps, check the result. Testing a component is like testing a whole restaurant — you need to check that the menu displays correctly, that orders are taken properly, that food comes out right, and that the bill is calculated correctly. There's a lot more going on.

Here's what makes component testing different:

- **Rendering**: Components produce HTML. You need to check that the right elements appear on the screen.
- **User interaction**: Users click buttons, type in inputs, scroll, and hover. You need to simulate these events and verify the component responds correctly.
- **Asynchronous updates**: Some things happen after a delay — API calls return, timers fire, animations complete. You need to wait for these updates before checking the result.
- **State changes**: Components update their display when state changes. You need to verify that the updates happen correctly and appear in the DOM.

The `@testing-library/svelte` library handles all of this. It gives you tools to render components, find elements, fire events, and wait for updates. It's designed to test what the user *sees*, not how the component is implemented internally.

### The render() Function

The `render` function mounts a Svelte component into a fake DOM (provided by jsdom). It returns the component instance and some utilities:

```typescript
import { render, screen } from '@testing-library/svelte';
import DownloadTab from '$lib/components/DownloadTab.svelte';

render(DownloadTab);
```

After rendering, the component's HTML is available in the fake DOM. You can query it, interact with it, and verify its state.

The `render` function does several things behind the scenes:
1. Creates a fake DOM container (a `<div>` element)
2. Mounts the Svelte component into it
3. Runs any reactive effects and initial updates
4. Returns utilities for querying and interacting with the component

You don't need to worry about any of this — just call `render` and start testing. The testing library handles all the complexity of mounting and unmounting components.

One thing to note: `render` returns an object with the component instance and a `unmount` function. In most cases, you don't need to call `unmount` manually — the testing library cleans up after each test automatically. But if you need to manually unmount (for example, to test cleanup behavior), you can:

### Finding Elements: screen.getByText(), screen.getByLabelText()

The `screen` object has methods for finding elements in the rendered DOM. Here are the most useful ones:

**`screen.getByText(text)`** — Find an element by its visible text content. Great for headings, labels, and buttons.

```typescript
screen.getByText('Download');
screen.getByText('My Story');
screen.getByText(/error/i);  // Regular expression for flexible matching
```

**`screen.getByLabelText(text)`** — Find a form element by its label. This is the best way to find inputs, selects, and textareas.

```typescript
const input = screen.getByLabelText('Fanfiction URL') as HTMLInputElement;
const select = screen.getByLabelText('Sort by') as HTMLSelectElement;
```

**`screen.getByRole(role, options)`** — Find an element by its ARIA role. Useful for semantic queries.

```typescript
screen.getByRole('button', { name: 'Download' });
screen.getByRole('textbox', { name: 'Search' });
screen.getByRole('heading', { name: 'Results' });
```

**`screen.queryByText(text)`** — Like `getByText`, but returns `null` instead of throwing if not found. Use this when you're not sure if an element exists.

```typescript
const error = screen.queryByText(/error/i);
if (error) {
  // Error message is shown
}
```

If the element isn't found, `getByText` throws a helpful error message showing you what elements are actually in the DOM. This is one of the best things about Testing Library — when things go wrong, it tells you exactly what's available.

### User Events: fireEvent

The `fireEvent` object simulates user interactions. Here are the most common events:

**Clicking a button:**

```typescript
import { fireEvent } from '@testing-library/svelte';

await fireEvent.click(screen.getByText('Download'));
```

**Typing in an input:**

```typescript
const input = screen.getByLabelText('Fanfiction URL');
await fireEvent.input(input, { target: { value: 'https://ao3.org/works/1' } });
```

**Changing a select:**

```typescript
const select = screen.getByLabelText('Sort by');
await fireEvent.change(select, { target: { value: 'updated' } });
```

**Focusing and blurring:**

```typescript
await fireEvent.focus(input);
await fireEvent.blur(input);
```

Note the `await` — events can trigger async updates, so always await them. If you forget the `await`, the event might not fully process before your assertions run, causing flaky tests.

### waitFor(): Waiting for Async Updates

Sometimes an element doesn't appear immediately. Maybe it appears after an API call completes, or after a state update. The `waitFor` function polls the DOM until a condition is met:

```typescript
import { waitFor } from '@testing-library/svelte';

// Wait for an element to appear
await waitFor(() => {
  expect(screen.getByText('My Story')).toBeTruthy();
});

// Wait for an element to disappear
await waitFor(() => {
  expect(screen.queryByText('Loading...')).toBeNull();
});

// Wait for a custom condition
await waitFor(() => {
  expect(screen.getByText('EPUB')).toBeInTheDocument();
});
```

This is essential for testing components that fetch data. You render the component, trigger the action, and then wait for the result to appear. Without `waitFor`, you'd be checking for elements that haven't rendered yet.

### Testing DownloadTab: The Complete Flow

Let's test the DownloadTab component — the one where users paste a fanfiction URL and download it as an EPUB. This is the most interesting component test because it involves user interaction, API calls, and conditional rendering.

First, here's the setup:

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
});

// Import the component lazily so $lib alias resolves after build.
async function loadDownloadTab() {
  return await import('$lib/components/DownloadTab.svelte');
}
```

The `loadDownloadTab` function uses a dynamic import — just like in the API client tests. This ensures the `$lib` alias resolves correctly in the test environment.

**Test 1: Empty URL shows error**

```typescript
it('shows an error when URL is empty', async () => {
  const { default: DownloadTab } = await loadDownloadTab();
  render(DownloadTab);
  await fireEvent.click(screen.getByText('Download'));
  expect(screen.getByText(/paste a fanfiction URL/i)).toBeTruthy();
});
```

This test says: "If the user clicks Download without entering a URL, they should see a helpful error message." We don't need to mock `fetch` because no API call should be made — the validation catches it first.

The flow is:
1. Load the component
2. Render it
3. Click the Download button (without typing anything)
4. Check that an error message appears

The regex `/paste a fanfiction URL/i` is case-insensitive (the `i` flag), so it matches "Paste a fanfiction URL" or "PASTE A FANFICTION URL" or any variation.

**Test 2: Successful download**

```typescript
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
  const input = screen.getByLabelText('Fanfiction URL') as HTMLInputElement;
  await fireEvent.input(input, { target: { value: 'https://archiveofourown.org/works/1' } });
  await fireEvent.click(screen.getByText('Download'));

  await waitFor(() => expect(screen.getByText('My Story')).toBeTruthy());
  expect(screen.getByText('EPUB')).toBeTruthy();
  expect(screen.getByText('HTML')).toBeTruthy();
});
```

This is the full flow:
1. Mock the API to return success with story data
2. Render the component
3. Type a URL into the input (using `fireEvent.input`)
4. Click the Download button (using `fireEvent.click`)
5. Wait for the story title to appear (using `waitFor`)
6. Verify the download links are shown (EPUB and HTML)

The `waitFor` is crucial here. After clicking Download, the component makes an API call, processes the response, and updates the DOM. This takes time (even in tests), so we need to wait for the title to appear before checking the download links.

### Testing with Mock Fetch

Component tests that interact with APIs need the same mock setup as API client tests:

```typescript
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
});
```

Without this, the component would try to make a real network request, which would fail in the test environment (there's no server running during tests).

The mock setup is identical to what we used in `client.test.ts`. The pattern is the same: create a mock, replace `fetch`, reset before each test. Consistency in your test setup makes your tests easier to read and maintain.

### Handling Loading States

Some components show a spinner or loading message while waiting for data. You can test this:

```typescript
it('shows loading state while fetching', async () => {
  // Make fetch hang (never resolve)
  mockFetch.mockReturnValue(new Promise(() => {}));

  render(DownloadTab);
  const input = screen.getByLabelText('Fanfiction URL');
  await fireEvent.input(input, { target: { value: 'https://ao3.org/works/1' } });
  await fireEvent.click(screen.getByText('Download'));

  // Should show loading indicator
  expect(screen.getByText('Loading...')).toBeTruthy();
});
```

This is a clever trick: we mock `fetch` to return a Promise that never resolves. This simulates a slow network connection, like a user on a poor mobile signal. While the API call is "in progress," the component should show a loading indicator.

### Handling Errors

What happens when the API returns an error? The component should show a user-friendly error message, not crash:

```typescript
it('shows error when API fails', async () => {
  mockFetch.mockResolvedValue({
    ok: false,
    status: 500,
    text: async () => 'Internal Server Error',
  });

  render(DownloadTab);
  const input = screen.getByLabelText('Fanfiction URL');
  await fireEvent.input(input, { target: { value: 'https://ao3.org/works/1' } });
  await fireEvent.click(screen.getByText('Download'));

  await waitFor(() => {
    expect(screen.getByText(/error/i)).toBeTruthy();
  });
});
```

This test verifies that when the server returns a 500 error, the component shows an error message instead of crashing or showing nothing. The user sees something helpful and informative, and you sleep better at night.

### The DownloadTab.test.ts Walkthrough

Here's the complete test file, with both tests. I'll walk through each part so you understand the full picture:

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
```

We import everything we need: test utilities from Vitest and testing helpers from `@testing-library/svelte`. The `render` function mounts components, `fireEvent` simulates user interactions, `screen` lets us find elements, and `waitFor` handles async updates.

```typescript
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
});
```

Same mock setup as the API client tests. We replace `fetch` with a mock and reset it before each test.

```typescript
async function loadDownloadTab() {
  return await import('$lib/components/DownloadTab.svelte');
}
```

Dynamic import to ensure the mock is set up before the component loads.

**Test 1: Empty URL shows error**

```typescript
it('shows an error when URL is empty', async () => {
  const { default: DownloadTab } = await loadDownloadTab();
  render(DownloadTab);
  await fireEvent.click(screen.getByText('Download'));
  expect(screen.getByText(/paste a fanfiction URL/i)).toBeTruthy();
});
```

Step by step:
1. Load the component (the `default` export is the component itself)
2. Render it into the fake DOM
3. Find the "Download" button by its text and click it
4. Check that an error message matching `/paste a fanfiction URL/i` appears

The test passes because the component has validation that checks if the URL is empty before making an API call. The regex pattern `/paste a fanfiction URL/i` is case-insensitive (the `i` flag), so it matches any capitalization.

**Test 2: Successful download**

```typescript
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
```

We set up the mock to return a full success response. Notice the response matches the exact structure of the real API — `err: 0` means success, `url_id` is a unique identifier, `meta` has all the story information, and `epub_url`/`html_url` are the download links.

```typescript
  render(DownloadTab);
  const input = screen.getByLabelText('Fanfiction URL') as HTMLInputElement;
  await fireEvent.input(input, {
    target: { value: 'https://archiveofourown.org/works/1' },
  });
  await fireEvent.click(screen.getByText('Download'));
```

We render the component, find the input by its label, type a URL into it, and click Download. The `as HTMLInputElement` cast lets us treat it as an input element.

```typescript
  await waitFor(() => expect(screen.getByText('My Story')).toBeTruthy());
  expect(screen.getByText('EPUB')).toBeTruthy();
  expect(screen.getByText('HTML')).toBeTruthy();
});
```

After clicking Download, the component makes an API call, processes the response, and updates the DOM. We use `waitFor` to wait for the title "My Story" to appear, then verify the download links are shown.

This test covers the complete happy path: user enters URL → clicks Download → sees story details and download links. If any step in this chain breaks, the test fails.

### The util.test.ts Walkthrough

Now let's look at the utility function tests. These are pure unit tests — no mocking, no DOM, just functions and results. They're the simplest and fastest tests in our project.

```typescript
import { describe, it, expect } from 'vitest';
import {
  formatWords,
  detectSite,
  stripHtml,
  relativeTime,
  cacheUrl,
} from './util';
```

We import each utility function by name. This is good practice — you know exactly which functions are being tested.

**formatWords — formatting numbers with separators:**

```typescript
describe('formatWords', () => {
  it('adds thousands separators', () => {
    expect(formatWords(1234567)).toBe('1,234,567');
    expect(formatWords(50000)).toBe('50,000');
    expect(formatWords(0)).toBe('0');
  });
});
```

Three assertions in one test. They're all testing the same behavior — that `formatWords` adds commas as thousands separators. The three cases cover large numbers, medium numbers, and zero. This is a common pattern: testing multiple inputs for the same function when they all verify the same behavior.

**detectSite — identifying fanfiction sites:**

```typescript
describe('detectSite', () => {
  it('detects AO3', () => {
    expect(detectSite('https://archiveofourown.org/works/1')).toBe('AO3');
  });

  it('detects FanFiction.net', () => {
    expect(detectSite('https://www.fanfiction.net/s/1/1/Title')).toBe(
      'FanFiction.net'
    );
  });

  it('detects XenForo forums', () => {
    expect(detectSite('https://forums.spacebattles.com/threads/x.1')).toBe(
      'Forum'
    );
  });

  it('returns Unknown for unrecognized', () => {
    expect(detectSite('https://example.com/story')).toBe('Unknown');
  });
});
```

Four tests, one per site type. Each test is a simple input-output check. The most important test is the last one — the "Unknown" case. Without it, you wouldn't know what happens when someone pastes a URL from a site you don't recognize. Does it crash? Return undefined? Return an empty string? The test documents the expected behavior: return "Unknown."

**stripHtml — cleaning up HTML content:**

```typescript
describe('stripHtml', () => {
  it('removes tags and decodes entities', () => {
    expect(stripHtml('<p>Hello &amp; welcome</p>')).toBe('Hello & welcome');
    expect(stripHtml('<br>line1<br>line2')).toBe('line1 line2');
    expect(stripHtml('')).toBe('');
  });
});
```

Three scenarios in one test:
1. Tags are removed and HTML entities are decoded (`&amp;` becomes `&`)
2. Self-closing tags like `<br>` become spaces
3. Empty input returns empty output

The entity decoding test is particularly important — it ensures that characters like `&`, `<`, and `>` are displayed correctly to users, not shown as raw HTML entities.

**relativeTime — time formatting:**

```typescript
describe('relativeTime', () => {
  it('returns recent for now', () => {
    expect(relativeTime(new Date().toISOString())).toContain('minute');
  });

  it('returns empty for empty input', () => {
    expect(relativeTime('')).toBe('');
  });
});
```

Two tests for time formatting. The first checks that a timestamp from "now" returns something containing "minute" (like "5 minutes ago"). The second checks that empty input returns empty string — no crash, no error, just silence.

**cacheUrl — building cache paths:**

```typescript
describe('cacheUrl', () => {
  it('builds correct cache path', () => {
    expect(cacheUrl('epub', 'abc', 'def')).toBe('/cache/epub/abc?h=def');
  });
});
```

One test, one assertion. This verifies that the cache URL is built with the correct format: `/cache/{type}/{id}?h={hash}`. If the format changes, this test catches it.

Nine tests total, covering five utility functions. Notice how clean and simple these are. No mocking, no async, no DOM. Just pure functions in, results out. This is the easiest kind of testing, and it's incredibly valuable. These tests catch bugs in your utility functions before they can affect the rest of your application.

### Running All Tests

When you run `npm run test`, you should see all your tests pass:

```
 ✓ src/lib/util.test.ts (9 tests) 12ms
 ✓ src/lib/api/client.test.ts (6 tests) 23ms
 ✓ src/lib/api/search.test.ts (14 tests) 8ms
 ✓ src/lib/components/DownloadTab.test.ts (2 tests) 156ms

 Test Files  4 passed (4)
      Tests  31 passed (31)
   Start at  14:32:07
   Duration  1.2s
```

All green! Every function, every component, every edge case — verified by a computer that never gets tired.

If you see red (failed tests), don't panic! Read the error message carefully. It tells you:
- Which test failed
- What was expected
- What was actually returned

Most test failures are simple mistakes: wrong expected value, missing `await`, or a mock that wasn't reset. Fix the mistake, run the tests again, and keep going.

### Practice: Write a Test for RecommendationsTab

Here's a challenge for you. Try writing a test for the `RecommendationsTab` component. Here's what you need to know:

1. The component calls `fetchRecommendations()` when the user clicks "Get Recommendations"
2. It displays a list of recommended stories
3. It shows an error message if the fetch fails

Your test should:

1. Mock `fetch` to return a success response with recommendations
2. Render the component
3. Click the "Get Recommendations" button
4. Wait for the recommendations to appear
5. Verify at least one recommendation is displayed

Here's a starter template:

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
});

async function loadRecommendationsTab() {
  return await import('$lib/components/RecommendationsTab.svelte');
}

describe('RecommendationsTab', () => {
  it('renders recommendations on success', async () => {
    const { default: RecommendationsTab } = await loadRecommendationsTab();
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        err: 0,
        recommendations: [
          {
            title: 'Great Story',
            author: 'Writer1',
            url: 'https://ao3.org/works/100',
          },
          {
            title: 'Another Story',
            author: 'Writer2',
            url: 'https://ao3.org/works/200',
          },
        ],
      }),
    });

    render(RecommendationsTab);
    await fireEvent.click(screen.getByText('Get Recommendations'));

    await waitFor(() => {
      expect(screen.getByText('Great Story')).toBeTruthy();
    });
    expect(screen.getByText('Another Story')).toBeTruthy();
  });

  it('shows error when fetch fails', async () => {
    const { default: RecommendationsTab } = await loadRecommendationsTab();
    mockFetch.mockResolvedValue({
      ok: false,
      status: 500,
      text: async () => 'Server error',
    });

    render(RecommendationsTab);
    await fireEvent.click(screen.getByText('Get Recommendations'));

    await waitFor(() => {
      expect(screen.getByText(/error/i)).toBeTruthy();
    });
  });
});
```

Give it a try! Modify the mock data, change the button text, add more assertions, experiment with different scenarios. The more you practice, the more natural testing will feel.

Here are some additional challenges:
- What if the API returns an empty list of recommendations?
- What if the user clicks the button twice quickly?
- What if the recommendation has a really long title?

Each of these scenarios is a potential bug waiting to happen. Writing tests for them is how you prevent those bugs from ever reaching your users.

### Watch Out: Common Component Testing Mistakes

**Forgetting to await fireEvent** — Always `await fireEvent.click(...)`. Without the `await`, the event might not fully process before your assertions run. This causes flaky tests — they pass sometimes and fail other times.

**Using getBy when element might not exist** — If you're not sure an element is in the DOM, use `queryByText` instead. `getByText` throws an error if not found; `queryByText` returns `null`. Use `getByText` when you *expect* the element to be there (it's a test assertion in itself).

**Not waiting for async updates** — If a component fetches data and updates its display, you need `waitFor` to wait for the update. Don't just check immediately after clicking. The API call takes time, even in tests.

**Testing implementation details** — Don't check that a specific internal variable has a specific value. Check that the *user* can see what they expect to see. "The story title is visible" is better than "the component's `story` property equals X."

**Mocking too much** — Don't mock everything. Mock only what you need to control (like `fetch`). Let the component do its real work — rendering, state management, event handling. The more real code you test, the more confident you can be.

**Ignoring the DOM output** — When a test fails, look at the actual DOM output. Testing Library shows you what elements are in the DOM, which helps you understand why your query failed. It's like having a debugger for your tests.

### What You've Learned

You've learned the fundamentals of testing, and you should be proud of yourself! Testing is one of those skills that separates good developers from great ones. Here's a recap of everything you've mastered:

**Why test** — Tests catch bugs early (before users find them), let you refactor safely (change code without fear), document behavior (tests are living documentation), and help you sleep better at night. The investment in writing tests pays for itself many times over in reduced bugs and faster development.

**How to set up Vitest** — Install the packages (`vitest`, `@testing-library/svelte`, `@testing-library/jest-dom`, `jsdom`), configure `vite.config.ts` with the test section, create the `test-setup.ts` file, and run `npm run test`. The setup is minimal but powerful.

**How to test functions** — Mock external dependencies (like `fetch`), test inputs and outputs, check both success and error cases, and use `beforeEach` to reset state between tests. Pure functions are the easiest to test — just give them inputs and check outputs.

**How to test components** — Render the component with `render()`, find elements with `screen` queries (`getByText`, `getByLabelText`), fire events with `fireEvent` (`click`, `input`), and wait for async updates with `waitFor`. Test what the user sees, not how the component is implemented.

**How to test API calls** — Mock `fetch` with `vi.fn()`, verify request bodies and responses, test both success and error scenarios, and use dynamic imports to ensure mocks are set up before the module loads.

Testing is a skill that gets better with practice. The more you write tests, the more natural it becomes. At first, you might feel like tests are slowing you down — extra code to write, extra things to think about. But very quickly, you'll start to feel the opposite: writing code *without* tests feels reckless, like driving without a seatbelt.

The payoff is enormous — confident refactoring, fewer bugs in production, and a codebase that's a joy to maintain. Your future self (and your teammates) will thank you for every test you write.

### The Testing Mindset

Beyond the technical skills, testing changes how you think about code. When you write tests, you start asking yourself questions before you even start coding:

- What should this function do?
- What inputs will it receive?
- What edge cases might come up?
- How should it handle errors?

This kind of thinking leads to better code design. Functions become smaller and more focused (because they're easier to test). Components become clearer in their responsibilities. API calls become more robust (because you've tested the error cases).

Testing also gives you the courage to make bold changes. Want to rename a function that's used in 15 places? With tests, you rename it, run `npm run test`, fix any failures, and you're done. Without tests, you'd spend an hour manually checking every usage.

### Common Testing Patterns

Here are some patterns you'll use again and again:

**The Arrange-Act-Assert pattern:**
```typescript
it('calculates total correctly', () => {
  // Arrange — set up the data
  const items = [{ price: 10 }, { price: 20 }, { price: 30 }];

  // Act — call the function
  const total = calculateTotal(items);

  // Assert — check the result
  expect(total).toBe(60);
});
```

**The mock-and-inspect pattern:**
```typescript
it('calls API with correct parameters', async () => {
  // Mock the API
  mockFetch.mockResolvedValue({ ok: true, json: async () => ({}) });

  // Call the function
  await fetchData('test');

  // Inspect what was sent
  const [url, options] = mockFetch.mock.calls[0];
  expect(url).toContain('q=test');
  expect(options.method).toBe('GET');
});
```

**The edge case pattern:**
```typescript
describe('formatEmail', () => {
  it('formats valid email', () => { /* ... */ });
  it('returns empty for empty input', () => { /* ... */ });
  it('handles special characters', () => { /* ... */ });
  it('handles very long email', () => { /* ... */ });
});
```

### In the Next Part...

In the next part, we'll deploy our application to the real world. But first, take a moment to celebrate: you now know how to write tests that verify your code works correctly. That's a superpower most developers take years to develop. You've earned it!

### Try It Yourself: Final Challenge

Before moving on, try this challenge. Open `src/lib/api/client.test.ts` and add a test for `fetchRecommendations` that checks the URL structure:

```typescript
it('builds correct URL with query and count', async () => {
  const { fetchRecommendations } = await importClient();
  mockFetch.mockResolvedValue({
    ok: true,
    json: async () => ({ err: 0, recommendations: [] }),
  });

  await fetchRecommendations('https://ao3.org/works/42', undefined, 10);

  const url = mockFetch.mock.calls[0][0] as string;
  expect(url).toContain('q=https%3A%2F%2Fao3.org%2Fworks%2F42');
  expect(url).toContain('n=10');
  expect(url).not.toContain('url_id=');
});
```

Run `npm run test` and verify it passes. Then modify the test to use a `url_id` instead of a query, and verify the URL changes accordingly:

```typescript
it('builds correct URL with url_id', async () => {
  const { fetchRecommendations } = await importClient();
  mockFetch.mockResolvedValue({
    ok: true,
    json: async () => ({ err: 0, recommendations: [] }),
  });

  await fetchRecommendations(undefined, 'xyz789', 5);

  const url = mockFetch.mock.calls[0][0] as string;
  expect(url).toContain('url_id=xyz789');
  expect(url).toContain('n=5');
  expect(url).not.toContain('q=');
});
```

This exercise reinforces the patterns you've learned and gives you confidence in writing your own tests. Every test you write makes you a better developer.

You're ready for the next chapter. Let's keep building!
# Part 8: Deployment and Beyond

---

# Chapter 33: Building for Production

## Dev vs Production: The Difference

Throughout this book, you've been running your FicHub app with commands like `npm run dev`. That command starts a development server — a special mode that's designed for *you*, the developer. It watches for file changes, gives you helpful error messages, and reloads the page automatically when you save a file.

But here's the thing: the development server is slow. It does extra work on every request. It doesn't optimize your code. It doesn't compress anything. It's like a construction site — it gets the job done, but it's messy and noisy.

A production build is the opposite. It's like the finished house — clean, optimized, and ready for visitors. The code is minified (all the extra whitespace removed), bundled (all your files combined into efficient chunks), and optimized (only the code you actually use is included).

Let's see the difference in action:

```bash
# Development mode — slow, verbose, watches for changes
npm run dev

# Production build — fast, optimized, ready for deployment
npm run build
```

When you run `npm run dev`, you'll see something like:

```
  VITE v5.4.0  ready in 312 ms

  ➜  Local:   http://localhost:5173/
  ➜  Network: use --host to expose
```

That `312 ms` startup time is fine for development. But what happens when 100 people try to access your app at the same time?

The development server would slow to a crawl. It's not built for that. It's built for one developer making changes and seeing the results immediately.

A production build, on the other hand, is built for speed. The server just serves pre-built files — no processing, no watching, no extra work. It's like the difference between a chef cooking your meal to order versus picking up a pre-packaged meal from the fridge. Both feed you, but one is much faster when you have a lot of hungry people.

Here's a side-by-side comparison:

| Feature | Development | Production |
|---------|-------------|------------|
| Speed | Slow (processes every request) | Fast (serves pre-built files) |
| File watching | Yes (rebuilds on save) | No (one-time build) |
| Error messages | Verbose, friendly | Minified, less helpful |
| Code optimization | None | Minified, tree-shaken |
| Source maps | Yes (for debugging) | Optional |
| Hot reload | Yes | No |

The development server also has features like Hot Module Replacement (HMR). When you change a file, the dev server injects the new code into the running page without a full reload. You see your changes instantly. This is great for development but adds overhead that isn't needed in production.

## npm run build: What Happens

Let's run the build and see what happens:

```bash
npm run build
```

You'll see output that looks something like this:

```
vite v5.4.0 building for production...
✓ 42 modules transformed.
build/_app/immutable/assets/0.a1b2c3d4.css  1.23 kB │ gzip:  0.68 kB
build/_app/immutable/assets/5.e6f7g8h9.css    0.87 kB │ gzip:  0.52 kB
build/_app/immutable/nodes/0.i1j2k3l4.js      0.21 kB │ gzip:  0.17 kB
build/_app/immutable/nodes/1.m5n6o7p8.js      0.15 kB │ gzip:  0.12 kB
build/_app/immutable/nodes/2.q9r0s1t2.js      4.56 kB │ gzip:  1.89 kB
build/_app/immutable/nodes/3.u3v4w5x6.js      2.34 kB │ gzip:  1.12 kB
build/_app/immutable/entry/start.y7z8a9b0.js  0.89 kB │ gzip:  0.47 kB
build/_app/immutable/entry/app.c1d2e3f4.js    5.67 kB │ gzip:  2.34 kB
build/index.html                               1.23 kB │ gzip:  0.67 kB
✓ built in 1.87s
```

What just happened? SvelteKit took all your `.svelte` files, your JavaScript, your CSS, and your HTML, and bundled them into a small set of optimized files. Let's break down what each line means.

The `vite v5.4.0 building for production` line tells you that Vite (the build tool) is running in production mode. Vite is the engine that powers SvelteKit's development server and build process. When you run `npm run dev`, Vite serves files on demand. When you run `npm run build`, Vite bundles everything into optimized static files.

The `✓ 42 modules transformed` line means SvelteKit processed 42 different files. That includes your components, your routes, your layout files, and all the dependencies they use. Each import, each function call, each Svelte component — SvelteKit analyzed all of them to figure out the most efficient way to bundle your code.

Each file in the output has two sizes: the original size and the gzipped size. Gzip is a compression algorithm that reduces file sizes for transfer over the network. When your browser downloads a file, it can request the gzipped version, which is much smaller. The `gzip:` numbers show how much smaller the file will be after gzip compression on the server.

For example, `5.e6f7g8h9.css` is 0.87 kB on disk, but only 0.52 kB after gzip. That's a 40% reduction. Across all your files, gzip typically saves 60-70% of bandwidth.

The `✓ built in 1.87s` line tells you the total build time. Under 2 seconds is excellent. Larger projects might take 10-30 seconds, but that's still fast compared to some other build tools.

> **Try It Yourself**
>
> Run the build and pay attention to the output:
> ```bash
> npm run build
> ```
> Write down the total number of modules transformed and the build time. Now make a small change to a component — add a word to some text — and rebuild. Notice how the hashes in the filenames change but the build is faster the second time (Vite caches).

## The Build Output: index.html, _app/immutable/

After the build completes, you'll have a `build/` directory. Let's look inside:

```bash
ls -la build/
```

You'll see something like:

```
build/
├── _app/
│   └── immutable/
│       ├── assets/
│       │   ├── 0.a1b2c3d4.css
│       │   └── 5.e6f7g8h9.css
│       ├── entry/
│       │   ├── app.c1d2e3f4.js
│       │   └── start.y7z8a9b0.js
│       └── nodes/
│           ├── 0.i1j2k3l4.js
│           ├── 1.m5n6o7p8.js
│           ├── 2.q9r0s1t2.js
│           └── 3.u3v4w5x6.js
└── index.html
```

There are two important things here: `index.html` and the `_app/immutable/` directory.

**index.html** is the entry point. When someone visits your website, this is the first file their browser downloads. It's a small HTML file that loads all the JavaScript and CSS your app needs. Let's look at its contents:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="modulepreload" href="/_app/immutable/entry/start.y7z8a9b0.js">
    <link rel="modulepreload" href="/_app/immutable/entry/app.c1d2e3f4.js">
    <link rel="stylesheet" href="/_app/immutable/assets/0.a1b2c3d4.css">
</head>
<body data-sveltekit-preload-data="hover">
    <div style="display: contents">%sveltekit.body%</div>
    <script>
        {
            const p = (t, h) => (t.__sveltekit_h = h, t);
            const h = location.pathname === '/' ? '' : location.pathname;
            var t = document.createElement('script');
            t.type = 'module';
            t.src = '/_app/immutable/entry/start.y7z8a9b0.js';
        }
    </script>
</body>
</html>
```

Notice the `%sveltekit.body%` placeholder — SvelteKit replaces this with the actual page content during rendering. The `<link rel="modulepreload">` tags tell the browser to start downloading the JavaScript files early, before the HTML parser even encounters them.

**_app/immutable/** contains all your bundled assets. The "immutable" part is crucial — it means these files will never change. Each filename includes a hash (that `a1b2c3d4` part), which is based on the file's contents. If you change the code and rebuild, the hash changes, and the filename changes. This is how SvelteKit handles cache busting — browsers know they can cache these files forever because a new URL means new content.

Let's look at the file sizes more carefully:

```bash
find build/ -type f -exec ls -la {} \; | awk '{print $5, $NF}' | sort -n
```

```
215  build/_app/immutable/nodes/1.m5n6o7p8.js
220  build/_app/immutable/nodes/0.i1j2k3l4.js
890  build/_app/immutable/entry/start.y7z8a9b0.js
891  build/_app/immutable/assets/5.e6f7g8h9.css
1230 build/_app/immutable/assets/0.a1b2c3d4.css
1230 build/index.html
2340 build/_app/immutable/nodes/3.u3v4w5x6.js
4560 build/_app/immutable/nodes/2.q9r0s1t2.js
5670 build/_app/immutable/entry/app.c1d2e3f4.js
```

The total comes to about 16 KB. That's the entire JavaScript and CSS for our application — under 16 kilobytes. After gzip compression, it'll be even smaller.

## adapter-static: Generating Static Files

Remember when we configured `adapter-static` in our `svelte.config.js`? This is where it shines. Let's look at that config again:

```javascript
import adapter from '@sveltejs/adapter-static';

export default {
    kit: {
        adapter: adapter({
            fallback: 'index.html',
            pages: 'build',
            assets: 'build',
            precompress: false
        })
    }
};
```

`adapter-static` tells SvelteKit to generate a static site — just HTML, CSS, and JavaScript files that can be served by any web server. There's no Node.js runtime needed on the server. No server-side rendering. Just files.

The `pages: 'build'` and `assets: 'build'` options tell SvelteKit where to put the output. Both go into the `build/` directory. You could put them in separate directories, but combining them simplifies deployment.

The `fallback: 'index.html'` option is crucial. It tells SvelteKit to generate a fallback page for routes that don't have a pre-rendered HTML file. This is how single-page applications (SPAs) work — the browser downloads `index.html` first, then JavaScript takes over and renders the correct page based on the URL.

The `precompress: false` option tells SvelteKit not to generate pre-compressed `.gz` files. We'll let nginx handle gzip compression at runtime instead, which is more flexible and allows on-the-fly compression for any file type.

There are other adapters available too:

| Adapter | Use Case |
|---------|----------|
| `adapter-static` | Static hosting (GitHub Pages, Netlify, Cloudflare Pages) |
| `adapter-auto` | Auto-detects the deployment platform |
| `adapter-node` | Node.js servers (VPS, your own server) |
| `adapter-cloudflare` | Cloudflare Workers |
| `adapter-vercel` | Vercel |
| `adapter-netlify` | Netlify |

We chose `adapter-static` because it gives us the most control. We can serve the files with any web server, on any platform, without needing Node.js running on the server.

## The Fallback: index.html for All Routes

Let's talk more about that `fallback` option, because it's one of the most important parts of your deployment.

When you have `fallback: 'index.html'`, SvelteKit generates a single `index.html` file that serves as the entry point for *every* route. If someone visits `https://your-site.com/fiction/123`, the server doesn't need a pre-rendered file at `/fiction/123/index.html`. Instead, it serves `index.html` for every request, and JavaScript handles the routing.

This is why we use `adapter-static` for our app. FicHub is a single-page application — once the initial HTML loads, all navigation happens through JavaScript without full page reloads.

```
Browser requests: /fiction/123
     ↓
Server serves:    /index.html (the fallback)
     ↓
JavaScript loads: Reacts to URL, renders the fiction page
     ↓
User sees:       The fiction page for ID 123
```

Without the fallback, visiting `/fiction/123` directly would return a 404 error because there's no file at that path. The fallback ensures every route works.

In nginx, the `try_files` directive makes this work:

```nginx
location / {
    root /var/www/fichub;
    try_files $uri $uri/ /index.html;
}
```

This tells nginx: "First, try to find the exact file the browser requested. If it doesn't exist, try finding a directory with an `index.html` inside. If that doesn't exist either, serve `/index.html` and let JavaScript handle the routing."

The flow looks like this:

```
1. Browser requests: /fiction/123
2. nginx looks for: /var/www/fichub/fiction/123 → not found
3. nginx looks for: /var/www/fichub/fiction/123/ → not found
4. nginx serves:    /var/www/fichub/index.html → found!
5. Browser loads JavaScript, SvelteKit renders the fiction page
```

This is the standard approach for SPAs. It's simple, efficient, and works with any framework.

> **Watch Out!**
>
> If you forget the `try_files` directive in nginx, direct URLs like `/fiction/123` will return 404 errors. The homepage (`/`) will work because there's an `index.html` at the root, but any deeper route will fail. Always include `try_files $uri $uri/ /index.html;` when serving an SPA.

## How SvelteKit Bundles: Entry Point, Chunks, Nodes

Let's dig into how SvelteKit organizes the build output. This might seem like a lot of detail, but understanding it helps you debug issues and optimize your app.

**Entry point** (`entry/start.js` and `entry/app.js`): These are the first JavaScript files that load. `start.js` initializes the SvelteKit router — it figures out what URL the browser is on, matches it to a route, and loads the right components. `app.js` sets up the application context — the shared state and configuration that all pages use. Think of them as the startup sequence for your app.

When the browser loads `start.js`, it:
1. Reads the current URL from `window.location`
2. Matches the URL to a route definition
3. Determines which "nodes" (components) need to load
4. Downloads those nodes in parallel
5. Renders the page
6. Attaches event listeners for navigation

**Nodes** (`nodes/0.js`, `nodes/1.js`, etc.): Each node corresponds to a route or a layout in your app. When you visit a page, SvelteKit loads only the nodes needed for that page. This is called "code splitting" — instead of downloading all your JavaScript at once, you download only what you need for the current page.

For example, if your app has:
- A layout (`+layout.svelte`) → Node 0
- A home page (`+page.svelte`) → Node 1
- A fiction page (`+page.svelte`) → Node 2
- A settings page (`+page.svelte`) → Node 3

When someone visits the home page, they download Node 0 (the layout) and Node 1 (the home page). If they navigate to the fiction page, they only need to download Node 2 — the layout is already loaded. This means navigating between pages is nearly instant.

Let's see what a node file looks like (simplified):

```javascript
// nodes/2.q9r0s1t2.js — the fiction page
import { SvelteComponent } from 'svelte';

// Component definition
class FictionPage extends SvelteComponent {
    constructor(options) {
        super();
        // ... component setup
    }
}

// Data loader
export async function load({ fetch, params }) {
    const response = await fetch(`/api/v1/fics/${params.id}`);
    const fic = await response.json();
    return { fic };
}

export { FictionPage as component };
```

Each node exports two things: a `load` function (which fetches data) and a `component` (which renders the UI). SvelteKit calls the `load` function first, then renders the component with the data.

**Chunks** (`chunks/`): Sometimes SvelteKit splits code further into "chunks" — shared pieces of code used by multiple pages. This prevents duplication. If two pages both import a helper function, that function goes into a shared chunk instead of being duplicated in both pages.

**CSS** (`assets/*.css`): Your styles are extracted into separate CSS files. Each page can have its own CSS file, and only the CSS needed for the current page is loaded. This means the home page doesn't load styles that are only used on the settings page.

This whole system is designed for one thing: making pages load as fast as possible. The browser downloads the minimum amount of code needed for the current page, and loads more as the user navigates.

## Cache Headers: Immutable Assets, No-Cache index

Now that we understand the build output, let's talk about how browsers cache these files. This is where the "immutable" naming convention becomes really clever.

**Immutable assets** (files in `_app/immutable/`): These files have a content hash in their filename. If the content changes, the filename changes. This means you can tell browsers to cache these files *forever*. The browser will never ask for the same filename again with different content — it's guaranteed to be the same.

Here's the nginx configuration that handles this:

```nginx
# Immutable assets — cache forever
location /_app/immutable/ {
    add_header Cache-Control "public, max-age=31536000, immutable";
}

# Other assets — cache for a short time
location /_app/ {
    add_header Cache-Control "public, max-age=3600";
}

# index.html — never cache
location / {
    add_header Cache-Control "no-cache";
}
```

The `Cache-Control: public, max-age=31536000, immutable` header tells the browser: "This file is good for one year (31,536,000 seconds) and will never change. Don't even ask me about it."

The `Cache-Control: no-cache` header for `index.html` tells the browser: "Always check with me before using a cached version." This ensures users always get the latest version of your app's entry point.

Why is this important? Because when you deploy a new version of FicHub, the immutable assets get new hashes and new filenames. Users' browsers request the new filenames and get the new code. Meanwhile, `index.html` (which references the new filenames) is always fetched fresh, so it always points to the latest version.

This strategy is called "cache busting" — you make each build's assets unique through their filenames, so you can cache them aggressively without worrying about users seeing old versions.

Let's see the full caching strategy:

```
File Type          Cache Duration    Header
─────────────────────────────────────────────
index.html         Always check     no-cache
_app/immutable/*   1 year           max-age=31536000, immutable
_other assets      1 hour           max-age=3600
API responses      5 minutes        Cache-Control via nginx proxy
```

The result: users load your app quickly (cached assets), always get the latest version (fresh index.html), and API data is fresh enough (5-minute cache).

## The .gitignore: Excluding Build Artifacts

After running `npm run build`, you'll have a `build/` directory full of generated files. These files should *not* be committed to your Git repository. They're generated from your source code — committing them would be like compiling code and committing the compiled output alongside the source.

Your `.gitignore` should include:

```gitignore
# Build output
build/

# Dependencies
node_modules/

# Environment variables
.env
.env.local
.env.*.local

# IDE files
.vscode/
.idea/
*.swp
*.swo

# OS files
.DS_Store
Thumbs.db

# SvelteKit
.svelte-kit/
```

The `build/` line is the important one. It tells Git to ignore the entire build directory. This keeps your repository clean and small.

The `.svelte-kit/` directory is also important. SvelteKit creates this directory during development to store generated type definitions and other temporary files. It's not needed for deployment, and it shouldn't be committed.

Let's check that our `.gitignore` is set up correctly:

```bash
git status
```

If `build/` appears in the output, it's being tracked. Remove it:

```bash
# Remove build artifacts from Git tracking (keeps the files)
git rm -r --cached build/
git commit -m "Remove build artifacts from tracking"
```

> **Watch Out!**
>
> If you accidentally commit the `build/` directory, don't panic. You can remove it from Git tracking without deleting the files using the command above. The `--cached` flag is the key — it removes files from Git's index (tracking) without deleting them from your computer.
>
> But also: add `build/` to `.gitignore` first, so this doesn't happen again.

## Checking the Build: npm run preview

After building, you should always check that everything works. SvelteKit provides a preview command for this:

```bash
npm run preview
```

This starts a local server that serves your built files. It's not the development server — it's serving the same files that would be deployed to production.

```
  ➜  Local:   http://localhost:4173/
  ➜  Network: use --host to expose
```

Visit `http://localhost:4173` and test your app:
- Does the home page load?
- Do all the links work?
- Does navigation between pages work without full reloads?
- Do the API calls work?
- Does the search function work?
- Do the filters work?
- Do recommendations appear?

If something is broken in preview but works in dev, it's usually a build configuration issue. Common causes include:
- Missing environment variables (the build process might not have the same env vars as dev)
- Incorrect adapter configuration
- Import paths that work in dev but not in production
- Code that uses browser APIs outside of `onMount`

The preview server runs on port 4173 by default (different from the dev server's 5173). If you need to test with your backend API, make sure the API is running on the correct port.

> **Try It Yourself**
>
> Run the build and preview commands, then do a thorough test:
> ```bash
> npm run build
> npm run preview
> ```
> Open `http://localhost:4173` and test every page. Open the browser's Network tab (F12 → Network) and watch the requests. Notice:
> - `index.html` is loaded first (check its Cache-Control header)
> - JavaScript and CSS files are loaded next (check their Cache-Control headers)
> - API requests go to your backend
> - After the first load, navigating between pages loads almost nothing (everything is cached!)

## Environment Variables in Production

One thing that often trips people up when moving from development to production is environment variables. In development, you might have a `.env` file with your database URL and API keys. In production, these need to be set differently.

When you run `npm run build`, Vite embeds environment variables into the built JavaScript. This means the values at build time are "baked in" to your production files. If you forget to set the right environment variables before building, your production app will have the wrong values.

```bash
# In development
DATABASE_URL=postgres://localhost/fichub
API_URL=http://localhost:3000/api

# In production (set these before building)
DATABASE_URL=postgres://fichub:password@localhost/fichub
API_URL=https://fichub.yourdomain.com/api
```

For SvelteKit, client-side environment variables must be prefixed with `PUBLIC_`:

```bash
# This is available in both client and server code
PUBLIC_API_URL=https://fichub.yourdomain.com/api

# This is only available in server-side code (load functions, actions, etc.)
DATABASE_URL=postgres://localhost/fichub
```

> **Watch Out!**
>
> Never put secret keys (like database passwords or API tokens) in `PUBLIC_` variables. These are embedded in the JavaScript that gets sent to users' browsers — anyone can see them. Only use `PUBLIC_` for values that are safe to expose, like your site URL or the API endpoint.

## Common Build Errors and Fixes

Build errors can be frustrating, but they almost always have clear explanations. Let's look at the most common ones.

**Error: "Could not resolve..."**
This means SvelteKit can't find a file you're importing. Check that the file exists and the import path is correct. Remember that SvelteKit uses aliases:

```svelte
<!-- Wrong — might not resolve in production -->
<script>
  import { fetchData } from '@/lib/api.js';
</script>

<!-- Right — use the SvelteKit alias -->
<script>
  import { fetchData } from '$lib/api.js';
</script>
```

The `$lib` alias points to your `src/lib` directory. It works in both development and production.

**Error: "Unexpected token"**
This usually means you have a syntax error in your code. The error message will tell you which file and which line. Common causes:
- Missing closing tags in HTML
- Missing semicolons in JavaScript
- Incorrect Svelte syntax
- Using modern JavaScript features that Vite doesn't support

**Error: "window is not defined"**
This happens when you use browser-only APIs (like `window`, `document`, or `localStorage`) in code that runs during server-side rendering or static generation. During the build process, SvelteKit tries to render your pages on the server, and browser APIs aren't available there.

Fix it by wrapping browser-only code:

```svelte
<script>
  import { onMount } from 'svelte';

  let data;

  onMount(() => {
    // This only runs in the browser, after the page is rendered
    data = localStorage.getItem('myData');
  });
</script>
```

Or use the `$effect` rune (in Svelte 5):

```svelte
<script>
  let data = $state(null);

  $effect(() => {
    // This only runs in the browser
    data = localStorage.getItem('myData');
  });
</script>
```

**Error: "hydrating" or "ssr" warnings**
These are usually not errors but warnings about code that runs differently on the server versus the client. If you're using `adapter-static`, these warnings are usually safe to ignore. The server renders the initial HTML, and the client "hydrates" it (attaches event listeners and makes it interactive).

**Error: "Cannot use import statement outside a module"**
This usually means a dependency is trying to use CommonJS (`require()`) instead of ES modules (`import`). Check which package is causing the issue and look for an alternative or a configuration option.

> **Watch Out!**
>
> If your build succeeds but the preview shows errors, check your browser's developer console (F12). Build-time errors appear in the terminal; runtime errors appear in the browser console. The most common runtime error after building is "Cannot read property of undefined" — usually caused by data that exists in development but is missing in production.

## The Build Size: How Big Is Our App?

Let's check how big our FicHub build is:

```bash
du -sh build/
```

```
156K    build/
```

156 kilobytes! That's tiny. Let's break it down:

```bash
du -sh build/_app/immutable/*
```

```
24K     build/_app/immutable/assets
16K     build/_app/immutable/entry
48K     build/_app/immutable/nodes
```

For comparison:
- A typical React app: 1-5 MB
- A typical Vue app: 500KB - 2MB
- A typical Angular app: 2-8 MB
- FicHub (SvelteKit + adapter-static): ~156KB

Svelte compiles your components into efficient vanilla JavaScript. There's no virtual DOM, no runtime framework to ship. The compiler does the heavy lifting at build time, so your users get smaller, faster code.

Let's dig deeper into what's in the build:

```bash
# Count files by type
find build/ -type f | sed 's/.*\.//' | sort | uniq -c | sort -rn
```

```
      6 js
      2 css
      1 html
```

Only 9 files total! And the JavaScript is tiny — each file is a few kilobytes at most. The total JavaScript is about 14 KB, and the total CSS is about 2 KB.

The gzipped sizes are even smaller. When gzip compression is enabled on your server:

```bash
# Total uncompressed size
find build/ -type f -exec cat {} + | wc -c
# ~16,000 bytes = 16 KB

# With gzip, expect about 60-70% reduction
# ~5-7 KB transferred over the network
```

This is one of the great things about Svelte — it doesn't ship a framework to your users. It ships the *output* of your framework, which is just JavaScript that does exactly what your app needs and nothing more. There's no React runtime, no Vue runtime, no Angular runtime. Just your code, compiled and optimized.

> **Try It Yourself**
>
> Compare your build size to other frameworks:
> ```bash
> # Your SvelteKit build
> du -sh build/
>
> # Try creating a React app and building it
> npx create-react-app my-react-app
> cd my-react-app && npm run build
> du -sh build/
>
> # Try creating a Vue app and building it
> npm create vite@latest my-vue-app -- --template vue
> cd my-vue-app && npm run build
> du -sh build/
> ```
>
> The size difference is dramatic. Svelte consistently produces the smallest builds.

---

# Chapter 34: Deploying to a Server

## What Is a Server? (A Computer That Never Sleeps)

You've been running FicHub on your own computer. But what happens when you close your laptop? The app stops. Nobody can access it.

A server is a computer that stays on all the time. It's connected to the internet, and it runs your app 24/7. When someone visits your website, they're connecting to the server, not your laptop.

Think of it this way: your laptop is like a food truck. It serves food, but only when you're parked and the engine is running. A server is like a restaurant — it has a fixed address, it's always open, and anyone can walk in anytime.

Servers come in many forms:

- **Physical servers**: Actual computers in a data center. Big companies like Google and Facebook run thousands of these. They're powerful but expensive (thousands of dollars).
- **Virtual Private Servers (VPS)**: Virtual computers rented from companies like DigitalOcean, Linode, or Hetzner. You get a slice of a physical server. Prices range from $5-50/month.
- **Cloud instances**: Scalable computing power from AWS, Google Cloud, or Azure. You pay for what you use, and you can scale up or down as needed.
- **Single-board computers**: Small, affordable computers like the Raspberry Pi or Orange Pi. Perfect for personal projects. Cost: $20-60.

For FicHub, we're going to use an Orange Pi — a small, affordable single-board computer that's perfect for personal projects. It's like a Raspberry Pi but cheaper and often more powerful.

Why not use a VPS or cloud instance? Because owning your own hardware is empowering. There's no monthly bill. No vendor lock-in. No worrying about pricing changes. And the hardware costs less than 6 months of a basic VPS.

## The Orange Pi: A Small, Affordable Server

The Orange Pi is a credit-card-sized computer that costs around $20-40. It has:
- A quad-core ARM processor (usually Allwinner H618 or similar)
- 1-4 GB of RAM
- Ethernet port for internet connectivity
- microSD card slot for storage
- GPIO pins for hardware projects
- USB ports for peripherals
- HDMI output (optional, for connecting a monitor)
- Runs Linux (usually Debian or Ubuntu)

It's like a Raspberry Pi but cheaper. For running FicHub, which serves a small community of fanfiction readers, it's more than powerful enough.

You'll need:
- An Orange Pi (any recent model works — Orange Pi 5, 5B, or 3B are great choices)
- A microSD card (at least 16GB, Class 10 or faster)
- An Ethernet cable (for reliable internet connection)
- A USB-C power supply (5V/3A is usually sufficient)
- A case (optional but recommended — keeps dust out and looks nice on a shelf)

**Setting up the hardware:**

1. Download the Orange Pi image from [orangepi.org](http://www.orangepi.org) — choose the Debian or Ubuntu image
2. Flash it to the microSD card using Balena Etcher (graphical) or `dd` (command line):
   ```bash
   # Find your microSD card (be careful! check with lsblk first)
   lsblk

   # Flash the image (replace /dev/sdX with your card)
   sudo dd if=Orangepi_image.img of=/dev/sdX bs=1M status=progress

   # Sync to ensure all data is written
   sync
   ```
3. Insert the microSD card into the Orange Pi
4. Connect the Ethernet cable to your router
5. Connect the power supply — the Orange Pi boots automatically

Once it's booted, find its IP address on your network:

```bash
# From another computer on the same network
nmap -sn 192.168.1.0/24

# Or check your router's admin page for connected devices
```

You should see the Orange Pi in the list. SSH into it:

```bash
ssh root@192.168.1.XXX
```

The default password is usually `orangepi` or `1234` (check the Orange Pi documentation for your specific model).

> **Try It Yourself**
>
> If you don't have an Orange Pi, you can follow along with a virtual machine. Install VirtualBox or use a cloud VPS (DigitalOcean has $4/month droplets). The deployment steps are the same — you just need a Linux machine that's always on.

## Setting Up the Server: Installing Node.js, PostgreSQL, Redis

Now that we have access to the server, let's install everything FicHub needs. Think of this as setting up the kitchen before you start cooking.

**Update the system:**

Always start by updating the system to get the latest security patches:

```bash
apt update && apt upgrade -y
```

**Install Node.js:**

We need Node.js for the frontend build process (even though the production site is static). We'll also use it for running scripts and tools:

```bash
# Install Node.js 20 LTS (Long Term Support)
curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
apt install -y nodejs

# Verify installation
node --version  # Should show v20.x.x
npm --version   # Should show 10.x.x
```

Node.js 20 LTS is the best choice for servers. It's stable, well-tested, and will receive security updates until April 2026.

**Install PostgreSQL:**

PostgreSQL is our database. It stores all the fanfiction data, user preferences, and community contributions:

```bash
# Install PostgreSQL
apt install -y postgresql postgresql-contrib

# Start and enable PostgreSQL (starts on boot)
systemctl start postgresql
systemctl enable postgresql

# Create a database user and database
sudo -u postgres psql
```

Inside the PostgreSQL prompt:

```sql
-- Create a user for FicHub
CREATE USER fichub WITH PASSWORD 'your_secure_password_here';

-- Create the database
CREATE DATABASE fichub OWNER fichub;

-- Grant all permissions
GRANT ALL PRIVILEGES ON DATABASE fichub TO fichub;

-- Exit the PostgreSQL prompt
\q
```

**Install Redis:**

Redis is our cache. It stores frequently accessed data in memory for lightning-fast responses:

```bash
# Install Redis
apt install -y redis-server

# Start and enable Redis
systemctl start redis-server
systemctl enable redis-server

# Test Redis
redis-cli ping
# Should respond: PONG
```

**Create the fichub user:**

For security, we don't want to run our app as root. Create a dedicated user:

```bash
# Create a system user (no login shell)
useradd -r -s /bin/false fichub

# Create directories for the app
mkdir -p /opt/fichub
mkdir -p /var/www/fichub
chown fichub:fichub /opt/fichub
chown fichub:fichub /var/www/fichub
```

> **Try It Yourself**
>
> SSH into your server and install all three services. Verify they're running:
> ```bash
> systemctl status postgresql
> systemctl status redis-server
> node --version
> ```
> All three should show "active (running)" or display their version number. If any service failed to start, check the logs with `journalctl -u <service-name> -n 50`.

## The Rust Backend: Cross-Compiling for ARM

Here's where things get interesting. Our server runs on ARM architecture (the Orange Pi's processor), but you probably developed on an x86 computer (Intel or AMD). We need to cross-compile our Rust backend for ARM.

Cross-compilation means building code on one architecture that runs on another. It's like writing a letter in English and having it translated to French before sending it.

First, install the ARM target on your development machine:

```bash
# Install the ARM64 target for Rust
rustup target add aarch64-unknown-linux-gnu

# Install cross-compilation tools
# On Ubuntu/Debian:
sudo apt install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu

# On Arch Linux:
sudo pacman -S aarch64-linux-gnu-gcc

# On macOS:
brew install aarch64-elf-gcc
```

Configure Cargo for cross-compilation by creating `.cargo/config.toml` in your backend directory:

```bash
mkdir -p .cargo
cat > .cargo/config.toml << 'EOF'
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
EOF
```

Now build the release binary:

```bash
cd fichub-backend

# Build for ARM64 with optimizations
cargo build --release --target aarch64-unknown-linux-gnu
```

This compiles your Rust code into a binary that runs on ARM processors. The `--release` flag enables optimizations — without it, the binary would be debug mode and much slower.

The resulting binary is self-contained. It doesn't need Rust installed on the server. It doesn't need any libraries. Just copy it over and run it. That's one of Rust's superpowers.

```bash
# Check the binary size
ls -lh target/aarch64-unknown-linux-gnu/release/fichub-backend
# About 3-5 MB, depending on dependencies

# Check what it links to
file target/aarch64-unknown-linux-gnu/release/fichub-backend
# Should show: ELF 64-bit LSB executable, ARM aarch64
```

Let's check the size of what we're shipping:

```bash
# Binary size
ls -lh target/aarch64-unknown-linux-gnu/release/fichub-backend

# With strip (remove debug symbols)
strip target/aarch64-unknown-linux-gnu/release/fichub-backend
ls -lh target/aarch64-unknown-linux-gnu/release/fichub-backend
# Usually under 3 MB
```

> **Watch Out!**
>
> Cross-compilation can take a while (10-30 minutes depending on your computer). Rust compiles slowly but produces highly optimized binaries. The resulting binary is small and fast.
>
> If cross-compilation fails, make sure you have the correct target installed (`rustup target add aarch64-unknown-linux-gnu`) and the cross-compilation linker is configured in `.cargo/config.toml`. Common errors:
> - "linker `aarch64-linux-gnu-gcc` not found" → install the cross-compilation tools
> - "can't find crate for `std`" → run `rustup target add aarch64-unknown-linux-gnu`

## Building the Frontend: npm run build

We covered the build process in Chapter 33, but let's do it one more time with deployment in mind:

```bash
cd fichub-frontend

# Make sure dependencies are installed
npm install

# Build for production
npm run build
```

The `build/` directory now contains everything we need for the frontend. It's ready to be served by any web server.

Let's verify the build is complete:

```bash
ls -la build/
# Should show: _app/  index.html

# Check total size
du -sh build/
# Should be under 200 KB
```

## Syncing to the Server: rsync

Now we need to get our built files from your computer to the server. The best tool for this is `rsync` — it's like `cp` (copy) but smarter. It only transfers the files that have changed, it preserves permissions, and it can compress data during transfer.

**Sync the frontend:**

```bash
rsync -avz --delete build/ root@192.168.1.XXX:/var/www/fichub/
```

Let's break down those flags:
- `-a`: Archive mode — preserves permissions, timestamps, and symlinks
- `-v`: Verbose — shows you what's being transferred
- `-z`: Compress — reduces the amount of data sent over the network
- `--delete`: Removes files on the server that aren't in the source (keeps things clean)

**Sync the backend binary:**

```bash
scp target/aarch64-unknown-linux-gnu/release/fichub-backend root@192.168.1.XXX:/usr/local/bin/
```

**Sync configuration files:**

```bash
# Create the config directory on the server
ssh root@192.168.1.XXX "mkdir -p /etc/fichub"

# Copy your .env file (but never commit it to Git!)
scp .env root@192.168.1.XXX:/etc/fichub/.env
```

**Run database migrations:**

```bash
# On the server, run the migrations
ssh root@192.168.1.XXX
cd /opt/fichub
DATABASE_URL="postgres://fichub:password@localhost/fichub" sqlx migrate run
```

> **Try It Yourself**
>
> Build your frontend and backend, then sync them to your server:
> ```bash
> # Build frontend
> cd ../fichub-frontend
> npm run build
> rsync -avz --delete build/ root@192.168.1.XXX:/var/www/fichub/
>
> # Build backend
> cd ../fichub-backend
> cargo build --release --target aarch64-unknown-linux-gnu
> scp target/aarch64-unknown-linux-gnu/release/fichub-backend root@192.168.1.XXX:/usr/local/bin/
>
> # Verify on the server
> ssh root@192.168.1.XXX "ls -la /var/www/fichub/ && ls -la /usr/local/bin/fichub-backend"
> ```

## The systemd Service: Starting on Boot

We need our Rust backend to start automatically when the server boots, and restart if it crashes. That's what systemd is for. systemd is the init system for most Linux distributions — it manages services, handles dependencies, and keeps things running.

Create a service file on the server:

```bash
ssh root@192.168.1.XXX

# Create the service file
cat > /etc/systemd/system/fichub-backend.service << 'EOF'
[Unit]
Description=FicHub Backend API
After=network.target postgresql.service redis-server.service
Wants=postgresql.service redis-server.service

[Service]
Type=simple
User=fichub
Group=fichub
WorkingDirectory=/opt/fichub
EnvironmentFile=/etc/fichub/.env
ExecStart=/usr/local/bin/fichub-backend
Restart=always
RestartSec=5

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/fichub

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=fichub-backend

[Install]
WantedBy=multi-user.target
EOF
```

Let's break down this service file:

- **After=network.target**: Start after the network is ready
- **After=postgresql.service redis-server.service**: Start after our database and cache are ready
- **Wants=postgresql.service redis-server.service**: Try to start these services if they're not running
- **User=fichub**: Run as the fichub user (not root, for security)
- **WorkingDirectory**: Where the app runs from
- **EnvironmentFile**: Load environment variables from this file
- **ExecStart**: The command to run
- **Restart=always**: If it crashes, restart it automatically
- **RestartSec=5**: Wait 5 seconds before restarting (prevents rapid restart loops)
- **NoNewPrivileges=true**: Security — the process can't gain new privileges
- **ProtectSystem=strict**: Security — the process can only write to approved paths
- **ProtectHome=true**: Security — the process can't access /home
- **StandardOutput=journal**: Send logs to the system journal

Now enable and start the service:

```bash
# Reload systemd to pick up the new service file
systemctl daemon-reload

# Enable the service (starts on boot)
systemctl enable fichub-backend

# Start the service now
systemctl start fichub-backend

# Check the status
systemctl status fichub-backend
```

You should see `active (running)` in the output. If it shows `failed`, check the logs:

```bash
journalctl -u fichub-backend -n 50
```

This shows the last 50 lines of log output. The error message will usually tell you exactly what's wrong. Common issues:
- "Permission denied" → The fichub user can't access the binary or config file
- "Connection refused" → PostgreSQL or Redis isn't running
- "Address already in use" → Another process is using port 3000

> **Watch Out!**
>
> Make sure your `.env` file on the server has the correct database connection string. A common mistake is copying the `.env` file with `localhost` as the database host — on the server, you might need to use `127.0.0.1` instead. Also, make sure the password matches what you set in PostgreSQL.

## nginx: Reverse Proxy and Static File Serving

Our app has two parts: static files (HTML, CSS, JavaScript) and an API (the Rust backend). nginx can serve both efficiently. nginx is one of the most popular web servers in the world — it's fast, reliable, and handles high traffic well.

First, install nginx:

```bash
apt install -y nginx

# Start and enable nginx
systemctl start nginx
systemctl enable nginx
```

Create the configuration file:

```bash
cat > /etc/nginx/sites-available/fichub << 'EOF'
server {
    listen 80;
    server_name fichub.yourdomain.com;

    # Gzip compression — reduces file sizes for transfer
    gzip on;
    gzip_vary on;
    gzip_min_length 256;
    gzip_types text/plain text/css application/json application/javascript
               text/xml application/xml text/javascript image/svg+xml;

    # Static files — the frontend
    location / {
        root /var/www/fichub;
        try_files $uri $uri/ /index.html;

        # Immutable assets — cache forever (they have content hashes in filenames)
        location /_app/immutable/ {
            add_header Cache-Control "public, max-age=31536000, immutable";
        }

        # Other assets — cache for 1 hour
        location /_app/ {
            add_header Cache-Control "public, max-age=3600";
        }

        # index.html — never cache (always check for updates)
        location = / {
            add_header Cache-Control "no-cache";
        }
    }

    # API — proxy to Rust backend
    location /api/ {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Disable server version disclosure
    server_tokens off;
}
EOF
```

Enable the site and restart nginx:

```bash
# Enable the site
ln -s /etc/nginx/sites-available/fichub /etc/nginx/sites-enabled/

# Remove the default site
rm /etc/nginx/sites-enabled/default

# Test the configuration
nginx -t

# Restart nginx
systemctl restart nginx
```

The nginx configuration does several important things:

1. **Serves static files** from `/var/www/fichub/` — HTML, CSS, JavaScript
2. **Handles SPA routing** with `try_files $uri $uri/ /index.html` — so all routes work
3. **Sets cache headers** for immutable assets (cache forever) and index.html (always fresh)
4. **Proxies API requests** to the Rust backend on port 3000
5. **Enables gzip compression** to reduce file sizes by 60-70%
6. **Adds security headers** to protect against common attacks
7. **Hides server information** so attackers can't identify what software you're running

## The Cloudflare Tunnel: Making It Accessible from Anywhere

Right now, your FicHub app is accessible from your local network (`http://192.168.1.XXX`). But what if you want to access it from outside your home? Or share it with friends?

You could set up port forwarding on your router and get a domain name, but there's a simpler way: Cloudflare Tunnels.

Cloudflare Tunnels create a secure connection between your server and Cloudflare's network. Anyone can access your app through a Cloudflare URL without exposing your server directly to the internet. This is actually more secure than traditional port forwarding because your server's IP address is never exposed.

Install `cloudflared` on your server:

```bash
# Download cloudflared for ARM64
wget https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-arm64
chmod +x cloudflared-linux-arm64
mv cloudflared-linux-arm64 /usr/local/bin/cloudflared

# Login to Cloudflare (opens a browser window)
cloudflared tunnel login

# Create a tunnel
cloudflared tunnel create fichub

# Note the tunnel ID from the output
```

Configure the tunnel:

```bash
cat > ~/.cloudflared/config.yml << EOF
tunnel: <your-tunnel-id>
credentials-file: /root/.cloudflared/<your-tunnel-id>.json

ingress:
  - hostname: fichub.yourdomain.com
    service: http://localhost:80
  - service: http_status:404
EOF
```

Add the DNS record:

```bash
cloudflared tunnel route dns fichub fichub.yourdomain.com
```

Run the tunnel:

```bash
cloudflared tunnel run fichub
```

For production, create a systemd service for cloudflared:

```bash
cat > /etc/systemd/system/cloudflared.service << 'EOF'
[Unit]
Description=Cloudflare Tunnel
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/cloudflared tunnel run fichub
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable cloudflared
systemctl start cloudflared
```

Now anyone can access your FicHub at `https://fichub.yourdomain.com` — secure, fast, and without exposing your server's IP address.

**Benefits of Cloudflare Tunnels:**
- Free (unlimited tunnels on the free plan)
- No port forwarding needed
- Automatic HTTPS (SSL certificates handled by Cloudflare)
- DDoS protection
- CDN caching (Cloudflare's global network)
- Your server's IP is never exposed to the public

## Verifying the Deployment: curl Tests

Before celebrating, let's verify everything is working. curl is a command-line tool for making HTTP requests. We'll use it to test each component.

**Test the frontend:**

```bash
curl -I http://localhost/
```

You should see:
```
HTTP/1.1 200 OK
Server: nginx/1.24.0
Content-Type: text/html
Cache-Control: no-cache
```

The `200 OK` means the page loaded successfully. The `Cache-Control: no-cache` header confirms that `index.html` is not being cached.

**Test the API:**

```bash
curl http://localhost/api/v1/health
```

You should see a JSON response:
```json
{
    "status": "healthy",
    "database": "connected",
    "cache": "connected",
    "uptime": 3600
}
```

**Test from outside (via Cloudflare):**

```bash
curl -I https://fichub.yourdomain.com/
```

Again, `200 OK` with the HTML content. Check that HTTPS is working.

**Test static assets:**

```bash
curl -I http://localhost/_app/immutable/entry/app.c1d2e3f4.js
```

Check the headers:
```
Cache-Control: public, max-age=31536000, immutable
```

The immutable caching is working correctly.

**Test the database connection:**

```bash
# SSH into the server
ssh root@192.168.1.XXX

# Test PostgreSQL
sudo -u postgres psql -c "SELECT 1;" -d fichub

# Test Redis
redis-cli ping
# PONG
```

**Check the logs:**

```bash
# Backend logs
journalctl -u fichub-backend -f

# nginx logs
tail -f /var/log/nginx/access.log
tail -f /var/log/nginx/error.log

# Cloudflare tunnel logs
journalctl -u cloudflared -f
```

The `-f` flag follows the log in real-time. Open your browser and visit your site — you should see the requests appearing in the access log.

## The Deployment Checklist

Before you go live, walk through this checklist. Print it out and check each item:

```bash
# Create a deployment script
cat > deploy.sh << 'EOF'
#!/bin/bash
set -e

echo "=== FicHub Deployment Checklist ==="

# 1. Build frontend
echo "1. Building frontend..."
cd ../fichub-frontend
npm run build

# 2. Build backend
echo "2. Building backend..."
cd ../fichub-backend
cargo build --release --target aarch64-unknown-linux-gnu

# 3. Sync frontend
echo "3. Syncing frontend..."
rsync -avz --delete build/ root@SERVER_IP:/var/www/fichub/

# 4. Sync backend
echo "4. Syncing backend..."
scp target/aarch64-unknown-linux-gnu/release/fichub-backend root@SERVER_IP:/usr/local/bin/

# 5. Sync config
echo "5. Syncing config..."
scp .env root@SERVER_IP:/etc/fichub/.env

# 6. Run migrations
echo "6. Running migrations..."
ssh root@SERVER_IP "cd /opt/fichub && DATABASE_URL=... sqlx migrate run"

# 7. Restart services
echo "7. Restarting services..."
ssh root@SERVER_IP "systemctl restart fichub-backend"

# 8. Verify
echo "8. Verifying..."
curl -f https://fichub.yourdomain.com/ > /dev/null
curl -f https://fichub.yourdomain.com/api/v1/health > /dev/null

echo "=== Deployment complete! ==="
EOF

chmod +x deploy.sh
```

Now you can deploy with a single command: `./deploy.sh`

## Backups and Monitoring

Deployment isn't a one-time thing. You need to think about what happens after deployment — specifically, backups and monitoring.

**Database backups:**

A simple cron job can back up your database daily:

```bash
# Create a backup script
cat > /opt/fichub/backup.sh << 'EOF'
#!/bin/bash
DATE=$(date +%Y-%m-%d)
BACKUP_DIR=/var/backups/fichub
mkdir -p $BACKUP_DIR

pg_dump -U fichub fichub | gzip > $BACKUP_DIR/fichub-$DATE.sql.gz

# Keep only last 30 days
find $BACKUP_DIR -name "*.sql.gz" -mtime +30 -delete
EOF

chmod +x /opt/fichub/backup.sh

# Add to crontab (runs daily at 3 AM)
crontab -e
# Add: 0 3 * * * /opt/fichub/backup.sh
```

**Health monitoring:**

You can set up a simple health check that pings your API and alerts you if it's down:

```bash
# Simple health check script
cat > /opt/fichub/health-check.sh << 'EOF'
#!/bin/bash
if ! curl -sf https://fichub.yourdomain.com/api/v1/health > /dev/null 2>&1; then
    echo "FicHub is DOWN at $(date)" | mail -s "FicHub Alert" you@email.com
    systemctl restart fichub-backend
fi
EOF

chmod +x /opt/fichub/health-check.sh

# Add to crontab (runs every 5 minutes)
# */5 * * * * /opt/fichub/health-check.sh
```

> **Try It Yourself**
>
> Set up automatic backups and health monitoring on your server. Even a simple daily backup can save you from disaster. Test the backup by restoring it to a different database and verifying the data is intact.

---

# Chapter 35: The Full FicHub Stack

## Everything We Built: Frontend + Backend + Database + Cache

Take a moment to appreciate what you've built over the course of this book. FicHub is a complete web application with:

- **Frontend**: A SvelteKit single-page application with responsive design, search, filters, and recommendations
- **Backend**: A Rust API server handling requests, authentication, and business logic
- **Database**: PostgreSQL storing fanfiction data, user preferences, and community contributions
- **Cache**: Redis storing frequently accessed data for lightning-fast responses
- **Recommendation engine**: Collaborative filtering that suggests fics based on reading patterns
- **Search system**: A syntax parser that supports complex queries with operators
- **Collection worker**: A background process that scrapes fanfiction sites and keeps the database updated

That's a lot of moving parts, but they all work together seamlessly. Let's trace a request from start to finish.

## The Request Flow: Browser → nginx → SvelteKit → API → Rust → PostgreSQL

When a user opens FicHub in their browser and searches for "Harry Potter romance", here's what happens:

```
1. Browser requests https://fichub.yourdomain.com/
   ↓
2. Cloudflare Tunnel forwards to server (encrypted)
   ↓
3. nginx receives the request
   ↓
4. nginx serves /index.html (SPA entry point)
   ↓
5. Browser loads JavaScript (entry/app.js, nodes)
   ↓
6. Svelte router initializes, loads the search page
   ↓
7. User types "Harry Potter romance" in search
   ↓
8. Svelte component calls fetch('/api/v1/search?q=Harry+Potter+romance')
   ↓
9. nginx receives /api/ request, proxies to port 3000
   ↓
10. Rust backend receives request
   ↓
11. Syntax parser parses query into tokens:
    ["Harry Potter", "romance"] (AND)
   ↓
12. Redis checks cache for this query
    Cache miss → proceed to database
   ↓
13. PostgreSQL full-text search executes:
    SELECT * FROM fictions
    WHERE to_tsvector('english', title || ' ' || summary)
    @@ to_tsquery('english', 'harry & potter & romance');
   ↓
14. Results returned to Rust backend (5-10ms)
   ↓
15. Rust caches results in Redis (TTL: 5 minutes)
   ↓
16. Rust serializes results as JSON
   ↓
17. Response flows back: Rust → nginx → Cloudflare → browser
   ↓
18. Svelte component renders the results
   ↓
19. User sees: 20 fic results in 50-100ms total
```

This entire flow takes about 50-100 milliseconds. The user sees results almost instantly. Let's break down why each layer matters.

**Cloudflare** provides the first layer of caching and security. Static assets might be served directly from Cloudflare's CDN without ever hitting your server. DDoS attacks are absorbed by Cloudflare's massive network.

**nginx** is the front door to your server. It handles SSL termination (HTTPS), gzip compression, static file serving, and request routing. It also adds security headers to protect against common attacks. When serving static files, nginx is incredibly efficient — it can handle thousands of simultaneous connections without breaking a sweat.

**SvelteKit** provides the user interface. It loads efficiently through code splitting (only downloading the JavaScript needed for the current page) and hydrates quickly because Svelte compiles to minimal JavaScript. The first page load downloads about 14 KB of JavaScript — the rest is loaded on demand.

**The Rust backend** handles the business logic. It's fast because Rust is a systems language with no garbage collector and zero-cost abstractions. It uses connection pooling for the database (reusing connections instead of creating new ones for every request) and async I/O (handling multiple requests concurrently without threads).

**Redis** caches the results of expensive queries. If someone searches for "Harry Potter romance" and another person searches for the same thing 5 minutes later, Redis serves the cached result without hitting the database. This can reduce database load by 60-80% for popular queries.

**PostgreSQL** stores and queries the data. Full-text search in PostgreSQL is incredibly fast because it uses GIN indexes (Generalized Inverted Index). These indexes map words to the documents that contain them, making search operations nearly instant.

## The Recommendation Engine: Collaborative Filtering

Our recommendation engine is based on collaborative filtering. This is the same algorithm Netflix uses to suggest movies and Spotify uses to suggest songs. It works by finding users with similar reading patterns and suggesting fics that similar users enjoyed but you haven't read yet.

Here's how it works in practice:

```
User A reads: Fic 1 (★★★★★), Fic 2 (★★★★), Fic 3 (★★★)
User B reads: Fic 1 (★★★★), Fic 2 (★★★★★), Fic 4 (★★★★)
User C reads: Fic 2 (★★★★), Fic 3 (★★★★★), Fic 5 (★★★★)

Similarity(A, B) = 0.67 (share 2 of 3 fics, similar ratings)
Similarity(A, C) = 0.67 (share 2 of 3 fics, similar ratings)
Similarity(B, C) = 0.33 (share 1 of 3 fics)

User A might like: Fic 4 (User B rated it highly) and Fic 5 (User C rated it highly)
```

The similarity score tells us how much two users' reading habits overlap. Users with high similarity are likely to enjoy similar fics. By recommending fics that similar users enjoyed, we leverage the collective taste of the community.

The Rust backend computes these similarities efficiently using vector math. For each user, we create a vector of their reading history (which fics they've read and how they rated them). The cosine similarity between two vectors tells us how similar their tastes are:

```
cosine_similarity(A, B) = (A · B) / (|A| × |B|)
```

This gives us a score between 0 and 1, where 1 means identical tastes and 0 means completely different tastes. We only recommend fics from users with similarity scores above 0.3 (somewhat similar tastes).

The recommendation process:
1. Find the top 20 users most similar to the current user
2. Collect fics those users have read that the current user hasn't
3. Weight by similarity score and rating
4. Return the top 10 recommendations

## The Collection Worker: Scraping Fanfiction Sites

FicHub doesn't just have the fics you manually add — it has a collection worker that automatically scrapes fanfiction sites and adds new fics to the database.

The worker runs as a background process, checking for new fics periodically:

```rust
// Simplified collection worker logic
async fn collect_fics() {
    let sites = vec![
        Site::AO3,      // Archive of Our Own
        Site::FFNet,     // FanFiction.net
        Site::Wattpad,
    ];

    for site in sites {
        println!("Scraping {}...", site.name());
        let fics = scrape_site(site).await?;
        let mut added = 0;
        let mut skipped = 0;

        for fic in fics {
            if !exists_in_database(&fic).await {
                insert_fic(&fic).await?;
                added += 1;
            } else {
                skipped += 1;
            }
        }

        println!("  Added: {}, Skipped: {}", added, skipped);

        // Rate limiting: wait between sites
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}
```

The worker is smart about scraping:
- **Respects robots.txt**: It only scrapes what sites allow. Before scraping a site, it fetches the robots.txt file and follows the rules.
- **Rate limits requests**: It doesn't hammer the site with too many requests too quickly. A 30-second delay between sites, and 2-second delays between individual fic pages.
- **Deduplicates**: It checks if a fic already exists in the database before adding it, using the site's unique ID as the key.
- **Extracts metadata**: Title, author, summary, tags, word count, kudos, comments, status, and publication date.
- **Handles errors gracefully**: If a page fails to load, it logs the error and moves on to the next one. It doesn't crash.

This is running 24/7 on the Orange Pi, quietly building up a database of fanfiction. Over time, FicHub's database grows, and recommendations get better because there's more data to work with.

## The Community Features: Suggestions, Voting

FicHub isn't just a database — it's a community. Users can:

- **Suggest new fics**: If you find a great fic that's not in FicHub, you can suggest it. Other users vote on whether it should be added. Suggestions with enough upvotes are automatically added to the database.
- **Vote on categorization**: Fics can be tagged with genres, tropes, and ratings. Users vote on whether the tags are accurate. This helps keep the metadata clean.
- **Report issues**: If metadata is wrong or a link is broken, users can flag it. Community moderation keeps the database accurate.

These community features are powered by the same Rust backend. The voting system uses a simple upvote/downvote mechanism:

```sql
-- Vote on a suggestion
INSERT INTO votes (user_id, suggestion_id, value)
VALUES ($1, $2, $3)
ON CONFLICT (user_id, suggestion_id)
DO UPDATE SET value = $3;

-- Get suggestion with vote count
SELECT s.*, SUM(v.value) as vote_count
FROM suggestions s
JOIN votes v ON s.id = v.suggestion_id
GROUP BY s.id
ORDER BY vote_count DESC;

-- Auto-approve suggestions with 10+ upvotes
UPDATE suggestions
SET status = 'approved'
WHERE id IN (
    SELECT s.id
    FROM suggestions s
    JOIN votes v ON s.id = v.suggestion_id
    GROUP BY s.id
    HAVING SUM(v.value) >= 10
);
```

This creates a self-curating database. The community helps keep the data accurate and discovers new fics that the automated scraper might miss.

## The Search System: Syntax Parser + Full-Text Search

Remember when we built the search syntax parser in an earlier chapter? This is where it shines. Users can write complex queries like:

```
romance AND hurt/comfort NOT angst
word_count:>50000 kudos:>1000
author:SomeAuthor status:complete
```

The syntax parser tokenizes this query into structured data:

```rust
struct SearchQuery {
    must: Vec<String>,      // AND terms
    must_not: Vec<String>,  // NOT terms
    filters: Vec<Filter>,   // Field-specific filters
}

struct Filter {
    field: String,   // "word_count", "kudos", "author", "status"
    op: Operator,    // ">", "<", ">=", "<=", "="
    value: String,   // "50000", "SomeAuthor", "complete"
}
```

Then PostgreSQL translates this into an efficient query:

```sql
SELECT f.*, ts_rank(
    to_tsvector('english', f.title || ' ' || f.summary),
    to_tsquery('english', 'romance & hurt/comfort & !angst')
) as rank
FROM fictions f
WHERE
    to_tsvector('english', f.title || ' ' || f.summary)
    @@ to_tsquery('english', 'romance & hurt/comfort & !angst')
    AND f.word_count > 50000
    AND f.kudos > 1000
    AND f.author = 'SomeAuthor'
    AND f.status = 'complete'
ORDER BY rank DESC
LIMIT 20;
```

The `ts_rank` function ranks results by how relevant they are to the search terms. The higher the rank, the better the match. The GIN index makes this query execute in milliseconds, even with millions of fics in the database.

## Memory Usage: The Rust Backend Uses Only 1.7MB

Here's something that might blow your mind. Let's check the memory usage of the Rust backend:

```bash
# On the server
ps aux | grep fichub-backend
```

```
fichub    1234  0.2  0.1  1740  580 ?  Ssl  10:00   0:05 /usr/local/bin/fichub-backend
```

The `1740` in the RSS column means the backend is using 1.7 MB of RAM. That's not a typo. One point seven megabytes.

For comparison:
- A typical Node.js app: 50-200 MB
- A typical Python app: 30-100 MB
- A typical Go app: 5-20 MB
- FicHub Rust backend: **1.7 MB**

This is possible because Rust doesn't need a garbage collector, doesn't need a runtime environment, and compiles to efficient machine code. The binary is self-contained — no Node.js, no Python, no JVM required.

On an Orange Pi with 2GB of RAM, this leaves plenty of room for everything else:

```
Component              RAM Usage
───────────────────────────────────
Linux kernel + OS      ~200 MB
nginx                  ~5 MB
PostgreSQL             ~50 MB (with data cached)
Redis                  ~20 MB
FicHub backend         1.7 MB
Cloudflare tunnel      ~30 MB
───────────────────────────────────
Total                  ~307 MB
Free for other uses    ~1.7 GB
```

The entire FicHub stack — frontend, backend, database, cache, web server — runs comfortably on a $30 single-board computer and uses less than 15% of the available RAM.

## Performance: Fast Responses, Efficient Caching

Let's look at some real performance numbers:

**API response times** (measured with `curl`):
- Health check: 1-2ms
- Search query (cache hit): 2-5ms (served from Redis)
- Search query (cache miss): 5-20ms (computed and cached)
- Recommendation: 10-50ms (depends on cache hit)
- Fic detail: 3-8ms

**Page load times** (measured in Chrome DevTools):
- First visit: 200-500ms (downloading JavaScript)
- Subsequent visits: 50-100ms (cached)
- Navigation between pages: 20-50ms (instant, no page reload)

**Cache hit rates**:
- Static assets (immutable): 99%+ (cached in browser for a year)
- API responses: 60-80% (Redis caching)
- Database queries: 40-60% (PostgreSQL buffer cache)

The key to performance is caching at every level:

```
Layer 1: Browser cache
  ↓ (miss → request goes to server)
Layer 2: Cloudflare CDN cache
  ↓ (miss → request hits your server)
Layer 3: nginx proxy cache
  ↓ (miss → request goes to backend)
Layer 4: Redis application cache
  ↓ (miss → request hits database)
Layer 5: PostgreSQL buffer cache
  ↓ (miss → disk I/O, rare)
Layer 6: Disk (SSD or microSD)
```

Each layer reduces the work the server needs to do. A user in Japan and a user in Brazil both get fast responses because Cloudflare has servers everywhere. Most requests never even reach your Orange Pi — they're served from Cloudflare's global network.

> **Try It Yourself**
>
> Check your FicHub's performance:
> ```bash
> # Check API response time
> curl -o /dev/null -s -w "Total time: %{time_total}s\n" http://localhost/api/v1/health
>
> # Check memory usage
> ps aux | grep fichub-backend
>
> # Check Redis hit rate
> redis-cli info stats | grep keyspace_hits
>
> # Check total server resource usage
> free -h
> ```
>
> You'll be amazed at how fast and lightweight your app is.

---

# Chapter 36: What's Next?

## Ideas for Improving FicHub

You've built something incredible — a full-stack web application with a Rust backend, SvelteKit frontend, PostgreSQL database, Redis cache, recommendation engine, and deployment pipeline. But there's always more to build. Here are some ideas to take FicHub to the next level.

### Adding More Fanfiction Sites

Right now, FicHub scrapes AO3, FanFiction.net, and Wattpad. But there are dozens more:

- **Quotev**: Popular for quizzes and fanfiction
- **LiveJournal**: Old-school but still active
- **Tumblr**: Lots of fanfiction posted as text posts
- **Archive of Our Own (AO3) series**: Collections of related fics
- **SpaceBattles/Sufficient Velocity**: Forum-based fiction
- **Royal Road**: Original fiction and fanfiction

Each site has different HTML structures, different rate limits, and different rules. Adding a new site means writing a new scraper that respects the site's terms of service.

```rust
// Adding a new site to the collection worker
impl Site {
    fn scraper(&self) -> Box<dyn Scraper> {
        match self {
            Site::AO3 => Box::new(AO3Scraper::new()),
            Site::FFNet => Box::new(FFNetScraper::new()),
            Site::Wattpad => Box::new(WattpadScraper::new()),
            Site::Quotev => Box::new(QuotevScraper::new()),  // New!
        }
    }
}
```

> **Try It Yourself**
>
> Pick a fanfiction site that isn't currently supported. Open it in your browser, view the source (Ctrl+U), and identify where the fic metadata is in the HTML. Write a simple Rust function that extracts:
> - Title and author
> - Summary
> - Tags/genres
> - Word count
> - Kudos/favorites
>
> Start with just extracting the data — you can worry about rate limiting and error handling later. Even a simple scraper that handles just one page format is a great start.

### User Accounts and Bookmarks

FicHub currently doesn't have user accounts. Adding them would unlock:

- **Bookmarks**: Save fics to read later
- **Reading history**: Track what you've read
- **Ratings**: Rate fics you've read
- **Personalized recommendations**: Better recommendations based on your reading history
- **Lists**: Create curated lists of fics (e.g., "Best Harry Potter fics under 10k words")
- **Notifications**: Get notified when fics you're subscribed to are updated

You could implement authentication with:
- **Email/password**: Classic but effective. Hash passwords with bcrypt.
- **OAuth**: Sign in with Google, GitHub, or AO3. Uses their identity system.
- **Magic links**: Passwordless authentication via email. Send a link, click it, you're logged in.

The backend changes would be significant — you'd need a users table, session management, and authorization checks on API endpoints. But the pattern is well-established, and Rust makes it straightforward.

### A Mobile App

FicHub works great on mobile browsers, but a native app could offer:

- **Offline reading**: Download fics for reading without internet
- **Push notifications**: Get notified when new fics match your preferences
- **Better performance**: Native apps are faster than web apps
- **App store presence**: Easier to discover

You could build a mobile app with:
- **Flutter**: Cross-platform (iOS and Android) with Dart. Google's framework.
- **React Native**: Cross-platform with JavaScript. Based on React.
- **Swift/Kotlin**: Native for each platform. Best performance, but two codebases.

Or, since you're already using Svelte, you could use **SvelteKit with Capacitor** to wrap your web app in a native shell. This gives you the best of both worlds — write once, deploy everywhere. Capacitor takes your web app and wraps it in a native container, giving you access to device features like the camera, push notifications, and file system.

## Contributing to Open Source

If you enjoyed building FicHub, consider contributing to open-source projects. The open-source community is welcoming and grateful for contributions.

**Where to start:**
- **Svelte**: Help improve the framework you just learned. They have a very active Discord community.
- **SvelteKit**: Contribute to the build tools and adapters. Great for understanding how frameworks work.
- **Rust**: The language has a thriving ecosystem with many beginner-friendly projects. Check out "good first issue" labels on GitHub.
- **PostgreSQL**: Help improve the database that powers your app. Even documentation contributions are valuable.
- **Fanfiction tools**: Build tools for the fanfiction community. There's a lot of room for improvement.

**Types of contributions:**
- **Bug fixes**: Find and fix bugs in existing code. Start with issues labeled "good first issue."
- **Documentation**: Write or improve docs. This is one of the best ways to learn a project's codebase.
- **Features**: Add new functionality. Discuss your idea in an issue first.
- **Reviews**: Review other people's pull requests. Even saying "this looks good" helps.
- **Issues**: Report bugs or suggest improvements. Good bug reports are incredibly valuable.

**How to contribute:**
1. Find a project you use and care about
2. Read their contributing guidelines (usually in `CONTRIBUTING.md`)
3. Start with small issues labeled "good first issue" or "help wanted"
4. Fork the repository, make your changes, and submit a pull request
5. Be patient — maintainers are usually volunteers and may take a few days to respond

> **Try It Yourself**
>
> Pick one open-source project you use. Read through their issues list and find one labeled "good first issue." Try to fix it. Even if you don't succeed, you'll learn a lot about how the project works. And if you do succeed, you've just made a contribution that helps everyone who uses that project.

## Learning More About SvelteKit

You've learned the fundamentals of SvelteKit, but there's much more to explore:

**Advanced routing:**
- **Route groups**: Organize routes without affecting the URL. Use parentheses: `(app)/+page.svelte`
- **Rest parameters**: Catch-all routes for dynamic paths: `[...slug]/+page.svelte`
- **Optional parameters**: Routes that work with or without a segment: `[[slug]]/+page.svelte`

**Server-side rendering:**
- `+page.server.js`: Load data on the server before rendering
- `+layout.server.js`: Share data across all pages in a layout
- Form actions: Handle form submissions without JavaScript (progressive enhancement)
- Cookies and sessions: Manage user state across requests

**Streaming and Suspense:**
- Stream data as it becomes available
- Show loading states while waiting for data
- Progressive enhancement — the page works even without JavaScript

**Deployment options:**
- Vercel: Zero-config deployment for SvelteKit
- Netlify: Similar to Vercel
- Cloudflare Workers: Edge computing (code runs in data centers worldwide)
- Self-hosted: What you learned in this book

Check out the [SvelteKit documentation](https://kit.svelte.dev/docs) for detailed guides on all these topics. The Svelte tutorial at [learn.svelte.dev](https://learn.svelte.dev) is also excellent for interactive learning.

## Learning More About Rust

Rust is a deep language with many facets. Here's what you can explore next:

**Async Rust:**
- Tokio: The async runtime you used in FicHub. Go deeper with its features.
- Async traits: Defining async interfaces for different implementations.
- Futures: Understanding how async works under the hood.
- Channels: Communication between async tasks.

**Web frameworks:**
- Axum: The framework we used — explore its middleware, extractors, and tower integration.
- Actix-Web: Another popular Rust web framework, known for extreme performance.
- Warp: A filters-based web framework with a different philosophy.

**Database access:**
- SQLx: Compile-time checked SQL queries (catches SQL errors before you run the code)
- Diesel: An ORM for Rust (object-relational mapping)
- SeaORM: A modern, async-first Rust ORM

**Systems programming:**
- File I/O: Reading and writing files efficiently with buffered I/O
- Networking: Building TCP/UDP servers from scratch
- Concurrency: Threads, channels, and async patterns
- Serialization: Serde (what we used) and other formats like MessagePack

**Resources:**
- [The Rust Programming Language](https://doc.rust-lang.org/book/): The official Rust book — read it cover to cover
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/): Learn by doing
- [Rustlings](https://github.com/rust-lang/rustlings): Small exercises to practice
- [Are We Web Yet?](https://www.arewewebyet.org/): Rust web ecosystem overview
- [Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/): Learn Rust's ownership model

## The Power of Building Things Yourself

Throughout this book, you've built a complete web application from scratch. You didn't use a template or a no-code tool. You wrote every line of code. You made every design decision. You solved every bug.

This is incredibly empowering. You now understand how web applications work at every level:

- How HTML, CSS, and JavaScript create user interfaces
- How browsers request and render pages
- How servers handle incoming requests
- How databases store and query data
- How caching improves performance
- How deployment makes your app accessible to the world
- How background workers keep data fresh
- How recommendation algorithms leverage community data

This knowledge transfers to any web project. Want to build a blog? You know how. A social network? You know the patterns. An e-commerce site? You understand the stack. A real-time chat app? You have the tools. A recipe manager? You've got the skills. A personal portfolio? Easy.

The tools might change — today it's SvelteKit and Rust, tomorrow it might be something else. But the concepts are the same. Understanding the fundamentals means you can adapt to new technologies quickly. When a new framework comes along, you'll be able to evaluate it critically: "Does this solve a problem I actually have? What are the tradeoffs? How does it compare to what I already know?"

**The debugging mindset:**
When something breaks, you now know how to investigate:
1. Check the browser console for JavaScript errors
2. Check the server logs for backend errors
3. Check the database for data issues
4. Use network tools to trace requests
5. Add logging to understand what's happening
6. Isolate the problem — is it frontend, backend, or database?
7. Read the error message carefully — it usually tells you exactly what's wrong

**The performance mindset:**
You now think about efficiency:
1. What can be cached? (Almost everything)
2. What can be compressed? (Text, JSON, HTML, CSS, JavaScript)
3. What can be lazy-loaded? (Images, scripts, routes)
4. What's the minimum data needed? (Don't fetch everything, fetch what you need)
5. Where are the bottlenecks? (Profile before optimizing)
6. Is this N+1 query problem? (Batch database queries)

**The security mindset:**
You now think about protecting your users:
1. Never trust user input (always validate and sanitize)
2. Use HTTPS everywhere (Cloudflare makes this easy)
3. Store passwords securely (hashed and salted with bcrypt)
4. Validate permissions on every request (not just the UI)
5. Keep dependencies updated (security patches matter)
6. Don't expose internal details (server versions, stack traces)

## You Are Now a Web Developer!

Congratulations! You've completed a journey that many professional developers spend years on. You've built a full-stack application with:

- A modern frontend framework (SvelteKit)
- A systems programming language (Rust)
- A relational database (PostgreSQL)
- A caching layer (Redis)
- A recommendation engine (collaborative filtering)
- A search system (syntax parser + full-text search)
- A background worker (collection scraper)
- A deployment pipeline (cross-compilation + rsync + systemd)
- A reverse proxy (nginx)
- A CDN and tunnel (Cloudflare)

This is real software. It's not a toy project or a tutorial exercise. FicHub is a functional application that serves real content to real users. It runs 24/7 on a $30 computer in your home.

The skills you've learned are in demand. Companies are looking for developers who can build full-stack applications, work with databases, deploy to servers, and write efficient code. You can do all of these things.

**What to do next:**

1. **Deploy FicHub and share it**: Put it on the internet. Let your friends use it. Get feedback. There's nothing like seeing real people use something you built.

2. **Add a feature**: Pick one of the ideas from this chapter and build it. Start small — even a simple bookmark feature is a great exercise.

3. **Read the docs**: Dive deeper into SvelteKit, Rust, PostgreSQL, or whatever interests you most. The documentation is your friend.

4. **Build something new**: Apply what you've learned to a different project. Maybe a personal blog, a recipe manager, or a game.

5. **Join a community**: Join the Svelte Discord, Rust users forum, or local developer meetup. Learning with others is faster and more fun.

6. **Teach someone else**: The best way to solidify your knowledge is to explain it to someone. Write a blog post, make a video, or help a friend get started.

**Remember:**
- Every expert was once a beginner
- Every bug you fix teaches you something
- Every feature you build makes you better
- Every deployment teaches you about real-world systems
- Every user who enjoys your app makes it worth it

You started this book knowing nothing about SvelteKit or Rust (or maybe you knew a little). Now you've built a complete web application. That's something to be proud of.

The fanfiction community now has a tool that helps readers discover great stories. You built that. You made it happen.

Welcome to web development. You belong here.

---

> **Try It Yourself**
>
> Take a moment to reflect on your journey. Open your terminal and look at what you've built:
>
> ```bash
> # Count the lines of code you wrote
> find . -name "*.svelte" -o -name "*.rs" -o -name "*.sql" | xargs wc -l | tail -1
>
> # Check the project size
> du -sh .
>
> # Look at your git history
> git log --oneline | head -20
>
> # See how many files you created
> find . -type f | wc -l
> ```
>
> That's a lot of code, and it all does something useful. You should be proud of what you've accomplished. Every line of code represents a problem you solved. Every file represents a feature you implemented. Every commit represents progress you made.
>
> Now go build something amazing.

---

*In the next and final part, we'll look at maintaining and growing FicHub — monitoring, updating, and building a community around your creation.*
