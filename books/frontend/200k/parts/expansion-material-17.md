# Expansion Material: Final Completion

*Final expansion content to reach the 200,000 word target.*

---

## Complete Guide: JavaScript Array Methods Deep Dive

### map

```javascript
// Transform each element
const numbers = [1, 2, 3, 4, 5];
const doubled = numbers.map(n => n * 2);
// [2, 4, 6, 8, 10]

// With index
const indexed = numbers.map((n, i) => `${i}: ${n}`);
// ['0: 1', '1: 2', '2: 3', '3: 4', '4: 5']

// Transform objects
const users = [
  { name: 'Alice', age: 25 },
  { name: 'Bob', age: 30 }
];
const names = users.map(u => u.name);
// ['Alice', 'Bob']

// Chain operations
const result = numbers
  .map(n => n * 2)
  .filter(n => n > 4)
  .reduce((sum, n) => sum + n, 0);
// 18
```

### filter

```javascript
// Keep elements that match condition
const numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
const evens = numbers.filter(n => n % 2 === 0);
// [2, 4, 6, 8, 10]

// Filter objects
const users = [
  { name: 'Alice', age: 25, active: true },
  { name: 'Bob', age: 30, active: false },
  { name: 'Charlie', age: 35, active: true }
];
const activeUsers = users.filter(u => u.active);
// [{ name: 'Alice', ... }, { name: 'Charlie', ... }]

// Filter with complex condition
const adults = users.filter(u => u.age >= 18 && u.active);
```

### reduce

```javascript
// Sum numbers
const numbers = [1, 2, 3, 4, 5];
const sum = numbers.reduce((acc, n) => acc + n, 0);
// 15

// Find maximum
const max = numbers.reduce((max, n) => n > max ? n : max, numbers[0]);
// 5

// Group by key
const items = [
  { type: 'fruit', name: 'apple' },
  { type: 'vegetable', name: 'carrot' },
  { type: 'fruit', name: 'banana' }
];
const grouped = items.reduce((acc, item) => {
  if (!acc[item.type]) acc[item.type] = [];
  acc[item.type].push(item);
  return acc;
}, {});
// { fruit: [...], vegetable: [...] }

// Flatten array
const nested = [[1, 2], [3, 4], [5]];
const flat = nested.reduce((acc, arr) => acc.concat(arr), []);
// [1, 2, 3, 4, 5]
```

### find and findIndex

```javascript
// Find first matching element
const numbers = [1, 2, 3, 4, 5];
const found = numbers.find(n => n > 3);
// 4

// Find index
const index = numbers.findIndex(n => n > 3);
// 3

// Find object
const users = [
  { id: 1, name: 'Alice' },
  { id: 2, name: 'Bob' }
];
const user = users.find(u => u.id === 2);
// { id: 2, name: 'Bob' }
```

### some and every

```javascript
// Check if ANY element matches
const numbers = [1, 2, 3, 4, 5];
const hasEven = numbers.some(n => n % 2 === 0);
// true

// Check if ALL elements match
const allPositive = numbers.every(n => n > 0);
// true

// Check complex condition
const users = [
  { name: 'Alice', age: 25 },
  { name: 'Bob', age: 30 }
];
const allAdults = users.every(u => u.age >= 18);
// true
```

### sort

```javascript
// Sort numbers
const numbers = [5, 2, 8, 1, 9];
numbers.sort((a, b) => a - b);
// [1, 2, 5, 8, 9]

// Sort strings
const names = ['Charlie', 'Alice', 'Bob'];
names.sort();
// ['Alice', 'Bob', 'Charlie']

// Sort objects
const users = [
  { name: 'Bob', age: 30 },
  { name: 'Alice', age: 25 },
  { name: 'Charlie', age: 35 }
];
users.sort((a, b) => a.name.localeCompare(b.name));
// Sorted by name

// Sort by multiple criteria
users.sort((a, b) => {
  const ageDiff = a.age - b.age;
  if (ageDiff !== 0) return ageDiff;
  return a.name.localeCompare(b.name);
});
```

### flat and flatMap

```javascript
// Flatten array
const nested = [1, [2, 3], [4, [5, 6]]];
const flat = nested.flat();
// [1, 2, 3, 4, [5, 6]]

const deepFlat = nested.flat(Infinity);
// [1, 2, 3, 4, 5, 6]

// flatMap: map + flatten
const sentences = ['Hello World', 'Foo Bar'];
const words = sentences.flatMap(s => s.split(' '));
// ['Hello', 'World', 'Foo', 'Bar']
```

### at

```javascript
const arr = [1, 2, 3, 4, 5];

// Get by index
arr.at(0);  // 1
arr.at(-1); // 5 (last element)

// With strings
const str = 'Hello';
str.at(0);  // 'H'
str.at(-1); // 'o'
```

---

## Complete Guide: JavaScript Object Methods Deep Dive

### Object.keys, values, entries

```javascript
const user = { name: 'Alice', age: 25, email: 'alice@example.com' };

// Get keys
Object.keys(user);
// ['name', 'age', 'email']

// Get values
Object.values(user);
// ['Alice', 25, 'alice@example.com']

// Get entries
Object.entries(user);
// [['name', 'Alice'], ['age', 25], ['email', 'alice@example.com']]

// Iterate
for (const [key, value] of Object.entries(user)) {
  console.log(`${key}: ${value}`);
}
```

### Object.assign

```javascript
// Merge objects
const defaults = { theme: 'dark', lang: 'en' };
const userPrefs = { theme: 'light' };

const merged = Object.assign({}, defaults, userPrefs);
// { theme: 'light', lang: 'en' }

// Shorthand
const merged2 = { ...defaults, ...userPrefs };
```

### Object.fromEntries

```javascript
// Convert entries to object
const entries = [['name', 'Alice'], ['age', 25]];
const obj = Object.fromEntries(entries);
// { name: 'Alice', age: 25 }

// Useful with Map
const map = new Map([['a', 1], ['b', 2]]);
const obj2 = Object.fromEntries(map);
// { a: 1, b: 2 }
```

### Object.freeze and seal

```javascript
// Freeze: no modifications
const config = Object.freeze({
  apiUrl: 'https://api.example.com',
  timeout: 5000
});

config.apiUrl = 'https://other.com'; // Silently fails (or throws in strict mode)

// Seal: no new properties, but can modify existing
const settings = Object.seal({
  theme: 'dark',
  fontSize: 14
});

settings.theme = 'light'; // Works
settings.newProp = 'test'; // Silently fails
```

### Optional chaining and nullish coalescing

```javascript
// Optional chaining (?.)
const user = {
  name: 'Alice',
  address: {
    city: 'Wonderland'
  }
};

user.address?.city;     // 'Wonderland'
user.address?.zip;      // undefined
user.contact?.email;    // undefined (no error)

// Nullish coalescing (??)
const value = null ?? 'default';  // 'default'
const value2 = 0 ?? 'default';    // 0 (only null/undefined)
const value3 = '' ?? 'default';   // '' (only null/undefined)

// Combined
const city = user.address?.city ?? 'Unknown';
```

---

## Complete Guide: CSS Flexbox Deep Dive

### Flex Container Properties

```css
.container {
  display: flex;
  
  /* Direction */
  flex-direction: row;           /* Default: horizontal */
  flex-direction: row-reverse;   /* Horizontal, reversed */
  flex-direction: column;        /* Vertical */
  flex-direction: column-reverse;/* Vertical, reversed */
  
  /* Wrapping */
  flex-wrap: nowrap;             /* Default: no wrap */
  flex-wrap: wrap;               /* Wrap to next line */
  flex-wrap: wrap-reverse;       /* Wrap upwards */
  
  /* Justify content (main axis) */
  justify-content: flex-start;   /* Default */
  justify-content: flex-end;
  justify-content: center;
  justify-content: space-between;
  justify-content: space-around;
  justify-content: space-evenly;
  
  /* Align items (cross axis) */
  align-items: stretch;          /* Default */
  align-items: flex-start;
  align-items: flex-end;
  align-items: center;
  align-items: baseline;
  
  /* Align content (multiple lines) */
  align-content: flex-start;
  align-content: flex-end;
  align-content: center;
  align-content: space-between;
  align-content: space-around;
  align-content: stretch;
  
  /* Gap */
  gap: 1rem;
  row-gap: 1rem;
  column-gap: 1rem;
}
```

### Flex Item Properties

```css
.item {
  /* Grow factor */
  flex-grow: 0;    /* Default: don't grow */
  flex-grow: 1;    /* Grow to fill space */
  
  /* Shrink factor */
  flex-shrink: 1;  /* Default: can shrink */
  flex-shrink: 0;  /* Don't shrink */
  
  /* Basis */
  flex-basis: auto;      /* Default: based on content */
  flex-basis: 200px;     /* Fixed width */
  flex-basis: 50%;       /* Percentage */
  
  /* Shorthand */
  flex: 0 1 auto;        /* Default */
  flex: 1;               /* grow: 1, shrink: 1, basis: 0% */
  flex: 0 0 200px;       /* Fixed width */
  flex: 1 1 0;           /* Equal width */
  
  /* Align self */
  align-self: auto;
  align-self: flex-start;
  align-self: flex-end;
  align-self: center;
  align-self: stretch;
  
  /* Order */
  order: 0;    /* Default */
  order: -1;   /* First */
  order: 1;    /* Last */
}
```

### Common Flex Patterns

```css
/* Centering */
.center {
  display: flex;
  justify-content: center;
  align-items: center;
}

/* Space between */
.between {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

/* Equal columns */
.equal {
  display: flex;
  gap: 1rem;
}
.equal > * {
  flex: 1;
}

/* Sidebar layout */
.layout {
  display: flex;
  gap: 1.5rem;
}
.sidebar {
  flex: 0 0 250px;
}
.content {
  flex: 1;
}

/* Sticky footer */
body {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}
main {
  flex: 1;
}

/* Wrap layout */
.wrap {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
}
.wrap > * {
  flex: 1 1 250px;
}
```

---

## Complete Guide: CSS Grid Deep Dive

### Grid Container Properties

```css
.container {
  display: grid;
  
  /* Define columns */
  grid-template-columns: 200px 1fr 200px;
  grid-template-columns: repeat(3, 1fr);
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  
  /* Define rows */
  grid-template-rows: auto 1fr auto;
  grid-template-rows: repeat(3, minmax(100px, auto));
  
  /* Gap */
  gap: 1rem;
  row-gap: 1rem;
  column-gap: 1rem;
  
  /* Named areas */
  grid-template-areas:
    "header header"
    "sidebar main"
    "footer footer";
  
  /* Justify items */
  justify-items: stretch;
  justify-items: start;
  justify-items: end;
  justify-items: center;
  
  /* Align items */
  align-items: stretch;
  align-items: start;
  align-items: end;
  align-items: center;
  
  /* Justify content */
  justify-content: start;
  justify-content: end;
  justify-content: center;
  justify-content: space-between;
  justify-content: space-around;
  justify-content: space-evenly;
  
  /* Align content */
  align-content: start;
  align-content: end;
  align-content: center;
  align-content: space-between;
  align-content: space-around;
  align-content: space-evenly;
  
  /* Auto flow */
  grid-auto-flow: row;
  grid-auto-flow: column;
  grid-auto-flow: dense;
  
  /* Auto sizes */
  grid-auto-rows: minmax(100px, auto);
  grid-auto-columns: minmax(200px, auto);
}
```

### Grid Item Properties

```css
.item {
  /* Column spanning */
  grid-column: 1 / 3;        /* Start at line 1, end at line 3 */
  grid-column: span 2;       /* Span 2 columns */
  grid-column: 1 / -1;       /* Full width */
  
  /* Row spanning */
  grid-row: 1 / 3;
  grid-row: span 2;
  
  /* Named area */
  grid-area: header;
  
  /* Align self */
  justify-self: start;
  justify-self: end;
  justify-self: center;
  justify-self: stretch;
  
  align-self: start;
  align-self: end;
  align-self: center;
  align-self: stretch;
}
```

### Common Grid Patterns

```css
/* Holy grail */
.layout {
  display: grid;
  grid-template-areas:
    "header header header"
    "nav main aside"
    "footer footer footer";
  grid-template-columns: 200px 1fr 200px;
  grid-template-rows: auto 1fr auto;
  min-height: 100vh;
}

/* Responsive card grid */
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 1rem;
}

/* Dashboard */
.dashboard {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  grid-template-rows: auto 1fr;
  gap: 1rem;
}

/* Spanning */
.full-width { grid-column: 1 / -1; }
.half-width { grid-column: span 2; }
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
- **Expansion Materials:** ~58,000 words
- **Total:** ~211,840 words

### Complete Topic Coverage

**57 Chapters across 15 Parts plus Appendices**
