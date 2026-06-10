# アーキテクチャ選定

## 概要

Rust でデスクトップアプリを作る際の GUI フレームワーク選択肢と、それぞれが向くユースケースを整理する。
コードエディタや高密度テキスト UI を構築する場合に避けて通れない GPU レンダリングとリアクティビティの方針を比較する。

## 主要フレームワーク比較

| フレームワーク | ライセンス | レンダリング方式 | リアクティビティ | 向くユースケース |
|---|---|---|---|---|
| **GPUI** | Apache-2.0 | GPU（macOS: Metal / Linux: Vulkan / Windows: DirectX 11、Blade 経由） | retained + observable model | 高速エディタ・大量テキスト描画 |
| **Tauri 2** | MIT/Apache-2.0 | OS WebView（HTML/CSS/JS） | フロントエンド任意（React, Svelte 等） | Web 技術資産を流用したい SaaS 系デスクトップ |
| **Iced** | MIT | wgpu（GPU） + Elm Architecture | 関数型 MVU（Model-View-Update） | 中規模ツール・関数型志向のチームに親和 |
| **egui** | MIT/Apache-2.0 | 即時モード（immediate） | ステートを毎フレーム再構築 | ゲーム内 UI・デバッグツール・小規模アプリ |
| **Dioxus** | MIT/Apache-2.0 | デスクトップは WebView または LiveView | React 風 hooks | クロスプラットフォーム前提でロジック共有したい |
| **Slint** | GPL-3.0 / 有償商用 / Royalty-Free（条件付き） | 独自 DSL + GPU | データバインディング | 組込み GUI・宣言的レイアウト重視 |

## 選定の判断基準

### 1. GPU レンダリングが必須か

数万行のコードを 60fps 以上で描画する用途では **GPU パスが前提**。
WebView（Tauri）はテキストの描画パフォーマンスで頭打ちになりやすい。

→ コードエディタ・グラフィック系: **GPUI / Iced / egui / Slint**
→ 一般的なフォーム系アプリ: **Tauri / Dioxus**

### 2. リアクティビティのモデル

- **observable model + retained view**: 状態を Model に保持し、変更通知で必要箇所のみ再描画。GPUI が採用。複雑な UI で性能と保守性のバランスが良い
- **immediate mode**: 毎フレーム UI ツリーを再構築。実装はシンプルだが大規模 UI では割高
- **MVU（Elm）**: pure な reducer。テスト容易性が高いが、長大なメッセージ enum が必要

### 3. プラットフォーム要件

- macOS / Windows / Linux 全部: GPUI / Tauri / Iced / egui
- モバイル含む: Dioxus / Slint
- 組込み: Slint / egui

### 4. ライセンス

- GPL を避けたい商用配布: **GPL ライセンスの上位プロジェクトを流用しない**。GPUI 単体は Apache-2.0 なので OK
- 完全オープン: Iced / egui / Tauri
- Slint を商用利用する場合は公式ライセンスページ（https://slint.dev/pricing.html）で適用条件を確認する

## コードエディタを構築する場合の推奨構成

```
[アプリケーション層]
  ├─ GPUI                        # ウィンドウ・描画・入力・リアクティビティ
  ├─ ropey / 自前 Rope           # テキストバッファ
  ├─ tree-sitter                 # 構文解析・ハイライト
  ├─ tower-lsp                   # LSP クライアント
  ├─ tokio もしくは smol         # 非同期ランタイム（UI と分離）
  └─ wasmtime / wasmer           # 拡張サンドボックス
```

GPUI は Apache-2.0 で公開されているため、GPL ライセンスの上位プロジェクトと混在するソースツリーから
直接 Git 依存で参照することは避け、Apache-2.0 単体配布のクレートを workspace に vendor するか、
公式に切り出された Apache-2.0 配布物を `Cargo.toml` に指定する形が安全。
いずれの場合も Apache-2.0 のライセンス表示と NOTICE を遵守する。

## 避けるべきアンチパターン

### WebView でテキストエディタを構築する

CodeMirror / Monaco を WebView で動かすと、ネイティブのスクロール慣性・IME・GPU 加速を犠牲にしやすい。
ネイティブ実装で高速とされるコードエディタの多くは WebView を採用していない。

### immediate mode で巨大な UI ツリーを毎フレーム再構築する

egui は便利だが、ファイル数千・行数十万のエディタには向かない。仮想スクロール前提の retained 設計に倒す。

### 同期 I/O を UI スレッドで実行する

ファイル読み込みや LSP 通信を UI スレッドで行うと、入力レイテンシが破綻する。
詳細は `async-runtime.md` を参照。

## 既存プロジェクトへ組み込む場合

既存の Tauri アプリにあとから GPUI を導入することは現実的ではない（描画モデルが衝突する）。
途中変更が必要なら、エディタペインだけを別プロセスに切り出して IPC で繋ぐ構成を検討する。
