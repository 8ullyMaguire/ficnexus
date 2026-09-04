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
