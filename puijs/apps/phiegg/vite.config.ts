import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@pui/components': path.resolve(__dirname, '../../src/components'),
      '@pui/commerce': path.resolve(__dirname, '../../src/components/commerce'),
      '@pui/store': path.resolve(__dirname, '../../src/store'),
      '@pui/services': path.resolve(__dirname, '../../src/services'),
      '@pui/api': path.resolve(__dirname, '../../src/api'),
      '@pui/hooks': path.resolve(__dirname, '../../src/hooks'),
      '@pui/providers': path.resolve(__dirname, '../../src/providers'),
      '@pui/styles': path.resolve(__dirname, '../../src/styles'),
      '@pui/tokens': path.resolve(__dirname, '../../src/tokens'),
      '@pui': path.resolve(__dirname, '../../src'),
      'puijs': path.resolve(__dirname, '../../src'),
      'next/link': path.resolve(__dirname, '../pui-book/src/mocks/Link.tsx'),
      'react': path.resolve(__dirname, './node_modules/react'),
      'react-dom': path.resolve(__dirname, './node_modules/react-dom'),
      'react/jsx-runtime': path.resolve(__dirname, './node_modules/react/jsx-runtime'),
    },
    dedupe: ['react', 'react-dom', 'lucide-react'],
  },
  css: {
    preprocessorOptions: {
      scss: {
        api: 'modern-compiler',
      },
    },
  },
  server: {
    port: 3000,
    host: '0.0.0.0',
  },
  build: {
    outDir: 'dist',
  },
});
