<script lang="ts">
  import { getPref } from '$lib/prefs';
  const uiMode = $derived(getPref('uiMode'));

  let workUrl = $state('');
  let workTitle = $state('');
  let claimantName = $state('');
  let claimantEmail = $state('');
  let reason = $state('');
  let sending = $state(false);
  let msg = $state('');
  let error = $state('');

  async function submit() {
    if (!workUrl.trim() || !claimantName.trim() || !claimantEmail.trim()) {
      error = 'Work URL, claimant name, and contact email are required.';
      return;
    }
    sending = true;
    error = '';
    msg = '';
    try {
      const res = await fetch('/api/copyright/notice', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          work_url_id: workUrl.trim(),
          work_title: workTitle.trim(),
          claimant_name: claimantName.trim(),
          claimant_email: claimantEmail.trim(),
          reason: reason.trim(),
        }),
      });
      const data = await res.json();
      if (data.err === 0) {
        msg = 'Notice received. We review takedown requests promptly.';
        workUrl = '';
        workTitle = '';
        reason = '';
      } else {
        error = data.msg || 'Failed to submit notice.';
      }
    } catch {
      error = 'Network error. Please try again.';
    } finally {
      sending = false;
    }
  }
</script>

<svelte:head>
  <title>Copyright / DMCA — FicNexus</title>
</svelte:head>

<div class="mx-auto max-w-2xl px-4 py-8">
  <h1 class="mb-2 text-2xl font-bold">Copyright &amp; Takedown Notices</h1>
  <p class="mb-6 text-sm opacity-70">
    FicNexus is a non-commercial transformative fanwork archive. We respect the rights of
    creators. If you are a rights holder and believe a work hosted here infringes your copyright,
    submit a notice below. We review all notices promptly and remove confirmed infringing works.
  </p>

  {#if msg}
    <div class="mb-4 rounded bg-emerald-100 px-3 py-2 text-sm text-emerald-800" role="status">{msg}</div>
  {/if}
  {#if error}
    <div class="mb-4 rounded bg-red-100 px-3 py-2 text-sm text-red-800" role="alert">{error}</div>
  {/if}

  <form
    class="space-y-4"
    onsubmit={(e) => {
      e.preventDefault();
      submit();
    }}
  >
    <div>
      <label class="mb-1 block text-sm font-medium" for="workUrl">Work URL or ID *</label>
      <input
        id="workUrl"
        class="w-full rounded border px-3 py-2 text-sm"
        placeholder="e.g. https://ficnexus.polarisocial.xyz/works/1_12345 or 1_12345"
        bind:value={workUrl}
      />
    </div>

    <div>
      <label class="mb-1 block text-sm font-medium" for="workTitle">Work title</label>
      <input
        id="workTitle"
        class="w-full rounded border px-3 py-2 text-sm"
        placeholder="Title of the infringing work"
        bind:value={workTitle}
      />
    </div>

    <div class="grid gap-4 sm:grid-cols-2">
      <div>
        <label class="mb-1 block text-sm font-medium" for="claimantName">Your name / entity *</label>
        <input
          id="claimantName"
          class="w-full rounded border px-3 py-2 text-sm"
          placeholder="Rights holder name"
          bind:value={claimantName}
        />
      </div>
      <div>
        <label class="mb-1 block text-sm font-medium" for="claimantEmail">Contact email *</label>
        <input
          id="claimantEmail"
          type="email"
          class="w-full rounded border px-3 py-2 text-sm"
          placeholder="you@example.com"
          bind:value={claimantEmail}
        />
      </div>
    </div>

    <div>
      <label class="mb-1 block text-sm font-medium" for="reason">Description of the claim</label>
      <textarea
        id="reason"
        class="w-full rounded border px-3 py-2 text-sm"
        rows="4"
        placeholder="Explain the infringement and your ownership of the rights."
        bind:value={reason}
      ></textarea>
    </div>

    <button
      type="submit"
      class="rounded bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-700 disabled:opacity-50"
      disabled={sending}
    >
      {sending ? 'Submitting…' : 'Submit notice'}
    </button>
  </form>
</div>
