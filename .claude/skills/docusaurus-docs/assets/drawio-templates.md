# drawio XML テンプレート集

drawio ファイルは XML 形式で、以下の構造を持つ。
このテンプレートを組み合わせてワイヤーフレームを組み立てる。

プロジェクト固有の値（アプリ名、メニュー項目等）は既存の drawio ファイルや CLAUDE.md から読み取る。

## 目次

1. [基本構造](#基本構造)
2. [ブラウザウィンドウ枠](#ブラウザウィンドウ枠)
3. [サイドバー付きレイアウト](#サイドバー付きレイアウト)
4. [センタリングレイアウト](#センタリングレイアウト)
5. [UIコンポーネント](#uiコンポーネント)
   - [ボタン](#ボタン)
   - [テキスト入力フィールド](#テキスト入力フィールド)
   - [テーブル](#テーブル)
   - [カード](#カード)
   - [バッジ](#バッジ)
   - [検索バー](#検索バー)
   - [ドラッグ＆ドロップエリア](#ドラッグドロップエリア)
   - [プログレスバー](#プログレスバー)
   - [アラート／メッセージ](#アラートメッセージ)
   - [ページネーション](#ページネーション)
   - [ダイアログ（モーダル）](#ダイアログモーダル)
   - [セクション見出し](#セクション見出し)
   - [セパレータ](#セパレータ)

---

## 基本構造

すべての drawio ファイルはこの外殻で囲む。`dx`, `dy` はビューのオフセット。

```xml
<mxfile host="app.diagrams.net" type="device">
  <diagram name="画面名" id="diagram-1">
    <mxGraphModel dx="1024" dy="768" grid="1" gridSize="10" guides="1" tooltips="1" connect="0" arrows="0" fold="1" page="1" pageScale="1" pageWidth="1400" pageHeight="900" math="0" shadow="0">
      <root>
        <mxCell id="0" />
        <mxCell id="1" parent="0" />
        <!-- ここにコンポーネントを配置 -->
      </root>
    </mxGraphModel>
  </diagram>
</mxfile>
```

`pageHeight` は画面内容の量に応じて調整する（最低 900）。

---

## ブラウザウィンドウ枠

画面全体をブラウザウィンドウ風のフレームで囲む。
`value` にはプロジェクトのアプリ表示名を使う（既存の drawio ファイルや CLAUDE.md から取得）。

```xml
<!-- ブラウザウィンドウ -->
<mxCell id="cell-2" value="{{project.display_name}}" style="shape=mxgraph.mockup.containers.browserWindow;whiteSpace=wrap;html=1;strokeColor=#666666;fillColor=#F5F5F5;align=left;verticalAlign=top;fontSize=12;mainText=;recurseResize=0;" vertex="1" parent="1">
  <mxGeometry x="50" y="20" width="1200" height="800" as="geometry" />
</mxCell>
```

ブラウザ内のコンテンツは `parent="cell-2"` とし、座標はブラウザ枠の左上を原点とする。
ただし、ブラウザウィンドウのタイトルバー分として y=70 からコンテンツを配置する。

---

## サイドバー付きレイアウト

サイドバー付きのレイアウト（管理画面など）。
ヘッダーテキストにはアプリ表示名、メニュー項目は既存の drawio ファイルから読み取る。

```xml
<!-- ヘッダーバー -->
<mxCell id="cell-10" value="" style="rounded=0;whiteSpace=wrap;html=1;fillColor=#333333;strokeColor=none;" vertex="1" parent="cell-2">
  <mxGeometry x="0" y="70" width="1200" height="56" as="geometry" />
</mxCell>
<mxCell id="cell-11" value="{{project.display_name}}" style="text;html=1;align=left;verticalAlign=middle;whiteSpace=wrap;fontSize=16;fontColor=#FFFFFF;fontStyle=1;" vertex="1" parent="cell-2">
  <mxGeometry x="16" y="70" width="200" height="56" as="geometry" />
</mxCell>
<mxCell id="cell-12" value="admin@example.com" style="text;html=1;align=right;verticalAlign=middle;whiteSpace=wrap;fontSize=12;fontColor=#CCCCCC;" vertex="1" parent="cell-2">
  <mxGeometry x="980" y="70" width="200" height="56" as="geometry" />
</mxCell>

<!-- サイドバー -->
<mxCell id="cell-20" value="" style="rounded=0;whiteSpace=wrap;html=1;fillColor=#F0F0F0;strokeColor=#DDDDDD;" vertex="1" parent="cell-2">
  <mxGeometry x="0" y="126" width="220" height="674" as="geometry" />
</mxCell>

<!-- サイドバーメニュー項目 — 既存の drawio ファイルに合わせて生成 -->
<!-- 各項目: y = 136 + (index * 40), height = 40 -->
<mxCell id="cell-21" value="{{menu_items[0].label}}" style="text;html=1;align=left;verticalAlign=middle;whiteSpace=wrap;fontSize=13;fontColor=#333333;spacingLeft=20;" vertex="1" parent="cell-2">
  <mxGeometry x="0" y="136" width="220" height="40" as="geometry" />
</mxCell>
<!-- 以降、menu_items の数だけ繰り返す -->

<!-- メインコンテンツエリア -->
<mxCell id="cell-30" value="" style="rounded=0;whiteSpace=wrap;html=1;fillColor=#FFFFFF;strokeColor=none;" vertex="1" parent="cell-2">
  <mxGeometry x="220" y="126" width="980" height="674" as="geometry" />
</mxCell>
```

メインコンテンツ内の要素は x=240（左余白20px）から配置し、幅は最大940px程度。

### アクティブなメニュー項目の表示

現在表示中の画面に対応するサイドバーメニューには背景色をつける：

```xml
<!-- アクティブ状態のメニュー項目（背景あり） -->
<mxCell id="cell-21-bg" value="" style="rounded=0;whiteSpace=wrap;html=1;fillColor=#E3F2FD;strokeColor=none;" vertex="1" parent="cell-2">
  <mxGeometry x="0" y="136" width="220" height="40" as="geometry" />
</mxCell>
<mxCell id="cell-21" value="ダッシュボード" style="text;html=1;align=left;verticalAlign=middle;whiteSpace=wrap;fontSize=13;fontColor=#1976D2;fontStyle=1;spacingLeft=20;" vertex="1" parent="cell-2">
  <mxGeometry x="0" y="136" width="220" height="40" as="geometry" />
</mxCell>
```

---

## センタリングレイアウト

センタリングレイアウト（ログイン画面、公開画面など）。

```xml
<!-- 背景 -->
<mxCell id="cell-10" value="" style="rounded=0;whiteSpace=wrap;html=1;fillColor=#F5F5F5;strokeColor=none;" vertex="1" parent="cell-2">
  <mxGeometry x="0" y="70" width="1200" height="730" as="geometry" />
</mxCell>

<!-- ヘッダーバー（シンプル） -->
<mxCell id="cell-11" value="" style="rounded=0;whiteSpace=wrap;html=1;fillColor=#FFFFFF;strokeColor=#E0E0E0;shadow=0;" vertex="1" parent="cell-2">
  <mxGeometry x="0" y="70" width="1200" height="56" as="geometry" />
</mxCell>
<mxCell id="cell-12" value="{{project.display_name}}" style="text;html=1;align=center;verticalAlign=middle;whiteSpace=wrap;fontSize=16;fontColor=#333333;fontStyle=1;" vertex="1" parent="cell-2">
  <mxGeometry x="0" y="70" width="1200" height="56" as="geometry" />
</mxCell>

<!-- メインカード（中央配置） -->
<mxCell id="cell-20" value="" style="rounded=8;whiteSpace=wrap;html=1;fillColor=#FFFFFF;strokeColor=#E0E0E0;shadow=1;" vertex="1" parent="cell-2">
  <mxGeometry x="360" y="200" width="480" height="400" as="geometry" />
</mxCell>
```

カード内の要素は x=380, 幅440px で配置する。

---

## UIコンポーネント

以下のテンプレートを組み合わせて各画面を構成する。
座標（x, y）と parent は画面に応じて調整する。

### ボタン

```xml
<!-- プライマリボタン（青） -->
<mxCell id="cell-btn-1" value="送信" style="rounded=4;whiteSpace=wrap;html=1;fillColor=#4A90D9;strokeColor=#3A7BC8;fontColor=#FFFFFF;fontSize=13;fontStyle=1;" vertex="1" parent="cell-2">
  <mxGeometry x="400" y="500" width="120" height="36" as="geometry" />
</mxCell>

<!-- セカンダリボタン（グレー） -->
<mxCell id="cell-btn-2" value="キャンセル" style="rounded=4;whiteSpace=wrap;html=1;fillColor=#FFFFFF;strokeColor=#CCCCCC;fontColor=#666666;fontSize=13;" vertex="1" parent="cell-2">
  <mxGeometry x="530" y="500" width="120" height="36" as="geometry" />
</mxCell>

<!-- 危険ボタン（赤） -->
<mxCell id="cell-btn-3" value="削除" style="rounded=4;whiteSpace=wrap;html=1;fillColor=#E74C3C;strokeColor=#C0392B;fontColor=#FFFFFF;fontSize=13;fontStyle=1;" vertex="1" parent="cell-2">
  <mxGeometry x="660" y="500" width="120" height="36" as="geometry" />
</mxCell>
```

### テキスト入力フィールド

```xml
<!-- ラベル + 入力フィールド -->
<mxCell id="cell-label-1" value="メールアドレス" style="text;html=1;align=left;verticalAlign=bottom;whiteSpace=wrap;fontSize=12;fontColor=#555555;" vertex="1" parent="cell-2">
  <mxGeometry x="400" y="200" width="200" height="20" as="geometry" />
</mxCell>
<mxCell id="cell-input-1" value="user@example.com" style="rounded=4;whiteSpace=wrap;html=1;fillColor=#FFFFFF;strokeColor=#CCCCCC;fontColor=#999999;fontSize=12;align=left;spacingLeft=10;" vertex="1" parent="cell-2">
  <mxGeometry x="400" y="224" width="400" height="36" as="geometry" />
</mxCell>
```

### テーブル

```xml
<!-- テーブルヘッダー背景 -->
<mxCell id="cell-th-bg" value="" style="rounded=0;whiteSpace=wrap;html=1;fillColor=#F5F5F5;strokeColor=#E0E0E0;" vertex="1" parent="cell-2">
  <mxGeometry x="240" y="200" width="940" height="40" as="geometry" />
</mxCell>

<!-- ヘッダーセル（各カラム） -->
<mxCell id="cell-th-1" value="名前" style="text;html=1;align=left;verticalAlign=middle;whiteSpace=wrap;fontSize=12;fontColor=#555555;fontStyle=1;spacingLeft=10;" vertex="1" parent="cell-2">
  <mxGeometry x="240" y="200" width="250" height="40" as="geometry" />
</mxCell>
<mxCell id="cell-th-2" value="ステータス" style="text;html=1;align=left;verticalAlign=middle;whiteSpace=wrap;fontSize=12;fontColor=#555555;fontStyle=1;spacingLeft=10;" vertex="1" parent="cell-2">
  <mxGeometry x="490" y="200" width="150" height="40" as="geometry" />
</mxCell>

<!-- データ行 -->
<mxCell id="cell-tr1-bg" value="" style="rounded=0;whiteSpace=wrap;html=1;fillColor=#FFFFFF;strokeColor=#E0E0E0;" vertex="1" parent="cell-2">
  <mxGeometry x="240" y="240" width="940" height="36" as="geometry" />
</mxCell>
<mxCell id="cell-td-1-1" value="サンプルデータ" style="text;html=1;align=left;verticalAlign=middle;whiteSpace=wrap;fontSize=12;fontColor=#333333;spacingLeft=10;" vertex="1" parent="cell-2">
  <mxGeometry x="240" y="240" width="250" height="36" as="geometry" />
</mxCell>

<!-- 2行目（薄いグレー背景でゼブラストライプ） -->
<mxCell id="cell-tr2-bg" value="" style="rounded=0;whiteSpace=wrap;html=1;fillColor=#FAFAFA;strokeColor=#E0E0E0;" vertex="1" parent="cell-2">
  <mxGeometry x="240" y="276" width="940" height="36" as="geometry" />
</mxCell>
```

### カード

```xml
<!-- 統計カード -->
<mxCell id="cell-card-1" value="" style="rounded=8;whiteSpace=wrap;html=1;fillColor=#FFFFFF;strokeColor=#E0E0E0;shadow=1;" vertex="1" parent="cell-2">
  <mxGeometry x="240" y="150" width="220" height="100" as="geometry" />
</mxCell>
<mxCell id="cell-card-1-title" value="タイトル" style="text;html=1;align=left;verticalAlign=top;whiteSpace=wrap;fontSize=12;fontColor=#888888;spacingLeft=16;spacingTop=12;" vertex="1" parent="cell-2">
  <mxGeometry x="240" y="150" width="220" height="30" as="geometry" />
</mxCell>
<mxCell id="cell-card-1-value" value="24" style="text;html=1;align=left;verticalAlign=middle;whiteSpace=wrap;fontSize=28;fontColor=#333333;fontStyle=1;spacingLeft=16;" vertex="1" parent="cell-2">
  <mxGeometry x="240" y="180" width="220" height="50" as="geometry" />
</mxCell>
```

### バッジ

```xml
<!-- 成功バッジ（緑） -->
<mxCell id="cell-badge-1" value="active" style="rounded=10;whiteSpace=wrap;html=1;fillColor=#E8F5E9;strokeColor=none;fontColor=#2E7D32;fontSize=11;" vertex="1" parent="cell-2">
  <mxGeometry x="500" y="248" width="60" height="22" as="geometry" />
</mxCell>

<!-- 無効バッジ（グレー） -->
<mxCell id="cell-badge-2" value="disabled" style="rounded=10;whiteSpace=wrap;html=1;fillColor=#F5F5F5;strokeColor=none;fontColor=#757575;fontSize=11;" vertex="1" parent="cell-2">
  <mxGeometry x="500" y="286" width="70" height="22" as="geometry" />
</mxCell>

<!-- 警告バッジ（赤） -->
<mxCell id="cell-badge-3" value="expired" style="rounded=10;whiteSpace=wrap;html=1;fillColor=#FFEBEE;strokeColor=none;fontColor=#C62828;fontSize=11;" vertex="1" parent="cell-2">
  <mxGeometry x="500" y="324" width="65" height="22" as="geometry" />
</mxCell>
```

### 検索バー

```xml
<mxCell id="cell-search" value="🔍 検索..." style="rounded=20;whiteSpace=wrap;html=1;fillColor=#FFFFFF;strokeColor=#CCCCCC;fontColor=#999999;fontSize=12;align=left;spacingLeft=14;" vertex="1" parent="cell-2">
  <mxGeometry x="240" y="150" width="400" height="36" as="geometry" />
</mxCell>
```

### ドラッグ＆ドロップエリア

```xml
<mxCell id="cell-dnd" value="" style="rounded=8;whiteSpace=wrap;html=1;fillColor=#FAFAFA;strokeColor=#CCCCCC;dashed=1;dashPattern=8 4;" vertex="1" parent="cell-2">
  <mxGeometry x="240" y="160" width="940" height="200" as="geometry" />
</mxCell>
<mxCell id="cell-dnd-icon" value="📁" style="text;html=1;align=center;verticalAlign=middle;whiteSpace=wrap;fontSize=36;" vertex="1" parent="cell-2">
  <mxGeometry x="560" y="200" width="80" height="60" as="geometry" />
</mxCell>
<mxCell id="cell-dnd-text" value="ファイルをドラッグ＆ドロップ&lt;br&gt;またはクリックして選択" style="text;html=1;align=center;verticalAlign=top;whiteSpace=wrap;fontSize=13;fontColor=#888888;" vertex="1" parent="cell-2">
  <mxGeometry x="460" y="270" width="280" height="40" as="geometry" />
</mxCell>
```

### プログレスバー

```xml
<mxCell id="cell-progress-bg" value="" style="rounded=4;whiteSpace=wrap;html=1;fillColor=#E0E0E0;strokeColor=none;" vertex="1" parent="cell-2">
  <mxGeometry x="400" y="400" width="200" height="8" as="geometry" />
</mxCell>
<mxCell id="cell-progress-fill" value="" style="rounded=4;whiteSpace=wrap;html=1;fillColor=#4A90D9;strokeColor=none;" vertex="1" parent="cell-2">
  <mxGeometry x="400" y="400" width="120" height="8" as="geometry" />
</mxCell>
<mxCell id="cell-progress-label" value="3 / 5" style="text;html=1;align=right;verticalAlign=middle;whiteSpace=wrap;fontSize=11;fontColor=#888888;" vertex="1" parent="cell-2">
  <mxGeometry x="610" y="393" width="60" height="22" as="geometry" />
</mxCell>
```

### アラート／メッセージ

```xml
<!-- エラーメッセージ（赤） -->
<mxCell id="cell-alert-error" value="⚠ エラーが発生しました" style="rounded=4;whiteSpace=wrap;html=1;fillColor=#FFEBEE;strokeColor=#FFCDD2;fontColor=#C62828;fontSize=12;align=left;spacingLeft=12;" vertex="1" parent="cell-2">
  <mxGeometry x="400" y="460" width="400" height="36" as="geometry" />
</mxCell>

<!-- 警告メッセージ（黄） -->
<mxCell id="cell-alert-warn" value="⚠ 注意が必要です" style="rounded=4;whiteSpace=wrap;html=1;fillColor=#FFF8E1;strokeColor=#FFE082;fontColor=#F57F17;fontSize=12;align=left;spacingLeft=12;" vertex="1" parent="cell-2">
  <mxGeometry x="240" y="300" width="940" height="36" as="geometry" />
</mxCell>

<!-- 成功メッセージ（緑） -->
<mxCell id="cell-alert-success" value="✓ 処理が完了しました" style="rounded=4;whiteSpace=wrap;html=1;fillColor=#E8F5E9;strokeColor=#C8E6C9;fontColor=#2E7D32;fontSize=12;align=left;spacingLeft=12;" vertex="1" parent="cell-2">
  <mxGeometry x="240" y="300" width="940" height="36" as="geometry" />
</mxCell>
```

### ページネーション

```xml
<mxCell id="cell-page-prev" value="&lt;" style="rounded=4;whiteSpace=wrap;html=1;fillColor=#FFFFFF;strokeColor=#CCCCCC;fontColor=#666666;fontSize=12;" vertex="1" parent="cell-2">
  <mxGeometry x="500" y="600" width="32" height="32" as="geometry" />
</mxCell>
<mxCell id="cell-page-1" value="1" style="rounded=4;whiteSpace=wrap;html=1;fillColor=#4A90D9;strokeColor=#3A7BC8;fontColor=#FFFFFF;fontSize=12;fontStyle=1;" vertex="1" parent="cell-2">
  <mxGeometry x="540" y="600" width="32" height="32" as="geometry" />
</mxCell>
<mxCell id="cell-page-2" value="2" style="rounded=4;whiteSpace=wrap;html=1;fillColor=#FFFFFF;strokeColor=#CCCCCC;fontColor=#666666;fontSize=12;" vertex="1" parent="cell-2">
  <mxGeometry x="580" y="600" width="32" height="32" as="geometry" />
</mxCell>
<mxCell id="cell-page-next" value="&gt;" style="rounded=4;whiteSpace=wrap;html=1;fillColor=#FFFFFF;strokeColor=#CCCCCC;fontColor=#666666;fontSize=12;" vertex="1" parent="cell-2">
  <mxGeometry x="620" y="600" width="32" height="32" as="geometry" />
</mxCell>
```

### ダイアログ（モーダル）

```xml
<!-- オーバーレイ背景 -->
<mxCell id="cell-overlay" value="" style="rounded=0;whiteSpace=wrap;html=1;fillColor=#000000;strokeColor=none;opacity=30;" vertex="1" parent="cell-2">
  <mxGeometry x="0" y="70" width="1200" height="730" as="geometry" />
</mxCell>

<!-- ダイアログ本体 -->
<mxCell id="cell-dialog" value="" style="rounded=8;whiteSpace=wrap;html=1;fillColor=#FFFFFF;strokeColor=#E0E0E0;shadow=1;" vertex="1" parent="cell-2">
  <mxGeometry x="350" y="200" width="500" height="300" as="geometry" />
</mxCell>
<mxCell id="cell-dialog-title" value="ダイアログタイトル" style="text;html=1;align=left;verticalAlign=middle;whiteSpace=wrap;fontSize=16;fontColor=#333333;fontStyle=1;spacingLeft=20;" vertex="1" parent="cell-2">
  <mxGeometry x="350" y="200" width="460" height="48" as="geometry" />
</mxCell>
<mxCell id="cell-dialog-close" value="✕" style="text;html=1;align=center;verticalAlign=middle;whiteSpace=wrap;fontSize=16;fontColor=#888888;" vertex="1" parent="cell-2">
  <mxGeometry x="810" y="200" width="40" height="48" as="geometry" />
</mxCell>
<mxCell id="cell-dialog-divider" value="" style="line;html=1;strokeColor=#E0E0E0;" vertex="1" parent="cell-2">
  <mxGeometry x="370" y="248" width="460" height="1" as="geometry" />
</mxCell>
```

### セクション見出し

```xml
<!-- ページタイトル -->
<mxCell id="cell-title" value="ページタイトル" style="text;html=1;align=left;verticalAlign=middle;whiteSpace=wrap;fontSize=20;fontColor=#333333;fontStyle=1;" vertex="1" parent="cell-2">
  <mxGeometry x="240" y="140" width="400" height="40" as="geometry" />
</mxCell>

<!-- セクション見出し -->
<mxCell id="cell-section" value="セクション名" style="text;html=1;align=left;verticalAlign=middle;whiteSpace=wrap;fontSize=15;fontColor=#333333;fontStyle=1;" vertex="1" parent="cell-2">
  <mxGeometry x="240" y="200" width="300" height="30" as="geometry" />
</mxCell>
```

### セパレータ

```xml
<mxCell id="cell-hr" value="" style="line;html=1;strokeColor=#E0E0E0;" vertex="1" parent="cell-2">
  <mxGeometry x="240" y="350" width="940" height="1" as="geometry" />
</mxCell>
```

---

## 組み立てのコツ

1. **ID の命名規則**: `cell-<番号>` で連番にする。セクションごとに番号帯を分けると管理しやすい
   - 2〜9: ブラウザ枠
   - 10〜19: ヘッダー
   - 20〜29: サイドバー
   - 30〜39: メインコンテンツ枠
   - 40〜: 画面固有のコンポーネント（100番台、200番台とセクションで分ける）

2. **Y座標の積み上げ**: コンポーネントを上から順に配置する際、前のコンポーネントの `y + height + margin` で次の y を計算する。一般的なマージンは 16px。

3. **テーブルの構築**: ヘッダー行を最初に配置し、データ行を `height=36` ずつ下にずらす。2〜3行のサンプルデータを入れると雰囲気が出る。

4. **pageHeight の調整**: すべてのコンポーネントを配置した後、最も下にあるコンポーネントの `y + height + 50` を `pageHeight` に設定する。

5. **プロジェクト固有の値**: `{{project.display_name}}` や `{{menu_items[n].label}}` はプレースホルダー表記。実際のファイル生成時には既存の drawio ファイルや CLAUDE.md から読み取った値に置き換える。
