import type { PageLoad } from './$types';

// SPA catch-all so deep links and refresh work under adapter-static fallback.
export const load: PageLoad = async ({ url }) => {
  return { path: url.pathname };
};
