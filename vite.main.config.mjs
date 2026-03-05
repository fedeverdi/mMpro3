import { defineConfig } from 'vite';
import dotenv from 'dotenv';

// Load environment variables from .env file
dotenv.config();

// https://vitejs.dev/config
export default defineConfig({
  define: {
    // Inject EDGE_CONFIG into the built code
    // This makes the connection string available in production builds
    'process.env.EDGE_CONFIG': JSON.stringify(process.env.EDGE_CONFIG || ''),
  }
});
