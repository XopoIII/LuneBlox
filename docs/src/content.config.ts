import { defineCollection } from 'astro:content';
import { docsLoader, i18nLoader } from '@astrojs/starlight/loaders';
import { docsSchema, i18nSchema } from '@astrojs/starlight/schema';

export const collections = {
	docs: defineCollection({ loader: docsLoader(), schema: docsSchema() }),
	// Starlight reads its UI strings from this collection whether or not a site overrides any, and
	// the build warns about every page when it is missing.
	i18n: defineCollection({ loader: i18nLoader(), schema: i18nSchema() }),
};
