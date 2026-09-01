<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { exportUserData } from '$lib/api/social';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));
  let exporting = $state(false);
  let exportMsg = $state('');
  let error = $state('');

  async function handleExport() {
    exporting = true;
    exportMsg = '';
    error = '';
    try {
      const res = await exportUserData();
      if (res.ok) {
        // Trigger download
        const blob = await res.blob();
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'ficnexus-user-data.zip';
        document.body.appendChild(a);
        a.click();
        a.remove();
        URL.revokeObjectURL(url);
        exportMsg = 'Your data export has been downloaded.';
      } else {
        error = 'Export failed.';
      }
    } catch {
      error = 'Export failed.';
    } finally {
      exporting = false;
    }
  }

  let deleting = $state(false);
  let confirmDelete = $state(false);
  let deleteMsg = $state('');

  async function handleDelete() {
    if (!confirmDelete || deleting) return;
    deleting = true;
    deleteMsg = '';
    try {
      const res = await fetch('/api/user/account', { method: 'DELETE', credentials: 'include' });
      const data = await res.json();
      if (data.err === 0) {
        deleteMsg = 'Account deleted. Redirecting…';
        setTimeout(() => {
          localStorage.clear();
          window.location.href = '/';
        }, 1500);
      } else {
        deleteMsg = data.msg || 'Deletion failed.';
      }
    } catch {
      deleteMsg = 'Deletion failed.';
    } finally {
      deleting = false;
    }
  }
</script>

<svelte:head>
  <title>Account — FicNexus</title>
</svelte:head>

<div class="mx-auto max-w-2xl px-4 py-8">
  <h1 class="mb-2 text-2xl font-bold">Account</h1>
  <p class="mb-6 text-sm opacity-70">
    Manage your data. FicNexus respects your GDPR rights: access, rectification, portability, and erasure.
  </p>

  <div class="mb-6 rounded-lg border p-4">
    <h2 class="mb-2 font-semibold">Download your data</h2>
    <p class="mb-3 text-sm opacity-70">
      Get a ZIP archive with all your personal data (profile, bookmarks, comments, follows, reading history).
    </p>
    <button
      class="rounded bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-700 disabled:opacity-50"
      onclick={handleExport}
      disabled={exporting}
    >
      {exporting ? 'Exporting…' : 'Download my data'}
    </button>
    {#if exportMsg}<p class="mt-2 text-sm text-emerald-600">{exportMsg}</p>{/if}
    {#if error}<p class="mt-2 text-sm text-red-600">{error}</p>{/if}
  </div>

  <div class="rounded-lg border border-red-200 p-4">
    <h2 class="mb-2 font-semibold text-red-700">Delete your account</h2>
    <p class="mb-3 text-sm opacity-70">
      Permanently erase your account and all personal data (GDPR Art. 17). This cannot be undone.
      Consider downloading your data first.
    </p>
    {#if !confirmDelete}
      <button
        class="rounded bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700"
        onclick={() => (confirmDelete = true)}
      >
        Delete my account
      </button>
    {:else}
      <div class="flex flex-wrap items-center gap-3">
        <label class="flex items-center gap-2 text-sm">
          <input type="checkbox" bind:checked={confirmDelete} />
          I understand this permanently deletes my account and data.
        </label>
        <button
          class="rounded bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700 disabled:opacity-50"
          onclick={handleDelete}
          disabled={!confirmDelete || deleting}
        >
          {deleting ? 'Deleting…' : 'Confirm deletion'}
        </button>
      </div>
      {#if deleteMsg}<p class="mt-2 text-sm {deleteMsg.includes('deleted') ? 'text-emerald-600' : 'text-red-600'}">{deleteMsg}</p>{/if}
    {/if}
  </div>
</div>
