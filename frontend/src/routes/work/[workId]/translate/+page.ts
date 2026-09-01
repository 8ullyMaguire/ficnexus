import type { PageLoad } from './$types';

export const load: PageLoad = ({ params }) => ({ workId: params.workId });
