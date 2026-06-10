#!/bin/bash
# Docusaurus ドキュメントのビルド確認スクリプト
# ドキュメント更新後に実行し、ビルドが通ることを確認する

set -e

# git ルートを起点に絶対パスで検索し、作業ディレクトリに依存しない
SEARCH_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
DOCUSAURUS_CONFIG=$(find "$SEARCH_ROOT" -maxdepth 3 -name 'docusaurus.config.*' -print -quit 2>/dev/null)

if [ -z "$DOCUSAURUS_CONFIG" ]; then
  echo "Error: docusaurus.config.js/.ts not found within $SEARCH_ROOT"
  exit 1
fi

DOCUSAURUS_DIR=$(dirname "$DOCUSAURUS_CONFIG")
echo "Building documentation in $DOCUSAURUS_DIR/ ..."
cd "$DOCUSAURUS_DIR" && npm run build

echo "Build succeeded."
