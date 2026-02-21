# Rust CFD テンソル型システム

## 決定事項

### 型の定義

OpenFOAM の `VectorSpace` 継承階層に対応する newtype を定義する。

```rust
type Scalar = f64;              // newtype にしない。alias のみ。

struct Vector([f64; 3]);        // [x, y, z]
struct Tensor([f64; 9]);        // row-major [xx,xy,xz, yx,yy,yz, zx,zy,zz]
struct SymmTensor([f64; 6]);    // [xx, xy, xz, yy, yz, zz]
struct SphericalTensor(f64);    // scalar * I
```

`diagTensor`（固有値分解の結果等）・2D 型は当面スコープ外。

### 内部表現

- `Tensor`：`[[f64; 3]; 3]` ではなく `[f64; 9]`（row-major）を採用
  - Rust の配列は値型のためどちらもメモリ連続だが、`[f64; 9]` の方が SIMD 向きで明示的

### 異型間演算

全組み合わせを手書きで実装（約 25 impl）。「爆発」ではなく管理可能な量。

| カテゴリ | 代表的な組み合わせ | impl 数 |
|---------|-----------------|--------|
| 加算（異型） | SymmTensor + SphericalTensor → SymmTensor など | ~6 |
| スカラー倍 | f64 × {Vector, Tensor, SymmTensor, SphericalTensor} × 両方向 | 8 |
| 縮約（内積） | Tensor × Vector → Vector など | ~6 |
| 二重縮約 | Tensor : Tensor → f64 など | 3 |

`From<SphericalTensor> for SymmTensor` などの型昇格は定義するが、
SymmTensor の省メモリ性（6成分）を損なう昇格には依存しない。

### 型変換メソッド

ランクや対称性を変える操作は `Tensor` のメソッドとして定義する。
自由関数ではなくメソッドを選ぶ理由：`f64::abs()` などの標準ライブラリの先例に倣い、
メソッドチェーン（左→右の読みやすさ）と IDE 補完による発見性を優先する。

```rust
impl Tensor {
    fn symm(&self) -> SymmTensor { ... }        // (T + Tᵀ) / 2
    fn two_symm(&self) -> SymmTensor { ... }    // T + Tᵀ
    fn sph(&self) -> SphericalTensor { ... }    // trace(T)/3 * I
    fn skew(&self) -> Tensor { ... }            // (T - Tᵀ) / 2
    fn dev(&self) -> Tensor { ... }             // T - sph(T)
    fn trace(&self) -> f64 { ... }              // tr() ではなく完全な名前
    fn det(&self) -> f64 { ... }
    fn transpose(&self) -> Tensor { ... }
}
```

---

## Trait 階層

### 第一層：`FieldValue`（`VolumeField<V>` が `V` に要求する基底 trait）

```rust
trait FieldValue:
    Copy +
    Add<Output = Self> +
    Sub<Output = Self> +
    Mul<f64, Output = Self> +
    Neg<Output = Self>
{
    fn zero() -> Self;
    fn mag(&self) -> f64;   // フロベニウスノルム。収束判定・境界条件で使用。
}
```

`Copy` を要求するのは、全実装型が固定長スタック値だから。

### 第二層：ランク昇降 trait

各 FVM 演算子が追加で要求する。associated type でランクの変化をコンパイル時に表現する。

```rust
trait HasGrad: FieldValue {
    type GradOutput: FieldValue;   // ランクが 1 上
}

trait HasDiv: FieldValue {
    type DivOutput: FieldValue;    // ランクが 1 下
}
```

| 型 | GradOutput | DivOutput |
|----|-----------|----------|
| f64 | Vector | — |
| Vector | Tensor | f64 |
| Tensor | — | Vector |
| SymmTensor | — | Vector |

`HasCurl`（`curl(Vector) → Vector`、3D 固有）は将来追加。Navier-Stokes 基本形では不要。

### `Quantity` trait との接続

`rust_cfd_types_and_notation.md` の `Dim<V, M, L, T>` は `Quantity` trait を実装し、
`type Value = V` を公開する。ここで両システムが接合する：

```
Dim<Vector, 0, 1, -1>  implements  Quantity { type Value = Vector }
                                                        ↓
                                              Vector implements FieldValue
```

`fn ddt<Q: Quantity>(...) -> FvMatrix<Q::Value>` の `Q::Value: FieldValue` 束縛が
次元システムとテンソル型システムの境界となる。

---

## 参考

- [Rust 設計ビジョン（型・記法）](./rust_cfd_types_and_notation.md)
- [OpenFOAM 責務分解の分析](./openfoam_responsibility_decomposition.md)
