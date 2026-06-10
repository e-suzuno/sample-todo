# 非同期 / タスクランタイム

## 概要

デスクトップアプリでは「UI スレッドを絶対にブロックしない」ことが最優先要件になる。
本ドキュメントでは Rust の非同期ランタイム選定、UI スレッドと Background Executor の分離、
spawn パターンの設計判断を整理する。

## ランタイム選定

| ランタイム | 特徴 | 向く用途 |
|---|---|---|
| **tokio** | 業界標準。エコシステム最大。マルチスレッド | サーバ寄り・ネットワーク重視・大量タスク |
| **smol / async-std** | 軽量・組み込みやすい | デスクトップ・組込み・cargo の起動時間を抑えたい |
| **GPUI 内蔵 Executor** | UI スレッドと Background スレッドを分離した独自ランタイム | GPUI ベースのアプリ（自動でこれが使われる） |
| **futures-executor** | 標準的な executor。テスト用途 | ライブラリの単体テスト |

GPUI を使う場合は基本的に GPUI の Executor に従う。
外部クレートが tokio 依存の場合は `tokio::runtime::Handle::current()` を Background Executor と橋渡しする。

## UI スレッドと Background スレッドの分離

```
[ UI スレッド ]                          [ Background Executor (N スレッド) ]
   - 入力イベント処理                       - ファイル I/O
   - レンダリング                           - LSP / RPC 通信
   - Model の更新（軽量）                   - Tree-sitter のフル再パース
                                            - Git 操作
                                            - ネットワーク
        ▲                                            │
        │                                            ▼
        └────── update(cx, |this, cx| { … }) ────────┘
                   Model に結果を反映して notify
```

**ルール**: UI スレッドでは **連続 1ms 以上かかる処理を絶対に走らせない**。

## GPUI の `spawn` パターン

### 単発のバックグラウンド処理

```rust
// 概念スケッチ
fn load_file(&mut self, path: PathBuf, cx: &mut Context<Self>) {
    cx.spawn(async move |this, mut cx| {
        let content = cx
            .background_spawn(async move {
                // background_spawn はブロッキングを許容する背景スレッドプール上で動くため、
                // 同期 std::fs をそのまま使ってよい（推奨）。
                // tokio::fs は GPUI が tokio runtime を持たないので panic。
                // smol::fs / async-fs も async-io の reactor が起動していない場合
                // 「no reactor running」で hang / panic するので、安易な置き換えはしない。
                std::fs::read_to_string(&path)
            })
            .await?;

        this.update(&mut cx, |this, cx| {
            this.buffer.set_content(content);
            cx.notify();
        })
    })
    .detach_and_log_err(cx);
}
```

- `cx.spawn` は UI スレッド側で動くタスクを生成する（戻り値で Model を触れる）
- `cx.background_spawn` の中身は Background スレッドで走る（ファイル読み込みなど）。
  ここはブロッキングを許容する設計なので、同期 API を `.await` 無しで呼んでよい
- 結果は `this.update` クロージャで Model に反映する

#### どうしても async I/O API を使いたい場合

`async-io` の reactor を別途起動しているクレートに同居している場合のみ `smol::fs` / `async-fs`
が動作する。reactor の存在に依存したくないなら、`blocking::unblock` で同期 I/O を明示的に
ブロッキングプールへオフロードする方法が安全：

```rust
// 概念スケッチ：blocking クレート経由で std::fs を非同期化する
cx.background_spawn(async move {
    blocking::unblock(move || std::fs::read_to_string(&path)).await
})
```

`smol::fs` を採用するなら、アプリ初期化時に `async-io` の reactor スレッドを明示的に
起動していることを確認する（`async_io::block_on` を呼ぶか、`smol::block_on` を立てる等）。

### キャンセル可能なタスク

長時間タスクは `Task<T>` を Model に保持し、新しい操作で **古いタスクを drop してキャンセル** する。

```rust
// 概念スケッチ
struct Editor {
    pending_search: Option<Task<()>>,
}

fn search(&mut self, query: String, cx: &mut Context<Self>) {
    self.pending_search = Some(cx.spawn(async move |this, mut cx| {
        let results = run_search(&query).await;
        this.update(&mut cx, |this, cx| {
            this.results = results;
            cx.notify();
        }).ok();
    }));
}
```

- `Task<T>` が drop されると内部 future も cancel される
- `.detach()` で「結果は捨てるが実行は続ける」、`.detach_and_log_err(cx)` でエラーログ付き

## デバウンス / スロットル

ユーザーのキー入力ごとに LSP / 検索を走らせると過負荷になる。デバウンスを挟む：

```rust
// 概念スケッチ
fn on_input_changed(&mut self, cx: &mut Context<Self>) {
    self.debounce_task = Some(cx.spawn(async move |this, mut cx| {
        cx.background_executor().timer(Duration::from_millis(150)).await;
        this.update(&mut cx, |this, cx| {
            this.trigger_search(cx);
        }).ok();
    }));
}
```

- 既存の `debounce_task` を上書き（drop）することで前のタイマーは自動キャンセル
- 検索は 150ms、LSP completion は 100ms、保存は 1s 程度が経験則

## チャネルパターン

UI と Background 間の継続的なやり取りには **mpsc / oneshot** チャネルを使う。

| 用途 | 推奨 |
|---|---|
| 単発の結果返却 | `oneshot::channel` |
| 連続イベント（LSP の diagnostics 等） | `mpsc::unbounded_channel` |
| 複数購読者へのブロードキャスト | `broadcast::channel` |
| GPUI の Model 間連携 | `cx.observe` を優先（チャネル不要） |

外部プロセス（LSP サーバ等）との通信は **専用タスク** を立て、内部では Model へ `notify` する。

## エラーハンドリング

### `Result` を最後まで運ぶ

`cx.spawn` 内で `?` を使う場合、戻り値型は `Result<T, anyhow::Error>` にして
`.detach_and_log_err(cx)` でログ集約する。

### `panic` を Background で起こさない

Background タスク内の panic は UI を巻き込まずにスレッドが死ぬ。
`std::panic::catch_unwind` か、`Result` で握りつぶす設計にする。

### キャンセルは正常系

`Task` の drop によるキャンセルは正常系なので、`tracing::warn!` で警告ログを出さない。
`anyhow::Error` の `downcast::<Cancelled>()` で識別する。

## デッドロック回避

GPUI の Model は内部的に `RefCell` 相当を使う。同時に複数 Model を `update` で
ロックしようとすると **borrow violation**（panic）が発生する。

対策：

1. **1 つのクロージャ内で 1 つの Model だけ `update`**
2. 複数 Model を触る必要があるときは、外側で必要な値を read してからクロージャを抜ける
3. async 境界（`.await`）をまたぐ場合は、`update` クロージャを毎回区切る

## 性能 Tips

- **タスクの粒度**: 1ms 未満で完了するタスクを大量に spawn するとオーバヘッドが目立つ。バッチ化する
- **Background スレッド数**: デフォルト（コア数）でほぼ問題ない。I/O バウンドなら倍にしてもよい
- **`block_on` は禁止**: UI スレッドで `futures::executor::block_on` を呼ぶと完全に固まる

## アンチパターン

- **UI スレッドで `std::fs::read_to_string`**: 大ファイルで死ぬ。`cx.background_spawn` に逃がす（背景プール内では同期 `std::fs` を使ってよい）
- **`Arc<Mutex<T>>` で Model を共有**: GPUI の `Entity` を使えば不要。並行性は `update` で保証される
- **`tokio::spawn` を直接呼ぶ**: GPUI 配下では `cx.background_spawn` を使い、ライフサイクルを統一する
- **タスクの結果を放置**: `.detach()` は結果を捨てるという宣言。エラーが起きてもログに出ない
