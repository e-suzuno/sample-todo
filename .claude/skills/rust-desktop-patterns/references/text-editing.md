# テキストエディタ実装

## 概要

コードエディタを Rust で書く際に必要となる中核データ構造とアルゴリズムを整理する。
`String` ではなく **Rope** を採用し、構文解析に **Tree-sitter** をインクリメンタルに走らせ、
Vim emulation を **状態機械** として実装する理由を、設計判断レベルで解説する。

## Rope データ構造

### なぜ `String` ではダメか

`String` は連続したバイト列であり、中央への挿入が **O(N)** になる。
100MB のファイル中央に 1 文字挿入すると 50MB をコピーすることになり、入力レイテンシが破綻する。

### Rope の特徴

Rope は文字列を **B-tree** 状の構造で持ち、以下の性質を持つ：

- 挿入・削除: **O(log N)**
- ランダムアクセス: O(log N)
- 連結（concat）: O(log N)
- バイト位置 ↔ 行番号変換: O(log N)（行数を各ノードにキャッシュする場合）

### 実装の選択肢

| クレート | 特徴 |
|---|---|
| `ropey` | Rust で最も枯れた Rope 実装。Unicode 安全・行/バイト/グラフェム変換あり |
| 自前実装 | 行カウント・スパン情報・履歴を統合した独自 Rope（成熟したエディタが採用する形） |
| `xi-rope` | xi-editor 由来。歴史的経緯で参照されるが現在はメンテ停滞 |

新規実装ではまず `ropey` を採用し、専用機能が必要になったら自前化を検討する。

### Rope と CRDT の関係

リアルタイム共同編集を行う場合、Rope の各セグメントに **CRDT のメタデータ**（クライアント ID、Lamport タイムスタンプ）
を埋め込む形になる。詳細は `realtime-collaboration.md` を参照。

## バッファ層の設計

```
[ Buffer ]                  # 1 ファイル相当の編集対象
   ├─ rope: Rope            # 本文
   ├─ syntax: SyntaxTree    # Tree-sitter のインクリメンタル AST
   ├─ diagnostics: Vec<…>   # LSP 由来の診断情報
   └─ edit_history: Vec<…>  # Undo/Redo スタック（または CRDT 履歴）
```

- Buffer は **1 ファイル = 1 インスタンス**
- 同じファイルを複数ペインで開く場合は、同一 Buffer を複数 View が参照する
- 編集オペレーションは `apply_edit(range, replacement)` の単一メソッドに集約し、内部で全副作用を発火する

## マルチカーソル

### データモデル

`Vec<Selection>` で複数選択を保持する。各 Selection は `{ anchor, head }` の組で、
順方向・逆方向の選択を区別する。

### 重なり解消

```
[ ユーザー操作（例: word 拡張） ]
        │
        ▼
[ 全 Selection に適用 ]
        │
        ▼
[ 重なり検出 & マージ ]  # 同じ範囲を選択する Selection を 1 つに統合
        │
        ▼
[ ソートして確定 ]
```

- マージし忘れると同じ位置に同じ編集が複数回適用されるバグになる
- 挿入は **末尾から先頭の順** に適用し、selection の再計算も同じく back-to-front で行う（先行カーソルの挿入によって後方カーソルの offset が動かない順序にする）

### 編集の同時適用

複数カーソルに同じ挿入を行うときは、**末尾から先頭に向かって** 順に `rope.insert` する。
先頭から行うと位置が後続でずれてオフセット計算が破綻する。

## Vim emulation

### 状態機械として実装する

Vim モードは以下の主要状態を持つ：

- **Normal** — コマンド入力待ち
- **Insert** — テキスト入力中
- **Visual** / **Visual Line** / **Visual Block** — 選択中
- **Operator-pending** — `d` `c` `y` の後でモーション待ち
- **Command-line** — `:` 入力中

状態を `enum VimMode` で表現し、キー入力ごとに状態遷移を発火する FSM として書く。
状態数が多くなったら **入れ子 enum** で表現すると保守しやすい。

### コマンドの分解

Vim コマンドは `[count][operator][motion]` の構造を持つ。例:

- `3dw` → count=3, operator=delete, motion=word_forward
- `ci"` → count=1, operator=change, text-object=inside_double_quote

各要素を独立して受け付け、揃ったら `apply_operator(count, operator, range)` を実行する。

### 既存実装の活用

ゼロから Vim emulation を実装するのは大変なので、既存 OSS の方針を参考にする：

- `helix` エディタ（MPL-2.0）の kakoune 風モーションは Vim と思想が異なるが応用可能
- `nvim` のソースは設計の正解集だが Lua 経由のロジックも多い
- GPL ライセンスの実装はクリーンルーム性を守るため「読んで設計を真似る」ことを避け、
  仕様は Vim 公式ドキュメント・書籍を一次情報とする

## シンタックスハイライト

### Tree-sitter のインクリメンタル解析

Tree-sitter は編集差分を受け取り、AST の影響範囲だけを再パースする。
これにより数万行のファイルでも数 ms で再ハイライトできる。

```
[ Buffer に edit が適用された ]
        │
        ▼
[ tree-sitter に Edit を通知 ]   # start_byte / old_end / new_end
        │
        ▼
[ tree-sitter が影響範囲を再パース ]
        │
        ▼
[ 新しい AST と Query で highlights を取得 ]
        │
        ▼
[ View に再描画通知 ]
```

### Query ファイル

`highlights.scm` `injections.scm` `locals.scm` などの S 式クエリで、AST ノードに
意味的なタグ（`@function` `@keyword.control` 等）を付与する。
テーマはこのタグから色を引く。

### 重い処理は Background Executor へ

Tree-sitter の再パースは UI スレッドで実行してもおおむね問題ないが、
1MB を超える Markdown 等の重いケースでは Background Executor に逃がす。
詳細は `async-runtime.md` を参照。

## シンタックス対応の段階的実装

1. **プレーンテキスト編集** — Rope + カーソル + 入力のみ
2. **行番号と可視範囲レンダリング** — 仮想スクロールを導入
3. **Tree-sitter ハイライト** — 静的な色付け
4. **LSP 統合** — 補完・診断・定義ジャンプ
5. **マルチカーソル** — 編集の同時適用
6. **Vim emulation** — モード切替
7. **共同編集** — CRDT 化

各段階で計測可能な性能基準（例: 1MB ファイルで 60fps）を設定し、超えたら次に進む。

## アンチパターン

- **`String` で全文を持つ**: 大きなファイルで破綻する。最初から Rope にする
- **編集ごとに全画面再描画**: 可視範囲だけ差分描画する
- **Tree-sitter を毎回フル再パース**: インクリメンタル API（`edit` + `parse`）を必ず使う
- **マルチカーソルを後付けで足す**: Selection の表現を最初から `Vec<Selection>` にする
- **Undo を文字単位で記録**: ユーザー操作単位（タイマー or 区切り文字）でグルーピングする

## 性能計測

- 入力レイテンシ: キー押下から画面反映まで **16ms 以下** を目標
- 大ファイル開く: 100MB のファイルを **1 秒以内** にスクロール可能状態にする
- メモリ: 100MB ファイルで RSS が **3 倍以内**（Rope のオーバヘッド分）

`tracing` の `span!` でホットパスを計測し、`flamegraph` で可視化する。
