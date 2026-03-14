import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    host: true,
    proxy: {
      '/api': {
        target: 'http://10.0.0.5:3000',
        changeOrigin: true,
      },
      '/ws': {
        target: 'ws://10.0.0.5:3000',
        ws: true,
        changeOrigin: true,
      }
    }
  }
})