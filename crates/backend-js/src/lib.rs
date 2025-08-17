//! MIR Backend JS - JavaScript/TypeScript code generation
//! 
//! This crate provides JavaScript/TypeScript compilation from MIR IR, including:
//! - JavaScript code generation with multiple target versions
//! - TypeScript type definition generation
//! - Source map generation for debugging support
//! - Optimization and minification
//! - OpenTelemetry instrumentation preservation
//! - Unified backend interface for complete compilation

pub mod codegen;
pub mod typescript;
pub mod sourcemap;
pub mod integration;

pub use codegen::{
    JsCodeGenerator, JsTarget, JsGenerationOptions, ModuleFormat,
    JsOutput, JsMetadata, CodegenError
};
pub use typescript::{
    TypeScriptGenerator, TypeScriptOptions, ModuleResolution,
    TypeScriptOutput, TypeScriptMetadata, TypeScriptError
};
pub use sourcemap::{
    SourceMapGenerator, SourceMap, Mapping, SourceLocation,
    SourceMapContext, SourceMapError
};
pub use integration::{
    JsBackend, BackendOptions, BackendOutput, BackendMetadata, BackendError
};