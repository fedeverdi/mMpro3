import { defineConfig } from 'vite';
import dotenv from 'dotenv';

// Load environment variables from .env for build time
dotenv.config();

// https://vitejs.dev/config
export default defineConfig({
  define: {
    // Inject EDGE_CONFIG at build time
    // Vite will replace all occurrences of process.env.EDGE_CONFIG with the actual value
    'process.env.EDGE_CONFIG': JSON.stringify(process.env.EDGE_CONFIG || ''),
  }
});
