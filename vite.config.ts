import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  root: 'web',
  plugins: [react(), tailwindcss()],
  build: { outDir: '../dist', emptyOutDir: true },
  server: {
    host: '0.0.0.0',
    allowedHosts: ['localmachine'],
    proxy: { '/api': 'http://100.115.188.46:1900' },
  },
});
