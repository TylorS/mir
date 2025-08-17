//! Scalar type implementations
//! 
//! Provides implementations for all scalar types with their operations.

use serde::{Deserialize, Serialize};

/// Integer types with specific bit widths
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IntegerType {
    I32(i32),
    I64(i64),
    I128(i128),
    U32(u32),
    U64(u64),
    U128(u128),
}

/// Integer operations trait
pub trait IntegerOperations {
    fn add(&self, other: &Self) -> Result<Self, ArithmeticError>
    where
        Self: Sized;
}

/// Floating point types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FloatType {
    F32(f32),
    F64(f64),
}

/// Float operations trait
pub trait FloatOperations {
    fn add(&self, other: &Self) -> Self
    where
        Self: Sized;
}

/// Boolean type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BooleanType(pub bool);

/// Boolean operations trait
pub trait BooleanOperations {
    fn and(&self, other: &Self) -> Self
    where
        Self: Sized;
}

/// String type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StringType {
    pub data: String,
}

/// String operations trait
pub trait StringOperations {
    fn concat(&self, other: &Self) -> Self
    where
        Self: Sized;
}

/// Arithmetic errors
#[derive(Debug, thiserror::Error)]
pub enum ArithmeticError {
    #[error("Overflow")]
    Overflow,
    #[error("Division by zero")]
    DivisionByZero,
}

/// String errors
#[derive(Debug, thiserror::Error)]
pub enum StringError {
    #[error("Invalid UTF-8")]
    InvalidUtf8,
    #[error("Index out of bounds")]
    IndexOutOfBounds,
}