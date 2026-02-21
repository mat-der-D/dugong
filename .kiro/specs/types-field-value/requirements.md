# 要件定義書

## はじめに

`types-field-value` spec は、`dugong-types` クレートに `FieldValue` trait とランク昇降 trait（`HasGrad` / `HasDiv`）を実装する。これらは FVM フィールド演算の基盤となる抽象化であり、フィールド値の加減算・スカラー倍・ゼロ値・マグニチュード計算を型安全に表現し、微分演算子（勾配・発散）の入出力型をコンパイル時に決定する仕組みを提供する。

**前提**: Spec 1-1（`types-tensor`）— `Scalar`（`f64` alias）、`Vector`、`Tensor`、`SymmTensor`、`SphericalTensor` が実装済みであること。

**成果物**: `crates/types/src/traits/` モジュール（`field_value.rs`、`has_grad.rs`、`has_div.rs`）

---

## 要件

### 要件 1: FieldValue trait の定義

**目的**: ライブラリ開発者として、フィールド値として使用可能な型の共通インターフェースを定義したい。これにより、フィールド演算（加減算・スカラー倍・零元・ノルム）を型安全に統一的に扱えるようにするため。

#### 受け入れ基準

1. The `dugong-types` crate shall define a public trait `FieldValue` with supertrait bounds `Copy + Add<Output = Self> + Sub<Output = Self> + Mul<f64, Output = Self> + Neg<Output = Self>`.
2. The `FieldValue` trait shall declare a static method `fn zero() -> Self` that returns the additive identity element for the implementing type.
3. The `FieldValue` trait shall declare an instance method `fn mag(&self) -> f64` that returns the magnitude (Euclidean / Frobenius norm) of the value as a non-negative `f64`.
4. When `zero()` is called, the `FieldValue` trait shall return a value `z` such that `z + x == x` holds for any value `x` of the same type (加法単位元の保証).
5. When `mag()` is called on the zero value, the `FieldValue` trait shall return `0.0`.

---

### 要件 2: 全テンソル型への FieldValue 実装

**目的**: ライブラリ開発者として、既存のすべてのテンソル型（`f64`、`Vector`、`Tensor`、`SymmTensor`、`SphericalTensor`）が `FieldValue` を満たすようにしたい。これにより、フィールドクレートがこれらの型を共通のインターフェースで扱えるようにするため。

#### 受け入れ基準

1. The `dugong-types` crate shall implement `FieldValue` for `f64`, where `zero()` returns `0.0_f64` and `mag()` returns `self.abs()`.
2. The `dugong-types` crate shall implement `FieldValue` for `Vector`, where `zero()` returns `Vector::new(0.0, 0.0, 0.0)` and `mag()` returns `√(x² + y² + z²)`.
3. The `dugong-types` crate shall implement `FieldValue` for `Tensor`, where `zero()` returns the 9-component zero tensor and `mag()` returns the Frobenius norm `√(Σᵢⱼ aᵢⱼ²)` over all 9 components.
4. The `dugong-types` crate shall implement `FieldValue` for `SymmTensor`, where `zero()` returns the 6-component zero tensor and `mag()` returns the Frobenius norm `√(xx² + yy² + zz² + 2·xy² + 2·xz² + 2·yz²)` that accounts for the symmetric off-diagonal components appearing twice.
5. The `dugong-types` crate shall implement `FieldValue` for `SphericalTensor`, where `zero()` returns `SphericalTensor::new(0.0)` and `mag()` returns `√3 · |s|` (the Frobenius norm of the tensor `s·I` in 3D).
6. When `mag()` is called on any zero value produced by `zero()`, the `dugong-types` crate shall return a value less than `1e-14` (数値ゼロの検証).

---

### 要件 3: HasGrad trait の定義とランク昇格実装

**目的**: ライブラリ開発者として、勾配演算子 `grad` の出力型をコンパイル時に決定したい。これにより `grad(ScalarField) → VectorField`、`grad(VectorField) → TensorField` という型の昇格を静的に保証するため。

#### 受け入れ基準

1. The `dugong-types` crate shall define a public trait `HasGrad` with an associated type `type GradOutput: FieldValue`.
2. The `dugong-types` crate shall implement `HasGrad` for `f64` with `GradOutput = Vector`.
3. The `dugong-types` crate shall implement `HasGrad` for `Vector` with `GradOutput = Tensor`.
4. When `HasGrad` is implemented for a type `T`, the `dugong-types` crate shall ensure that `T::GradOutput` also implements `FieldValue`, so the gradient result is valid for further field arithmetic.
5. If a type does not implement `HasGrad`, the `dugong-types` crate shall produce a compile-time error when that type is used in a context requiring `HasGrad` (verifiable via `compile_fail` test).

---

### 要件 4: HasDiv trait の定義とランク降格実装

**目的**: ライブラリ開発者として、発散演算子 `div` の出力型をコンパイル時に決定したい。これにより `div(VectorField) → ScalarField`、`div(TensorField) → VectorField` という型の降格を静的に保証するため。

#### 受け入れ基準

1. The `dugong-types` crate shall define a public trait `HasDiv` with an associated type `type DivOutput: FieldValue`.
2. The `dugong-types` crate shall implement `HasDiv` for `Vector` with `DivOutput = f64`.
3. The `dugong-types` crate shall implement `HasDiv` for `Tensor` with `DivOutput = Vector`.
4. The `dugong-types` crate shall implement `HasDiv` for `SymmTensor` with `DivOutput = Vector`.
5. When `HasDiv` is implemented for a type `T`, the `dugong-types` crate shall ensure that `T::DivOutput` also implements `FieldValue`, so the divergence result is valid for further field arithmetic.
6. If a type does not implement `HasDiv`, the `dugong-types` crate shall produce a compile-time error when that type is used in a context requiring `HasDiv` (verifiable via `compile_fail` test).

---

### 要件 5: モジュール構成・公開 API・コンパイル品質

**目的**: ライブラリ利用者として、`dugong-types` クレートから trait を直感的にインポートでき、コードが高品質であることを確認したい。これにより下流クレート（`fields`、`discretization`）での開発体験を向上させるため。

#### 受け入れ基準

1. The `dugong-types` crate shall organize the traits in a `traits` submodule located at `crates/types/src/traits/`, using separate files for each trait (`field_value.rs`, `has_grad.rs`, `has_div.rs`).
2. The `dugong-types` crate shall re-export `FieldValue`, `HasGrad`, and `HasDiv` from the crate root so that downstream crates can use `use dugong_types::{FieldValue, HasGrad, HasDiv}` directly.
3. The `dugong-types` crate shall provide `///` documentation comments on all public trait items (`FieldValue`, `HasGrad`, `HasDiv` and their methods) describing their semantics and mathematical guarantees.
4. The `dugong-types` crate shall compile without errors under `cargo build`.
5. The `dugong-types` crate shall produce no warnings under `cargo clippy`.
6. The `dugong-types` crate shall pass all unit tests under `cargo test -p dugong-types`, including numerical accuracy tests for `zero()` and `mag()` and type-level correctness tests for `HasGrad`/`HasDiv` associated types.
