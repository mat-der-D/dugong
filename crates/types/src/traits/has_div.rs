use crate::tensor::{SymmTensor, Tensor, Vector};

use super::field_value::FieldValue;

/// 発散演算子の出力型をコンパイル時に決定する trait。
///
/// `T: HasDiv` のとき、`div(T-Field)` の結果は `T::DivOutput-Field` になる。
/// 関連型 `DivOutput` には `FieldValue` バウンドが付いており、
/// 発散結果がさらなるフィールド演算で使用できることをコンパイル時に保証する。
///
/// # 実装テーブル
///
/// | 入力型        | `DivOutput` |
/// |--------------|-------------|
/// | `Vector`     | `f64`       |
/// | `Tensor`     | `Vector`    |
/// | `SymmTensor` | `Vector`    |
///
/// # compile_fail 例
///
/// `f64` は `HasDiv` を実装しないため、以下はコンパイルエラーになる:
///
/// ```compile_fail
/// use dugong_types::HasDiv;
/// fn check<T: HasDiv>() {}
/// check::<f64>();
/// ```
pub trait HasDiv {
    /// 発散演算子の出力型。`FieldValue` を実装していることを保証する。
    type DivOutput: FieldValue;
}

/// ベクトル値の発散はスカラー値になる: `Vector → f64`
impl HasDiv for Vector {
    type DivOutput = f64;
}

/// テンソル値の発散はベクトル値になる: `Tensor → Vector`
impl HasDiv for Tensor {
    type DivOutput = Vector;
}

/// 対称テンソル値の発散はベクトル値になる: `SymmTensor → Vector`
impl HasDiv for SymmTensor {
    type DivOutput = Vector;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tensor::{SymmTensor, Tensor, Vector};

    #[test]
    fn test_vector_div_output_is_f64() {
        // 型アノテーションで <Vector as HasDiv>::DivOutput が f64 であることを検証
        let _: <Vector as HasDiv>::DivOutput = 0.0_f64;
    }

    #[test]
    fn test_tensor_div_output_is_vector() {
        // 型アノテーションで <Tensor as HasDiv>::DivOutput が Vector であることを検証
        let _: <Tensor as HasDiv>::DivOutput = Vector::new(0.0, 0.0, 0.0);
    }

    #[test]
    fn test_symm_tensor_div_output_is_vector() {
        // 型アノテーションで <SymmTensor as HasDiv>::DivOutput が Vector であることを検証
        let _: <SymmTensor as HasDiv>::DivOutput = Vector::new(0.0, 0.0, 0.0);
    }
}
