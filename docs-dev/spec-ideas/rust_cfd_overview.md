# Rust CFD フレームワーク設計概要

OpenFOAM の設計思想を継承しつつ、Rust の型システムを活かして再構築する CFD フレームワークの設計決定を概観する。各トピックの詳細は個別ファイルを参照。

---

## 設計文書一覧

| ファイル | 主題 | 状態 |
|---------|------|------|
| [rust_cfd_types_and_notation.md](./rust_cfd_types_and_notation.md) | 次元システム・演算子設計・方程式記法 | 確定 |
| [rust_cfd_tensor_types.md](./rust_cfd_tensor_types.md) | テンソル型の定義・trait 階層 | 確定 |
| [rust_cfd_runtime_selection.md](./rust_cfd_runtime_selection.md) | ビルドモデル・実行時型選択 | 確定 |
| [rust_cfd_mesh_field_parallel.md](./rust_cfd_mesh_field_parallel.md) | メッシュ・フィールド・並列化・境界条件 | 確定 |
| [rust_cfd_object_registry.md](./rust_cfd_object_registry.md) | objectRegistry 責務の分散吸収 | 確定 |

---

## 1. 根本方針

- **OpenFOAM の翻訳ではなく re-imagination。** Rust の型システムで数学的正しさを静的に保証する。
- **「コードは数学のように見えるべき」** という OpenFOAM の中核思想は継承する。
- **全コード静的リンク（dlopen なし）。** LTO によるインライン展開・最適化を最大化する。

---

## 2. 次元システムとPDE記法 → [types_and_notation](./rust_cfd_types_and_notation.md)

**次元付き量：** `Dim<V, M, L, T>` で const generics によるコンパイル時次元検査。

```rust
type Pressure = Dim<f64, 1, -1, -2>;      // Pa
type Velocity = Dim<[f64; 3], 0, 1, -1>;  // m/s
```

**fvm/fvc 演算子：** コンテキストオブジェクト方式。スキーム設定を保持する `ImplicitOps` / `ExplicitOps` に演算子メソッドを定義。陰的/陽的の混同を型で防止。

**次元消去の境界：** `fvm` 演算子が次元付きフィールド → 次元なし `FvMatrix` への変換点。行列・ソルバーは物理次元を知らない。

**PDE 記法：** `.rhs()` メソッド＋将来的に `pde!` マクロで `==` 記法を提供。

---

## 3. テンソル型 → [tensor_types](./rust_cfd_tensor_types.md)

**型定義：** `Scalar`（f64 alias）、`Vector([f64; 3])`、`Tensor([f64; 9])`、`SymmTensor([f64; 6])`、`SphericalTensor(f64)`。

**trait 階層：**
- 第一層 `FieldValue`：`VolumeField<V>` が `V` に要求する基底（`Copy + Add + Sub + Mul<f64> + Neg + zero + mag`）
- 第二層 `HasGrad` / `HasDiv`：associated type でランク昇降をコンパイル時表現

**Quantity trait との接合：** `Dim<V, M, L, T>` → `Quantity { type Value = V }` → `V: FieldValue` で次元システムとテンソル型を接続。

---

## 4. ビルドモデルと実行時選択 → [runtime_selection](./rust_cfd_runtime_selection.md)

**ビルドモデル：** 全部まとめてビルド。dlopen 不採用（`unsafe` 回避・LTO 活用）。

**実行時型選択：** `inventory` crate でファクトリを分散登録＋ `dyn Trait` でランタイム多態。設定ファイルの文字列からモデルオブジェクトを生成。

---

## 5. メッシュ・フィールド・並列化・境界条件 → [mesh_field_parallel](./rust_cfd_mesh_field_parallel.md)

**並列化：** rsmpi（MPI）+ rayon ハイブリッド。`Threading::Funneled` で MPI はメインスレッドのみ。

**ライフタイム構造：**
```
mpi::Universe → world → FvMesh → VolumeField<'mesh, T, State>
```
メッシュは `&'mesh FvMesh`（不変参照）。`Arc` 不使用。メッシュは MPI ライフタイムに依存しない。

**typestate パターン：** `Fresh` / `Stale` でフィールド境界条件の鮮度を型レベルで保証。
- `evaluate_boundaries()`: `Stale → Fresh`
- `map_internal()`: `Fresh → Stale`
- 離散化演算は `Fresh` のみ受付 → 未評価フィールドの使用はコンパイルエラー

**境界条件：** `BoundaryPatch<T>` enum で物理BC（`dyn PhysicalBC<T>`）とプロセッサBC（`ProcessorPatch<T>`）を内部区別。フィールドレベルでは統一アクセス（`boundary_values(patch_id)`）。

**通信方式：** 当面ブロッキング。OpenFOAM も実質的に通信と計算の重畳を行っていないため同等。

---

## 6. objectRegistry の代替 → [object_registry](./rust_cfd_object_registry.md)

OpenFOAM の objectRegistry が担う5責務を Rust の言語機能に分散吸収。文字列ベースのフィールドレジストリは不要。

| 責務 | Rust での実現 |
|------|-------------|
| 実行時型選択 | `inventory` + `dyn Trait` |
| プラグインからのフィールド参照 | 依存性注入（型付き参照） |
| I/O 一括処理 | ソルバーが明示列挙 |
| 鮮度追跡 | typestate (`Fresh`/`Stale`) |
| 階層構造 | 不要 |

**文字列ルックアップが不要な理由：** OpenFOAM では dlopen 境界のため `.so` 側がソルバーの型情報を持てず文字列参照が必須だった。静的リンクではソルバーがプラグインに型付き参照を直接注入できる。

---

## 設計間の依存関係

```
types_and_notation ──→ tensor_types
        │                    │
        ▼                    ▼
  runtime_selection    mesh_field_parallel
        │                    │
        └────────┬───────────┘
                 ▼
         object_registry
```

- **types_and_notation** が全体の基盤（次元システム・演算子の形）
- **tensor_types** が値型の具体定義（FieldValue trait で mesh_field_parallel と接合）
- **runtime_selection** のビルドモデル決定が object_registry の「文字列ルックアップ不要」の前提
- **mesh_field_parallel** の typestate が object_registry の鮮度追跡を代替
