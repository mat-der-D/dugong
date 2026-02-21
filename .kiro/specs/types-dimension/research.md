# リサーチ・設計決定ログ — types-dimension

---
**Purpose**: 発見フェーズの調査結果・アーキテクチャ評価・設計根拠を記録する。
**Usage**: `design.md` の背景として参照。設計の結論は `design.md` に転記済み。

---

## Summary

- **Feature**: `types-dimension`
- **Discovery Scope**: Extension（既存 `crates/types` クレートへの新モジュール追加）
- **Key Findings**:
  1. 異次元乗除算のコンパイル時型推論には `#![feature(generic_const_exprs)]`（nightly 限定）が必要
  2. 既存の `tensor.rs` + `tensor/` パターンが明確なモジュール構成テンプレートを提供
  3. `FieldValue` スーパートレイトバウンドは `Dim<V, M, L, T>` の演算子実装を通じて完全に充足可能
  4. `tests/` ディレクトリへの `compile_fail` テスト配置には `trybuild` クレートが必要

---

## Research Log

### 1. const generics 算術の安定性調査

- **Context**: 要件 4 の `Dim<V1, M1, L1, T1> * Dim<V2, M2, L2, T2> → Dim<..., {M1+M2}, ...>` を実現するための Rust 言語機能を調査
- **Findings**:
  - Rust 安定版（Edition 2024 / 1.85+）では const generic 位置での算術式 `{ M1 + M2 }` は未安定
  - `#![feature(generic_const_exprs)]` が必要（nightly チャンネル専用）
  - この feature は長期間 nightly に留まっており、健全性の課題から安定化が遅延している
  - 代替案として trait ベースの次元算術があるが、`i8` 範囲の全組み合わせ実装は非現実的
- **Implications**:
  - プロジェクトは nightly toolchain を採用する必要がある
  - `rust-toolchain.toml` を追加してチャンネルを明示固定する
  - 将来的に安定化された際はフィーチャフラグを除去するだけで移行可能

### 2. 既存モジュール構成パターン

- **Context**: `dimension` モジュールの配置方法を既存コードベースから調査
- **Findings**:
  - `tensor.rs`（エントリポイント）+ `tensor/`（サブモジュール群）パターンを確認
  - `tensor/` 内のファイル構成: `types.rs`, `ops.rs`, `cross_ops.rs`, `convert.rs`, `special.rs`, `tests.rs`
  - `traits.rs` + `traits/` パターンも同様のスタイルで確認
  - `mod.rs` は使用されていない（Rust 2018+ スタイル徹底）
- **Implications**:
  - `dimension.rs` + `dimension/` ディレクトリ構成を採用
  - `lib.rs` は `pub mod dimension;` を追加し、`pub use dimension::{...}` で再エクスポート

### 3. FieldValue スーパートレイトバウンドの伝播検証

- **Context**: `impl FieldValue for Dim<V, M, L, T>` の実装可能性を確認
- **Findings**:
  - `FieldValue` スーパートレイト: `Copy + Add<Output=Self> + Sub<Output=Self> + Mul<f64, Output=Self> + Neg<Output=Self>`
  - `V: FieldValue` ならば `V` は上記すべてを満たす
  - `Dim<V, M, L, T>` の演算子実装（要件 3）により `Dim` 自体も上記を満たせる
  - `HasGrad::GradOutput: FieldValue` バウンドについて: `V::GradOutput: FieldValue`（HasGrad 保証）+ 上記ブランケット impl により充足可能
- **Implications**:
  - 演算子実装を完成させた後に `FieldValue` ブランケット impl を追加する順序が重要
  - `HasGrad` / `HasDiv` の `type GradOutput = Dim<V::GradOutput, M, L, T>` も同様に問題なし

### 4. compile_fail テスト配置方法の検討

- **Context**: 要件 5.3「tests/ ディレクトリに含め」の実現方法を調査
- **Findings**:
  - Rust 統合テストファイル（`tests/*.rs`）は通常 `#[test]` を使用
  - `compile_fail` 属性は doc-test 専用であり、統合テストファイルには使用不可
  - `trybuild` クレート（dtolnay 作）が `tests/` ディレクトリへの compile_fail テスト配置の標準解
  - 既存の `has_grad.rs` は doc-test compile_fail を使用（source 内配置）
  - `trybuild` は安定した広く使われている依存クレート（MIT/Apache-2.0）
- **Implications**:
  - `trybuild` を `dev-dependency` として追加
  - `crates/types/tests/compile_fail_dimension.rs` にテストランナーを配置
  - `crates/types/tests/compile_fail/` ディレクトリに失敗すべき `.rs` ファイルを配置
  - 既存の doc-test compile_fail（has_grad.rs パターン）は dimension 源ファイルにも追加（文書化目的）

---

## Architecture Pattern Evaluation

| オプション | 説明 | 強み | リスク / 制限 | 採否 |
|-----------|------|------|--------------|------|
| `typenum` クレート | 型レベル整数 `P1`, `N1`, `Z0` + `Sum`/`Diff` で次元算術 | stable Rust・nalgebra/uom と同一実績・完全一般的 | 外部依存追加・エラーメッセージが冗長 | ✅ 採用 |
| `generic_const_exprs`（nightly） | const 位置での算術式 `{M1+M2}` を直接使用 | 型推論が自然で読みやすい、完全に一般的 | nightly 必須、健全性問題で安定化遅延中・安定化見込みが低い | ❌ 不採用 |
| trait ベース次元算術 | `DimMul<Rhs>` trait で出力次元を指定 | 安定版で動作 | 全次元組み合わせを手動実装が必要、非現実的 | ❌ 不採用 |
| マクロ生成（限定実装） | よく使う次元組み合わせのみマクロで生成 | 安定版、高速コンパイル | 一般性なし、保守困難 | ❌ 不採用 |

---

## Design Decisions

### Decision: `typenum` クレートによる型レベル次元算術の採用

- **Context**: 要件 4「異次元乗除算で次元指数の算術が型に反映される」の実現。当初 `generic_const_exprs`（nightly）を検討したが、安定化見込みが低く致命的な設計リスクと判断
- **Alternatives Considered**:
  1. `typenum`（stable） — 型レベル整数 `P1`/`N1`/`Z0` と `Sum`/`Diff` で次元算術
  2. `generic_const_exprs`（nightly） — const 算術 `{ M1 + M2 }` を型パラメータに直接使用
  3. Dim * Dim を今回スコープ外にする — 同次元演算のみ実装し将来に延期
- **Selected Approach**: `typenum = "1"` を regular dependency として追加し、`M: Integer, L: Integer, T: Integer` 型パラメータと `Sum<M1,M2>`/`Diff<M1,M2>` で異次元乗除算を実現
- **Rationale**: `generic_const_exprs` はサウンドネス問題で長期間 nightly に留まっており、安定化見込みが現時点でかなり低い。`typenum` は `nalgebra`・`uom` 等の主要クレートが採用する確立済みパターンで、stable Rust で完全な次元算術が実現できる
- **Trade-offs**: const generic の `1, -1, -2` 記法から `P1, N1, N2` への変更でやや冗長になるが、型エイリアス（`Pressure`、`Velocity` 等）を通じた使用では体験は同等。エラーメッセージが typenum 型名を含んで冗長になる点は許容範囲内
- **Follow-up**: stable toolchain で `cargo build` / `cargo test` が通ることを確認

### Decision: trybuild による compile_fail テスト

- **Context**: 要件 5.3「tests/ ディレクトリへの compile_fail テスト配置」
- **Alternatives Considered**:
  1. `trybuild` — `tests/` ディレクトリで compile_fail テストを実行する専用クレート
  2. doc-test `compile_fail` — source ファイル内の doc コメントに `compile_fail` を記述（has_grad.rs パターン）
- **Selected Approach**: `trybuild` を `dev-dependency` として追加し、`tests/compile_fail_dimension.rs` にテストランナーを配置
- **Rationale**: 要件 5.3 の「tests/ ディレクトリ」という指定に明示的に対応する。doc-test compile_fail も補完的に source ファイルに追加
- **Trade-offs**: `trybuild` dev-dependency 追加。ただし軽量で広く採用されているクレート
- **Follow-up**: `cargo test -p dugong-types` で trybuild テストが実行されることを確認

### Decision: モジュール構成（dimension.rs + dimension/ ディレクトリ）

- **Context**: 新 `dimension` モジュールのファイル配置
- **Selected Approach**: `src/dimension.rs`（エントリ）+ `src/dimension/`（サブモジュール）
- **Rationale**: 既存の `tensor.rs` + `tensor/`、`traits.rs` + `traits/` パターンと完全に整合する。`mod.rs` 不使用（Rust 2018+ スタイル）
- **Trade-offs**: なし（明確に確立されたプロジェクト規約に準拠）

### Decision: FieldValue ブランケット impl の方針

- **Context**: `Dim<V, M, L, T>` が `FieldValue` を実装する条件
- **Selected Approach**: `impl<V: FieldValue, M: Integer, L: Integer, T: Integer> FieldValue for Dim<V, M, L, T>`（ブランケット impl）
- **Rationale**: `V: FieldValue` がすべてのスーパートレイトバウンドを充足する。演算子実装（要件 3）が先行して存在するため、コンパイラがバウンドを解決できる
- **Trade-offs**: なし

---

## Risks & Mitigations

1. **typenum エラーメッセージの冗長性** — `Dim<f64, PInt<UInt<UTerm, B1>>, ...>` のような型名がエラーに現れる。型エイリアスを通じた使用を推奨することで緩和
2. **`typenum::Add` と `std::ops::Add` の命名衝突** — ops.rs で `use` を明示的に分離して管理（例: `use typenum::Add as TAdd`）。設計上の既知の注意点
3. **FieldValue ブランケット impl の孤児ルール** — `Dim` と `FieldValue` は同一クレート内のため孤児ルール問題なし ✓
4. **`Mul<f64>` と `Mul<Dim<...>>` の競合** — `Rhs` 型が異なるため impl 競合なし ✓
5. **typenum の将来性** — typenum は長期メンテナンスされている（v1.x 系）。仮に deprecated になっても `type` エイリアスの付け替えで移行コストは低い

---

## References

- [Rust RFC: generic_const_exprs](https://github.com/rust-lang/rust/issues/76560) — 機能の現状追跡
- [trybuild crate](https://crates.io/crates/trybuild) — compile_fail テスト用ライブラリ
- [Rust Reference: Const generics](https://doc.rust-lang.org/reference/items/generics.html#const-generics) — const generics 仕様
