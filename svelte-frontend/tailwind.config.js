/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],

	theme: {
		extend: {
			colors: {
				'discord-blue': '#5865F2',
				cultGrey: {
					DEFAULT: "#343336",
					light: "#808080",
				},
        		cultPink: '#EB60D6',
        		cultTurq: {
					DEFAULT: '#61eae0',
					dark: '#6FCCCF',
				},
				//TODO: still up for review
				cultOne: "#012A36",
				cultTwo: "#29274C",
				cultThree: "#7E52A0",
			}
		}
	},

	plugins: [
  	  function ({ addComponents }) {
  	    	const buttons = {
  	    	  '.cult-btn-menu': {
  	    	    '@apply bg-cultTurq text-black font-bold py-2 px-4 rounded focus:outline-none hover:bg-cultPink border-2 border-cultTurq hover:border-cultPink focus:border-cultPink transition-all duration-200': {},
  	    	  },
  	    	}
			const surfaces = {
				'.cult-surface': {
					'@apply bg-cultGrey rounded-lg shadow-xl': {},
				},
				'.cult-bg-gradient': {
					'@apply bg-gradient-to-br from-cultTwo to-cultThree': {},
				},
			}
  	    	addComponents(buttons);
			addComponents(surfaces);
		},
  	],
};
