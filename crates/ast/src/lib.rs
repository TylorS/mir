//! MIR AST - Abstract Syntax Tree definitions
//! 
//! This crate provides the intermediate representation structure for MIR, including:
//! - AST node definitions
//! - JSON serialization support
//! - Source mapping information
//! - IR serialization with version metadata

pub mod ast;
pub mod module;
pub mod namespace;
pub mod expression;
pub mod statement;
pub mod ir_serialization;

pub use ast::*;
pub use module::Module;
pub use namespace::{
    ModuleNamespace, NamespaceResolver, ValueBinding, TypeBinding, 
    Visibility, ResolutionContext, ResolutionResult, NamespaceError
};
pub use expression::Expression;
pub use statement::Statement;
pub use ir_serialization::{
    IRSerializer, IRDeserializer, SerializedIR, IRContent, IRMetadata,
    IRVersion, JsonSerializationOptions, JsonDeserializationOptions,
    TypeDefinition, TypeDefinitionKind, FieldDefinition, FunctionSignature,
    EnumVariant, SourceLocation, DependencyInfo
};