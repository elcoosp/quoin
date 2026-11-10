import { defineDocsConfig } from '@elcoosp/docs-toolkit/config';
import starlightLinksValidator from 'starlight-links-validator';
import starlightImageZoom from 'starlight-image-zoom';
import starlightHeadingBadges from 'starlight-heading-badges';
import starlightLlmsTxt from 'starlight-llms-txt';


export default await defineDocsConfig({
  site: 'https://quoin-docs.vercel.app',
  domain: 'quoin-docs.vercel.app',
  title: "Quoin",
  description: "One reactive core, many frameworks. Write reactive business logic once in Rust and run it on GPUI, Leptos, Dioxus, Xilem, or Floem.",
  sidebar: [],
  extraStarlight: {
      expressiveCode: {
      	themes: ['github-light', 'github-dark']
      },
      title: 'Quoin',
      description:
        'One reactive core, many frameworks. Write reactive business logic once in Rust and run it on GPUI, Leptos, Dioxus, Xilem, or Floem.',
      favicon: '/favicon.png',
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/elcoosp/quoin' },
      ],
      editLink: {
        baseUrl: 'https://github.com/elcoosp/quoin/edit/main/'
      },
      defaultLocale: 'en',
      locales: {
        root: { label: 'English', lang: 'en' }
      },
      head: [
        {
          tag: 'link',
          attrs: { rel: 'icon', type: 'image/png', href: '/favicon.png' }
        },
      ],
      sidebar: [
        {
          label: 'Start Here',
          items: [
            { label: 'Overview', link: '/' },
          ]
        },
        {
          label: 'Tutorials',
          items: [
            { label: 'Build a Cross-Framework Counter', slug: 'tutorials/build-a-counter' },
          ]
        },
        {
          label: 'Guides',
          items: [
            { label: 'Installation', slug: 'guides/installation' },
            { label: 'Writing Portable Hooks', slug: 'guides/portable-hooks' },
            { label: 'Loading Async Data', slug: 'guides/async-data' },
            { label: 'Cancelling Work', slug: 'guides/cancelling-work' },
            { label: 'Sharing Global State', slug: 'guides/global-state' },
            { label: 'Declarative UI Macros', slug: 'guides/ui-macros' },
            { label: 'Composing UCP Components', slug: 'guides/ucp-components' },
            { label: 'Styling with Tailwind Classes', slug: 'guides/styling' },
            { label: 'Testing with quoin-testkit', slug: 'guides/testing' },
            { label: 'Tracing with quoin-devtools', slug: 'guides/devtools' },
          ]
        },
        {
          label: 'Reference',
          items: [
            { label: 'Signals', slug: 'reference/signals' },
            { label: 'Reactive Context', slug: 'reference/reactive-context' },
            { label: 'Executors & Tasks', slug: 'reference/executors' },
            { label: 'Derived Signals & Memos', slug: 'reference/derived' },
            { label: 'Resources', slug: 'reference/resources' },
            { label: 'Batched Updates', slug: 'reference/batching' },
            { label: 'Cancellation Tokens', slug: 'reference/cancellation' },
            { label: 'Typed Events', slug: 'reference/events' },
            { label: 'Macros', slug: 'reference/macros' },
            { label: 'Universal Component Protocol', slug: 'reference/ucp' },
            { label: 'Feature Flags', slug: 'reference/feature-flags' },
            { label: 'Crates', slug: 'reference/crates' },
            { label: 'quoin-testkit', slug: 'reference/testkit' },
            { label: 'quoin-devtools', slug: 'reference/devtools' },
          ]
        },
        {
          label: 'Explanation',
          items: [
            { label: 'Why Quoin Exists', slug: 'explanation/why-quoin' },
            { label: 'Architecture', slug: 'explanation/architecture' },
            { label: 'The Reactivity Model', slug: 'explanation/reactivity-model' },
            { label: 'Backend Parity', slug: 'explanation/backend-parity' },
          ]
        },
      ],
      plugins: [
        // Fails `astro build` on broken internal links — release blocker.
        starlightLinksValidator(),
        // Click-to-zoom for images.
        starlightImageZoom(),
        // Badges (e.g. "Experimental") rendered inline in headings and the TOC.
        starlightHeadingBadges(),
        // Emits /llms.txt, /llms-small.txt and /llms-full.txt for AI agents.
        starlightLlmsTxt({
          projectName: 'Quoin',
          details: 'Quoin is a framework-agnostic reactive abstraction layer for Rust. It ships signals, derived state, resources, executors, cancellation, global state, declarative UI macros with Tailwind class transpilation, a Universal Component Protocol, a headless testkit, and signal-tracing devtools — all shared across the GPUI, Leptos, Dioxus, Xilem, and Floem adapters.'
        }),
      ]
    },
});
