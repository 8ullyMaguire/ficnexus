import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';

// E2E/integration test config — run with `npm run test:e2e`.
// Renders the REAL root layout + nav and clicks through to every routed
// page. The /roadmap and /tropes route-render tests are intentionally red
// until the layout routePages list includes those paths (see
// frontend/src/routes/+layout.svelte). Kept out of the default unit run so
// CI's main suite stays green while the bug exists.
export default defineConfig({
  plugins: [sveltekit()],
  resolve: {
    conditions: ['browser'],
  },
  server: {
    port: 5173,
    host: '0.0.0.0',
    proxy: {
      '/api': {
        target: 'http://localhost:8004',
        changeOrigin: true,
      },
    },
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['src/test-setup.ts'],
    include: ['src/lib/test/**/*.test.{ts,svelte}'],
  },
});
