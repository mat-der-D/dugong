use crate::tensor::{Tensor, Vector};

use super::field_value::FieldValue;

/// 勾配演算子の出力型をコンパイル時に決定する trait。
///
/// `T: HasGrad` のとき、`grad(T-Field)` の結果は `T::GradOutput-Field` になる。
/// 関連型 `GradOutput` には `FieldValue` バウンドが付いており、
/// 勾配結果がさらなるフィールド演算で使用できることをコンパイル時に保証する。
///
/// # 実装テーブル
///
/// | 入力型 | `GradOutput` |
/// |--------|-------------|
/// | `f64`  | `Vector`    |
/// | `Vector` | `Tensor`  |
///
/// # compile_fail 例
///
/// `SymmTensor` は `HasGrad` を実装しないため、以下はコンパイルエラーになる:
///
/// ```compile_fail
/// use dugong_types::HasGrad;
/// use dugong_types::tensor::SymmTensor;
/// fn check<T: HasGrad>() {}
/// check::<SymmTensor>();
/// ```
pub trait HasGrad {
    /// 勾配演算子の出力型。`FieldValue` を実装していることを保証する。
    type GradOutput: FieldValue;
}

/// スカラー値の勾配はベクトル値になる: `f64 → Vector`
impl HasGrad for f64 {
    type GradOutput = Vector;
}

/// ベクトル値の勾配はテンソル値になる: `Vector → Tensor`
impl HasGrad for Vector {
    type GradOutput = Tensor;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tensor::{Tensor, Vector};

    #[test]
    fn test_f64_grad_output_is_vector() {
        // 型アノテーションで <f64 as HasGrad>::GradOutput が Vector であることを検証
        let _: <f64 as HasGrad>::GradOutput = Vector::new(0.0, 0.0, 0.0);
    }

    #[test]
    fn test_vector_grad_output_is_tensor() {
        // 型アノテーションで <Vector as HasGrad>::GradOutput が Tensor であることを検証
        let _: <Vector as HasGrad>::GradOutput =
            Tensor::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    }
}
