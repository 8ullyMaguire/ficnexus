<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref, setPref, type UserPrefs } from '$lib/prefs';
  import { getMyLevel, type LevelProgress } from '$lib/api/forum';
  import {
    listSiteCredentials,
    setSiteCredentials,
    deleteSiteCredentials,
    type SiteCredential,
  } from '$lib/api/siteCredentials';
  import { getNotificationPrefs, updateNotificationPrefs } from '$lib/api/social';
  import type { NotificationPreferences } from '$lib/api/social-types';
  // Login-requiring sites supported by the fanfic-scrapers crate.
  const LOGIN_SITES = [
    { domain: 'fanfics.me', name: 'Fanfics.me' },
    { domain: 'fictionhunt.com', name: 'FictionHunt' },
    { domain: 'inkbunny.net', name: 'InkBunny' },
    { domain: 'sofurry.com', name: 'SoFurry' },
    { domain: 'dokuga.com', name: 'Dokuga' },
  ];

  let loading = $state(true);
  let error = $state('');
  let saved = $state('');
  let credentials = $state<SiteCredential[]>([]);

  // ── Personalized recs pref ─────────────────────────────────────────
  let personalizedRecs = $state(true);
  let recsSaving = $state(false);

  // ── Notification preferences ─────────────────────────────────────
  const notifDefaults: NotificationPreferences = {
    comment_reply: true,
    follow_update: true,
    work_update: true,
    badge_earned: true,
    recommendation: true,
    curator_promotion: true,
    email_digest: 'none',
    comments_on_work: true,
    replies_to_comments: true,
    kudos_on_work: true,
    bookmarks_on_work: true,
    follows: true,
    mentions: true,
  };
  let notifPrefs = $state<NotificationPreferences>({ ...notifDefaults });
  let notifSaved = $state('');
  let notifError = $state('');

  // ── Interface style & archive skin ─────────────────────────────────────
  let uiMode = $state<'archive' | 'modern'>(getPref('uiMode'));
  let archiveSkin = $state<'zerafina' | 'ao3'>(getPref('archiveSkin'));
  let skinSaved = $state('');
  const isArchiveMode = $derived(uiMode === 'archive');

  // ── F7 leveling display ──────────────────────────────────────────────
  let level = $state<LevelProgress | null>(null);
  let levelError = $state('');

  // Per-site form state, keyed by domain. Eagerly initialized so the
  // template never mutates $state during render (Svelte 5 forbids that).
  let forms = $state<Record<string, { username: string; password: string; consent: boolean; saving: boolean }>>({});
  let removing = $state<string | null>(null);

  function credFor(domain: string): SiteCredential | undefined {
    return credentials.find((c) => c.domain === domain);
  }

  // Svelte 5 cannot `bind:` to a function call, so expose an explicit
  // accessor (identity-preserving) the template can bind against.
  function formState(domain: string) {
    return forms[domain];
  }

  function formatDate(iso: string): string {
    try {
      return new Date(iso).toLocaleDateString();
    } catch {
      return iso;
    }
  }

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn) {
      loading = false;
      return;
    }
    // Seed per-site form state once — the template binds to it and Svelte 5
    // forbids mutating $state during render.
    for (const site of LOGIN_SITES) {
      forms[site.domain] = { username: '', password: '', consent: false, saving: false };
    }
    await loadLevel();
    await loadCredentials();
    await loadRecsPref();
    await loadNotifPrefs();
  });

  // ── Personalized recs pref ─────────────────────────────────────────
  async function loadRecsPref() {
    try {
      const res = await fetch('/api/me/prefs', { credentials: 'include' });
      if (res.ok) {
        const prefs = await res.json();
        const found = prefs.find((p: { key: string; value: unknown }) => p.key === 'recs.personalized');
        if (found) personalizedRecs = found.value !== 'false';
      }
    } catch { /* ignore — default is true */ }
  }

  async function togglePersonalizedRecs() {
    recsSaving = true;
    try {
      const res = await fetch('/api/me/prefs', {
        method: 'PUT',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify([{ key: 'recs.personalized', value: personalizedRecs ? 'true' : 'false' }]),
      });
      if (res.ok) saved = t('settings.saved');
    } catch { /* silent */ }
    recsSaving = false;
  }

  // ── Notification preferences ─────────────────────────────────────
  async function loadNotifPrefs() {
    try {
      const res = await getNotificationPrefs();
      if (res.err === 0) {
        if (res.preferences) {
          notifPrefs = { ...notifDefaults, ...res.preferences };
        } else {
          // Direct field format
          notifPrefs = { ...notifPrefs };
        }
      }
    } catch { /* ignore — defaults are fine */ }
  }

  async function saveNotifPrefs() {
    notifSaved = '';
    notifError = '';
    try {
      const res = await updateNotificationPrefs(notifPrefs);
      if (res.err === 0) {
        notifSaved = 'Notification preferences saved.';
      } else {
        notifError = 'Failed to save preferences.';
      }
    } catch {
      notifError = 'Network error saving preferences.';
    }
  }

  // ── Interface style ────────────────────────────────────────────────
  function setUiMode(mode: NonNullable<UserPrefs['uiMode']>) {
    uiMode = mode;
    setPref('uiMode', mode);
  }

  function setArchiveSkin(skin: 'zerafina' | 'ao3') {
    archiveSkin = skin;
    setPref('archiveSkin' as keyof UserPrefs, skin as NonNullable<UserPrefs['archiveSkin']>);
    skinSaved = t('settings.skinSaved');
  }

  // ── F7 leveling ─────────────────────────────────────────────────────
  async function loadLevel() {
    levelError = '';
    try {
      const res = await getMyLevel();
      if (res.err === 0) {
        level = res;
      } else {
        levelError = res.msg ?? t('settings.levelError');
      }
    } catch {
      levelError = t('settings.levelError');
    }
  }

  async function loadCredentials() {
    loading = true;
    error = '';
    try {
      const res = await listSiteCredentials();
      if (res.err !== 0) {
        error = t('settings.credentialsLoadError');
        return;
      }
      credentials = res.credentials;
    } catch {
      error = t('settings.credentialsNetworkError');
    } finally {
      loading = false;
    }
  }

  async function handleSave(domain: string) {
    const form = formState(domain);
    if (!form.consent) {
      error = `You must check the consent box to store credentials for ${domain}.`;
      return;
    }
    if (!form.username.trim() || !form.password) {
      error = 'Username and password are required.';
      return;
    }
    form.saving = true;
    error = '';
    saved = '';
    try {
      await setSiteCredentials(domain, form.username.trim(), form.password);
      form.password = '';
      form.consent = false;
      saved = `Credentials saved for ${domain}. They expire in 30 days.`;
      await loadCredentials();
    } catch {
      error = `Failed to save credentials for ${domain}.`;
    } finally {
      form.saving = false;
    }
  }

  async function handleRemove(domain: string) {
    removing = domain;
    error = '';
    try {
      await deleteSiteCredentials(domain);
      credentials = credentials.filter((c) => c.domain !== domain);
    } catch {
      error = `Failed to remove credentials for ${domain}.`;
    } finally {
      removing = null;
    }
  }
</script>

<div class="settings-page">
  <h1>{t('settings.title')}</h1>

  {#if !auth.isLoggedIn}
    <div class="card empty">
      <p class="muted">{t('settings.loginToManage')}</p>
    </div>
  {:else if loading}
    <div class="card">
      <p class="muted"><span class="spinner"></span> {t('common.loading')}</p>
    </div>
  {:else}
    {#if error}
      <div class="card error-card"><p class="error-text">{error}</p></div>
    {/if}
    {#if saved}
      <div class="card success-card"><p>{saved}</p></div>
    {/if}

    <!-- ── Interface Style ─────────────────────────────────────────── -->
    {#if uiMode === 'archive'}
      <fieldset class="archive-fieldset">
        <legend>Interface Style</legend>
        <p class="archive-muted">Choose the look and feel of FicNexus.</p>
        <dl class="archive-dl">
          <dt>Theme</dt>
          <dd>
            <div class="archive-radio-group">
              <label class="archive-radio">
                <input type="radio" name="uimode" checked onchange={() => setUiMode('archive')} />
                Archive (AO3-style)
              </label>
              <label class="archive-radio">
                <input type="radio" name="uimode" onchange={() => setUiMode('modern')} />
                Modern
              </label>
            </div>
          </dd>
        </dl>
      </fieldset>
    {:else}
      <section class="card">
        <h2>Interface Style</h2>
        <p class="muted">Choose the look and feel of FicNexus.</p>
        <div class="ui-options">
          <button
            class="ui-option"
            class:selected={isArchiveMode}
            type="button"
            onclick={() => setUiMode('archive')}
          >
            <span class="ui-option-title">Archive (AO3-style)</span>
            <span class="ui-option-desc">Classic, text-dense layout inspired by AO3</span>
          </button>
          <button
            class="ui-option"
            class:selected={uiMode === 'modern'}
            type="button"
            onclick={() => setUiMode('modern')}
          >
            <span class="ui-option-title">Modern</span>
            <span class="ui-option-desc">Clean, spacious design with cards</span>
          </button>
        </div>
      </section>
    {/if}

    <!-- ── Archive Skin Selector ───────────────────────────────────── -->
    {#if uiMode === 'archive'}
      <section class="card">
        <h2>{t('settings.skinTitle')}</h2>
        <p class="muted">{t('settings.skinDesc')}</p>
        <div class="ui-options">
          <button
            class="ui-option"
            class:selected={archiveSkin === 'zerafina'}
            type="button"
            onclick={() => setArchiveSkin('zerafina')}
          >
            <span class="ui-option-title">{t('settings.skinZerafina')}</span>
            <span class="ui-option-desc">Full AO3-style styling with enhanced stats icons and tag boxes</span>
          </button>
          <button
            class="ui-option"
            class:selected={archiveSkin === 'ao3'}
            type="button"
            onclick={() => setArchiveSkin('ao3')}
          >
            <span class="ui-option-title">{t('settings.skinAo3')}</span>
            <span class="ui-option-desc">Minimal AO3 default header styling</span>
          </button>
        </div>
        {#if skinSaved}
          <p class="success-text">{skinSaved}</p>
        {/if}
      </section>
    {/if}


    <!-- ── F7: level badge + exp progress ─────────────────────────── -->
    <section class="card level-card">
      <h2>{t('settings.levelTitle')}</h2>
      {#if level}
        <div class="level-row">
          <span class="level-badge">{t('settings.level', { level: level.level })}</span>
          <span class="level-exp muted">{level.exp} {t('settings.exp')}</span>
        </div>
        {#if level.level >= 100}
          <p class="muted">{t('settings.levelMax')}</p>
        {:else}
          <div
            class="progress-track"
            role="progressbar"
            aria-valuenow={Math.round(level.progress * 100)}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-label={t('settings.expProgressAria')}
          >
            <div class="progress-fill" style="width: {Math.round(level.progress * 100)}%"></div>
          </div>
          <p class="muted level-to-next">
            {t('settings.expToNext', { exp: level.exp_to_next })}
          </p>
        {/if}
      {:else if levelError}
        <p class="muted error-text">{levelError}</p>
      {:else}
        <p class="muted"><span class="spinner"></span> {t('common.loading')}</p>
      {/if}
    </section>

    <section class="card">
      <h2>{t('settings.credentialsTitle')}</h2>
      <p class="muted">
        {t('settings.credentialsIntro')} <strong>{t('settings.credentialsExpiry')}</strong> {t('settings.credentialsTail')}
      </p>

      <div class="cred-list">
        {#each LOGIN_SITES as site (site.domain)}
          {@const cred = credFor(site.domain)}
          <div class="cred-item">
            <div class="cred-head">
              <span class="tag">{site.name}</span>
              <span class="muted domain">{site.domain}</span>
            </div>

            {#if cred}
              <div class="cred-configured">
                <p class="muted">
                  Username: <strong>{cred.username}</strong> · expires {formatDate(cred.expires_at)}
                </p>
                <button
                  class="btn btn-secondary sm"
                  onclick={() => handleRemove(site.domain)}
                  disabled={removing === site.domain}
                >
                  {#if removing === site.domain}<span class="spinner"></span>{:else}Remove{/if}
                </button>
              </div>
            {:else}
              {@const f = formState(site.domain)}
              <div class="cred-form">
                <label>
                  Username
                  <input
                    type="text"
                    bind:value={f.username}
                    autocomplete="username"
                    placeholder="your username on {site.name}"
                  />
                </label>
                <label>
                  Password
                  <input
                    type="password"
                    bind:value={f.password}
                    autocomplete="current-password"
                    placeholder="••••••••"
                  />
                </label>
                <label class="consent">
                  <input type="checkbox" bind:checked={f.consent} />
                  I consent to storing these credentials for 30 days
                </label>
                <button
                  class="btn btn-primary sm"
                  onclick={() => handleSave(site.domain)}
                  disabled={f.saving}
                >
                  {#if f.saving}<span class="spinner"></span>{:else}Save{/if}
                </button>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </section>

    <!-- ── Recommendations ─────────────────────────────────────────── -->
    <section class="card">
      <h2>{t('settings.recsTitle')}</h2>
      <label class="toggle-row">
        <span>
          <strong>{t('settings.personalizedRecs')}</strong>
          <span class="muted">{t('settings.personalizedRecsDesc')}</span>
        </span>
        <input
          type="checkbox"
          bind:checked={personalizedRecs}
          onchange={togglePersonalizedRecs}
          disabled={recsSaving}
        />
      </label>
    </section>

    <!-- ── Notification Preferences ────────────────────────────────────────── -->
    <section class="card">
      <h2>{t('settings.notificationsTitle')}</h2>
      {#if !auth.isLoggedIn}
        <p class="muted">Log in to manage notification preferences.</p>
      {:else}
        {#if notifError}
          <p class="error-text">{notifError}</p>
        {/if}
        {#if notifSaved}
          <p class="success-text">{notifSaved}</p>
        {/if}
        <dl class="archive-dl">
          <div class="dl-row dl-row-group">
            <dt class="dl-group-label">Activity on my work</dt>
          </div>
          <div class="dl-row">
            <dt>{t('settings.notifCommentReply')}</dt>
            <dd><input type="checkbox" bind:checked={notifPrefs.comment_reply} onchange={saveNotifPrefs} /></dd>
          </div>
          <div class="dl-row">
            <dt>{t('settings.notifCommentsOnWork')}</dt>
            <dd><input type="checkbox" bind:checked={notifPrefs.comments_on_work} onchange={saveNotifPrefs} /></dd>
          </div>
          <div class="dl-row">
            <dt>{t('settings.notifKudosOnWork')}</dt>
            <dd><input type="checkbox" bind:checked={notifPrefs.kudos_on_work} onchange={saveNotifPrefs} /></dd>
          </div>
          <div class="dl-row">
            <dt>{t('settings.notifBookmarksOnWork')}</dt>
            <dd><input type="checkbox" bind:checked={notifPrefs.bookmarks_on_work} onchange={saveNotifPrefs} /></dd>
          </div>

          <div class="dl-row dl-row-group">
            <dt class="dl-group-label">Comments &amp; replies</dt>
          </div>
          <div class="dl-row">
            <dt>{t('settings.notifWorkUpdate')}</dt>
            <dd><input type="checkbox" bind:checked={notifPrefs.work_update} onchange={saveNotifPrefs} /></dd>
          </div>
          <div class="dl-row">
            <dt>{t('settings.notifRepliesToComments')}</dt>
            <dd><input type="checkbox" bind:checked={notifPrefs.replies_to_comments} onchange={saveNotifPrefs} /></dd>
          </div>

          <div class="dl-row dl-row-group">
            <dt class="dl-group-label">Social</dt>
          </div>
          <div class="dl-row">
            <dt>{t('settings.notifFollowUpdate')}</dt>
            <dd><input type="checkbox" bind:checked={notifPrefs.follow_update} onchange={saveNotifPrefs} /></dd>
          </div>
          <div class="dl-row">
            <dt>{t('settings.notifFollows')}</dt>
            <dd><input type="checkbox" bind:checked={notifPrefs.follows} onchange={saveNotifPrefs} /></dd>
          </div>
          <div class="dl-row">
            <dt>{t('settings.notifMentions')}</dt>
            <dd><input type="checkbox" bind:checked={notifPrefs.mentions} onchange={saveNotifPrefs} /></dd>
          </div>

          <div class="dl-row dl-row-group">
            <dt class="dl-group-label">Recognition &amp; platform</dt>
          </div>
          <div class="dl-row">
            <dt>{t('settings.notifBadgeEarned')}</dt>
            <dd><input type="checkbox" bind:checked={notifPrefs.badge_earned} onchange={saveNotifPrefs} /></dd>
          </div>
          <div class="dl-row">
            <dt>{t('settings.notifCuratorPromotion')}</dt>
            <dd><input type="checkbox" bind:checked={notifPrefs.curator_promotion} onchange={saveNotifPrefs} /></dd>
          </div>
          <div class="dl-row">
            <dt>{t('settings.notifRecommendation')}</dt>
            <dd><input type="checkbox" bind:checked={notifPrefs.recommendation} onchange={saveNotifPrefs} /></dd>
          </div>

          <div class="dl-row dl-row-group">
            <dt class="dl-group-label">Email digest</dt>
          </div>
          <div class="dl-row">
            <dt>{t('settings.notifDigest')}</dt>
            <dd>
              <select bind:value={notifPrefs.email_digest} onchange={saveNotifPrefs}>
                <option value="none">No digest</option>
                <option value="weekly">Weekly</option>
                <option value="daily">Daily</option>
              </select>
            </dd>
          </div>
        </dl>
      {/if}
    </section>
  {/if}
</div>

<style>
  .settings-page {
    max-width: var(--max-width);
    margin: 0 auto;
    padding: 1rem;
  }
  h1 {
    margin-top: 0;
    margin-bottom: 1rem;
  }
  h2 {
    margin: 0 0 0.5rem;
    font-size: 1.2rem;
  }
  .empty {
    text-align: center;
    padding: 2rem;
  }
  .error-card {
    border-color: var(--color-error);
  }
  .success-card {
    border-color: var(--color-success, #2e7d32);
  }
  .error-text {
    color: var(--color-error);
  }
  .level-card {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }
  .level-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  .level-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    background: var(--color-chip-bg, #eef);
    border: 1px solid var(--color-border, #ddd);
    padding: 0.2rem 0.7rem;
    border-radius: 999px;
    font-weight: 700;
    font-size: 0.95rem;
  }
  .level-exp {
    font-size: 0.85rem;
  }
  .progress-track {
    height: 0.6rem;
    border-radius: 999px;
    background: var(--color-surface-2, #1c1c1c);
    border: 1px solid var(--color-border, #333);
    overflow: hidden;
  }
  .progress-fill {
    height: 100%;
    border-radius: 999px;
    background: linear-gradient(90deg, var(--color-primary, #2b4bd7), var(--color-primary-light, #5b7cf0));
    transition: width 0.3s ease;
  }
  .level-to-next {
    font-size: 0.82rem;
    margin: 0;
  }
  .cred-list {
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
    margin-top: 1rem;
  }
  .cred-item {
    border: 1px solid var(--color-border, #333);
    border-radius: 8px;
    padding: 0.8rem;
    background: var(--color-surface-2, #1c1c1c);
  }
  .cred-head {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }
  .domain {
    font-size: 0.85rem;
  }
  .cred-configured {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }
  .cred-configured p {
    margin: 0;
  }
  .cred-form {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .cred-form label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.85rem;
  }
  .cred-form input[type='text'],
  .cred-form input[type='password'] {
    padding: 0.4rem 0.6rem;
    border-radius: 6px;
    border: 1px solid var(--color-border, #444);
    background: var(--color-surface, #141414);
    color: var(--color-text);
  }
  .cred-form .consent {
    flex-direction: row;
    align-items: center;
    gap: 0.4rem;
  }
  .cred-form .consent input {
    margin: 0;
  }
  .btn.sm {
    padding: 0.35rem 0.7rem;
    font-size: 0.82rem;
  }
  .spinner {
    display: inline-block;
    width: 0.8rem;
    height: 0.8rem;
    border: 2px solid var(--color-border, #555);
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    vertical-align: middle;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .toggle-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    cursor: pointer;
  }
  .toggle-row span {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }
  .toggle-row input[type="checkbox"] {
    width: 1.2rem;
    height: 1.2rem;
    flex-shrink: 0;
    cursor: pointer;
  }
  /* ── Interface style cards ──────────────────────────────────────── */
  .ui-options {
    display: flex;
    gap: 0.8rem;
    margin-top: 0.75rem;
  }
  .ui-option {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    padding: 0.85rem 1rem;
    border: 2px solid var(--color-border, #333);
    border-radius: 8px;
    background: var(--color-surface-2, #1c1c1c);
    cursor: pointer;
    text-align: left;
    transition: border-color 0.15s, background 0.15s;
  }
  .ui-option:hover {
    border-color: var(--color-primary, #2b4bd7);
  }
  .ui-option.selected {
    border-color: var(--color-primary, #2b4bd7);
    background: color-mix(in srgb, var(--color-primary, #2b4bd7) 12%, var(--color-surface-2, #1c1c1c));
  }
  .ui-option-title {
    font-weight: 700;
    font-size: 0.95rem;
    color: var(--color-text);
  }
  .ui-option-desc {
    font-size: 0.82rem;
    color: var(--color-muted);
  }
  @media (max-width: 480px) {
    .ui-options {
      flex-direction: column;
    }
  }
  /* ── Archive fieldset / dl rows ────────────────────────────────────── */
  .archive-fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.75rem 1rem;
    font-family: 'Georgia', 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
    background: var(--archive-bg, #ffffff);
  }
  .archive-fieldset legend {
    font-weight: 700;
    color: var(--archive-heading, #990000);
    padding: 0 0.4rem;
  }
  .archive-dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.4rem 1rem;
    margin: 0.5rem 0 0;
  }
  .archive-dl dt {
    font-weight: 600;
    color: var(--archive-heading, #333333);
  }
  .archive-dl dd {
    margin: 0;
  }
  .archive-radio-group {
    display: flex;
    gap: 1.2rem;
  }
  .archive-radio {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    cursor: pointer;
    font-size: 0.9rem;
  }
  .archive-radio input {
    margin: 0;
  }
  .archive-muted {
    color: var(--archive-muted, #666666);
    font-style: italic;
    margin: 0.2rem 0 0.5rem;
  }
</style>
