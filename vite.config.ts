import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import tailwindcss from '@tailwindcss/vite';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: { '@': '/src' },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: { ignored: ['**/src-tauri/**'] },
    proxy: {
      '/api': {
        target: 'http://localhost:14200',
        changeOrigin: true,
      },
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: process.env.TAURI_ENV_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    rollupOptions: {
      output: {
        manualChunks: {
          'vendor-vue': ['vue', 'pinia'],
          'vendor-markdown': ['markdown-it', '@traptitech/markdown-it-katex'],
          'vendor-highlight': ['highlight.js'],
          'vendor-tauri': ['@tauri-apps/api/core', '@tauri-apps/plugin-dialog', '@tauri-apps/plugin-fs'],
        },
      },
    },
    chunkSizeWarningLimit: 1000,
  },
});
