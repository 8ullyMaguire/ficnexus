<script lang="ts">
  import { auth } from '$lib/stores/auth.svelte';
  import { onMount } from 'svelte';
  import { getSiteInfo, type RegistrationMode } from '$lib/api/forum';
  import { t } from '$lib/i18n/index.svelte';

  let searchQuery = $state('');

  // ── Accessible dropdown layer (click/touch + keyboard; hover stays CSS) ──
  let openDropdown = $state<string | null>(null);
  let headerEl: HTMLElement | undefined = $state(undefined);

  function toggleDropdown(id: string, e?: Event) {
    e?.preventDefault();
    openDropdown = openDropdown === id ? null : id;
  }

  function closeAll() {
    openDropdown = null;
  }

  function onGlobalKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') closeAll();
  }

  function onGlobalClick(e: MouseEvent) {
    if (headerEl && !headerEl.contains(e.target as Node)) closeAll();
  }

  onMount(() => {
    document.addEventListener('keydown', onGlobalKeydown);
    document.addEventListener('click', onGlobalClick);
    return () => {
      document.removeEventListener('keydown', onGlobalKeydown);
      document.removeEventListener('click', onGlobalClick);
    };
  });

  // F7: site registration mode (open | invite | application). Drives the
  // register form fields and the inline hint, mirroring AuthBar.
  let regMode = $state<RegistrationMode | null>(null);

  onMount(async () => {
    try {
      const res = await getSiteInfo();
      if (res.err === 0 && res.site) {
        regMode = res.site.registration_mode;
      }
    } catch {
      /* site info unavailable — hide invite-only hint, keep register open */
    }
  });

  let loginName = $state('');
  let loginPass = $state('');
  let loginRemember = $state(false);
  let loginError = $state('');

  // Register form state (reuse auth store handler)
  let regUser = $state('');
  let regPass = $state('');
  let regEmail = $state('');
  let regInvite = $state('');
  let regError = $state('');

  function handleSearch(e: Event) {
    e.preventDefault();
    if (searchQuery.trim()) {
      window.location.href = `/search?q=${encodeURIComponent(searchQuery.trim())}`;
    }
  }

  async function handleArchiveLogin() {
    loginError = '';
    const ok = await auth.handleLogin(loginName.trim(), loginPass, loginRemember);
    if (ok) {
      loginName = '';
      loginPass = '';
      loginRemember = false;
    } else {
      loginError = 'The password or user name are incorrect.';
    }
  }

  async function handleArchiveRegister(e: Event) {
    e.preventDefault();
    regError = '';
    const code = regMode === 'invite' ? regInvite.trim() : undefined;
    const ok = await auth.handleRegister(regUser.trim(), regPass, regEmail || undefined, code);
    if (ok) {
      regUser = '';
      regPass = '';
      regEmail = '';
      regInvite = '';
      closeAll();
    } else {
      regError = t('auth.registerFailed');
    }
  }
</script>

<header class="archive-header" bind:this={headerEl}>
  <!-- ROW 1: Logo + user area -->
  <div class="archive-top-row">
    <div class="archive-top-inner">
      <a href="/" class="archive-brand" aria-label={t('nav.brandAria')}>
        FicNexus<sup class="archive-brand-sup">archive</sup>
      </a>

      <div class="archive-top-right">
        {#if auth.isLoggedIn}
          <div class="archive-user-dropdown" class:open={openDropdown === 'user'}>
            <a class="archive-top-link archive-dropdown-trigger" role="button" tabindex="0" aria-haspopup="true" aria-expanded={openDropdown === 'user'} onclick={(e) => toggleDropdown('user', e)} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') toggleDropdown('user', e); }}>
              Hi, {auth.username ?? 'there'}!
            </a>
            <div class="archive-user-menu" role="menu">
              <a class="archive-dd-item" href="/dashboard" role="menuitem">Dashboard</a>
              <a class="archive-dd-item" href="/my-works" role="menuitem">My Works</a>
              <a class="archive-dd-item" href="/bookmarks" role="menuitem">Bookmarks</a>
              <a class="archive-dd-item" href="/collections" role="menuitem">Collections</a>
              <a class="archive-dd-item" href="/history" role="menuitem">Reading History</a>
              <a class="archive-dd-item" href="/notifications" role="menuitem">Notifications</a>
              <a class="archive-dd-item" href="/settings" role="menuitem">Preferences</a>
            </div>
          </div>
          <div class="archive-dropdown" class:open={openDropdown === 'post'}>
            <a class="archive-top-link archive-dropdown-trigger" role="button" tabindex="0" aria-haspopup="true" aria-expanded={openDropdown === 'post'} onclick={(e) => toggleDropdown('post', e)} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') toggleDropdown('post', e); }}>Post</a>
            <div class="archive-dropdown-menu archive-dropdown-menu-right" role="menu">
              <a class="archive-dd-item" href="/work-proposals/new" role="menuitem">New Work</a>
              <a class="archive-dd-item" href="/upload" role="menuitem">Import Work</a>
            </div>
          </div>
          <a class="archive-top-link" href="/logout" onclick={(e) => { e.preventDefault(); auth.handleLogout(); }}>Log Out</a>
        {:else}
          <div id="login" class="archive-login-dropdown" class:open={openDropdown === 'login'}>
            <a class="archive-top-link archive-dropdown-trigger" role="button" tabindex="0" aria-haspopup="true" aria-expanded={openDropdown === 'login'} onclick={(e) => toggleDropdown('login', e)} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') toggleDropdown('login', e); }}>Log In</a>
            <div class="archive-login-menu">
              <form
                class="archive-login-form"
                onsubmit={(e) => { e.preventDefault(); handleArchiveLogin(); }}
              >
                <p class="archive-login-heading">Log In</p>
                {#if loginError}<p class="archive-login-error">{loginError}</p>{/if}
                <label for="archive-login-user">Username or email:</label>
                <input id="archive-login-user" type="text" name="user_login" bind:value={loginName} autocomplete="username" />
                <label for="archive-login-pass">Password:</label>
                <input id="archive-login-pass" type="password" name="user_password" bind:value={loginPass} autocomplete="current-password" />
                <div class="archive-login-row">
                  <label class="archive-login-remember">
                    <input type="checkbox" name="user_remember_me" bind:checked={loginRemember} />
                    Remember Me
                  </label>
                  <button class="archive-btn" type="submit" disabled={auth.loading}>
                    {auth.loading ? 'Logging in…' : 'Log In'}
                  </button>
                </div>
              </form>
            </div>
          </div>
          <div id="register" class="archive-login-dropdown" class:open={openDropdown === 'register'}>
            <a class="archive-top-link archive-dropdown-trigger" role="button" tabindex="0" aria-haspopup="true" aria-expanded={openDropdown === 'register'} onclick={(e) => toggleDropdown('register', e)} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') toggleDropdown('register', e); }}>Register</a>
            <div class="archive-register-menu">
              {#if regMode && regMode !== 'open'}
                <p class="archive-reg-hint">
                  {#if regMode === 'invite'}
                    Registration is by invitation only.
                  {:else if regMode === 'application'}
                    Open registration is disabled — <a href="/forum/apply">apply for an account</a>.
                  {/if}
                </p>
              {/if}
              {#if regMode !== 'application'}
                <form
                  class="archive-login-form"
                  onsubmit={handleArchiveRegister}
                >
                  <p class="archive-login-heading">Register</p>
                  {#if regError}<p class="archive-login-error">{regError}</p>{/if}
                  <label for="archive-reg-user">Username:</label>
                  <input id="archive-reg-user" type="text" name="username" bind:value={regUser} autocomplete="username" />
                  <label for="archive-reg-pass">Password:</label>
                  <input id="archive-reg-pass" type="password" name="password" bind:value={regPass} autocomplete="new-password" />
                  <label for="archive-reg-email">Email (optional):</label>
                  <input id="archive-reg-email" type="email" name="email" bind:value={regEmail} autocomplete="email" />
                  {#if regMode === 'invite'}
                    <label for="archive-reg-invite">Invite code:</label>
                    <input id="archive-reg-invite" type="text" name="invite_code" bind:value={regInvite} />
                  {/if}
                  <div class="archive-login-row">
                    <button class="archive-btn" type="submit" disabled={auth.loading}>
                      {auth.loading ? 'Registering…' : 'Register'}
                    </button>
                  </div>
                </form>
              {/if}
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>

  <!-- ROW 2: Red navbar -->
  <nav class="archive-navbar" aria-label="Primary navigation">
    <div class="archive-navbar-inner">
      <div class="archive-nav-left">
        <a class="archive-nav-link" href="/fandoms">Fandoms</a>

        <div class="archive-dropdown" class:open={openDropdown === 'community'}>
          <a class="archive-nav-link archive-dropdown-trigger" role="button" tabindex="0" aria-haspopup="true" aria-expanded={openDropdown === 'community'} onclick={(e) => toggleDropdown('community', e)} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') toggleDropdown('community', e); }}>Community</a>
          <div class="archive-dropdown-menu" role="menu">
            <a class="archive-dd-item" href="/requests" role="menuitem">Requests</a>
            <a class="archive-dd-item" href="/forum" role="menuitem">Forum</a>
            <a class="archive-dd-item" href="/ask" role="menuitem">Ask</a>
          </div>
        </div>

        <div class="archive-dropdown" class:open={openDropdown === 'browse'}>
          <a class="archive-nav-link archive-dropdown-trigger" role="button" tabindex="0" aria-haspopup="true" aria-expanded={openDropdown === 'browse'} onclick={(e) => toggleDropdown('browse', e)} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') toggleDropdown('browse', e); }}>Browse</a>
          <div class="archive-dropdown-menu" role="menu">
            <a class="archive-dd-item" href="/trending" role="menuitem">Trending</a>
            <a class="archive-dd-item" href="/leaderboard" role="menuitem">Leaderboard</a>
            <a class="archive-dd-item" href="/bookmarks" role="menuitem">Bookmarks</a>
            <a class="archive-dd-item" href="/tags" role="menuitem">Tags</a>
            <a class="archive-dd-item" href="/collections" role="menuitem">Collections</a>
          </div>
        </div>

        {#if auth.level >= 10}
          <div class="archive-dropdown" class:open={openDropdown === 'curator'}>
            <a class="archive-nav-link archive-dropdown-trigger" role="button" tabindex="0" aria-haspopup="true" aria-expanded={openDropdown === 'curator'} onclick={(e) => toggleDropdown('curator', e)} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') toggleDropdown('curator', e); }}>Curator</a>
            <div class="archive-dropdown-menu" role="menu">
              <a class="archive-dd-item" href="/curator/flags" role="menuitem">Flags</a>
              <a class="archive-dd-item" href="/curator/proposals" role="menuitem">Queue</a>
              <a class="archive-dd-item" href="/curator/coverage" role="menuitem">Coverage</a>
              <a class="archive-dd-item" href="/work-proposals" role="menuitem">Proposals</a>
              <a class="archive-dd-item" href="/curator/authors" role="menuitem">Authors</a>
            </div>
          </div>
        {/if}

        {#if auth.level >= 100}
          <div class="archive-dropdown" class:open={openDropdown === 'admin'}>
            <a class="archive-nav-link archive-dropdown-trigger" role="button" tabindex="0" aria-haspopup="true" aria-expanded={openDropdown === 'admin'} onclick={(e) => toggleDropdown('admin', e)} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') toggleDropdown('admin', e); }}>Admin</a>
            <div class="archive-dropdown-menu" role="menu">
              <a class="archive-dd-item" href="/admin" role="menuitem">Dashboard</a>
              <a class="archive-dd-item" href="/admin/moderation" role="menuitem">Moderation</a>
              <a class="archive-dd-item" href="/admin/users" role="menuitem">Users</a>
              <a class="archive-dd-item" href="/admin/scrapers" role="menuitem">Scrapers</a>
              <a class="archive-dd-item" href="/admin/stats" role="menuitem">Stats</a>
              <a class="archive-dd-item" href="/admin/auto-tag" role="menuitem">Auto-Tag</a>
              <a class="archive-dd-item" href="/admin/bulk-actions" role="menuitem">Bulk Actions</a>
              <a class="archive-dd-item" href="/admin/blacklist" role="menuitem">Blacklist</a>
            </div>
          </div>
        {/if}

        <div class="archive-dropdown" class:open={openDropdown === 'search'}>
          <a class="archive-nav-link archive-dropdown-trigger" role="button" tabindex="0" aria-haspopup="true" aria-expanded={openDropdown === 'search'} onclick={(e) => toggleDropdown('search', e)} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') toggleDropdown('search', e); }}>Search</a>
          <div class="archive-dropdown-menu" role="menu">
            <a class="archive-dd-item" href="/search" role="menuitem">Works</a>
            <a class="archive-dd-item" href="/bookmarks?tab=search" role="menuitem">Bookmarks</a>
            <a class="archive-dd-item" href="/tags" role="menuitem">Tags</a>
            <a class="archive-dd-item" href="/people" role="menuitem">People</a>
          </div>
        </div>

        <div class="archive-dropdown" class:open={openDropdown === 'about'}>
          <a class="archive-nav-link archive-dropdown-trigger" role="button" tabindex="0" aria-haspopup="true" aria-expanded={openDropdown === 'about'} onclick={(e) => toggleDropdown('about', e)} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') toggleDropdown('about', e); }}>About</a>
          <div class="archive-dropdown-menu" role="menu">
            <a class="archive-dd-item" href="/docs" role="menuitem">Docs</a>
            <a class="archive-dd-item" href="/stats" role="menuitem">Statistics</a>
            <a class="archive-dd-item" href="/roadmap" role="menuitem">Roadmap</a>
            <a class="archive-dd-item" href="/roadmap/consensus" role="menuitem">Consensus</a>
            <a class="archive-dd-item" href="/modlog" role="menuitem">Modlog</a>
          </div>
        </div>
      </div>

      <div class="archive-nav-right">
        <form class="archive-search-form" onsubmit={handleSearch} role="search">
          <input
            class="archive-search-input"
            type="search"
            name="q"
            placeholder="Search"
            aria-label="Search FicNexus"
            bind:value={searchQuery}
          />
          <button class="archive-search-btn" type="submit">Search</button>
        </form>
      </div>
    </div>
  </nav>
</header>

<style>
  .archive-header {
    /* AO3: #header font 0.875em, margin 0 0 1em, position relative — no border */
    font-size: 0.875em;
    margin: 0 0 1em;
    padding: 0;
    position: relative;
    background: #ffffff;
  }

  /* ── Row 1: Logo + user — AO3 #header .heading float left 0.375em; #login float right 0.375em ── */
  .archive-top-row {
    background: #ffffff;
  }

  .archive-top-inner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1rem;
    max-width: 1100px;
    margin: 0 auto;
    min-height: 3.1em;
  }

  .archive-brand {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.714em;
    line-height: 1.75em;
    font-weight: 700;
    color: #900;
    text-decoration: none;
    white-space: nowrap;
    float: left;
    padding: 0.375em;
    display: inline-block;
  }

  .archive-brand:hover {
    color: #900;
  }

  .archive-brand-sup {
    font-size: 0.42em;
    font-weight: 400;
    vertical-align: super;
    margin-left: 0.15em;
    color: #666666;
  }

  .archive-top-right {
    display: flex;
    align-items: center;
    gap: 0;
    flex-shrink: 0;
    float: right;
    margin-right: 0.375em;
    position: relative;
    z-index: 20;
    padding: 0.429em 0 0;
  }

  .archive-hello {
    font-size: 1em;
    font-weight: 400;
    color: #2a2a2a;
  }

  .archive-top-link {
    font-size: 1em;
    font-weight: 400;
    color: #111;
    background: transparent;
    text-decoration: none;
    padding: 0.25em 0.75em;
    border-radius: 0;
    border: none;
  }

  .archive-top-link:hover {
    background: #ddd;
    color: #111;
  }

  #login .archive-top-link:hover,
  #register .archive-top-link:hover,
  .archive-user-dropdown .archive-top-link:hover {
    color: #900;
    border-radius: 0.25em;
  }

  .archive-logout-btn {
    font-size: 0.82em;
    font-weight: 600;
    color: var(--archive-muted, #666666);
    background: none;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 3px;
    padding: 0.2em 0.6em;
    cursor: pointer;
    font-family: inherit;
  }

  .archive-logout-btn:hover {
    color: #990000;
    border-color: #990000;
  }

  /* ── Row 2: Red navbar — AO3 #header .primary bg #900 red-ao3, inset -6px 10px, first-li margin-left 2em, a pad 0.429em 0.75em ── */
  .archive-navbar {
    background: #900 url('/images/red-ao3.png');
    padding: 0 0;
    width: 100%;
    box-shadow:
      inset 0 -6px 10px rgba(0, 0, 0, 0.35),
      1px 1px 3px -1px rgba(0, 0, 0, 0.25),
      inset 0 -1px 0 rgba(0, 0, 0, 0.85);
  }

  .archive-navbar-inner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    max-width: 1100px;
    margin: 0 auto;
    padding: 0 1rem;
    gap: 0;
  }

  .archive-nav-left,
  .archive-nav-right {
    display: flex;
    align-items: center;
    gap: 0;
  }

  /* AO3: li float left, position relative; .primary a 0.429em 0.75em #fff 400 */
  .archive-nav-left > .archive-nav-link:first-child,
  .archive-nav-left > .archive-dropdown:first-child .archive-nav-link {
    margin-left: 2em;
  }

  .archive-nav-link {
    display: block;
    float: left;
    position: relative;
    padding: 0.429em 0.75em;
    font-size: 1em;
    font-weight: 400;
    color: #fff;
    text-decoration: none;
    white-space: nowrap;
    line-height: 1.5;
  }

  /* AO3: .primary .open a / .dropdown:hover a background #ddd */
  .archive-nav-link:hover,
  .archive-dropdown:hover > .archive-nav-link {
    background: #ddd;
    color: #111;
  }

  /* ── Dropdowns (hover-based, AO3 style) ── */
  .archive-dropdown,
  .archive-login-dropdown,
  .archive-user-dropdown {
    position: relative;
    display: inline-block;
  }

  .archive-dropdown-trigger,
  .archive-user-dropdown > a {
    cursor: pointer;
    list-style: none;
    user-select: none;
  }

  .archive-dropdown:hover .archive-dropdown-menu,
  .archive-login-dropdown:hover .archive-login-menu,
  .archive-login-dropdown:hover .archive-register-menu,
  .archive-user-dropdown:hover .archive-user-menu,
  .archive-dropdown.open .archive-dropdown-menu,
  .archive-login-dropdown.open .archive-login-menu,
  .archive-login-dropdown.open .archive-register-menu,
  .archive-user-dropdown.open .archive-user-menu {
    visibility: visible;
    opacity: 1;
    transform: translateY(0);
  }

  .archive-dropdown:hover > .archive-dropdown-trigger,
  .archive-login-dropdown:hover > a,
  .archive-user-dropdown:hover > a {
    background: rgba(255, 255, 255, 0.15);
  }

  .archive-dropdown.open > .archive-nav-link,
  .archive-dropdown.open > .archive-dropdown-trigger {
    background: #ddd;
    color: #111;
  }

  /* AO3: #header .menu width 20em bg #ddd gradient + shadow 1px 1px 3px -1px #444 */
  .archive-dropdown-menu,
  .archive-register-menu {
    position: absolute;
    top: 100%;
    left: 0;
    right: auto;
    width: 20em;
    min-width: 20em;
    background: #ddd;
    background-image: linear-gradient(to bottom, rgba(221, 221, 221, 0.98) 0%, rgba(204, 204, 204, 0.98) 100%);
    border: none;
    box-shadow: 1px 1px 3px -1px #444;
    padding: 0.25em 0;
    z-index: 55;
    border-radius: 0;
    visibility: hidden;
    opacity: 0;
    transform: translateY(-4px);
    transition: opacity 0.15s ease, transform 0.15s ease;
  }

  /* Register menu sits in the top-right auth area, same as login menu */
  .archive-register-menu {
    right: 0;
    left: auto;
    min-width: 240px;
    padding: 0.5rem;
    background-image: linear-gradient(rgba(221, 221, 221, 0.98), rgba(204, 204, 204, 0.98));
  }

  .archive-dropdown-menu-right {
    right: 0;
    left: auto;
  }

  .archive-dd-item {
    display: block;
    width: 100%;
    padding: 0.75em 0.5em 0.5em;
    font-size: 1em;
    font-weight: 400;
    color: #111;
    text-decoration: none;
    background: transparent;
    border: none;
    border-bottom: 1px solid #888;
    margin: 0 0.25em;
    width: auto;
    text-align: left;
    cursor: pointer;
    font-family: inherit;
  }

  .archive-dd-item:last-child {
    border-bottom: none;
  }

  /* AO3 menu-item hover = white wash, text stays #111 (keep the colour
     pinned: the global link-hover grey would otherwise win). */
  .archive-dd-item:hover {
    background: rgba(255, 255, 255, 0.25);
    color: #111;
  }

  /* ── Login dropdown (AO3 #login.dropdown style) ── */
  .archive-login-dropdown {
    position: relative;
    display: inline-block;
  }

  .archive-login-dropdown > a {
    cursor: pointer;
    list-style: none;
    user-select: none;
  }

  .archive-login-menu {
    position: absolute;
    top: 100%;
    right: 0;
    min-width: 240px;
    background: #dddddd;
    border: 1px solid #bbbbbb;
    box-shadow: 0 3px 8px rgba(0, 0, 0, 0.15);
    padding: 0.5rem;
    z-index: 80;
    border-radius: 4px;
    background-image: linear-gradient(rgba(221, 221, 221, 0.98), rgba(204, 204, 204, 0.98));
    visibility: hidden;
    opacity: 0;
    transform: translateY(-4px);
    transition: opacity 0.15s ease, transform 0.15s ease;
  }

  .archive-login-menu::before {
    content: '';
    position: absolute;
    top: -6px;
    right: 10px;
    border-left: 6px solid transparent;
    border-right: 6px solid transparent;
    border-bottom: 6px solid #dddddd;
  }

  .archive-login-error {
    color: var(--archive-error, #cc0000);
    font-size: 0.82em;
    margin: 0 0 0.2rem;
  }

  .archive-reg-hint {
    font-size: 0.8em;
    color: var(--archive-muted, #666);
    margin: 0.3rem 0;
    padding: 0.3rem 0.5rem;
    background: var(--archive-surface-2, #f5f5f5);
    border: 1px solid var(--archive-border, #ddd);
    border-radius: 3px;
  }

  /* AO3-style inline login form */
  .archive-login-form {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .archive-login-heading {
    margin: 0 0 0.15rem;
    font-family: Georgia, 'Times New Roman', serif;
    font-weight: bold;
    color: #2a2a2a;
  }

  .archive-login-form label {
    font-size: 0.85em;
    color: #333333;
    margin-top: 0.2rem;
  }

  .archive-login-form input[type='text'],
  .archive-login-form input[type='password'] {
    width: 100%;
    padding: 0.25rem 0.4rem;
    border: 1px solid #aaaaaa;
    background: #ffffff;
    font-family: inherit;
    box-sizing: border-box;
  }

  .archive-login-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 0.45rem;
    gap: 0.75rem;
  }

  .archive-login-remember {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.85em;
    margin-top: 0 !important;
  }

  .archive-login-error {
    margin: 0;
    color: #990000;
    font-size: 0.85em;
  }

  .archive-login-form .archive-btn {
    background: linear-gradient(#f5f5f5, #dddddd);
    border: 1px solid #bbbbbb;
    color: #222222;
    padding: 0.25rem 1rem;
    cursor: pointer;
    font-weight: bold;
    white-space: nowrap;
  }

  .archive-login-form .archive-btn:hover {
    background: linear-gradient(#ffffff, #e8e8e8);
  }

  /* ── User greeting dropdown ── */
  .archive-user-dropdown {
    position: relative;
    display: inline-block;
  }

  .archive-user-dropdown > a {
    cursor: pointer;
    list-style: none;
    user-select: none;
  }

  .archive-user-menu {
    position: absolute;
    top: 100%;
    right: 0;
    min-width: 160px;
    background: #dddddd;
    border: 1px solid #bbbbbb;
    box-shadow: 0 3px 8px rgba(0, 0, 0, 0.15);
    padding: 0.25rem 0;
    z-index: 80;
    border-radius: 4px;
    background-image: linear-gradient(rgba(221, 221, 221, 0.98), rgba(204, 204, 204, 0.98));
    visibility: hidden;
    opacity: 0;
    transform: translateY(-4px);
    transition: opacity 0.15s ease, transform 0.15s ease;
  }

  .archive-user-menu::before {
    content: '';
    position: absolute;
    top: -6px;
    right: 10px;
    border-left: 6px solid transparent;
    border-right: 6px solid transparent;
    border-bottom: 6px solid #dddddd;
  }

  .archive-dd-divider {
    border: none;
    border-top: 1px solid #888;
    margin: 0 0.25em;
  }

  .archive-dd-inert {
    color: #999999;
    cursor: default;
    opacity: 0.6;
  }

  .archive-dd-inert:hover {
    background: none;
  }

  /* ── Search form — AO3 li.search float right margin 0.25em; input 0.15em 0.2857em gradient, button same gradient ── */
  .archive-search-form {
    display: flex;
    align-items: center;
    float: right;
    margin-right: 0.25em;
    color: #2a2a2a;
  }

  .archive-search-form fieldset { border: none; padding: 0; margin: 0; }

  .archive-search-input {
    width: 155px;
    padding: 0.15em;
    margin: 0.2857em 0.429em;
    font-size: 1em;
    border: none;
    border-radius: 0;
    background: #eee;
    background-image: linear-gradient(#fff 2%, #ddd 95%, #bbb 100%);
    color: #2a2a2a;
    font-family: inherit;
    outline: none;
  }

  .archive-search-input::placeholder {
    color: #888;
  }

  .archive-search-input:focus {
    color: #2a2a2a;
    outline: 1px dotted;
  }

  .archive-search-btn {
    padding: 0.25em 0.75em;
    margin: 0;
    font-size: 1em;
    font-weight: 400;
    color: #444;
    background: #eee;
    background-image: linear-gradient(#fff 2%, #ddd 95%, #bbb 100%);
    border: none;
    border-radius: 0.25em;
    cursor: pointer;
    font-family: inherit;
  }

  .archive-search-btn:hover {
    background: #fff;
    background-image: linear-gradient(#fff 2%, #eee 95%, #ccc 100%);
  }

  /* ── Mobile ── */
  @media (max-width: 767px) {
    .archive-top-inner {
      padding: 0.4rem 0.75rem;
    }

    .archive-navbar-inner {
      flex-wrap: wrap;
      padding: 0 0.75rem;
      gap: 0;
    }

    .archive-nav-left,
    .archive-nav-right {
      flex-wrap: wrap;
    }

    .archive-nav-right {
      width: 100%;
      padding-top: 0.25rem;
      border-top: 1px solid rgba(255, 255, 255, 0.2);
      margin-top: 0.25rem;
    }

    .archive-search-input {
      width: 100px;
    }
  }
</style>
