---
name: rust-desktop-patterns
description: |
  Rust でデスクトップアプリ（高性能ネイティブ GUI・コードエディタなど大量テキスト描画を扱う UI）を
  実装するときのアーキテクチャ選定・GPUI の使い方・テキストエディタ実装・非同期ランタイム・
  リアルタイムコラボ・拡張機構の共通実装パターンを参照したいときに使用するスキル。
  「Rust でデスクトップアプリを作りたい」「GPUI の使い方」「Rust でネイティブなコードエディタを作りたい」
  「Tauri と GPUI のどちらを選ぶか」などの依頼で起動する。
  エージェントがいない環境でもメインセッションから直接呼び出して完遂できる独立型スキル。任意のエージェントからも参照できる。
compatibility: Rust 1.85+ / GPUI / Tauri 2+ / Iced 0.12+ / egui 0.27+
---

# Rust デスクトップアプリ実装パターン集

## 役割

依頼内容からパターン種別を判定し、対応するリファレンスファイルを読んで提示・適用する。
ネイティブ・高速・コラボラティブなコードエディタを Rust で構築するシナリオを主軸に据える。

## ライセンス上の注意（必読）

- 本スキルおよび references/ 配下は **GPL ライセンスの OSS コードを本リポジトリに転載しない**。
  アーキテクチャ・設計思想・API シグネチャ・利用パターンを「自分の言葉で解説する」方針で書く。
- GPUI のような Apache-2.0 / MIT のクレートの公開 API を用いた最小例は問題ないが、
  長文転載は避け、必要なら NOTICE 表示を行う。
- 外部 OSS のソースを参照する場合は **URL リンクとして引用** に留め、ファイル内に貼り込まない。
- クリーンルーム性を守るため、商用 GPL プロダクトのソースは「設計を真似する目的で読む」ことを避け、
  Apache-2.0 / MIT の OSS ドキュメントや論文を一次情報とする。

## 共通ルール

- Rust エディションは **2024**、MSRV は **1.85+** を前提とする
  （GPUI の上流ワークスペースが [`edition = "2024"`](https://github.com/zed-industries/zed/blob/f78f6da255afe353fa2b726addca578dbcfd78c8/Cargo.toml) を採用しており、
  Rust 2024 edition は Rust 1.85（2025-02 安定化）以降でしかコンパイルできないため。
  GPUI 側に明示的な `rust-version` フィールドは無いので、上流の edition を踏襲する）
- 公開 API は **`#![deny(missing_docs)]`** で docstring を強制することを推奨
- 依存は `Cargo.toml` の `[workspace.dependencies]` で一元管理する（モノレポ前提）
- `unsafe` を使う箇所は SAFETY コメントで invariant を明示する
- ロギングは `tracing` + `tracing-subscriber`、エラーは `thiserror`（ライブラリ）/ `anyhow`（バイナリ）で統一する
- UI スレッドをブロックしない（重い処理は Background Executor / spawn_blocking に逃がす）

## ルーティング表

依頼内容からキーワードを判定し、対応するリファレンスファイルを Read して適用する。

| 依頼のキーワード・文脈 | 種別 | リファレンスファイル |
|---|---|---|
| アーキテクチャ選定、GPUI vs Tauri、Iced、egui、Dioxus、Slint、技術選定、トレードオフ | アーキテクチャ選定 | `references/architecture-selection.md` |
| GPUI、Element、View、Model、Context、Render、Window、リアクティビティ | GPUI 基礎 | `references/gpui-fundamentals.md` |
| テキストエディタ、Rope、バッファ、マルチカーソル、Vim mode、シンタックスハイライト、Tree-sitter | テキストエディタ実装 | `references/text-editing.md` |
| async、非同期、Executor、tokio、async-std、Background Task、UI スレッド、smol | 非同期 / タスクランタイム | `references/async-runtime.md` |
| コラボレーション、リアルタイム共同編集、CRDT、Yjs、Automerge、Operational Transform、Presence | リアルタイムコラボ | `references/realtime-collaboration.md` |
| 拡張、プラグイン、WebAssembly、Wasm、LSP、Language Server、parser.wasm、tree-sitter プラグイン配布、テーマ | 拡張機構 | `references/extensibility.md` |

## 判定のフロー

1. ユーザーの依頼文を分析してキーワードを抽出する
2. ルーティング表で種別を決定する（複数該当する場合はすべて Read する）
3. 対応するリファレンスファイルを Read する
4. プロジェクトの既存コードを確認し、既存パターンを踏襲しつつリファレンスを適用する

## 種別が曖昧な場合

「Rust でネイティブなコードエディタを作りたい」のような包括的な依頼では、
まず `architecture-selection.md` を Read して土台を固め、続いて `gpui-fundamentals.md`
→ `text-editing.md` → `async-runtime.md` の順で読むと全体像が把握しやすい。

## 参考リンク（外部）

- GPUI ドキュメント: https://www.gpui.rs/
- Rust 公式ブック: https://doc.rust-lang.org/book/
- The Rustonomicon（`unsafe` 解説）: https://doc.rust-lang.org/nomicon/
