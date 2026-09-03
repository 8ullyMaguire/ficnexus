<script lang="ts">
  import { adminFetch } from '$lib/api/admin';
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  interface VisitorRow { date?: string; week_start?: string; month?: string; visitors: number }
  interface Engagement {
    active_users: Record<'1d' | '7d' | '30d', number>;
    view_only_users: Record<'1d' | '7d' | '30d', number>;
    action_events: Record<'7d' | '30d', number>;
    total_events: Record<'7d' | '30d', number>;
  }
  interface RecentEvent { at: string; client_id: string; path: string; type: string; user_agent?: string | null }
  interface EndpointRow { path: string; event_type: string; count: number; group: string }
  interface GroupRow { group: string; count: number }

  let daily = $state<VisitorRow[]>([]);
  let weekly = $state<VisitorRow[]>([]);
  let monthly = $state<VisitorRow[]>([]);
  let engagement = $state<Engagement | null>(null);
  let recent = $state<RecentEvent[]>([]);
  let endpoints = $state<EndpointRow[]>([]);
  let groups = $state<GroupRow[]>([]);
  let epDays = $state(7);
  let epLoading = $state(false);
  let loading = $state(true);
  let error = $state('');

  async function loadEndpoints() {
    epLoading = true;
    try {
      const res = await adminFetch(`/api/admin/endpoint-usage?days=${epDays}&limit=20`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      endpoints = body.endpoints ?? [];
      groups = body.groups ?? [];
    } catch {}
    finally { epLoading = false; }
  }

  async function load() {
    loading = true;
    error = '';
    try {
      const res = await adminFetch('/api/admin/analytics');
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      daily = body.unique_visitors?.daily ?? [];
      weekly = body.unique_visitors?.weekly ?? [];
      monthly = body.unique_visitors?.monthly ?? [];
      engagement = body.engagement ?? null;
      recent = body.recent_events ?? [];
      await loadEndpoints();
    } catch (e) {
      error = `Failed to load analytics: ${e instanceof Error ? e.message : e}`;
    } finally {
      loading = false;
    }
  }

  onMount(load);

  function fmtDate(iso: string): string {
    if (!iso) return '—';
    const d = new Date(iso + (iso.length === 7 ? '-01' : ''));
    return d.toISOString().slice(0, 10);
  }

  function maxVisitors(rows: VisitorRow[]): number {
    return Math.max(1, ...rows.map((r) => r.visitors));
  }

  function barWidth(v: number, max: number): string {
    return `${Math.max(2, Math.round((v / max) * 100))}%`;
  }

  function eventLabel(e: RecentEvent): string {
    // Trim query strings for readability; mark actions with a dot.
    const p = e.path.split('?')[0];
    return p.length > 60 ? p.slice(0, 58) + '…' : p;
  }

  function shortClient(cid: string): string {
    return cid.length > 8 ? cid.slice(0, 8) + '…' : cid;
  }

  function sum(rows: VisitorRow[]): number {
    return rows.reduce((a, r) => a + r.visitors, 0);
  }
</script>

{#if uiMode === 'archive'}
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Usage Analytics</h1>
        <p class="archive-summary">Non-PII anonymous usage (X-Client-ID). Unique visitors daily/weekly/monthly, plus engagement: who performs actions vs who just browses.</p>
      </header>
      <div class="archive-form-actions">
        <button class="archive-btn" type="button" onclick={load}>↻ Refresh</button>
      </div>
      {#if loading}
        <p class="archive-note">Loading…</p>
      {:else if error}
        <p class="archive-error" role="alert">{error}</p>
      {:else}
        {#if engagement}
          <fieldset class="archive-fieldset">
            <legend class="archive-legend">Engagement by window</legend>
            <table class="archive-table">
              <thead><tr><th>Window</th><th>Active users</th><th>View-only</th><th>Action events</th><th>Total events</th><th>Action rate</th></tr></thead>
              <tbody>
                {#each (['1d', '7d', '30d'] as const) as win}
                  <tr>
                    <td>{win === '1d' ? 'Today' : win === '7d' ? 'Last 7 days' : 'Last 30 days'}</td>
                    <td>{engagement.active_users[win]}</td>
                    <td>{engagement.view_only_users[win]}</td>
                    <td>{win === '1d' ? '—' : engagement.action_events[win]}</td>
                    <td>{win === '1d' ? '—' : engagement.total_events[win]}</td>
                    <td>{win !== '1d' && engagement.total_events[win] > 0 ? Math.round((engagement.action_events[win] / engagement.total_events[win]) * 100) + '%' : '—'}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
            <dl class="archive-dl">
              <div class="dl-row"><dt>Unique visitors (30d)</dt><dd>{sum(daily)} <span class="archive-note">distinct anonymous ids</span></dd></div>
              <div class="dl-row"><dt>Active users (30d)</dt><dd>{engagement.active_users['30d']} <span class="archive-note">performed an action</span></dd></div>
              <div class="dl-row"><dt>View-only (30d)</dt><dd>{engagement.view_only_users['30d']} <span class="archive-note">browsed but never acted</span></dd></div>
              <div class="dl-row"><dt>Actions (30d)</dt><dd>{engagement.action_events['30d']} <span class="archive-note">downloads / votes / etc.</span></dd></div>
            </dl>
          </fieldset>
        {/if}
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Daily unique visitors (30d)</legend>
          {#if daily.length === 0}
            <p class="archive-note">No usage events yet — they start recording the moment users hit the site.</p>
          {:else}
            <table class="archive-table">
              <tbody>
                {#each daily as d (d.date)}
                  <tr title="{d.date}: {d.visitors} visitors">
                    <td class="archive-mono">{fmtDate(d.date!)}</td>
                    <td class="bar-cell"><span class="bar" style="width: {barWidth(d.visitors, maxVisitors(daily))}"></span></td>
                    <td>{d.visitors}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        </fieldset>
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Weekly unique visitors</legend>
          {#if weekly.length === 0}<p class="archive-note">No data yet.</p>{:else}
            <table class="archive-table">
              <tbody>
                {#each weekly as w (w.week_start)}
                  <tr title="{fmtDate(w.week_start!)}: {w.visitors} visitors">
                    <td class="archive-mono">{fmtDate(w.week_start!)}</td>
                    <td class="bar-cell"><span class="bar" style="width: {barWidth(w.visitors, maxVisitors(weekly))}"></span></td>
                    <td>{w.visitors}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        </fieldset>
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Monthly unique visitors</legend>
          {#if monthly.length === 0}<p class="archive-note">No data yet.</p>{:else}
            <table class="archive-table">
              <tbody>
                {#each monthly as m (m.month)}
                  <tr title="{fmtDate(m.month!)}: {m.visitors} visitors">
                    <td class="archive-mono">{fmtDate(m.month!)}</td>
                    <td class="bar-cell"><span class="bar" style="width: {barWidth(m.visitors, maxVisitors(monthly))}"></span></td>
                    <td>{m.visitors}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        </fieldset>
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Top endpoints (last {epDays}d) — view vs action</legend>
          <div style="margin-bottom:0.5rem">
            <label class="archive-note">Window: <select value={epDays} onchange={(e)=>{epDays=parseInt((e.target as HTMLSelectElement).value); loadEndpoints();}}><option value={7}>7 days</option><option value={30}>30 days</option></select> {#if epLoading}<span class="archive-note"> Loading…</span>{/if}</label>
          </div>
          {#if groups.length>0}
            <div style="display:flex; gap:0.5rem; flex-wrap:wrap; margin-bottom:0.6rem">
              {#each groups as g}<span class="archive-mono" style="border:1px solid #ddd; padding:2px 6px">{g.group}: {g.count}</span>{/each}
            </div>
          {/if}
          {#if endpoints.length===0}<p class="archive-note">No endpoint data yet.</p>{:else}
            <table class="archive-table">
              <thead><tr><th>Path</th><th>Type</th><th>Group</th><th>Count</th><th></th></tr></thead>
              <tbody>{#each endpoints as r}<tr><td class="archive-mono">{r.path}</td><td>{r.event_type}</td><td>{r.group}</td><td>{r.count}</td><td><span class="bar" style="width:{Math.max(2,Math.round((r.count/Math.max(1, endpoints[0]?.count||1))*60))}px"></span></td></tr>{/each}</tbody>
            </table>
          {/if}
        </fieldset>
        <fieldset class="archive-fieldset">
          <legend class="archive-legend">Recent activity timeline</legend>
          {#if recent.length === 0}
            <p class="archive-note">No events yet.</p>
          {:else}
            <table class="archive-table">
              <thead><tr><th>Time</th><th>Client</th><th>Path</th><th>Type</th></tr></thead>
              <tbody>
                {#each recent as e (e.at + e.client_id)}
                  <tr>
                    <td class="archive-mono">{new Date(e.at).toLocaleString()}</td>
                    <td class="archive-mono">{shortClient(e.client_id)}</td>
                    <td class="archive-mono">{eventLabel(e)}</td>
                    <td>{e.type}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        </fieldset>
      {/if}
    </div>
  </main>
{:else}
<div class="stats-page">
  <header class="page-head">
    <h1>📊 Usage Analytics</h1>
    <p class="subtitle">
      Non-PII anonymous usage (X-Client-ID). Unique visitors daily/weekly/monthly, plus
      engagement: who performs actions vs who just browses.
    </p>
    <div class="controls">
      <button class="btn" onclick={load}>↻ Refresh</button>
    </div>
  </header>

  {#if loading}
    <p class="empty">Loading…</p>
  {:else if error}
    <div class="error-card"><strong>⚠️ {error}</strong></div>
  {:else}
    <!-- Engagement summary cards -->
    {#if engagement}
      <div class="cards">
        <div class="card">
          <h3>👀 Unique visitors (30d)</h3>
          <p class="big">{sum(daily)}</p>
          <p class="muted">distinct anonymous ids</p>
        </div>
        <div class="card">
          <h3>⚡ Active users (30d)</h3>
          <p class="big">{engagement.active_users['30d']}</p>
          <p class="muted">performed an action</p>
        </div>
        <div class="card">
          <h3>👁️ View-only (30d)</h3>
          <p class="big">{engagement.view_only_users['30d']}</p>
          <p class="muted">browsed but never acted</p>
        </div>
        <div class="card">
          <h3>🎯 Actions (30d)</h3>
          <p class="big">{engagement.action_events['30d']}</p>
          <p class="muted">downloads / votes / etc.</p>
        </div>
      </div>

      <!-- Engagement by window -->
      <section class="panel">
        <h2>Engagement by window</h2>
        <div class="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Window</th>
                <th>Active users</th>
                <th>View-only</th>
                <th>Action events</th>
                <th>Total events</th>
                <th>Action rate</th>
              </tr>
            </thead>
            <tbody>
              {#each (['1d', '7d', '30d'] as const) as win}
                <tr>
                  <td>{win === '1d' ? 'Today' : win === '7d' ? 'Last 7 days' : 'Last 30 days'}</td>
                  <td>{engagement.active_users[win]}</td>
                  <td>{engagement.view_only_users[win]}</td>
                  <td>{win === '1d' ? '—' : engagement.action_events[win]}</td>
                  <td>{win === '1d' ? '—' : engagement.total_events[win]}</td>
                  <td>
                    {win !== '1d' && engagement.total_events[win] > 0
                      ? Math.round((engagement.action_events[win] / engagement.total_events[win]) * 100) + '%'
                      : '—'}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </section>
    {/if}

    <!-- Daily visitors -->
    <section class="panel">
      <h2>Daily unique visitors (30d)</h2>
      {#if daily.length === 0}
        <p class="empty">No usage events yet — they start recording the moment users hit the site.</p>
      {:else}
        <div class="bars">
          {#each daily as d (d.date)}
            <div class="bar-row" title="{d.date}: {d.visitors} visitors">
              <span class="bar-label">{fmtDate(d.date!)}</span>
              <div class="bar-track">
                <div class="bar-fill" style="width: {barWidth(d.visitors, maxVisitors(daily))}"></div>
              </div>
              <span class="bar-val">{d.visitors}</span>
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <!-- Weekly + monthly -->
    <div class="two-col">
      <section class="panel">
        <h2>Weekly unique visitors</h2>
        {#if weekly.length === 0}
          <p class="empty">No data yet.</p>
        {:else}
          <div class="bars">
            {#each weekly as w (w.week_start)}
              <div class="bar-row" title="{fmtDate(w.week_start!)}: {w.visitors} visitors">
                <span class="bar-label">{fmtDate(w.week_start!)}</span>
                <div class="bar-track">
                  <div class="bar-fill" style="width: {barWidth(w.visitors, maxVisitors(weekly))}"></div>
                </div>
                <span class="bar-val">{w.visitors}</span>
              </div>
            {/each}
          </div>
        {/if}
      </section>
      <section class="panel">
        <h2>Monthly unique visitors</h2>
        {#if monthly.length === 0}
          <p class="empty">No data yet.</p>
        {:else}
          <div class="bars">
            {#each monthly as m (m.month)}
              <div class="bar-row" title="{fmtDate(m.month!)}: {m.visitors} visitors">
                <span class="bar-label">{fmtDate(m.month!)}</span>
                <div class="bar-track">
                  <div class="bar-fill" style="width: {barWidth(m.visitors, maxVisitors(monthly))}"></div>
                </div>
                <span class="bar-val">{m.visitors}</span>
              </div>
            {/each}
          </div>
        {/if}
      </section>
    </div>

    <!-- Per-endpoint breakdown -->
    <section class="panel">
      <h2>Top endpoints (last {epDays}d)</h2>
      <div class="controls" style="margin-bottom:0.6rem">
        <label>Window:
          <select value={epDays} onchange={(e)=>{epDays=parseInt((e.target as HTMLSelectElement).value); loadEndpoints();}}>
            <option value={7}>7 days</option>
            <option value={30}>30 days</option>
          </select>
        </label>
        {#if epLoading}<span class="empty"> Loading…</span>{/if}
      </div>
      {#if groups.length>0}
        <div style="display:flex; gap:0.5rem; flex-wrap:wrap; margin-bottom:0.6rem">
          {#each groups as g}
            <span class="mono" style="background:#eef; border:1px solid #ccd; border-radius:6px; padding:2px 8px">{g.group}: {g.count}</span>
          {/each}
        </div>
      {/if}
      {#if endpoints.length===0}
        <p class="empty">No endpoint data yet.</p>
      {:else}
        <div class="table-wrap">
          <table>
            <thead><tr><th>Path</th><th>Type</th><th>Group</th><th>Count</th><th></th></tr></thead>
            <tbody>
              {#each endpoints as r}
                <tr>
                  <td class="mono">{r.path}</td>
                  <td><span class="tl-type {r.event_type==='action' ? 'badge-action' : 'badge-view'}">{r.event_type}</span></td>
                  <td>{r.group}</td>
                  <td>{r.count}</td>
                  <td style="width:80px"><div class="bar-track"><div class="bar-fill" style="width:{Math.max(4, Math.round((r.count / Math.max(1, endpoints[0]?.count||1))*100))}%"></div></div></td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </section>

    <!-- Recent timeline -->
    <section class="panel">
      <h2>Recent activity timeline</h2>
      {#if recent.length === 0}
        <p class="empty">No events yet.</p>
      {:else}
        <div class="timeline">
          {#each recent as e (e.at + e.client_id)}
            <div class="tl-item">
              <span class="tl-dot {e.type === 'action' ? 'action' : 'view'}"></span>
              <span class="tl-time mono">{new Date(e.at).toLocaleString()}</span>
              <span class="tl-client mono">{shortClient(e.client_id)}</span>
              <span class="tl-path mono">{eventLabel(e)}</span>
              <span class="tl-type {e.type === 'action' ? 'badge-action' : 'badge-view'}">{e.type}</span>
            </div>
          {/each}
        </div>
      {/if}
    </section>
  {/if}
</div>
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
  .archive-content { max-width: 960px; margin: 0 auto; padding: 1rem; }
  .archive-header {
    padding: 0.8rem 1rem 0.6rem;
    border-bottom: 2px solid var(--archive-link, #990000);
    background: var(--archive-bg, #ffffff);
  }
  .archive-page-title { font-size: 1.6em; font-weight: 700; margin: 0; }
  .archive-summary { color: var(--archive-muted, #666666); font-size: 0.95em; margin: 0.4em 0 0; }
  .archive-form-actions { margin: 1rem 0; }
  .archive-fieldset { border: 1px solid var(--archive-border, #dddddd); padding: 1em 1.25em; margin: 1.2rem 0; }
  .archive-legend { font-weight: 700; font-size: 1.05em; padding: 0 0.4em; }
  .archive-table { width: 100%; border-collapse: collapse; font-size: 0.9em; }
  .archive-table th,
  .archive-table td { text-align: left; padding: 0.45em 0.6em; border-bottom: 1px solid var(--archive-border, #dddddd); }
  .archive-table th { font-weight: 700; font-size: 0.85em; border-bottom: 2px solid var(--archive-border, #dddddd); }
  .archive-table td.bar-cell { width: 40%; }
  .archive-table td.bar-cell .bar { display: inline-block; height: 0.7em; min-width: 2px; background: var(--archive-accent, #990000); }
  .archive-mono { font-family: 'Courier New', Courier, monospace; font-size: 0.92em; }
  .archive-dl { margin: 0.9em 0 0; padding: 0; }
  .archive-dl .dl-row dt { font-weight: 700; font-size: 0.9em; margin: 0.5em 0 0.1em; }
  .archive-dl .dl-row:first-child dt { margin-top: 0; }
  .archive-dl .dl-row dd { margin: 0; }
  .archive-btn {
    display: inline-block; padding: 0.35em 1.1em; font-size: 0.88em; font-weight: 700;
    border: 1px solid var(--archive-border, #dddddd); background: var(--archive-btn-bg, #f5f5f5);
    color: var(--archive-text, #2a2a2a); cursor: pointer; font-family: inherit;
  }
  .archive-btn:hover:not(:disabled) { background: #eeeeee; }
  .archive-error { color: #990000; font-weight: 700; font-size: 0.92em; }
  .archive-note { color: var(--archive-muted, #666666); font-size: 0.88em; }
  p.archive-note { margin: 0.5em 0; }

  /* ── Modern mode ──────────────────────────────────────────── */
  .stats-page { padding: 1rem; max-width: 1200px; margin: 0 auto; }
  .page-head { margin-bottom: 1rem; }
  .subtitle { color: var(--color-text-muted, #888); font-size: 0.9rem; }
  .controls { margin-top: 0.5rem; }
  .btn {
    background: var(--color-primary, #4a7);
    color: #fff; border: 0; border-radius: 6px; padding: 0.45rem 0.9rem; cursor: pointer;
  }
  .empty { color: var(--color-text-muted, #888); padding: 1rem 0; }
  .error-card { background: #fdd; border: 1px solid #f99; padding: 0.75rem; border-radius: 6px; }
  .cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 0.75rem; margin-bottom: 1rem; }
  .card { background: var(--color-surface, #fff); border: 1px solid var(--color-border, #ddd); border-radius: 8px; padding: 0.9rem; }
  .card h3 { margin: 0 0 0.3rem; font-size: 0.85rem; color: var(--color-text-muted, #888); }
  .card .big { font-size: 1.6rem; font-weight: 700; margin: 0; }
  .card .muted { color: var(--color-text-muted, #888); font-size: 0.8rem; margin: 0.2rem 0 0; }
  .panel { background: var(--color-surface, #fff); border: 1px solid var(--color-border, #ddd); border-radius: 8px; padding: 1rem; margin-bottom: 1rem; }
  .panel h2 { margin: 0 0 0.6rem; font-size: 1rem; }
  .two-col { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }
  @media (max-width: 900px) { .two-col { grid-template-columns: 1fr; } }
  .table-wrap { overflow-x: auto; }
  table { width: 100%; border-collapse: collapse; font-size: 0.9rem; }
  th, td { padding: 0.45rem 0.6rem; text-align: left; border-bottom: 1px solid var(--color-border, #eee); }
  th { color: var(--color-text-muted, #888); font-weight: 600; }
  .mono { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 0.82rem; }
  .bars { display: flex; flex-direction: column; gap: 3px; }
  .bar-row { display: grid; grid-template-columns: 90px 1fr 42px; align-items: center; gap: 8px; font-size: 0.8rem; }
  .bar-label { text-align: right; color: var(--color-text-muted, #888); }
  .bar-track { background: var(--color-border, #eee); border-radius: 3px; height: 14px; overflow: hidden; }
  .bar-fill { background: var(--color-primary, #4a7); height: 100%; border-radius: 3px; transition: width 0.3s; }
  .bar-val { font-weight: 600; }
  .timeline { display: flex; flex-direction: column; gap: 4px; max-height: 360px; overflow-y: auto; }
  .tl-item { display: flex; align-items: center; gap: 8px; font-size: 0.8rem; padding: 2px 0; }
  .tl-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
  .tl-dot.view { background: #999; }
  .tl-dot.action { background: var(--color-success, #4a7); }
  .tl-time { color: var(--color-text-muted, #888); min-width: 150px; }
  .tl-client { color: var(--color-text-muted, #888); min-width: 70px; }
  .tl-path { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .tl-type { font-size: 0.7rem; padding: 1px 6px; border-radius: 8px; }
  .badge-view { background: #eee; color: #666; }
  .badge-action { background: #d4f0dd; color: #2a7a45; }
</style>
