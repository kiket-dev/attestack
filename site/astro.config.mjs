// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	site: 'https://kiket-dev.github.io',
	base: '/attestack/',
	trailingSlash: 'always',
	integrations: [
		starlight({
			title: 'Attestack',
			description:
				'Local-first proof layer for AI-assisted software work — hash-chained sessions, signed bundles, offline verification.',
			logo: {
				light: './src/assets/logo.svg',
				dark: './src/assets/logo.svg',
				replacesTitle: true,
			},
			defaultLocale: 'root',
			locales: {
				root: {
					label: 'English',
					lang: 'en',
				},
			},
			social: [
				{
					icon: 'github',
					label: 'GitHub',
					href: 'https://github.com/kiket-dev/attestack',
				},
			],
			editLink: {
				baseUrl: 'https://github.com/kiket-dev/attestack/edit/main/',
			},
			customCss: ['./src/styles/custom.css'],
			head: [
				{
					tag: 'script',
					attrs: { is: 'inline' },
					content: `(function(){try{var k='starlight-theme';if(!localStorage.getItem(k))localStorage.setItem(k,'dark');}catch(e){}})();`,
				},
				{
					tag: 'link',
					attrs: {
						rel: 'preconnect',
						href: 'https://fonts.googleapis.com',
					},
				},
				{
					tag: 'link',
					attrs: {
						rel: 'preconnect',
						href: 'https://fonts.gstatic.com',
						crossorigin: true,
					},
				},
				{
					tag: 'link',
					attrs: {
						rel: 'stylesheet',
						href: 'https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600&family=IBM+Plex+Sans:ital,wght@0,400;0,500;0,600;0,700;1,400&display=swap',
					},
				},
			],
			components: {
				// SiteTitle: './src/components/SiteTitle.astro',
			},
			sidebar: [
				{
					label: 'Getting started',
					items: [
						{ label: 'Overview', slug: 'index' },
						{ label: 'Installation', slug: 'getting-started/installation' },
						{ label: 'Quick start', slug: 'getting-started/quickstart' },
						{ label: 'Use cases', slug: 'getting-started/use-cases' },
						{ label: 'Scenarios', slug: 'getting-started/scenarios' },
					],
				},
				{
					label: 'Integrations',
					items: [
						{ label: 'CI integration', slug: 'integrations/ci' },
						{ label: 'Agent setup', slug: 'integrations/agent-setup' },
						{ label: 'Agent guide', slug: 'integrations/agent-guide' },
						{ label: 'Harness integrations', slug: 'integrations/harnesses' },
						{ label: 'Distribution', slug: 'integrations/distribution' },
					],
				},
				{
					label: 'Reference',
					items: [{ label: 'CLI reference', slug: 'reference/cli' }],
				},
				{
					label: 'Concepts',
					items: [
						{ label: 'Data model', slug: 'concepts/data-model' },
						{ label: 'Architecture', slug: 'concepts/architecture' },
						{ label: 'Security model', slug: 'concepts/security-model' },
					],
				},
				{
					label: 'Project',
					items: [
						{ label: 'Product brief', slug: 'project/product-brief' },
						{ label: 'Contributing', slug: 'project/contributing' },
						{ label: 'Releasing', slug: 'project/releasing' },
						{ label: 'Testing strategy', slug: 'project/testing-strategy' },
					],
				},
			],
		}),
	],
});
