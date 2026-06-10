# GPUI 基礎

## 概要

GPUI は Apache-2.0 で公開されている Rust 製の GPU 描画 UI フレームワーク。
GPU パイプライン（macOS は Metal、Linux は Vulkan、Windows は DirectX 11、いずれも Blade 経由）に直結し、
Element ベースの retained ビュー + Observable Model のリアクティビティを組み合わせる。

このドキュメントでは GPUI の **コアコンセプトと設計判断** を解説する。コードサンプルは
公開 API を用いた最小例に留め、GPL ライセンスの上位プロジェクトのコードは参照しない。

## コアコンセプト

### Model

アプリケーション状態を保持する型。`Entity<T>` で参照され、変更時に `cx.notify()` で
購読者に通知する。Reactive Programming における Observable に近い。

- 状態は **値で持ち、不変参照経由で読む**
- 変更は `update` クロージャを通してのみ可能（borrow checker と整合する）
- 1 つの Model に複数の View が購読できる（View ↔ Model の多対多）

### View

`Render` トレイトを実装した型。Model の状態を Element ツリーに変換する。

- `fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement`
- View 自身も Model の一種で、`cx.notify()` で再描画される
- Element ツリーは毎フレーム再構築されるが、内部レイアウトはキャッシュされる

### Element

UI の最小単位。`div()` `text()` `img()` などのビルダー関数で生成し、
fluent API でスタイルを積み上げる。Tailwind に強く影響を受けたスタイル DSL。

```rust
// あくまで API シグネチャ理解のためのスケッチ（架空の例）
fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .p_4()
        .bg(rgb(0x1e1e2e))
        .child(text("Hello, GPUI"))
}
```

### Context

`Context<T>` は View / Model のメソッド第二引数として渡される。
購読・スポーン・通知・参照取得などのフレームワーク操作はここから行う。

- `cx.notify()` — 自身を変更通知（再描画要求）
- `cx.observe(&model, callback)` — 他 Model を購読
- `cx.spawn(async move |this, cx| { ... })` — async タスクを起動
- `cx.background_executor()` — UI スレッドから外す重い処理用

### Window

ウィンドウ単位のコンテキスト。フォーカス・キーバインド・モーダル・サイズ等を扱う。
複数ウィンドウを開く場合はそれぞれに `Window` が紐付く。

## リアクティビティモデル

```
[ Model 変更 ] ── cx.notify() ──→ [ 購読 View ] ── render() ──→ [ Element ツリー差分 ]
                                                                     │
                                                                     ▼
                                                          [ GPU で再描画 ]
```

- View は **Model に "暗黙に" 依存しない**。`cx.observe` で明示的に購読する
- 変更通知は **次フレームでバッチ処理** されるため、1 操作で複数 `notify` しても再描画は 1 回
- React の VDOM と異なり、Element ツリーは「差分」ではなく「レイアウト計算入力」として扱う

## レイアウトとスタイル

GPUI のスタイル DSL は CSS Flexbox + Tailwind のサブセットに近い。

| カテゴリ | メソッド例 |
|---|---|
| レイアウト | `.flex()` `.flex_col()` `.flex_row()` `.gap_N()` `.grid()` |
| サイズ | `.w_full()` `.h_8()` `.size_4()` `.min_w_0()` |
| パディング・マージン | `.p_2()` `.px_4()` `.my_1()` |
| 色 | `.bg(rgb(0x…))` `.text_color(…)` `.border_color(…)` |
| インタラクション | `.hover(\|s\| s.bg(…))` `.cursor_pointer()` |
| 配置 | `.absolute()` `.relative()` `.top_0()` `.right_0()` |

数値はピクセル換算で `1 unit = 4px`（Tailwind 同様）。`rems()` `px()` などの単位構造体も存在する。

## イベントハンドリング

```rust
// API 利用例のスケッチ
div()
    .on_mouse_down(MouseButton::Left, cx.listener(|this, event, window, cx| {
        this.handle_click(event, window, cx);
    }))
```

- `cx.listener` で `&mut Self` を借りるクロージャを作る（一般的な Rust の borrow を回避するためのユーティリティ）
- `on_click` `on_key_down` `on_focus` などイベント別メソッドが用意される
- グローバルキーバインドは `KeyBindingContext` を通じて宣言的に登録できる

## 設計のベストプラクティス

### Model はドメイン単位で分ける

1 つの Model に全状態を詰めず、`EditorState` `SidebarState` `ProjectModel` のように
責務で分割する。購読粒度を細かく保つと再描画が減る。

### View は薄く、ロジックは Model に置く

View の `render` メソッドが 200 行を超えたら、サブビューに分割する。
ビジネスロジック（ファイル保存、構文解析の起動など）は Model のメソッドに寄せる。

### `cx.spawn` で async タスクを起動するとき、戻りで Model を更新する

```rust
// 概念スケッチ
fn save(&mut self, cx: &mut Context<Self>) {
    cx.spawn(async move |this, mut cx| {
        let result = save_to_disk().await;
        this.update(&mut cx, |this, cx| {
            this.last_saved = Some(Instant::now());
            cx.notify();
        }).ok();
    }).detach();
}
```

- `this.update(&mut cx, |this, cx| { ... })` のクロージャ内でのみ `&mut self` を持てる
- 通知し忘れると UI が更新されない（典型的なバグ）

### Element ツリーで巨大リストを直接描画しない

数千行のテキストや数千のファイルリストは **仮想スクロール**（uniform_list / list 等）を使う。
詳細は `text-editing.md` を参照。

## アンチパターン

- **`render` 内で `Vec` を毎回 alloc**: 静的に組めるなら `with_children` で済ませる
- **`cx.notify()` を呼び忘れる**: 状態を変えたら必ず通知する（または `update` 経由で変更する）
- **UI スレッドで I/O**: `cx.background_executor()` か `cx.spawn` に必ず逃がす
- **Model の循環参照**: `Entity` は Rc/Arc 相当の強参照。循環したらリークするので Weak で切る

## デバッグ Tips

- 描画パフォーマンスは GPUI のフレーム計測 API（`cx.refresh()` の頻度）で観測する
- レイアウト崩れは `.border_1().border_color(red())` で範囲を可視化するのが手早い
- `tracing` ログを仕込み、`RUST_LOG=gpui=trace` で内部イベントを追跡する
