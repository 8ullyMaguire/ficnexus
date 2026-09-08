<script lang="ts">
  // PollBar — renders a ForumPoll attached to a topic, with live voting.
  // Archive-only styling (modern mode was removed Sep 2026).
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { voteOnPoll, type ForumPoll } from '$lib/api/forum';

  let { poll }: { poll: ForumPoll } = $props();
  let voting = $state(false);
  let error = $state('');
  let userVote = $state<number | null>(null);

  function selectedOptions(): number[] {
    if (userVote === null) return [];
    return [userVote];
  }

  async function handleVote(optionId: number) {
    if (!auth.isLoggedIn) {
      error = t('forum.loginToParticipate');
      return;
    }
    const alreadyVoted = userVote === optionId;
    if (alreadyVoted && !poll.allow_change) return;
    voting = true;
    error = '';
    try {
      const res = await voteOnPoll(poll.id, optionId);
      if (res.err === 0) {
        userVote = optionId;
      } else {
        error = res.msg ?? t('forum.actionFailed');
      }
    } catch {
      error = t('forum.actionFailed');
    } finally {
      voting = false;
    }
  }

  function isVoted(optionId: number): boolean {
    return userVote === optionId;
  }

  function canVote(optionId: number): boolean {
    if (poll.is_closed) return false;
    const selected = selectedOptions();
    if (selected.includes(optionId)) return poll.allow_change;
    return selected.length < poll.max_selections;
  }

  function optionBarWidth(vote_count: number): string {
    const max = Math.max(...poll.options.map(o => o.vote_count), 1);
    return `${(vote_count / max) * 100}%`;
  }
</script>

{#if poll.is_closed}
  <div class="arc-poll">
    <div class="arc-question"><strong>{poll.question}</strong> <span class="arc-chip">{t('forum.pollClosed')}</span></div>
    <ul class="arc-options">
      {#each poll.options as opt (opt.id)}
        <li class="arc-option">
          <span class="arc-opt-text">{opt.text}</span>
          <div class="arc-bar-wrap">
            <div class="arc-bar" style="width: {optionBarWidth(opt.vote_count)}"></div>
            <span class="arc-count">{opt.vote_count}</span>
          </div>
        </li>
      {/each}
    </ul>
    <div class="arc-meta muted">{poll.options.reduce((s, o) => s + o.vote_count, 0)} {t('forum.totalVotes')}</div>
  </div>
{:else}
  <div class="arc-poll">
    <div class="arc-question"><strong>{poll.question}</strong></div>
    <ul class="arc-options">
      {#each poll.options as opt (opt.id)}
        <li>
          <button
            class="arc-btn"
            class:arc-selected={isVoted(opt.id)}
            disabled={voting || !canVote(opt.id)}
            type="button"
            onclick={() => handleVote(opt.id)}
          >
            {opt.text} · {opt.vote_count}
          </button>
        </li>
      {/each}
    </ul>
    <div class="arc-meta muted">
      {poll.allow_change ? t('forum.pollCanChange') : t('forum.pollNoChange')} ·
      {t('forum.pollMultiSelect', { n: poll.max_selections })}
    </div>
  </div>
{/if}

{#if error}
  <p class="error-card">{error}</p>
{/if}

<style>
  .arc-poll {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.75rem 1rem;
    margin: 0.75rem 0;
    background: var(--archive-bg, #ffffff);
    font-family: Georgia, 'Times New Roman', serif;
  }
  .arc-question { font-size: 1em; margin-bottom: 0.5rem; }
  .arc-chip {
    font-size: 0.7em;
    text-transform: uppercase;
    color: var(--archive-muted, #666666);
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.1em 0.4em;
    border-radius: 999px;
    margin-left: 0.4em;
  }
  .arc-options { list-style: none; margin: 0.5rem 0; padding: 0; }
  .arc-options li { margin: 0.35rem 0; }
  .arc-option { display: flex; align-items: center; gap: 0.5rem; }
  .arc-opt-text { flex: 1; }
  .arc-bar-wrap { position: relative; flex: 2; height: 1.25rem; background: var(--archive-bg-raised, #f5f5f5); border: 1px solid var(--archive-border, #dddddd); }
  .arc-bar { height: 100%; background: var(--archive-link, #990000); }
  .arc-count { position: absolute; right: 0.4em; font-size: 0.8em; font-weight: 700; }
  .arc-btn {
    display: block;
    width: 100%;
    text-align: left;
    padding: 0.4em 0.6em;
    color: var(--archive-link, #990000);
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg, #ffffff);
    font-family: Georgia, 'Times New Roman', serif;
    cursor: pointer;
  }
  .arc-selected { background: var(--archive-link, #990000); color: #fff; }
  .arc-meta { font-size: 0.8em; margin-top: 0.4em; }
  .muted { color: var(--archive-muted, #666666); }
  .error-card { color: var(--archive-link, #990000); }
</style>

