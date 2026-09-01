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
