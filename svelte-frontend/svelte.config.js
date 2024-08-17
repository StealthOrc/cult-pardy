
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import fs from 'fs';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	// Consult https://kit.svelte.dev/docs/integrations#preprocessors
	// for more information about preprocessors
	preprocess: vitePreprocess(),

	kit: {
		// adapter-auto only supports some environments, see https://kit.svelte.dev/docs/adapter-auto for a list.
		// If your environment is not supported, or you settled on a specific environment, switch out the adapter.
		// See https://kit.svelte.dev/docs/adapters for more information about adapters.
		adapter: adapter({
			// default options are shown. On some platforms
			// these options are set automatically — see below
			pages: 'build',
			assets: 'build',
			fallback: "index.html",
			precompress: false,
			strict: true
		}),
		
		paths: {
			//TODO PRODUTION IP NEED TO BE SET, Currently local Test IP set
			assets:  getAssetPath(),
		}
	}
};


function getAssetPath()  { 
	try {
		let file = fs.readFileSync('../Settings.toml', 'utf-8');
		let uri = 'http://localhost:8000/assets';
		const settings = parseToml(file);
		if (settings && settings.frontend_settings && settings.frontend_settings.host && settings.frontend_settings.port && settings.frontend_settings.ssl) {
			let ssl = settings.frontend_settings.ssl ? 'https://' : 'http://';
			uri = ssl + settings.frontend_settings.host + ':' + settings.frontend_settings.port + '/assets';
		}
		return uri;
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	} catch (err ) {
		return 'http://localhost:8000/assets';
	}
}

export default config;

function parseToml(tomlString) {
	const result = {};
	let currentSection = result;
  
	tomlString.split('\n').forEach(line => {
	  line = line.trim();
	  if (!line || line.startsWith('#')) return; 

	  if (line.startsWith('[') && line.endsWith(']')) {
		const section = line.slice(1, -1).trim();
		result[section] = {};
		currentSection = result[section];
	  } else if (line.includes('=')) {
		let [key, value] = line.split('=').map(part => part.trim());
		if (value.startsWith('"') && value.endsWith('"')) {
		  value = value.slice(1, -1);
		} else if (value === 'true' || value === 'false') {
		  value = value === 'true'; 
		} else if (!isNaN(value)) {
		  value = Number(value); 
		}
  
		currentSection[key] = value;
	  }
	});
  
	return result;
  }