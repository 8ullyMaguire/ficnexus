# Part 5 — Page-by-Page Restyle

Welcome to the final stretch! 🎉 In this part, you'll tour every single page in the FicHub archive that received a restyle treatment. By Chapter 4 you learned how individual components look when used in isolation. Now it's time to see them assembled into real pages — landing pages, tag directories, bookmark managers, forums, settings, and more. Each page takes those building pieces and arranges them into something a human actually uses every day.

Think of a page like a house built from bricks. The components are your bricks — walls, windows, doors — but only by putting them together in the right order do they become livable space. That's what these six chapters will show you: the full room layouts.

---

## Chapter 19: ArchiveHome — The Landing Page

You've just opened FicHub. Before you clicks anywhere, you land on the home page — the digital front door of the entire archive. This is `ArchiveHome.svelte`, a 470-line component that serves as the primary navigation hub for everyone who isn't searching yet.

### The Compact Download Input

Right at the top sits a deceptively simple input box. This is the **compact download field** — a quick way to paste a FicHub work ID or URL and jump straight to reading or downloading. Instead of using a large hero banner, the designers kept it compact so there's more screen real estate for actual content below.

```svelte
<!-- ArchiveHome.svelte — compact download input -->
<section class="archive-section">
  <input type="text" placeholder="Enter work ID to download..." />
  <ArchiveButton on:click={handleDownload}>Download</ArchiveButton>
</section>
```

The `ArchiveButton` here wraps around a standard HTML `<button>` but carries the Archive CSS styling automatically — no extra classes needed. Polymorphic components like this save you from having to remember which button class to apply on every page.

### Two Columns: Recent Works Meets Trending Now

Below the download input, the layout splits into two columns. On the left, the main column hosts **Recent Works** — stories that have been published recently, shown as WorkBlurb cards. On the right sidebar sits **Trending Now**, a curated list of popular stories.

```svelte
<!-- ArchiveHome.svelte — two-column layout -->
<div class="archive-grid">
  <main class="archive-main">
    <h2>Recent Works</h2>
    {#each recentWorks as work}
      <WorkBlurb {work} />
    {/each}
  </main>
  <aside class="archive-sidebar">
    <h2>Trending Now</h2>
    {#each trending as work}
      <WorkBlurb {work} />
    {/each}
  </aside>
</div>
```

The `archive-grid` class sets up a classic two-column layout using CSS Grid. The main area takes up the majority of the width while the sidebar occupies a fixed column — similar to how AO3 structures its home page. If you inspect the CSS, you'll find something like:

```css
.archive-grid {
  display: grid;
  grid-template-columns: 1fr 300px;
  gap: var(--archive-gap);
}
```

This is a responsive design too — when the viewport narrows, the grid collapses into a single column and the sidebar moves below the main content. No media queries magic required if the base template already handles it; Svelte's declarative structure makes reordering straightforward.

### Personalized Recommendations (For Logged-In Users Only)

Here's where things get clever. When a user is logged in *and* has set enough preferences in their profile data, a third section appears: **Personalized Recommendations**. This is pulled from the `/api/recs/personalized` endpoint. But critically — it only shows if both conditions are true:

```svelte
<!-- ArchiveHome.svelte — conditional personalized section -->
{#if $authStore.isLoggedIn && $prefsStore.hasEnoughData}
  <section class="archive-section">
    <h2>Recommended For You</h2>
    {#each recs as work}
      <WorkBlurb {work} />
    {/each}
  </section>
{/if}
```

This pattern — checking a store variable for login state combined with a property check on user prefs — is the canonical way to gate any feature that requires personalization across the entire app. If the conditions aren't met, the section simply doesn't render. No flicker, no blank card, nothing wasted.

### Parallel Data Loading

Fetching three separate sections (recent works, trending, personalized recs) could easily slow down the page load. If you fetched them one after another sequentially, the total wait time would be sum of all three requests. Instead, the page fires all three simultaneously using `Promise.allSettled`:

```typescript
// ArchiveHome.svelte — parallel data fetching
const [recentRes, trendingRes, recsRes] = await Promise.allSettled([
  fetch("/api/recent"),
  fetch("/api/trending"),
  isLoggedIn && prefs.hasEnoughData ? fetch("/api/recs/personalized") : Promise.resolve(null),
]);
```

Why `allSettled` instead of the more common `all`? Because `Promise.allSettled` doesn't abort the other requests if one fails. Imagine the personalized recs endpoint times out — with `Promise.all`, both "Recent Works" and "Trending Now" would fail too. With `allSettled`, they still load successfully and the user sees partial content rather than a blank page. The code checks each promise's `status` field (`"fulfilled"` vs `"rejected"`) and renders whichever data came back:

```typescript
const recentWorks = recentRes.status === "fulfilled" ? recentRes.value.data : [];
const trending = trendingRes.status === "fulfilled" ? trendingRes.value.data : [];
const recs = recsRes.status === "fulfilled" ? recsRes.value.data : [];
```

This defensive approach means your users always see *something*, even when an API misbehaves.

### Skeleton States: Shimmer While Waiting

During the loading period before any data arrives, the page doesn't sit blank. Each section independently renders skeleton placeholders — rows of shimmering gray bars that mimic the shape of WorkBlurbs. These use CSS animations driven by `@keyframes pulse`:

```svelte
<!-- ArchiveHome.svelte — shimmer skeleton loader -->
{#if loading}
  <div class="skeleton-row">
    <div class="shimmer"></div>
    <div class="shimmer short"></div>
  </div>
  <div class="skeleton-row">
    <div class="shimmer"></div>
    <div class="shimmer short"></div>
  </div>
  <!-- ... repeated for expected number of items ... -->
{/if}
```

Each skeleton row looks roughly like a WorkBlurb — a wider bar for the title, a shorter bar for metadata. The shimmer effect is pure CSS:

```css
@keyframes shimmer-pulse {
  0% { opacity: 0.4; }
  50% { opacity: 0.7; }
  100% { opacity: 0.4; }
}
.shimmer {
  background: #e0e0e0;
  animation: shimmer-pulse 1.5s ease-in-out infinite;
  border-radius: var(--archive-radius);
}
```

The beauty of independent skeletons per section is that if "Trending Now" finishes first, it drops in immediately while Recent Works still shimmers. Your eyes catch the updates one section at a time — much less jarring than a single spinner that covers everything at once.

> **🧪 Try It Yourself**
>
> Open the FicHub home page and keep the Network tab open in your browser dev tools. Watch how three XHR requests fire simultaneously. Notice how the response times differ — some APIs respond faster than others. The skeleton UI disappears independently for each section, exactly as described above.

---

## Chapter 20: Tags Page — A Brand New Route

If ArchiveHome existed before Phase 2, the Tags page did not. It was created fresh as part of the refactoring effort. Gone was the idea that browsing tags required knowing specific search URLs. Here comes `/tags/+page.svelte` — a directory of every tag in the system, organized neatly by category.

### Fetching All Six Tag Types in Parallel

Every tag on FicHub belongs to one of six categories. The page loads them all at once:

```typescript
// /tags/+page.svelte — fetch all tag types
const [fandoms, characters, relationships, additionalTags, warnings, categories] =
  await Promise.all([
    fetch("/api/tags?type=1").then(r => r.json()),
    fetch("/api/tags?type=2").then(r => r.json()),
    fetch("/api/tags?type=3").then(r => r.json()),
    fetch("/api/tags?type=4").then(r => r.json()),
    fetch("/api/tags?type=5").then(r => r.json()),
    fetch("/api/tags?type=6").then(r => r.json()),
  ]);
```

The API module lives at `frontend/src/lib/api/tags.ts`, a dedicated module for anything tag-related. Keeping it separate from the general API client keeps imports clean:

```typescript
// frontend/src/lib/api/tags.ts
export async function getTags(type: number) {
  const res = await fetch(`/api/tags?type=${type}`);
  if (!res.ok) throw new Error("Failed to fetch tags");
  return res.json();
}
```

### Organized by Category

Once all six arrays of tag data arrive, the template groups them under labeled headers. The organization mirrors AO3's tag directory style: a bold gray label followed by a vertical list of maroon links. The key insight here is using an array-of-object iterator — instead of writing six separate `{#each}` blocks (one for each tag type), you define a single loop that maps types to labels dynamically:

```svelte
<!-- /tags/+page.svelte — grouped tag rendering -->
{#each [
  { type: 1, label: "Fandoms", data: fandoms },
  { type: 2, label: "Characters", data: characters },
  { type: 3, label: "Relationships", data: relationships },
  { type: 4, label: "Additional Tags", data: additionalTags },
  { type: 5, label: "Warnings", data: warnings },
  { type: 6, label: "Categories", data: categories },
] as group}
  <section class="tag-group">
    <h2 class="tag-group-title">{group.label}</h2>
    <ul class="tag-list">
      {#each group.data as tag}
        <li>
          <a href="/search?include_tags={group.type}:{tag.name}" class="maroon-link">
            {tag.name} <span class="tag-count">({tag.usage_count})</span>
          </a>
        </li>
      {/each}
    </ul>
  </section>
{/each}
```

This approach has a nice side effect: if a new tag type gets added later (say, "Alternate Universe Tropes" becomes type 7), you only need to add one line to the array, not duplicate a whole section of HTML. Scalability through repetition avoidance — a principle worth remembering whenever your Svelte templates start looking like a copy-paste factory.

Each tag link includes the usage count in parentheses — that little `(247)` next to "Marvel" tells readers how many works carry that tag. The destination URL embeds the tag type as a prefix: `include_tags=1:Marvel Cinematic Universe`. The search system then filters exclusively within that tag category. This URL structure is intentional: it lets the search engine know exactly which taxonomy bucket to look in, avoiding ambiguity between a character named "Marvel" and a fandom called "Marvel."

#### Alphabetical Sorting

The backend returns tags sorted alphabetically within each category, but the frontend can reorder them if needed. For example, you might want to sort by popularity instead:

```svelte
<!-- Sort tags by usage count descending -->
{#each [...group.data].sort((a, b) => b.usage_count - a.usage_count) as tag}
```

Notice the spread operator `[...group.data]` creates a shallow copy before sorting. If you sorted `group.data` directly, you'd mutate the original data fetched from the API — Svelte's reactivity would fire unexpectedly, and sorting Fandoms could accidentally reorder Characters too. The spread ensures each group's data stays independent.

> **💡 Key Concept — Why Spread Before Sort?**
>
> JavaScript arrays are reference types. Calling `.sort()` on an existing array mutates it in place. By spreading into a new array first, you get a fresh array that `.sort()` modifies without touching the original. It's a tiny safety net that prevents subtle bugs where one section's sort order bleeds into another.

#### Scaling to Thousands of Tags

FicHub doesn't paginate its tag directory — every single tag renders on one page. This means the browser receives a large DOM tree at once. That's fine because:

1. Each tag row is lightweight HTML — just an `<a>` element inside an `<li>`, nothing heavy.
2. There's no image loading, no network calls per item, no JavaScript event handlers attached individually.
3. Readers who need to find a specific tag use their browser's native Find-in-Page (Ctrl+F / Cmd+F), which is instant against a fully-rendered document.

If FicHub ever grows to hundreds of thousands of tags, this pattern might need revisiting — pagination or virtual scrolling could enter the conversation. But for the current scale, the "render everything at once" approach is fast enough and much simpler to implement and maintain.

### Styling Details

The visual treatment reinforces the AO3 aesthetic without copying it pixel-for-pixel:

```css
.tag-group-title {
  color: #555;
  font-weight: 600;
  border-bottom: 1px solid var(--archive-border);
  padding-bottom: 0.25rem;
  margin-bottom: 0.5rem;
}

.maroon-link {
  color: var(--color-maroon);
  text-decoration: none;
}

.maroon-link:hover {
  text-decoration: underline;
}

.tag-count {
  color: #888;
  font-size: 0.85em;
}
```

The group header uses a medium-gray weight instead of the archive's traditional maroon red — a deliberate choice to create visual hierarchy. The eye scans from the category heading down through the alphabetically sorted tag links. And because there's no pagination, a tag-heavy site like FicHub benefits from the browser's native page-search (Ctrl+F) — the list is fully rendered in the DOM at once, making it instant to find what you need.

> **💡 Key Concept — Why Group Headers?**
>
> A flat list of thousands of tags would be overwhelming. By grouping into six categories, each section stays digestible. It also mirrors how authors think about tagging: "Oh, I need a *fandom* tag, a couple *character* tags, and maybe one *additional* tag." The interface follows the mental model.

---

## Chapter 21: Bookmarks, Authors, Notifications

Three pages that serve different user workflows, but all share the same archive conditional treatment. Let's walk through each one.

### Bookmarks — Saved Works, One Click Away

The bookmarks page lives at `/bookmarks/+page.svelte` and applies the archive styling conditionally. Under the hood, each bookmark maps to a `WorkBlurb` card — the exact same component used on search results and the home page:

```svelte
<!-- /bookmarks/+page.svelte — bookmark listing -->
{#each bookmarks as bookmark}
  <WorkBlurb work={bookmark.work} />
  {#if bookmark.notes}
    <p class="bookmark-notes">{bookmark.notes}</p>
  {/if}
{/each}
```

#### Graceful Handling of Deleted Works

Here's where the design shines. Sometimes authors delete their fics, or admins remove works for policy violations. What happens when a bookmarked work vanishes? The page doesn't crash or show a broken card. Instead, it falls back to a minimal card showing just the work ID, the reader's notes, and a "Remove" button:

```svelte
<!-- /bookmarks/+page.svelte — deleted work fallback -->
{#if bookmark.work}
  <WorkBlurb work={bookmark.work} />
{:else}
  <div class="deleted-work-card">
    <span>Work #{bookmark.workId}</span>
    {#if bookmark.notes}
      <p class="bookmark-notes">{bookmark.notes}</p>
    {/if}
    <ArchiveButton on:click={() => removeBookmark(bookmark.id)}>Remove</ArchiveButton>
  </div>
{/if}
```

This `deleted-work-card` div uses the same archive border and spacing tokens as regular blurbs, so visually it blends in seamlessly. The reader loses nothing beyond the work content itself. The Remove button lets them clean up their bookmark list without manual database edits.

#### Pagination and Scroll Behavior

For users who have saved hundreds of bookmarks, the API paginates results automatically. Each page fetch adds another batch of WorkBlurbs or deleted-work-cards to the DOM. The bookmark page typically loads 25 items per page — enough to feel like a substantial browsing session while keeping each HTTP response small. If you're curious how pagination works under the hood, the query parameter is just `page=N` appended to the `/api/bookmarks` endpoint call, passed through SvelteKit's standard URL search parameters.

> **💡 Key Concept — Bookmark Notes as Meta-Layers**
>
> Every bookmark can carry freeform text notes — the reader's own annotations about why they saved that story. "This fic saved me during finals week" or "Perfect for rereading before Christmas." Those notes render beneath each WorkBlurb when present, turning your bookmark list into something between a reading log and a personal diary. It's a small feature with outsized emotional value.

### Authors — People Search Made Simple

The `/authors/+page.svelte` page is wonderfully minimal: a plain People Search tool. An input field, a search button, and a result list. Despite its simplicity, it demonstrates a useful pattern — using the archive wrapper for a non-archive feature. The author search component lives entirely inside an archive-styled shell even though searching authors isn't specifically about fanfiction archives; it's a general people-discovery utility that happens to use the same visual language.

```svelte
<!-- /authors/+page.svelte — author search -->
<input
  type="text"
  bind:value={query}
  placeholder="Search authors..."
  class="archive-input"
/>
<ArchiveButton on:click={searchAuthors}>Search</ArchiveButton>

{#if loading}
  <div class="loading-skeleton">
    {#each [1, 2, 3] as _}
      <div class="shimmer" style="height: 1.5rem;"></div>
    {/each}
  </div>
{/if}

{#if query && !loading}
  <ul class="author-list">
    {#each results as author}
      <li>
        <a href="/authors/{author.id}" class="maroon-link">
          {author.display_name}
        </a>
      </li>
    {/each}
  </ul>
{/if}
```

Notice the `archive-input` class on the text field — a small touch that gives the search box the same bordered, spaced look as inputs in the search form. The loading skeleton mirrors the patterns we saw on ArchiveHome: three stacked shimmer bars during the fetch. This consistency matters because readers move fluidly between searching for stories and searching for authors. Their eyes expect the same visual rhythm regardless of which search bar they're typing into.

The API call that powers this is straightforward:

```typescript
async function searchAuthors() {
  loading = true;
  try {
    const res = await fetch(`/api/authors?q=${encodeURIComponent(query)}`);
    results = await res.json();
  } finally {
    loading = false;
  }
}
```

A subtle but important point: the `finally` block ensures `loading` always returns to `false`, even if the API throws an error. Without it, a failed request would leave the skeletons permanently displayed — a jarring "stuck loading" state that makes readers think the page has frozen. Defensive programming doesn't have to be complicated.

No archive-specific components are needed beyond the wrapper elements. The author list renders as plain links — the archiving cosmetics come from the shared styling classes applied to inputs, buttons, and skeletons.

> **🧪 Try It Yourself**
>
> Go to the Authors search page, clear the input box, then type an author name slowly character by character. Notice that each keystroke *doesn't* trigger a new search — only clicking the Search button does. This is intentional: the page avoids firing excessive network requests for every single typed character. The debounce approach (auto-searching on each keypress) looks cleaner but wastes bandwidth. Sometimes doing one thing at a time explicitly is the better design choice.

### Notifications — Flat, Clean Rows

Notifications take the opposite approach from Bookmarks. Instead of cards with depth, they use a flat list — simple rows displaying title, body preview, date, and a read/unread indicator. No `WorkBlurb`, no nested components, just clean HTML structure:

```svelte
<!-- /notifications/+page.svelte — notification listing -->
<div class="notification-header">
  <ArchiveButton on:click={markAllRead}>Mark All Read</ArchiveButton>
</div>

{#each notifications as notification}
  <div class={notification.read ? "notification-row read" : "notification-row unread"}>
    <h3>{notification.title}</h3>
    <p class="notification-body">{notification.bodyPreview}</p>
    <time datetime={notification.date}>{formatDate(notification.date)}</time>
  </div>
{/each}
```

The "Mark All Read" button sits at the very top in the `notification-header`. Every other element follows in a flat cascade. There's no sidebar, no filtering, no complexity. Just a chronological stream of activity.

The `notification-row` class applies the archive border styling:

```css
.notification-row {
  border: 1px solid var(--archive-border);
  padding: var(--archive-padding-sm);
  margin-bottom: var(--archive-gap);
}

.notification-row.unread {
  border-left: 3px solid var(--color-maroon);
  background: var(--archive-bg-hover);
}
```

Unread notifications get a distinctive left border in maroon and a slightly darker background — an instantly recognizable visual cue that doesn't require reading any text.

#### Marking Individual Notifications

Clicking on a notification row navigates to its detail view and marks it read. The state flips server-side, so the next page load shows the notification without the maroon left-border highlight. This gives readers a clear sense of progress through their notification backlog.

> **⚠️ Watch Out for Stale Data**
>
> If the user leaves notifications open in a tab while simultaneously clicking "Mark All Read" on another tab, the first tab won't automatically update until refreshed. To prevent this mismatch, some teams add a WebSocket subscription or a periodic poll that syncs notification states across tabs. FicHub currently uses a simple approach: the settings page can be toggled to force a refresh when returning to the notifications tab via the Visibility API (`document.addEventListener('visibilitychange', ...)`). It's not perfect but avoids the complexity of real-time subscriptions.

---

## Chapter 22: Requests & Forum

FicHub isn't just about reading stories — it's also about community. The Request system lets fans post wish-lists for specific story prompts, and the Forum provides threaded discussions. Both receive the full archive restyle treatment.

### Requests — Browse, Create, and Detail Views

The Requests system spans three distinct views, each with its own template:

#### List View (`/requests`)

The index page displays all open requests as rows with three key pieces of information: title, status badge, and upvote count.

```svelte
<!-- /requests/+page.svelte — request list -->
{#each requests as request}
  <div class="request-row">
    <a href="/requests/{request.id}" class="maroon-link">
      {request.title}
    </a>
    <span class="status-badge status-{request.status}">
      {request.status}
    </span>
    <span class="upvote-count">▲ {request.upvotes}</span>
  </div>
{/each}
```

Status badges use CSS classes prefixed with the status value, allowing color-coded appearance without JavaScript logic:

```css
.status-pending { background: #fff3cd; color: #856404; }
.status-completed { background: #d4edda; color: #155724; }
.status-rejected { background: #f8d7da; color: #721c24; }
```

#### Detail View (`/requests/:id`)

When viewing a single request, the content is wrapped in a `<fieldset>` element styled with archive conventions — the same pattern used in search forms and settings:

```svelte
<!-- /requests/[id]/+page.svelte — request detail -->
<fieldset class="archive-fieldset">
  <legend class="field-legend">{request.title}</legend>
  
  <dl class="detail-definition">
    <dt>Status</dt><dd>{request.status}</dd>
    <dt>Posted by</dt><dd>{request.author.display_name}</dd>
    <dt>Date</dt><dd>{formatDate(request.created_at)}</dd>
    <dt>Upvotes</dt><dd>{request.upvotes}</dd>
  </dl>
  
  <div class="request-description">{request.description}</div>
</fieldset>
```

The `<fieldset>` + `<legend>` combination is semantically meaningful — it says "this entire block is one form-like unit" — and the archive styles make it look beautiful with borders and indentation.

The `<dl>` (definition list) pattern is used throughout for key-value pairs. It's semantically cleaner than `<div>` spam and requires zero extra classes since the archive stylesheet targets `<dl>` inside `.archive-fieldset`.

#### New Request Form (`/requests/new`)

Creating a new request uses a `<dl>`-based input layout — rows of label/value pairs:

```svelte
<!-- /requests/new/+page.svelte — new request form -->
<form on:submit|preventDefault={submitRequest}>
  <fieldset class="archive-fieldset">
    <legend class="field-legend">Create a New Request</legend>
    
    <dl class="form-dl">
      <dt><label for="title">Title</label></dt>
      <dd><input id="title" name="title" bind:value={form.title} /></dd>
      
      <dt><label for="description">Description</label></dt>
      <dd><textarea id="description" name="description" bind:value={form.description}></textarea></dd>
      
      <dt><label for="prompt">Prompt Tags</label></dt>
      <dd>
        <input id="prompt" bind:value={form.promptTag} />
        <button type="button" on:click={addSeedChip}>Add</button>
        <div class="seed-chips">
          {#each form.seeds as chip}
            <span class="seed-chip">{chip}<button type="button" on:click={() => removeSeed(chip)}>×</button></span>
          {/each}
        </div>
      </dd>
    </dl>
    
    <ArchiveButton type="submit">Submit Request</ArchiveButton>
  </fieldset>
</form>
```

The **seed chips** deserve special attention. They're small pill-shaped labels representing tag suggestions that the user adds incrementally. Each chip shows its text and a small × button to remove it. Below the chips, the description renders as freeform text. This incremental-add pattern keeps the form tidy even when a user wants to attach ten tags to a single request.

### Forum — Tables and Bordered Boxes

The forum takes a different visual approach: bordered tables for lists, and bordered boxes for individual posts.

#### Category Index (`/forum/`)

The main forum page shows all discussion categories as a bordered table:

```svelte
<!-- /forum/+page.svelte — category index table -->
<table class="forum-table">
  <thead>
    <tr>
      <th>Title</th>
      <th>Topics</th>
      <th>Posts</th>
      <th>Last Post</th>
    </tr>
  </thead>
  <tbody>
    {#each categories as category}
      <tr>
        <td>
          <a href="/forum/{category.slug}" class="maroon-link">
            {category.name}
          </a>
        </td>
        <td>{topicCount(category.slug)}</td>
        <td>{postCount(category.slug)}</td>
        <td class="last-post-date">{formatDate(category.lastPostAt)}</td>
      </tr>
    {/each}
  </tbody>
</table>
```

The table uses archive styling — `--archive-border` on borders, compact row heights, and maroon-colored links. Crucially, the `border-radius` is set to `0` everywhere to maintain the sharp, utilitarian archive aesthetic:

```css
.forum-table {
  border-collapse: collapse;
  width: 100%;
}

.forum-table th, .forum-table td {
  border: 1px solid var(--archive-border);
  padding: var(--archive-padding-sm);
  border-radius: 0;
}

.forum-table th {
  background: var(--archive-bg-header);
  font-weight: 600;
}
```

#### Topic List (`/forum/:slug`)

Individual category topic lists use the same table pattern with slightly different columns — typically Topic | Author | Replies | Last Activity. Reusing the `.forum-table` class means zero duplicate CSS between these two pages.

#### Post Detail (`/forum/:slug/:id`)

Threaded posts break away from the table pattern. Each post is rendered as an individual boxed div with a gray header row inside:

```svelte
<!-- /forum/[slug]/[id]/+page.svelte — post rendering -->
{#each posts as post}
  <div class="forum-post-box">
    <div class="post-header">
      <span class="post-author">{post.author.display_name}</span>
      <time datetime={post.created_at}>{formatDate(post.created_at)}</time>
    </div>
    <div class="post-content">{post.content}</div>
  </div>
{/each}
```

```css
.forum-post-box {
  border: 1px solid var(--archive-border);
  margin-bottom: var(--archive-gap);
}

.post-header {
  background: var(--archive-bg-header);
  padding: var(--archive-padding-sm);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.post-author {
  color: var(--color-maroon);
  font-weight: 600;
}
```

The gray header row inside each box separates metadata (author + date) from content. The author name uses maroon text to tie back to the overall archive palette. Posts stack vertically without threading indentation — each box stands alone on equal footing.

#### New Topic Form (`/forum/new`)

The new topic form mirrors the Requests form pattern: a `<fieldset>` with `<dl>` rows for inputs:

```svelte
<form on:submit|preventDefault={submitTopic}>
  <fieldset class="archive-fieldset">
    <legend class="field-legend">Start a New Topic</legend>
    <dl class="form-dl">
      <dt><label for="title">Topic Title</label></dt>
      <dd><input id="title" bind:value={form.title} /></dd>
      <dt><label for="content">Body</label></dt>
      <dd><textarea id="content" bind:value={form.content} rows="10"></textarea></dd>
    </dl>
    <ArchiveButton type="submit">Create Topic</ArchiveButton>
  </fieldset>
</form>
```

The consistent use of `<fieldset>` + `<legend>` + `<dl>` across Requests and Forms means every community feature feels like it belongs to the same family, even though the underlying data and interactions differ dramatically.

---

## Chapter 23: Ask, Settings, Fic Detail

Three final pages that round out the archive experience — asking questions to the Archive AI, configuring preferences, and reading a full fic.

### Ask the Archive — Q&A with Style

The Ask page at `/ask/+page.svelte` is a simple question-and-answer interface. Users type a prompt into a large textarea, submit it, and receive results displayed as WorkBlurb cards below.

```svelte
<!-- /ask/+page.svelte — ask interface -->
<h1 class="ask-heading">Ask the Archive</h1>

<fieldset class="archive-fieldset">
  <legend class="field-legend">Your Question</legend>
  <textarea
    id="ask-input"
    bind:value={question}
    rows="5"
    placeholder="What kind of story are you looking for?"
  ></textarea>
  <ArchiveButton on:click={submitQuestion} disabled={!question.trim()}>
    Ask the Archive
  </ArchiveButton>
</fieldset>

{#if error}
  <div class="error-note">
    <small>⚠️ {error}</small>
  </div>
{/if}

{#if results.length > 0}
  <div class="ask-results">
    {#each results as work}
      <WorkBlurb {work} />
    {/each}
  </div>
{/if}
```

The textarea gets archive-style borders through shared input CSS. The Ask heading uses the archive serif style for consistency. Results appear below the form as the same `WorkBlurb` cards used everywhere else — reinforcing that search results and ask results are fundamentally the same data shape.

Errors render in AO3's footnote style — a small, understated note with a warning icon, placed near where the action failed:

```css
.error-note {
  border: 1px solid #f5c6cb;
  padding: var(--archive-padding-sm);
  margin-top: var(--archive-gap);
}

.error-note small {
  color: #721c24;
  font-size: 0.85em;
}
```

### Settings — Choosing Your Interface Flavor

The Settings page at `/settings/+page.svelte` is famously minimalist — a single section, a few radio buttons, and a Save button. But don't let its simplicity fool you: it controls the most fundamental toggle in the entire application.

```svelte
<!-- /settings/+page.svelte — interface style selector -->
<fieldset class="archive-fieldset">
  <legend class="field-legend">Interface Style</legend>
  <div class="radio-group">
    <label>
      <input
        type="radio"
        name="interfaceStyle"
        value="modern"
        bind:checked={$prefsStore.interfaceStyle === 'modern'}
      />
      Modern
    </label>
    <label>
      <input
        type="radio"
        name="interfaceStyle"
        value="archive"
        bind:checked={$prefsStore.interfaceStyle === 'archive'}
      />
      Archive
    </label>
  </div>
  <ArchiveButton on:click={saveSettings}>Save Preferences</ArchiveButton>
</fieldset>
```

Two important observations here:

1. The `<fieldset>` with `<legend>` creates a clear visual grouping for the setting options — matching the pattern used in search forms and other form-like sections.
2. **Emojis are completely absent.** Throughout Settings, the interface relies on text labels and the existing icon library. The designers made a deliberate choice to keep emojis off UI surfaces to maintain the clean archival aesthetic.

When the user clicks "Save," the preference propagates through `$prefsStore` and triggers an immediate re-render of all pages in archive or modern mode. No page refresh needed — the reactive stores handle it. This is one of those moments where you can feel the power of SvelteKit's reactive programming: change one variable, watch half the app update simultaneously.

> **💡 Key Concept — Interface Style as State**
>
> The interface style preference lives in two places simultaneously: the user's account on the server (so it persists across devices), and `$prefsStore` in the browser (so the current session reflects it instantly). When you switch from Modern to Archive, both layers update. If your internet drops right after clicking Save, you won't lose your preference — it's already on the server. The localStorage fallback ensures offline sessions remember their choice even if the server call fails.

### Fic Detail — Reading a Full Story

The fic detail page at `/fic/[urlId]/+page.svelte` is arguably the most important page in the archive. This is where readers spend hours absorbed in someone else's creativity. The page composes multiple archive components together:

```svelte
<!-- /fic/[urlId]/+page.svelte — full fic detail layout -->
<svelte:head>
  <title>{work.title} by {work.author.display_name} — FicHub</title>
</svelte:head>

<ArchiveWork {work} />

<div class="fic-meta">
  <div class="tag-table">
    {#each work.tags as tag}
      <span class="tag-pill tag-{tag.category}">
        <a href="/search?include_tags={tag.category}:{tag.name}">
          {tag.label}
        </a>
      </span>
    {/each}
  </div>
</div>

<div class="summary-block">
  <h3>Summary</h3>
  <blockquote>{work.summary}</blockquote>
</div>

<div class="action-buttons">
  <ArchiveButton on:click={kudosAction}>Leave Kudos</ArchiveButton>
  <ArchiveButton on:click={commentAction}>Comment</ArchiveButton>
  <ArchiveButton on:click={() => bookmark(work.id)}>Bookmark</ArchiveButton>
</div>

<div class="chapters-list">
  <h3>Chapters</h3>
  {#each work.chapters as chapter}
    <a href="#chapter-{chapter.index}" class="maroon-link">
      {chapter.title || `Chapter ${chapter.index}`}
    </a>
  {/each}
</div>

<div class="author-notes">
  <h3>Author Notes</h3>
  {#each work.notes as note}
    <div class="note-block">
      <small class="note-date">{formatDate(note.date)}</small>
      <p>{note.text}</p>
    </div>
  {/each}
</div>

<div class="download-section">
  <details>
    <summary>Download Options</summary>
    <ul>
      <li><a href="/fic/{work.urlId}/download/epub">EPUB</a></li>
      <li><a href="/fic/{work.urlId}/download/pdf">PDF</a></li>
      <li><a href="/fic/{work.urlId}/download/text">Plain Text</a></li>
    </ul>
  </details>
</div>
```

Several things happen on this page:

- **Tag Table** — Uses the `TagSoup` component with category-colored pills, identical to how tags render on search results. Clicking a tag navigates to a filtered search.
- **Summary Block** — Rendered as a `<blockquote>` with archive-styled borders, giving the summary visual separation from the rest of the page.
- **Action Buttons** — Three prominent `ArchiveButton` instances for leaving kudos, commenting, and bookmarking. All styled identically regardless of context.
- **Chapters List** — A simple numbered list of anchor links jumping to each chapter on the page. Multi-chapter fics benefit greatly from this internal navigation.
- **Author Notes** — Displayed chronologically at the bottom, each wrapped in a small `.note-block` with a timestamp. Readers often treat these as a second narrative thread.
- **Download Dropdown** — Uses the native HTML `<details>` element for a collapsible dropdown listing EPUB, PDF, and Plain Text formats. No custom accordion JS needed — the browser handles open/close natively.

The `ArchiveWork` component encapsulates the core fic rendering — the actual prose text formatted for comfortable reading, with proper typography and line height adjustments. Everything else wraps around it.

---

## Chapter 24: ArchiveListPage — The Shared List Component

If there's a MVP award for the most reusable component in the archive, it goes to `ArchiveListPage.svelte`. At 216 lines, this component does one thing perfectly: renders a list of items with polished loading, empty, and error states. Multiple pages reuse it, saving hundreds of lines of duplicated code.

### Props: The Contract

Every consumer passes the same four props plus an optional subtitle:

```svelte
<!-- ArchiveListPage.svelte — props definition -->
<script lang="ts">
  export let title: string = "";
  export let items: unknown[] = [];
  export let loading: boolean = false;
  export let errorMessage: string = "";
  export let subtitle: string = "";
  export let children: Snippet; // Renders item-level slot content
</script>
```

The `children` prop uses Svelte's snippet feature — a way to pass callable JSX-like content that gets rendered repeatedly for each item. This keeps `ArchiveListPage` generic while letting consumers define exactly how each item looks:

```svelte
<!-- Usage on the home page -->
<ArchiveListPage
  title="Recent Works"
  {items}
  {loading}
  {errorMessage}
>
  {@render children}
</ArchiveListPage>

<!-- Parent provides the item renderer -->
<ArchiveListPage title="Trending" {items} {loading} loadingLabel="Loading trending...">
  {#snippet children()}
    {#each items as work}
      <WorkBlurb {work} />
    {/each}
  {/snippet}
</ArchiveListPage>
```

### Loading Skeleton: Pulsing Blurbs

When `loading` is true, the component generates five skeleton blurbs with a pulsing animation:

```svelte
<!-- ArchiveListPage.svelte — loading skeleton -->
{#if loading}
  <div class="list-loading">
    {#each [1, 2, 3, 4, 5] as _}
      <div class="skeleton-blurb">
        <div class="skeleton-line title"></div>
        <div class="skeleton-line meta"></div>
        <div class="skeleton-line tags"></div>
      </div>
    {/each}
  </div>
{/if}
```

Each skeleton blurb contains three horizontal bars of varying widths, mimicking a WorkBlurb's layout: a long title bar, a medium-length metadata bar, and a short tag bar. The pulsing animation runs continuously:

```css
@keyframes pulse {
  0%, 100% { opacity: 0.3; }
  50% { opacity: 0.7; }
}

.skeleton-blurb {
  border: 1px solid var(--archive-border);
  padding: var(--archive-padding-sm);
  margin-bottom: var(--archive-gap);
}

.skeleton-line {
  background: var(--archive-skeleton);
  animation: pulse 1.5s ease-in-out infinite;
  border-radius: 2px;
  margin-bottom: 0.5rem;
}

.skeleton-line.title {
  height: 1rem;
  width: 80%;
}

.skeleton-line.meta {
  height: 0.75rem;
  width: 50%;
}

.skeleton-line.tags {
  height: 0.5rem;
  width: 60%;
}
```

Five skeleton blurbs match the typical default fetch size of results — enough to fill the visible viewport and give the impression of a "real" list coming in.

### Empty State: Nothing Found, Nothing Shown

When the list is genuinely empty (not loading — just zero results):

```svelte
<!-- ArchiveListPage.svelte — empty state -->
{:else if items.length === 0 && !loading}
  <div class="empty-state">
    <p>No results found.</p>
  </div>
{/if}
```

Notice the design decision: **no icon**. The empty state is deliberately minimal — just text saying "No results found." No emoji, no illustration, no decorative element. This restraint keeps the page focused and avoids the cluttered feeling that icons can bring to a sparse page.

### Error State: Red Border, Retry Button

When data fetching fails:

```svelte
<!-- ArchiveListPage.svelte — error state -->
{:else if errorMessage}
  <div class="error-box">
    <p>{errorMessage}</p>
    <ArchiveButton on:click={() => retry()}>Retry</ArchiveButton>
  </div>
{/if}
```

The error message appears in a red-bordered box with a Retry button underneath. The `retry()` function is provided by the consuming component (home page, search page, etc.), ensuring the retry behavior matches whatever context triggered the fetch.

### Who Uses ArchiveListPage?

Four major places depend on this shared component:

1. **Recent Works on the Home Page** — Shows freshly published stories with loading skeletons while the API resolves.
2. **Trending Sidebar** — Displays trending works in a compact sidebar container, sharing the same component for consistency.
3. **Search Results Page** — Renders paginated search results with the same loading/empty/error handling as everything else.
4. **Bookmark Listing** — Used on the Bookmarks page under the archive conditional to render saved works consistently.

By centralizing list presentation in one component, every page that shows a collection of items looks and behaves identically. When the design team decides tomorrow that skeleton animations should be slightly brighter, they change it in one place and every affected page updates automatically.

---

## Wrapping Up

We've covered six pages — the landing page, the tag directory, bookmarks/authors/notifications, requests/forum, ask/settings/detail, and the shared list component that ties them all together. Through it all, a few patterns repeat like a melody:

- **Archive containers** (`<fieldset>`, `.archive-section`, `.archive-fieldset`) create visual rhythm across pages.
- **Shared components** (`WorkBlurb`, `ArchiveButton`, `TagSoup`) ensure consistency without repetition.
- **Skeleton states** provide smooth transitions during loading.
- **Graceful degradation** handles missing data and API failures without breaking.

These patterns exist because someone decided early on that every page should feel like it belongs to the same family — whether it's a reader discovering their first bookmark or an author posting their last chapter.

That's all for Part 5. You now understand every route page in the FicHub archive frontend, from the grand landing page to the humblest notification row. The pieces fit together, and with that knowledge, you're ready to contribute. 🎓
