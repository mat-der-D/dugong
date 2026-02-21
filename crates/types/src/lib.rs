//! Fundamental type system for Dugong CFD solver
//!
//! Provides dimension-aware quantities, tensor types, and field value traits.

pub mod tensor;
pub mod traits;

pub use traits::{FieldValue, HasDiv, HasGrad};
