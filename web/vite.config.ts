import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig(({ command }) => {
  // E2E uses a dedicated API port so it never reuses (or writes into) a
  // developer's running server.
  const apiTarget =
    (globalThis as { process?: { env?: Record<string, string | undefined> } }).process?.env
      ?.OMNIM_API_URL || 'http://localhost:3000';
  return {
    plugins: [react()],
    base: command === 'build' ? './' : '/',
    build: {
      rollupOptions: {
        input: 'index.html',
      },
      outDir: 'dist',
    },
    server: {
      // Same failure philosophy as the Rust server: an occupied port means a
      // stale instance (or the E2E server on 5174) — fail loudly instead of
      // silently drifting to the next port.
      strictPort: true,
      proxy: {
        '/api': apiTarget,
      },
    },
  };
});
