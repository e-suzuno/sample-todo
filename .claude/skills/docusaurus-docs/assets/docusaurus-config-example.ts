/**
 * 動作実績のある docusaurus.config.ts テンプレート
 *
 * 新規プロジェクトで Docusaurus をセットアップする際や、
 * OpenAPI (Swagger) ドキュメント生成が失敗した場合にこのファイルを参考にする。
 *
 * プロジェクト固有の値（{{...}} で示す箇所）は実際の値に置き換えること。
 *
 * === 動作確認済みの組み合わせ（バージョンは docusaurus-package-example.json に定義）===
 * - Node.js >= 20.0
 * - Docusaurus 3.9.2（`future.v4` 有効）
 * - docusaurus-plugin-openapi-docs / docusaurus-theme-openapi-docs は同一マイナーバージョンで揃える
 * - React 19 系
 *
 * 互換性問題が発生した場合は assets/docusaurus-package-example.json のピン版に戻すこと。
 * エージェントが独自判断でメジャーバージョンを上げない（特に Docusaurus 4 系への自動アップグレードは禁止）。
 */
import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import type * as OpenApiPlugin from 'docusaurus-plugin-openapi-docs';

const config: Config = {
  title: '{{project.title}}',
  tagline: '{{project.tagline}}',
  favicon: 'img/favicon.ico',

  future: {
    v4: true,
  },

  url: '{{project.url}}',
  baseUrl: '/',

  onBrokenLinks: 'throw',

  i18n: {
    defaultLocale: 'ja',
    locales: ['ja'],
  },

  // Mermaid 図のサポート
  markdown: {
    mermaid: true,
  },

  themes: [
    '@docusaurus/theme-mermaid',
    // OpenAPI (Swagger) ドキュメント表示用テーマ
    'docusaurus-theme-openapi-docs',
    // ローカル検索
    [
      '@easyops-cn/docusaurus-search-local',
      {
        hashed: true,
        language: ['ja'],
        indexBlog: false,
        docsRouteBasePath: '/',
      },
    ],
  ],

  plugins: [
    // drawio ファイルの埋め込みサポート
    'docusaurus-plugin-drawio',
    // Node.js polyfill（OpenAPI プラグインで必要）
    function polyfillPlugin() {
      return {
        name: 'node-polyfill',
        configureWebpack() {
          return {
            resolve: {
              fallback: {
                path: false,
                fs: false,
              },
            },
          };
        },
      };
    },
    // OpenAPI (Swagger) ドキュメント生成プラグイン
    [
      'docusaurus-plugin-openapi-docs',
      {
        id: 'api',
        docsPluginId: 'classic',
        config: {
          {{project.api_id}}: {
            specPath: 'openapi.yaml',
            outputDir: 'docs/api',
            sidebarOptions: {
              groupPathsBy: 'tag',
            },
          } satisfies OpenApiPlugin.Options,
        },
      },
    ],
  ],

  presets: [
    [
      'classic',
      {
        docs: {
          path: '../docs',
          sidebarPath: require.resolve('./sidebars.js'),
          routeBasePath: '/',
          // OpenAPI ドキュメントの表示に必要
          docItemComponent: '@theme/ApiItem',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: '{{project.title}}',
      items: [
        // サイドバーごとにナビゲーション項目を追加
        // sidebarId は sidebars.ts で定義したキーと一致させる
        {
          type: 'docSidebar',
          sidebarId: '{{sidebar_id}}',
          position: 'left',
          label: '{{sidebar_label}}',
        },
      ],
    },
    footer: {
      style: 'dark',
      copyright: `{{project.title}} ドキュメント`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
    mermaid: {
      theme: {light: 'default', dark: 'dark'},
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
