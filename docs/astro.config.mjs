// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import starlightLinksValidator from 'starlight-links-validator';
import starlightLlmsTxt from 'starlight-llms-txt';

// The sidebar, in reading order. The llms.txt files below follow the same order.
const sidebar = [
	{
		label: 'Getting Started',
		items: [
			'getting-started/installation',
			'getting-started/quick-start',
			'getting-started/cli',
			'getting-started/editor-setup',
		],
	},
	{
		label: 'Guides',
		items: [
			'guides/why-luneblox',
			'guides/migrating-from-lune',
			'guides/running-scripts',
			'guides/files',
			'guides/networking',
			'guides/processes',
			'guides/task-scheduler',
			'guides/roblox-files',
			'guides/standalone-executables',
			'guides/fast-flags',
			'guides/keeping-luau-current',
		],
	},
	{
		// The library pages are written by scripts/generate-reference.mjs from the typedefs.
		label: 'Reference',
		items: [
			'reference/datetime',
			'reference/fs',
			'reference/luau',
			'reference/net',
			'reference/process',
			'reference/regex',
			'reference/roblox',
			'reference/serde',
			'reference/stdio',
			'reference/task',
			'reference/environment-variables',
			'reference/directories',
		],
	},
	'changelog',
];

// The plain-text copies of the site for AI assistants.
const llmsTxt = {
	details: [
		'LuneBlox is a standalone Luau runtime, a hard fork of Lune that runs the Luau version Roblox runs',
		'with the fast flags the live Roblox client sets. The command is `luneblox`; the script folders',
		'(`lune`, `.lune`) and the `@lune/*` libraries keep their names. Take API names from these pages.',
	].join('\n'),
	promote: ['index*', ...sidebar.flatMap((entry) => (typeof entry === 'string' ? [entry] : entry.items))],
	demote: ['changelog'],
	exclude: ['changelog'],
	minify: { note: false },
	customSelectors: { all: ['h1'] },
};

export default defineConfig({
	vite: {
		build: {
			rolldownOptions: {
				// Astro marks every MDX page with its own "use astro:head-inject" directive, and Vite's
				// bundler warns once per page that it does not know it. Astro reads the directive itself,
				// so only that warning is dropped; every other one still prints.
				onwarn(warning, warn) {
					if (warning.code === 'MODULE_LEVEL_DIRECTIVE' && warning.message.includes('astro:head-inject')) {
						return;
					}
					warn(warning);
				},
			},
		},
	},
	site: 'https://xopoiii.github.io',
	base: '/LuneBlox',
	integrations: [
		starlight({
			title: 'LuneBlox',
			// src/pages/404.astro says why.
			disable404Route: true,
			description: 'A standalone Luau runtime that runs Luau the way Roblox runs it.',
			logo: { src: './src/assets/logo.png', alt: 'LuneBlox' },
			favicon: '/favicon-32.png',
			head: [
				{ tag: 'link', attrs: { rel: 'icon', type: 'image/png', sizes: '192x192', href: '/LuneBlox/favicon.png' } },
				{ tag: 'link', attrs: { rel: 'apple-touch-icon', href: '/LuneBlox/apple-touch-icon.png' } },
				{ tag: 'meta', attrs: { property: 'og:image', content: 'https://xopoiii.github.io/LuneBlox/og.png' } },
				{ tag: 'meta', attrs: { name: 'twitter:image', content: 'https://xopoiii.github.io/LuneBlox/og.png' } },
			],
			social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/XopoIII/LuneBlox' }],
			editLink: { baseUrl: 'https://github.com/XopoIII/LuneBlox/edit/main/docs/' },
			lastUpdated: true,
			customCss: ['./src/styles/custom.css'],
			expressiveCode: { themes: ['catppuccin-mocha', 'catppuccin-latte'] },
			plugins: [
				// The changelog page imports CHANGELOG.md, and one of Lune's old entries links to the
				// repository's README by a path that only resolves on GitHub.
				starlightLinksValidator({ exclude: ['/README.md'] }),
				starlightLlmsTxt(llmsTxt),
			],
			sidebar,
		}),
	],
});
