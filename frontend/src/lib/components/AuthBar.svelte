<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';
  import { getSiteInfo, type RegistrationMode } from '$lib/api/forum';
  import { t } from '$lib/i18n/index.svelte';

  let showAuth = $state(false);
  let mode = $state<'login' | 'register'>('login');
  let username = $state('');
  let password = $state('');
  let email = $state('');
  let inviteCode = $state('');
  let rememberMe = $state(false);
  let error = $state('');

  // F7: site registration mode (open | invite | application) — shown as a
  // hint on the login/register modal; the invite-code field only appears
  // when the site runs on invitations.
  let regMode = $state<RegistrationMode | null>(null);
  let siteError = $state('');

  onMount(async () => {
    try {
      const res = await getSiteInfo();
      if (res.err === 0 && res.site) {
        regMode = res.site.registration_mode;
      }
    } catch {
      siteError = t('site.loadError');
    }
  });

  function open(m: 'login' | 'register') {
    mode = m;
    error = '';
    showAuth = true;
  }

  function close() {
    showAuth = false;
    error = '';
  }

  async function handleSubmit() {
    error = '';
    let success = false;
    if (mode === 'login') {
      success = await auth.handleLogin(username, password, rememberMe);
    } else {
      const code = regMode === 'invite' ? inviteCode.trim() : undefined;
      success = await auth.handleRegister(username, password, email || undefined, code);
    }
    if (success) {
      close();
      username = '';
      password = '';
      email = '';
      inviteCode = '';
    } else {
      error = mode === 'login' ? t('auth.loginFailed') : t('auth.registerFailed');
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') handleSubmit();
  }
</script>

{#if !auth.isLoggedIn}
  <button class="auth-btn" onclick={() => open('login')}>{t('nav.login')}</button>
  <button class="auth-btn secondary" onclick={() => open('register')}>{t('nav.register')}</button>
{:else}
  <span class="user-badge">
    👤 {auth.username}
    <span class="level-chip">★ {auth.level}</span>
    <a class="trust-link" href="/settings/trust" title="Your trust level">Trust</a>
  </span>
  <button class="auth-btn secondary" onclick={() => auth.handleLogout()}>{t('nav.logout')}</button>
{/if}

{#if showAuth}
  <div class="modal-backdrop" onclick={close} role="presentation">
    <div class="modal" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()} role="dialog" aria-modal="true" tabindex="-1">
      <h3>{mode === 'login' ? t('nav.login') : t('nav.register')}</h3>
      {#if regMode}
        <p class="mode-hint">
          {#if regMode === 'invite'}
            {t('site.registrationInvite')}
          {:else if regMode === 'application'}
            {t('site.registrationApplication')}
          {:else}
            {t('site.registrationOpen')}
          {/if}
        </p>
      {:else if siteError}
        <p class="mode-hint error-text">{siteError}</p>
      {/if}
      <label for="auth-username">
        {t('auth.username')}
        <input id="auth-username" type="text" bind:value={username} onkeydown={onKeydown} placeholder={t('auth.passwordPlaceholder')} />
      </label>
      <label>
        {t('auth.password')}
        <input type="password" bind:value={password} onkeydown={onKeydown} placeholder={t('auth.password')} />
      </label>
      {#if mode === 'login'}
        <div class="modal-row checkbox-row">
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={rememberMe} />
            <span>{t('auth.rememberMe')}</span>
          </label>
          <a href="/forgot" class="forgot-link">{t('auth.forgotPassword')}</a>
        </div>
      {/if}
      {#if mode === 'register'}
        <label>
          {t('auth.email')}
          <input type="email" bind:value={email} onkeydown={onKeydown} placeholder="email@example.com" />
        </label>
        {#if regMode === 'invite'}
          <label>
            {t('auth.inviteCode')}
            <input type="text" bind:value={inviteCode} onkeydown={onKeydown} placeholder={t('auth.inviteCodePlaceholder')} />
          </label>
        {/if}
      {/if}
      {#if error}
        <p class="error-text">{error}</p>
      {/if}
      <div class="modal-actions">
        <button class="btn btn-secondary" onclick={close}>{t('common.cancel')}</button>
        <button class="btn" onclick={handleSubmit} disabled={auth.loading}>
          {#if auth.loading}<span class="spinner"></span> ...{:else}{mode === 'login' ? t('nav.login') : t('nav.register')}{/if}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .auth-btn {
    background: var(--color-primary);
    color: white;
    border: none;
    border-radius: var(--radius-sm);
    padding: 0.35rem 0.8rem;
    font-weight: 600;
    font-size: 0.85rem;
    cursor: pointer;
  }
  .auth-btn.secondary {
    background: var(--color-surface-2);
    color: var(--color-text);
    border: 1px solid var(--color-border);
  }
  .user-badge {
    color: var(--color-text);
    font-size: 0.85rem;
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
  }
  .level-chip {
    font-size: 0.72rem;
    background: var(--color-chip-bg, #eef);
    border: 1px solid var(--color-border, #ddd);
    padding: 0.1rem 0.45rem;
    border-radius: 999px;
    font-weight: 700;
  }
  .trust-link {
    font-size: 0.72rem;
    color: inherit;
    text-decoration: none;
    opacity: 0.75;
  }
  .trust-link:hover {
    opacity: 1;
    text-decoration: underline;
  }
  .mode-hint {
    font-size: 0.82rem;
    color: var(--color-muted);
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 0.45rem 0.6rem;
    margin: 0 0 0.6rem;
  }
  .error-text {
    color: var(--color-error);
  }
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: 1rem;
  }
  .modal {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    padding: 1.5rem;
    width: 100%;
    max-width: 380px;
    box-shadow: var(--shadow);
  }
  .modal h3 {
    margin-top: 0;
    margin-bottom: 1rem;
  }
  .modal label {
    display: block;
    margin: 0.6rem 0;
    font-size: 0.88rem;
    color: var(--color-muted);
  }
  .modal label input {
    width: 100%;
    margin-top: 0.3rem;
  }
  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.6rem;
    margin-top: 1rem;
  }
  .modal-row.checkbox-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 0.6rem 0;
  }
  .checkbox-label {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.85rem;
    color: var(--color-text);
    cursor: pointer;
    margin: 0;
  }
  .checkbox-label input {
    width: auto;
    margin: 0;
  }
  .forgot-link {
    font-size: 0.8rem;
    color: var(--color-muted);
    text-decoration: none;
  }
  .forgot-link:hover {
    text-decoration: underline;
  }
</style>
