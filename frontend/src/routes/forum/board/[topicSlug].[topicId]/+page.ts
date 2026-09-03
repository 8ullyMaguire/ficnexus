import type { PageLoad } from './$types';

// /forum/board/{topicSlug}.{topicId} — canonical topic URL. The path segment
// carries both the human-readable slug and the numeric id separated by a dot.
// The page itself resolves the topic by slug first (fresh URL), and falls
// back to the numeric id when the slug no longer matches (renamed topic or
// legacy link).
export const load: PageLoad = async ({ params }) => {
  return {
    topicSlug: params.topicSlug,
    topicId: params.topicId,
  };
};
