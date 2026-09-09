# FicHub Current Frontend Design Language

## Stack
- **Framework:** SvelteKit (Svelte 5 runes)
- **Build:** Vite, `@sveltejs/adapter-static` (SPA mode)
- **Language:** TypeScript
- **Testing:** Vitest (unit), Playwright (E2E)
- **249 source files**

## Design Philosophy

FicHub follows a **content-first, minimal chrome** approach. The interface is designed to disappear — the fic content is the hero. Think "Kindle meets archive."

### Core Principles
1. **Readable** — typography-optimized for long-form reading
2. **Accessible** — WCAG 2.1 AA compliant, keyboard navigable
3. **Dark mode first** — dark theme is the default, light is available
4. **Mobile-first** — responsive design, bottom nav on mobile
5. **Progressive** — features unlock as users level up

## Color System

### CSS Custom Properties (Design Tokens)
```css
:root {
  /* Backgrounds */
  --color-bg: #0f0f14;           /* Main background (dark mode) */
  --color-bg-alt: #1a1a24;       /* Card/panel background */
  --color-surface: #22222e;      /* Elevated surfaces */
  --color-surface-hover: #2a2a38;

  /* Text */
  --color-text: #e8e8ed;         /* Primary text */
  --color-text-muted: #8888a0;   /* Secondary text */
  --color-text-faint: #55556a;   /* Disabled/placeholder */

  /* Accent */
  --color-primary: #6366f1;      /* Indigo-500 — primary actions */
  --color-primary-hover: #818cf8;
  --color-link: #818cf8;         /* Links */

  /* Status */
  --color-success: #22c55e;
  --color-warning: #f59e0b;
  --color-error: #ef4444;

  /* Borders */
  --color-border: #2a2a3a;
  --color-border-strong: #3a3a4a;

  /* Radius */
  --radius-sm: 6px;
  --radius-md: 8px;
  --radius-lg: 12px;
  --radius-xl: 16px;

  /* Spacing scale: 4px base */
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-6: 24px;
  --space-8: 32px;
}
```

### Light Mode Overrides
```css
:root[data-theme="light"] {
  --color-bg: #f8f8fa;
  --color-bg-alt: #ffffff;
  --color-surface: #f0f0f4;
  --color-text: #1a1a2e;
  --color-text-muted: #666680;
  --color-border: #e0e0e8;
}
```

## Typography

### Font Stack
```css
body {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-size: 16px;
  line-height: 1.6;
  color: var(--color-text);
}

/* Reading mode */
.reader-content {
  font-family: 'Literata', 'Georgia', serif;
  font-size: 18px;
  line-height: 1.8;
  max-width: 680px;
  margin: 0 auto;
}
```

### Type Scale
| Token | Size | Use |
|-------|------|-----|
| `text-xs` | 12px | Captions, badges |
| `text-sm` | 14px | Secondary text, meta |
| `text-base` | 16px | Body text |
| `text-lg` | 18px | Reading content |
| `text-xl` | 20px | Section headers |
| `text-2xl` | 24px | Page titles |
| `text-3xl` | 30px | Hero text |

## Component Patterns

### Cards
```css
.card {
  background: var(--color-bg-alt);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--space-4);
  transition: border-color 0.15s;
}
.card:hover {
  border-color: var(--color-border-strong);
}
```

### Buttons
```css
.btn-primary {
  background: var(--color-primary);
  color: white;
  padding: var(--space-2) var(--space-4);
  border-radius: var(--radius-sm);
  font-weight: 600;
  border: none;
  cursor: pointer;
  transition: background 0.15s;
}
.btn-primary:hover {
  background: var(--color-primary-hover);
}

.btn-secondary {
  background: transparent;
  color: var(--color-text);
  border: 1px solid var(--color-border);
  padding: var(--space-2) var(--space-4);
  border-radius: var(--radius-sm);
}
```

### Chips/Tags
```css
.chip {
  background: var(--color-surface);
  color: var(--color-text-muted);
  padding: 2px 8px;
  border-radius: 999px;
  font-size: var(--text-xs);
  white-space: nowrap;
}
```

### Inputs
```css
input, textarea, select {
  background: var(--color-surface);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  padding: var(--space-2) var(--space-3);
  font: inherit;
}
input:focus, textarea:focus {
  border-color: var(--color-primary);
  outline: none;
}
```

## Layout Patterns

### Page Container
```css
.page {
  max-width: 1200px;
  margin: 0 auto;
  padding: var(--space-6) var(--space-4);
}
```

### Grid
```css
.grid-2 {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--space-4);
}
@media (max-width: 768px) {
  .grid-2 { grid-template-columns: 1fr; }
}
```

### Flex Utilities
```css
.flex { display: flex; }
.flex-col { flex-direction: column; }
.items-center { align-items: center; }
.justify-between { justify-content: space-between; }
.gap-2 { gap: var(--space-2); }
.gap-4 { gap: var(--space-4); }
```

## Navigation

### TopNav (Desktop)
- Fixed top, full width
- Logo left, nav links center, user menu right
- Height: 56px
- Background: `var(--color-bg-alt)` with `border-bottom: 1px solid var(--color-border)`

### BottomNav (Mobile)
- Fixed bottom, full width
- 4-5 icon buttons
- Height: 56px
- Only visible on screens < 768px

## Animations

### Transitions
```css
/* Subtle hover effects */
transition: color 0.15s, background 0.15s, border-color 0.15s, opacity 0.15s;

/* Page transitions */
transition: opacity 0.2s ease;
```

### Loading States
- Skeleton placeholders with `@keyframes shimmer`
- Spinner: CSS-only rotating border
- No heavy animation libraries (no Framer Motion, GSAP)

## Icons

- **Lucide** icon set (SVG, inline)
- Consistent 20px size for nav, 16px for inline
- Stroke-based, matches the clean aesthetic

## Responsive Breakpoints

| Breakpoint | Width | Layout |
|-----------|-------|--------|
| Mobile | < 768px | Single column, bottom nav, stacked cards |
| Tablet | 768-1024px | Two-column, top nav, side panels |
| Desktop | > 1024px | Full layout, sidebar navigation |

## Dark/Light Mode

- Default: **Dark mode** (`data-theme="dark"` on `<html>`)
- Toggle in settings
- Stored in `user_prefs` table (key: `theme`)
- CSS custom properties switch via `[data-theme]` selector
- No flash of wrong theme (inline `<script>` in `<head>` reads preference)

## Accessibility

- **Focus visible** — custom focus ring using `outline: 2px solid var(--color-primary)`
- **Skip link** — "Skip to content" link at top
- **ARIA labels** — on all interactive elements
- **Keyboard navigation** — full tab order, Enter/Space to activate
- **Screen reader** — semantic HTML, `role` attributes where needed
- **Color contrast** — all text meets 4.5:1 ratio on dark backgrounds

## CSS Architecture

### Approach
- **Scoped CSS** — Svelte `<style>` blocks (auto-scoped to component)
- **CSS custom properties** — for theming and design tokens
- **No CSS framework** — no Tailwind, no Bootstrap, no CSS-in-JS
- **No preprocessor** — plain CSS (no Sass/Less)

### Why No Framework?
1. **Bundle size** — Tailwind adds ~10-30KB to every page
2. **Design consistency** — custom tokens enforce the design system
3. **Svelte scoping** — built-in CSS scoping eliminates most issues
4. **Simplicity** — fewer dependencies, fewer breaking changes

## i18n (Internationalization)

### System
- **15 languages** supported (English, Chinese, Spanish, etc.)
- Translation keys in `frontend/src/lib/i18n/dictionaries/*.ts`
- `t('key', { param })` function for translation lookup
- Locale stored in user prefs, defaults to browser locale

### Translation Key Convention
```
section.itemName
e.g.:
forum.postReply
settings.personalizedRecs
nav.search
```

## Component Organization

```
frontend/src/lib/components/
├── common/          # Shared UI (Button, Card, Modal, Toast)
├── home/            # Home dashboard widgets
├── search/          # Search facets, results
├── fic/             # Fic detail components
├── reader/          # Web reader components
├── forum/           # Forum components
├── admin/           # Admin panel components
├── social/          # Comments, reviews, kudos
└── layout/          # Nav, sidebar, footer
```

## State Management

- **Svelte 5 runes** — `$state`, `$derived`, `$effect`
- **No external state library** — no Redux, no Zustand, no Svelte stores (old pattern)
- **Auth state** — global `$state` in `auth.svelte.ts`
- **Page data** — passed via SvelteKit `load` functions or `onMount`
- **No global cache** — each page fetches independently

## Performance

- **Static adapter** — pre-built HTML/JS/CSS, served from NFS
- **Immutable assets** — hashed filenames, 1-year cache headers
- **Code splitting** — SvelteKit automatic route-based splitting
- **No heavy dependencies** — no lodash, no moment.js
- **Lazy loading** — images use `loading="lazy"`
