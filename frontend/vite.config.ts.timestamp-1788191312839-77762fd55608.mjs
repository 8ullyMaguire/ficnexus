// vite.config.ts
import { sveltekit } from "file:///personal/documents/code/rust/ficnexus/frontend/node_modules/@sveltejs/kit/src/exports/vite/index.js";
import { defineConfig } from "file:///personal/documents/code/rust/ficnexus/frontend/node_modules/vitest/dist/config.js";
var vite_config_default = defineConfig({
  plugins: [sveltekit()],
  resolve: {
    conditions: ["browser"]
  },
  // Dep-optimize cache stays under node_modules/.vite (default, pinned here so
  // nothing ever lands outside node_modules).
  cacheDir: "node_modules/.vite",
  server: {
    port: 5173,
    host: "0.0.0.0",
    proxy: {
      "/api": {
        target: "http://localhost:8004",
        changeOrigin: true
      }
    }
  },
  test: {
    // RAM guard: vitest 2.x workers-per-CPU defaults can spawn 15+ workers x
    // ~2-3GB each on a 16-core box (2026-08-09 OOM hang on gamingpc). Cap the
    // thread pool. Threads (not forks): the forks pool times out on this NFS
    // box, and a hard-killed run also leaks `*.timestamp-*.mjs` config temp
    // files into the repo root (vite deletes them in a finally-block that
    // never runs on SIGKILL).
    pool: "threads",
    environment: "jsdom",
    environmentOptions: { jsdom: { url: "http://localhost" } },
    globals: true,
    setupFiles: ["src/test-setup.ts"],
    include: ["src/**/*.test.{ts,svelte}"],
    // The E2E/integration suite (src/lib/test/) runs via `npm run test:e2e`
    // with vitest.e2e.config.ts; exclude it from the default unit run so the
    // main suite stays green while the roadmap/tropes route-render tests are
    // intentionally red (layout routePages bug).
    exclude: ["src/lib/test/**", "node_modules/**"],
    // Coverage gate — run via `npm run coverage`. Thresholds target the
    // logic layer (src/lib: API clients, stores, utils) which is the
    // meaningful, testable signal; full Svelte-page markup coverage is noisy
    // and drags the gate below any useful bar. CI fails the build when the
    // lib thresholds are not met.
    coverage: {
      provider: "v8",
      reporter: ["text", "json-summary", "html"],
      reportsDirectory: "coverage",
      include: ["src/lib/**"],
      exclude: [
        "src/lib/test/**",
        "**/*.test.{ts,svelte}",
        "src/test-setup.ts"
      ],
      thresholds: {
        // Baseline 2026-08-08: 70.27% lines / 58.08% funcs on src/lib.
        // Coverage wave (2026-08-09): 90.3% lines / 82.66% funcs / 81.31%
        // branches — set ~4-5pts under the achieved baseline so the gate is
        // a regression guard today, with headroom toward the 80/70 target
        // (now exceeded; the guard keeps future refactors honest).
        lines: 85,
        functions: 78,
        branches: 76,
        statements: 85
      }
    }
  }
});
export {
  vite_config_default as default
};
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsidml0ZS5jb25maWcudHMiXSwKICAic291cmNlc0NvbnRlbnQiOiBbImNvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9kaXJuYW1lID0gXCIvcGVyc29uYWwvZG9jdW1lbnRzL2NvZGUvcnVzdC9maWNuZXh1cy9mcm9udGVuZFwiO2NvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9maWxlbmFtZSA9IFwiL3BlcnNvbmFsL2RvY3VtZW50cy9jb2RlL3J1c3QvZmljbmV4dXMvZnJvbnRlbmQvdml0ZS5jb25maWcudHNcIjtjb25zdCBfX3ZpdGVfaW5qZWN0ZWRfb3JpZ2luYWxfaW1wb3J0X21ldGFfdXJsID0gXCJmaWxlOi8vL3BlcnNvbmFsL2RvY3VtZW50cy9jb2RlL3J1c3QvZmljbmV4dXMvZnJvbnRlbmQvdml0ZS5jb25maWcudHNcIjtpbXBvcnQgeyBzdmVsdGVraXQgfSBmcm9tICdAc3ZlbHRlanMva2l0L3ZpdGUnO1xuLy8gTk9URTogbXVzdCBjb21lIGZyb20gJ3ZpdGVzdC9jb25maWcnIChub3QgJ3ZpdGUnKSBvciB0aGUgYHRlc3RgIGtleSBiZWxvd1xuLy8gZmFpbHMgVXNlckNvbmZpZ0V4cG9ydCB0eXBlLWNoZWNraW5nLlxuaW1wb3J0IHsgZGVmaW5lQ29uZmlnIH0gZnJvbSAndml0ZXN0L2NvbmZpZyc7XG5cbmV4cG9ydCBkZWZhdWx0IGRlZmluZUNvbmZpZyh7XG4gIHBsdWdpbnM6IFtzdmVsdGVraXQoKV0sXG4gIHJlc29sdmU6IHtcbiAgICBjb25kaXRpb25zOiBbJ2Jyb3dzZXInXSxcbiAgfSxcbiAgLy8gRGVwLW9wdGltaXplIGNhY2hlIHN0YXlzIHVuZGVyIG5vZGVfbW9kdWxlcy8udml0ZSAoZGVmYXVsdCwgcGlubmVkIGhlcmUgc29cbiAgLy8gbm90aGluZyBldmVyIGxhbmRzIG91dHNpZGUgbm9kZV9tb2R1bGVzKS5cbiAgY2FjaGVEaXI6ICdub2RlX21vZHVsZXMvLnZpdGUnLFxuICBzZXJ2ZXI6IHtcbiAgICBwb3J0OiA1MTczLFxuICAgIGhvc3Q6ICcwLjAuMC4wJyxcbiAgICBwcm94eToge1xuICAgICAgJy9hcGknOiB7XG4gICAgICAgIHRhcmdldDogJ2h0dHA6Ly9sb2NhbGhvc3Q6ODAwNCcsXG4gICAgICAgIGNoYW5nZU9yaWdpbjogdHJ1ZSxcbiAgICAgIH0sXG4gICAgfSxcbiAgfSxcbiAgdGVzdDoge1xuICAgIC8vIFJBTSBndWFyZDogdml0ZXN0IDIueCB3b3JrZXJzLXBlci1DUFUgZGVmYXVsdHMgY2FuIHNwYXduIDE1KyB3b3JrZXJzIHhcbiAgICAvLyB+Mi0zR0IgZWFjaCBvbiBhIDE2LWNvcmUgYm94ICgyMDI2LTA4LTA5IE9PTSBoYW5nIG9uIGdhbWluZ3BjKS4gQ2FwIHRoZVxuICAgIC8vIHRocmVhZCBwb29sLiBUaHJlYWRzIChub3QgZm9ya3MpOiB0aGUgZm9ya3MgcG9vbCB0aW1lcyBvdXQgb24gdGhpcyBORlNcbiAgICAvLyBib3gsIGFuZCBhIGhhcmQta2lsbGVkIHJ1biBhbHNvIGxlYWtzIGAqLnRpbWVzdGFtcC0qLm1qc2AgY29uZmlnIHRlbXBcbiAgICAvLyBmaWxlcyBpbnRvIHRoZSByZXBvIHJvb3QgKHZpdGUgZGVsZXRlcyB0aGVtIGluIGEgZmluYWxseS1ibG9jayB0aGF0XG4gICAgLy8gbmV2ZXIgcnVucyBvbiBTSUdLSUxMKS5cbiAgICBwb29sOiAndGhyZWFkcycsXG4gICAgZW52aXJvbm1lbnQ6ICdqc2RvbScsXG4gICAgZW52aXJvbm1lbnRPcHRpb25zOiB7IGpzZG9tOiB7IHVybDogJ2h0dHA6Ly9sb2NhbGhvc3QnIH0gfSxcbiAgICBnbG9iYWxzOiB0cnVlLFxuICAgIHNldHVwRmlsZXM6IFsnc3JjL3Rlc3Qtc2V0dXAudHMnXSxcbiAgICBpbmNsdWRlOiBbJ3NyYy8qKi8qLnRlc3Que3RzLHN2ZWx0ZX0nXSxcbiAgICAvLyBUaGUgRTJFL2ludGVncmF0aW9uIHN1aXRlIChzcmMvbGliL3Rlc3QvKSBydW5zIHZpYSBgbnBtIHJ1biB0ZXN0OmUyZWBcbiAgICAvLyB3aXRoIHZpdGVzdC5lMmUuY29uZmlnLnRzOyBleGNsdWRlIGl0IGZyb20gdGhlIGRlZmF1bHQgdW5pdCBydW4gc28gdGhlXG4gICAgLy8gbWFpbiBzdWl0ZSBzdGF5cyBncmVlbiB3aGlsZSB0aGUgcm9hZG1hcC90cm9wZXMgcm91dGUtcmVuZGVyIHRlc3RzIGFyZVxuICAgIC8vIGludGVudGlvbmFsbHkgcmVkIChsYXlvdXQgcm91dGVQYWdlcyBidWcpLlxuICAgIGV4Y2x1ZGU6IFsnc3JjL2xpYi90ZXN0LyoqJywgJ25vZGVfbW9kdWxlcy8qKiddLFxuICAgIC8vIENvdmVyYWdlIGdhdGUgXHUyMDE0IHJ1biB2aWEgYG5wbSBydW4gY292ZXJhZ2VgLiBUaHJlc2hvbGRzIHRhcmdldCB0aGVcbiAgICAvLyBsb2dpYyBsYXllciAoc3JjL2xpYjogQVBJIGNsaWVudHMsIHN0b3JlcywgdXRpbHMpIHdoaWNoIGlzIHRoZVxuICAgIC8vIG1lYW5pbmdmdWwsIHRlc3RhYmxlIHNpZ25hbDsgZnVsbCBTdmVsdGUtcGFnZSBtYXJrdXAgY292ZXJhZ2UgaXMgbm9pc3lcbiAgICAvLyBhbmQgZHJhZ3MgdGhlIGdhdGUgYmVsb3cgYW55IHVzZWZ1bCBiYXIuIENJIGZhaWxzIHRoZSBidWlsZCB3aGVuIHRoZVxuICAgIC8vIGxpYiB0aHJlc2hvbGRzIGFyZSBub3QgbWV0LlxuICAgIGNvdmVyYWdlOiB7XG4gICAgICBwcm92aWRlcjogJ3Y4JyxcbiAgICAgIHJlcG9ydGVyOiBbJ3RleHQnLCAnanNvbi1zdW1tYXJ5JywgJ2h0bWwnXSxcbiAgICAgIHJlcG9ydHNEaXJlY3Rvcnk6ICdjb3ZlcmFnZScsXG4gICAgICBpbmNsdWRlOiBbJ3NyYy9saWIvKionXSxcbiAgICAgIGV4Y2x1ZGU6IFtcbiAgICAgICAgJ3NyYy9saWIvdGVzdC8qKicsXG4gICAgICAgICcqKi8qLnRlc3Que3RzLHN2ZWx0ZX0nLFxuICAgICAgICAnc3JjL3Rlc3Qtc2V0dXAudHMnLFxuICAgICAgXSxcbiAgICAgIHRocmVzaG9sZHM6IHtcbiAgICAgICAgLy8gQmFzZWxpbmUgMjAyNi0wOC0wODogNzAuMjclIGxpbmVzIC8gNTguMDglIGZ1bmNzIG9uIHNyYy9saWIuXG4gICAgICAgIC8vIENvdmVyYWdlIHdhdmUgKDIwMjYtMDgtMDkpOiA5MC4zJSBsaW5lcyAvIDgyLjY2JSBmdW5jcyAvIDgxLjMxJVxuICAgICAgICAvLyBicmFuY2hlcyBcdTIwMTQgc2V0IH40LTVwdHMgdW5kZXIgdGhlIGFjaGlldmVkIGJhc2VsaW5lIHNvIHRoZSBnYXRlIGlzXG4gICAgICAgIC8vIGEgcmVncmVzc2lvbiBndWFyZCB0b2RheSwgd2l0aCBoZWFkcm9vbSB0b3dhcmQgdGhlIDgwLzcwIHRhcmdldFxuICAgICAgICAvLyAobm93IGV4Y2VlZGVkOyB0aGUgZ3VhcmQga2VlcHMgZnV0dXJlIHJlZmFjdG9ycyBob25lc3QpLlxuICAgICAgICBsaW5lczogODUsXG4gICAgICAgIGZ1bmN0aW9uczogNzgsXG4gICAgICAgIGJyYW5jaGVzOiA3NixcbiAgICAgICAgc3RhdGVtZW50czogODUsXG4gICAgICB9LFxuICAgIH0sXG4gIH0sXG59KTtcbiJdLAogICJtYXBwaW5ncyI6ICI7QUFBK1QsU0FBUyxpQkFBaUI7QUFHelYsU0FBUyxvQkFBb0I7QUFFN0IsSUFBTyxzQkFBUSxhQUFhO0FBQUEsRUFDMUIsU0FBUyxDQUFDLFVBQVUsQ0FBQztBQUFBLEVBQ3JCLFNBQVM7QUFBQSxJQUNQLFlBQVksQ0FBQyxTQUFTO0FBQUEsRUFDeEI7QUFBQTtBQUFBO0FBQUEsRUFHQSxVQUFVO0FBQUEsRUFDVixRQUFRO0FBQUEsSUFDTixNQUFNO0FBQUEsSUFDTixNQUFNO0FBQUEsSUFDTixPQUFPO0FBQUEsTUFDTCxRQUFRO0FBQUEsUUFDTixRQUFRO0FBQUEsUUFDUixjQUFjO0FBQUEsTUFDaEI7QUFBQSxJQUNGO0FBQUEsRUFDRjtBQUFBLEVBQ0EsTUFBTTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBLElBT0osTUFBTTtBQUFBLElBQ04sYUFBYTtBQUFBLElBQ2Isb0JBQW9CLEVBQUUsT0FBTyxFQUFFLEtBQUssbUJBQW1CLEVBQUU7QUFBQSxJQUN6RCxTQUFTO0FBQUEsSUFDVCxZQUFZLENBQUMsbUJBQW1CO0FBQUEsSUFDaEMsU0FBUyxDQUFDLDJCQUEyQjtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUEsSUFLckMsU0FBUyxDQUFDLG1CQUFtQixpQkFBaUI7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUEsSUFNOUMsVUFBVTtBQUFBLE1BQ1IsVUFBVTtBQUFBLE1BQ1YsVUFBVSxDQUFDLFFBQVEsZ0JBQWdCLE1BQU07QUFBQSxNQUN6QyxrQkFBa0I7QUFBQSxNQUNsQixTQUFTLENBQUMsWUFBWTtBQUFBLE1BQ3RCLFNBQVM7QUFBQSxRQUNQO0FBQUEsUUFDQTtBQUFBLFFBQ0E7QUFBQSxNQUNGO0FBQUEsTUFDQSxZQUFZO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBLFFBTVYsT0FBTztBQUFBLFFBQ1AsV0FBVztBQUFBLFFBQ1gsVUFBVTtBQUFBLFFBQ1YsWUFBWTtBQUFBLE1BQ2Q7QUFBQSxJQUNGO0FBQUEsRUFDRjtBQUNGLENBQUM7IiwKICAibmFtZXMiOiBbXQp9Cg==
