/// <reference types="vitest" />
import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  root: '.',
  build: {
    rollupOptions: {
      input: {
        menu: resolve(__dirname, 'src/menu/index.html'),
        addItem: resolve(__dirname, 'src/dialogs/add-item/index.html'),
        approve: resolve(__dirname, 'src/dialogs/approve/index.html'),
        categorySettings: resolve(__dirname, 'src/dialogs/category-settings/index.html'),
      },
    },
  },
  test: {
    environment: 'jsdom',
    globals: true,
  },
});
