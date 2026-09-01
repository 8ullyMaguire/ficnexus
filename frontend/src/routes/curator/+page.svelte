<script lang="ts">
  import { adminFetch } from '$lib/api/admin';
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  interface QueueCount {
    href: string;
    label: string;
    icon: string;
    desc: string;
    count: number | null;
    fetching: boolean;
  }

  let checking = $state(true);
  let countError = $state('');
  const isAdmin = $derived(auth.level >= 100);

  // Curator/admin queues, in review order. Counts are best-effort: each is
  // fetched from the same API its queue page uses, and a failure (or a
  // per-queue API that needs the curator token / another privilege) simply
  // leaves the card without a badge instead of blocking the hub.
  let queues = $state<QueueCount[]>([
    {
      href: '/curator/consensus',
      label: 'Consensus Feed',
      icon: '🧭',
      desc: 'Everything peer-voted in one filterable view — fixes, flags, aliases, merges, roadmap.',
      count: null,
      fetching: false,
    },
    {
      href: '/curator/marginalia',
      label: 'Marginalia',
      icon: 'M',
      desc: 'Per-passage reader discussions — recent topics with work, chapter and reply counts.',
      count: null,
      fetching: true,
    },
    {
      href: '/curator/authors',
      label: 'Author Merges',
      icon: '🧬',
      desc: 'Review merge proposals that link source accounts to author profiles.',
      count: null,
      fetching: true,
    },
    {
      href: '/curator/flags',
      label: 'Tag Flags',
      icon: '🚩',
      desc: 'Tag flags reported by readers/curators, awaiting resolution.',
      count: null,
      fetching: true,
    },
    {
      href: '/curator/approvals',
      label: 'Approvals',
      icon: '✓',
      desc: 'Unified queue for collection submissions, curator vote proposals, comment triage, and forum edits.',
      count: null,
      fetching: true,
    },
    {
      href: '/curator/work-deletions',
      label: 'Work Deletion Requests',
      icon: 'X',
      desc: 'Pending requests to permanently delete works — approve deletes the work (can not be undone).',
      count: null,
      fetching: true,
    },
    {
      href: '/work-proposals',
      label: 'Work Proposals',
      icon: '🗳️',
      desc: 'Community-curated merge / split proposals for canonical works.',
      count: null,
      fetching: true,
    },
    {
      href: '/admin/moderation',
      label: 'Upload Moderation',
      icon: '📋',
      desc: 'Pending manual uploads waiting for approve / reject.',
      count: null,
      fetching: true,
    },
    {
      href: '/admin/comment-triage',
      label: 'Comment Triage',
      icon: '💬',
      desc: 'Comments the LLM triage flagged for human review.',
      count: null,
      fetching: true,
    },
  ]);

  function deriveCount(url: string, body: any): number | null {
    if (!body || typeof body !== 'object') return null;
    // Author merges: { err, items: [...] } — paginated list, no total field.
    if (url.includes('/curator/authors/pending') && Array.isArray(body.items)) {
      return body.items.length;
    }
    // Work proposals: { err, proposals: [...] } — full pending list.
    if (url.includes('/api/work-proposals') && Array.isArray(body.proposals)) {
      return body.proposals.length;
    }
    // Comment triage: { err, items: [...] } — full flagged list.
    if (url.includes('/api/admin/moderation/comments') && Array.isArray(body.items)) {
      return body.items.length;
    }
    // Upload moderation: { err, items: [...], total } — paginated, total is authoritative.
    if (url.includes('/api/admin/moderation/queue') && typeof body.total === 'number') {
      return body.total;
    }
    // Tag flags: { err, flags: [...] } — full unresolved list.
    if (url.includes('/api/curator/flags') && Array.isArray(body.flags)) {
      return body.flags.length;
    }
    // Approvals: { err, items, pending_counts } — sum the per-type pending counts.
    if (url.includes('/api/curator/approvals') && body.pending_counts) {
      return Object.values(body.pending_counts).reduce((a: number, b: unknown) => a + (Number(b) || 0), 0);
    }
    // Work deletion requests: { err, proposals: [...] } — full pending list.
    if (url.includes('/api/curator/work-deletions') && Array.isArray(body.proposals)) {
      return body.proposals.length;
    }
    return null;
  }

  const countSources: { href: string; url: string }[] = [
    { href: '/curator/authors', url: '/api/curator/authors/pending' },
    { href: '/curator/flags', url: '/api/curator/flags' },
    { href: '/curator/approvals', url: '/api/curator/approvals?status=pending' },
    { href: '/curator/work-deletions', url: '/api/curator/work-deletions' },
    { href: '/curator/marginalia', url: '/api/curator/marginalia?limit=1' },
    { href: '/work-proposals', url: '/api/work-proposals' },
    { href: '/admin/moderation', url: '/api/admin/moderation/queue?per_page=1' },
    { href: '/admin/comment-triage', url: '/api/admin/moderation/comments' },
  ];

  async function loadCounts() {
    countError = '';
    const settled = await Promise.allSettled(
      countSources.map(async (src) => {
        const res = await adminFetch(src.url, { credentials: 'include' });
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return { href: src.href, body: await res.json() };
      }),
    );
    settled.forEach((outcome, i) => {
      const q = queues.find((x) => x.href === countSources[i].href);
      if (!q) return;
      if (outcome.status === 'fulfilled') {
        q.count = deriveCount(countSources[i].url, outcome.value.body);
      } else {
        q.count = null;
      }
      q.fetching = false;
    });
    const failures = settled.filter((s) => s.status === 'rejected').length;
    if (failures > 0 && failures === settled.length) {
      countError = 'Could not load queue counts — the pages still work, badges just won\u2019t show.';
    }
  }

  function pendingCount(q: QueueCount): number | null {
    if (q.fetching) return null;
    return q.count;
  }

  function fmtCount(q: QueueCount): string {
    if (q.fetching) return '\u2026';
    if (q.count === null) return 'n/a';
    return String(q.count);
  }

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn || auth.level < 100) {
      goto('/');
      return;
    }
    checking = false;
    void loadCounts();
  });
</script>

<svelte:head><title>Curator Hub — FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
  {#if checking}
    <div class="loading"><p>Checking access...</p></div>
  {:else if isAdmin}
    <main class="archive-main">
      <div class="archive-content">
        <header class="archive-header">
          <h1 class="archive-page-title">Curator Hub</h1>
          <p class="archive-summary">One place for every queue a curator or admin has to review — counts are live from the same APIs the queue pages use.</p>
        </header>
        {#if countError}
          <p class="archive-note" role="status">{countError}</p>
        {/if}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Review Queues</legend>
          <dl class="archive-dl">
            {#each queues as q (q.href)}
              <div class="dl-row">
                <dt><a class="archive-link" href={q.href}>{q.label}</a></dt>
                <dd><span class="archive-count">{fmtCount(q)}</span> <span class="archive-note">— {q.desc}</span></dd>
              </div>
            {/each}
          </dl>
        </fieldset>
        <p class="archive-note"><a class="archive-link" href="/admin">Admin Dashboard</a> · <a class="archive-link" href="/requests">Requests Board</a></p>
      </div>
    </main>
  {/if}
{:else}
{#if checking}
  <div class="loading"><p>Checking access...</p></div>
{:else if isAdmin}
  <div class="hub">
    <header class="hub-head">
      <h1>🛡️ Curator Hub</h1>
      <p class="subtitle">
        One place for every queue a curator or admin has to review — author merges,
        tag flags, work proposals, upload moderation, and comment triage. Counts are
        live from the same APIs the queue pages use.
      </p>
    </header>

    {#if countError}
      <div class="notice" role="status">{countError}</div>
    {/if}

    <div class="grid">
      {#each queues as q (q.href)}
        <a class="card queue-card" href={q.href} data-sveltekit-preload>
          <div class="queue-icon" aria-hidden="true">{q.icon}</div>
          <div class="queue-body">
            <div class="queue-title-row">
              <h2>{q.label}</h2>
              <span
                class="badge"
                class:badge-zero={pendingCount(q) === 0}
                class:badge-na={pendingCount(q) === null}
                title={q.count === null ? 'Count unavailable (API not reachable or needs extra auth)' : `${q.count} pending`}
              >
                {fmtCount(q)}
              </span>
            </div>
            <p class="queue-desc">{q.desc}</p>
          </div>
        </a>
      {/each}
    </div>

    <p class="hub-foot">
      <a href="/admin">🛠️ Admin Dashboard</a>
      <span class="dot">·</span>
      <a href="/requests">🙋 Requests Board</a>
    </p>
  </div>
  {/if}
{/if}

<style>
  /* ── Archive mode ─────────────────────────────────────────── */
  .archive-main {
    max-width: var(--archive-max-width, 1100px);
    margin: 0 auto;
    padding: 0;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-content { max-width: 900px; margin: 0 auto; padding: 1rem; }
  .archive-header {
    padding: 0.8rem 1rem 0.6rem;
    border-bottom: 2px solid var(--archive-link, #990000);
    background: var(--archive-bg, #ffffff);
  }
  .archive-page-title { font-size: 1.6em; font-weight: 700; margin: 0; }
  .archive-summary { color: var(--archive-muted, #666666); font-size: 0.95em; margin: 0.4em 0 0; }
  .archive-fieldset { border: 1px solid var(--archive-border, #dddddd); padding: 1em 1.25em; margin: 1.2rem 0; }
  .archive-legend { font-weight: 700; font-size: 1.05em; padding: 0 0.4em; }
  .archive-dl { margin: 0; padding: 0; }
  .archive-dl .dl-row dt { font-weight: 700; font-size: 0.95em; margin: 0.7em 0 0.15em; }
  .archive-dl .dl-row:first-child dt { margin-top: 0; }
  .archive-dl .dl-row dd { margin: 0; }
  .archive-count { display: inline-block; min-width: 1.8rem; text-align: center; border: 1px solid var(--archive-border, #dddddd); background: var(--archive-bg-raised, #f5f5f5); padding: 0 0.4em; font-size: 0.85em; }
  .archive-link { color: var(--archive-link, #990000); }
  .archive-error { color: #990000; font-weight: 700; font-size: 0.92em; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.88em; }
  p.archive-note { margin: 0.5em 0; }

  /* ── Modern mode ──────────────────────────────────────────── */
  .loading { text-align: center; padding: 3rem; }
  .hub { max-width: 960px; margin: 0 auto; padding: 1.5rem; }
  .hub-head h1 { margin-bottom: 0.3rem; }
  .subtitle { color: var(--color-muted); font-size: 0.92rem; max-width: 90ch; line-height: 1.55; }
  .notice {
    margin: 1rem 0;
    padding: 0.7rem 1rem;
    border: 1px solid var(--color-warning, #f59e0b);
    border-radius: var(--radius-sm);
    color: var(--color-warning, #f59e0b);
    font-size: 0.85rem;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 1rem;
    margin-top: 1.4rem;
  }
  .queue-card {
    display: flex;
    gap: 0.85rem;
    align-items: flex-start;
    text-decoration: none;
    color: inherit;
    transition: border-color 0.15s, transform 0.05s;
  }
  .queue-card:hover {
    border-color: var(--color-primary, #6366f1);
    text-decoration: none;
  }
  .queue-card:active { transform: translateY(1px); }
  .queue-icon { font-size: 1.5rem; line-height: 1.2; }
  .queue-body { min-width: 0; flex: 1; }
  .queue-title-row { display: flex; align-items: center; gap: 0.5rem; }
  .queue-title-row h2 { font-size: 1.02rem; margin: 0; }
  .badge {
    margin-left: auto;
    min-width: 1.9rem;
    text-align: center;
    background: color-mix(in srgb, var(--color-primary, #6366f1) 18%, transparent);
    color: var(--color-primary-hover, #818cf8);
    border-radius: 999px;
    padding: 0.08rem 0.5rem;
    font-size: 0.78rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .badge-zero { background: color-mix(in srgb, var(--color-success, #22c55e) 16%, transparent); color: var(--color-success, #22c55e); }
  .badge-na { background: var(--color-surface-2); color: var(--color-muted); }
  .queue-desc { margin: 0.3rem 0 0; font-size: 0.85rem; color: var(--color-muted); line-height: 1.5; }
  .hub-foot { margin-top: 1.6rem; font-size: 0.88rem; color: var(--color-muted); }
  .hub-foot a { color: var(--color-primary-hover); }
  .hub-foot .dot { margin: 0 0.4rem; }
</style>
