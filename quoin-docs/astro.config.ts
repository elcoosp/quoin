import { defineDocsConfig } from '@elcoosp/docs-toolkit/config';
import starlightLinksValidator from 'starlight-links-validator';
import starlightImageZoom from 'starlight-image-zoom';
import starlightHeadingBadges from 'starlight-heading-badges';
import starlightLlmsTxt from 'starlight-llms-txt';

export default await defineDocsConfig({
  llmsTxt: {},
  changelog: { repo: 'elcoosp/quoin' },
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
        { icon: 'github', label: 'GitHub', translations: { fr: "GitHub" }, href: 'https://github.com/elcoosp/quoin' },
      ],
      editLink: {
        baseUrl: 'https://github.com/elcoosp/quoin/edit/main/'
      },

            head: [
        {
          tag: 'link',
          attrs: { rel: 'icon', type: 'image/png', href: '/favicon.png' }
        },
      ],
      sidebar: [
        {
          label: 'Start Here', translations: { fr: 'Commencer ici' },
          items: [
            { label: 'Overview', translations: { fr: 'Aperçu' }, link: '/' },
          ]
        },
        {
          label: 'Tutorials', translations: { fr: 'Tutoriels' },
          items: [
            { label: 'Build a Cross-Framework Counter', translations: { fr: "Construire un compteur multi-framework" }, slug: 'tutorials/build-a-counter' },
          ]
        },
        {
          label: 'Guides', translations: { fr: 'Guides' },
          items: [
            { label: 'Installation', translations: { fr: "Installation" }, slug: 'guides/installation' },
            { label: 'Writing Portable Hooks', translations: { fr: "Écrire des hooks portables" }, slug: 'guides/portable-hooks' },
            { label: 'Loading Async Data', translations: { fr: "Chargement de données asynchrones" }, slug: 'guides/async-data' },
            { label: 'Cancelling Work', translations: { fr: "Annulation du travail" }, slug: 'guides/cancelling-work' },
            { label: 'Sharing Global State', translations: { fr: "Partager l'état global" }, slug: 'guides/global-state' },
            { label: 'Declarative UI Macros', translations: { fr: "Macros d'interface utilisateur déclaratives" }, slug: 'guides/ui-macros' },
            { label: 'Composing UCP Components', translations: { fr: "Composer des composants UCP" }, slug: 'guides/ucp-components' },
            { label: 'Styling with Tailwind Classes', translations: { fr: "Style avec les classes Tailwind" }, slug: 'guides/styling' },
            { label: 'Testing with quoin-testkit', translations: { fr: "Tester avec quoin-testkit" }, slug: 'guides/testing' },
            { label: 'Tracing with quoin-devtools', translations: { fr: "Tracer avec quoin-devtools" }, slug: 'guides/devtools' },
          ]
        },
        {
          label: 'Reference', translations: { fr: 'Référence' },
          items: [
            { label: 'Signals', translations: { fr: "Signaux" }, slug: 'reference/signals' },
            { label: 'Reactive Context', translations: { fr: "Contexte réactif" }, slug: 'reference/reactive-context' },
            { label: 'Executors & Tasks', translations: { fr: "Exécuteurs et tâches" }, slug: 'reference/executors' },
            { label: 'Derived Signals & Memos', translations: { fr: "Signaux dérivés et mémos" }, slug: 'reference/derived' },
            { label: 'Resources', translations: { fr: "Ressources" }, slug: 'reference/resources' },
            { label: 'Batched Updates', translations: { fr: "Mises à jour par lots" }, slug: 'reference/batching' },
            { label: 'Cancellation Tokens', translations: { fr: "Jetons d'annulation" }, slug: 'reference/cancellation' },
            { label: 'Typed Events', translations: { fr: "Événements typés" }, slug: 'reference/events' },
            { label: 'Macros', translations: { fr: "Macros" }, slug: 'reference/macros' },
            { label: 'Universal Component Protocol', translations: { fr: "Protocole de composant universel" }, slug: 'reference/ucp' },
            { label: 'Feature Flags', slug: 'reference/feature-flags' },
            { label: 'Crates', translations: { fr: "Crates" }, slug: 'reference/crates' },
            { label: 'quoin-testkit', translations: { fr: "quoin-testkit" }, slug: 'reference/testkit' },
            { label: 'quoin-devtools', translations: { fr: "quoin-devtools" }, slug: 'reference/devtools' },
          ]
        },
        {
          label: 'Explanation', translations: { fr: 'Explication' },
          items: [
            { label: 'Why Quoin Exists', translations: { fr: "Pourquoi Quoin existe" }, slug: 'explanation/why-quoin' },
            { label: 'Architecture', translations: { fr: "Architecture" }, slug: 'explanation/architecture' },
            { label: 'The Reactivity Model', translations: { fr: "Le modèle de réactivité" }, slug: 'explanation/reactivity-model' },
            { label: 'Backend Parity', translations: { fr: "Parité des backends" }, slug: 'explanation/backend-parity' },
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
