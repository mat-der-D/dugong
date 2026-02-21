# クレート設計 — ワークスペース構成と依存方向

---

## ワークスペース全体像

```
dugong/
├── Cargo.toml              # [workspace]
├── crates/
│   ├── types/              # 次元システム・テンソル型
│   ├── mesh/               # メッシュ構造・トポロジ
│   ├── fields/             # フィールド型・typestate・境界条件
│   ├── discretization/     # fvm/fvc 演算子・FvMatrix
│   ├── solvers/            # 線形ソルバー
│   ├── models/             # 乱流モデル等の物理モデル
│   ├── runtime/            # inventory ベースの実行時選択機構
│   └── io/                 # I/O・設定ファイル読み込み
└── apps/
    └── simple-solver/      # ソルバーアプリケーション（検証用）
```

---

## 各クレートの責務

### `types` — 基底型システム

- `Dim<V, M, L, T>`: 次元付き量（const generics）
- `Quantity` trait: `type Value = V` を公開
- テンソル型: `Scalar`, `Vector`, `Tensor`, `SymmTensor`, `SphericalTensor`
- `FieldValue` trait: `Copy + Add + Sub + Mul<f64> + Neg + zero + mag`
- ランク昇降 trait: `HasGrad`, `HasDiv`
- 異型間演算の impl（約 25 個）

**依存:** なし（外部クレート不要）

### `mesh` — メッシュ構造

- `FvMesh`: セル・フェイス・ポイントのトポロジ
- パッチ定義（境界面のグループ）
- メッシュ読み込み（OpenFOAM フォーマットまたは簡易フォーマット）

**依存:** `types`

### `fields` — フィールドとtypestate

- `VolumeField<'mesh, T, State>`: セル中心フィールド + typestate
- `SurfaceField<'mesh, T>`: 面中心フィールド
- `Fresh` / `Stale` マーカー型
- `evaluate_boundaries()`: `Stale → Fresh` 遷移（MPI 通信含む）
- `BoundaryPatch<T>` enum: `Physical(Box<dyn PhysicalBC<T>>)` / `Processor(ProcessorPatch<T>)`
- `PhysicalBC<T>` trait

**依存:** `types`, `mesh`, `mpi`(rsmpi)

### `discretization` — 離散化演算

- `ImplicitOps`: 陰的離散化演算子（`ddt`, `div`, `laplacian` → `FvMatrix`）
- `ExplicitOps`: 陽的評価演算子（`ddt`, `div`, `grad`, `laplacian`, `curl` → フィールド値）
- `FvMatrix<V>`: 次元なしの離散化行列
- `Schemes`: 数値スキーム設定の保持

**依存:** `types`, `mesh`, `fields`

### `solvers` — 線形ソルバー

- `FvMatrix<V>` を解く線形ソルバー群
- CG, BiCGSTAB, GAMG 等
- `dyn LinearSolver` による実行時選択

**依存:** `types`, `discretization`

### `models` — 物理モデル

- 乱流モデル（`TurbulenceModel` trait + 具象実装）
- `inventory` によるファクトリ登録
- 各モデルは `PhysicalBC<T>` の具象実装を含みうる

**依存:** `types`, `mesh`, `fields`, `discretization`, `runtime`

### `runtime` — 実行時選択機構

- `inventory` を使ったファクトリ登録のユーティリティ
- `create_model(name: &str)` 等の汎用解決関数
- ファクトリ構造体のマクロ（将来）

**依存:** `inventory`

### `io` — 入出力

- 設定ファイル読み込み（辞書システム。フォーマット未決定）
- フィールドの読み書き
- メッシュの読み込み

**依存:** `types`, `mesh`, `fields`, `serde`

---

## 依存方向図

```
                    types
                   ╱  │  ╲
                  ╱   │   ╲
               mesh  runtime  io
                │      │     ╱
                ▼      │    ╱
              fields ──┼───╱
              ╱   ╲    │
             ╱     ╲   │
    discretization  │  │
         │     ╲    │  │
         ▼      ╲   ▼  │
      solvers   models  │
         ╲        │    ╱
          ╲       ▼   ╱
           ╲   apps  ╱
            ╲   │   ╱
             ▼  ▼  ╱
          simple-solver
```

### 依存の原則

1. **依存は上から下へのみ。** 循環依存は禁止
2. **`types` はどこにも依存しない。** プロジェクト全体の基盤
3. **`fields` → `mesh` は借用で接続。** `'mesh` ライフタイムパラメータ
4. **`models` → `runtime` で実行時選択を使用。** `inventory` 経由
5. **`apps/` 配下のソルバーが全クレートを統合する。** 依存性注入の起点

---

## ビルド設定

### LTO（Link-Time Optimization）

全クレートを静的リンクし、LTO を有効にする。`inventory` の分散登録はリンク時に解決される。

```toml
# Cargo.toml (workspace root)
[profile.release]
lto = true
codegen-units = 1
```

### Feature flags（検討中）

| Feature | 用途 | 状態 |
|---------|------|------|
| `mpi` | MPI 並列（rsmpi 依存） | デフォルト有効 |
| `serial` | MPI なしビルド | 未決定（`rust_cfd_open_questions.md`） |

---

## 段階的な実装順序

検証プロジェクトのため、全クレートを同時に構築するのではなく以下の順序で進める：

1. **`types`** — 次元システム・テンソル型・FieldValue trait
2. **`mesh`** — 最小限のメッシュ構造（直交格子）
3. **`fields`** — VolumeField + typestate（シリアル版）
4. **`discretization`** — 基本演算子（ddt, grad, div, laplacian）
5. **`solvers`** — 最小限のソルバー（CG）
6. **`runtime`** — inventory ベースのファクトリ
7. **`models`** — 乱流モデル 1 つ（k-omega SST 等）
8. **`io`** — 設定ファイル・フィールド I/O
9. **`apps/simple-solver`** — SIMPLE アルゴリズムで統合検証

各段階は cc-sdd の spec 単位と対応させる。
