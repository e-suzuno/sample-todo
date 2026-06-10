# 拡張機構

## 概要

エディタを「ユーザーが拡張可能」にするための機構を整理する。
プラグイン実行（WebAssembly サンドボックス）、言語サポート（LSP / Tree-sitter）、
テーマ・キーマップのカスタマイズの 3 軸でパターンを示す。

## 拡張モデルの選択肢

| 方式 | 特徴 | 採用例 |
|---|---|---|
| **WebAssembly (Wasm) + WIT** | サンドボックス強・多言語サポート | 各種コードエディタ拡張、Figma plugins |
| **動的ライブラリ (dylib)** | 高速・型安全だが ABI 互換性に悩む | VS Code（一部）、Helix |
| **スクリプト言語埋め込み** | 動的・低い参入障壁。型安全性は犠牲 | Neovim (Lua)、Emacs (Elisp) |
| **外部プロセス + RPC** | クラッシュ耐性が高い。レイテンシが大きい | LSP サーバそのもの |

新規実装では **Wasm + Component Model (WIT)** を推奨。
セキュリティ・移植性・性能のバランスが最も良い。

## WebAssembly 拡張の設計

### コンポーネント構成

```
[ Host (Rust) ]                          [ Extension (Wasm) ]
   ├─ wasmtime / wasmer                     ├─ Rust / Go / TypeScript で書ける
   ├─ WIT インターフェース定義              │   （WIT に対応する言語）
   ├─ Capability 注入                       └─ Host が公開した API のみ使える
   │   - read_file (許可された範囲)
   │   - http_get (許可されたドメイン)
   │   - register_language_server
   └─ サンドボックス（ファイル・ネット制限）
```

### WIT インターフェース

WIT (WebAssembly Interface Types) で Host ↔ Extension の境界を宣言する。

```wit
// 概念スケッチ。実装ごとに差異あり
package my-editor:plugin;

interface plugin {
  init: func() -> result<_, string>;
  on-open-buffer: func(path: string);
  provide-completions: func(buffer: string, offset: u32) -> list<completion>;
  record completion {
    label: string,
    detail: option<string>,
    insert-text: string,
  }
}
```

- インターフェースを安定化したら **後方互換を守る**（フィールド追加 OK、削除/変更は破壊的）
- バージョン管理は `package <ns>:<name>@<semver>` で明示

### 実行ランタイム

| クレート | 特徴 |
|---|---|
| **wasmtime** | Bytecode Alliance の標準実装。WASI / Component Model フル対応 |
| **wasmer** | プラグインエコシステム重視。SDK が豊富 |
| **wasmi** | 軽量インタプリタ。組込み向け |

デスクトップアプリで本格運用するなら **wasmtime**。

### Capability ベースのセキュリティ

拡張に「何ができるか」を明示的に許可する設計：

- **ファイルアクセス**: ルートディレクトリを `Dir::open` で限定して preopen する
- **ネットワーク**: 許可ドメインのリストを Host 側で持つ
- **CPU / メモリ**: `wasmtime::Config` で fuel と max_memory を制限
- **時間**: 拡張から `wall_clock` を取れるが Host が決定論的に渡す

「明示的に許可されない限り何もできない」原則を貫く。

## LSP 統合

### tower-lsp / lsp-types を使う

Rust で LSP クライアントを書くなら `lsp-types`（型定義）と `tower-lsp`（サーバ側もある）を活用する。

クライアント実装の骨子：

```
[ エディタ ]
   ├─ LspManager
   │   ├─ child process per language (rust-analyzer, gopls, …)
   │   ├─ stdin: JSON-RPC requests
   │   ├─ stdout: JSON-RPC responses / notifications
   │   └─ stderr: ログ収集
   └─ Buffer
       └─ didOpen / didChange / didClose を発火
```

### 通信パターン

```
[ ユーザー入力 ]
        │
        ▼
[ Buffer 更新 ] ─→ [ debounce 100ms ] ─→ [ didChange 通知 ]
        │
        ▼
[ 補完要求トリガー ] ─→ [ textDocument/completion ]
                                   │
                                   ▼
                          [ LSP サーバから応答 ]
                                   │
                                   ▼
                          [ 補完 UI に反映 ]
```

- **didChange** は incremental（差分のみ）で送る。フル送信は大ファイルで死ぬ
- 同一リクエストは古いものを **キャンセル**（`$/cancelRequest`）
- LSP プロセスがクラッシュしたら自動再起動・状態を再同期

### LSP サーバの配布

- 言語ごとに `rust-analyzer` `gopls` `pyright` 等のバイナリが必要
- 自動ダウンロードする場合は Capability 拡張から `download-binary` API を提供
- 各 OS の path / 実行権限の違いに注意

## Tree-sitter 統合

詳細は `text-editing.md` を参照。拡張機構との接続点は以下：

- 拡張は `parser.wasm` と `highlights.scm` を提供する
- Host は拡張から受け取ったパーサを Tree-sitter ランタイムにロード
- インクリメンタル解析の状態は Buffer ごとに Host が保持

## テーマとキーマップ

### テーマ

JSON / TOML で定義し、Tree-sitter のキャプチャ名 → 色のマップとして実装：

```toml
# 概念スケッチ
[colors]
"keyword.control"      = "#ff79c6"
"function"             = "#50fa7b"
"string"               = "#f1fa8c"
"comment"              = "#6272a4"

[ui]
background             = "#282a36"
foreground             = "#f8f8f2"
selection.background   = "#44475a"
```

- 配色はテーマ作者が増えていくので **スキーマを厳密に定義** し、JSON Schema で検証する
- ダーク / ライト / High Contrast の 3 系統は最低限カバー

### キーマップ

JSON ベースで「コンテキスト + キー + コマンド」のマッピングを宣言：

```json
[
  {
    "context": "Editor",
    "bindings": {
      "cmd-s": "editor::Save",
      "cmd-shift-p": "command_palette::Toggle"
    }
  }
]
```

- **コンテキスト** で発火範囲を絞る（Editor / Terminal / Modal 等）
- 衝突時は **より specific なコンテキスト** が優先
- Vim mode を入れる場合は `vim_mode == normal` のような条件付きキー定義を許す

## 拡張の配布

### マーケットプレイスの設計

- 中央レジストリ（独自 or GitHub repo）に manifest.json を集約
- 各拡張は独立リポジトリ + GitHub Release で `.wasm` を配布
- Manifest にバージョン・対応エディタバージョン・権限要求を記載
- インストール時にユーザーへ **権限ダイアログ** を出す

### 署名と検証

- 拡張の `.wasm` に対して Ed25519 等で署名
- Host はインストール時に公開鍵で検証
- 改ざんを検出したらインストール拒否

## アンチパターン

- **動的 .so / .dll を直接 dlopen**: ABI 互換性で破綻する。Wasm を選ぶ
- **拡張に Host 全権を渡す**: ファイル全領域・無制限ネット・unlimited memory はサンドボックスにならない
- **LSP の didChange を毎キーストロークで送信**: debounce 必須
- **テーマを Rust コードに hardcode**: ユーザーがいじれない。設定ファイル化する
- **キーマップを 1 つのフラットリストで管理**: コンテキストごとに分離する

## 関連リンク

- WebAssembly Component Model: https://component-model.bytecodealliance.org/
- LSP 仕様: https://microsoft.github.io/language-server-protocol/
- Tree-sitter ドキュメント: https://tree-sitter.github.io/tree-sitter/
- wasmtime: https://wasmtime.dev/
