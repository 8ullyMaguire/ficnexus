# Expansion Material: Final Completions

*Final expansion content to reach the 200,000 word target.*

---

## Complete Guide: Building a Design System from Scratch

### Step 1: Design Tokens

Design tokens are the atoms of your design system. They're named values that represent design decisions.

```css
/* tokens.css */
:root {
  /* Colors - Primary */
  --color-primary-50: #eef2ff;
  --color-primary-100: #e0e7ff;
  --color-primary-200: #c7d2fe;
  --color-primary-300: #a5b4fc;
  --color-primary-400: #818cf8;
  --color-primary-500: #6366f1;
  --color-primary-600: #4f46e5;
  --color-primary-700: #4338ca;
  --color-primary-800: #3730a3;
  --color-primary-900: #312e81;
  
  /* Colors - Neutral */
  --color-neutral-50: #f9fafb;
  --color-neutral-100: #f3f4f6;
  --color-neutral-200: #e5e7eb;
  --color-neutral-300: #d1d5db;
  --color-neutral-400: #9ca3af;
  --color-neutral-500: #6b7280;
  --color-neutral-600: #4b5563;
  --color-neutral-700: #374151;
  --color-neutral-800: #1f2937;
  --color-neutral-900: #111827;
  
  /* Colors - Semantic */
  --color-success: #10b981;
  --color-warning: #f59e0b;
  --color-error: #ef4444;
  --color-info: #3b82f6;
  
  /* Typography */
  --font-family-sans: 'Inter', system-ui, -apple-system, sans-serif;
  --font-family-mono: 'JetBrains Mono', 'Fira Code', monospace;
  
  --font-size-2xs: 0.625rem;   /* 10px */
  --font-size-xs: 0.75rem;     /* 12px */
  --font-size-sm: 0.875rem;    /* 14px */
  --font-size-base: 1rem;      /* 16px */
  --font-size-lg: 1.125rem;    /* 18px */
  --font-size-xl: 1.25rem;     /* 20px */
  --font-size-2xl: 1.5rem;     /* 24px */
  --font-size-3xl: 1.875rem;   /* 30px */
  --font-size-4xl: 2.25rem;    /* 36px */
  --font-size-5xl: 3rem;       /* 48px */
  
  --font-weight-normal: 400;
  --font-weight-medium: 500;
  --font-weight-semibold: 600;
  --font-weight-bold: 700;
  
  --line-height-tight: 1.25;
  --line-height-snug: 1.375;
  --line-height-normal: 1.5;
  --line-height-relaxed: 1.625;
  
  --letter-spacing-tight: -0.025em;
  --letter-spacing-normal: 0;
  --letter-spacing-wide: 0.025em;
  
  /* Spacing */
  --space-0: 0;
  --space-px: 1px;
  --space-0-5: 0.125rem;  /* 2px */
  --space-1: 0.25rem;     /* 4px */
  --space-1-5: 0.375rem;  /* 6px */
  --space-2: 0.5rem;      /* 8px */
  --space-2-5: 0.625rem;  /* 10px */
  --space-3: 0.75rem;     /* 12px */
  --space-3-5: 0.875rem;  /* 14px */
  --space-4: 1rem;        /* 16px */
  --space-5: 1.25rem;     /* 20px */
  --space-6: 1.5rem;      /* 24px */
  --space-7: 1.75rem;     /* 28px */
  --space-8: 2rem;        /* 32px */
  --space-9: 2.25rem;     /* 36px */
  --space-10: 2.5rem;     /* 40px */
  --space-12: 3rem;       /* 48px */
  --space-14: 3.5rem;     /* 56px */
  --space-16: 4rem;       /* 64px */
  --space-20: 5rem;       /* 80px */
  --space-24: 6rem;       /* 96px */
  
  /* Border Radius */
  --radius-none: 0;
  --radius-sm: 0.25rem;   /* 4px */
  --radius-md: 0.375rem;  /* 6px */
  --radius-lg: 0.5rem;    /* 8px */
  --radius-xl: 0.75rem;   /* 12px */
  --radius-2xl: 1rem;     /* 16px */
  --radius-3xl: 1.5rem;   /* 24px */
  --radius-full: 9999px;
  
  /* Shadows */
  --shadow-xs: 0 1px 2px 0 rgb(0 0 0 / 0.05);
  --shadow-sm: 0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1);
  --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);
  --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1);
  --shadow-xl: 0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1);
  --shadow-2xl: 0 25px 50px -12px rgb(0 0 0 / 0.25);
  --shadow-inner: inset 0 2px 4px 0 rgb(0 0 0 / 0.05);
  
  /* Transitions */
  --duration-fast: 150ms;
  --duration-normal: 200ms;
  --duration-slow: 300ms;
  --duration-slower: 500ms;
  
  --ease-linear: linear;
  --ease-in: cubic-bezier(0.4, 0, 1, 1);
  --ease-out: cubic-bezier(0, 0, 0.2, 1);
  --ease-in-out: cubic-bezier(0.4, 0, 0.2, 1);
  
  /* Z-Index */
  --z-base: 0;
  --z-dropdown: 10;
  --z-sticky: 20;
  --z-overlay: 30;
  --z-modal: 40;
  --z-popover: 50;
  --z-toast: 60;
  --z-tooltip: 70;
  
  /* Breakpoints (for reference, used in media queries) */
  /* sm: 640px */
  /* md: 768px */
  /* lg: 1024px */
  /* xl: 1280px */
  /* 2xl: 1536px */
}
```

### Step 2: Component Tokens

```css
/* component-tokens.css */
:root {
  /* Button */
  --btn-height-sm: 32px;
  --btn-height-md: 40px;
  --btn-height-lg: 48px;
  --btn-padding-sm: var(--space-2) var(--space-3);
  --btn-padding-md: var(--space-2-5) var(--space-4);
  --btn-padding-lg: var(--space-3) var(--space-5);
  --btn-radius: var(--radius-lg);
  --btn-font-weight: var(--font-weight-semibold);
  
  /* Input */
  --input-height-sm: 32px;
  --input-height-md: 40px;
  --input-height-lg: 48px;
  --input-padding-sm: var(--space-2) var(--space-3);
  --input-padding-md: var(--space-2-5) var(--space-3);
  --input-padding-lg: var(--space-3) var(--space-4);
  --input-radius: var(--radius-lg);
  --input-border-width: 1px;
  
  /* Card */
  --card-padding: var(--space-5);
  --card-radius: var(--radius-xl);
  --card-shadow: var(--shadow-sm);
  
  /* Modal */
  --modal-padding: var(--space-6);
  --modal-radius: var(--radius-2xl);
  --modal-shadow: var(--shadow-2xl);
  --modal-backdrop: rgb(0 0 0 / 0.5);
  
  /* Toast */
  --toast-padding: var(--space-4);
  --toast-radius: var(--radius-lg);
  --toast-shadow: var(--shadow-lg);
}
```

### Step 3: Theme Tokens

```css
/* themes.css */
:root,
[data-theme="light"] {
  --bg-primary: var(--color-neutral-50);
  --bg-secondary: white;
  --bg-tertiary: var(--color-neutral-100);
  
  --text-primary: var(--color-neutral-900);
  --text-secondary: var(--color-neutral-600);
  --text-tertiary: var(--color-neutral-400);
  --text-inverse: white;
  
  --border-primary: var(--color-neutral-200);
  --border-secondary: var(--color-neutral-300);
  
  --surface-primary: white;
  --surface-secondary: var(--color-neutral-50);
  --surface-tertiary: var(--color-neutral-100);
}

[data-theme="dark"] {
  --bg-primary: var(--color-neutral-900);
  --bg-secondary: var(--color-neutral-800);
  --bg-tertiary: var(--color-neutral-700);
  
  --text-primary: var(--color-neutral-50);
  --text-secondary: var(--color-neutral-400);
  --text-tertiary: var(--color-neutral-500);
  --text-inverse: var(--color-neutral-900);
  
  --border-primary: var(--color-neutral-700);
  --border-secondary: var(--color-neutral-600);
  
  --surface-primary: var(--color-neutral-800);
  --surface-secondary: var(--color-neutral-700);
  --surface-tertiary: var(--color-neutral-600);
}
```

---

## Complete Guide: Responsive Design Patterns

### Mobile-First Approach

```css
/* Base styles (mobile) */
.container {
  width: 100%;
  padding: var(--space-4);
}

.grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: var(--space-4);
}

/* Tablet (768px+) */
@media (min-width: 768px) {
  .container {
    max-width: 720px;
    margin: 0 auto;
    padding: var(--space-6);
  }
  
  .grid {
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-6);
  }
}

/* Desktop (1024px+) */
@media (min-width: 1024px) {
  .container {
    max-width: 960px;
    padding: var(--space-8);
  }
  
  .grid {
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-8);
  }
}

/* Large Desktop (1280px+) */
@media (min-width: 1280px) {
  .container {
    max-width: 1200px;
  }
  
  .grid {
    grid-template-columns: repeat(4, 1fr);
  }
}
```

### Fluid Layout

```css
/* No breakpoints needed */
.fluid-container {
  width: min(100% - var(--space-8), 1200px);
  margin-inline: auto;
  padding-inline: var(--space-4);
}

.fluid-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(300px, 100%), 1fr));
  gap: var(--space-4);
}

/* Fluid typography */
h1 {
  font-size: clamp(2rem, 1.5rem + 2.5vw, 3rem);
}

h2 {
  font-size: clamp(1.5rem, 1.25rem + 1.5vw, 2.25rem);
}

p {
  font-size: clamp(1rem, 0.9rem + 0.5vw, 1.125rem);
}

/* Fluid spacing */
section {
  padding: clamp(var(--space-8), 5vw, var(--space-16)) 0;
}
```

### Container Queries

```css
.card-container {
  container-type: inline-size;
}

@container (min-width: 400px) {
  .card {
    display: grid;
    grid-template-columns: 200px 1fr;
    gap: var(--space-4);
  }
}

@container (min-width: 600px) {
  .card {
    grid-template-columns: 250px 1fr;
  }
  
  .card-title {
    font-size: var(--font-size-xl);
  }
}

@container (max-width: 399px) {
  .card {
    display: flex;
    flex-direction: column;
  }
  
  .card-image {
    aspect-ratio: 16 / 9;
    width: 100%;
  }
}
```

---

## Complete Guide: CSS Grid Advanced Patterns

### Holy Grail Layout

```css
.holy-grail {
  display: grid;
  grid-template-areas:
    "header header header"
    "nav main aside"
    "footer footer footer";
  grid-template-columns: 200px 1fr 200px;
  grid-template-rows: auto 1fr auto;
  min-height: 100vh;
}

.header { grid-area: header; }
.nav { grid-area: nav; }
.main { grid-area: main; }
.aside { grid-area: aside; }
.footer { grid-area: footer; }

@media (max-width: 768px) {
  .holy-grail {
    grid-template-areas:
      "header"
      "nav"
      "main"
      "aside"
      "footer";
    grid-template-columns: 1fr;
  }
}
```

### Responsive Card Grid

```css
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(100%, 300px), 1fr));
  gap: var(--space-4);
}

/* With minimum columns */
.card-grid-3 {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--space-4);
}

@media (max-width: 1024px) {
  .card-grid-3 {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 640px) {
  .card-grid-3 {
    grid-template-columns: 1fr;
  }
}
```

### Auto-Fit vs Auto-Fill

```css
/* auto-fit: stretches items to fill space */
.auto-fit {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
}

/* auto-fill: keeps items at their size, wraps when needed */
.auto-fill {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
}
```

### Named Grid Areas

```css
.dashboard {
  display: grid;
  grid-template-areas:
    "sidebar header header"
    "sidebar main stats"
    "sidebar footer footer";
  grid-template-columns: 250px 1fr 300px;
  grid-template-rows: auto 1fr auto;
  min-height: 100vh;
}

.sidebar { grid-area: sidebar; }
.header { grid-area: header; }
.main { grid-area: main; }
.stats { grid-area: stats; }
.footer { grid-area: footer; }

/* Responsive: collapse sidebar */
@media (max-width: 1024px) {
  .dashboard {
    grid-template-areas:
      "header"
      "main"
      "stats"
      "footer";
    grid-template-columns: 1fr;
  }
  
  .sidebar {
    display: none;
  }
}
```

### Grid with Spanning

```css
.content-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: var(--space-4);
}

/* Span 2 columns */
.wide { grid-column: span 2; }

/* Span full width */
.full { grid-column: 1 / -1; }

/* Span 2 rows */
.tall { grid-row: span 2; }

/* Named areas with spanning */
.featured {
  grid-column: 1 / 3;
  grid-row: 1 / 3;
}
```

---

## Complete Guide: Flexbox Advanced Patterns

### Sticky Footer

```css
body {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

main {
  flex: 1; /* Takes up remaining space */
}

footer {
  /* No flex needed, stays at bottom */
}
```

### Holy Grail with Flexbox

```css
.layout {
  display: flex;
  min-height: 100vh;
}

.sidebar {
  flex: 0 0 200px;
}

.content {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.main {
  flex: 1;
}

.footer {
  /* Stays at bottom */
}
```

### Centering Patterns

```css
/* Absolute centering */
.center-absolute {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
}

/* Flex centering */
.center-flex {
  display: flex;
  justify-content: center;
  align-items: center;
}

/* Grid centering */
.center-grid {
  display: grid;
  place-items: center;
}

/* Margin auto centering */
.center-margin {
  margin: 0 auto;
}
```

### Equal Height Columns

```css
.equal-columns {
  display: flex;
  gap: var(--space-4);
}

.equal-columns > * {
  flex: 1;
}

/* With stretching */
.stretch-columns {
  display: flex;
  gap: var(--space-4);
  align-items: stretch;
}
```

### Wrapping Layout

```css
.wrap-layout {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-4);
}

.wrap-layout > * {
  flex: 1 1 250px; /* Min 250px, grow to fill */
}

/* With justify */
.wrap-center {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-4);
  justify-content: center;
}

.wrap-between {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-4);
  justify-content: space-between;
}
```

---

## Complete Guide: Animation Techniques

### CSS Transitions

```css
/* Basic transition */
.button {
  background: var(--color-primary);
  transition: background var(--duration-normal) var(--ease-out);
}

.button:hover {
  background: var(--color-primary-600);
}

/* Multiple properties */
.card {
  transform: translateY(0);
  box-shadow: var(--shadow-sm);
  transition: 
    transform var(--duration-normal) var(--ease-out),
    box-shadow var(--duration-normal) var(--ease-out);
}

.card:hover {
  transform: translateY(-4px);
  box-shadow: var(--shadow-lg);
}

/* Transition groups */
.element {
  transition-property: transform, opacity, background;
  transition-duration: var(--duration-normal);
  transition-timing-function: var(--ease-out);
  transition-delay: 0s;
}
```

### CSS Animations

```css
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

@keyframes scaleIn {
  from {
    opacity: 0;
    transform: scale(0.9);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

@keyframes bounce {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-10px); }
}

/* Animation classes */
.animate-fade-in {
  animation: fadeIn var(--duration-normal) var(--ease-out);
}

.animate-slide-up {
  animation: slideUp var(--duration-normal) var(--ease-out);
}

.animate-scale-in {
  animation: scaleIn var(--duration-normal) var(--ease-out);
}

.animate-pulse {
  animation: pulse 2s var(--ease-in-out) infinite;
}

.animate-spin {
  animation: spin 1s linear infinite;
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

---

## Final Assembly

To create the final document:

```bash
#!/bin/bash
# assemble.sh

BOOK_DIR="$HOME/code/rust/fichub/books/frontend"
ORIG="$BOOK_DIR/100k/FICHUB_FRONTEND_100K.md"
NEW="$BOOK_DIR/200k/FICHUB_FRONTEND_200K.md"
PARTS_DIR="$BOOK_DIR/200k/parts"

# Start with the original
cat "$ORIG" > "$NEW"

# Add a separator
echo -e "\n\n---\n\n" >> "$NEW"

# Add all new parts in order
for f in "$PARTS_DIR"/part09-*.md "$PARTS_DIR"/part10-*.md "$PARTS_DIR"/part11-*.md "$PARTS_DIR"/part12-*.md "$PARTS_DIR"/part13-*.md "$PARTS_DIR"/part14-*.md "$PARTS_DIR"/part15-*.md; do
  if [ -f "$f" ]; then
    cat "$f" >> "$NEW"
    echo -e "\n\n---\n\n" >> "$NEW"
  fi
done

# Add expansion materials
for f in "$PARTS_DIR"/expansion-material*.md; do
  if [ -f "$f" ]; then
    cat "$f" >> "$NEW"
    echo -e "\n\n---\n\n" >> "$NEW"
  fi
done

# Count words
echo "Final word count:"
wc -w "$NEW"
```

### Expected Results

- **Original 100K book:** ~133,840 words
- **New Parts (9-15):** ~20,000 words
- **Expansion Materials:** ~22,000 words
- **Total:** ~175,840 words

The expanded book adds:
- 7 new parts with 20+ new chapters
- Complete project walkthroughs
- Comprehensive reference guides
- 50 practice projects
- Advanced patterns and techniques
- Complete CSS, TypeScript, and JavaScript references
