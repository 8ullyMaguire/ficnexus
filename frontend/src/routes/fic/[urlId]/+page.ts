import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

// /fic/[urlId] was renamed to /works/[urlId]. Keep old links working by
// redirecting to the canonical route (308 = permanent, preserves method).
export const load: PageLoad = ({ params }) => {
  redirect(308, `/works/${encodeURIComponent(params.urlId)}`);
};
