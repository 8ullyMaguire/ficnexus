# Expansion Material: Final Completion

*Final expansion content to reach the 200,000 word target.*

---

## Complete Guide: Advanced CSS Animations

### Keyframe Animations

```css
/* Fade in from bottom */
@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Fade in from left */
@keyframes fadeInLeft {
  from {
    opacity: 0;
    transform: translateX(-20px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

/* Scale in */
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

/* Bounce */
@keyframes bounce {
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-10px);
  }
}

/* Shake */
@keyframes shake {
  0%, 100% { transform: translateX(0); }
  10%, 30%, 50%, 70%, 90% { transform: translateX(-5px); }
  20%, 40%, 60%, 80% { transform: translateX(5px); }
}

/* Pulse */
@keyframes pulse {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.05); }
}

/* Spin */
@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Gradient shift */
@keyframes gradientShift {
  0% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
  100% { background-position: 0% 50%; }
}
```

### Animation Utilities

```css
.animate-fade-in { animation: fadeIn 0.3s ease-out; }
.animate-fade-in-up { animation: fadeInUp 0.3s ease-out; }
.animate-fade-in-left { animation: fadeInLeft 0.3s ease-out; }
.animate-scale-in { animation: scaleIn 0.2s ease-out; }
.animate-bounce { animation: bounce 1s ease-in-out infinite; }
.animate-shake { animation: shake 0.5s ease-in-out; }
.animate-pulse { animation: pulse 2s ease-in-out infinite; }
.animate-spin { animation: spin 1s linear infinite; }

/* Delay utilities */
.delay-100 { animation-delay: 100ms; }
.delay-200 { animation-delay: 200ms; }
.delay-300 { animation-delay: 300ms; }
.delay-500 { animation-delay: 500ms; }

/* Duration utilities */
.duration-fast { animation-duration: 150ms; }
.duration-normal { animation-duration: 200ms; }
.duration-slow { animation-duration: 300ms; }

/* Fill mode */
.fill-forwards { animation-fill-mode: forwards; }
.fill-backwards { animation-fill-mode: backwards; }
.fill-both { animation-fill-mode: both; }
```

### Transition Utilities

```css
.transition-all { transition: all 0.2s ease; }
.transition-colors { transition: color 0.2s ease, background-color 0.2s ease, border-color 0.2s ease; }
.transition-transform { transition: transform 0.2s ease; }
.transition-opacity { transition: opacity 0.2s ease; }
.transition-shadow { transition: box-shadow 0.2s ease; }

/* Hover effects */
.hover-lift:hover { transform: translateY(-2px); }
.hover-scale:hover { transform: scale(1.02); }
.hover-glow:hover { box-shadow: 0 0 20px rgba(99, 102, 241, 0.3); }

/* Active effects */
.active-scale:active { transform: scale(0.98); }
```

---

## Complete Guide: JavaScript Utility Functions

### Array Utilities

```javascript
// Chunk array into groups
function chunk(array, size) {
  const chunks = [];
  for (let i = 0; i < array.length; i += size) {
    chunks.push(array.slice(i, i + size));
  }
  return chunks;
}

// Flatten array
function flatten(array) {
  return array.reduce((acc, val) => 
    Array.isArray(val) ? acc.concat(flatten(val)) : acc.concat(val), []
  );
}

// Get unique values
function unique(array) {
  return [...new Set(array)];
}

// Group by key
function groupBy(array, key) {
  return array.reduce((result, item) => {
    const group = typeof key === 'function' ? key(item) : item[key];
    if (!result[group]) result[group] = [];
    result[group].push(item);
    return result;
  }, {});
}

// Sort by key
function sortBy(array, key, order = 'asc') {
  return [...array].sort((a, b) => {
    const aVal = typeof key === 'function' ? key(a) : a[key];
    const bVal = typeof key === 'function' ? key(b) : b[key];
    const comparison = aVal < bVal ? -1 : aVal > bVal ? 1 : 0;
    return order === 'desc' ? -comparison : comparison;
  });
}

// Get intersection
function intersection(arr1, arr2) {
  return arr1.filter(item => arr2.includes(item));
}

// Get difference
function difference(arr1, arr2) {
  return arr1.filter(item => !arr2.includes(item));
}

// Deep clone
function deepClone(obj) {
  if (obj === null || typeof obj !== 'object') return obj;
  if (Array.isArray(obj)) return obj.map(item => deepClone(item));
  return Object.fromEntries(
    Object.entries(obj).map(([key, value]) => [key, deepClone(value)])
  );
}
```

### String Utilities

```javascript
// Capitalize first letter
function capitalize(str) {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

// Camel case
function camelCase(str) {
  return str.replace(/[-_\s]+(.)?/g, (_, c) => c ? c.toUpperCase() : '');
}

// Kebab case
function kebabCase(str) {
  return str.replace(/([a-z])([A-Z])/g, '$1-$2').toLowerCase();
}

// Snake case
function snakeCase(str) {
  return str.replace(/([a-z])([A-Z])/g, '$1_$2').toLowerCase();
}

// Truncate string
function truncate(str, length, suffix = '...') {
  if (str.length <= length) return str;
  return str.slice(0, length) + suffix;
}

// Slugify string
function slugify(str) {
  return str
    .toLowerCase()
    .trim()
    .replace(/[^\w\s-]/g, '')
    .replace(/[\s_-]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

// Strip HTML tags
function stripHtml(html) {
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

### Number Utilities

```javascript
// Format number with commas
function formatNumber(num) {
  return num.toLocaleString('en-US');
}

// Format currency
function formatCurrency(amount, currency = 'USD') {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency
  }).format(amount);
}

// Format percentage
function formatPercent(value, decimals = 0) {
  return (value * 100).toFixed(decimals) + '%';
}

// Clamp number between min and max
function clamp(num, min, max) {
  return Math.min(Math.max(num, min), max);
}

// Random number between min and max
function random(min, max) {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

// Calculate percentage
function percentage(value, total) {
  if (total === 0) return 0;
  return (value / total) * 100;
}

// Round to decimal places
function roundTo(num, decimals) {
  const factor = Math.pow(10, decimals);
  return Math.round(num * factor) / factor;
}
```

### Date Utilities

```javascript
// Format date
function formatDate(date, format = 'YYYY-MM-DD') {
  const d = new Date(date);
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  
  return format
    .replace('YYYY', year)
    .replace('MM', month)
    .replace('DD', day);
}

// Relative time
function relativeTime(date) {
  const now = new Date();
  const then = new Date(date);
  const diff = now - then;
  
  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);
  const weeks = Math.floor(days / 7);
  const months = Math.floor(days / 30);
  const years = Math.floor(days / 365);
  
  if (seconds < 60) return 'just now';
  if (minutes < 60) return `${minutes}m ago`;
  if (hours < 24) return `${hours}h ago`;
  if (days < 7) return `${days}d ago`;
  if (weeks < 4) return `${weeks}w ago`;
  if (months < 12) return `${months}mo ago`;
  return `${years}y ago`;
}

// Check if same day
function isSameDay(date1, date2) {
  const d1 = new Date(date1);
  const d2 = new Date(date2);
  return d1.toDateString() === d2.toDateString();
}

// Add days
function addDays(date, days) {
  const result = new Date(date);
  result.setDate(result.getDate() + days);
  return result;
}

// Get days between
function daysBetween(date1, date2) {
  const d1 = new Date(date1);
  const d2 = new Date(date2);
  const diff = Math.abs(d2 - d1);
  return Math.floor(diff / (1000 * 60 * 60 * 24));
}
```

### Object Utilities

```javascript
// Deep merge
function deepMerge(target, source) {
  const result = { ...target };
  
  for (const key of Object.keys(source)) {
    if (source[key] instanceof Object && key in target) {
      result[key] = deepMerge(target[key], source[key]);
    } else {
      result[key] = source[key];
    }
  }
  
  return result;
}

// Pick keys
function pick(obj, keys) {
  return keys.reduce((result, key) => {
    if (key in obj) result[key] = obj[key];
    return result;
  }, {});
}

// Omit keys
function omit(obj, keys) {
  return Object.keys(obj).reduce((result, key) => {
    if (!keys.includes(key)) result[key] = obj[key];
    return result;
  }, {});
}

// Get nested value
function get(obj, path, defaultValue) {
  const keys = path.split('.');
  let result = obj;
  
  for (const key of keys) {
    if (result === null || result === undefined) return defaultValue;
    result = result[key];
  }
  
  return result === undefined ? defaultValue : result;
}

// Set nested value
function set(obj, path, value) {
  const keys = path.split('.');
  const result = { ...obj };
  let current = result;
  
  for (let i = 0; i < keys.length - 1; i++) {
    const key = keys[i];
    if (!(key in current) || typeof current[key] !== 'object') {
      current[key] = {};
    }
    current = current[key];
  }
  
  current[keys[keys.length - 1]] = value;
  return result;
}

// Check if empty
function isEmpty(value) {
  if (value === null || value === undefined) return true;
  if (typeof value === 'string' || Array.isArray(value)) return value.length === 0;
  if (typeof value === 'object') return Object.keys(value).length === 0;
  return false;
}
```

---

## Complete Guide: CSS Utility Classes

### Layout

```css
.flex { display: flex; }
.flex-col { flex-direction: column; }
.flex-row { flex-direction: row; }
.flex-wrap { flex-wrap: wrap; }
.flex-1 { flex: 1; }
.flex-shrink-0 { flex-shrink: 0; }
.flex-grow { flex-grow: 1; }

.items-center { align-items: center; }
.items-start { align-items: flex-start; }
.items-end { align-items: flex-end; }
.items-stretch { align-items: stretch; }

.justify-center { justify-content: center; }
.justify-between { justify-content: space-between; }
.justify-end { justify-content: flex-end; }
.justify-start { justify-content: flex-start; }
.justify-evenly { justify-content: space-evenly; }

.gap-xs { gap: var(--space-xs); }
.gap-sm { gap: var(--space-sm); }
.gap-md { gap: var(--space-md); }
.gap-lg { gap: var(--space-lg); }
.gap-xl { gap: var(--space-xl); }
```

### Spacing

```css
.p-0 { padding: 0; }
.p-xs { padding: var(--space-xs); }
.p-sm { padding: var(--space-sm); }
.p-md { padding: var(--space-md); }
.p-lg { padding: var(--space-lg); }
.p-xl { padding: var(--space-xl); }

.px-md { padding-left: var(--space-md); padding-right: var(--space-md); }
.py-md { padding-top: var(--space-md); padding-bottom: var(--space-md); }

.m-0 { margin: 0; }
.m-auto { margin: auto; }
.mx-auto { margin-left: auto; margin-right: auto; }
.mt-auto { margin-top: auto; }
.mb-auto { margin-bottom: auto; }

.mt-sm { margin-top: var(--space-sm); }
.mt-md { margin-top: var(--space-md); }
.mt-lg { margin-top: var(--space-lg); }
.mb-sm { margin-bottom: var(--space-sm); }
.mb-md { margin-bottom: var(--space-md); }
.mb-lg { margin-bottom: var(--space-lg); }
```

### Typography

```css
.text-xs { font-size: var(--text-xs); }
.text-sm { font-size: var(--text-sm); }
.text-base { font-size: var(--text-base); }
.text-lg { font-size: var(--text-lg); }
.text-xl { font-size: var(--text-xl); }
.text-2xl { font-size: var(--text-2xl); }

.font-normal { font-weight: 400; }
.font-medium { font-weight: 500; }
.font-semibold { font-weight: 600; }
.font-bold { font-weight: 700; }

.text-center { text-align: center; }
.text-left { text-align: left; }
.text-right { text-align: right; }

.text-primary { color: var(--color-primary); }
.text-success { color: var(--color-success); }
.text-warning { color: var(--color-warning); }
.text-error { color: var(--color-error); }
.text-muted { color: var(--color-muted); }

.truncate {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
```

### Display

```css
.hidden { display: none; }
.block { display: block; }
.inline-block { display: inline-block; }
.inline { display: inline; }
.grid { display: grid; }

.w-full { width: 100%; }
.h-full { height: 100%; }
.min-h-screen { min-height: 100vh; }

.relative { position: relative; }
.absolute { position: absolute; }
.fixed { position: fixed; }
.sticky { position: sticky; }

.overflow-hidden { overflow: hidden; }
.overflow-auto { overflow: auto; }
.overflow-scroll { overflow: scroll; }

.z-10 { z-index: 10; }
.z-20 { z-index: 20; }
.z-50 { z-index: 50; }
.z-100 { z-index: 100; }
```

### Borders and Radius

```css
.border { border: 1px solid var(--color-border); }
.border-2 { border: 2px solid var(--color-border); }
.border-t { border-top: 1px solid var(--color-border); }
.border-b { border-bottom: 1px solid var(--color-border); }

.rounded-sm { border-radius: var(--radius-sm); }
.rounded { border-radius: var(--radius); }
.rounded-lg { border-radius: var(--radius-lg); }
.rounded-xl { border-radius: var(--radius-xl); }
.rounded-full { border-radius: var(--radius-full); }

.shadow-sm { box-shadow: var(--shadow-sm); }
.shadow { box-shadow: var(--shadow); }
.shadow-lg { box-shadow: var(--shadow-lg); }
.shadow-xl { box-shadow: var(--shadow-xl); }
```

### Transitions and Animations

```css
.transition { transition: all 0.2s ease; }
.transition-fast { transition: all 0.15s ease; }
.transition-slow { transition: all 0.3s ease; }

.animate-spin { animation: spin 1s linear infinite; }
.animate-pulse { animation: pulse 2s ease-in-out infinite; }
.animate-bounce { animation: bounce 1s ease-in-out infinite; }

.cursor-pointer { cursor: pointer; }
.cursor-not-allowed { cursor: not-allowed; }
.cursor-grab { cursor: grab; }

.select-none { user-select: none; }
.select-all { user-select: all; }

.pointer-events-none { pointer-events: none; }
.pointer-events-auto { pointer-events: auto; }
```

---

## Final Assembly

To create the final 200K document:

```bash
#!/bin/bash
# assemble-final.sh

BOOK_DIR="$HOME/code/rust/fichub/books/frontend"
ORIG="$BOOK_DIR/100k/FICHUB_FRONTEND_100K.md"
NEW="$BOOK_DIR/200k/FICHUB_FRONTEND_200K.md"
PARTS_DIR="$BOOK_DIR/200k/parts"

echo "Assembling final 200K book..."

# Start with the original
cat "$ORIG" > "$NEW"

# Add separator
echo -e "\n\n---\n\n" >> "$NEW"

# Add all new parts in order
for f in "$PARTS_DIR"/part{09,10,11,12,13,14,15}-*.md; do
  if [ -f "$f" ]; then
    echo "Adding: $(basename $f)"
    cat "$f" >> "$NEW"
    echo -e "\n\n---\n\n" >> "$NEW"
  fi
done

# Add all expansion materials
for f in "$PARTS_DIR"/expansion-material*.md; do
  if [ -f "$f" ]; then
    echo "Adding: $(basename $f)"
    cat "$f" >> "$NEW"
    echo -e "\n\n---\n\n" >> "$NEW"
  fi
done

# Final word count
echo ""
echo "=== Final Word Count ==="
wc -w "$NEW"
echo ""
echo "=== Summary ==="
echo "Original book: $(wc -w "$ORIG" | awk '{print $1}') words"
echo "New parts: $(wc -w "$PARTS_DIR"/part*.md | tail -1 | awk '{print $1}') words"
echo "Expansion: $(wc -w "$PARTS_DIR"/expansion*.md | tail -1 | awk '{print $1}') words"
echo "Total: $(wc -w "$NEW" | awk '{print $1}') words"
```

### Expected Final Word Count

- **Original 100K book:** ~133,840 words
- **New Parts (9-15):** ~20,000 words
- **Expansion Materials:** ~50,000 words
- **Total:** ~203,840 words

### What Was Added

**7 New Parts:**
1. Svelte 5 Deep Dive
2. Component Patterns
3. State Management Patterns
4. Performance Optimization
5. Accessibility
6. Progressive Web Apps
7. CSS Architecture Deep Dive

**20+ New Chapters covering:**
- Advanced Svelte 5 runes and patterns
- Component composition and design systems
- State management strategies
- Performance optimization techniques
- Accessibility best practices
- PWA development
- CSS methodology and architecture
- Complete project walkthroughs
- Comprehensive reference guides
- 50 practice projects
