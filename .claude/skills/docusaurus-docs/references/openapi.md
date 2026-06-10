# OpenAPI 定義 フォーマット定義

対象: ドキュメントルート配下の OpenAPI 仕様ファイル（通常 `detailed-design/api/openapi.yaml`）

## 重要性

OpenAPI 定義は **API 仕様の唯一の正** とする。
画面定義（`screen-doc.md`）やシーケンス図（`sequence.md`）は、エンドポイント名・パス・フィールド詳細をこのファイルに委譲する。

レガシープロジェクトでも **雛形レベルでまず作成する** ことを強く推奨する。
完全でなくてよい。「エンドポイント一覧として OpenAPI を見れば良い」状態を最初に作ることに価値がある。

## ファイル形式

- バージョン: **OpenAPI 3.1**（最新機能を使う必要がなければ 3.0.x でも可）
- 形式: YAML を第一選択（人間が読み書きしやすい）。既存プロジェクトが JSON を使っている場合はそれを踏襲する
- 配置: `detailed-design/api/openapi.yaml`（bootstrap で合意したパスに従う）

## 最小構成テンプレート

```yaml
openapi: 3.1.0
info:
  title: <プロジェクト名> API
  version: 0.1.0
  description: |
    <プロジェクト名> の HTTP API 仕様。
    エンドポイント追加・変更時は必ずこのファイルを先に更新する。
servers:
  - url: https://api.example.com
    description: 本番環境
  - url: http://localhost:3000
    description: ローカル開発環境
tags:
  - name: auth
    description: 認証関連
  - name: users
    description: ユーザー管理
paths:
  /auth/login:
    post:
      tags: [auth]
      summary: ログイン
      operationId: login
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/LoginRequest'
      responses:
        '200':
          description: 成功
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/LoginResponse'
        '401':
          $ref: '#/components/responses/Unauthorized'
components:
  schemas:
    LoginRequest:
      type: object
      required: [email, password]
      properties:
        email:
          type: string
          format: email
        password:
          type: string
          format: password
    LoginResponse:
      type: object
      required: [token]
      properties:
        token:
          type: string
    ErrorResponse:
      type: object
      required: [code, message]
      properties:
        code:
          type: string
        message:
          type: string
  responses:
    Unauthorized:
      description: 認証が必要
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/ErrorResponse'
```

## 作成手順（レガシープロジェクト向け）

既存コードから事実を抽出して雛形化する。**想像で書かない**。

1. **ルート定義を Grep** — フレームワーク別の抽出パターンで検索してエンドポイント一覧を作る（下記参照）
2. **タグ分類** — URL プレフィックス（`/auth/*` `/users/*` など）で `tags` にグループ分けする
3. **パス・メソッドのみ先行で定義** — リクエスト/レスポンスのスキーマは後回しでよい。まずは `paths` 配下にすべてのエンドポイントを列挙する
4. **共通レスポンスを `components/responses` に切り出す** — `401 Unauthorized` `403 Forbidden` `404 NotFound` `422 UnprocessableEntity` など
5. **ドメインスキーマを `components/schemas` に切り出す** — ユビキタス言語定義のドメイン用語と対応させる

### フレームワーク別のルート抽出パターン

プロジェクトで使用されているフレームワークに応じて以下の Grep パターンを使う。

| フレームワーク / 言語 | Grep パターン | 備考 |
|---|---|---|
| **Express / Hono / Fastify** (Node.js) | `(app\|router)\.(get\|post\|put\|delete\|patch)\(` | コールバック形式のルート定義 |
| **NestJS** (Node.js) | `@(Get\|Post\|Put\|Delete\|Patch\|All)\(` | デコレータ形式。`@Controller('<prefix>')` と組み合わせてパスを解決 |
| **Laravel** (PHP) | `Route::(get\|post\|put\|patch\|delete\|any\|match)\(` | `routes/api.php` / `routes/web.php` を中心に探す。`Route::prefix` / `Route::group` のネストに注意 |
| **Laravel (attribute-based)** | `#\[Route\(` | Laravel 11+ のアトリビュート形式 |
| **Spring Boot** (Java / Kotlin) | `@(GetMapping\|PostMapping\|PutMapping\|DeleteMapping\|PatchMapping\|RequestMapping)\(` | `@RestController` / `@RequestMapping` でクラスレベルのプレフィックスも確認 |
| **Ruby on Rails** | `^\s*(get\|post\|put\|patch\|delete)\s+['"]` | `config/routes.rb`。`resources :xxx` も検索対象（展開後のパスを導出） |
| **Django (DRF)** | `path\(\|re_path\(\|router\.register\(` | `urls.py`。ViewSet の場合は `router.register` から CRUD パスを展開 |
| **FastAPI** (Python) | `@(app\|router)\.(get\|post\|put\|delete\|patch)\(` | Flask も類似 |
| **ASP.NET Core** (C#) | `\[Http(Get\|Post\|Put\|Delete\|Patch)\]` | `[Route("...")]` でクラスレベルのプレフィックスも確認 |
| **Go (net/http, chi, gin, echo)** | `\.(Handle\|HandleFunc\|GET\|POST\|PUT\|DELETE\|PATCH)\(` | ルーターライブラリにより記法が異なるため複数パターンを試す |

### 抽出時の注意

- **プレフィックスの合成を忘れない** — `@Controller('users')` + `@Get(':id')` のように、クラスレベルとメソッドレベルが別々に定義されている場合は合成してフルパスにする
- **動的パスパラメータを OpenAPI 形式に変換** — `:id` `{id}` `<id>` `(?P<id>[^/]+)` など実装の書き方を `{id}` に統一する
- **RESTful リソース展開の抜けに注意** — Rails の `resources :posts` や DRF の `router.register` は自動で7つのエンドポイントを生成するため、**展開後のパスをすべて列挙する**
- **認証ミドルウェアの存在を確認** — `/api/v1/*` が認証必須であることなどは `security` セクションに反映する

## 命名の原則

- `operationId` は camelCase（例: `createUser`, `listPosts`）
- スキーマ名は PascalCase（例: `UserCreateRequest`, `PostListResponse`）
- フィールド名は **ユビキタス言語定義のフィールド命名規則に従う**
- パスは kebab-case または小文字単語（例: `/users/{userId}/reset-password`）

## 書いてはいけないもの

- 実装の詳細（DB 構造、内部サービス呼び出し、ミドルウェア順序）
- フロントエンド固有のバリデーションルール（API 仕様として必要なもののみ書く）
- 実装言語固有の型名（TypeScript の `number` を OpenAPI の `integer`/`number` と混同しない）

## 整合性チェックの方針

OpenAPI は `docusaurus-reviewer` エージェントが **「API 仕様の正」** として扱う。
レビュー時には以下を検査する:

- OpenAPI のパス vs 実装のルート定義
- OpenAPI のスキーマフィールド名 vs ユビキタス言語定義のフィールド名
- OpenAPI のスキーマフィールド名 vs 実装の DTO/モデルクラスのプロパティ名

OpenAPI を書いていないと、このチェックが機能しない。
**「OpenAPI が無い = API 仕様のレビューが不可能」** と認識すること。

## 注意事項

- OpenAPI を一度書いたら **実装変更時に必ずこのファイルも更新する**。陳腐化すると「書いてあるが実装と違う」状態になり、「書いていない」より有害
- 雛形作成時点で完全性を目指さなくてよい。**増分で育てていく**
- 巨大化したら `$ref` で別ファイル分割を検討する（`paths/auth.yaml` など）。ただし Docusaurus の表示互換性も考慮する

## Docusaurus プラグイン統合（docusaurus-plugin-openapi-docs）

OpenAPI 仕様を Docusaurus のサイドバーに UI 描画付きで表示する場合は `docusaurus-plugin-openapi-docs` + `docusaurus-theme-openapi-docs` を使う。

### パッケージ

`assets/docusaurus-package-example.json` を使って bootstrap した場合、以下は**すでに含まれている**ため追加インストール不要。

- `docusaurus-plugin-openapi-docs`
- `docusaurus-theme-openapi-docs`

既存プロジェクトに後から追加する場合のみ `npm install docusaurus-plugin-openapi-docs docusaurus-theme-openapi-docs` を実行する。プラグインとテーマは**同一マイナーバージョン**で揃えること。

### docusaurus.config の設定

```js
{
  themes: ['docusaurus-theme-openapi-docs', '@docusaurus/theme-mermaid'],
  plugins: [
    [
      'docusaurus-plugin-openapi-docs',
      {
        id: 'api',
        docsPluginId: 'classic',
        config: {
          myApi: {
            specPath: '../docs/detailed-design/api/openapi.yaml',
            outputDir: '../docs/detailed-design/api/generated',
            sidebarOptions: { groupPathsBy: 'tag', categoryLinkSource: 'tag' },
          },
        },
      },
    ],
  ],
  presets: [['classic', { docs: { docItemComponent: '@theme/ApiItem' } }]],
}
```

### サイドバー設定の **重要な注意**

プラグインは `outputDir` に `sidebar.ts` を生成する。この生成物を**必ず `require()` で sidebars に組み込む**こと。
`autogenerated` に任せると、プラグイン生成 `sidebar.ts` は無視され、全エンドポイント MDX とタグカテゴリページ（`*.tag.mdx`）が**平坦に並ぶ**だけになる（タグ別ネスト構造が効かない）。

```js
// sidebars.js
mainSidebar: [
  // ... 他のカテゴリ
  {
    type: 'category',
    label: 'API 仕様',
    link: { type: 'doc', id: 'detailed-design/api/index' },
    items: require('../docs/detailed-design/api/generated/sidebar.ts').default,
  },
]
```

### openapi.yaml の YAML 構文上の注意

プラグインの YAML パーサ（`@redocly/openapi-core`）は厳格な YAML 1.1 仕様で処理する。
クォートなしのスカラー値にダブルクォートを含めると `unexpected end of stream` エラーになる。

```yaml
# NG — クォートなしスカラー値にダブルクォートが含まれる
description: エラー種別（例: "Bad Request", ...）

# OK — シングルクォートで囲む
description: 'エラー種別（例: "Bad Request", ...）'
```

### 生成物の扱い

`outputDir` に生成される MDX は **`.gitignore` 推奨**。`openapi.yaml` から `npm run api-docs` で再生成可能なためコミットする必要はない。

```gitignore
docs/**/api/generated/
docs-site/.docusaurus/
```
