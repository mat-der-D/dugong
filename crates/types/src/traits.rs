/// フィールド演算の基盤となる trait 群。
///
/// - [`FieldValue`][]: フィールド値の統一インターフェース（加減算・スカラー倍・零元・ノルム）
/// - [`HasGrad`][]: 勾配演算子の出力型をコンパイル時に決定する
/// - [`HasDiv`][]: 発散演算子の出力型をコンパイル時に決定する
mod field_value;
mod has_div;
mod has_grad;

pub use field_value::FieldValue;
pub use has_div::HasDiv;
pub use has_grad::HasGrad;
