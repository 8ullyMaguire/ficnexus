<script lang="ts">
  // RouteHarness — renders the REAL root +layout.svelte with a REAL route
  // page as its child, simulating SvelteKit's route rendering. The route is
  // derived from the $page store, so when the navigation integration test
  // drives a route change (as the SvelteKit router does on link click), the
  // harness re-renders the child page — and the layout's isRoutePage
  // decision is re-evaluated against the SAME store the app uses.
  import { page } from '$app/stores';
  import Layout from '../../routes/+layout.svelte';
  import RoadmapPage from '../../routes/roadmap/+page.svelte';
  import BlindDatePage from '../../routes/blind-date/+page.svelte';
  import TrendingPage from '../../routes/trending/+page.svelte';
  import SearchPage from '../../routes/search/+page.svelte';
  import AskPage from '../../routes/ask/+page.svelte';
  import LeaderboardPage from '../../routes/leaderboard/+page.svelte';

  let route = $derived($page.url.pathname);
</script>

<Layout>
  {#if route === '/roadmap'}
    <RoadmapPage />
  {:else if route === '/blind-date'}
    <BlindDatePage />
  {:else if route === '/trending'}
    <TrendingPage />
  {:else if route === '/search'}
    <SearchPage />
  {:else if route === '/ask'}
    <AskPage />
  {:else if route === '/leaderboard'}
    <LeaderboardPage />
  {/if}
</Layout>
