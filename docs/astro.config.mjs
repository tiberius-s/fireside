import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import mermaid from 'astro-mermaid';

export default defineConfig({
  site: 'https://tiberius.github.io',
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
        github: 'https://github.com/tiberius/fireside',
      },
      editLink: {
        baseUrl: 'https://github.com/tiberius/fireside/edit/main/docs/',
      },
      tableOfContents: {
        minHeadingLevel: 2,
        maxHeadingLevel: 3,
      },
      sidebar: [
        {
          label: 'Crates',
          items: [
            { label: 'fireside-core', link: '/crates/fireside-core/' },
            { label: 'fireside-engine', link: '/crates/fireside-engine/' },
            {
              label: 'fireside-tui',
              items: [
                { label: 'Overview', link: '/crates/fireside-tui/' },
                { label: 'App State Machine', link: '/crates/fireside-tui/app-state-machine/' },
                { label: 'Rendering Pipeline', link: '/crates/fireside-tui/rendering-pipeline/' },
                {
                  label: 'Theme & Design System',
                  link: '/crates/fireside-tui/theme-design-system/',
                },
              ],
            },
            { label: 'fireside-cli', link: '/crates/fireside-cli/' },
          ],
        },
        {
          label: 'Specification',
          items: [
            { label: '§1 Introduction', link: '/spec/introduction/' },
            { label: '§2 Data Model', link: '/spec/data-model/' },
            { label: '§3 Traversal', link: '/spec/traversal/' },
            { label: '§4 Validation', link: '/spec/validation/' },
            { label: '§5 Extensibility', link: '/spec/extensibility/' },
            { label: '§6 Serialization', link: '/spec/serialization/' },
            { label: 'Migration', link: '/spec/migration/' },
            { label: 'Appendix A — Design System', link: '/spec/appendix-design-system/' },
            { label: 'Appendix B — Engine Guidelines', link: '/spec/appendix-engine-guidelines/' },
            { label: 'Appendix C — Content Blocks', link: '/spec/appendix-content-blocks/' },
          ],
        },
        {
          label: 'Schemas',
          autogenerate: { directory: 'schemas' },
        },
        {
          label: 'Reference',
          items: [
            { label: 'Keybindings', link: '/reference/keybindings/' },
            { label: 'Data Model Quick Reference', link: '/reference/data-model-quick-reference/' },
            { label: 'Domain Vocabulary', link: '/reference/domain-vocabulary/' },
          ],
        },
        {
          label: 'Guides',
          items: [
            { label: 'Your First Fireside Session', link: '/guides/getting-started/' },
            { label: 'Branching Adventures', link: '/guides/branching-adventures/' },
            { label: 'For Designers', link: '/guides/for-designers/' },
            { label: 'Theme Authoring', link: '/guides/theme-authoring/' },
            { label: 'Extension Authoring', link: '/guides/extension-authoring/' },
            {
              label: 'Learn Rust with Fireside',
              items: [
                { label: 'Series Overview', link: '/guides/learn-rust/' },
                { label: '1. Your First Data Model', link: '/guides/learn-rust/01-data-model/' },
                { label: '2. Errors That Help', link: '/guides/learn-rust/02-errors/' },
                {
                  label: '3. Ownership, Borrowing, and Collections',
                  link: '/guides/learn-rust/03-ownership/',
                },
                { label: '4. Traits and Polymorphism', link: '/guides/learn-rust/04-traits/' },
                {
                  label: "5. When Derive Isn't Enough",
                  link: '/guides/learn-rust/05-custom-serde/',
                },
                { label: '6. State Machines', link: '/guides/learn-rust/06-state-machines/' },
                {
                  label: '7. Undo/Redo with the Command Pattern',
                  link: '/guides/learn-rust/07-command-pattern/',
                },
                {
                  label: '8. The Elm Architecture in Rust',
                  link: '/guides/learn-rust/08-tea-architecture/',
                },
              ],
            },
          ],
        },
        {
          label: 'Explanation',
          autogenerate: { directory: 'explanation' },
        },
      ],
    }),
  ],
});
