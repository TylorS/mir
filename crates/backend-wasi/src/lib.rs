//! MIR Backend WASI - WebAssembly System Interface support
//! 
//! This crate provides WASI component model compilation, including:
//! - WASI component generation
//! - Interface type generation
//! - NAPI bindings

pub mod codegen;
pub mod component;
pub mod napi;

pub use codegen::WasiCodeGenerator;
pub use component::WasiComponent;
pub use napi::WasiNapiBindings;