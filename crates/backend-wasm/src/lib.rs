//! MIR Backend WASM - WebAssembly code generation
//! 
//! This crate provides WebAssembly compilation from MIR IR, including:
//! - WASM module generation
//! - NAPI bindings for Node.js
//! - Optimization passes

pub mod codegen;
pub mod napi;
pub mod optimization;

pub use codegen::WasmCodeGenerator;
pub use napi::WasmNapiBindings;