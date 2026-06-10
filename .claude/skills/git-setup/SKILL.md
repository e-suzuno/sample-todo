---
name: git-setup
description: >
  開発タスク開始時の Git 環境セットアップ（Git 環境の検出・フィーチャーブランチの作成・
  GitHub Issue の起票）を一括で実行したいときに使用するスキル。
  「ブランチを切って」「開発環境をセットアップして」「Issue を作って」「タスクを始めるので準備して」
  などタスク着手時の依頼で起動する。コミットメッセージ生成は git-commitmsg スキル、
  PR 作成は git-pull-request スキルに委ねる。
---

# Git セットアップスキル

開発タスク開始時に必要なGit環境のセットアップを行う。
環境検出・ブランチ作成・Issue起票を一貫して実行する。

## 実行する内容

1. Git環境の検出（GitHub有無・デフォルトブランチ・gh CLI）
2. フィーチャーブランチの作成
3. GitHub Issue の起票（または計画ファイルの出力）

## ステップ1: Git環境の検出

以下のコマンドで環境を把握する:

```bash
# Git状態の確認
git status
git remote -v
git branch --show-current

# GitHub リモートの有無
git remote -v | grep -i github

# デフォルトブランチの確認
git remote show origin 2>/dev/null | grep "HEAD branch"

# gh CLI の確認
gh --version 2>/dev/null
```

### 検出結果の整理

| 項目 | 確認方法 | 結果の用途 |
|------|---------|-----------|
| GitHub リモート有無 | `git remote -v` に github を含む | Issue起票・PR作成の可否判断 |
| デフォルトブランチ名 | `git remote show origin` の HEAD branch | ブランチ作成の起点 |
| gh CLI 有無 | `gh --version` | Issue・PR操作の方法選択 |
| 現在のブランチ | `git branch --show-current` | 作業済みブランチがあるか確認 |
| ワーキングツリーの状態 | `git status` | 未コミットの変更がないか確認 |

### 未コミット変更がある場合

ワーキングツリーに未コミットの変更がある場合はユーザーに報告し、判断を仰ぐ:
- ステージングして先にコミットする
- スタッシュして退避する
- 変更を破棄する（ユーザーの明示的な指示がある場合のみ）

## ステップ2: フィーチャーブランチの作成

### ブランチ命名規則

| 種別 | Issue番号あり | Issue番号なし |
|------|-------------|-------------|
| 新機能 | `feature/#<Issue番号>-<短い説明>` | `feature/<短い説明>` |
| バグ修正 | `fix/#<Issue番号>-<短い説明>` | `fix/<短い説明>` |
| リファクタリング | `refactor/#<Issue番号>-<短い説明>` | `refactor/<短い説明>` |

`<短い説明>` はケバブケース（小文字・ハイフン区切り）で簡潔に記述する。

### 作成手順

```bash
# デフォルトブランチを確認
DEFAULT_BRANCH=$(git remote show origin 2>/dev/null | grep "HEAD branch" | sed 's/.*: //')
DEFAULT_BRANCH=${DEFAULT_BRANCH:-main}

# 最新化
git fetch origin
git checkout $DEFAULT_BRANCH
git pull origin $DEFAULT_BRANCH

# フィーチャーブランチ作成
git checkout -b <ブランチ名>
```

### 既にフィーチャーブランチ上にいる場合

`git branch --show-current` の結果が `feature/` や `fix/` で始まる場合は、新規ブランチ作成をスキップしてそのまま使用する。ユーザーに確認して判断する。

## ステップ3: GitHub Issue の起票

### GitHub リモートあり + gh CLI あり の場合

```bash
gh issue create --title "<タスクタイトル>" --body "$(cat <<'EOF'
## 概要
[タスクの概要]

## 実装ステップ
- [ ] ステップ1: ...
- [ ] ステップ2: ...

## テスト戦略
[テスト方針]

## 完了条件
- [ ] テストがすべてグリーン
- [ ] レビューのブロッカーがゼロ
EOF
)"
```

Issue番号を取得し、以降のブランチ名・コミットメッセージに使用する。

Issue作成後にブランチ名を修正する必要がある場合:
```bash
# 現在のブランチ名を変更
git branch -m <新しいブランチ名>
```

### GitHub リモートなし、または gh CLI なし の場合

計画内容をマークダウンファイルとして出力する:

```bash
mkdir -p .plan
```

`.plan/<タスク名>.md` に計画ファイルを作成する。フォーマット:

```markdown
# [タスクタイトル]
**作成日**: [日付]
**ブランチ**: [ブランチ名]

## 概要
[タスクの概要]

## 実装ステップ
- [ ] ステップ1: ...
- [ ] ステップ2: ...

## テスト戦略
[テスト方針]

## 完了条件
- [ ] テストがすべてグリーン
- [ ] レビューのブロッカーがゼロ
```

## セットアップ完了の報告

セットアップ完了後、以下の情報をまとめて報告する:

```
## セットアップ完了

- **ブランチ**: `feature/#12-add-upload-api`
- **GitHub Issue**: #12 (または `.plan/add-upload-api.md`)
- **デフォルトブランチ**: `main`
- **GitHub リモート**: あり / なし
- **gh CLI**: あり / なし
```
