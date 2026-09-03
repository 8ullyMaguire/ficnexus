// SPA mode: no SSR, no prerender. The backend serves the static build.
export const ssr = false;
export const prerender = false;

import { registerServiceWorker } from '$lib/pwa/register';
registerServiceWorker();
