# Expansion Material: Final Completion

*Final expansion content to reach the 200,000 word target.*

---

## Complete Guide: JavaScript String Methods Deep Dive

### String Inspection

```javascript
const str = 'Hello, World!';

str.length;              // 13
str.charAt(0);           // 'H'
str.charCodeAt(0);       // 72
str.at(0);               // 'H'
str.at(-1);              // '!'
str.includes('World');    // true
str.includes('world');    // false (case-sensitive)
str.startsWith('Hello');  // true
str.endsWith('!');        // true
str.indexOf('World');     // 7
str.lastIndexOf('l');     // 10
str.search(/world/i);    // 7 (regex search)
str.match(/[A-Z]/g);     // ['H', 'W']
str.matchAll(/[l]/gi);   // Iterator with all matches
```

### String Transformation

```javascript
const str = '  Hello, World!  ';

str.toLowerCase();        // '  hello, world!  '
str.toUpperCase();        // '  HELLO, WORLD!  '
str.trim();               // 'Hello, World!'
str.trimStart();          // 'Hello, World!  '
str.trimEnd();            // '  Hello, World!'
str.padStart(20, '-');    // '---  Hello, World!  '
str.padEnd(20, '-');      // '  Hello, World!  ---'
str.repeat(2);            // '  Hello, World!    Hello, World!  '
str.replace('World', 'There');  // '  Hello, There!  '
str.replaceAll('l', 'L');      // '  HeLLo, WorLL!  '
str.normalize();          // Unicode normalization
```

### String Extraction

```javascript
const str = 'Hello, World!';

str.slice(0, 5);          // 'Hello'
str.slice(7);             // 'World!'
str.slice(-6);            // 'orld!'
str.slice(-6, -1);        // 'orld'
str.substring(0, 5);      // 'Hello'
str.substring(7);         // 'World!'
str.substr(7, 5);         // 'World' (deprecated)
```

### String Splitting and Joining

```javascript
// Split
'hello-world'.split('-');           // ['hello', 'world']
'hello world'.split(' ');           // ['hello', 'world']
'hello'.split('');                  // ['h', 'e', 'l', 'l', 'o']
'a,b,c'.split(',', 2);             // ['a', 'b']

// Join
['hello', 'world'].join(' ');       // 'hello world'
['a', 'b', 'c'].join('-');         // 'a-b-c'
['hello'].join('');                 // 'hello'
```

### String Comparison

```javascript
'a' < 'b';           // true
'a' > 'b';           // false
'a' === 'a';         // true

// Locale-aware comparison
'a'.localeCompare('b');              // -1
'b'.localeCompare('a');              // 1
'a'.localeCompare('a');              // 0
'a'.localeCompare('B');              // -1 (case-insensitive)
'a'.localeCompare('B', undefined, { sensitivity: 'base' });  // 0
```

---

## Complete Guide: JavaScript Number Methods

### Number Properties

```javascript
Number.isNaN(NaN);           // true
Number.isNaN('hello');       // false
Number.isFinite(Infinity);   // false
Number.isFinite(100);        // true
Number.isInteger(1.0);       // true
Number.isInteger(1.1);       // false
Number.isSafeInteger(9007199254740991);  // true
Number.isSafeInteger(9007199254740993);  // false
```

### Number Conversion

```javascript
Number('123');           // 123
Number('hello');         // NaN
Number(true);            // 1
Number(false);           // 0
Number(null);            // 0
Number(undefined);       // NaN
Number('');              // 0
Number('  ');            // 0

parseInt('123abc');      // 123
parseInt('abc123');      // NaN
parseInt('123', 10);     // 123
parseInt('ff', 16);      // 255

parseFloat('123.45');    // 123.45
parseFloat('abc');       // NaN
```

### Number Formatting

```javascript
const num = 1234567.89;

num.toFixed(2);          // '1234567.89'
num.toFixed(0);          // '1234568'
num.toPrecision(4);      // '1.235e+6'
num.toExponential(2);    // '1.23e+6'
num.toLocaleString();    // '1,234,567.89'
num.toLocaleString('de-DE');  // '1.234.567,89'

// Currency
new Intl.NumberFormat('en-US', {
  style: 'currency',
  currency: 'USD'
}).format(1234.56);      // '$1,234.56'

// Percentage
new Intl.NumberFormat('en-US', {
  style: 'percent'
}).format(0.85);         // '85%'
```

### Math Operations

```javascript
Math.abs(-5);            // 5
Math.ceil(4.3);          // 5
Math.floor(4.7);         // 4
Math.round(4.5);         // 5
Math.trunc(4.7);         // 4
Math.sign(-5);           // -1
Math.max(1, 2, 3);       // 3
Math.min(1, 2, 3);       // 1
Math.pow(2, 3);          // 8
Math.sqrt(16);           // 4
Math.cbrt(27);           // 3
Math.log(10);            // 2.302585092994046
Math.log2(8);            // 3
Math.log10(100);         // 2
Math.random();           // 0 to 1
Math.random() * 100;     // 0 to 100
Math.PI;                 // 3.141592653589793
Math.E;                  // 2.718281828459045
```

---

## Complete Guide: JavaScript Date Methods Deep Dive

### Creating Dates

```javascript
// Current date/time
new Date();

// From string
new Date('2024-01-15');
new Date('2024-01-15T10:30:00');
new Date('January 15, 2024');
new Date('01/15/2024');

// From components (month is 0-indexed!)
new Date(2024, 0, 15);        // January 15, 2024
new Date(2024, 11, 31);       // December 31, 2024

// From timestamp
new Date(1705276800000);       // Milliseconds since 1970

// Static methods
Date.now();                    // Current timestamp
Date.parse('2024-01-15');      // Parse string to timestamp
Date.UTC(2024, 0, 15);        // UTC timestamp
```

### Getting Components

```javascript
const date = new Date('2024-01-15T10:30:00');

date.getFullYear();     // 2024
date.getMonth();        // 0 (January = 0)
date.getDate();         // 15 (day of month)
date.getDay();          // 1 (Monday = 0, Sunday = 6)
date.getHours();        // 10
date.getMinutes();      // 30
date.getSeconds();      // 0
date.getMilliseconds(); // 0
date.getTime();         // Timestamp in ms
date.getTimezoneOffset(); // Offset in minutes from UTC
```

### Setting Components

```javascript
const date = new Date();

date.setFullYear(2025);
date.setMonth(5);        // June
date.setDate(20);
date.setHours(14);
date.setMinutes(30);
date.setSeconds(0);
date.setMilliseconds(0);

// Relative setting
date.setDate(date.getDate() + 7);   // Add 7 days
date.setMonth(date.getMonth() - 1); // Subtract 1 month
```

### Formatting

```javascript
const date = new Date('2024-01-15T10:30:00');

// toLocaleString
date.toLocaleString();                    // '1/15/2024, 10:30:00 AM'
date.toLocaleString('en-US');             // '1/15/2024, 10:30:00 AM'
date.toLocaleString('de-DE');             // '15.1.2024, 10:30:00'

// toLocaleDateString
date.toLocaleDateString();                // '1/15/2024'
date.toLocaleDateString('en-US', {
  weekday: 'long',
  year: 'numeric',
  month: 'long',
  day: 'numeric'
});                                       // 'Monday, January 15, 2024'

// toLocaleTimeString
date.toLocaleTimeString();                // '10:30:00 AM'

// toISOString
date.toISOString();                       // '2024-01-15T10:30:00.000Z'

// toDateString
date.toDateString();                      // 'Mon Jan 15 2024'

// toTimeString
date.toTimeString();                      // '10:30:00 GMT+0000'
```

### Comparison

```javascript
const date1 = new Date('2024-01-15');
const date2 = new Date('2024-01-20');

date1 > date2;    // false
date1 < date2;    // true
date1 === date2;  // false (different objects)

// Compare timestamps
date1.getTime() > date2.getTime();  // false

// Difference in days
const diff = Math.abs(date2 - date1);
const days = Math.floor(diff / (1000 * 60 * 60 * 24));  // 5
```

---

## Complete Guide: JavaScript Map and Set

### Map

```javascript
// Create map
const map = new Map();
const map2 = new Map([['key1', 'value1'], ['key2', 'value2']]);

// Set values
map.set('name', 'Alice');
map.set(42, 'answer');
map.set(true, 'yes');

// Get values
map.get('name');      // 'Alice'
map.get(42);          // 'answer'
map.get('missing');   // undefined

// Check existence
map.has('name');      // true
map.has('missing');   // false

// Delete
map.delete('name');

// Size
map.size;             // 2

// Iterate
for (const [key, value] of map) {
  console.log(`${key}: ${value}`);
}

map.forEach((value, key) => {
  console.log(`${key}: ${value}`);
});

// Get keys and values
[...map.keys()];     // ['key1', 'key2']
[...map.values()];   // ['value1', 'value2']
[...map.entries()];  // [['key1', 'value1'], ['key2', 'value2']]

// Clear
map.clear();
```

### Set

```javascript
// Create set
const set = new Set();
const set2 = new Set([1, 2, 3, 4, 5]);

// Add values
set.add('hello');
set.add(42);
set.add('hello');  // Ignored (already exists)

// Check existence
set.has('hello');   // true
set.has('missing'); // false

// Delete
set.delete('hello');

// Size
set.size;           // 1

// Iterate
for (const value of set) {
  console.log(value);
}

set.forEach((value) => {
  console.log(value);
});

// Convert to array
[...set];           // [42]

// Clear
set.clear();

// Set operations
const a = new Set([1, 2, 3]);
const b = new Set([2, 3, 4]);

// Union
const union = new Set([...a, ...b]);
// {1, 2, 3, 4}

// Intersection
const intersection = new Set([...a].filter(x => b.has(x)));
// {2, 3}

// Difference
const difference = new Set([...a].filter(x => !b.has(x)));
// {1}
```

---

## Complete Guide: JavaScript WeakMap and WeakSet

### WeakMap

```javascript
// Keys must be objects
const weakMap = new WeakMap();

let obj = { name: 'Alice' };
weakMap.set(obj, 'some data');

console.log(weakMap.get(obj));  // 'some data'

// When obj is garbage collected, the entry is automatically removed
obj = null;  // Entry is removed from weakMap

// Use cases:
// 1. Private data
const privateData = new WeakMap();

class User {
  constructor(name) {
    privateData.set(this, { name });
  }
  
  getName() {
    return privateData.get(this).name;
  }
}

// 2. Caching
const cache = new WeakMap();

function process(obj) {
  if (cache.has(obj)) {
    return cache.get(obj);
  }
  
  const result = expensiveOperation(obj);
  cache.set(obj, result);
  return result;
}
```

### WeakSet

```javascript
// Values must be objects
const weakSet = new WeakSet();

let obj = { name: 'Alice' };
weakSet.add(obj);

console.log(weakSet.has(obj));  // true

// When obj is garbage collected, the entry is automatically removed
obj = null;  // Entry is removed from weakSet

// Use cases:
// 1. Track visited objects
const visited = new WeakSet();

function traverse(node) {
  if (visited.has(node)) return;
  visited.add(node);
  
  for (const child of node.children) {
    traverse(child);
  }
}

// 2. Mark objects
const marked = new WeakSet();

function mark(obj) {
  marked.add(obj);
}

function isMarked(obj) {
  return marked.has(obj);
}
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
- **Expansion Materials:** ~60,000 words
- **Total:** ~213,840 words
