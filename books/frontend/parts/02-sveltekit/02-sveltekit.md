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
