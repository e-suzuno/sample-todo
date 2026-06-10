# データベース設計（ER 図・テーブル定義）フォーマット定義

対象: ドキュメントルート配下の `database.md` / `er.md` / `schema.md` 等
- ブートストラップで合意済みの場合はそのパス（既定は `detailed-design/database.md`）
- ディレクトリ構成が未確立の場合は、先に `references/bootstrap.md` を実行すること
- テーブル名・カラム名はユビキタス言語定義に準拠する（未整備の場合は `references/ubiquitous-language.md` を先に実行する）

## セクション構成

```markdown
---
sidebar_position: <番号>
title: "データベース設計"
---

# データベース設計

## 概要

[対象データベース、ストレージ方式（RDB / DynamoDB / Firestore 等）、全体方針を 1〜3 文]

## ER 図

\`\`\`mermaid
erDiagram
  USER ||--o{ ORDER : places
  USER {
    uuid id PK
    string email
    timestamp createdAt
  }
  ORDER ||--|{ ORDER_ITEM : contains
  ORDER {
    uuid id PK
    uuid userId FK
    string status
    timestamp placedAt
  }
  ORDER_ITEM {
    uuid id PK
    uuid orderId FK
    uuid productId FK
    int quantity
  }
\`\`\`

## テーブル定義

### `users`

ユーザーアカウント情報を保持する。認証・プロフィール・権限の基点となる。

| カラム名 | 型 | 制約 | 説明 |
|---|---|---|---|
| `id` | UUID | PK | ユーザー識別子 |
| `email` | VARCHAR(255) | UNIQUE, NOT NULL | ログインに使用するメールアドレス |
| `status` | VARCHAR(32) | NOT NULL | `active` / `suspended` / `deleted` |
| `created_at` | TIMESTAMP | NOT NULL | 作成日時（UTC） |

### `orders`

...

## インデックス方針

- 検索頻度の高いカラムにインデックスを張る方針を箇条書きで書く
- 具体的な `CREATE INDEX` 文はマイグレーションコードに委ねる

## 参照整合性

- 外部キー制約を張るテーブル・カラムと `ON DELETE` / `ON UPDATE` の方針を記載する
- カスケード削除を避ける場合、その理由（論理削除との兼ね合い等）を書く
```

## 書き方の指針

- **方針・構造を書き、DDL 文そのものは書かない** — `CREATE TABLE` 文はマイグレーションツール（Prisma / TypeORM / Flyway / Alembic 等）のスキーマ定義を正として管理し、ドキュメントはその意図を説明する
- **テーブル名・カラム名はユビキタス言語定義に揃える** — ここで新しい命名を決めない
- **物理名と論理名の対応**を明示する（例: ユビキタス言語の「ユーザー」→ テーブル `users`、カラム `user_id`）
- **ER 図は主要なエンティティ間の関係に絞る**（詳細テーブルや監査ログ等は別図に分ける）
- **データ型は抽象度を保つ** — `VARCHAR(255)` レベルで十分。精度・照合順序はマイグレーション側で管理

## Mermaid ER 記法のポイント

| 記法 | 意味 |
|---|---|
| `\|\|--o\{` | 1 対多（多側は 0 以上） |
| `\|\|--\|\{` | 1 対多（多側は 1 以上） |
| `\|\|--\|\|` | 1 対 1 |
| `\}o--o\{` | 多対多 |
| カラム行の `PK` / `FK` / `UK` | 主キー / 外部キー / 一意キー |

## DynamoDB / NoSQL の場合

RDB テーブルの代わりに以下を記載する:

- **テーブル名** と **パーティションキー / ソートキー**
- **GSI（Global Secondary Index）** の一覧とアクセスパターン
- **アクセスパターン一覧**（「このクエリで何を取得するか」をテーブルにする）

```markdown
### アクセスパターン

| パターン名 | クエリ | 使用インデックス |
|---|---|---|
| ユーザー詳細取得 | PK=`USER#<userId>`, SK=`PROFILE` | ベーステーブル |
| ユーザーの注文一覧 | PK=`USER#<userId>`, SK begins_with `ORDER#` | ベーステーブル |
| 日次売上集計 | GSI1PK=`DATE#<YYYY-MM-DD>` | GSI1 |
```

## 書いてはいけないもの

- DDL 文（`CREATE TABLE ...`）そのもの — マイグレーションコードで管理
- アプリケーション側の ORM エンティティクラス — コードで管理
- マイグレーション履歴の列挙 — マイグレーションツールの履歴機能で管理
- カラムの VARCHAR 長・数値精度等の実装詳細（方針レベルで十分）

## 作成後の更新

- 新規テーブル追加時はテーブル定義節に追記し、ER 図にもエンティティを追加する
- カラムの論理名変更はユビキタス言語定義と必ず同時に更新する
- 廃止テーブルは削除せず「廃止」マークを付けて履歴を残すか、ADR で廃止判断を記録する
