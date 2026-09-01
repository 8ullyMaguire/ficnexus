<script lang="ts">
  // Curator consensus queue — universal proposals (translate / ui_string /
  // content_fix / post_edit / …). AO3 archive styling; one filterable page
  // per the product rule. Votes: approve/dismiss, per-kind quorum shown.
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';
  import { t } from '$lib/i18n/index.svelte';
  import {
    listProposals, voteProposal, decideProposal,
    type Proposal,
  } from '$lib/api/proposals';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';
  import ProposalDiff from '$lib/components/ProposalDiff.svelte';

  let items = $state<Proposal[]>([]);
  let loading = $state(true);
  let error = $state('');
  let busy = $state<number | null>(null);
  let kind = $state('');
  let status = $state('pending');
  let locale = $state('');
  let page = $state(1);
  let note = $state('');
  let openNotes = $state<number | null>(null);

  const KINDS = ['', 'translate', 'ui_string', 'content_fix', 'metadata_fix', 'post_edit', 'request_edit', 'doc_edit', 'work_deletion', 'collection_add'];
  const STATUSES = ['pending', 'approved', 'dismissed', 'superseded', ''];

  async function load() {
    loading = true; error = '';
    try {
      const res = await listProposals({ kind: kind || undefined, status: status || undefined, locale: locale || undefined, page });
      if (res.err !== 0) throw new Error('failed to load proposals');
      items = res.items ?? [];
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load proposals';
    } finally { loading = false; }
  }

  async function vote(p: Proposal, d: 'approve' | 'dismiss') {
    busy = p.id;
    try {
      const res = await voteProposal(p.id, d);
      if (res.err !== 0) error = res.msg ?? 'vote failed';
      await load();
    } finally { busy = null; }
  }

  async function fastDecide(p: Proposal, d: 'approve' | 'dismiss') {
    busy = p.id;
    try {
      const res = await decideProposal(p.id, d, note || undefined);
      if (res.err !== 0) error = res.msg ?? 'decide failed';
      note = ''; openNotes = null;
      await load();
    } finally { busy = null; }
  }

  function translateText(p: Proposal): string {
    const pl = p.payload as Record<string, unknown>;
    return typeof pl?.text === 'string' ? pl.text : JSON.stringify(pl ?? {});
  }
  function payloadMeta(p: Proposal): string {
    const pl = p.payload as Record<string, unknown>;
    const bits: string[] = [];
    if (pl?.locale) bits.push(String(pl.locale));
    if (pl?.field) bits.push(String(pl.field));
    if (pl?.reason) bits.push(String(pl.reason));
    return bits.join(' · ');
  }
  function targetHref(p: Proposal): string | null {
    const id = String(p.target_id ?? '');
    switch (p.target_type) {
      case 'request': return `/requests/${id.split(':')[0]}`;
      case 'forum_topic': return `/forum/topic/${id.split(':')[0]}`;
      case 'work_meta': return `/works/${id}`;
      case 'chapter': return `/read/${id}`;
      default: return null;
    }
  }

  onMount(async () => {
    await auth.init();
    // Curator gate: legacy role>=5 → site level >= 50.
    if (!auth.isLoggedIn || auth.level < 50) { goto('/'); return; }
    await load();
  });
</script>

<svelte:head><title>{t('curator.proposalsTitle')} — FicNexus</title></svelte:head>

<div class="queue-page">
  <h1 class="page-title">{t('curator.proposalsTitle')}</h1>
  <p class="page-sub">{t('curator.proposalsSub')}</p>

  <div class="toolbar">
    <label>{t('curator.filterKind')}
      <select bind:value={kind} onchange={load}>
        {#each KINDS as k (k)}<option value={k}>{k === '' ? 'all' : k}</option>{/each}
      </select>
    </label>
    <label>{t('curator.filterStatus')}
      <select bind:value={status} onchange={load}>
        {#each STATUSES as s (s)}<option value={s}>{s === '' ? 'all' : s}</option>{/each}
      </select>
    </label>
    <label>{t('curator.filterLocale')}
      <input type="text" size="5" placeholder="es" bind:value={locale} onkeydown={(e) => e.key === 'Enter' && load()} />
    </label>
    <button class="q-btn" onclick={load}>{t('common.filter')}</button>
  </div>

  {#if loading}
    <p class="muted">{t('common.loading')}</p>
  {:else if error}
    <p class="err">{error}</p>
  {:else if items.length === 0}
    <p class="muted">{t('curator.empty')}</p>
  {:else}
    <ul class="prop-list">
      {#each items as p (p.id)}
        {@const votes = (p.approves ?? 0) + (p.dismisses ?? 0)}
        <li class="prop" class:pending={p.status === 'pending'}>
          <div class="prop-head">
            <span class="badge">{p.kind}</span>
            {#if p.source === 'llm'}<span class="badge llm">machine</span>{/if}
            <a class="target" href={targetHref(p) ?? '#'}>{p.target_type}:{p.target_id}</a>
            <span class="meta">{payloadMeta(p)}</span>
            <span class="spacer"></span>
            <span class="status s-{p.status}">{p.status}</span>
          </div>
          <div class="prop-body">{translateText(p).slice(0, 600)}</div>
          {#if p.kind === 'translate' || p.kind === 'ui_string'}
            {@const pl = p.payload as Record<string, unknown>}
            {@const orig = typeof pl?.original === 'string' ? pl.original : ''}
            {@const prop = typeof pl?.text === 'string' ? pl.text : translateText(p)}
            <ProposalDiff original={orig} proposed={prop} field={typeof pl?.field === 'string' ? pl.field : undefined} />
          {/if}
          <div class="prop-foot">
            <span class="who">{p.proposer ?? 'system'} · {new Date(p.created_at).toLocaleDateString()} · {votes}/{p.quorum ?? '?'} votes ({p.approves ?? 0}+ / {p.dismisses ?? 0}−)</span>
            {#if p.status === 'pending'}
              <span class="acts">
                {#if p.source !== 'llm' || p.kind === 'translate'}
                  <button class="q-btn ok" disabled={busy === p.id} onclick={() => vote(p, 'approve')}>{t('curator.approve')}</button>
                  <button class="q-btn no" disabled={busy === p.id} onclick={() => vote(p, 'dismiss')}>{t('curator.dismiss')}</button>
                {/if}
                <button class="q-btn fast" disabled={busy === p.id} onclick={() => (openNotes = openNotes === p.id ? null : p.id)} title={t('curator.fastHelp')}>⚡</button>
              </span>
            {/if}
            {#if openNotes === p.id}
              <span class="note-row">
                <input type="text" placeholder={t('curator.notePh')} bind:value={note} />
                <button class="q-btn ok" onclick={() => fastDecide(p, 'approve')}>{t('curator.approve')}</button>
                <button class="q-btn no" onclick={() => fastDecide(p, 'dismiss')}>{t('curator.dismiss')}</button>
              </span>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
    <div class="pager">
      {#if page > 1}<ArchiveButton href="#" onclick={(e) => { e.preventDefault(); page--; load(); }}>← {t('common.prev')}</ArchiveButton>{/if}
      <span class="muted">p{page}</span>
      {#if items.length === 20}<ArchiveButton href="#" onclick={(e) => { e.preventDefault(); page++; load(); }}>{t('common.next')} →</ArchiveButton>{/if}
    </div>
  {/if}
</div>

<style>
  .queue-page { max-width: 860px; margin: 0 auto; font-family: Georgia, 'Times New Roman', serif; color: #000; }
  .page-title { font-size: 1.7em; font-weight: 400; color: #900; margin: 0 0 .15em; border-bottom: 1px solid #ddd; padding-bottom: .3em; }
  .page-sub { color: #666; font-size: .92em; margin: 0 0 1em; }
  .toolbar { display: flex; gap: .8rem; align-items: end; margin-bottom: 1rem; flex-wrap: wrap; font-size: .9em; }
  .toolbar label { display: flex; flex-direction: column; gap: .15em; color: #666; }
  .toolbar select, .toolbar input { border: 1px solid #bbb; border-radius: .25em; padding: .2em .4em; font-family: inherit; background: #fff; }
  .q-btn { background: #eee; background-image: linear-gradient(#fff 2%, #ddd 95%, #bbb 100%); color: #444; border: 1px solid #bbb; border-bottom: 1px solid #aaa; border-radius: .25em; padding: .25em .75em; cursor: pointer; font: inherit; font-size: .85em; }
  .q-btn:hover { color: #900; border-top-color: #999; border-left-color: #999; box-shadow: inset 2px 2px 2px #bbb; }
  .q-btn.ok:hover { color: #2e9e5b; }
  .q-btn.no:hover { color: #c0392b; }
  .prop-list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: .6rem; }
  .prop { border: 1px solid #ddd; padding: .7em .9em; background: #fff; }
  .prop.pending { border-left: 4px solid #900; }
  .prop-head { display: flex; gap: .5rem; align-items: baseline; flex-wrap: wrap; font-size: .86em; }
  .badge { background: #900; color: #fff; border-radius: .25em; padding: .05em .5em; font-size: .85em; }
  .badge.llm { background: #666; }
  .target { color: #111; border-bottom: 1px solid; text-decoration: none; }
  .target:hover { color: #999; }
  .meta { color: #666; }
  .spacer { flex: 1; }
  .status { font-variant: small-caps; color: #666; }
  .status.s-approved { color: #2e9e5b; }
  .status.s-dismissed { color: #c0392b; }
  .status.s-superseded { color: #999; }
  .prop-body { margin: .45em 0; line-height: 1.45; white-space: pre-wrap; word-break: break-word; }
  .prop-foot { display: flex; justify-content: space-between; align-items: center; gap: .6rem; flex-wrap: wrap; font-size: .82em; color: #666; }
  .acts { display: flex; gap: .35rem; }
  .note-row { display: flex; gap: .35rem; width: 100%; }
  .note-row input { flex: 1; border: 1px solid #bbb; border-radius: .25em; padding: .2em .4em; font: inherit; font-size: .95em; }
  .pager { margin-top: 1rem; display: flex; gap: .6rem; align-items: center; }
  .muted { color: #666; }
  .err { color: #c0392b; }
</style>
