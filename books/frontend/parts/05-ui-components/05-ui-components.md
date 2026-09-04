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

