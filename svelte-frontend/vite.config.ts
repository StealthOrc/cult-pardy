import { sveltekit } from '@sveltejs/kit/vite';
import { searchForWorkspaceRoot } from 'vite';
import wasm from 'vite-plugin-wasm';


// Get the parrrent directory of the current working directory
const parentDir : string = searchForWorkspaceRoot(process.cwd());
// Get the root directory of the project
const rootFolder : string = parentDir.substring(0, parentDir.lastIndexOf("\\"));
// Print the root folder


export default {
  plugins: [sveltekit(), wasm()],
  kit: {
    // Any specific configuration for SvelteKit
  },
  server: {
    fs: {
      allow: [rootFolder],
    },
    proxy: {
      '/api': {
        target: 'http://localhost:8000',
        changeOrigin: true,
      },
    }
  },
};