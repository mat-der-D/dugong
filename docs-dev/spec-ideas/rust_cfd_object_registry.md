# Rust CFD objectRegistry 相当の設計

議論を経て確定した設計判断をここに記録する。

---

## 結論：objectRegistry は Rust では単一コンポーネントとして存在しない

OpenFOAM の `objectRegistry` が担う5つの責務は、Rust では既存の言語機能と設計パターンに分散して吸収される。文字列ベースのフィールドレジストリは不要。

---

## OpenFOAM の objectRegistry：構造と責務

### C++ での実装

`objectRegistry` は `HashTable<regIOobject*>` を継承し、型消去されたオブジェクトを名前で格納する。`lookupObject<T>(name)` で `dynamic_cast` により型を復元する。

```
Time  (is-a objectRegistry)
  └── fvMesh  (is-a objectRegistry)
        ├── "p"  → regIOobject*  → [volScalarField]
        ├── "U"  → regIOobject*  → [volVectorField]
        └── "phi" → regIOobject* → [surfaceScalarField]
```

### 5つの責務

| # | 責務 | C++ での実装 |
|---|------|------------|
| 1 | 実行時型選択（設定ファイル → オブジェクト生成） | `runTimeSelectionTable` + `New` |
| 2 | プラグインからのフィールド参照 | `lookupObject<T>(name)` + `dynamic_cast` |
| 3 | I/O 一括処理 | レジストリ走査 + `writeObject` |
| 4 | 鮮度追跡（派生量のキャッシュ無効化） | `eventNo_` カウンタ |
| 5 | 階層構造（Time → Mesh → Fields） | `parent_` チェーンによる上方探索 |

---

## 文字列ルックアップが不要である理由

### OpenFOAM で文字列ルックアップを使う呼び出し元の分析

| 呼び出し元 | 呼び出し数 | 例 |
|-----------|-----------|---|
| 境界条件 | ~24ファイル | `pressureInletVelocityBC` が `"phi"` を参照 |
| Function Objects | 149箇所 | `wallHeatFlux` が `"T"`, `"qr"` を参照 |
| fvModels（ソース項） | 62箇所 | `buoyancyEnergy` が `"g"`, `"U"` を参照 |
| 乱流モデル | 11箇所 | `buoyantKEpsilon` が `"g"` を参照 |
| **ソルバー本体** | **0箇所** | フィールドを直接所有 |

**ソルバー本体は `lookupObject` を一切使っていない。**

### 文字列ルックアップが必要だった理由：dlopen 境界

OpenFOAM では境界条件・乱流モデル等が共有ライブラリ（`.so`）として動的にロードされる。`.so` 側はソルバーの型情報を持たないため、文字列 + `dynamic_cast` でしかフィールドにアクセスできなかった。

```
ソルバー (実行ファイル)          ← コンパイル単位 A
   │  dlopen (ABI 境界)
境界条件・乱流モデル (.so)       ← コンパイル単位 B（ソルバーの型情報なし）
```

### Rust では dlopen を採用しない（決定済み）

[ビルドモデルの決定](./rust_cfd_runtime_selection.md)により、全コードが静的にリンクされる。ソルバーはプラグインの型を知っており、プラグインもソルバーの型にアクセスできる。

```
ソルバー + 境界条件 + 乱流モデル  ← 同一コンパイル単位（LTO でインライン可）
```

したがって、プラグインにはソルバーが **型付き参照を直接注入** できる。文字列による間接参照は不要。

---

## 責務ごとの Rust での実現方法

### 責務1：実行時型選択 → `inventory` + `dyn Trait`（決定済み）

設定ファイルの文字列（例：`"kOmegaSST"`）からオブジェクトを生成する仕組み。文字列が必要な唯一の場面だが、これはフィールドレジストリではなくファクトリパターンの問題。

```rust
inventory::iter::<TurbulenceModelFactory>()
    .find(|f| f.name == name)
    .map(|f| (f.constructor)())
```

詳細は [実行時選択メカニズム](./rust_cfd_runtime_selection.md) を参照。

### 責務2：プラグインからのフィールド参照 → 依存性注入

静的リンクにより、ソルバーがプラグイン構築時に型付き参照を渡せる。設定ファイルのフィールド名はソルバー側で解決する。

```rust
// ソルバー側: BC 構築時に必要な参照を注入
let inlet_bc = PressureInletVelocity::new(
    config,
    &phi,       // 型付き参照。文字列ルックアップ不要。
);

// 壁関数: ソルバーが &mut を明示的に渡す
wall_function.apply(&mut g_field, &k, &epsilon);
```

OpenFOAM では壁関数（`epsilonWallFunction`）が `lookupObjectRef` で他フィールド（`G`）を `&mut` で書き換えるパターンがあったが、Rust ではソルバーが所有権フローを制御することで借用規則との衝突を回避する。

### 責務3：I/O 一括処理 → ソルバーが明示的に列挙

フィールドの所有者はソルバー構造体。I/O もソルバーが制御する。

```rust
impl<'mesh> Solver<'mesh> {
    fn write(&self, dir: &Path) -> io::Result<()> {
        self.p.write(dir)?;
        self.u.write(dir)?;
        self.phi.write(dir)?;
        self.turbulence.write(dir)?;
        Ok(())
    }
}
```

フィールド追加時の書き忘れリスクはあるが、テストで検出可能。動的レジストリの複雑さより明示性を優先する。

### 責務4：鮮度追跡 → typestate（決定済み）

OpenFOAM の `eventNo_` ランタイムカウンタに対し、Rust では `Fresh` / `Stale` typestate でコンパイル時に鮮度を強制する。

```rust
// Stale なフィールドを離散化演算に渡すとコンパイルエラー
fn gradient(field: &VolumeField<f64, Fresh>) -> Vec<Vector> { ... }
```

詳細は [メッシュ・フィールド・並列化](./rust_cfd_mesh_field_parallel.md) を参照。

### 責務5：階層構造 → 不要

OpenFOAM の `Time → Mesh → Fields` 階層が解いていた問題：

- **上方探索**（`lookupObject` の `parent_` チェーン）→ 責務2で依存性注入に置き換えたため不要
- **I/O パス構築**（`time/instance/local/name`）→ ソルバーが明示的にパスを渡す
- **`Time` によるライフサイクル管理** → Rust の所有権で自然に実現

---

## まとめ

| # | 責務 | Rust での実現 | 状態 |
|---|------|-------------|------|
| 1 | 実行時型選択 | `inventory` + `dyn Trait` | 既決定 |
| 2 | フィールド参照 | 依存性注入（型付き参照） | 本文書で決定 |
| 3 | I/O 一括処理 | ソルバーが明示列挙 | 本文書で決定 |
| 4 | 鮮度追跡 | typestate (`Fresh`/`Stale`) | 既決定 |
| 5 | 階層構造 | 不要 | 本文書で決定 |

---

## 参考

- [実行時選択メカニズム・ビルドモデル](./rust_cfd_runtime_selection.md)
- [メッシュ・フィールド・並列化](./rust_cfd_mesh_field_parallel.md)
- [テンソル型システム](./rust_cfd_tensor_types.md)
- [OpenFOAM 責務分解の分析](./openfoam_responsibility_decomposition.md)
