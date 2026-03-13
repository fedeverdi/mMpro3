import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import path from 'node:path';

// https://vitejs.dev/config
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '~': path.resolve(__dirname, 'src'),
      '@': path.resolve(__dirname, 'src')
    }
  },
  build: {
    rollupOptions: {
      input: {
        main: path.resolve(__dirname, 'index.html'),
        'detached-master-eq': path.resolve(__dirname, 'detached-master-eq.html'),
        'detached-spectrum': path.resolve(__dirname, 'detached-spectrum.html'),
        'detached-aux-master': path.resolve(__dirname, 'detached-aux-master.html'),
        'detached-master-fx': path.resolve(__dirname, 'detached-master-fx.html'),
      },
    },
  },
  server: {
    host: '0.0.0.0', // Espone il server su tutte le interfacce di rete
    port: 5173,
    strictPort: false, // Se la porta è occupata, prova la successiva
  }
});