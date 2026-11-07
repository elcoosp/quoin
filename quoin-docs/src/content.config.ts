import { defineCollection } from 'astro:content';
import { docsLoader } from '@astrojs/starlight/loaders';
import { docsSchema } from '@astrojs/starlight/schema';
import { z } from 'astro/zod';

export const collections = {
  docs: defineCollection({
    loader: docsLoader(),
    schema: docsSchema({
      extend: z.object({
        // Diátaxis information type — one type per page, never mixed.
        // Required on every documentation page so the build enforces it.
        type: z.enum(['tutorial', 'how-to', 'reference', 'explanation']),
      }),
    }),
  }),
};
