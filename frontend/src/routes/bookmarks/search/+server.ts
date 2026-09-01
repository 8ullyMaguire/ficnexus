/** Redirect deep links /bookmarks/search to the unified /bookmarks?tab=search. */
import { redirect } from '@sveltejs/kit';
import type { RequestHandler } from '@sveltejs/kit';

export const GET: RequestHandler = () => {
  throw redirect(308, '/bookmarks?tab=search');
};
