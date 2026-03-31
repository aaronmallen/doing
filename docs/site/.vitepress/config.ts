import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'doing',
  description: 'A command line tool for remembering what you were doing and tracking what you\'ve done',

  head: [
    ['meta', { name: 'theme-color', content: '#F97316' }],
  ],

  themeConfig: {
    nav: [
      { text: 'Docs', link: '/getting-started/installation' },
    ],

    sidebar: [
      {
        text: 'Getting Started',
        items: [
          { text: 'Installation', link: '/getting-started/installation' },
          { text: 'Quick Start', link: '/getting-started/quick-start' },
          { text: 'Core Concepts', link: '/getting-started/concepts' },
        ],
      },
      {
        text: 'Adding Entries',
        collapsed: false,
        items: [
          { text: 'now', link: '/cli/now' },
          { text: 'done', link: '/cli/done' },
          { text: 'again', link: '/cli/again' },
          { text: 'meanwhile', link: '/cli/meanwhile' },
        ],
      },
      {
        text: 'Completing Entries',
        collapsed: false,
        items: [
          { text: 'finish', link: '/cli/finish' },
          { text: 'cancel', link: '/cli/cancel' },
        ],
      },
      {
        text: 'Viewing Entries',
        collapsed: false,
        items: [
          { text: 'recent', link: '/cli/recent' },
          { text: 'today', link: '/cli/today' },
          { text: 'yesterday', link: '/cli/yesterday' },
          { text: 'last', link: '/cli/last' },
          { text: 'show', link: '/cli/show' },
          { text: 'on', link: '/cli/on' },
          { text: 'since', link: '/cli/since' },
          { text: 'grep', link: '/cli/grep' },
        ],
      },
      {
        text: 'Interactive Selection',
        collapsed: false,
        items: [
          { text: 'choose', link: '/cli/choose' },
          { text: 'select', link: '/cli/select' },
        ],
      },
      {
        text: 'Editing Entries',
        collapsed: false,
        items: [
          { text: 'note', link: '/cli/note' },
          { text: 'tag', link: '/cli/tag' },
          { text: 'autotag', link: '/cli/autotag' },
          { text: 'mark', link: '/cli/mark' },
          { text: 'reset', link: '/cli/reset' },
        ],
      },
      {
        text: 'Organization',
        collapsed: false,
        items: [
          { text: 'sections', link: '/cli/sections' },
          { text: 'tags', link: '/cli/tags' },
          { text: 'archive', link: '/cli/archive' },
          { text: 'rotate', link: '/cli/rotate' },
        ],
      },
      {
        text: 'Templates & Views',
        collapsed: false,
        items: [
          { text: 'template', link: '/cli/template' },
          { text: 'view', link: '/cli/view' },
          { text: 'views', link: '/cli/views' },
          { text: 'colors', link: '/cli/colors' },
        ],
      },
      {
        text: 'Configuration',
        collapsed: false,
        items: [
          { text: 'config', link: '/cli/config' },
          { text: 'budget', link: '/cli/budget' },
          { text: 'tag-dir', link: '/cli/tag-dir' },
          { text: 'open', link: '/cli/open' },
          { text: 'Config Reference', link: '/configuration/' },
        ],
      },
      {
        text: 'Import & Export',
        collapsed: false,
        items: [
          { text: 'import', link: '/cli/import' },
          { text: 'plugins', link: '/cli/plugins' },
        ],
      },
      {
        text: 'System',
        collapsed: false,
        items: [
          { text: 'undo', link: '/cli/undo' },
          { text: 'redo', link: '/cli/redo' },
          { text: 'completion', link: '/cli/completion' },
          { text: 'self-update', link: '/cli/self-update' },
          { text: 'commands', link: '/cli/commands' },
          { text: 'changes', link: '/cli/changes' },
        ],
      },
      {
        text: 'Deviations',
        link: '/deviations',
      },
    ],

    search: {
      provider: 'local',
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/aaronmallen/doing' },
    ],

    editLink: {
      pattern: 'https://github.com/aaronmallen/doing/edit/main/docs/site/:path',
      text: 'Edit this page on GitHub',
    },

    footer: {
      message: 'Released under the MIT License.',
    },
  },
})
