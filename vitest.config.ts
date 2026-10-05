import { fileURLToPath } from 'node:url';
import vue from '@vitejs/plugin-vue';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  test: {
    environment: 'jsdom',
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    passWithNoTests: false,
    coverage: {
      include: ['src/**/*.{ts,vue}'],
      exclude: ['src/**/*.test.ts', 'src/locales/**'],
    },
  },
});
