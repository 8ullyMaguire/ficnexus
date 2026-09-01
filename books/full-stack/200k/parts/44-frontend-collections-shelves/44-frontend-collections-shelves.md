# Part 44 — Frontend Collections and Shelves

So far our book has built the data model, the API, and the backend routes for user
organization features — reading lists, custom shelves, and reading status. This
part flips the camera around and looks at the **frontend**: the SvelteKit pages and
components that turn those APIs into something a person actually clicks.

We cover three pieces of the UI, in the order a user meets them:

1. The **Work Proposals** page — where curators vote on merge/split proposals.
2. The **Curator work-deletions** page — a protected queue for permanent deletes.
3. The **collection, shelf, and list editors** — the three surfaces users use every
   day to organize their reading.

> 💡 Kid-friendly framing: Think of the frontend as the *picture book* that sits on
> top of the *instruction manual* (the backend). Every pretty button and list you see
> here is just calling the same clean APIs we built in Parts 12 and 15. The magic is
> mostly **state management** — "how do I remember what's loading, and what do I show
> when it's done?" — plus a sprinkle of **archive mode** styling that makes FicHub
> look like a fanfiction library instead of a startup landing page.

---

## 44.1 `src/routes/work-proposals/+page.svelte` — Voting on work proposals

### Goal

Let curators browse pending merge/split proposals, see the vote tally, and cast a
`+1` / `0` / `-1` vote. The page wires the `listProposals` and `voteProposal` API
helpers (from `src/lib/api/social.ts`) to a Svelte 5 runes component.

### Actions

**Step 1 — Imports and reactive state.** The `<script>` block imports the API
helpers, the auth store, and the i18n helper, then reads a `uiMode` preference so
the same page can render AO3-style ("archive") or modern chrome:

```svelte
<!-- frontend/src/routes/work-proposals/+page.svelte (lines 1-18) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { listProposals, voteProposal } from '$lib/api/social';
  import type { WorkProposal } from '$lib/api/social-types';
  import { auth } from '$lib/stores/auth.svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let proposals = $state<WorkProposal[]>([]);
  let loading = $state(true);
  let error = $state('');
  let votingId = $state<number | null>(null);
  let voteMsg = $state('');

  // Voting is open to every signed-in account (trusted+ = level ≥ 1) —
  // mirrors the backend gate used by the proposal vote endpoint.
  const canVote = $derived(auth.level >= 1);
```

> 🔑 **Why `$derived` for `canVote`?** It depends on `auth.level`, which is a getter
> on the auth store (`frontend/src/lib/stores/auth.svelte.ts`). When auth loads in
> the background, `auth.level` goes from `0` to the real value, and `canVote` flips
> automatically. No `onAuthChange` callback needed.

**Step 2 — Fetch + error handling.** `load()` calls `listProposals()` (the API helper
that hits `GET /api/work-proposals`) and guards against both an API error code and a
network exception:

```svelte
// frontend/src/routes/work-proposals/+page.svelte (lines 19-35)
  async function load() {
    loading = true;
    error = '';
    try {
      const res = await listProposals();
      if (res.err === 0) {
        proposals = res.proposals ?? [];
      } else {
        error = 'Failed to load proposals';
      }
    } catch {
      error = 'Failed to load proposals';
    } finally {
      loading = false;
    }
  }
```

**Step 3 — Voting.** The handler disables itself while a vote is in flight
(`votingId !== null` on each button), then updates the proposal object in place from
the server's response:

```svelte
// frontend/src/routes/work-proposals/+page.svelte (lines 21-61)
  async function handleVote(p: WorkProposal, vote: -1 | 0 | 1) {
    if (!canVote || votingId !== null) return;
    votingId = p.id;
    voteMsg = '';
    try {
      const res = await voteProposal(p.id, vote);
      p.vote_sum = res.vote_sum;
      p.voter_count = res.voter_count;
      voteMsg = res.msg;
    } catch (e) {
      voteMsg = e instanceof Error ? e.message : 'Vote failed';
    } finally {
      votingId = null;
    }
  }

  function describe(p: WorkProposal): string {
    if (p.action_type === 'merge') {
      return `Merge work #${p.source_work_id ?? '?'} into work #${p.target_work_id ?? '?'}`;
    }
    if (p.action_type === 'split') {
      return `Split work #${p.work_id ?? '?'}`;
    }
    return p.action_type;
  }

  onMount(async () => {
    await auth.init();
    await load();
  });
```

The template renders two layouts behind a single `{#if uiMode === 'archive'}` check.
In archive mode, each proposal is a list item with type/status/vote chips and three
vote buttons:

```svelte
<!-- frontend/src/routes/work-proposals/+page.svelte (archive mode) -->
        {#each proposals as p (p.id)}
          <li class="arc-proposal">
            <div class="arc-proposal-main">
              <div class="arc-proposal-row">
                <span class="arc-chip">{p.action_type}</span>
                <span class="arc-chip">{p.status}</span>
                <span class="arc-chip arc-votes">+{p.vote_sum} / {p.voter_count} voter{p.voter_count === 1 ? '' : 's'}</span>
              </div>
              <strong class="arc-proposal-desc">{describe(p)}</strong>
              <span class="arc-muted">Proposed by user #{p.proposer_id} &middot; {new Date(p.created_at).toLocaleDateString()}</span>
              {#if p.details}
                <pre class="arc-details">{JSON.stringify(p.details, null, 2)}</pre>
              {/if}
            </div>

            {#if canVote}
              <div class="arc-vote-group">
                <button class="archive-btn" type="button" onclick={() => handleVote(p, 1)} disabled={votingId !== null}>▲ +1</button>
                <button class="archive-btn" type="button" onclick={() => handleVote(p, 0)} disabled={votingId !== null}>— 0</button>
                <button class="archive-btn" type="button" onclick={() => handleVote(p, -1)} disabled={votingId !== null}>▼ −1</button>
              </div>
            {/if}
          </li>
        {/each}
```

### Check

- Open `/work-proposals` while signed in as a curator (level ≥ 1). You should see a
  list of pending proposals, each showing its `action_type`, `status`, and the
  `+vote_sum / voter_count` tally.
- Click **▲ +1** on a proposal. The tally should update immediately from the
  `voteProposal` response (`res.vote_sum`, `res.voter_count`).
- While signed out, the vote buttons are hidden and a notice explains curators only.

### Troubleshooting

- **Votes don't update:** Open devtools → Network, click a vote, and confirm the
  `POST /api/work-proposals/{id}/vote` request fires with `{"vote":1}`. A 401 means
  your token is stale — the auth store's `init()` should redirect you to `/login`
  (the `request()` helper in `social.ts` auto-clears the token on 401).
- **Blank page / "Loading proposals…" forever:** `listProposals()` hits
  `/api/work-proposals` (plural). Check the route is registered server-side and that
  the `WorkProposal` type in `social-types.ts` matches the `details` field shape.
- **Curator can't vote:** `canVote = auth.level >= 1`. The `onMount` awaits
  `auth.init()` before `load()`, so a stale `level=0` usually means
  `getMe()` failed on the server.

---

## 44.2 `src/routes/curator/work-deletions/+page.svelte` — The deletion request queue

### Goal

Show only curators (level ≥ 100 / admin) a table of pending work-deletion requests,
with **Approve** and **Reject** buttons. Approving permanently deletes a work, so it
requires a `confirm()` dialog. This component lives at `/curator/work-deletions` and
is guarded by a level check.

### Actions

**Step 1 — Route guard + `adminFetch`.** Unlike the proposals page (readable by
anyone), the deletions queue is admin-only. The `onMount` hook checks
`auth.level >= 100` and redirects to `/` if too low-level:

```svelte
// frontend/src/routes/curator/work-deletions/+page.svelte (lines 1-22)
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';
  import { adminFetch } from '$lib/api/admin';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';
  import { relativeTime } from '$lib/util';

  const uiMode = $derived(getPref('uiMode'));
  const isAdmin = $derived(auth.level >= 100);

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn || auth.level < 100) {
      goto('/');
      return;
    }
    checking = false;
    await load();
  });
```

> ⚠️ **Why `adminFetch`?** The backend's `AuthUser` extractor reads the JWT **only**
> from the `Authorization: Bearer *** header** — raw `fetch()` with `credentials:
> 'include'` won't carry it. `adminFetch` (from `src/lib/api/admin.ts`) wraps every
> request with `authHeaders()` so the token attaches automatically:
>
> ```typescript
> // frontend/src/lib/api/admin.ts (lines 17-22)
> export async function adminFetch(path: string, init?: RequestInit): Promise<Response> {
>   const headers = new Headers(init?.headers);
>   const auth = authHeaders();
>   for (const [k, v] of Object.entries(auth)) headers.set(k, v);
>   return fetch(path, { ...init, headers, credentials: 'include' });
> }
> ```

**Step 2 — Typed proposal interface + loading.** The component declares its own
`DeletionProposal` interface (it needs `url_id` and `reason`, which `WorkProposal`
doesn't have). `load()` calls `GET /api/curator/work-deletions` and handles the
response, mapping errors to the i18n `workDeletions.error` key:

```svelte
// frontend/src/routes/curator/work-deletions/+page.svelte (lines 13-79)
  interface DeletionProposal {
    id: number;
    url_id: string;
    reason: string | null;
    proposed_by: number | null;
    proposed_by_username: string | null;
    status: string;
    created_at: string;
  }

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await adminFetch('/api/curator/work-deletions');
      if (!res.ok) {
        let msg = `HTTP ${res.status}`;
        try {
          const b = await res.json();
          if (b?.msg) msg = b.msg;
        } catch { /* ignore */ }
        throw new Error(msg);
      }
      const body = await res.json();
      proposals = Array.isArray(body?.proposals) ? body.proposals : [];
    } catch (e) {
      error = `${t('workDeletions.error')} ${e instanceof Error ? e.message : e}`;
      proposals = [];
    } finally {
      loading = false;
    }
  }
```

**Step 3 — Resolving (approve/reject).** The `resolve()` function posts an
`{ action: 'approve' | 'reject' }` body. Approval is gated behind a `confirm()` using
the i18n key `workDeletions.confirm` ("Delete this work permanently? This cannot be
undone."). While the mutation is pending, the button text flips to `…` for feedback:

```svelte
// frontend/src/routes/curator/work-deletions/+page.svelte (lines 70-90)
  async function resolve(p: DeletionProposal, action: 'approve' | 'reject') {
    if (busyId !== null) return;
    if (action === 'approve' && !confirm(t('workDeletions.confirm'))) return;
    busyId = p.id;
    error = '';
    actionMsg = '';
    try {
      const res = await adminFetch(`/api/curator/work-deletions/${p.id}/resolve`, {
        method: 'POST',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ action }),
      });
      ...
      // Refetch so the resolved proposal disappears from the pending queue.
      await load();
    } catch (e) {
      error = `${t('workDeletions.error')} ${e instanceof Error ? e.message : e}`;
    } finally {
      busyId = null;
    }
  }
```

The archive-mode template renders a `<table>` with one row per proposal:

```svelte
<!-- frontend/src/routes/curator/work-deletions/+page.svelte (archive mode) -->
          <tbody>
            {#each proposals as p (p.id)}
              <tr>
                <td><a class="archive-link archive-mono" href={`/works/${p.url_id}`}>{p.url_id}</a></td>
                <td>{p.reason ?? '—'}</td>
                <td class="archive-mono">{displayName(p)}</td>
                <td class="archive-mono">{relativeTime(p.created_at)}</td>
                <td>
                  <button class="archive-btn ok" type="button" disabled={busyId !== null}
                          onclick={() => resolve(p, 'approve')}>{busyId === p.id ? '…' : t('approvals.approve')}</button>
                  <button class="archive-btn bad" type="button" disabled={busyId !== null}
                          onclick={() => resolve(p, 'reject')}>{busyId === p.id ? '…' : t('approvals.reject')}</button>
                </td>
              </tr>
            {/each}
          </tbody>
```

### Check

- Sign in as an admin (level 100). Navigate to `/curator/work-deletions`. You should
  see a table of pending deletion requests, each with a `url_id` link to the work page.
- Click **Approve** on a request. The confirm dialog should fire; if you confirm,
  the row should disappear after the refetch (because `load()` is called again).
- As a non-admin, the page should redirect to `/` and briefly show "Checking
  access…".

### Troubleshooting

- **"Access denied" / silent redirect:** Confirm `auth.level >= 100`. The `isAdmin`
  getter reads from the same `auth.svelte.ts` store as Part 8; if `init()` hasn't
  resolved your role yet, the redirect wins. Check the `Authorization` header in
  devtools — it must be the JWT, not a session cookie.
- **Approve button does nothing:** The `!confirm(...)` short-circuits before setting
  `busyId`, so the button looks enabled. Open the browser console — if `t()` throws
  (missing `workDeletions.confirm` key), the exception aborts the handler.
- **Table stays empty after approve:** `resolve()` calls `load()` at the end. If that
  fails silently, the `catch` block sets `error` — look for the red error text below
  the table. Check `POST /api/curator/work-deletions/:id/resolve` returns `{"err":0}`.

---

## 44.3 Frontend: collection view, shelf manager, and list editor

### Goal

These three route pairs are the bread-and-butter organization UI users interact with
daily. They map onto the backend APIs we built in Part 12 (*Reading Lists*) and its
siblings:

| URL | Component | Backend API calls |
|-----|-----------|-------------------|
| `/collections` → `/collections/{id}` | `collections/+page.svelte`, `collections/[id]/+page.svelte` | `listReadingLists`, `createReadingList`, `getReadingListDetail`, `addReadingListItem` |
| `/shelves` → `/shelves/{id}` | `shelves/+page.svelte`, `shelves/[id]/+page.svelte` | `listShelves`, `createShelf`, `listWorksInShelf`, `removeWorkFromShelf`, `updateReadingStatus` |
| `/lists` → `/lists/{id}` | `lists/+page.svelte`, `lists/[id]/+page.svelte` | `listReadingLists`, `createReadingList`, `getReadingListDetail`, `addReadingListItem`, `removeReadingListItem` |

> 📚 **Naming note:** FicHub uses "collection" and "reading list" somewhat
> interchangeably in the data model — both are backed by the `lists` table and the
> `/lists` API. The *collection* page is the user-facing brand for shared, ordered
> bundles; the *lists* page is the same feature viewed as a personal library shelf.
> In code you'll see `collections/+page.svelte` importing `createReadingList` —
> that's intentional.

### Actions

#### 19.3a — The collections index (`/collections`)

The collections list page (`src/routes/collections/+page.svelte`) is a textbook
SvelteKit + runes pattern: `$state` for mutable data, `$derived` for computed
values, and `onMount` for the initial fetch. It imports helpers from
`src/lib/api/social.ts` and the `ArchiveButton` component from
`src/lib/ui/archive/ArchiveButton.svelte`:

```svelte
<!-- frontend/src/routes/collections/+page.svelte (lines 1-31) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { getPref } from '$lib/prefs';
  import { listReadingLists, createReadingList, updateReadingList, deleteReadingList } from '$lib/api/social';
  import type { ReadingList } from '$lib/api/social-types';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';

  const uiMode = $derived(getPref('uiMode'));

  let lists = $state<ReadingList[]>([]);
  let loading = $state(true);
  let error = $state('');
  let showCreateForm = $state(false);
  let editingId = $state<number | null>(null);
```

The create handler calls `createReadingList` and, on success, hides the form and
refreshes the list:

```svelte
// frontend/src/routes/collections/+page.svelte (lines 19-62)
  async function handleCreate() {
    if (!newTitle.trim()) return;
    creating = true;
    try {
      const res = await createReadingList(newTitle, newDescription, newIsPublic);
      if (res.err === 0) {
        showCreateForm = false;
        newTitle = '';
        newDescription = '';
        await loadLists();
      }
    } catch {
      error = 'Failed to create collection.';
    } finally {
      creating = false;
    }
  }
```

> 🔍 **Why `createReadingList` and not `createCollection`?** The API surface lives
> under `/lists` (see `src/lib/api/social.ts` line 751). The collections page is just
> a differently-branded view over the same endpoint — the backend doesn't distinguish
> "collections" from "reading lists" at the data layer.

The archive-mode template reuses the dual-layout trick. Each collection renders as a
card with edit/delete buttons. The edit form swaps in place via
`{#if editingId === list.id}`, and `ArchiveButton` is a reusable button wrapper that
renders as either an `<a>` (when `href` is set) or a `<button>`:

```svelte
<!-- frontend/src/routes/collections/+page.svelte (archive mode) -->
    {#each lists as list (list.id)}
      <div class="collection-card">
        <div class="collection-header">
          {#if editingId === list.id}
            <input type="text" bind:value={editTitle} class="edit-title" />
            <input type="text" bind:value={editDescription} class="edit-desc" />
            <p class="submit actions">
              <ArchiveButton onclick={saveEdit}>Save</ArchiveButton>
              <ArchiveButton onclick={() => editingId = null}>Cancel</ArchiveButton>
            </p>
          {:else}
            <h3><a href={`/collections/${list.id}`}>{list.title}</a></h3>
            <p class="collection-desc">{list.description}</p>
            <dl class="stats">
              <div class="stat"><dt>Items</dt><dd>{list.item_count}</dd></div>
              <div class="stat"><dt>Visibility</dt><dd>{list.is_public ? 'Public' : 'Private'}</dd></div>
              <div class="stat"><dt>Updated</dt><dd>{list.updated_at}</dd></div>
            </dl>
            <div class="collection-actions">
              <ArchiveButton onclick={() => startEdit(list)}>Edit</ArchiveButton>
              <ArchiveButton onclick={() => deleteList(list.id)}>Delete</ArchiveButton>
            </div>
          {/if}
        </div>
      </div>
    {/each}
```

#### 19.3b — The collection detail view (`/collections/{id}`)

The `+page.ts` sibling reads `params.id` and passes it as `data` to the component:

```typescript
// frontend/src/routes/collections/[id]/+page.ts
import type { PageLoad } from './$types';

export const load: PageLoad = ({ params }) => {
	return {
		list_id: Number(params.id),
	};
};
```

Back in the `.svelte` file, the component pulls `list_id` from props and fetches the
detail on mount. The owner-only "Add Work to Collection" form takes a numeric work id
and posts it via `addReadingListItem`:

```svelte
<!-- frontend/src/routes/collections/[id]/+page.svelte (lines 12-62) -->
  let { data } = $props<{ data?: { list_id: number } }>();
  const listId = $derived(data?.list_id ?? 0);

  let listDetail = $state<ReadingListDetail | null>(null);
  ...
  async function loadDetail() {
    loading = true;
    error = '';
    try {
      const res = await getReadingListDetail(listId);
      if (res.err === 0) listDetail = res;
    } catch {
      error = 'Network error loading collection.';
    } finally {
      loading = false;
    }
  }

  async function addItem() {
    if (!newWorkId.trim()) return;
    addingWorkId = parseInt(newWorkId, 10);
    try {
      const res = await addReadingListItem(listId, addingWorkId);
      if (res.err === 0) {
        newWorkId = '';
        await loadDetail();
      }
    } catch {
      error = 'Failed to add item.';
    } finally {
      addingWorkId = null;
    }
  }
```

The template only shows the add-item form when `listDetail.is_owner` is true — a
server-returned boolean telling the client who owns the list:

```svelte
<!-- frontend/src/routes/collections/[id]/+page.svelte (archive mode) -->
  {#if listDetail.is_owner}
    <div class="card">
      <h3>Add Work to Collection</h3>
      <dl>
        <div class="dl-row">
          <dt><label for="work-id">Work ID</label></dt>
          <dd><input id="work-id" type="number" bind:value={newWorkId} placeholder="Enter work ID" /></dd>
        </div>
      </dl>
      <p class="submit actions">
        <ArchiveButton onclick={addItem} disabled={addingWorkId !== null}>
          {addingWorkId !== null ? 'Adding...' : 'Add to Collection'}
        </ArchiveButton>
      </p>
    </div>
  {/if}
```

> 💡 **Why numeric work IDs?** The collection editor asks the user to type a work
> *id* (an integer from the work's page URL `/work/{id}`). It doesn't yet have a
> search/autocomplete widget — that's a future enhancement. The backend resolves
> the id to a title+author when it loads the list detail, so the displayed item
> shows the human-readable name, not the number.

#### 19.3c — The shelves page (`/shelves`) and shelf detail (`/shelves/{id}`)

The shelves index (`src/routes/shelves/+page.svelte`) is the richest of the three
because it combines **reading-status shelves** (the AO3-style "Want to Read /
Currently Reading / Finished / Dropped" quartet) with **custom shelves**
(user-named organizers like "To Re-read" or "Halloween Faves").

The status quartet is driven by a typed constant array that maps each
`ReadingStatus` value to an icon and label:

```typescript
// frontend/src/routes/shelves/+page.svelte (lines 11-17)
  const STATUS_SHELVES: { key: ReadingStatus; label: string; icon: string }[] = [
    { key: 'want_to_read', label: 'Want to Read', icon: '📋' },
    { key: 'reading', label: 'Currently Reading', icon: '📖' },
    { key: 'completed', label: 'Finished', icon: '✅' },
    { key: 'dropped', label: 'Dropped', icon: '❌' },
  ];

// frontend/src/lib/api/social-types.ts (line 357)
export type ReadingStatus = 'want_to_read' | 'reading' | 'completed' | 'dropped';
```

The `loadStatusShelves()` function calls `getReadingList()` (the reading-status
endpoint `GET /api/reading/list`, *not* the reading-list-bundles endpoint) and
groups results client-side by status. It uses `Promise.allSettled` so a single
failed `getWork()` call won't crash the whole section:

```typescript
// frontend/src/routes/shelves/+page.svelte (lines 21-71)
  async function loadStatusShelves() {
    statusLoading = true;
    try {
      const res = await getReadingList();
      if (res.err === 0) {
        const grouped: Record<ReadingStatus, ReadingListItem[]> = {
          want_to_read: [], reading: [], completed: [], dropped: [],
        };
        for (const item of res.reading_list) {
          grouped[item.status] = [...(grouped[item.status] ?? []), item];
        }
        statusItems = grouped;
        await Promise.allSettled(
          res.reading_list.map(async (r) => {
            try {
              const w = await getWork(r.work_id);
              works[r.work_id] = w.work;
            } catch { works[r.work_id] = null; }
          })
        );
      }
    } catch { /* status shelves are best-effort */ }
    finally { statusLoading = false; }
  }

  async function handleStatusChange(workId: number, status: ReadingStatus) {
    try {
      await updateReadingStatus(workId, status);
      await loadStatusShelves();
    } catch { /* ignore */ }
  }
```

The archive-mode template renders the status section as a `<fieldset>` with one
`<h3>` per status group, then a list of works with inline status buttons:

```svelte
<!-- frontend/src/routes/shelves/+page.svelte (archive mode) -->
    <fieldset class="archive-fieldset">
      <legend class="archive-legend">Reading Status</legend>
      {#if statusLoading}
        <p class="archive-muted"><span class="spinner"></span> Loading...</p>
      {:else}
        {#each STATUS_SHELVES as shelf (shelf.key)}
          <h3 class="archive-subheading">{shelf.label} <span class="archive-muted">({statusItems[shelf.key].length})</span></h3>
          {#if statusItems[shelf.key].length === 0}
            <p class="archive-muted">Nothing here yet.</p>
          {:else}
            <ul class="archive-list">
              {#each statusItems[shelf.key] as item (item.id)}
                {@const meta = workMeta(item.work_id)}
                <li class="archive-list-row">
                  {#if meta}
                    <a class="archive-link" href="/work/{item.work_id}">{meta.title}</a>
                    <span class="archive-muted"> by {meta.author}</span>
                  {:else}
                    <span class="archive-list-title">Work #{item.work_id}</span>
                  {/if}
                  <span class="archive-status-controls">
                    {#each STATUS_SHELVES as s (s.key)}
                      <button class="archive-btn archive-btn-sm"
                        class:current={item.status === s.key}
                        onclick={() => handleStatusChange(item.work_id, s.key)}
                        title="Set status: {s.label}"
                        aria-pressed={item.status === s.key}>{s.icon}</button>
                    {/each}
                  </span>
                </li>
              {/each}
            </ul>
          {/if}
        {/each}
      {/if}
    </fieldset>
```

The **custom shelves** section (same page, below) is a straightforward
create/list/delete flow. `createShelf` hits `POST /api/shelves`, and `deleteShelf`
hits `DELETE /api/shelves/{id}`. Note the delete confirmation: "Works inside will
not be affected." — deleting a shelf only removes the *container*, not the works
it referenced.

The shelf-detail page (`src/routes/shelves/[id]/+page.svelte`) lists works in a
shelf and lets the owner remove individual works. It preloads each work's metadata
via `getWork()` so the list shows real titles. It also uses three utility helpers
from `src/lib/util.ts` inline: `formatWords` (formats word counts), `detectSite`
(maps a source URL to "AO3" etc.), and `relativeTime` (formats timestamps):

```svelte
<!-- frontend/src/routes/shelves/[id]/+page.svelte (archive mode) -->
      <ul class="archive-list">
        {#each entries as e (e.id)}
          {@const meta = workMeta(e.work_id)}
          <li class="archive-list-row">
            <div class="archive-list-main">
              {#if meta}
                <a class="archive-link archive-list-title" href="/work/{e.work_id}">{meta.title}</a>
                <div class="archive-muted">by {meta.author} · {detectSite(meta.source) || 'Unknown'}</div>
                <div class="archive-meta-line">{formatWords(meta.words)} words · {meta.chapters} chapters</div>
              {:else}
                <span class="archive-list-title">Work #{e.work_id}</span>
                <div class="archive-muted">Loading metadata…</div>
              {/if}
              <div class="archive-meta-line">Added {relativeTime(e.added_at)}</div>
            </div>
            <button class="archive-btn" onclick={() => handleRemove(e.work_id)}>Remove</button>
          </li>
        {/each}
      </ul>
```

#### 19.3d — The reading-lists editor (`/lists` and `/lists/{id}`)

The lists pages (`src/routes/lists/+page.svelte` and `src/routes/lists/[id]/+page.svelte`)
mirror the collections pages in structure — same imports, same state shape, same
create/edit/delete flow — but the detail page adds an "Add Work" form with an
optional **blurb** textarea, since reading lists are *ordered bundles with
annotations*:

```svelte
<!-- frontend/src/routes/lists/[id]/+page.svelte (archive mode) -->
        {#if detail.is_owner}
          <fieldset class="archive-fieldset">
            <legend class="archive-legend">Manage</legend>
            <div class="archive-actions">
              <button class="archive-btn archive-btn-primary" onclick={() => { showAddForm = !showAddForm; addError = ''; }}>
                {showAddForm ? 'Cancel' : '+ Add Work'}
              </button>
            </div>

            {#if showAddForm}
              <form class="archive-form" onsubmit={(e) => { e.preventDefault(); handleAdd(); }}>
                <p class="archive-muted form-hint">Enter the numeric work id (from the work's page URL: /work/{'{'}id{'}'}).</p>
                <dl class="archive-dl">
                  <div class="dl-row"><dt><label for="add-work-id">Work id</label></dt>
                    <dd><input id="add-work-id" class="archive-input" type="text" inputmode="numeric" placeholder="Work id" bind:value={newWorkId} /></dd></div>
                  <div class="dl-row"><dt><label for="add-blurb">Blurb</label></dt>
                    <dd><input id="add-blurb" class="archive-input" type="text" placeholder="One-line blurb (optional)" bind:value={newBlurb} maxlength="500" /></dd></div>
                </dl>
                {#if addError}<p class="archive-error">{addError}</p>{/if}
                <button class="archive-btn archive-btn-primary" type="submit" disabled={adding}>
                  {adding ? 'Adding...' : 'Add to list'}
                </button>
              </form>
            {/if}
          </fieldset>
        {/if}
```

The list items render as an ordered list (`<ol>`) so the position number is semantic
HTML. Each item's blurb, if present, is shown in curly quotes:

```svelte
<!-- frontend/src/routes/lists/[id]/+page.svelte (archive mode) -->
        <ol class="archive-list">
          {#each detail.items as item (item.id)}
            <li class="archive-list-row">
              <span class="item-pos">{item.position}.</span>
              <span class="archive-list-main">
                <a class="archive-link archive-list-title" href="/work/{item.work_id}">{item.title}</a>
                <span class="archive-muted"> by {item.author}</span>
                {#if item.blurb}
                  <span class="item-blurb">“{item.blurb}”</span>
                {/if}
              </span>
              {#if detail.is_owner}
                <button class="archive-btn" onclick={() => handleRemove(item.work_id)} title="Remove from list">Remove</button>
              {/if}
            </li>
          {/each}
        </ol>
```

> 🌐 **i18n note:** The lists and collections pages currently use *hardcoded* English
> strings (e.g. "Reading Lists", "Work id") rather than `t('keys')`. The work-proposals
> page, by contrast, imports `t` from `$lib/i18n/index.svelte.ts` and routes labels
> through keys like `approvals.title` and `workDeletions.empty`. The dictionaries in
> `src/lib/i18n/dictionaries/en.ts` already declare the `approvals.*` and
> `workDeletions.*` namespaces — the organization pages are a migration work in
> progress, which is why Part 19 is a "frontend" part rather than a "backend" part.

### Check

- **Collections:** Sign in, go to `/collections`, click "+ New Collection", fill in a
  title and description, and click Create. The new card should appear in the list.
  Click its title to visit `/collections/{id}`, then use "Add Work to Collection"
  with a real work id from a `/work/{id}` page.
- **Shelves:** Go to `/shelves`. Under "Reading Status", find a work in "Want to
  Read" and click the 📖 button to move it to "Currently Reading". Under "Custom
  Shelves", create a shelf named "My Favorites", then visit its detail page and
  remove a work from it.
- **Lists:** Go to `/lists`. Create a list, add a work with a blurb, then visit the
  detail page and confirm the blurb appears next to the work title.

### Troubleshooting

- **"Network error loading collections":** This is the `catch` branch in
  `loadLists()`. Open devtools → Network and look for the `GET /api/lists` request.
  A 401 means your token is missing or expired — reload the page; the auth store
  re-runs `init()` on every mount. A 500 almost always means a backend SQL error
  (check the server logs for the exact query).
- **Work metadata shows "Loading metadata…" forever:** The `getWork()` helper calls
  `GET /api/works/{id}`. If that endpoint returns a non-`{err:0}` response, the
  `Promise.allSettled` catch sets `works[id] = null` and the component never
  retries. A common fix in development is to verify the work id exists in the
  database.
- **Archive mode not applying:** Every organization page starts with
  `const uiMode = $derived(getPref('uiMode'))`. The `getPref` function (from
  `src/lib/prefs.ts`) reads a preference store backed by `localStorage`. If `uiMode`
  is anything other than `'archive'`, the `{#if uiMode === 'archive'}` block is
  skipped and the modern-mode template renders instead. Use the Theme/Preferences
  drawer in the UI to set it to "archive".
- **Edit form doesn't save:** The `saveEdit()` handler in `collections/+page.svelte`
  calls `updateReadingList` with a `{ title, description }` patch. If the API returns
  `err !== 0`, the component silently stays in edit mode. Add a temporary
  `console.log(res)` inside `saveEdit` to inspect the response — most often this is
  a validation error on the backend (e.g. title too long).

---

## What you have now

- You can read and vote on work proposals from the frontend, with optimistic
  tally updates and a dual archive/modern UI.
- You understand the curator-only deletion-request queue: its route guard,
  `adminFetch` wrapper, confirm-on-approve safety check, and table layout.
- You can build a collections/shelves/lists frontend that calls the
  `listReadingLists`, `createReadingList`, `listShelves`, `createShelf`,
  `getReadingListDetail`, `addReadingListItem`, `updateReadingStatus`, and
  `removeWorkFromShelf` API helpers — and that renders a parallel archive-style
  (AO3-clone) and modern-style template driven by one `$derived` `uiMode`.

Next up: **Part 21** puts all of these features under test. You'll learn how FicHub
runs its frontend test suite with Vitest + `@testing-library/svelte`, how to mock
the `social` API module, and how the existing `page.test.ts` files for the
work-proposals and shelves pages are structured so you can add tests of your own.

---

*End of Part 44. On to [Part 45 — Testing](./45-testing/45-testing.md).*
