import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import mermaid from 'astro-mermaid';

export default defineConfig({
  site: 'https://tiberius-s.github.io',
  base: '/fireside',
  output: 'static',
  trailingSlash: 'always',
  integrations: [
    mermaid(),
    starlight({
      title: 'Fireside',
      description: 'A portable format for branching presentations and lessons.',
      disable404Route: true,
      lastUpdated: true,
      social: {
        github: 'https://github.com/tiberius-s/fireside',
      },
      editLink: {
        baseUrl: 'https://github.com/tiberius-s/fireside/edit/main/docs/',
      },
      tableOfContents: {
        minHeadingLevel: 2,
        maxHeadingLevel: 3,
      },
      sidebar: [
        {
          label: 'Specification',
          items: [
            { label: '§1 Introduction', link: '/spec/introduction/' },
            { label: '§2 Data Model', link: '/spec/data-model/' },
            { label: '§3 Traversal', link: '/spec/traversal/' },
            { label: '§4 Validation', link: '/spec/validation/' },
            { label: '§5 Serialization', link: '/spec/serialization/' },
            { label: 'Mental Models', link: '/spec/mental-models/' },
            { label: 'Appendix B — Engine Guidelines', link: '/spec/appendix-engine-guidelines/' },
            { label: 'Appendix C — Content Blocks', link: '/spec/appendix-content-blocks/' },
            {
              label: 'Appendix D — Engine Extensions',
              link: '/spec/appendix-engine-extensions/',
            },
          ],
        },
        {
          label: 'Guides',
          items: [
            { label: 'Your First Fireside Graph', link: '/guides/getting-started/' },
            { label: 'Authoring a Deck in Markdown', link: '/guides/authoring-markdown/' },
            { label: 'Presenting a Deck', link: '/guides/presenting/' },
          ],
        },
        {
          label: 'Reference',
          items: [
            { label: 'CLI Reference', link: '/reference/cli/' },
            { label: 'Data Model Quick Reference', link: '/reference/data-model-quick-reference/' },
            { label: 'Domain Vocabulary', link: '/reference/domain-vocabulary/' },
            { label: 'Conformance', link: '/reference/conformance/' },
          ],
        },
      ],
    }),
  ],
});
