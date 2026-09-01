<script lang="ts">
  import { getPref } from '$lib/prefs';
  const uiMode = $derived(getPref('uiMode'));
</script>

<svelte:head>
  <title>Privacy Policy — FicNexus</title>
</svelte:head>

<div class="prose mx-auto max-w-2xl px-4 py-8">
  <h1 class="mb-4 text-2xl font-bold">Privacy Policy</h1>
  <p class="mb-4 text-sm opacity-70">Last updated: August 30, 2026</p>

  <section class="mb-6">
    <h2 class="mb-2 text-lg font-semibold">1. What data we collect</h2>
    <ul class="list-disc space-y-1 pl-5 text-sm">
      <li><strong>Account data</strong> — username and email (email optional at registration).</li>
      <li><strong>Content you submit</strong> — comments, bookmarks, ratings, reviews, forum posts, uploaded works.</li>
      <li><strong>Usage data</strong> — IP address (for rate limiting and abuse prevention), browser client ID, pages visited, download history.</li>
      <li><strong>Local storage</strong> — we use your browser's localStorage for preferences (theme, interface mode). We do not use third-party tracking cookies.</li>
    </ul>
  </section>

  <section class="mb-6">
    <h2 class="mb-2 text-lg font-semibold">2. How we use your data</h2>
    <ul class="list-disc space-y-1 pl-5 text-sm">
      <li>To operate the archive: show your works, comments, bookmarks, and recommendations.</li>
      <li>To personalize recommendations (your reading signals shape what we suggest; this can be disabled in Settings → Personalized recommendations).</li>
      <li>To prevent abuse: rate limiting, spam detection, and moderation.</li>
    </ul>
  </section>

  <section class="mb-6">
    <h2 class="mb-2 text-lg font-semibold">3. Legal basis (GDPR)</h2>
    <p class="text-sm">
      We process personal data under GDPR Art. 6(1)(b) (contract — providing the service),
      Art. 6(1)(f) (legitimate interest — abuse prevention, archive integrity), and
      Art. 6(1)(a) (consent — recorded at registration for the Terms of Service).
    </p>
  </section>

  <section class="mb-6">
    <h2 class="mb-2 text-lg font-semibold">4. Your rights</h2>
    <ul class="list-disc space-y-1 pl-5 text-sm">
      <li><strong>Access</strong> — view your data on your profile and in Settings.</li>
      <li><strong>Portability</strong> — download all your data as a ZIP in Settings → Account → Download my data.</li>
      <li><strong>Erasure</strong> — delete your account and all personal data in Settings → Account → Delete my account (GDPR Art. 17).</li>
      <li><strong>Rectification</strong> — edit your profile and preferences in Settings.</li>
      <li><strong>Object</strong> — disable personalized recommendations in Settings.</li>
    </ul>
  </section>

  <section class="mb-6">
    <h2 class="mb-2 text-lg font-semibold">5. Copyright</h2>
    <p class="text-sm">
      FicNexus is a transformative fanwork archive. If you are a rights holder and believe a work
      hosted here infringes your copyright, please submit a takedown notice via our
      <a class="text-indigo-600 underline" href="/copyright">Copyright page</a>.
    </p>
  </section>

  <section>
    <h2 class="mb-2 text-lg font-semibold">6. Contact</h2>
    <p class="text-sm">
      For privacy questions, contact the site operator via the
      <a class="text-indigo-600 underline" href="/forum">forum</a> or
      <a class="text-indigo-600 underline" href="https://github.com/ficnexus/ficnexus/issues">GitHub issues</a>.
    </p>
  </section>
</div>
