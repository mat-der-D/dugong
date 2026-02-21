# プロジェクト構造

## 構成方針

**レイヤード＋クレート分離**: 責務ごとにクレートを分離し、依存は上位（基盤）から下位（アプリケーション）への一方向のみ。各クレートは単一の責務を持つ。

## ディレクトリパターン

### ライブラリクレート
**場所**: `crates/<name>/`
**目的**: フレームワークの各レイヤーを担う再利用可能なライブラリ
**パッケージ名**: `dugong-<name>`（例: `dugong-types`, `dugong-mesh`）
**構成**: `src/lib.rs` をエントリポイントとする標準的な Rust ライブラリ構造

### アプリケーション
**場所**: `apps/<name>/`
**目的**: 全クレートを統合するソルバーアプリケーション（実行バイナリ）
**構成**: `src/main.rs` をエントリポイントとする `[[bin]]` クレート

### 設計文書
**場所**: `docs-dev/spec-ideas/`
**目的**: アーキテクチャ決定の記録と設計アイデアのドキュメント
**例**: `rust_cfd_overview.md`, `crate_design.md`

### 仕様管理
**場所**: `.kiro/specs/` および `.kiro/steering/`
**目的**: AI-DLC（AI Development Life Cycle）によるスペック駆動開発の管理

## 命名規約

- **クレート名**: snake_case（例: `types`, `discretization`）
- **パッケージ名**: `dugong-` プレフィックス + クレート名（例: `dugong-types`）
- **ファイル**: Rust 標準の snake_case（`lib.rs`, `main.rs`）
- **型名**: PascalCase（例: `VolumeField`, `FvMesh`, `ImplicitOps`）
- **trait 名**: PascalCase（例: `FieldValue`, `HasGrad`, `PhysicalBC`）
- **関数・メソッド**: snake_case（例: `evaluate_boundaries`, `map_internal`）

## 依存方向の原則

```
types（基盤・依存なし）
  ↓
mesh, runtime
  ↓
fields（mesh をライフタイムで借用）
  ↓
discretization
  ↓
solvers, models
  ↓
apps（全クレートを統合）
```

1. 依存は上から下へのみ。循環依存は禁止
2. `types` はどこにも依存しない（プロジェクト全体の基盤）
3. `fields` → `mesh` は `&'mesh FvMesh`（借用）で接続
4. `models` → `runtime` で `inventory` による実行時選択
5. `apps/` 配下のソルバーが全クレートの統合点・依存性注入の起点

## ワークスペース設定パターン

- `[workspace.package]` で共通メタデータ（version, edition, authors, license）を一元管理
- 各クレートは `version.workspace = true` で継承
- リリースプロファイルでは `lto = true`, `codegen-units = 1`, `opt-level = 3`

## 段階的実装順序

types → mesh → fields → discretization → solvers → runtime → models → io → apps/simple-solver の順で段階的に実装。各段階は `.kiro/specs/` のスペック単位と対応。

---
_パターンを記録。ファイルツリーではない。パターンに従う新ファイルは更新不要_
