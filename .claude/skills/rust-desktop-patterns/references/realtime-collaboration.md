# リアルタイムコラボレーション

## 概要

複数ユーザーが同じファイルを同時に編集する機能を
Rust で実装する際のデータ構造・同期プロトコル・Presence 機構を整理する。

## CRDT vs OT

リアルタイム共同編集には大きく 2 つのアプローチがある。

| 観点 | OT (Operational Transform) | CRDT (Conflict-free Replicated Data Type) |
|---|---|---|
| 概念 | 操作を変換して順序を揃える | 操作の順序に依存せず可換になるデータ構造 |
| 中央サーバ | 必須（合流順序を決定） | 不要（P2P 可能）だがあると便利 |
| 実装難度 | 高い（変換関数の網羅性が難しい） | 中程度（実装はあるが選定が肝心） |
| メモリ | 通常少ない | tombstone のため肥大化しがち |
| 採用例 | Google Docs、Apache Wave | Yjs、Automerge |

**新規実装では CRDT を推奨**。OT は実装の罠が多く、現代では Yjs / Automerge / 自前 CRDT の方が扱いやすい。

## CRDT の選択肢

| 実装 | 言語 | 特徴 |
|---|---|---|
| **Yjs** | TypeScript（Rust binding: `yrs`） | テキストに最適化。WebRTC 連携が容易 |
| **Automerge** | Rust（公式） | JSON 全般。テキスト性能は改善中 |
| **diamond-types** | Rust | テキスト特化。極めて高速 |
| 自前 | Rust | Rope と統合して効率的（大規模エディタが採用する形） |

コードエディタ用途では：
- 軽量に始めるなら **`yrs`**（Yjs の Rust 実装）
- 性能を突き詰めるなら **`diamond-types`** または自前
- ドキュメント全体（カーソル位置・ペイン構成等）を共有するなら **Automerge**

## アーキテクチャ全体像

```
[ クライアント A ]                  [ Relay Server ]                  [ クライアント B ]
   ├─ Editor (Rope + CRDT)          ├─ WebSocket router              ├─ Editor
   ├─ CRDT operations               ├─ Auth / Access control         ├─ CRDT operations
   └─ Presence (cursor, selection)  └─ Pub/Sub (Redis 等)            └─ Presence
        │                                  ▲                                ▲
        └──── WebSocket / QUIC ────────────┴──────────── WebSocket / QUIC ──┘
```

- 全クライアントは同じ CRDT を保持し、Relay サーバはオペレーションを全員に転送する
- Relay は **永続化責務を持たない**（CRDT の合流性に頼る）か、Snapshot を定期的に保存する
- Auth と権限制御だけはサーバ側で集中管理する

## ローカル操作と Remote 操作の合流

```
[ ユーザー入力 ] ─→ [ Local CRDT op 生成 ] ─→ [ Rope に適用 ]
                              │
                              ▼
                     [ サーバへ送信 ]
                              │
                              ▼
                     [ 他クライアント受信 ]
                              │
                              ▼
                     [ Remote CRDT op 適用 ] ─→ [ Rope 更新 + 再描画 ]
```

CRDT は順序に依存しないので、ネットワーク順序が前後しても最終状態は一致する。
ただし **再描画タイミングでカーソル位置の補正** が必要（他人の挿入で自分のカーソルが押される）。

## Presence（カーソル位置・選択範囲の共有）

Presence は CRDT とは別チャネルで送る。理由：

- カーソル位置は永続化不要・古くなった情報は捨ててよい
- 高頻度更新（マウス移動ごと）なので CRDT に混ぜるとログが肥大化する
- 一時的な障害で消えても問題ない

実装パターン：

```
[ クライアント A ] ─→ [ サーバ ] ─→ [ クライアント B, C, … ]
   Presence: { user, file, anchor, head, color, ts }
```

- `anchor` `head` は **CRDT 上のアンカー ID** を持つ（バイトオフセットだと他人の挿入でズレる）
- `ts` でタイムアウトし、N 秒更新がなければ非表示にする
- 色はユーザー ID から決定的にハッシュして衝突を減らす

## ネットワーク層

| プロトコル | 用途 |
|---|---|
| **WebSocket** | 業界標準・ブラウザ親和性が必要なら |
| **QUIC** | 低レイテンシ・モバイル前提（`quinn` クレート） |
| **gRPC bidi-streaming** | バイナリ・型安全・既存インフラ親和（`tonic`） |
| **WebRTC DataChannel** | P2P・サーバなしも可能（`webrtc-rs`） |

成熟したコードエディタには QUIC を採用する例もある。新規実装では：
- まず WebSocket + JSON で素早く立ち上げ
- 性能要件が出たら MessagePack / Protocol Buffers にバイナリ化
- 最終的に QUIC へ移行

## 認証と権限

- セッション開始時に **JWT or OAuth** で認証
- ルーム単位で `read` / `write` / `admin` の権限を管理
- CRDT のオペレーションには **author_id を必ず付与**（監査ログのため）
- サーバ側で **書き込み権限を持たないユーザーの op を握りつぶす**

## オフライン編集

CRDT は本質的にオフライン耐性を持つ。
クライアントがオフライン時に行った編集はローカル CRDT に蓄積され、再接続時に一括同期される。

注意点：

- **長期間オフライン**: 大量のオペレーションが溜まると合流処理が重くなる。定期的に Snapshot を取る
- **同一行を別ユーザーが大幅編集**: CRDT は崩壊しないが、結果が「両方の挿入が並ぶ」状態になる。UI で diff 表示を提供する
- **時刻同期**: Lamport / Vector Clock を使い、wall clock には依存しない

## アンチパターン

- **バイトオフセットを直接通信**: 他人の挿入でズレる。アンカー ID（CRDT 内の不変識別子）を使う
- **Presence を CRDT に入れる**: ログが肥大化する。別チャネルへ
- **サーバで CRDT を解釈してマージ**: 不要。サーバはルータに徹し、CRDT は端末で合流させる
- **再接続時に全文を再送**: 差分（不足オペレーション）だけ送る
- **WebSocket を平文で運用**: 必ず WSS（TLS）にする

## 性能の指標

- 編集の伝播レイテンシ: 同一地域 **100ms 以内**、跨地域 **300ms 以内**
- 1 ルームの同時編集人数: **10〜50 人** を現実的な上限とする
- CRDT のメモリ: 100MB ファイルで **元の 1.5〜3 倍** に収める（tombstone GC 必須）

## 関連リンク

- Yjs ドキュメント: https://docs.yjs.dev/
- Automerge ドキュメント: https://automerge.org/
- CRDT 概観論文: Shapiro et al., "Conflict-free Replicated Data Types"（2011）
