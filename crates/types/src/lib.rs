//! MIR Types - Core type system definitions
//! 
//! This crate provides the fundamental type system for MIR, including:
//! - Content-addressable hashing
//! - Type descriptors and schemas
//! - Universal value operations
//! - RTTI support

pub mod content_hash;
pub mod type_descriptor;
pub mod value;
pub mod schema;
pub mod type_registry;
pub mod scalar_types;
pub mod composite_types;
pub mod advanced_types;
pub mod function_types;

pub use content_hash::{ContentHash, Hashable};
pub use type_descriptor::{TypeDescriptor, TypeHash};
pub use value::{
    Value, UniversalValueOperations, SerializedValue,
    SerializationMetadata, SerializationError
};
pub use schema::{
    Schema, SchemaRegistry, SchemaVersion, Encoder, Decoder,
    EncodingContext, DecodingContext, CompressionType,
    MigrationFunction, MigrationError, MigrationChain, MigrationStep,
    CompatibilityResult, JsonEncoder, JsonDecoder
};
pub use type_registry::TypeRegistry;
pub use scalar_types::{
    IntegerType, IntegerOperations, FloatType, FloatOperations,
    BooleanType, BooleanOperations, StringType, StringOperations,
    ArithmeticError, StringError
};
pub use composite_types::{
    GCHeader, StructType, StructOperations, ArrayType, ArrayOperations,
    RecordType, RecordOperations, UnionType, UnionOperations,
    StructError, ArrayError, RecordError, UnionError
};
pub use advanced_types::{
    EnumType, EnumDefinition, VariantDefinition, EnumOperations,
    RegexType, RegexFlags, RegexOperations, Match,
    ResourceType, ResourceId, ResourceTypeDefinition, ResourceHandle, 
    ResourceMetadata, ResourceOperations, FinalizerFunction,
    EnumError, RegexError, ResourceError
};
pub use function_types::{
    FunctionType, FunctionSignature, FunctionImplementation, FunctionId,
    CaptureEnvironment, CapturedValue, CaptureMode, CallingConvention,
    FunctionOperations, FunctionMetadata, SourceMap, SourceLocation,
    FunctionError, ClosureType, ClosureOperations,
    ContinuationType, ContinuationId, PromptTag, StackFrame,
    ContinuationOperations, ContinuationError, Prompt, ControlOperators
};