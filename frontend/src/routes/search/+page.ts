import type { PageLoad } from './$types';

// Read URL search params and pass them to the component.
export const load: PageLoad = async ({ url }) => {
  return {
    q: url.searchParams.get('q') ?? '',
    tab: url.searchParams.get('tab') ?? 'work',
  };
};
