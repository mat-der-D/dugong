# 要件定義書

## プロジェクト説明（入力）

Spec 1-3 `types-dimension`: コンパイル時次元検査システム

`crates/types` に `Dim<V, const M: i8, const L: i8, const T: i8>` 構造体を実装し、
const generics によって物理次元の整合性をコンパイル時に保証する。
前段の Spec 1-1（テンソル型）・Spec 1-2（`FieldValue` trait）が完了済みであることを前提とする。

---

## 要件

### 要件 1: Dim 構造体の定義

**目的**: CFD フレームワークの開発者として、コンパイル時に物理次元が型として表現できることを望む。これにより次元の誤りがランタイムではなくコンパイル時に検出される。

#### 受け入れ基準

1. The `Dim`型 shall `struct Dim<V, const M: i8, const L: i8, const T: i8>` として定義され、内部値を `value: V` フィールドで保持すること。
2. The `Dim`型 shall `new(value: V) -> Self` コンストラクタを提供し、値を包んで次元付き量を生成できること。
3. The `Dim`型 shall `value(&self) -> V`（または `V: Copy` のとき）メソッドを提供し、内部の生の値を取り出せること。
4. The `Dim`型 shall `#[derive(Debug, Clone, Copy, PartialEq)]` を付与し、標準的な Rust トレイトを実装すること（ただし `V` が各トレイトを満たす場合）。
5. The `Dim`型 shall `crates/types/src/dimension/` モジュール以下に実装され、`lib.rs` から `pub use` で再エクスポートされること。

---

### 要件 2: Quantity トレイトの定義と実装

**目的**: CFD フレームワークの開発者として、次元付き量を統一的に扱えるインターフェースを望む。これにより `fvm` 演算子が次元付きフィールドを受け取り次元なし行列を返す境界を明確にできる。

#### 受け入れ基準

1. The `Quantity`型 shall `trait Quantity` として定義され、`type Value` の associated type を持つこと。
2. The `Quantity`型 shall `impl<V, const M: i8, const L: i8, const T: i8> Quantity for Dim<V, M, L, T>` として実装され、`type Value = V` となること。
3. The `types`クレート shall `Quantity` トレイトを `crates/types/src/dimension/` に配置し、`lib.rs` から公開すること。
4. When `Quantity` トレイトを使用するとき、the `types`クレート shall `Quantity::Value: FieldValue` という bound で次元システムとテンソル型システムを接合できること。

---

### 要件 3: 同次元の加算・減算

**目的**: CFD フレームワークの開発者として、同一次元の物理量どうしの加算・減算がコンパイルエラーなく動作することを望む。これにより `Pressure + Pressure` のような物理的に正しい演算が自然に記述できる。

#### 受け入れ基準

1. When 同一次元の `Dim<V, M, L, T>` 同士を加算するとき、the `Dim`型 shall `Add<Output = Dim<V, M, L, T>>` を実装し、内部値の加算結果を同次元型で返すこと。
2. When 同一次元の `Dim<V, M, L, T>` 同士を減算するとき、the `Dim`型 shall `Sub<Output = Dim<V, M, L, T>>` を実装し、内部値の減算結果を同次元型で返すこと。
3. The `Dim`型 shall `Neg<Output = Dim<V, M, L, T>>` を実装し、符号反転を同次元型で返すこと。
4. The `Dim`型 shall `Mul<f64, Output = Dim<V, M, L, T>>` を実装し、無次元スカラーとのスカラー倍を同次元型で返すこと。

---

### 要件 4: 異次元の乗算（次元指数の算術）

**目的**: CFD フレームワークの開発者として、異なる次元の物理量を掛け合わせると正しい結果次元の型が得られることを望む。これにより `Density * Volume → Mass` のような演算が型安全に記述できる。

#### 受け入れ基準

1. When `Dim<V1, M1, L1, T1>` と `Dim<V2, M2, L2, T2>` を乗算するとき、the `Dim`型 shall `Dim<..., {M1+M2}, {L1+L2}, {T1+T2}>` を返すこと（const generics の算術による次元指数の加算）。
2. When `Dim<V1, M1, L1, T1>` を `Dim<V2, M2, L2, T2>` で除算するとき、the `Dim`型 shall `Dim<..., {M1-M2}, {L1-L2}, {T1-T2}>` を返すこと（次元指数の減算）。
3. The `Dim`型 shall `Dim<f64, M, L, T>` の `f64` 乗算・除算において、内部値の乗除算結果と次元指数の算術が一致して計算されること。
4. When `Dim<V, M, L, T>` を無次元スカラー `f64`（= `Dim<f64, 0, 0, 0>` に相当）で乗算するとき、the `Dim`型 shall 結果次元が元と同じ `Dim<..., M, L, T>` になること。

---

### 要件 5: コンパイル時次元不整合の検出

**目的**: CFD フレームワークの開発者として、異なる次元の物理量の加算・減算がコンパイルエラーになることを望む。これにより `Pressure + Velocity` のような物理的に無意味な演算が実行前に検出される。

#### 受け入れ基準

1. If 異次元の `Dim<V, M1, L1, T1>` と `Dim<V, M2, L2, T2>`（次元指数が一つでも異なる）を加算しようとしたとき、the Rust コンパイラ shall コンパイルエラーを発生させること（`compile_fail` テストで検証）。
2. If 異次元の `Dim<V, M1, L1, T1>` と `Dim<V, M2, L2, T2>` を減算しようとしたとき、the Rust コンパイラ shall コンパイルエラーを発生させること（`compile_fail` テストで検証）。
3. The `types`クレート shall `compile_fail` テストを `tests/` ディレクトリに含め、異次元加算・減算がコンパイルエラーになることを仕様として固定すること。

---

### 要件 6: CFD 標準物理量の型エイリアス

**目的**: CFD フレームワークの開発者として、頻繁に使用する物理量に対して意味のある型エイリアスを使えることを望む。これによりコードが `Dim<f64, 1, -1, -2>` ではなく `Pressure` として読めるようになる。

#### 受け入れ基準

1. The `types`クレート shall 以下の型エイリアスを `crates/types/src/dimension/` に定義し公開すること:
   - `type Pressure = Dim<f64, 1, -1, -2>` （Pa = kg·m⁻¹·s⁻²）
   - `type Velocity = Dim<Vector, 0, 1, -1>` （m·s⁻¹）
   - `type Density = Dim<f64, 1, -3, 0>` （kg·m⁻³）
2. The `types`クレート shall 追加として以下の型エイリアスも定義すること:
   - `type DynamicViscosity = Dim<f64, 1, -1, -1>` （Pa·s = kg·m⁻¹·s⁻¹）
   - `type KinematicViscosity = Dim<f64, 0, 2, -1>` （m²·s⁻¹）
   - `type Length = Dim<f64, 0, 1, 0>` （m）
   - `type Time = Dim<f64, 0, 0, 1>` （s）
   - `type Mass = Dim<f64, 1, 0, 0>` （kg）
3. The 型エイリアス shall `use dugong_types::{Pressure, Velocity, Density};` で直接インポートして使用できること。

---

### 要件 7: FieldValue トレイトとの接合

**目的**: CFD フレームワークの開発者として、次元付き型が上位レイヤー（`fields` クレート等）の `FieldValue` bound を通じてフィールド型と統合できることを望む。

#### 受け入れ基準

1. The `Dim`型 shall `V: FieldValue` のとき `FieldValue` を実装し、`zero()` は `Dim::new(V::zero())`、`mag()` は内部値の `mag()` を委譲して返すこと。
2. When `V: FieldValue + HasGrad` のとき、the `Dim`型 shall `HasGrad` を実装し、`type GradOutput = Dim<V::GradOutput, M, L, T>` となること（次元は変わらず値のランクが昇格）。
3. When `V: FieldValue + HasDiv` のとき、the `Dim`型 shall `HasDiv` を実装し、`type DivOutput = Dim<V::DivOutput, M, L, T>` となること。
4. The `Dim`型 shall `FieldValue` の実装により、`VolumeField<Dim<V, M, L, T>, State>` として次元付きフィールドを構築できること（フィールドシステムとの統合が型レベルで成立すること）。

---

### 要件 8: ドキュメントとコード品質

**目的**: CFD フレームワークの開発者として、次元システムの公開 API に十分なドキュメントがあることを望む。これにより他の開発者が型エイリアスや演算の意味を迅速に理解できる。

#### 受け入れ基準

1. The `types`クレート shall `Dim` 構造体・`Quantity` トレイト・すべての公開型エイリアスに対して `///` ドキュメントコメントを付与すること。
2. The `types`クレート shall `cargo clippy` を警告ゼロで通過すること。
3. The `types`クレート shall `cargo build` および `cargo test -p dugong-types` を正常終了すること。
4. The `types`クレート shall 公開 API のドキュメントコメント内に、少なくとも `Pressure`・`Velocity` の使用例を含む `# Examples` セクションを持つこと。
