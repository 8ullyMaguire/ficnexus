# Part 1: Welcome to Web Development

---


## Chapter 1: What We're Building

Welcome, friend. Pull up a chair. You're about to learn something amazing.

By the time you finish this book, you'll have built a real, working website from scratch—a website that people actually use. Not a toy. Not a "hello world" page that sits in a drawer. A real application that does something useful and looks great doing it.

The website is called **FicHub**.

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

## Chapter 2: How the Web Works

Before we write a single line of code, let's understand what we're working with. The web is the most complex system humans have ever built, but the core ideas are surprisingly simple. Let's break it down.

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
    // ... (truncated)
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

## Chapter 3: Setting Up Your Workshop

Time to get our hands dirty. In this chapter, we're setting up your development environment—the tools you'll use every day to write, run, and test your code.

Think of this like setting up a workshop before starting a woodworking project. You need the right tools, organized and ready to go.

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

    // ... (truncated)
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

>
> 1. Look at the page in your browser. Click around on the links.
    // ... (truncated)

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
    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
  padding: var(--space-sm) var(--space-md);
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
}
```

When you want to change the theme, you change the variables in one place (`:root`), and everything updates automatically. It's like having a master light switch that controls every light in the house.

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
    // ... (truncated)

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
    // ... (truncated)
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
    // ... (truncated)
      </div>
      <button class="download-btn">Download EPUB</button>
    </div>
  </div>
</body>
```

Now you have a three-column grid of cards on desktop, two columns on tablet, and one column on phone. Resize your browser window to see the responsive layout in action.

>
> 1. Open your `card.html` file in the browser.
> 2. Make the browser window really wide. You should see three cards in a row.
> 3. Slowly narrow the window. At around 900px, the cards should rearrange into two columns.
> 4. Narrow it further. Below 600px, you should see one card per row—like a phone screen.
> 5. In Chrome, press `F12` to open Developer Tools, then click the "Toggle Device Toolbar" icon (looks like a phone and tablet). You can now see exactly how your page looks on different phone models.
>
> This is how professional web developers test responsive design. The browser's built-in tools simulate different screen sizes.

# Part 2: SvelteKit Fundamentals

*Welcome back, fellow builder! In Part 1, we got our feet wet with HTML, CSS, and JavaScript — the three ingredients that make up every web page. Now it's time to level up. We're going to meet SvelteKit, a modern framework that makes building web apps feel like snapping LEGO bricks together. Let's dive in!*

---


## Chapter 5: What is SvelteKit?

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
    // ... (truncated)
│   └── images/
├── svelte.config.js       ← SvelteKit configuration
├── vite.config.ts         ← Vite build configuration
├── tsconfig.json          ← TypeScript configuration
└── package.json           ← Dependencies and scripts
```

Don't worry if this feels like a lot — we'll build this structure step by step in the next chapter. For now, just remember: **files are routes**.

The magic of file-based routing is that it's **obvious**. You don't need to open a separate router configuration file to understand how URLs map to pages. Just look at the folder structure! If you see `blog/[slug]/+page.svelte`, you know there's a dynamic blog route. It's self-documenting.

> **🧪 Try It Yourself:** Before we move on, think about a website you use every day (like YouTube or Instagram). Can you figure out what its routes might be? YouTube has `/watch`, `/channel`, `/playlist`, and `/results` routes. Instagram has `/`, `/explore`, `/accounts`, and `/direct`. Once you start thinking about URLs as file paths, the whole web starts to make more sense!

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

    // ... (truncated)
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

    // ... (truncated)
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

    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
  
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
    // ... (truncated)
  
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
    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
<div class="stats">
  {@render statCard('Users', stats.users)}
  {@render statCard('Posts', stats.posts)}
  {@render statCard('Comments', stats.comments)}
</div>
```

> **💡 Snippets vs Components:** Use snippets when you want to reuse markup *within* the same component. Use components when you want to reuse markup *across* different components. Snippets are private to the component they're defined in — like helper functions that only exist inside one file.

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
    // ... (truncated)
  
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
    // ... (truncated)
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

    // ... (truncated)
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
    // ... (truncated)
  
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
    // ... (truncated)
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
    // ... (truncated)
  
  .sidebar a:hover {
    color: #3b82f6;
  }
</style>
```

> **⚠️ Watch Out:** Layout load functions only run once for their section. If you navigate from `/blog/post-1` to `/blog/post-2`, the blog layout's load function doesn't re-run. Only the page's load function runs again. This is a performance optimization — if the categories don't change, why fetch them again? If you need the layout data to refresh, you can use `invalidate()`.

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
    // ... (truncated)
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
    // ... (truncated)

// Parameters<T>: Get the parameter types of a function
function createStory(title: string, wordCount: number) { /* ... */ }
type CreateParams = Parameters<typeof createStory>;
// [string, number]
```

These utility types are like power tools. Once you start using them, you'll wonder how you ever lived without them. They let you create exactly the type you need from existing types, rather than defining everything from scratch. The most common ones you'll use are `Partial<T>` (for optional updates), `Pick<T, K>` (for selecting specific fields), and `Omit<T, K>` (for removing fields).

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
    // ... (truncated)
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

    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
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

    // ... (truncated)
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
    // ... (truncated)
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

    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
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

    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
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

    // ... (truncated)
  (stories) => `Found ${stories.length} stories`,
  (error, canRetry) => canRetry
    ? `Error: ${error}. Click to retry.`
    : `Error: ${error}`
);
```

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

### Advanced Pattern: Type-Safe API Client

For larger projects, you might want a centralized API client that's fully type-safe:

```typescript
// src/lib/api-client.ts

type Method = "GET" | "POST" | "PUT" | "DELETE";

interface RequestOptions {
  method?: Method;
  body?: unknown;
    // ... (truncated)
const story = await api.getStory("abc-123");
const newStory = await api.createStory({
  title: "My Story",
  author: "Alice"
});
```

>
> Extend the `ApiClient` class above by adding:
> 1. A `searchStories(query: string)` method that returns `PaginatedResponse<StoryMetadata>`
> 2. An `exportStory(id: string, format: string)` method that returns `ExportResponse`
> 3. A `getRecommendations()` method that returns `PaginatedResponse<RecResult>`
>
> Make sure every method has proper type annotations and handles errors correctly.

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
    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
  source_id: number

  /** Author's numeric ID on the source site */
  author_id: number
}
```

Look at the types carefully:

- `chapters: number` — TypeScript uses `number` for both integers and floats. In JavaScript/TypeScript, there's no separate `int` type.
- `words: number` — Same thing. The Rust backend uses `i64` (a 64-bit integer), but TypeScript just calls it `number`.
- `status: string` — This could be a union type like `'ongoing' | 'complete' | 'hiatus' | 'cancelled'`, but the Rust backend doesn't constrain it, so we use `string` to be safe.

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

    // ... (truncated)
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

    // ... (truncated)
  total: number
  page: number
  per_page: number
  results: SearchResult[]
}
```

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

## The request() Helper Function

In FicHub's API client, we don't call `fetch()` directly for every request. Instead, we create a **helper function** that handles all the common stuff:

```typescript
const BASE_URL = ''  // Same origin — no need for full URL

/**
 * Make an HTTP request to the FicHub API.
 *
 * @param path - The API path (e.g., '/api/v0/epub')
 * @param options - Fetch options (method, body, etc.)
    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
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

    // ... (truncated)
    onretry={handleRetry}
  />
{:else if data}
  <FicDisplay {data} />
{/if}
```

The three states (`loading`, `error`, `data`) are mutually exclusive — only one is shown at a time. The `finally` block ensures `loading` is set to `false` whether the request succeeds or fails.

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
    // ... (truncated)
  page?: number

  /** Results per page (max: configured server limit) */
  per_page?: number
}
```

> **Watch Out!** The `include_tags` format is tricky! It's a comma-separated string of `type_id:name` pairs, like `"1:Harry Potter,2:Hermione Granger"`. The `type_id` is a number that identifies the tag category. Don't mix this up with regular query parameters.

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
    // ... (truncated)
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

>
> **Bonus challenge:** Add a "Clear all filters" button that resets to `defaultFilters()`. This is a common UX pattern that users love — it lets them start fresh without refreshing the page.

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

# Part 5: Building the UI Components

---


# Chapter 19: Global Styles and CSS Variables

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
    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)
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

## The parseSearchQuery() Function: Full Walkthrough

Here's the main function that ties everything together:

```typescript
export function parseSearchQuery(raw: string): SearchFilters {
  const filters = defaultFilters();
  if (!raw.trim()) return filters;

  const tokens = greedyTokenize(raw);
  const bareWords: string[] = [];

    // ... (truncated)
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

# Part 7: Testing Your Code

---


## Chapter 29: Why Test?

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
    // ... (truncated)
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
    // ... (truncated)

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
    // ... (truncated)
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
    // ... (truncated)
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
    // ... (truncated)

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

### Watch Out: Common Component Testing Mistakes

**Forgetting to await fireEvent** — Always `await fireEvent.click(...)`. Without the `await`, the event might not fully process before your assertions run. This causes flaky tests — they pass sometimes and fail other times.

**Using getBy when element might not exist** — If you're not sure an element is in the DOM, use `queryByText` instead. `getByText` throws an error if not found; `queryByText` returns `null`. Use `getByText` when you *expect* the element to be there (it's a test assertion in itself).

**Not waiting for async updates** — If a component fetches data and updates its display, you need `waitFor` to wait for the update. Don't just check immediately after clicking. The API call takes time, even in tests.

**Testing implementation details** — Don't check that a specific internal variable has a specific value. Check that the *user* can see what they expect to see. "The story title is visible" is better than "the component's `story` property equals X."

**Mocking too much** — Don't mock everything. Mock only what you need to control (like `fetch`). Let the component do its real work — rendering, state management, event handling. The more real code you test, the more confident you can be.

**Ignoring the DOM output** — When a test fails, look at the actual DOM output. Testing Library shows you what elements are in the DOM, which helps you understand why your query failed. It's like having a debugger for your tests.

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

```
