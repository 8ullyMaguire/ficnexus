<script lang="ts">
  // /translate?target=request:12:title&locale=es — improve a translation.
  // Prefills the editor with the machine (or current approved) text so the
  // human job is editing, not translating from scratch; submits a
  // kind='translate' proposal for curator consensus (+points on approval).
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { requestTranslations } from '$lib/api/translate';
  import { createProposal } from '$lib/api/proposals';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';

  interface Parts { type: string; id: string; field: string }
  function parseTarget(raw: string | null): Parts | null {
    if (!raw) return null;
    const [type, id, field] = raw.split(':');
    if (!type || !id || !field) return null;
    return { type, id, field };
  }

  let parts = $derived(parseTarget($page.url.searchParams.get('target')));
  let locale = $state($page.url.searchParams.get('locale') || 'es');
  let original = $state('');
  let current = $state('');
  let draft = $state('');
  let statusNote = $state('');
  let sourceHash = $state('');
  let loading = $state(true);
  let error = $state('');
  let msg = $state('');
  let submitting = $state(false);
  let formOpenedAt = $state(0);

  onMount(async () => {
    await auth.init();
    formOpenedAt = Date.now() - 5000;
    if (!parts) { error = 'Missing or malformed ?target=type:id:field'; loading = false; return; }
    if (!auth.isLoggedIn) { error = t('auth.loginToTranslate'); loading = false; return; }
    try {
      const all = await requestTranslations(
        [{ type: parts.type, id: parts.id, field: parts.field }], locale);
      const res = all[`${parts.type}:${parts.id}:${parts.field}:${locale}`];
      // Current translation (machine or approved) prefills the editor so the
      // human job is *editing*, not translating from scratch. The original
      // source text is handed over by TranslatedText via sessionStorage
      // (URL params would break on long bodies); ?original= wins if present.
      const stashKey = `fichub_tr_original:${parts.type}:${parts.id}:${parts.field}`;
      original = $page.url.searchParams.get('original')
        ?? sessionStorage.getItem(stashKey)
        ?? '';
      if (res?.text) { current = res.text; draft = res.text; }
      if (res?.status === 'machine') statusNote = t('translate.machineBadge');
      else if (res?.status === 'approved') statusNote = t('translate.approvedNote');
    } catch {
      error = t('translate.loadFailed');
    } finally { loading = false; }
  });

  async function submit() {
    if (!parts || submitting) return;
    const text = draft.trim();
    if (!text) return;
    if (text === current.trim()) { msg = t('translate.unchanged'); return; }
    submitting = true; error = '';
    try {
      const res = await createProposal({
        kind: 'translate',
        target_type: parts.type,
        target_id: parts.id,
        payload: { field: parts.field, locale, text, source_hash: sourceHash, prior_status: current ? 'replacing' : 'new' },
        form_opened_at: String(formOpenedAt),
      });
      if (res.err === 0) { msg = t('translate.submitted'); current = text; }
      else error = res.msg ?? t('translate.submitFailed');
    } catch {
      error = t('translate.submitFailed');
    } finally { submitting = false; }
  }
</script>

<svelte:head><title>{t('translate.improve')} — FicNexus</title></svelte:head>

<div class="tr-page">
  <h1 class="tr-title">{t('translate.improve')}</h1>
  {#if !parts}
    <p class="tr-err">{error || t('translate.missingTarget')}</p>
  {:else if loading}
    <p class="tr-muted">{t('common.loading')}</p>
  {:else}
    <p class="tr-sub">
      <span class="tr-key">{parts.type}:{parts.id}:{parts.field}</span> → {locale}
      {#if statusNote}<span class="tr-badge">{statusNote}</span>{/if}
    </p>

    {#if error}<p class="tr-err">{error}</p>{/if}
    {#if msg}<p class="tr-ok">{msg}</p>{/if}

    {#if original}
      <h2 class="tr-h2">{t('translate.originalHeading')}</h2>
      <div class="tr-orig">{original}</div>
    {/if}

    <h2 class="tr-h2">{t('translate.yourVersion')}</h2>
    <textarea class="tr-area" rows="10" bind:value={draft} disabled={!auth.isLoggedIn}></textarea>
    <p class="tr-count">{draft.length} · {t('translate.pointsHint')}</p>
    <div class="tr-acts">
      <button class="tr-btn" disabled={submitting || !draft.trim()} onclick={submit}>{t('translate.submit')}</button>
      <ArchiveButton href="/requests">{t('common.back')}</ArchiveButton>
    </div>
  {/if}
</div>

<style>
  .tr-page { max-width: 860px; margin: 0 auto; font-family: Georgia, 'Times New Roman', serif; color: #000; }
  .tr-title { font-size: 1.7em; font-weight: 400; color: #900; margin: 0 0 .15em; border-bottom: 1px solid #ddd; padding-bottom: .3em; }
  .tr-sub { font-size: .9em; color: #666; }
  .tr-key { border-bottom: 1px solid #999; }
  .tr-badge { background: #900; color: #fff; border-radius: .25em; padding: .05em .5em; font-size: .85em; margin-left: .4em; }
  .tr-h2 { font-size: 1.05em; font-weight: 700; color: #2a2a2a; margin: 1.1em 0 .3em; }
  .tr-orig { border: 1px solid #ddd; background: #f5f5f5; padding: .6em .8em; white-space: pre-wrap; line-height: 1.5; max-height: 14em; overflow: auto; }
  .tr-area { width: 100%; border: 1px solid #bbb; border-radius: .25em; font: inherit; padding: .5em .7em; box-sizing: border-box; }
  .tr-area:focus { background: #f3efec; }
  .tr-count { font-size: .8em; color: #666; }
  .tr-acts { display: flex; gap: .6rem; margin-top: .4rem; }
  .tr-btn { background: #eee; background-image: linear-gradient(#fff 2%, #ddd 95%, #bbb 100%); color: #444; border: 1px solid #bbb; border-bottom: 1px solid #aaa; border-radius: .25em; padding: .25em 1em; cursor: pointer; font: inherit; }
  .tr-btn:hover { color: #900; border-top-color: #999; border-left-color: #999; box-shadow: inset 2px 2px 2px #bbb; }
  .tr-err { color: #c0392b; }
  .tr-ok { color: #2e9e5b; }
  .tr-muted { color: #666; }
</style>
