//! MIR Compiler - Core compilation logic
//! 
//! This crate provides the compilation pipeline for MIR, including:
//! - Module linking and dependency resolution
//! - Compile-time optimizations
//! - Change analysis for hot-reloading

pub mod linker;
pub mod optimizer;
pub mod analysis;

pub use linker::ModuleLinker;
pub use optimizer::Optimizer;
pub use analysis::ChangeAnalyzer;