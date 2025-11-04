import path from 'node:path'
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

export default defineConfig({
  plugins: [
    react({
      babel: {
        plugins: ['babel-plugin-react-compiler'],
      },
    }),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          'react-vendor': ['react', 'react-dom', 'react-router', 'react-router-dom'],
          'urql-vendor': ['urql', '@urql/exchange-graphcache', '@urql/exchange-auth'],
          'chakra-vendor': ['@chakra-ui/react'],
        },
      },
    },
  },
})
