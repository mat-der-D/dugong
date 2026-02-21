use std::ops::{Add, Mul, Neg, Sub};

use crate::tensor::{SphericalTensor, SymmTensor, Tensor, Vector};

/// フィールド値として使用可能な型の共通インターフェース。
///
/// スーパートレイトバウンドとして加算・減算・スカラー倍・符号反転を要求し、
/// 零元（加法単位元）とノルム（Euclidean / Frobenius）を定義する。
/// 静的ディスパッチ専用設計のため `dyn FieldValue` は意図的に非サポート。
pub trait FieldValue:
    Copy + Add<Output = Self> + Sub<Output = Self> + Mul<f64, Output = Self> + Neg<Output = Self>
{
    /// 加法単位元を返す。`Self::zero() + x == x` をすべての `x` について保証する。
    fn zero() -> Self;

    /// Euclidean ノルム（ベクトル）または Frobenius ノルム（テンソル）を返す。
    ///
    /// 常に非負の `f64` を返す。零元に対して `Self::zero().mag() < 1e-14` を保証する。
    fn mag(&self) -> f64;
}

// ===== f64 =====

impl FieldValue for f64 {
    fn zero() -> Self {
        0.0_f64
    }

    fn mag(&self) -> f64 {
        self.abs()
    }
}

// ===== Vector =====

impl FieldValue for Vector {
    fn zero() -> Self {
        Vector::new(0.0, 0.0, 0.0)
    }

    fn mag(&self) -> f64 {
        let a = self.as_array();
        (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
    }
}

// ===== Tensor =====

impl FieldValue for Tensor {
    fn zero() -> Self {
        Tensor::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
    }

    fn mag(&self) -> f64 {
        self.as_array().iter().map(|x| x * x).sum::<f64>().sqrt()
    }
}

// ===== SymmTensor =====

impl FieldValue for SymmTensor {
    fn zero() -> Self {
        SymmTensor::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
    }

    /// フロベニウスノルム。対角外成分（xy, xz, yz）は対称性により 2 倍で計上する。
    ///
    /// `√(xx² + yy² + zz² + 2·xy² + 2·xz² + 2·yz²)`
    fn mag(&self) -> f64 {
        let xx = self.xx();
        let xy = self.xy();
        let xz = self.xz();
        let yy = self.yy();
        let yz = self.yz();
        let zz = self.zz();
        (xx * xx + yy * yy + zz * zz + 2.0 * xy * xy + 2.0 * xz * xz + 2.0 * yz * yz).sqrt()
    }
}

// ===== SphericalTensor =====

impl FieldValue for SphericalTensor {
    fn zero() -> Self {
        SphericalTensor::new(0.0)
    }

    /// `√3 · |s|`（3D 単位テンソルのスカラー倍 `sI` の Frobenius ノルム）。
    fn mag(&self) -> f64 {
        3.0_f64.sqrt() * self.value().abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f64_zero_mag_is_zero() {
        assert!(f64::zero().mag() < 1e-14);
    }

    #[test]
    fn test_vector_zero_mag_is_zero() {
        assert!(Vector::zero().mag() < 1e-14);
    }

    #[test]
    fn test_tensor_zero_mag_is_zero() {
        assert!(Tensor::zero().mag() < 1e-14);
    }

    #[test]
    fn test_symm_tensor_zero_mag_is_zero() {
        assert!(SymmTensor::zero().mag() < 1e-14);
    }

    #[test]
    fn test_spherical_tensor_zero_mag_is_zero() {
        assert!(SphericalTensor::zero().mag() < 1e-14);
    }

    #[test]
    fn test_vector_mag_known_value() {
        let v = Vector::new(3.0, 4.0, 0.0);
        let expected = 5.0_f64;
        let got = v.mag();
        let rel_err = (got - expected).abs() / expected;
        assert!(rel_err < 1e-14, "relative error {rel_err} >= 1e-14");
    }

    #[test]
    fn test_symm_tensor_frobenius_off_diagonal_doubled() {
        // SymmTensor::new(xx, xy, xz, yy, yz, zz)
        // xy=1, 他はゼロ → mag = √(2·1²) = √2
        let s = SymmTensor::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0);
        let expected = 2.0_f64.sqrt();
        let got = s.mag();
        let diff = (got - expected).abs();
        assert!(diff < 1e-14, "diff {diff} >= 1e-14");
    }

    #[test]
    fn test_spherical_tensor_mag() {
        let s = SphericalTensor::new(1.0);
        let expected = 3.0_f64.sqrt();
        let got = s.mag();
        let diff = (got - expected).abs();
        assert!(diff < 1e-14, "diff {diff} >= 1e-14");
    }
}
