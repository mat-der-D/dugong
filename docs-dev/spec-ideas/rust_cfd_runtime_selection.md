# Rust CFD 設計決定録

議論を経て確定した設計判断をここに記録する。

---

## 1. ビルドモデル：全部まとめてビルド（dlopen なし）

OpenFOAM は `controlDict::libs` による dlopen でランタイムプラグインを実現しているが、**このプロジェクトでは採用しない**。

**理由：**
- dlopen 対応には `unsafe` と ABI 管理が必要で、Rust らしくない
- vtable 経由の境界が最適化の壁になる（境界をまたいだインライン展開不可）
- Rust のワークスペース全体 LTO により、モデルの内部実装までインライン展開できる可能性がある
- 「全モデルをコンパイル時に確定させる」制約は CFD の数値計算性能にとってむしろ有利

**OpenFOAM との対比：**
C++ は「動的な仕組みで静的な配線を模倣」していた。Rust 版では「リンカの静的な仕組みで同じ動的な体験を実現」する。

---

## 2. 実行時選択メカニズム：`inventory` crate + `dyn Trait`

設定ファイルの文字列（例：`"kOmegaSST"`）からモデルオブジェクトを生成する仕組みを `inventory` crate で実現する。

```rust
// 登録側（各モデルクレートに記述）
inventory::submit! {
    TurbulenceModelFactory {
        name: "kOmegaSST",
        constructor: || Box::new(KOmegaSST::new()),
    }
}

// 解決側
fn create_model(name: &str) -> Box<dyn TurbulenceModel> {
    inventory::iter::<TurbulenceModelFactory>()
        .find(|f| f.name == name)
        .map(|f| (f.constructor)())
        .unwrap_or_else(|| panic!("unknown model: {name}"))
}
```

**役割分担：**
- `inventory`：リンク時にファクトリを分散登録（安全・`unsafe` 不要）
- `dyn Trait`：返却されるモデルオブジェクトのランタイム多態

**採用しない手法とその理由：**
- enum dispatch：全 variant のコンパイル時列挙が必要なため、`inventory` による拡張性と両立しない
- dlopen + 手動登録：決定 1 の方針により不採用

---

## 参考

- [Rust CFD 型システムと記法](./rust_cfd_types_and_notation.md)
- [OpenFOAM 責務分解の分析](./openfoam_responsibility_decomposition.md)
