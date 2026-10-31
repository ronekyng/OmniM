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
      proxy: {
        '/api': apiTarget,
      },
    },
  };
});
