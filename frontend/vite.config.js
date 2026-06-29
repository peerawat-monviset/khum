import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';

export default defineConfig({
  plugins: [solidPlugin()],
  server: {
    port: 3000,
  },
  build: {
    target: 'esnext',
    outDir: '../public',
    emptyOutDir: false, // Don't wipe the whole public directory since we have locales and icons there
    rollupOptions: {
      output: {
        entryFileNames: 'main.js',
        assetFileNames: (assetInfo) => {
          if (assetInfo.name === 'index.css') return 'main.css';
          return assetInfo.name;
        },
      },
    },
  },
});
