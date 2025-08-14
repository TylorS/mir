//! MIR Backend JS - JavaScript/TypeScript code generation
//! 
//! This crate provides JavaScript/TypeScript compilation from MIR IR, including:
//! - JavaScript code generation
//! - TypeScript type definitions
//! - Source map generation

pub mod codegen;
pub mod typescript;
pub mod sourcemap;

pub use codegen::JsCodeGenerator;
pub use typescript::TypeScriptGenerator;