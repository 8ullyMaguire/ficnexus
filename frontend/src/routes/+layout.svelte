<script lang="ts">
 import '../app.css';
 import { onMount } from 'svelte';
 import HomeDashboard from '$lib/components/HomeDashboard.svelte';
 import AuthBar from '$lib/components/AuthBar.svelte';
 import LocaleSelector from '$lib/components/LocaleSelector.svelte';
 import NotificationBell from '$lib/components/NotificationBell.svelte';
 import NavDropdown from '$lib/components/NavDropdown.svelte';
 import OfflineIndicator from '$lib/components/OfflineIndicator.svelte';
import UpdatePrompt from '$lib/components/UpdatePrompt.svelte';
import InstallPrompt from '$lib/components/InstallPrompt.svelte';
 import HelpModal from '$lib/components/HelpModal.svelte';
 import CommandPalette from '$lib/components/CommandPalette.svelte';
 import DocLink from '$lib/components/DocLink.svelte';
 import { auth } from '$lib/stores/auth.svelte';
 import { exportUserData } from '$lib/api/social';
 import { getPref, setPref, applyUiParam } from '$lib/prefs';
 import ArchiveLayout from '$lib/ui/archive/ArchiveLayout.svelte';

 import { applyTheme, loadTheme } from '$lib/themes/apply.js';
 import { goto } from '$app/navigation';
 import { page } from '$app/stores';
 import { i18n, t, initI18n, setLocale } from '$lib/i18n/index.svelte';

 let { children } = $props();

 // Determine if we're on a route page (needs {children}) or a tab page
 const routePages = ['/search', '/ask', '/leaderboard', '/bookmarks', '/works/', '/notifications', '/follows', '/updates', '/badges', '/trending', '/stats', '/roadmap', '/roadmap/board', '/roadmap/changelog', '/blind-date', '/quests', '/read/', '/work/', '/curator', '/requests', '/series', '/authors', '/lists', '/shelves', '/admin', '/admin/auto-tag', '/admin/bulk-actions', '/admin/comment-triage', '/admin/blacklist', '/admin/stats', '/curator/flags', '/work-proposals', '/recommendations', '/download', '/settings', '/settings/theme', '/forum', '/forum/new', '/forum/search', '/forum/moderate', '/forum/metamod', '/forum/invites', '/forum/apply', '/forum/blocks', '/forum/groups', '/messages', '/fandoms', '/fandom', '/tags', '/upload', '/my-works'];
 let isRoutePage = $derived(routePages.some(p => $page.url.pathname.startsWith(p)));

 // Landing page renders HomeDashboard directly; all other paths are real
 // route pages ({@render children()}). The old tab-switch flow (goTab /
 // activeTab) was removed when /download became a real route page.

 const discoverLinks = [
 { href: '/trending', label: t('nav.trending'), icon: '' },
 { href: '/fandoms', label: 'Fandoms', icon: '' },
 { href: '/leaderboard', label: t('nav.rankings'), icon: '' },
 { href: '/ask', label: t('nav.askTheArchive'), icon: '' },
 { href: '/roadmap', label: t('nav.roadmap'), icon: '' },
 { href: '/roadmap/board', label: 'Roadmap Board', icon: '' },
 { href: '/roadmap/changelog', label: 'Changelog', icon: '' },
 { href: '/blind-date', label: t('nav.blindDate'), icon: '' },
 { href: '/requests', label: t('nav.requests'), icon: '' },
 { href: '/work-proposals', label: t('nav.workProposals'), icon: '' },
 { href: '/forum', label: t('nav.forum'), icon: '' },
 { href: '/recommendations', label: t('nav.recommendations'), icon: '' },
 { href: '/marketplace', label: 'Marketplace', icon: '' },
 ];
 const libraryLinks = [
 { href: '/bookmarks', label: t('nav.bookmarks'), icon: '' },
 { href: '/lists', label: t('nav.lists'), icon: '' },
 { href: '/shelves', label: t('nav.shelves'), icon: '' },
 { href: '/follows', label: t('nav.following'), icon: '' },
 { href: '/updates', label: t('nav.updates'), icon: '' },
 { href: '/feed', label: t('nav.feed'), icon: '' },
 ];
 const userLinks = [
 { href: '/stats', label: t('nav.profileStats'), icon: '' },
 { href: '/settings', label: 'Settings', icon: '' },
 { href: '/settings/theme', label: 'Theme', icon: '' },
 { href: '/modlog', label: t('nav.modlog'), icon: '' },
 { href: '/badges', label: t('nav.badges'), icon: '' },
 { href: '/quests', label: t('nav.quests'), icon: '' },
 ];

 let searchQuery = $state('');

 // UI mode: archive (AO3-style) or modern (default SvelteKit UI)
 let uiMode = $derived(getPref('uiMode'));

 // Mobile hamburger menu
 let mobileOpen = $state(false);

 // Level-gated moderation tooling: admin dashboard / curator hub / flag
 // queue (site-wide level ≥ 100 = admin; the level gate replaces the
 // legacy role check).
 const isCurator = $derived(auth.level >= 100);

 // Resolve the initial locale as early as possible: localStorage 
 // auth.user.locale browser auto-detect. Called on mount (after
 // auth.init) so a signed-in user's server-side preference wins when the
 // visitor has no explicit choice. The $state store is reactive, so all
 // t() call sites re-render the moment the locale changes.
 onMount(async () => {
 // Apply ?ui= URL parameter override
 applyUiParam($page.url);
 // Apply saved theme before anything renders.
 applyTheme(loadTheme());
 initI18n({ userLocale: auth.user?.locale ?? null });
 auth.init().then(async () => {
 // User locale may only be known after auth resolves.
 if (!localStorage.getItem('fichub_locale') && auth.user?.locale) {
 setLocale(auth.user.locale);
 }
 });
 });

 // Keep<html lang="..."> in sync with the active locale.
 $effect(() => {
 document.documentElement.lang = i18n.locale;
 });

 // Close the mobile menu when a link inside it is clicked
 function onMobileNavClick() {
 mobileOpen = false;
 }

 function onSearchKeydown(e: KeyboardEvent) {
 if (e.key === 'Enter' && searchQuery.trim()) {
 goto(`/search?q=${encodeURIComponent(searchQuery.trim())}`);
 }
 }

 function switchToArchive() {
 setPref('uiMode', 'archive');
 window.location.reload();
 }

 // One-click full user data export: fetch the ZIP and trigger the browser
 // download. Uses a token from localStorage via the API helper.
 let exporting = $state(false);
 let exportError = $state('');

 async function handleExportData() {
 if (exporting) return;
 exporting = true;
 exportError = '';
 try {
 const res = await exportUserData();
 const blob = await res.blob();
 const url = URL.createObjectURL(blob);
 const a = document.createElement('a');
 // The server names the archive ficnexus-user-data-<username>.zip; match
 // it on the client so the download lands with a clear filename.
     a.href = url;
 a.download = 'ficnexus-user-data.zip';
 a.click();
 URL.revokeObjectURL(url);
 } catch (e) {
 exportError = 'Export failed. Please try again.';
 console.error('User data export failed', e);
 } finally {
 exporting = false;
 }
 }
</script>

<svelte:head>
<link rel="manifest" href="/manifest.webmanifest" />
<meta name="theme-color" content="#171a23" />
<link rel="alternate" type="application/atom+xml" title="FicNexus — New Arrivals" href="/feed.xml" />
</svelte:head>

{#if uiMode === 'archive'}
<ArchiveLayout>
 {@render children()}
</ArchiveLayout>
{:else}
<div class="app">
<header class="topbar">
<div class="brand">
<a href="/" class="brand-link" aria-label="FicNexus home">
<span class="logo"></span>
<span class="title">FicNexus</span>
</a>
</div>

<nav class="main-nav" aria-label={t('nav.primaryAria')}>
<NavDropdown label={t('nav.discover')} icon="">
 {#each discoverLinks as link}
<a class="dd-item" href={link.href}>{link.icon} {link.label}</a>
 {/each}
</NavDropdown>

 {#if auth.isLoggedIn}
<NavDropdown label={t('nav.library')} icon="">
 {#each libraryLinks as link}
<a class="dd-item" href={link.href}>{link.icon} {link.label}</a>
 {/each}
</NavDropdown>
 {/if}
</nav>

<div class="search-area">
<input
 class="nav-search"
 type="search"
 placeholder={t('nav.searchPlaceholder')}
 bind:value={searchQuery}
 onkeydown={onSearchKeydown}
 aria-label={t('nav.searchAria')}
 />
<a class="adv-link" href="/search?advanced=1" title={t('nav.advancedSearch')}></a>
</div>

<a class="btn cta-download" href="/download">
<span aria-hidden="true"></span> {t('nav.download')}
</a>

<a class="support-link" href="https://liberapay.com/Critical_Nexus" target="_blank" rel="noopener" title="Support FicNexus on Liberapay">
<span aria-hidden="true"></span> Support
</a>

<div class="auth-area">
 {#if auth.isLoggedIn}
<NavDropdown label={auth.username ?? t('nav.account')} icon="" align="right">
 {#each userLinks as link}
<a class="dd-item" href={link.href}>{link.icon} {link.label}</a>
 {/each}
<button class="dd-item" type="button" onclick={handleExportData} disabled={exporting}>
 {exporting ? t('nav.exporting') : t('nav.exportMyData')}
</button>
 {#if exportError}
<span class="dd-row dd-error">{exportError}</span>
 {/if}
<span class="dd-row"><NotificationBell /></span>
 {#if isCurator}
<a class="dd-item" href="/admin"> {t('nav.adminDashboard')}</a>
<a class="dd-item" href="/curator"> {t('nav.curatorHub')}</a>
<a class="dd-item" href="/curator/flags"> {t('nav.flagQueue')}</a>
 {/if}
<div class="dd-divider"></div>
<span class="dd-row dd-inline"><LocaleSelector /></span>
<div class="dd-divider"></div>
<button class="dd-item" type="button" onclick={switchToArchive}> Switch to Archive</button>
<button class="dd-item" type="button" onclick={() => auth.handleLogout()}> {t('nav.logout')}</button>
</NavDropdown>
 {:else}
<AuthBar />
 {/if}
</div>

<!-- Mobile hamburger -->
<button
 class="hamburger"
 type="button"
 aria-label={t('nav.toggleMenu')}
 aria-expanded={mobileOpen}
 onclick={() => (mobileOpen = !mobileOpen)}
 >
<span aria-hidden="true">{mobileOpen ? '' : ''}</span>
</button>
</header>

 {#if mobileOpen}
<nav class="mobile-menu" aria-label={t('nav.mobileAria')}>
<span class="mobile-group-label">{t('nav.discover')}</span>
 {#each discoverLinks as link}
<a class="dd-item" href={link.href} onclick={onMobileNavClick}>{link.icon} {link.label}</a>
 {/each}
<a class="dd-item" href="/download" onclick={onMobileNavClick}> {t('nav.download')}</a>
<a class="dd-item" href="https://liberapay.com/Critical_Nexus" target="_blank" rel="noopener" onclick={onMobileNavClick}> Support</a>

 {#if auth.isLoggedIn}
<span class="mobile-group-label">{t('nav.library')}</span>
 {#each libraryLinks as link}
<a class="dd-item" href={link.href} onclick={onMobileNavClick}>{link.icon} {link.label}</a>
 {/each}
<span class="mobile-group-label">{t('nav.account')}</span>
 {#each userLinks as link}
<a class="dd-item" href={link.href} onclick={onMobileNavClick}>{link.icon} {link.label}</a>
 {/each}
<button class="dd-item" type="button" onclick={() => { handleExportData(); onMobileNavClick(); }} disabled={exporting}>
 {exporting ? t('nav.exporting') : t('nav.exportMyData')}
</button>
<a class="dd-item" href="/notifications" onclick={onMobileNavClick}> {t('nav.notifications')}</a>
 {#if isCurator}
<a class="dd-item" href="/admin" onclick={onMobileNavClick}> {t('nav.adminDashboard')}</a>
<a class="dd-item" href="/curator" onclick={onMobileNavClick}> {t('nav.curatorHub')}</a>
<a class="dd-item" href="/curator/flags" onclick={onMobileNavClick}> {t('nav.flagQueue')}</a>
 {/if}
<span class="mobile-group-label">{t('nav.locale')}</span>
<div class="mobile-locale"><LocaleSelector /></div>
<button class="dd-item" type="button" onclick={() => { auth.handleLogout(); onMobileNavClick(); }}> {t('nav.logout')}</button>
 {:else}
<span class="mobile-group-label">{t('nav.account')}</span>
<div class="mobile-auth"><AuthBar /></div>
 {/if}
</nav>
 {/if}

<main class="container">
 {#if isRoutePage}
 {@render children()}
 {:else}
<HomeDashboard />
 {/if}
</main>

<OfflineIndicator />
<UpdatePrompt />
<InstallPrompt />

<footer class="footer muted">
<span>FicNexus — {t('footer.tagline')}</span>
<span class="footer-links">
<button class="footer-link-btn" type="button" onclick={switchToArchive}>Prefer the AO3 look? Switch interface</button>
<a href="https://opencommit.eu/MagicZhang/fichub" target="_blank" rel="noopener" class="footer-icon" title="Source code on Forgejo">
<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
<path d="M12 2C6.48 2 2 6.48 2 12c0 4.42 2.87 8.17 6.84 9.5.5.09.66-.22.66-.48v-1.7c-2.78.6-3.37-1.34-3.37-1.34-.45-1.16-1.11-1.47-1.11-1.47-.91-.62.07-.6.07-.6 1 .07 1.53 1.03 1.53 1.03.89 1.52 2.34 1.08 2.91.83.09-.65.35-1.09.63-1.34-2.22-.25-4.55-1.11-4.55-4.94 0-1.09.39-1.98 1.03-2.68-.1-.25-.45-1.27.1-2.65 0 0 .84-.27 2.75 1.02.8-.22 1.65-.33 2.5-.33.85 0 1.7.11 2.5.33 1.91-1.29 2.75-1.02 2.75-1.02.55 1.38.2 2.4.1 2.65.64.7 1.03 1.59 1.03 2.68 0 3.84-2.34 4.69-4.57 4.94.36.31.68.92.68 1.85v2.74c0 .27.16.58.67.48A10 10 0 0022 12c0-5.52-4.48-10-10-10z"/>
</svg>
</a>
<a href="/docs/" class="footer-icon" title="Read the docs" data-sveltekit-reload></a>
<DocLink slug="downloading#choosing-a-format" label="" title="Download formats & e-reader setup" inline />
<a href="/feed.xml" class="footer-icon" title="Subscribe to new arrivals (Atom feed)"></a>
<a href="https://discord.gg/AmE93m23dW" target="_blank" rel="noopener" class="footer-icon" title="Join our Discord">
<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
<path d="M20.317 4.37a19.79 19.79 0 00-4.885-1.515.074.074 0 00-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 00-5.487 0 12.64 12.64 0 00-.617-1.25.077.077 0 00-.079-.037A19.736 19.736 0 003.677 4.37a.07.07 0 00-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 00.031.057 19.9 19.9 0 005.993 3.03.078.078 0 00.084-.028 14.09 14.09 0 001.226-1.994.076.076 0 00-.041-.106 13.107 13.107 0 01-1.872-.892.077.077 0 01-.008-.128 10.2 10.2 0 00.372-.292.074.074 0 01.077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 01.078.01c.12.098.246.198.373.292a.077.077 0 01-.006.127 12.299 12.299 0 01-1.873.892.077.077 0 00-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 00.084.028 19.839 19.839 0 006.002-3.03.077.077 0 00.032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 00-.031-.03zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z"/>
</svg>
</a>
</span>
</footer>
</div>
{/if}
<HelpModal />
<CommandPalette />

<style>
 .app {
 min-height: 100vh;
 display: flex;
 flex-direction: column;
 background: var(--color-surface);
 }
 .topbar {
 position: sticky;
 top: 0;
 z-index: 10;
 background: var(--color-surface);
 border-bottom: 1px solid var(--color-border);
 padding: 0.6rem 1rem;
 display: flex;
 align-items: center;
 gap: 0.8rem;
 flex-wrap: nowrap;
 }
 .brand {
 display: flex;
 align-items: center;
 gap: 0.4rem;
 flex-shrink: 0;
 }
 .brand-link {
 display: flex;
 align-items: center;
 gap: 0.4rem;
 color: inherit;
 text-decoration: none;
 }
 .brand-link:hover {
 text-decoration: none;
 }
 .logo {
 font-size: 1.4rem;
 }
 .title {
 font-weight: 800;
 font-size: 1.2rem;
 letter-spacing: -0.02em;
 }
 .main-nav {
 display: flex;
 align-items: center;
 gap: 0.3rem;
 flex-shrink: 0;
 }
 .search-area {
 margin-left: auto;
 display: flex;
 align-items: center;
 gap: 0.5rem;
 flex-shrink: 0;
 }
 .nav-search {
 width: 180px;
 padding: 0.4rem 0.7rem;
 font-size: 0.88rem;
 border-radius: var(--radius-sm);
 }
 .adv-link {
 color: var(--color-muted);
 font-size: 0.95rem;
 font-weight: 700;
 line-height: 1;
 padding: 0.15rem 0.4rem;
 border: 1px solid var(--color-border);
 border-radius: var(--radius-sm);
 background: var(--color-surface-2);
 white-space: nowrap;
 }
 .adv-link:hover {
 color: var(--color-text);
 text-decoration: none;
 border-color: var(--color-primary);
 }
 .cta-download {
 padding: 0.45rem 0.9rem;
 font-size: 0.9rem;
 flex-shrink: 0;
 white-space: nowrap;
 }
 .support-link {
 display: inline-flex;
 align-items: center;
 gap: 0.3rem;
 padding: 0.45rem 0.9rem;
 font-size: 0.9rem;
 font-weight: 600;
 flex-shrink: 0;
 white-space: nowrap;
 color: var(--color-text);
 border: 1px solid var(--color-border);
 border-radius: var(--radius-sm);
 background: var(--color-surface-2);
 text-decoration: none;
 transition: border-color 0.15s, color 0.15s;
 }
 .support-link:hover {
 color: var(--color-primary);
 border-color: var(--color-primary);
 text-decoration: none;
 }
 .auth-area {
 display: flex;
 align-items: center;
 gap: 0.4rem;
 flex-shrink: 0;
 }
 .hamburger {
 display: none;
 align-items: center;
 justify-content: center;
 width: 2.2rem;
 height: 2.2rem;
 font-size: 1.15rem;
 background: var(--color-surface-2);
 border: 1px solid var(--color-border);
 border-radius: var(--radius-sm);
 color: var(--color-text);
 flex-shrink: 0;
 }
 .mobile-menu {
 display: none;
 flex-direction: column;
 gap: 0.15rem;
 background: var(--color-surface);
 border-bottom: 1px solid var(--color-border);
 padding: 0.6rem 1rem 0.8rem;
 }
 .mobile-menu .dd-item {
 padding: 0.5rem 0.6rem;
 }
 .mobile-group-label {
 font-size: 0.72rem;
 font-weight: 700;
 text-transform: uppercase;
 letter-spacing: 0.06em;
 color: var(--color-muted);
 margin: 0.6rem 0 0.2rem;
 }
 .mobile-group-label:first-child {
 margin-top: 0;
 }
 .mobile-locale,
 .mobile-auth {
 padding: 0.2rem 0.6rem;
 }
 main {
 flex: 1;
 }
 .footer {
 border-top: 1px solid var(--color-border);
 padding: 1rem;
 display: flex;
 justify-content: space-between;
 flex-wrap: wrap;
 gap: 0.5rem;
 font-size: 0.82rem;
 }
 .footer-links {
 display: flex;
 align-items: center;
 gap: 0.8rem;
 }
 .footer-icon {
 color: var(--color-muted);
 display: flex;
 align-items: center;
 transition: color 0.15s;
 }
 .footer-icon:hover {
 color: var(--color-text);
 text-decoration: none;
 }
 .footer-link-btn {
 background: none;
 border: none;
 color: var(--color-muted);
 font: inherit;
 font-size: inherit;
 cursor: pointer;
 padding: 0;
 transition: color 0.15s;
 }
 .footer-link-btn:hover {
 color: var(--color-text);
 text-decoration: underline;
 }
 @media (max-width: 767px) {
 .main-nav,
 .cta-download,
 .support-link,
 .auth-area {
 display: none;
 }
 .hamburger {
 display: inline-flex;
 }
 .mobile-menu {
 display: flex;
 }
 .search-area {
 flex: 1;
 margin-left: 0;
 }
 .nav-search {
 flex: 1;
 width: auto;
 }
 }
</style>
