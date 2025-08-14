//! Scalar types with operations for the MIR type system


use serde::{Deserialize, Serialize};
use std::fmt;

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

impl IntegerType {
    /// Get the type name as a string
    pub fn type_name(&self) -> &'static str {
        match self {
            IntegerType::I32(_) => "i32",
            IntegerType::I64(_) => "i64",
            IntegerType::I128(_) => "i128",
            IntegerType::U32(_) => "u32",
            IntegerType::U64(_) => "u64",
            IntegerType::U128(_) => "u128",
        }
    }
    
    /// Convert to i128 for operations (with overflow checking)
    pub fn to_i128(&self) -> Result<i128, ArithmeticError> {
        match self {
            IntegerType::I32(v) => Ok(*v as i128),
            IntegerType::I64(v) => Ok(*v as i128),
            IntegerType::I128(v) => Ok(*v),
            IntegerType::U32(v) => Ok(*v as i128),
            IntegerType::U64(v) => {
                if *v <= i128::MAX as u64 {
                    Ok(*v as i128)
                } else {
                    Err(ArithmeticError::Overflow)
                }
            },
            IntegerType::U128(v) => {
                if *v <= i128::MAX as u128 {
                    Ok(*v as i128)
                } else {
                    Err(ArithmeticError::Overflow)
                }
            },
        }
    }
}

/// Integer operations trait
pub trait IntegerOperations {
    fn add(&self, other: &Self) -> Result<Self, ArithmeticError> where Self: Sized;
    fn subtract(&self, other: &Self) -> Result<Self, ArithmeticError> where Self: Sized;
    fn multiply(&self, other: &Self) -> Result<Self, ArithmeticError> where Self: Sized;
    fn divide(&self, other: &Self) -> Result<Self, ArithmeticError> where Self: Sized;
    fn modulo(&self, other: &Self) -> Result<Self, ArithmeticError> where Self: Sized;
    fn bitwise_and(&self, other: &Self) -> Self where Self: Sized;
    fn bitwise_or(&self, other: &Self) -> Self where Self: Sized;
    fn bitwise_xor(&self, other: &Self) -> Self where Self: Sized;
    fn shift_left(&self, positions: u32) -> Result<Self, ArithmeticError> where Self: Sized;
    fn shift_right(&self, positions: u32) -> Result<Self, ArithmeticError> where Self: Sized;
}

impl IntegerOperations for IntegerType {
    fn add(&self, other: &Self) -> Result<Self, ArithmeticError> {
        match (self, other) {
            (IntegerType::I32(a), IntegerType::I32(b)) => {
                a.checked_add(*b).map(IntegerType::I32).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::I64(a), IntegerType::I64(b)) => {
                a.checked_add(*b).map(IntegerType::I64).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::I128(a), IntegerType::I128(b)) => {
                a.checked_add(*b).map(IntegerType::I128).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::U32(a), IntegerType::U32(b)) => {
                a.checked_add(*b).map(IntegerType::U32).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::U64(a), IntegerType::U64(b)) => {
                a.checked_add(*b).map(IntegerType::U64).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::U128(a), IntegerType::U128(b)) => {
                a.checked_add(*b).map(IntegerType::U128).ok_or(ArithmeticError::Overflow)
            },
            _ => Err(ArithmeticError::TypeMismatch),
        }
    }
    
    fn subtract(&self, other: &Self) -> Result<Self, ArithmeticError> {
        match (self, other) {
            (IntegerType::I32(a), IntegerType::I32(b)) => {
                a.checked_sub(*b).map(IntegerType::I32).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::I64(a), IntegerType::I64(b)) => {
                a.checked_sub(*b).map(IntegerType::I64).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::I128(a), IntegerType::I128(b)) => {
                a.checked_sub(*b).map(IntegerType::I128).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::U32(a), IntegerType::U32(b)) => {
                a.checked_sub(*b).map(IntegerType::U32).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::U64(a), IntegerType::U64(b)) => {
                a.checked_sub(*b).map(IntegerType::U64).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::U128(a), IntegerType::U128(b)) => {
                a.checked_sub(*b).map(IntegerType::U128).ok_or(ArithmeticError::Overflow)
            },
            _ => Err(ArithmeticError::TypeMismatch),
        }
    }
    
    fn multiply(&self, other: &Self) -> Result<Self, ArithmeticError> {
        match (self, other) {
            (IntegerType::I32(a), IntegerType::I32(b)) => {
                a.checked_mul(*b).map(IntegerType::I32).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::I64(a), IntegerType::I64(b)) => {
                a.checked_mul(*b).map(IntegerType::I64).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::I128(a), IntegerType::I128(b)) => {
                a.checked_mul(*b).map(IntegerType::I128).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::U32(a), IntegerType::U32(b)) => {
                a.checked_mul(*b).map(IntegerType::U32).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::U64(a), IntegerType::U64(b)) => {
                a.checked_mul(*b).map(IntegerType::U64).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::U128(a), IntegerType::U128(b)) => {
                a.checked_mul(*b).map(IntegerType::U128).ok_or(ArithmeticError::Overflow)
            },
            _ => Err(ArithmeticError::TypeMismatch),
        }
    }
    
    fn divide(&self, other: &Self) -> Result<Self, ArithmeticError> {
        match (self, other) {
            (IntegerType::I32(a), IntegerType::I32(b)) => {
                if *b == 0 { return Err(ArithmeticError::DivisionByZero); }
                a.checked_div(*b).map(IntegerType::I32).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::I64(a), IntegerType::I64(b)) => {
                if *b == 0 { return Err(ArithmeticError::DivisionByZero); }
                a.checked_div(*b).map(IntegerType::I64).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::I128(a), IntegerType::I128(b)) => {
                if *b == 0 { return Err(ArithmeticError::DivisionByZero); }
                a.checked_div(*b).map(IntegerType::I128).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::U32(a), IntegerType::U32(b)) => {
                if *b == 0 { return Err(ArithmeticError::DivisionByZero); }
                Ok(IntegerType::U32(a / b))
            },
            (IntegerType::U64(a), IntegerType::U64(b)) => {
                if *b == 0 { return Err(ArithmeticError::DivisionByZero); }
                Ok(IntegerType::U64(a / b))
            },
            (IntegerType::U128(a), IntegerType::U128(b)) => {
                if *b == 0 { return Err(ArithmeticError::DivisionByZero); }
                Ok(IntegerType::U128(a / b))
            },
            _ => Err(ArithmeticError::TypeMismatch),
        }
    }
    
    fn modulo(&self, other: &Self) -> Result<Self, ArithmeticError> {
        match (self, other) {
            (IntegerType::I32(a), IntegerType::I32(b)) => {
                if *b == 0 { return Err(ArithmeticError::DivisionByZero); }
                a.checked_rem(*b).map(IntegerType::I32).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::I64(a), IntegerType::I64(b)) => {
                if *b == 0 { return Err(ArithmeticError::DivisionByZero); }
                a.checked_rem(*b).map(IntegerType::I64).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::I128(a), IntegerType::I128(b)) => {
                if *b == 0 { return Err(ArithmeticError::DivisionByZero); }
                a.checked_rem(*b).map(IntegerType::I128).ok_or(ArithmeticError::Overflow)
            },
            (IntegerType::U32(a), IntegerType::U32(b)) => {
                if *b == 0 { return Err(ArithmeticError::DivisionByZero); }
                Ok(IntegerType::U32(a % b))
            },
            (IntegerType::U64(a), IntegerType::U64(b)) => {
                if *b == 0 { return Err(ArithmeticError::DivisionByZero); }
                Ok(IntegerType::U64(a % b))
            },
            (IntegerType::U128(a), IntegerType::U128(b)) => {
                if *b == 0 { return Err(ArithmeticError::DivisionByZero); }
                Ok(IntegerType::U128(a % b))
            },
            _ => Err(ArithmeticError::TypeMismatch),
        }
    }
    
    fn bitwise_and(&self, other: &Self) -> Self {
        match (self, other) {
            (IntegerType::I32(a), IntegerType::I32(b)) => IntegerType::I32(a & b),
            (IntegerType::I64(a), IntegerType::I64(b)) => IntegerType::I64(a & b),
            (IntegerType::I128(a), IntegerType::I128(b)) => IntegerType::I128(a & b),
            (IntegerType::U32(a), IntegerType::U32(b)) => IntegerType::U32(a & b),
            (IntegerType::U64(a), IntegerType::U64(b)) => IntegerType::U64(a & b),
            (IntegerType::U128(a), IntegerType::U128(b)) => IntegerType::U128(a & b),
            _ => panic!("Type mismatch in bitwise_and"),
        }
    }
    
    fn bitwise_or(&self, other: &Self) -> Self {
        match (self, other) {
            (IntegerType::I32(a), IntegerType::I32(b)) => IntegerType::I32(a | b),
            (IntegerType::I64(a), IntegerType::I64(b)) => IntegerType::I64(a | b),
            (IntegerType::I128(a), IntegerType::I128(b)) => IntegerType::I128(a | b),
            (IntegerType::U32(a), IntegerType::U32(b)) => IntegerType::U32(a | b),
            (IntegerType::U64(a), IntegerType::U64(b)) => IntegerType::U64(a | b),
            (IntegerType::U128(a), IntegerType::U128(b)) => IntegerType::U128(a | b),
            _ => panic!("Type mismatch in bitwise_or"),
        }
    }
    
    fn bitwise_xor(&self, other: &Self) -> Self {
        match (self, other) {
            (IntegerType::I32(a), IntegerType::I32(b)) => IntegerType::I32(a ^ b),
            (IntegerType::I64(a), IntegerType::I64(b)) => IntegerType::I64(a ^ b),
            (IntegerType::I128(a), IntegerType::I128(b)) => IntegerType::I128(a ^ b),
            (IntegerType::U32(a), IntegerType::U32(b)) => IntegerType::U32(a ^ b),
            (IntegerType::U64(a), IntegerType::U64(b)) => IntegerType::U64(a ^ b),
            (IntegerType::U128(a), IntegerType::U128(b)) => IntegerType::U128(a ^ b),
            _ => panic!("Type mismatch in bitwise_xor"),
        }
    }
    
    fn shift_left(&self, positions: u32) -> Result<Self, ArithmeticError> {
        match self {
            IntegerType::I32(a) => {
                if positions >= 32 { return Err(ArithmeticError::InvalidShift); }
                a.checked_shl(positions).map(IntegerType::I32).ok_or(ArithmeticError::Overflow)
            },
            IntegerType::I64(a) => {
                if positions >= 64 { return Err(ArithmeticError::InvalidShift); }
                a.checked_shl(positions).map(IntegerType::I64).ok_or(ArithmeticError::Overflow)
            },
            IntegerType::I128(a) => {
                if positions >= 128 { return Err(ArithmeticError::InvalidShift); }
                a.checked_shl(positions).map(IntegerType::I128).ok_or(ArithmeticError::Overflow)
            },
            IntegerType::U32(a) => {
                if positions >= 32 { return Err(ArithmeticError::InvalidShift); }
                a.checked_shl(positions).map(IntegerType::U32).ok_or(ArithmeticError::Overflow)
            },
            IntegerType::U64(a) => {
                if positions >= 64 { return Err(ArithmeticError::InvalidShift); }
                a.checked_shl(positions).map(IntegerType::U64).ok_or(ArithmeticError::Overflow)
            },
            IntegerType::U128(a) => {
                if positions >= 128 { return Err(ArithmeticError::InvalidShift); }
                a.checked_shl(positions).map(IntegerType::U128).ok_or(ArithmeticError::Overflow)
            },
        }
    }
    
    fn shift_right(&self, positions: u32) -> Result<Self, ArithmeticError> {
        match self {
            IntegerType::I32(a) => {
                if positions >= 32 { return Err(ArithmeticError::InvalidShift); }
                a.checked_shr(positions).map(IntegerType::I32).ok_or(ArithmeticError::Overflow)
            },
            IntegerType::I64(a) => {
                if positions >= 64 { return Err(ArithmeticError::InvalidShift); }
                a.checked_shr(positions).map(IntegerType::I64).ok_or(ArithmeticError::Overflow)
            },
            IntegerType::I128(a) => {
                if positions >= 128 { return Err(ArithmeticError::InvalidShift); }
                a.checked_shr(positions).map(IntegerType::I128).ok_or(ArithmeticError::Overflow)
            },
            IntegerType::U32(a) => {
                if positions >= 32 { return Err(ArithmeticError::InvalidShift); }
                a.checked_shr(positions).map(IntegerType::U32).ok_or(ArithmeticError::Overflow)
            },
            IntegerType::U64(a) => {
                if positions >= 64 { return Err(ArithmeticError::InvalidShift); }
                a.checked_shr(positions).map(IntegerType::U64).ok_or(ArithmeticError::Overflow)
            },
            IntegerType::U128(a) => {
                if positions >= 128 { return Err(ArithmeticError::InvalidShift); }
                a.checked_shr(positions).map(IntegerType::U128).ok_or(ArithmeticError::Overflow)
            },
        }
    }
}

/// Floating point types with IEEE compliance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FloatType {
    F32(f32),
    F64(f64),
    // Note: f128 is not yet stable in Rust, so we'll use a placeholder
    F128([u8; 16]), // 128-bit IEEE 754 quad precision placeholder
}

impl FloatType {
    /// Get the type name as a string
    pub fn type_name(&self) -> &'static str {
        match self {
            FloatType::F32(_) => "f32",
            FloatType::F64(_) => "f64",
            FloatType::F128(_) => "f128",
        }
    }
}

/// Floating point operations trait
pub trait FloatOperations {
    fn add(&self, other: &Self) -> Result<Self, ArithmeticError> where Self: Sized;
    fn subtract(&self, other: &Self) -> Result<Self, ArithmeticError> where Self: Sized;
    fn multiply(&self, other: &Self) -> Result<Self, ArithmeticError> where Self: Sized;
    fn divide(&self, other: &Self) -> Result<Self, ArithmeticError> where Self: Sized;
    fn sqrt(&self) -> Result<Self, ArithmeticError> where Self: Sized;
    fn sin(&self) -> Result<Self, ArithmeticError> where Self: Sized;
    fn cos(&self) -> Result<Self, ArithmeticError> where Self: Sized;
    fn is_nan(&self) -> bool;
    fn is_infinite(&self) -> bool;
    fn is_finite(&self) -> bool;
}

impl FloatOperations for FloatType {
    fn add(&self, other: &Self) -> Result<Self, ArithmeticError> {
        match (self, other) {
            (FloatType::F32(a), FloatType::F32(b)) => Ok(FloatType::F32(a + b)),
            (FloatType::F64(a), FloatType::F64(b)) => Ok(FloatType::F64(a + b)),
            (FloatType::F128(_), FloatType::F128(_)) => {
                // Placeholder for f128 operations
                Err(ArithmeticError::UnsupportedOperation("f128 operations not yet implemented".to_string()))
            },
            _ => Err(ArithmeticError::TypeMismatch),
        }
    }
    
    fn subtract(&self, other: &Self) -> Result<Self, ArithmeticError> {
        match (self, other) {
            (FloatType::F32(a), FloatType::F32(b)) => Ok(FloatType::F32(a - b)),
            (FloatType::F64(a), FloatType::F64(b)) => Ok(FloatType::F64(a - b)),
            (FloatType::F128(_), FloatType::F128(_)) => {
                Err(ArithmeticError::UnsupportedOperation("f128 operations not yet implemented".to_string()))
            },
            _ => Err(ArithmeticError::TypeMismatch),
        }
    }
    
    fn multiply(&self, other: &Self) -> Result<Self, ArithmeticError> {
        match (self, other) {
            (FloatType::F32(a), FloatType::F32(b)) => Ok(FloatType::F32(a * b)),
            (FloatType::F64(a), FloatType::F64(b)) => Ok(FloatType::F64(a * b)),
            (FloatType::F128(_), FloatType::F128(_)) => {
                Err(ArithmeticError::UnsupportedOperation("f128 operations not yet implemented".to_string()))
            },
            _ => Err(ArithmeticError::TypeMismatch),
        }
    }
    
    fn divide(&self, other: &Self) -> Result<Self, ArithmeticError> {
        match (self, other) {
            (FloatType::F32(a), FloatType::F32(b)) => {
                if *b == 0.0 { return Err(ArithmeticError::DivisionByZero); }
                Ok(FloatType::F32(a / b))
            },
            (FloatType::F64(a), FloatType::F64(b)) => {
                if *b == 0.0 { return Err(ArithmeticError::DivisionByZero); }
                Ok(FloatType::F64(a / b))
            },
            (FloatType::F128(_), FloatType::F128(_)) => {
                Err(ArithmeticError::UnsupportedOperation("f128 operations not yet implemented".to_string()))
            },
            _ => Err(ArithmeticError::TypeMismatch),
        }
    }
    
    fn sqrt(&self) -> Result<Self, ArithmeticError> {
        match self {
            FloatType::F32(a) => {
                if *a < 0.0 { return Err(ArithmeticError::InvalidOperation("sqrt of negative number".to_string())); }
                Ok(FloatType::F32(a.sqrt()))
            },
            FloatType::F64(a) => {
                if *a < 0.0 { return Err(ArithmeticError::InvalidOperation("sqrt of negative number".to_string())); }
                Ok(FloatType::F64(a.sqrt()))
            },
            FloatType::F128(_) => {
                Err(ArithmeticError::UnsupportedOperation("f128 operations not yet implemented".to_string()))
            },
        }
    }
    
    fn sin(&self) -> Result<Self, ArithmeticError> {
        match self {
            FloatType::F32(a) => Ok(FloatType::F32(a.sin())),
            FloatType::F64(a) => Ok(FloatType::F64(a.sin())),
            FloatType::F128(_) => {
                Err(ArithmeticError::UnsupportedOperation("f128 operations not yet implemented".to_string()))
            },
        }
    }
    
    fn cos(&self) -> Result<Self, ArithmeticError> {
        match self {
            FloatType::F32(a) => Ok(FloatType::F32(a.cos())),
            FloatType::F64(a) => Ok(FloatType::F64(a.cos())),
            FloatType::F128(_) => {
                Err(ArithmeticError::UnsupportedOperation("f128 operations not yet implemented".to_string()))
            },
        }
    }
    
    fn is_nan(&self) -> bool {
        match self {
            FloatType::F32(a) => a.is_nan(),
            FloatType::F64(a) => a.is_nan(),
            FloatType::F128(_) => false, // Placeholder
        }
    }
    
    fn is_infinite(&self) -> bool {
        match self {
            FloatType::F32(a) => a.is_infinite(),
            FloatType::F64(a) => a.is_infinite(),
            FloatType::F128(_) => false, // Placeholder
        }
    }
    
    fn is_finite(&self) -> bool {
        match self {
            FloatType::F32(a) => a.is_finite(),
            FloatType::F64(a) => a.is_finite(),
            FloatType::F128(_) => true, // Placeholder
        }
    }
}

/// Boolean type with logical operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BooleanType(pub bool);

impl BooleanType {
    pub fn new(value: bool) -> Self {
        BooleanType(value)
    }
    
    pub fn value(&self) -> bool {
        self.0
    }
}

/// Boolean operations trait
pub trait BooleanOperations {
    fn and(&self, other: &Self) -> Self;
    fn or(&self, other: &Self) -> Self;
    fn not(&self) -> Self;
    fn xor(&self, other: &Self) -> Self;
}

impl BooleanOperations for BooleanType {
    fn and(&self, other: &Self) -> Self {
        BooleanType(self.0 && other.0)
    }
    
    fn or(&self, other: &Self) -> Self {
        BooleanType(self.0 || other.0)
    }
    
    fn not(&self) -> Self {
        BooleanType(!self.0)
    }
    
    fn xor(&self, other: &Self) -> Self {
        BooleanType(self.0 ^ other.0)
    }
}

/// Unicode-aware string type with grapheme boundary support
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StringType {
    data: String,
    // For now, we'll compute grapheme boundaries on demand
    // In a full implementation, we'd cache them for performance
}

impl StringType {
    pub fn new(data: String) -> Self {
        StringType { data }
    }
    
    pub fn from_str(s: &str) -> Self {
        StringType { data: s.to_string() }
    }
    
    pub fn as_str(&self) -> &str {
        &self.data
    }
    
    pub fn into_string(self) -> String {
        self.data
    }
}

/// String operations trait
pub trait StringOperations {
    fn concat(&self, other: &Self) -> Self;
    fn substring(&self, start: usize, end: usize) -> Result<Self, StringError> where Self: Sized;
    fn length_graphemes(&self) -> usize;
    fn length_bytes(&self) -> usize;
    fn length_chars(&self) -> usize;
    fn to_uppercase(&self) -> Self;
    fn to_lowercase(&self) -> Self;
    fn split(&self, delimiter: &str) -> Vec<Self> where Self: Sized;
    fn contains(&self, pattern: &str) -> bool;
    fn starts_with(&self, prefix: &str) -> bool;
    fn ends_with(&self, suffix: &str) -> bool;
}

impl StringOperations for StringType {
    fn concat(&self, other: &Self) -> Self {
        StringType::new(format!("{}{}", self.data, other.data))
    }
    
    fn substring(&self, start: usize, end: usize) -> Result<Self, StringError> {
        if start > end {
            return Err(StringError::InvalidRange);
        }
        
        let chars: Vec<char> = self.data.chars().collect();
        if end > chars.len() {
            return Err(StringError::IndexOutOfBounds);
        }
        
        let substring: String = chars[start..end].iter().collect();
        Ok(StringType::new(substring))
    }
    
    fn length_graphemes(&self) -> usize {
        // For now, use char count as approximation
        // In a full implementation, we'd use a proper grapheme cluster library
        self.data.chars().count()
    }
    
    fn length_bytes(&self) -> usize {
        self.data.len()
    }
    
    fn length_chars(&self) -> usize {
        self.data.chars().count()
    }
    
    fn to_uppercase(&self) -> Self {
        StringType::new(self.data.to_uppercase())
    }
    
    fn to_lowercase(&self) -> Self {
        StringType::new(self.data.to_lowercase())
    }
    
    fn split(&self, delimiter: &str) -> Vec<Self> {
        self.data.split(delimiter)
            .map(|s| StringType::new(s.to_string()))
            .collect()
    }
    
    fn contains(&self, pattern: &str) -> bool {
        self.data.contains(pattern)
    }
    
    fn starts_with(&self, prefix: &str) -> bool {
        self.data.starts_with(prefix)
    }
    
    fn ends_with(&self, suffix: &str) -> bool {
        self.data.ends_with(suffix)
    }
}

/// Arithmetic error types
#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticError {
    Overflow,
    Underflow,
    DivisionByZero,
    InvalidShift,
    TypeMismatch,
    InvalidOperation(String),
    UnsupportedOperation(String),
}

impl fmt::Display for ArithmeticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArithmeticError::Overflow => write!(f, "Arithmetic overflow"),
            ArithmeticError::Underflow => write!(f, "Arithmetic underflow"),
            ArithmeticError::DivisionByZero => write!(f, "Division by zero"),
            ArithmeticError::InvalidShift => write!(f, "Invalid shift operation"),
            ArithmeticError::TypeMismatch => write!(f, "Type mismatch in operation"),
            ArithmeticError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            ArithmeticError::UnsupportedOperation(msg) => write!(f, "Unsupported operation: {}", msg),
        }
    }
}

impl std::error::Error for ArithmeticError {}

/// String error types
#[derive(Debug, Clone, PartialEq)]
pub enum StringError {
    InvalidRange,
    IndexOutOfBounds,
    InvalidEncoding,
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StringError::InvalidRange => write!(f, "Invalid range for substring operation"),
            StringError::IndexOutOfBounds => write!(f, "Index out of bounds"),
            StringError::InvalidEncoding => write!(f, "Invalid string encoding"),
        }
    }
}

impl std::error::Error for StringError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_integer_operations() {
        let a = IntegerType::I32(10);
        let b = IntegerType::I32(5);
        
        assert_eq!(a.add(&b).unwrap(), IntegerType::I32(15));
        assert_eq!(a.subtract(&b).unwrap(), IntegerType::I32(5));
        assert_eq!(a.multiply(&b).unwrap(), IntegerType::I32(50));
        assert_eq!(a.divide(&b).unwrap(), IntegerType::I32(2));
        assert_eq!(a.modulo(&b).unwrap(), IntegerType::I32(0));
    }
    
    #[test]
    fn test_integer_bitwise_operations() {
        let a = IntegerType::U32(0b1010);
        let b = IntegerType::U32(0b1100);
        
        assert_eq!(a.bitwise_and(&b), IntegerType::U32(0b1000));
        assert_eq!(a.bitwise_or(&b), IntegerType::U32(0b1110));
        assert_eq!(a.bitwise_xor(&b), IntegerType::U32(0b0110));
    }
    
    #[test]
    fn test_float_operations() {
        let a = FloatType::F64(10.0);
        let b = FloatType::F64(3.0);
        
        assert_eq!(a.add(&b).unwrap(), FloatType::F64(13.0));
        assert_eq!(a.subtract(&b).unwrap(), FloatType::F64(7.0));
        assert_eq!(a.multiply(&b).unwrap(), FloatType::F64(30.0));
        
        let result = a.divide(&b).unwrap();
        if let FloatType::F64(val) = result {
            assert!((val - (10.0 / 3.0)).abs() < 1e-10);
        }
    }
    
    #[test]
    fn test_boolean_operations() {
        let a = BooleanType::new(true);
        let b = BooleanType::new(false);
        
        assert_eq!(a.and(&b), BooleanType::new(false));
        assert_eq!(a.or(&b), BooleanType::new(true));
        assert_eq!(a.not(), BooleanType::new(false));
        assert_eq!(a.xor(&b), BooleanType::new(true));
    }
    
    #[test]
    fn test_string_operations() {
        let a = StringType::from_str("Hello");
        let b = StringType::from_str(" World");
        
        assert_eq!(a.concat(&b), StringType::from_str("Hello World"));
        assert_eq!(a.length_chars(), 5);
        assert_eq!(a.to_uppercase(), StringType::from_str("HELLO"));
        assert!(a.contains("ell"));
        assert!(a.starts_with("Hel"));
        assert!(a.ends_with("llo"));
    }
    
    #[test]
    fn test_string_substring() {
        let s = StringType::from_str("Hello World");
        let sub = s.substring(0, 5).unwrap();
        assert_eq!(sub, StringType::from_str("Hello"));
        
        let sub2 = s.substring(6, 11).unwrap();
        assert_eq!(sub2, StringType::from_str("World"));
    }
    
    #[test]
    fn test_arithmetic_errors() {
        let a = IntegerType::I32(i32::MAX);
        let b = IntegerType::I32(1);
        
        assert_eq!(a.add(&b), Err(ArithmeticError::Overflow));
        
        let zero = IntegerType::I32(0);
        assert_eq!(b.divide(&zero), Err(ArithmeticError::DivisionByZero));
    }
}