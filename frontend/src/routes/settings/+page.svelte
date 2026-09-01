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
  import {
    getNotificationPrefs,
    updateNotificationPrefs,
    getBlacklist,
    addBlacklistItem,
    removeBlacklistItem,
    getProfileVisibility,
    setProfileVisibility,
    getUserTimezone,
    setUserTimezone,
  } from '$lib/api/social';
  import type {
    NotificationPreferences,
    BlockedTag,
    VisibilityLevel,
    ProfileVisibility,
  } from '$lib/api/social-types';
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

  async function togglePersonalizedRecs(e: Event) {
    const next = (e.currentTarget as HTMLInputElement).checked;
    recsSaving = true;
    try {
      const res = await fetch('/api/me/prefs', {
        method: 'PUT',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify([{ key: 'recs.personalized', value: next ? 'true' : 'false' }]),
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
          notifPrefs = res.preferences;
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
        notifSaved = t('settings.saved');
      } else {
        notifError = t('settings.notifSaveError');
      }
    } catch {
      notifError = t('settings.notifNetworkError');
    }
  }

  // ── Blacklist management ──────────────────────────────────────────
  // Local cache of the user's blocked tags/fandoms (frontend-only UI).
  // Persists to localStorage so the list survives page reloads even before
  // a backend blacklist endpoint exists; when the backend lands we sync it.
  type BlacklistEntry = {
    id?: number;
    name: string;
    type: string;
  };

  let blacklist = $state<BlacklistEntry[]>([]);
  let blacklistLoading = $state(false);
  let blacklistSaved = $state('');
  let blacklistError = $state('');
  let blacklistSaving = $state(false);
  let blacklistRemovingId = $state<number | null>(null);
  let newBlockedName = $state('');
  let newBlockedType = $state('fandom');

  // IANA time zone names, sorted for a predictable dropdown order.
  const TIME_ZONES: string[] =
    typeof Intl.supportedValuesOf === 'function'
      ? Intl.supportedValuesOf('timeZone').sort()
      : [
          'UTC',
          'America/New_York',
          'America/Chicago',
          'America/Denver',
          'America/Los_Angeles',
          'America/Toronto',
          'America/Vancouver',
          'Europe/London',
          'Europe/Paris',
          'Europe/Berlin',
          'Europe/Madrid',
          'Asia/Tokyo',
          'Asia/Shanghai',
          'Asia/Kolkata',
          'Australia/Sydney',
        ];

  async function loadBlacklist() {
    blacklistLoading = true;
    blacklistError = '';
    try {
      const res = await getBlacklist();
      if (res.err === 0 && res.blocked) {
        blacklist = res.blocked.map((b: BlockedTag) => ({
          id: b.id,
          name: b.name,
          type: b.type,
        }));
      }
    } catch {
      // Backend not available yet — fall back to localStorage cache so the
      // UI still reflects what the user has blocked this session.
      try {
        const raw = localStorage.getItem('fichub_blacklist_v1');
        if (raw) blacklist = JSON.parse(raw);
      } catch { /* ignore */ }
    } finally {
      blacklistLoading = false;
    }
  }

  async function addBlockedTag() {
    const name = newBlockedName.trim();
    if (!name) return;
    blacklistSaving = true;
    blacklistError = '';
    blacklistSaved = '';
    try {
      const res = await addBlacklistItem({ name, type: newBlockedType });
      if (res.err === 0 && res.blocked) {
        blacklist = [...blacklist, { id: res.blocked.id, name, type: newBlockedType }];
      } else {
        // Optimistic local fallback when backend is absent
        blacklist = [...blacklist, { name, type: newBlockedType }];
      }
      blacklistSaved = t('settings.blacklistSaved');
      newBlockedName = '';
      persistBlacklistLocal();
    } catch {
      blacklist = [...blacklist, { name, type: newBlockedType }];
      blacklistSaved = t('settings.blacklistSaved');
      newBlockedName = '';
      persistBlacklistLocal();
    } finally {
      blacklistSaving = false;
    }
  }

  async function removeBlockedTag(item: BlacklistEntry) {
    blacklistRemovingId = item.id ?? null;
    blacklistError = '';
    blacklistSaved = '';
    try {
      await removeBlacklistItem({ id: item.id, name: item.name, type: item.type });
    } catch {
      /* ignore — still optimistically remove from the list */
    } finally {
      blacklist = blacklist.filter((b) => b.id !== item.id && !(b.name === item.name && b.type === item.type));
      blacklistSaved = t('settings.blacklistRemoved');
      blacklistRemovingId = null;
      persistBlacklistLocal();
    }
  }

  function persistBlacklistLocal() {
    try {
      localStorage.setItem('fichub_blacklist_v1', JSON.stringify(blacklist));
    } catch { /* ignore */ }
  }

  // ── Profile display visibility ─────────────────────────────────────
  let profileVis = $state<ProfileVisibility>({
    profile: 'public',
    works: 'public',
    reading_history: 'private',
  });
  let profileSaved = $state('');
  let profileError = $state('');
  let profileSaving = $state(false);

  async function loadProfileVisibility() {
    profileError = '';
    try {
      const res = await getProfileVisibility();
      if (res.err === 0) {
        if (res.profile !== undefined) profileVis.profile = res.profile;
        if (res.works !== undefined) profileVis.works = res.works;
        if (res.reading_history !== undefined) profileVis.reading_history = res.reading_history;
      }
    } catch {
      // Fall back to localStorage cached preference or defaults.
      try {
        const raw = localStorage.getItem('fichub_profile_visibility');
        if (raw) {
          const cached = JSON.parse(raw);
          profileVis = { ...profileVis, ...cached };
        }
      } catch { /* ignore */ }
    }
  }

  async function saveProfileVisibility() {
    profileSaving = true;
    profileSaved = '';
    profileError = '';
    const value: ProfileVisibility = { ...profileVis };
    try {
      const res = await setProfileVisibility(value);
      if (res.err === 0) {
        profileSaved = t('settings.profileSaved');
        try {
          localStorage.setItem('fichub_profile_visibility', JSON.stringify(value));
        } catch { /* ignore */ }
      } else {
        profileError = t('settings.profileError');
      }
    } catch {
      // Optimistically persist locally when backend is absent.
      try {
        localStorage.setItem('fichub_profile_visibility', JSON.stringify(value));
      } catch { /* ignore */ }
      profileSaved = t('settings.profileSaved');
    } finally {
      profileSaving = false;
    }
  }

  // ── Time zone ──────────────────────────────────────────────────────
  let userTimezone = $state(getPref('timezone') || '');
  let timezoneSaved = $state('');
  let timezoneError = $state('');
  let timezoneSaving = $state(false);

  async function loadTimezone() {
    timezoneError = '';
    try {
      const res = await getUserTimezone();
      if (res.err === 0 && res.timezone) {
        userTimezone = res.timezone;
        setPref('timezone', res.timezone);
      } else if (!userTimezone) {
        // Fall back to browser-detected zone.
        const browser = Intl.DateTimeFormat().resolvedOptions().timeZone;
        if (browser) {
          userTimezone = browser;
          setPref('timezone', browser);
        }
      }
    } catch {
      if (!userTimezone) {
        const browser = Intl.DateTimeFormat().resolvedOptions().timeZone;
        if (browser) userTimezone = browser;
      }
    }
  }

  function useBrowserTimezone() {
    const browser = Intl.DateTimeFormat().resolvedOptions().timeZone;
    if (browser) {
      userTimezone = browser;
      saveTimezone();
    }
  }

  async function saveTimezone() {
    timezoneSaving = true;
    timezoneSaved = '';
    timezoneError = '';
    const tz = userTimezone;
    try {
      const res = await setUserTimezone(tz);
      if (res.err === 0) {
        timezoneSaved = t('settings.timezoneSaved');
        setPref('timezone', tz);
      } else {
        timezoneError = t('settings.timezoneError');
      }
    } catch {
      // Persist locally when backend is absent.
      setPref('timezone', tz);
      try {
        localStorage.setItem('fichub_timezone', tz);
      } catch { /* ignore */ }
      timezoneSaved = t('settings.timezoneSaved');
    } finally {
      timezoneSaving = false;
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

  {#if uiMode === 'archive'}
    <header class="archive-header">
      <h1 class="archive-title">{t('settings.title')}</h1>
      <blockquote class="archive-summary">
        Manage your FicNexus account, profile visibility, credentials, notification preferences and more.
      </blockquote>
    </header>
  {/if}

  {#if !auth.isLoggedIn}
    {#if isArchiveMode}
      <div class="archive-notice"><p>{t('settings.loginToManage')}</p></div>
    {:else}
      <div class="card empty">
        <p class="muted">{t('settings.loginToManage')}</p>
      </div>
    {/if}
  {:else if loading}
    {#if isArchiveMode}
      <div class="archive-notice"><p><span class="spinner"></span> {t('common.loading')}</p></div>
    {:else}
      <div class="card">
        <p class="muted"><span class="spinner"></span> {t('common.loading')}</p>
      </div>
    {/if}
  {:else}
    {#if error}
      {#if isArchiveMode}
        <div class="archive-error"><strong>{error}</strong></div>
      {:else}
        <div class="card error-card"><p class="error-text">{error}</p></div>
      {/if}
    {/if}
    {#if saved}
      {#if isArchiveMode}
        <p class="archive-success">{saved}</p>
      {:else}
        <div class="card success-card"><p>{saved}</p></div>
      {/if}
    {/if}

    <!-- ── Archive Skin Selector ───────────────────────────────────── -->
    {#if uiMode === 'archive'}
      <fieldset class="archive-fieldset">
        <legend>{t('settings.skinTitle')}</legend>
        <blockquote class="archive-summary">
          {t('settings.skinDesc')}
        </blockquote>
        <div class="archive-ui-options">
          <label class="archive-ui-option">
            <input type="radio" name="skin" checked={archiveSkin === 'zerafina'} onchange={() => setArchiveSkin('zerafina')} />
            <span class="archive-ui-option-title">{t('settings.skinZerafina')}</span>
            <span class="archive-muted archive-ui-option-desc">Full AO3-style styling with enhanced stats icons and tag boxes</span>
          </label>
          <label class="archive-ui-option">
            <input type="radio" name="skin" checked={archiveSkin === 'ao3'} onchange={() => setArchiveSkin('ao3')} />
            <span class="archive-ui-option-title">{t('settings.skinAo3')}</span>
            <span class="archive-muted archive-ui-option-desc">Minimal AO3 default header styling</span>
          </label>
        </div>
        {#if skinSaved}
          <p class="archive-success">{skinSaved}</p>
        {/if}
      </fieldset>
    {:else}
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

    <!-- ── Interface Style ─────────────────────────────────────────── -->
    {#if uiMode === 'archive'}
      <fieldset class="archive-fieldset">
        <legend>Interface Style</legend>
        <blockquote class="archive-summary">
          Choose the look and feel of FicNexus.
        </blockquote>
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

    <!-- ── F7: level badge + exp progress ─────────────────────────── -->
    {#if isArchiveMode}
      <fieldset class="archive-fieldset">
        <legend>{t('settings.levelTitle')}</legend>
        <dl class="archive-dl archive-stats-dl">
          <dt>Level</dt><dd>{level ? level.level : '—'}</dd>
          <dt>XP</dt><dd>{level ? level.exp : '—'}</dd>
          <dt>To next</dt><dd>{level ? level.exp_to_next : '—'}</dd>
        </dl>
        {#if level}
          <p class="archive-pref-blurb">
            <span class="archive-level-badge">{t('settings.level', { level: level.level })}</span>
            <span class="archive-muted">{level.exp} {t('settings.exp')}</span>
          </p>
          {#if level.level >= 100}
            <p class="archive-muted">{t('settings.levelMax')}</p>
          {:else}
            <dl class="archive-dl">
              <dt>{t('settings.expProgressAria')}</dt>
              <dd>
                <div
                  class="archive-progress-track"
                  role="progressbar"
                  aria-valuenow={Math.round(level.progress * 100)}
                  aria-valuemin={0}
                  aria-valuemax={100}
                >
                  <div class="archive-progress-fill" style="width: {Math.round(level.progress * 100)}%"></div>
                </div>
                <span class="archive-muted">{t('settings.expToNext', { exp: level.exp_to_next })}</span>
              </dd>
            </dl>
          {/if}
        {:else if levelError}
          <p class="archive-error">{levelError}</p>
        {:else}
          <p class="archive-muted"><span class="spinner"></span> {t('common.loading')}</p>
        {/if}
      </fieldset>
    {:else}
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
    {/if}

    {#if isArchiveMode}
      <fieldset class="archive-fieldset">
        <legend>{t('settings.credentialsTitle')}</legend>
        <blockquote class="archive-summary">
          {t('settings.credentialsIntro')} <strong>{t('settings.credentialsExpiry')}</strong> {t('settings.credentialsTail')}
        </blockquote>
        <ul class="archive-pref-list">
          {#each LOGIN_SITES as site (site.domain)}
            {@const cred = credFor(site.domain)}
            <li class="archive-pref-item">
              <span class="archive-pref-label">{site.name} <span class="archive-muted">({site.domain})</span></span>
              {#if cred}
                <span class="archive-muted">Username: <strong>{cred.username}</strong> · expires {formatDate(cred.expires_at)}</span>
                <button
                  class="archive-btn"
                  onclick={() => handleRemove(site.domain)}
                  disabled={removing === site.domain}
                >
                  {#if removing === site.domain}<span class="spinner"></span>{:else}Remove{/if}
                </button>
              {:else}
                {@const f = formState(site.domain)}
                <dl class="archive-dl archive-cred-form">
                  <dt>Username</dt>
                  <dd><input type="text" bind:value={f.username} autocomplete="username" placeholder="your username on {site.name}" /></dd>
                  <dt>Password</dt>
                  <dd><input type="password" bind:value={f.password} autocomplete="current-password" placeholder="••••••••" /></dd>
                  <dt class="archive-consent-row" style="grid-column: 1 / -1;">
                    <label class="archive-checkbox-label">
                      <input type="checkbox" bind:checked={f.consent} />
                      I consent to storing these credentials for 30 days
                    </label>
                  </dt>
                  <dd style="grid-column: 1 / -1;">
                    <button class="archive-btn archive-btn-primary" onclick={() => handleSave(site.domain)} disabled={f.saving}>
                      {#if f.saving}<span class="spinner"></span>{:else}Save{/if}
                    </button>
                  </dd>
                </dl>
              {/if}
            </li>
          {/each}
        </ul>
      </fieldset>
    {:else}
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
    {/if}

    <!-- ── Recommendations ─────────────────────────────────────────── -->
    {#if isArchiveMode}
      <fieldset class="archive-fieldset">
        <legend>{t('settings.recsTitle')}</legend>
        <dl class="archive-dl">
          <dt>{t('recs.personalizedToggle')}</dt>
          <dd>
            <label class="archive-checkbox-label">
              <input type="checkbox" bind:checked={personalizedRecs} onchange={togglePersonalizedRecs} disabled={recsSaving} />
              {t('recs.personalizedToggleDesc')}
            </label>
          </dd>
        </dl>
      </fieldset>
    {:else}
      <section class="card">
        <h2>{t('settings.recsTitle')}</h2>
        <label class="toggle-row">
          <span>
            <strong>{t('recs.personalizedToggle')}</strong>
            <span class="muted">{t('recs.personalizedToggleDesc')}</span>
          </span>
          <input
            type="checkbox"
            bind:checked={personalizedRecs}
            onchange={togglePersonalizedRecs}
            disabled={recsSaving}
          />
        </label>
      </section>
    {/if}

    <!-- ── Notification Preferences ────────────────────────────────────────── -->
    {#if isArchiveMode}
      <fieldset class="archive-fieldset">
        <legend>{t('settings.notificationsTitle')}</legend>
        {#if !auth.isLoggedIn}
          <p class="archive-muted">{t('settings.loginToManage')}</p>
        {:else}
          {#if notifError}
            <p class="archive-error">{notifError}</p>
          {/if}
          {#if notifSaved}
            <p class="archive-success">{notifSaved}</p>
          {/if}
          <dl class="archive-dl">
            <div class="dl-row dl-row-group">
              <dt class="dl-group-label">{t('settings.notifGroupWork')}</dt>
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
              <dt class="dl-group-label">{t('settings.notifGroupComments')}</dt>
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
              <dt class="dl-group-label">{t('settings.notifGroupSocial')}</dt>
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
              <dt class="dl-group-label">{t('settings.notifGroupRecognition')}</dt>
            </div>
            <div class="dl-row">
              <dt>{t('settings.notifBadgeEarned')}</dt>
              <dd><input type="checkbox" bind:checked={notifPrefs.badge_earned} onchange={saveNotifPrefs} /></dd>
            </div>
            <div class="dl-row">
              <dt>{t('settings.notifCuratorPromotion')}</dt>
              <dd><input type="checkbox" bind:checked={notifPrefs.curator_promotion} onchange={saveNotifPrefs} /></dd>
            </div>

            <div class="dl-row dl-row-group">
              <dt class="dl-group-label">{t('settings.notifGroupEmail')}</dt>
            </div>
            <div class="dl-row">
              <dt>{t('settings.notifDigest')}</dt>
              <dd>
                <select bind:value={notifPrefs.email_digest} onchange={saveNotifPrefs}>
                  <option value="none">{t('settings.emailDigestNone')}</option>
                  <option value="daily">{t('settings.emailDigestDaily')}</option>
                  <option value="weekly">{t('settings.emailDigestWeekly')}</option>
                </select>
              </dd>
            </div>
          </dl>
        {/if}
      </fieldset>
    {:else}
      <section class="card">
        <h2>{t('settings.notificationsTitle')}</h2>
        {#if !auth.isLoggedIn}
          <p class="muted">{t('settings.loginToManage')}</p>
        {:else}
          {#if notifError}
            <p class="error-text">{notifError}</p>
          {/if}
          {#if notifSaved}
            <p class="success-text">{notifSaved}</p>
          {/if}
          <dl class="archive-dl">
            <div class="dl-row dl-row-group">
              <dt class="dl-group-label">{t('settings.notifGroupWork')}</dt>
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
              <dt class="dl-group-label">{t('settings.notifGroupComments')}</dt>
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
              <dt class="dl-group-label">{t('settings.notifGroupSocial')}</dt>
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
              <dt class="dl-group-label">{t('settings.notifGroupRecognition')}</dt>
            </div>
            <div class="dl-row">
              <dt>{t('settings.notifBadgeEarned')}</dt>
              <dd><input type="checkbox" bind:checked={notifPrefs.badge_earned} onchange={saveNotifPrefs} /></dd>
            </div>
            <div class="dl-row">
              <dt>{t('settings.notifCuratorPromotion')}</dt>
              <dd><input type="checkbox" bind:checked={notifPrefs.curator_promotion} onchange={saveNotifPrefs} /></dd>
            </div>

            <div class="dl-row dl-row-group">
              <dt class="dl-group-label">{t('settings.notifGroupEmail')}</dt>
            </div>
            <div class="dl-row">
              <dt>{t('settings.notifDigest')}</dt>
              <dd>
                <select bind:value={notifPrefs.email_digest} onchange={saveNotifPrefs}>
                  <option value="none">{t('settings.emailDigestNone')}</option>
                  <option value="daily">{t('settings.emailDigestDaily')}</option>
                  <option value="weekly">{t('settings.emailDigestWeekly')}</option>
                </select>
              </dd>
            </div>
          </dl>
        {/if}
      </section>
    {/if}

    <!-- ── Blacklist Management ─────────────────────────────────────────→ -->
    {#if isArchiveMode}
      <fieldset class="archive-fieldset">
        <legend>{t('settings.blacklistTitle')}</legend>
        <blockquote class="archive-summary">{t('settings.blacklistIntro')}</blockquote>
        {#if blacklistError}
          <p class="archive-error">{blacklistError}</p>
        {/if}
        {#if blacklistSaved}
          <p class="archive-success">{blacklistSaved}</p>
        {/if}
        <dl class="archive-dl">
          <dt>{t('settings.blacklistAddPlaceholder')}</dt>
          <dd class="archive-inline-form">
            <input type="text" bind:value={newBlockedName} placeholder={t('settings.blacklistAddPlaceholder')} onkeypress={(e) => e.key === 'Enter' && addBlockedTag()} />
            <select bind:value={newBlockedType}>
              <option value="fandom">{t('settings.blacklistTypeFandom')}</option>
              <option value="character">{t('settings.blacklistTypeCharacter')}</option>
              <option value="relationship">{t('settings.blacklistTypeRelationship')}</option>
              <option value="freeform">{t('settings.blacklistTypeFreeform')}</option>
              <option value="warning">{t('settings.blacklistTypeWarning')}</option>
              <option value="category">{t('settings.blacklistTypeCategory')}</option>
            </select>
            <button class="archive-btn archive-btn-primary" onclick={addBlockedTag}>
              {t('settings.blacklistAdd')}
            </button>
          </dd>
        </dl>
        {#if blacklistLoading}
          <p class="archive-muted"><span class="spinner"></span> {t('common.loading')}</p>
        {:else if blacklist.length === 0}
          <p class="archive-muted">{t('settings.blacklistEmpty')}</p>
        {:else}
          <ul class="archive-pref-list">
            {#each blacklist as item (item.id ?? item.name)}
              <li class="archive-pref-item">
                <span>{item.name} <span class="archive-muted">({item.type})</span></span>
                <button class="archive-btn" onclick={() => removeBlockedTag(item)} disabled={blacklistRemovingId === (item.id ?? null)}>
                  {#if blacklistRemovingId === (item.id ?? null)}<span class="spinner"></span>{:else}{t('settings.blacklistRemove')}{/if}
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </fieldset>
    {:else}
      <section class="card">
        <h2>{t('settings.blacklistTitle')}</h2>
        <p class="muted">{t('settings.blacklistIntro')}</p>
        {#if blacklistError}
          <p class="error-text">{blacklistError}</p>
        {/if}
        {#if blacklistSaved}
          <p class="success-text">{blacklistSaved}</p>
        {/if}
        <div class="blacklist-add-form">
          <input
            type="text"
            bind:value={newBlockedName}
            placeholder={t('settings.blacklistAddPlaceholder')}
            onkeypress={(e) => e.key === 'Enter' && addBlockedTag()}
          />
          <select bind:value={newBlockedType}>
            <option value="fandom">{t('settings.blacklistTypeFandom')}</option>
            <option value="character">{t('settings.blacklistTypeCharacter')}</option>
            <option value="relationship">{t('settings.blacklistTypeRelationship')}</option>
            <option value="freeform">{t('settings.blacklistTypeFreeform')}</option>
            <option value="warning">{t('settings.blacklistTypeWarning')}</option>
            <option value="category">{t('settings.blacklistTypeCategory')}</option>
          </select>
          <button class="btn btn-primary sm" onclick={addBlockedTag}>
            {t('settings.blacklistAdd')}
          </button>
        </div>
        {#if blacklistLoading}
          <p class="muted"><span class="spinner"></span> {t('common.loading')}</p>
        {:else if blacklist.length === 0}
          <p class="muted">{t('settings.blacklistEmpty')}</p>
        {:else}
          <div class="blacklist-tags">
            {#each blacklist as item (item.id ?? item.name)}
              <div class="blacklist-tag-item">
                <span class="tag-name">{item.name}</span>
                <span class="tag-type muted">{item.type}</span>
                <button
                  class="btn btn-secondary sm"
                  onclick={() => removeBlockedTag(item)}
                  disabled={blacklistRemovingId === (item.id ?? null)}
                >
                  {#if blacklistRemovingId === (item.id ?? null)}<span class="spinner"></span>{:else}{t('settings.blacklistRemove')}{/if}
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </section>
    {/if}

    <!-- ── Profile Display ──────────────────────────────────────────────→ -->
    {#if isArchiveMode}
      <fieldset class="archive-fieldset">
        <legend>{t('settings.profileDisplayTitle')}</legend>
        <blockquote class="archive-summary">{t('settings.profileDisplayIntro')}</blockquote>
        {#if profileError}
          <p class="archive-error">{profileError}</p>
        {/if}
        {#if profileSaved}
          <p class="archive-success">{profileSaved}</p>
        {/if}
        <dl class="archive-dl">
          <dt>{t('settings.profileVisibility')}</dt>
          <dd>
            <select bind:value={profileVis.profile} onchange={saveProfileVisibility}>
              <option value="public">{t('settings.visibilityPublic')}</option>
              <option value="followers">{t('settings.visibilityFollowers')}</option>
              <option value="private">{t('settings.visibilityPrivate')}</option>
            </select>
          </dd>
          <dt>{t('settings.worksVisibility')}</dt>
          <dd>
            <select bind:value={profileVis.works} onchange={saveProfileVisibility}>
              <option value="public">{t('settings.visibilityPublic')}</option>
              <option value="followers">{t('settings.visibilityFollowers')}</option>
              <option value="private">{t('settings.visibilityPrivate')}</option>
            </select>
          </dd>
          <dt>{t('settings.historyVisibility')}</dt>
          <dd>
            <select bind:value={profileVis.reading_history} onchange={saveProfileVisibility}>
              <option value="public">{t('settings.visibilityPublic')}</option>
              <option value="followers">{t('settings.visibilityFollowers')}</option>
              <option value="private">{t('settings.visibilityPrivate')}</option>
            </select>
          </dd>
        </dl>
      </fieldset>
    {:else}
      <section class="card">
        <h2>{t('settings.profileDisplayTitle')}</h2>
        <p class="muted">{t('settings.profileDisplayIntro')}</p>
        {#if profileError}
          <p class="error-text">{profileError}</p>
        {/if}
        {#if profileSaved}
          <p class="success-text">{profileSaved}</p>
        {/if}
        <dl class="archive-dl">
          <div class="dl-row archive-dl-row">
            <dt>{t('settings.profileVisibility')}</dt>
            <dd>
              <select bind:value={profileVis.profile} onchange={saveProfileVisibility}>
                <option value="public">{t('settings.visibilityPublic')}</option>
                <option value="followers">{t('settings.visibilityFollowers')}</option>
                <option value="private">{t('settings.visibilityPrivate')}</option>
              </select>
            </dd>
          </div>
          <div class="dl-row">
            <dt>{t('settings.worksVisibility')}</dt>
            <dd>
              <select bind:value={profileVis.works} onchange={saveProfileVisibility}>
                <option value="public">{t('settings.visibilityPublic')}</option>
                <option value="followers">{t('settings.visibilityFollowers')}</option>
                <option value="private">{t('settings.visibilityPrivate')}</option>
              </select>
            </dd>
          </div>
          <div class="dl-row">
            <dt>{t('settings.historyVisibility')}</dt>
            <dd>
              <select bind:value={profileVis.reading_history} onchange={saveProfileVisibility}>
                <option value="public">{t('settings.visibilityPublic')}</option>
                <option value="followers">{t('settings.visibilityFollowers')}</option>
                <option value="private">{t('settings.visibilityPrivate')}</option>
              </select>
            </dd>
          </div>
        </dl>
      </section>
    {/if}

    <!-- ── Time Zone ────────────────────────────────────────────────────→ -->
    {#if isArchiveMode}
      <fieldset class="archive-fieldset">
        <legend>{t('settings.timezoneTitle')}</legend>
        <blockquote class="archive-summary">{t('settings.timezoneIntro')}</blockquote>
        {#if timezoneError}
          <p class="archive-error">{timezoneError}</p>
        {/if}
        {#if timezoneSaved}
          <p class="archive-success">{timezoneSaved}</p>
        {/if}
        <dl class="archive-dl">
          <dt>{t('settings.timezoneAuto')}</dt>
          <dd>
            <button class="archive-btn" onclick={useBrowserTimezone}>
              {t('settings.timezoneAuto')}
            </button>
          </dd>
          <dt>{t('settings.timezoneCustom')}</dt>
          <dd>
            <select bind:value={userTimezone} onchange={saveTimezone}>
              {#each TIME_ZONES as tz}
                <option value={tz}>{tz}</option>
              {/each}
            </select>
          </dd>
        </dl>
      </fieldset>
    {:else}
      <section class="card">
        <h2>{t('settings.timezoneTitle')}</h2>
        <p class="muted">{t('settings.timezoneIntro')}</p>
        {#if timezoneError}
          <p class="error-text">{timezoneError}</p>
        {/if}
        {#if timezoneSaved}
          <p class="success-text">{timezoneSaved}</p>
        {/if}
        <dl class="archive-dl">
          <div class="dl-row">
            <dt>{t('settings.timezoneAuto')}</dt>
            <dd>
              <button class="btn btn-secondary sm" onclick={useBrowserTimezone}>
                {t('settings.timezoneAuto')}
              </button>
            </dd>
          </div>
          <div class="dl-row">
            <dt>{t('settings.timezoneCustom')}</dt>
            <dd>
              <select bind:value={userTimezone} onchange={saveTimezone}>
                {#each TIME_ZONES as tz}
                  <option value={tz}>{tz}</option>
                {/each}
              </select>
            </dd>
          </div>
        </dl>
      </section>
    {/if}
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

  /* Archive buttons and preference list (settings-specific supplements) */
  .archive-btn {
    padding: 0.3rem 0.8rem;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-text, #2a2a2a);
    font-family: inherit;
    font-size: 0.88rem;
    cursor: pointer;
  }
  .archive-btn:hover { background: var(--archive-border, #dddddd); }
  .archive-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .archive-btn-primary {
    color: var(--archive-bg, #ffffff);
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }
  .archive-btn-primary:hover { background: var(--archive-link-visited, #660066); border-color: var(--archive-link-visited, #660066); }
  .archive-notice {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.6rem 0.9rem;
    background: var(--archive-bg-raised, #fafafa);
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-error { color: var(--archive-heading, #990000); font-weight: 700; font-size: 0.9rem; }
  .archive-success { color: #326342; font-weight: 700; font-size: 0.9rem; }
  .archive-summary {
    margin: 0 0 0.6rem;
    padding: 0.3rem 0 0.3rem 0.6rem;
    border-left: 2px solid var(--archive-border, #dddddd);
    color: var(--archive-muted, #666666);
    font-style: italic;
    font-size: 0.88rem;
  }
  .archive-pref-list {
    list-style: none;
    margin: 0.5rem 0 0;
    padding: 0;
  }
  .archive-pref-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.5rem 0;
    border-bottom: 1px solid var(--archive-border, #eeeeee);
    font-size: 0.9rem;
  }
  .archive-pref-item:last-child { border-bottom: none; }
  .archive-pref-label { font-weight: 600; }
  .archive-checkbox-label {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    cursor: pointer;
    font-size: 0.9rem;
  }
  .archive-inline-form {
    display: flex;
    gap: 0.4rem;
    align-items: center;
    flex-wrap: wrap;
  }
  .archive-inline-form input[type='text'] {
    padding: 0.35rem 0.5rem;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.9rem;
  }
  .archive-inline-form select {
    padding: 0.35rem 0.5rem;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.9rem;
  }
  .archive-stats-dl {
    display: grid;
    grid-template-columns: auto 1fr auto 1fr auto 1fr;
    gap: 0.3rem 1rem;
    margin: 0.5rem 0;
  }
  .archive-progress-track {
    height: 0.55rem;
    background: var(--archive-bg-raised, #eeeeee);
    border: 1px solid var(--archive-border, #dddddd);
    overflow: hidden;
    margin: 0.3rem 0;
  }
  .archive-progress-fill {
    height: 100%;
    background: var(--archive-link, #990000);
    transition: width 0.3s ease;
  }
  .archive-level-badge {
    display: inline-block;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
    padding: 0.15rem 0.5rem;
    font-weight: 700;
    font-size: 0.88rem;
  }
  .archive-pref-blurb { margin: 0.4rem 0; display: flex; gap: 0.5rem; align-items: center; }
  .archive-cred-form { margin-top: 0.4rem; }
  .archive-cred-form input {
    width: 100%;
    box-sizing: border-box;
    padding: 0.35rem 0.5rem;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.9rem;
  }
  .archive-cred-form input[type='checkbox'] { width: auto; }

</style>
